use crate::world_npc_skill_authority::*;
use ffone_protocol::{
    NpcSkillCorruptionHitPrefix0104, NpcSkillHitPrefix0104,
    packet::{P_FE2CL_NPC_SKILL_CORRUPTION_HIT, P_FE2CL_NPC_SKILL_HIT},
};

fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn skill_payload(skill_type: i32, records: &[Vec<u8>]) -> Vec<u8> {
    let prefix = NpcSkillHitPrefix0104 {
        npc_id: 314,
        skill_id: 27,
        pack_padding: [0xa5, 0x5a],
        position: [-100, 200, 300],
        skill_type,
        target_count: i32::try_from(records.len()).unwrap(),
    };
    let mut bytes = prefix.encode_prefix();
    for record in records {
        bytes.extend_from_slice(record);
    }
    bytes
}

fn corruption_payload(style: i16, records: &[Vec<u8>]) -> Vec<u8> {
    let prefix = NpcSkillCorruptionHitPrefix0104 {
        npc_id: 314,
        skill_id: 52,
        style,
        position: [-100, 200, 300],
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
fn ordinary_damage_heal_debuff_and_buff_project_absolute_state() {
    let mut damage = target_record(20, 1, 77);
    write_i32(&mut damage, 8, 0);
    write_i32(&mut damage, 12, 125);
    write_i32(&mut damage, 16, 875);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(1, &[damage]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        projected.kind,
        WorldNpcSkillCastKind0104::Skill { skill_type: 1 }
    );
    assert_eq!(projected.caster.npc_id, 314);
    assert_eq!(projected.caster.position, [-100, 200, 300]);
    assert_eq!(projected.targets[0].absolute_hp, Some(875));

    let mut heal = target_record(16, 4, 9001);
    write_i32(&mut heal, 8, 60);
    write_i32(&mut heal, 12, 940);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(34, &[heal]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(projected.targets[0].absolute_hp, Some(940));

    let mut debuff = target_record(32, 1, 77);
    write_i32(&mut debuff, 8, 0);
    write_i32(&mut debuff, 12, 25);
    write_i32(&mut debuff, 16, 700);
    write_i16(&mut debuff, 20, 55);
    write_i32(&mut debuff, 24, 1);
    write_i32(&mut debuff, 28, 0x400);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(8, &[debuff]),
    )
    .unwrap()
    .unwrap();
    let target = projected.targets[0];
    assert_eq!(target.absolute_hp, Some(700));
    assert_eq!(target.absolute_condition_bit_flag, Some(0x400));
    assert_eq!(target.absolute_nano_stamina, Some(55));
    assert_eq!(target.nano_deactivated, Some(true));

    let mut buff = target_record(16, 4, 9001);
    write_i32(&mut buff, 8, 0);
    write_i32(&mut buff, 12, 0x800);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(10, &[buff]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(projected.targets[0].absolute_hp, None);
    assert_eq!(
        projected.targets[0].absolute_condition_bit_flag,
        Some(0x800)
    );
}

#[test]
fn ordinary_battery_move_resurrect_leech_and_empty_noop_project_strictly() {
    let mut drain = target_record(40, 1, 77);
    write_i32(&mut drain, 8, 0);
    write_i32(&mut drain, 12, 10);
    write_i32(&mut drain, 16, 90);
    write_i32(&mut drain, 20, 20);
    write_i32(&mut drain, 24, 180);
    write_i16(&mut drain, 28, 45);
    write_i32(&mut drain, 32, 1);
    write_i32(&mut drain, 36, 0x1000);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(21, &[drain]),
    )
    .unwrap()
    .unwrap();
    let target = projected.targets[0];
    assert_eq!(target.absolute_weapon_battery, Some(90));
    assert_eq!(target.absolute_nano_battery, Some(180));
    assert_eq!(target.absolute_nano_stamina, Some(45));
    assert_eq!(target.nano_deactivated, Some(true));

    let mut movement = target_record(24, 1, 77);
    write_i32(&mut movement, 8, 7);
    write_i32(&mut movement, 12, -100);
    write_i32(&mut movement, 16, 200);
    write_i32(&mut movement, 20, 300);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(28, &[movement]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        projected.targets[0].movement,
        Some(WorldNpcSkillMovePostState0104 {
            map_number: 7,
            position: [-100, 200, 300],
        })
    );

    let mut resurrect = target_record(12, 4, 9001);
    write_i32(&mut resurrect, 8, 500);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(26, &[resurrect]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(projected.targets[0].absolute_hp, Some(500));

    let mut leech = vec![0; 36];
    write_i32(&mut leech, 0, 1);
    write_i32(&mut leech, 4, 77);
    write_i32(&mut leech, 8, 40);
    write_i32(&mut leech, 12, 640); // NPC caster HP, despite target ID 77
    write_i32(&mut leech, 16, 1);
    write_i32(&mut leech, 20, 77);
    write_i32(&mut leech, 24, 0);
    write_i32(&mut leech, 28, 80);
    write_i32(&mut leech, 32, 120);
    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_HIT,
        &skill_payload(30, &[leech]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(projected.caster.absolute_hp, Some(640));
    assert_eq!(projected.targets[0].absolute_hp, Some(120));

    let no_op = NpcSkillHitPrefix0104 {
        npc_id: 314,
        skill_id: 27,
        pack_padding: [0; 2],
        position: [0; 3],
        skill_type: 29,
        target_count: 0,
    }
    .encode_prefix();
    let projected = decode_world_npc_skill_authority_0104(P_FE2CL_NPC_SKILL_HIT, &no_op)
        .unwrap()
        .unwrap();
    assert!(projected.targets.is_empty());
}

#[test]
fn corruption_projects_signed_hp_and_full_nano_condition_post_state() {
    let mut result = target_record(40, 1, 77);
    write_i32(&mut result, 8, 0);
    write_i32(&mut result, 12, 450);
    write_i32(&mut result, 16, -25);
    result[20] = 16;
    result[21] = 0xa5;
    write_i16(&mut result, 22, 2);
    write_i32(&mut result, 24, 1);
    write_i16(&mut result, 28, 17);
    write_i16(&mut result, 30, 0);
    write_i32(&mut result, 32, 0x400);
    write_i32(&mut result, 36, 0x200);

    let projected = decode_world_npc_skill_authority_0104(
        P_FE2CL_NPC_SKILL_CORRUPTION_HIT,
        &corruption_payload(2, &[result]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        projected.kind,
        WorldNpcSkillCastKind0104::Corruption { style: 2 }
    );
    let target = projected.targets[0];
    assert_eq!(
        target.target,
        WorldNpcSkillEntity0104 {
            kind: WorldNpcSkillEntityKind0104::Player,
            id: 77,
        }
    );
    assert_eq!(target.absolute_hp, Some(-25));
    assert_eq!(target.absolute_condition_bit_flag, Some(0x400));
    assert_eq!(target.absolute_active_nano_slot, Some(2));
    assert_eq!(target.absolute_nano_id, Some(17));
    assert_eq!(target.absolute_nano_stamina, Some(0));
    assert_eq!(target.nano_deactivated, Some(true));
    assert_eq!(target.condition_status_deleted, Some(0x200));
}

#[test]
fn malformed_protocol_and_semantics_return_no_partial_projection() {
    let mut truncated = skill_payload(1, &[target_record(20, 1, 77)]);
    truncated.pop();
    assert!(matches!(
        decode_world_npc_skill_authority_0104(P_FE2CL_NPC_SKILL_HIT, &truncated),
        Err(WorldNpcSkillProjectionError0104::Protocol(
            NpcSkillAuthorityDecodeError0104::Fixed(_)
        ))
    ));

    let mut invalid_target = target_record(20, 3, 77);
    write_i32(&mut invalid_target, 8, 0);
    write_i32(&mut invalid_target, 12, 25);
    write_i32(&mut invalid_target, 16, 975);
    assert_eq!(
        decode_world_npc_skill_authority_0104(
            P_FE2CL_NPC_SKILL_HIT,
            &skill_payload(1, &[invalid_target]),
        ),
        Err(WorldNpcSkillProjectionError0104::UnsupportedEntityType(3))
    );

    let mut invalid_corruption_target = target_record(40, 4, 9001);
    write_i32(&mut invalid_corruption_target, 8, 0);
    write_i32(&mut invalid_corruption_target, 12, 25);
    invalid_corruption_target[20] = 8;
    write_i16(&mut invalid_corruption_target, 22, -1);
    assert!(matches!(
        decode_world_npc_skill_authority_0104(
            P_FE2CL_NPC_SKILL_CORRUPTION_HIT,
            &corruption_payload(1, &[invalid_corruption_target]),
        ),
        Err(WorldNpcSkillProjectionError0104::PlayerTargetRequired {
            context: "NPC corruption hit",
            ..
        })
    ));

    let mut invalid_flag = target_record(40, 1, 77);
    write_i32(&mut invalid_flag, 8, 0);
    write_i32(&mut invalid_flag, 12, 25);
    invalid_flag[20] = 1;
    write_i16(&mut invalid_flag, 22, -1);
    assert_eq!(
        decode_world_npc_skill_authority_0104(
            P_FE2CL_NPC_SKILL_CORRUPTION_HIT,
            &corruption_payload(1, &[invalid_flag]),
        ),
        Err(WorldNpcSkillProjectionError0104::InvalidCorruptionHitFlag(
            1
        ))
    );

    let valid_result = {
        let mut record = target_record(40, 1, 77);
        write_i32(&mut record, 8, 0);
        record[20] = 8;
        write_i16(&mut record, 22, -1);
        record
    };
    assert_eq!(
        decode_world_npc_skill_authority_0104(
            P_FE2CL_NPC_SKILL_CORRUPTION_HIT,
            &corruption_payload(3, &[valid_result]),
        ),
        Err(WorldNpcSkillProjectionError0104::InvalidCorruptionStyle(3))
    );
    assert_eq!(
        decode_world_npc_skill_authority_0104(0xdead_beef, &[0xde, 0xad]),
        Ok(None)
    );
}
