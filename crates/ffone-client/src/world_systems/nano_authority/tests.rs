use crate::world_nano_authority::*;
use ffone_protocol::{
    NanoSkillDamageResult0104, NanoSkillUseSuccessPrefix0104,
    packet::{P_FE2CL_NANO_SKILL_USE, P_FE2CL_NANO_SKILL_USE_SUCC},
};

fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn payload(skill_type: i32, records: &[Vec<u8>]) -> Vec<u8> {
    let prefix = NanoSkillUseSuccessPrefix0104 {
        pc_id: 77,
        bullet_id: -1,
        pack_padding: [0xa5],
        skill_id: 144,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        nano_deactivated: 0,
        nano_id: 17,
        nano_stamina: 73,
        skill_type,
        target_count: i32::try_from(records.len()).unwrap(),
    };
    let mut bytes = prefix.encode_prefix();
    for record in records {
        bytes.extend_from_slice(record);
    }
    bytes
}

fn target_record(size: usize, entity_type: i32, id: i32) -> Vec<u8> {
    let mut record = vec![0; size];
    write_i32(&mut record, 0, entity_type);
    write_i32(&mut record, 4, id);
    record
}

#[test]
fn local_damage_and_remote_heal_project_absolute_hp_and_delivery() {
    let mut damage = target_record(20, 4, 9001);
    write_i32(&mut damage, 8, 0);
    write_i32(&mut damage, 12, 125);
    write_i32(&mut damage, 16, 875);
    let local =
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE_SUCC, &payload(1, &[damage]))
            .unwrap()
            .unwrap();
    assert_eq!(local.delivery, NanoSkillUseDelivery0104::LocalSuccess);
    assert_eq!(local.caster.pc_id, 77);
    assert_eq!(local.targets.len(), 1);
    assert_eq!(
        local.targets[0].target,
        WorldNanoEntity0104 {
            kind: WorldNanoEntityKind0104::Mob,
            id: 9001,
        }
    );
    assert_eq!(local.targets[0].absolute_hp, Some(875));
    assert!(matches!(
        local.targets[0].source_result,
        NanoSkillResult0104::Damage(NanoSkillDamageResult0104 { damage: 125, .. })
    ));

    let mut heal = target_record(16, 1, 88);
    write_i32(&mut heal, 8, 60);
    write_i32(&mut heal, 12, 940);
    let remote = decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE, &payload(2, &[heal]))
        .unwrap()
        .unwrap();
    assert_eq!(remote.delivery, NanoSkillUseDelivery0104::RemoteUse);
    assert_eq!(remote.targets[0].absolute_hp, Some(940));
}

#[test]
fn debuff_buff_and_battery_project_only_their_absolute_authority() {
    let mut debuff = target_record(32, 2, 41);
    write_i32(&mut debuff, 8, 0);
    write_i32(&mut debuff, 12, 25);
    write_i32(&mut debuff, 16, 700);
    write_i16(&mut debuff, 20, 55);
    debuff[22..24].copy_from_slice(&[0xab, 0xcd]);
    write_i32(&mut debuff, 24, 0);
    write_i32(&mut debuff, 28, 0x400);
    let projected =
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE_SUCC, &payload(8, &[debuff]))
            .unwrap()
            .unwrap();
    assert_eq!(projected.targets[0].absolute_hp, Some(700));
    assert_eq!(
        projected.targets[0].absolute_condition_bit_flag,
        Some(0x400)
    );
    assert_eq!(projected.targets[0].absolute_nano_stamina, None);

    let mut buff = target_record(16, 4, 42);
    write_i32(&mut buff, 8, 0);
    write_i32(&mut buff, 12, 0x800);
    let projected =
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE, &payload(10, &[buff]))
            .unwrap()
            .unwrap();
    assert_eq!(projected.targets[0].absolute_hp, None);
    assert_eq!(
        projected.targets[0].absolute_condition_bit_flag,
        Some(0x800)
    );

    let mut drain = target_record(40, 1, 88);
    write_i32(&mut drain, 8, 0);
    write_i32(&mut drain, 12, 10);
    write_i32(&mut drain, 16, 90);
    write_i32(&mut drain, 20, 20);
    write_i32(&mut drain, 24, 180);
    write_i16(&mut drain, 28, 45);
    drain[30..32].copy_from_slice(&[0x12, 0x34]);
    write_i32(&mut drain, 32, 1);
    write_i32(&mut drain, 36, 0x1000);
    let projected =
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE_SUCC, &payload(21, &[drain]))
            .unwrap()
            .unwrap();
    let target = projected.targets[0];
    assert_eq!(target.absolute_weapon_battery, Some(90));
    assert_eq!(target.absolute_nano_battery, Some(180));
    assert_eq!(target.absolute_nano_stamina, Some(45));
    assert_eq!(target.nano_deactivated, Some(true));
    assert_eq!(target.absolute_condition_bit_flag, Some(0x1000));
}

#[test]
fn move_resurrect_and_leech_keep_target_and_caster_authority_distinct() {
    let mut movement = target_record(24, 1, 88);
    write_i32(&mut movement, 8, 7);
    write_i32(&mut movement, 12, -100);
    write_i32(&mut movement, 16, 200);
    write_i32(&mut movement, 20, 300);
    let projected =
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE, &payload(28, &[movement]))
            .unwrap()
            .unwrap();
    assert_eq!(
        projected.targets[0].movement,
        Some(WorldNanoMovePostState0104 {
            map_number: 7,
            position: [-100, 200, 300],
        })
    );

    let mut resurrect = target_record(12, 1, 88);
    write_i32(&mut resurrect, 8, 500);
    let projected = decode_world_nano_authority_0104(
        P_FE2CL_NANO_SKILL_USE_SUCC,
        &payload(26, &[resurrect]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(projected.targets[0].absolute_hp, Some(500));

    let mut leech = vec![0; 36];
    write_i32(&mut leech, 0, 4);
    write_i32(&mut leech, 4, 9001);
    write_i32(&mut leech, 8, 40);
    write_i32(&mut leech, 12, 640); // caster HP, despite target ID 9001
    write_i32(&mut leech, 16, 4);
    write_i32(&mut leech, 20, 9001);
    write_i32(&mut leech, 24, 0);
    write_i32(&mut leech, 28, 80);
    write_i32(&mut leech, 32, 120);
    let projected =
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE_SUCC, &payload(30, &[leech]))
            .unwrap()
            .unwrap();
    assert_eq!(projected.caster.pc_id, 77);
    assert_eq!(projected.caster.absolute_hp, Some(640));
    assert_eq!(projected.targets[0].target.id, 9001);
    assert_eq!(projected.targets[0].absolute_hp, Some(120));
}

#[test]
fn malformed_packets_and_semantics_fail_before_returning_a_projection() {
    let mut damage = target_record(20, 4, 9001);
    write_i32(&mut damage, 8, 0);
    write_i32(&mut damage, 12, 25);
    write_i32(&mut damage, 16, 975);
    let mut truncated = payload(1, &[damage.clone()]);
    truncated.pop();
    assert!(matches!(
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE_SUCC, &truncated),
        Err(WorldNanoProjectionError0104::Protocol(
            NanoSkillUseDecodeError0104::Fixed(_)
        ))
    ));

    let unsupported = NanoSkillUseSuccessPrefix0104 {
        pc_id: 77,
        bullet_id: 0,
        pack_padding: [0],
        skill_id: 1,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        nano_deactivated: 0,
        nano_id: 1,
        nano_stamina: 80,
        skill_type: 6,
        target_count: 1,
    }
    .encode_prefix();
    assert_eq!(
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE, &unsupported),
        Err(WorldNanoProjectionError0104::Protocol(
            NanoSkillUseDecodeError0104::UnsupportedSkillType { skill_type: 6 }
        ))
    );

    write_i32(&mut damage, 0, 3);
    assert_eq!(
        decode_world_nano_authority_0104(
            P_FE2CL_NANO_SKILL_USE_SUCC,
            &payload(1, &[damage.clone()]),
        ),
        Err(WorldNanoProjectionError0104::UnsupportedEntityType(3))
    );

    write_i32(&mut damage, 0, 4);
    write_i32(&mut damage, 8, 2);
    assert!(matches!(
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE_SUCC, &payload(1, &[damage]),),
        Err(WorldNanoProjectionError0104::InvalidBoolean {
            field: "Damage.bProtected",
            value: 2,
        })
    ));

    let empty = NanoSkillUseSuccessPrefix0104 {
        pc_id: 77,
        bullet_id: 0,
        pack_padding: [0],
        skill_id: 1,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        nano_deactivated: 0,
        nano_id: 1,
        nano_stamina: 80,
        skill_type: 1,
        target_count: 0,
    }
    .encode_prefix();
    assert_eq!(
        decode_world_nano_authority_0104(P_FE2CL_NANO_SKILL_USE_SUCC, &empty),
        Err(WorldNanoProjectionError0104::Protocol(
            NanoSkillUseDecodeError0104::EmptyTargetResults
        ))
    );
    assert_eq!(
        decode_world_nano_authority_0104(0xdead_beef, &[1, 2]),
        Ok(None)
    );
}
