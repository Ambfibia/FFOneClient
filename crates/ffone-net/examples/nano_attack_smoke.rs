//! T10. Mutates only an explicitly selected isolated server; requires a GM test character.
use ffone_net::{GameplayReceiver, GameplaySender, LoginSession, ShardSession};
use ffone_protocol::{
    NanoActiveRequest0104, NanoEquipRequest0104, NanoSkillUseRequest0104, NanoSkillUseSuccess0104,
    NanoTuneRequest0104, NanoTuneSuccess0104, NpcEnter0104, RegisteredGameplayRequest0104,
    WirePayload, packet,
    wire_0104::{NpcSummonRequest0104, PcGiveNanoRequest0104},
};
use std::{env, error::Error};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn send(sender: &GameplaySender, id: u32, payload: Vec<u8>) -> Result<()> {
    sender.send_registered_request(&RegisteredGameplayRequest0104::new(id, payload)?)?;
    Ok(())
}

fn receive(receiver: &mut GameplayReceiver, expected: u32) -> Result<Vec<u8>> {
    for _ in 0..512 {
        let frame = receiver.read_next()?;
        if frame.packet_type == expected {
            return Ok(frame.payload);
        }
    }
    Err(format!("missing reply {expected:#x}").into())
}

fn reject(
    sender: &GameplaySender,
    receiver: &mut GameplayReceiver,
    req: &NanoSkillUseRequest0104,
    slot: i16,
) -> Result<()> {
    send(sender, packet::P_CL2FE_REQ_NANO_SKILL_USE, req.encode()?)?;
    // Reaffirming the same Nano is an ordered barrier and must not reset cooldown.
    send(
        sender,
        packet::P_CL2FE_REQ_NANO_ACTIVE,
        NanoActiveRequest0104 { nano_slot: slot }.encode(),
    )?;
    loop {
        let frame = receiver.read_next()?;
        assert_ne!(
            frame.packet_type,
            packet::P_FE2CL_NANO_SKILL_USE_SUCC,
            "rejected cast returned success"
        );
        if frame.packet_type == packet::P_FE2CL_REP_NANO_ACTIVE_SUCC {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    if env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref() != Ok("1") {
        return Err("requires FFONE_ISOLATED_TEST_DATABASE=1 and isolated GM account".into());
    }
    let mut login = LoginSession::connect_password(
        env::var("FFONE_LOGIN_ADDRESS")?,
        &env::var("FFONE_USERNAME")?,
        &env::var("FFONE_PASSWORD")?,
    )?;
    let uid = login.first_character_uid()?;
    let mut shard = ShardSession::connect(login.select_character(uid)?)?;
    let player_id = shard.player_id();
    let bank = shard.load_data().nano_bank();
    shard.complete_loading()?;
    let (sender, mut receiver) = shard.into_gameplay()?.into_parts();
    for (slot, nano, tune, skill, cost, count) in [(0, 7, 19, 19, 26, 1), (1, 6, 21, 21, 39, 3)] {
        let initial_stamina = if bank[nano as usize].id == 0 {
            150
        } else {
            bank[nano as usize].stamina
        };
        assert!(
            initial_stamina >= cost,
            "recharge the isolated fixture before running"
        );
        if bank[nano as usize].id == 0 {
            send(
                &sender,
                packet::P_CL2FE_REQ_PC_GIVE_NANO,
                PcGiveNanoRequest0104 { nano_id: nano }.encode(),
            )?;
            receive(&mut receiver, packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC)?;
            sender.send_nano_tune(&NanoTuneRequest0104 {
                nano_id: nano,
                tune_id: tune,
                needed_item_slots: [-1; 10],
            })?;
            let tuned = NanoTuneSuccess0104::decode(&receive(
                &mut receiver,
                packet::P_FE2CL_REP_NANO_TUNE_SUCC,
            )?)?;
            assert_eq!(tuned.skill_id, skill);
        } else {
            assert_eq!(
                bank[nano as usize].skill_id, skill,
                "prepare the accepted tune before repeating this test"
            );
        }
        send(
            &sender,
            packet::P_CL2FE_REQ_NANO_EQUIP,
            NanoEquipRequest0104 {
                nano_id: nano,
                nano_slot: slot,
            }
            .encode(),
        )?;
        receive(&mut receiver, packet::P_FE2CL_REP_NANO_EQUIP_SUCC)?;
        send(
            &sender,
            packet::P_CL2FE_REQ_NANO_ACTIVE,
            NanoActiveRequest0104 { nano_slot: slot }.encode(),
        )?;
        receive(&mut receiver, packet::P_FE2CL_REP_NANO_ACTIVE_SUCC)?;
        send(
            &sender,
            0x1300_0045,
            NpcSummonRequest0104 {
                npc_type: 59,
                npc_cnt: count,
            }
            .encode(),
        )?;
        let mut ids = Vec::new();
        let mut before = Vec::new();
        while ids.len() < count as usize {
            let npc = NpcEnter0104::decode(&receive(&mut receiver, packet::P_FE2CL_NPC_ENTER)?)?
                .appearance;
            if npc.npc_type == 59 {
                ids.push(npc.npc_id);
                before.push(npc.hp);
            }
        }
        let req = NanoSkillUseRequest0104 {
            bullet_id: -1,
            arg1: ids[0],
            arg2: 0,
            arg3: 0,
            target_ids: ids,
        };
        for rejected in [vec![], vec![i32::MAX], vec![req.arg1, req.arg1]] {
            let mut bad = req.clone();
            bad.target_ids = rejected;
            reject(&sender, &mut receiver, &bad, slot)?;
        }
        println!("request Nano={nano} tune={tune} skill={skill}: {req:?}; HP before={before:?}");
        send(&sender, packet::P_CL2FE_REQ_NANO_SKILL_USE, req.encode()?)?;
        let raw = receive(&mut receiver, packet::P_FE2CL_NANO_SKILL_USE_SUCC)?;
        let reply = NanoSkillUseSuccess0104::decode(&raw)?;
        println!("response: {reply:?}");
        assert_eq!(reply.prefix().skill_id, skill);
        assert_eq!(reply.prefix().target_count, i32::from(count));
        // Result records are Damage: eCT, ID, protected, damage, authoritative HP.
        for (record, hp) in reply.result_bytes().chunks_exact(20).zip(before) {
            let damage = i32::from_le_bytes(record[12..16].try_into()?);
            let after = i32::from_le_bytes(record[16..20].try_into()?);
            assert!(damage > 0);
            assert_eq!(after, hp - damage);
        }
        assert!(
            (initial_stamina - cost - 2..=initial_stamina - cost + 1)
                .contains(&reply.prefix().nano_stamina)
        );
        reject(&sender, &mut receiver, &req, slot)?;
        send(
            &sender,
            packet::P_CL2FE_REQ_NANO_ACTIVE,
            NanoActiveRequest0104 { nano_slot: -1 }.encode(),
        )?;
        receive(&mut receiver, packet::P_FE2CL_REP_NANO_ACTIVE_SUCC)?;
        send(
            &sender,
            packet::P_CL2FE_REQ_NANO_ACTIVE,
            NanoActiveRequest0104 { nano_slot: slot }.encode(),
        )?;
        receive(&mut receiver, packet::P_FE2CL_REP_NANO_ACTIVE_SUCC)?;
        reject(&sender, &mut receiver, &req, slot)?;
        println!("PASS Nano={nano}: real server HP decreased; stamina charged once");
    }
    sender.send_pc_exit(player_id)?;
    receive(&mut receiver, packet::P_FE2CL_REP_PC_EXIT_SUCC)?;
    Ok(())
}
