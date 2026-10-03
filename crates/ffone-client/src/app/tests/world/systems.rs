use super::*;

#[test]
fn local_infection_tick_renews_the_openfusion_combat_lease() {
    let infection = TimeBuffDotDamageTick0104 {
        character_type: 1,
        character_id: 41,
        time_buff_id: 17,
        result_character_type: 1,
        result_character_id: 41,
        protected: false,
        damage: 150,
        hp: 850,
        stamina: 0,
        nano_deactivated: false,
        condition_bit_flag: 1 << 16,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK,
        flags: 0,
        checksum: 0,
        payload: infection.encode(),
    };
    assert!(frame_observes_local_combat(&frame, 41));
    assert!(!frame_observes_local_combat(&frame, 42));

    let heal = DecodedFrame {
        payload: TimeBuffDotDamageTick0104 {
            time_buff_id: 24,
            ..infection
        }
        .encode(),
        ..frame.clone()
    };
    assert!(!frame_observes_local_combat(&heal, 41));

    let malformed = DecodedFrame {
        payload: vec![0; TimeBuffDotDamageTick0104::SIZE - 1],
        ..frame
    };
    assert!(!frame_observes_local_combat(&malformed, 41));
}

#[test]
fn pc_tick_reconciles_exact_nano_slots_and_battery_without_inventing_active_slot() {
    let mut remaining = [0; 28];
    let mut write_nano = |index: usize, id: i16, skill: i16, stamina: i16| {
        let start = index * 6;
        remaining[start..start + 2].copy_from_slice(&id.to_le_bytes());
        remaining[start + 2..start + 4].copy_from_slice(&skill.to_le_bytes());
        remaining[start + 4..start + 6].copy_from_slice(&stamina.to_le_bytes());
    };
    write_nano(0, 1, 8, 60);
    write_nano(1, 18, 21, 70);
    write_nano(2, 0, 0, 0);
    remaining[20..24].copy_from_slice(&57_i32.to_le_bytes());
    remaining[24..28].copy_from_slice(&1_i32.to_le_bytes());
    let tick = PcTick0104 { hp: 625, remaining };
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            hp: Some(700),
            nano_slots: [
                RuntimeNanoSlot {
                    nano_id: Some(1),
                    skill_id: 1,
                    stamina: 80,
                    active: true,
                },
                RuntimeNanoSlot {
                    nano_id: Some(17),
                    skill_id: 20,
                    stamina: 80,
                    active: true,
                },
                RuntimeNanoSlot {
                    nano_id: Some(36),
                    skill_id: 144,
                    stamina: 50,
                    active: false,
                },
            ],
            pending_nano_activation: Some(2),
            nano_battery: 3,
            ..default()
        },
        ..default()
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_TICK,
        flags: 0,
        checksum: 0,
        payload: tick.encode(),
    };

    assert_eq!(apply_runtime_frame(&frame, &mut runtime), Some(625));
    assert_eq!(runtime.hp, Some(625));
    assert_eq!(runtime.nano_battery, 57);
    assert_eq!(
        runtime.nano_slots,
        [
            RuntimeNanoSlot {
                nano_id: Some(1),
                skill_id: 8,
                stamina: 60,
                active: true,
            },
            RuntimeNanoSlot {
                nano_id: Some(18),
                skill_id: 21,
                stamina: 70,
                active: false,
            },
            RuntimeNanoSlot::default(),
        ]
    );
    assert_eq!(
        runtime.pending_nano_activation,
        Some(2),
        "PC_TICK has no active-slot field and cannot resolve a pending activation"
    );
}
