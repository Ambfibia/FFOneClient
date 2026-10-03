use super::*;

#[test]
fn shard_pc_tick_cannot_overwrite_local_tutorial_damage_or_granted_nano() {
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_TICK,
        flags: 0,
        checksum: 0,
        payload: PcTick0104 {
            hp: 1_000,
            remaining: [0; 28],
        }
        .encode(),
    };
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            hp: Some(950),
            nano_slots: [
                RuntimeNanoSlot {
                    nano_id: Some(1),
                    skill_id: 1,
                    stamina: 100,
                    active: false,
                },
                RuntimeNanoSlot::default(),
                RuntimeNanoSlot::default(),
            ],
            ..default()
        },
        ..default()
    };

    assert_eq!(
        apply_runtime_frame_with_authority(&frame, &mut runtime, false),
        None
    );
    assert_eq!(runtime.hp, Some(950));
    assert_eq!(
        runtime.nano_slots[0],
        RuntimeNanoSlot {
            nano_id: Some(1),
            skill_id: 1,
            stamina: 100,
            active: false,
        },
        "the shard's pre-tutorial loadout must not revoke the locally granted Buttercup Nano"
    );

    assert_eq!(
        apply_runtime_frame_with_authority(&frame, &mut runtime, true),
        Some(1_000)
    );
    assert_eq!(runtime.hp, Some(1_000));
    assert_eq!(runtime.nano_slots, [RuntimeNanoSlot::default(); 3]);
}

#[test]
fn skill_buff_tick_router_only_decodes_the_condition_bearing_infection_tail() {
    let mut model = SkillBuffUiModel::default();
    let mut movement = movement_buffs::MovementBuffs::default();
    model.local_character_id = Some(77);
    model.target = Some(SkillBuffTargetUi {
        character_type: 2,
        character_id: 88,
        condition_bit_flag: 0,
    });
    let mut bounding_ball = vec![0; 32];
    bounding_ball[8..10].copy_from_slice(&19_i16.to_le_bytes());
    assert_eq!(
        apply_skill_buff_frame(
            &DecodedFrame {
                packet_type: packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK,
                flags: 0,
                checksum: 0,
                payload: bounding_ball,
            },
            &mut model,
            &mut movement,
        ),
        Ok(true)
    );
    assert_eq!(model.local_condition_bit_flag, 0);

    let mut truncated_infection = vec![0; 32];
    truncated_infection[8..10].copy_from_slice(&17_i16.to_le_bytes());
    assert!(
        apply_skill_buff_frame(
            &DecodedFrame {
                packet_type: packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK,
                flags: 0,
                checksum: 0,
                payload: truncated_infection,
            },
            &mut model,
            &mut movement,
        )
        .is_err()
    );

    assert_eq!(
        apply_skill_buff_frame(
            &DecodedFrame {
                packet_type: packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK,
                flags: 0,
                checksum: 0,
                payload: TimeBuffDotDamageTick0104 {
                    character_type: 4,
                    character_id: 88,
                    time_buff_id: 17,
                    result_character_type: 1,
                    result_character_id: 77,
                    protected: true,
                    damage: 100,
                    hp: 500,
                    stamina: 0,
                    nano_deactivated: false,
                    condition_bit_flag: 0x10080,
                }
                .encode(),
            },
            &mut model,
            &mut movement,
        ),
        Ok(true)
    );
    assert_eq!(model.local_condition_bit_flag, 0);
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x10080)
    );
}
