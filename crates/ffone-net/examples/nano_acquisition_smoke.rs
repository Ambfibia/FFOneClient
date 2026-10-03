//! Mutating local-server acceptance test. Run only against an isolated test database.
//! Endpoint, credentials and Nano/tune IDs are supplied through the environment.
use ffone_net::{GameplayReceiver, LoginSession, ShardSession};
use ffone_protocol::{
    NanoEquipRequest0104, NanoTuneRequest0104, NanoTuneSuccess0104, PcNanoCreateSuccess0104,
    RegisteredGameplayRequest0104, WirePayload, packet,
    wire_0104::{NanoBookSubsetReply0104, NanoEquipSuccess0104, PcGiveNanoRequest0104},
};
use std::{env, error::Error};

fn receive(receiver: &mut GameplayReceiver, expected: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    for _ in 0..512 {
        let frame = receiver.read_next()?;
        if frame.packet_type == expected {
            return Ok(frame.payload);
        }
        if frame.packet_type == packet::P_FE2CL_REP_NANO_TUNE_FAIL {
            return Err("unexpected tuning failure".into());
        }
    }
    Err("expected Nano response did not arrive".into())
}

fn main() -> Result<(), Box<dyn Error>> {
    run(
        env::var("FFONE_TEST_NANO_ID")?.parse()?,
        env::var("FFONE_TEST_TUNE_ID")?.parse()?,
        env::var("FFONE_TEST_SKILL_ID")?.parse()?,
    )
}

pub fn run(nano_id: i16, tune_id: i16, skill_id: i16) -> Result<(), Box<dyn Error>> {
    run_with_observer(nano_id, tune_id, skill_id, &mut ())
}

pub trait AcquisitionObserver {
    fn bootstrap(
        &mut self,
        _load: &ffone_protocol::PcLoadData0104,
        _player_id: i32,
        _bootstrap: &ffone_net::WorldBootstrap,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn created(&mut self, _reply: &PcNanoCreateSuccess0104) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn tuned(&mut self, _reply: &NanoTuneSuccess0104) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn restored(&mut self, _nano_id: i16, _skill_id: i16) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl AcquisitionObserver for () {}

pub fn run_with_observer(
    nano_id: i16,
    tune_id: i16,
    skill_id: i16,
    observer: &mut impl AcquisitionObserver,
) -> Result<(), Box<dyn Error>> {
    if env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref() != Ok("1") {
        return Err("set FFONE_ISOLATED_TEST_DATABASE=1 only for an isolated test server".into());
    }
    let mut login = LoginSession::connect_password(
        &env::var("FFONE_LOGIN_ADDRESS")?,
        &env::var("FFONE_USERNAME")?,
        &env::var("FFONE_PASSWORD")?,
    )?;
    let pc_uid = login.first_character_uid()?;
    let ticket = login.select_character(pc_uid)?;
    let mut shard = ShardSession::connect(ticket)?;
    let player_id = shard.player_id();
    let loaded = shard.complete_loading()?;
    let pages = loaded
        .prelude
        .iter()
        .filter(|frame| frame.packet_type == packet::P_FE2CL_REP_NANO_BOOK_SUBSET)
        .map(|frame| NanoBookSubsetReply0104::decode(&frame.payload))
        .collect::<Result<Vec<_>, _>>()?;
    let size = pages.first().ok_or("missing Nano book")?.book_size;
    assert!(size > i32::from(nano_id));
    for page in &pages {
        assert_eq!(page.pcuid, i64::from(player_id));
        assert_eq!(page.book_size, size);
        for (index, nano) in page.element.iter().enumerate() {
            assert!(nano.id == 0 || i32::from(nano.id) == page.element_offset + index as i32);
        }
    }
    observer.bootstrap(shard.load_data(), player_id, &loaded.into_world_bootstrap())?;
    let (sender, mut receiver) = shard.into_gameplay()?.into_parts();
    sender.send_registered_request(&RegisteredGameplayRequest0104::new(
        packet::P_CL2FE_REQ_PC_GIVE_NANO,
        PcGiveNanoRequest0104 { nano_id }.encode(),
    )?)?;
    let created = PcNanoCreateSuccess0104::decode(&receive(
        &mut receiver,
        packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC,
    )?)?;
    assert_eq!(created.nano.id, nano_id);
    assert_eq!(created.nano.skill_id, 0);
    observer.created(&created)?;
    if let Ok(rejected) = env::var("FFONE_TEST_REJECT_TUNE_ID") {
        sender.send_nano_tune(&NanoTuneRequest0104 {
            nano_id,
            tune_id: rejected.parse()?,
            needed_item_slots: [0; 10],
        })?;
        receive(&mut receiver, packet::P_FE2CL_REP_NANO_TUNE_FAIL)?;
    }
    sender.send_nano_tune(&NanoTuneRequest0104 {
        nano_id,
        tune_id,
        needed_item_slots: [0; 10],
    })?;
    let tuned =
        NanoTuneSuccess0104::decode(&receive(&mut receiver, packet::P_FE2CL_REP_NANO_TUNE_SUCC)?)?;
    assert_eq!(tuned.nano_id, nano_id);
    assert_eq!(tuned.skill_id, skill_id);
    assert_eq!(tuned.fusion_matter, created.fusion_matter);
    assert_eq!(tuned.item_slots, [-1; 10]);
    observer.tuned(&tuned)?;
    sender.send_registered_request(&RegisteredGameplayRequest0104::new(
        packet::P_CL2FE_REQ_NANO_EQUIP,
        NanoEquipRequest0104 {
            nano_id,
            nano_slot: 2,
        }
        .encode(),
    )?)?;
    let equipped = NanoEquipSuccess0104::decode(&receive(
        &mut receiver,
        packet::P_FE2CL_REP_NANO_EQUIP_SUCC,
    )?)?;
    assert_eq!(equipped.nano_id, nano_id);
    assert_eq!(equipped.nano_slot_num, 2);
    sender.send_pc_exit(player_id)?;
    receive(&mut receiver, packet::P_FE2CL_REP_PC_EXIT_SUCC)?;
    drop(sender);
    drop(receiver);
    drop(login);

    // A successful tune reply alone does not prove persistence. Reconnect to
    // the same character and inspect the server's saved Nano book.
    let mut login = LoginSession::connect_password(
        &env::var("FFONE_LOGIN_ADDRESS")?,
        &env::var("FFONE_USERNAME")?,
        &env::var("FFONE_PASSWORD")?,
    )?;
    let mut shard = ShardSession::connect(login.select_character(pc_uid)?)?;
    let loaded = shard.complete_loading()?;
    let mut restored = shard.load_data().nano_bank().get(nano_id as usize).copied();
    for frame in &loaded.prelude {
        if frame.packet_type != packet::P_FE2CL_REP_NANO_BOOK_SUBSET {
            continue;
        }
        let page = NanoBookSubsetReply0104::decode(&frame.payload)?;
        let index = i32::from(nano_id) - page.element_offset;
        if let Ok(index) = usize::try_from(index)
            && let Some(nano) = page.element.get(index)
        {
            restored = Some(ffone_protocol::Nano0104 {
                id: nano.id,
                skill_id: nano.skill_id,
                stamina: nano.stamina,
            });
        }
    }
    let restored = restored.ok_or("Nano missing after relogin")?;
    assert_eq!(restored.id, nano_id, "saved Nano identity after relogin");
    assert_eq!(restored.skill_id, skill_id, "saved ability after relogin");
    observer.bootstrap(shard.load_data(), shard.player_id(), &loaded.into_world_bootstrap())?;
    observer.restored(nano_id, skill_id)?;
    let player_id = shard.player_id();
    let (sender, mut receiver) = shard.into_gameplay()?.into_parts();
    sender.send_pc_exit(player_id)?;
    receive(&mut receiver, packet::P_FE2CL_REP_PC_EXIT_SUCC)?;
    eprintln!(
        "PASS: bank size={size}, pages={}, Nano {nano_id}, skill {skill_id}, free tuning, authoritative equip, saved ability after relogin",
        pages.len()
    );
    Ok(())
}
