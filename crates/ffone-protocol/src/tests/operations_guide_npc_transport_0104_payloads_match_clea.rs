use super::*;

#[test]
fn buddy_lifecycle_0104_outbound_requests_zero_exact_pack4_padding() {
    let make = BuddyMakeRequest0104 {
        buddy_id: 0x1020_3040,
        buddy_pc_uid: 0x0102_0304_0506_0708,
    };
    let make_bytes = make.encode();
    assert_eq!(make_bytes.len(), 12);
    assert_eq!(&make_bytes[0..4], &0x1020_3040i32.to_le_bytes());
    assert_eq!(&make_bytes[4..12], &0x0102_0304_0506_0708i64.to_le_bytes());
    assert_eq!(BuddyMakeRequest0104::decode(&make_bytes), Ok(make));

    let accept = BuddyAcceptRequest0104 {
        accept_flag: 1,
        buddy_id: -17,
        buddy_pc_uid: 0x1112_1314_1516_1718,
    };
    let accept_bytes = accept.encode();
    assert_eq!(accept_bytes.len(), 16);
    assert_eq!(accept_bytes[0], 1);
    assert_eq!(&accept_bytes[1..4], &[0, 0, 0]);
    assert_eq!(&accept_bytes[4..8], &(-17i32).to_le_bytes());
    assert_eq!(
        &accept_bytes[8..16],
        &0x1112_1314_1516_1718i64.to_le_bytes()
    );
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY,
            &accept_bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::AcceptRequest(accept)))
    );

    let target_uid = 0x2122_2324_2526_2728;
    let block = BuddySetBlockRequest0104 {
        buddy_pc_uid: target_uid,
        buddy_slot: -7,
    };
    let remove = BuddyRemoveRequest0104 {
        buddy_pc_uid: target_uid,
        buddy_slot: 8,
    };
    let warp = BuddyWarpRequest0104 {
        buddy_pc_uid: target_uid,
        buddy_slot: 9,
    };
    for bytes in [block.encode(), remove.encode(), warp.encode()] {
        assert_eq!(bytes.len(), 12);
        assert_eq!(&bytes[9..12], &[0, 0, 0]);
    }
    assert_eq!(
        BuddyStateRequest0104 { unused: 0 }.encode(),
        vec![0],
        "the clean managed empty struct still occupies one byte"
    );
}

#[test]
fn pc_regen_0104_ids_layout_decoder_and_fixed_sizes_are_exact() {
    assert_eq!(packet::P_CL2FE_REQ_PC_REGEN, 0x1300_0009);
    assert_eq!(packet::P_FE2CL_REP_PC_REGEN_SUCC, 0x3100_0017);
    assert_eq!(packet::P_FE2CL_PC_REGEN, 0x3100_0099);
    assert_eq!(packet::P_FE2CL_PC_SUDDEN_DEAD, 0x3100_00ed);
    assert_eq!(PcRegenRequest0104::SIZE, 12);
    assert_eq!(PcRegenData0104::SIZE, 40);
    assert_eq!(PcRegenSuccess0104::SIZE, 48);
    assert_eq!(PcRegen0104::SIZE, 36);
    assert_eq!(PcSuddenDead0104::SIZE, 16);

    let request = PcRegenRequest0104 {
        regen_type: 6,
        e_il: 1,
        index: 0x0102_0304,
    };
    let request_bytes = request.encode();
    assert_eq!(request_bytes, vec![6, 0, 0, 0, 1, 0, 0, 0, 4, 3, 2, 1]);
    assert_eq!(
        decode_pc_regen_packet_0104(packet::P_CL2FE_REQ_PC_REGEN, &request_bytes),
        Ok(Some(PcRegenPacket0104::Request(request)))
    );

    let success = PcRegenSuccess0104 {
        regen_data: PcRegenData0104 {
            hp: 500,
            map_number: 7,
            position: [100, -200, 300],
            active_nano_slot: -1,
            nanos: [
                Nano0104 {
                    id: 1,
                    skill_id: 11,
                    stamina: 75,
                },
                Nano0104 {
                    id: 2,
                    skill_id: 12,
                    stamina: 50,
                },
                Nano0104 {
                    id: 3,
                    skill_id: 13,
                    stamina: 25,
                },
            ],
        },
        move_location: 1,
        fusion_matter: 0x1122_3344,
    };
    let success_bytes = success.encode();
    assert_eq!(&success_bytes[0..4], &500i32.to_le_bytes());
    assert_eq!(&success_bytes[4..8], &7i32.to_le_bytes());
    assert_eq!(&success_bytes[8..12], &100i32.to_le_bytes());
    assert_eq!(&success_bytes[12..16], &(-200i32).to_le_bytes());
    assert_eq!(&success_bytes[16..20], &300i32.to_le_bytes());
    assert_eq!(&success_bytes[20..22], &(-1i16).to_le_bytes());
    assert_eq!(&success_bytes[22..28], &[1, 0, 11, 0, 75, 0]);
    assert_eq!(&success_bytes[34..40], &[3, 0, 13, 0, 25, 0]);
    assert_eq!(&success_bytes[40..44], &1i32.to_le_bytes());
    assert_eq!(&success_bytes[44..48], &0x1122_3344i32.to_le_bytes());
    assert_eq!(
        decode_pc_regen_packet_0104(packet::P_FE2CL_REP_PC_REGEN_SUCC, &success_bytes),
        Ok(Some(PcRegenPacket0104::Success(success)))
    );

    let broadcast = PcRegen0104 {
        pc_id: 81,
        hp: 250,
        position: [-10, 20, -30],
        angle: 45,
        condition_bit_flag: 0x1020_3040,
        pc_state: -2,
        special_state: 3,
        nano: Nano0104 {
            id: 4,
            skill_id: 14,
            stamina: 60,
        },
    };
    let broadcast_bytes = broadcast.encode();
    assert_eq!(&broadcast_bytes[0..4], &81i32.to_le_bytes());
    assert_eq!(&broadcast_bytes[24..28], &0x1020_3040i32.to_le_bytes());
    assert_eq!(broadcast_bytes[28], 0xfe);
    assert_eq!(broadcast_bytes[29], 3);
    assert_eq!(&broadcast_bytes[30..36], &[4, 0, 14, 0, 60, 0]);
    assert_eq!(
        decode_pc_regen_packet_0104(packet::P_FE2CL_PC_REGEN, &broadcast_bytes),
        Ok(Some(PcRegenPacket0104::Broadcast(broadcast)))
    );

    let sudden_dead = PcSuddenDead0104 {
        pc_id: 81,
        sudden_dead_reason: 2,
        damage: 900,
        hp: 0,
    };
    let sudden_dead_bytes = sudden_dead.encode();
    assert_eq!(
        sudden_dead_bytes,
        vec![81, 0, 0, 0, 2, 0, 0, 0, 132, 3, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        decode_pc_regen_packet_0104(packet::P_FE2CL_PC_SUDDEN_DEAD, &sudden_dead_bytes),
        Ok(Some(PcRegenPacket0104::SuddenDead(sudden_dead)))
    );

    for (packet_type, expected) in [
        (packet::P_CL2FE_REQ_PC_REGEN, PcRegenRequest0104::SIZE),
        (packet::P_FE2CL_REP_PC_REGEN_SUCC, PcRegenSuccess0104::SIZE),
        (packet::P_FE2CL_PC_REGEN, PcRegen0104::SIZE),
        (packet::P_FE2CL_PC_SUDDEN_DEAD, PcSuddenDead0104::SIZE),
    ] {
        assert_eq!(fixed_payload_size(packet_type), Some(expected));
        for actual in [expected - 1, expected + 1] {
            assert_eq!(
                decode_pc_regen_packet_0104(packet_type, &vec![0; actual]),
                Err(PayloadError::WrongSize { expected, actual })
            );
        }
    }

    assert_eq!(
        decode_pc_regen_packet_0104(0x3100_0bad, &[0xaa, 0xbb, 0xcc]),
        Ok(None)
    );
}

#[test]
fn pc_exit_0104_ids_layout_decoder_and_fixed_sizes_are_exact() {
    assert_eq!(packet::P_CL2FE_REQ_PC_EXIT, 0x1300_0002);
    assert_eq!(packet::P_FE2CL_REP_PC_EXIT_FAIL, 0x3100_0004);
    assert_eq!(packet::P_FE2CL_REP_PC_EXIT_SUCC, 0x3100_0005);
    assert_eq!(PcExitRequest0104::SIZE, 4);
    assert_eq!(PcExitFailure0104::SIZE, 8);
    assert_eq!(PcExitSuccess0104::SIZE, 8);

    let request = PcExitRequest0104 { pc_id: 0x0102_0304 };
    let request_bytes = request.encode();
    assert_eq!(request_bytes, 0x0102_0304i32.to_le_bytes());
    assert_eq!(
        decode_pc_exit_packet_0104(packet::P_CL2FE_REQ_PC_EXIT, &request_bytes),
        Ok(Some(PcExitPacket0104::Request(request)))
    );

    let failure = PcExitFailure0104 {
        pc_id: 0x1112_1314,
        error_code: -7,
    };
    let failure_bytes = failure.encode();
    assert_eq!(&failure_bytes[0..4], &0x1112_1314i32.to_le_bytes());
    assert_eq!(&failure_bytes[4..8], &(-7i32).to_le_bytes());
    assert_eq!(
        decode_pc_exit_packet_0104(packet::P_FE2CL_REP_PC_EXIT_FAIL, &failure_bytes),
        Ok(Some(PcExitPacket0104::Failure(failure)))
    );

    let success = PcExitSuccess0104 {
        pc_id: 0x2122_2324,
        exit_code: 3,
    };
    let success_bytes = success.encode();
    assert_eq!(&success_bytes[0..4], &0x2122_2324i32.to_le_bytes());
    assert_eq!(&success_bytes[4..8], &3i32.to_le_bytes());
    assert_eq!(
        decode_pc_exit_packet_0104(packet::P_FE2CL_REP_PC_EXIT_SUCC, &success_bytes),
        Ok(Some(PcExitPacket0104::Success(success)))
    );

    for (packet_type, expected) in [
        (packet::P_CL2FE_REQ_PC_EXIT, PcExitRequest0104::SIZE),
        (packet::P_FE2CL_REP_PC_EXIT_FAIL, PcExitFailure0104::SIZE),
        (packet::P_FE2CL_REP_PC_EXIT_SUCC, PcExitSuccess0104::SIZE),
    ] {
        assert_eq!(fixed_payload_size(packet_type), Some(expected));
        for actual in [expected - 1, expected + 1] {
            assert_eq!(
                decode_pc_exit_packet_0104(packet_type, &vec![0; actual]),
                Err(PayloadError::WrongSize { expected, actual })
            );
        }
    }

    assert_eq!(
        decode_pc_exit_packet_0104(packet::P_FE2CL_PC_EXIT, &[0; PcExit0104::SIZE]),
        Ok(None)
    );
    assert_eq!(
        decode_pc_exit_packet_0104(0x3100_0bad, &[0xaa, 0xbb, 0xcc]),
        Ok(None)
    );
}

#[test]
fn guide_npc_transport_0104_ids_and_fixed_sizes_are_exact() {
    let fixed = [
        (
            packet::P_CL2FE_REQ_PC_CHANGE_MENTOR,
            0x1300_0054,
            PcChangeMentorRequest0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_CHANGE_MENTOR_SUCC,
            0x3100_0093,
            PcChangeMentorSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_CHANGE_MENTOR_FAIL,
            0x3100_0094,
            PcChangeMentorFailure0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_WARP_USE_NPC,
            0x1300_004b,
            PcWarpUseNpcRequest0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_WARP_USE_NPC_SUCC,
            0x3100_0090,
            PcWarpUseNpcSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_WARP_USE_NPC_FAIL,
            0x3100_0091,
            PcWarpUseNpcFailure0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_GOTO_SUCC,
            0x3100_0031,
            PcGotoSuccess0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_TASK_STOP,
            0x1300_0012,
            PcTaskStopRequest0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_TASK_STOP_SUCC,
            0x3100_002d,
            PcTaskStopSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_TASK_STOP_FAIL,
            0x3100_002e,
            PcTaskStopFailure0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH,
            0x1300_007a,
            PcSpecialStateSwitchRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_NPC_INTERACTION,
            0x1300_0078,
            NpcInteractionRequest0104::SIZE,
        ),
    ];

    assert_eq!(PcChangeMentorRequest0104::SIZE, 2);
    assert_eq!(PcChangeMentorSuccess0104::SIZE, 8);
    assert_eq!(PcChangeMentorFailure0104::SIZE, 8);
    assert_eq!(PcWarpUseNpcRequest0104::SIZE, 24);
    assert_eq!(
        PcWarpUseNpcSuccess0104::SIZE,
        36,
        "clean Retrobution/0104 uses the 12-byte sItemBase"
    );
    assert_eq!(PcWarpUseNpcFailure0104::SIZE, 4);
    assert_eq!(PcGotoSuccess0104::SIZE, 12);
    assert_eq!(PcTaskStopRequest0104::SIZE, 4);
    assert_eq!(PcTaskStopSuccess0104::SIZE, 4);
    assert_eq!(PcTaskStopFailure0104::SIZE, 4);
    assert_eq!(PcSpecialStateSwitchRequest0104::SIZE, 8);
    assert_eq!(NpcInteractionRequest0104::SIZE, 8);

    for (actual_id, expected_id, expected_size) in fixed {
        assert_eq!(actual_id, expected_id);
        assert_eq!(fixed_payload_size(actual_id), Some(expected_size));
    }
}

#[test]
fn guide_npc_transport_0104_payloads_match_clean_golden_layouts() {
    macro_rules! assert_strict_size {
        ($payload:ty) => {{
            let expected = <$payload>::SIZE;
            for actual in [expected - 1, expected + 1] {
                assert_eq!(
                    <$payload>::decode(&vec![0; actual]),
                    Err(PayloadError::WrongSize { expected, actual })
                );
            }
        }};
    }

    let mentor_request = PcChangeMentorRequest0104 { mentor: 4 };
    assert_eq!(mentor_request.encode(), vec![4, 0]);
    assert_eq!(
        PcChangeMentorRequest0104::decode(&mentor_request.encode()),
        Ok(mentor_request)
    );

    let mentor_success = PcChangeMentorSuccess0104 {
        mentor: 2,
        mentor_count: 1,
        fusion_matter: 0x0102_0304,
    };
    assert_eq!(mentor_success.encode(), vec![2, 0, 1, 0, 4, 3, 2, 1]);
    assert_eq!(
        PcChangeMentorSuccess0104::decode(&mentor_success.encode()),
        Ok(mentor_success)
    );

    let mentor_failure = PcChangeMentorFailure0104 {
        mentor: 3,
        error_code: -7,
    };
    assert_eq!(
        mentor_failure.encode(),
        vec![3, 0, 0, 0, 0xf9, 0xff, 0xff, 0xff]
    );
    let mut nonzero_mentor_padding = mentor_failure.encode();
    nonzero_mentor_padding[2..4].copy_from_slice(&[0xaa, 0xbb]);
    assert_eq!(
        PcChangeMentorFailure0104::decode(&nonzero_mentor_padding),
        Ok(mentor_failure),
        "ABI padding is ignored rather than incorrectly validated"
    );

    let warp_request = PcWarpUseNpcRequest0104 {
        npc_id: 2_661,
        warp_id: 76,
        e_il_1: 4,
        item_slot_1: 0,
        e_il_2: 4,
        item_slot_2: 0,
    };
    assert_eq!(
        warp_request.encode(),
        vec![
            0x65, 0x0a, 0, 0, 76, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        ]
    );
    assert_eq!(
        PcWarpUseNpcRequest0104::decode(&warp_request.encode()),
        Ok(warp_request)
    );

    let warp_success = PcWarpUseNpcSuccess0104 {
        position: [1, -2, 3],
        e_il: 4,
        item_slot_num: 6,
        item: ItemBase0104 {
            item_type: 7,
            item_id: 8,
            option: 9,
            time_limit: 10,
        },
        candy: 11,
    };
    assert_eq!(
        warp_success.encode(),
        vec![
            1, 0, 0, 0, 0xfe, 0xff, 0xff, 0xff, 3, 0, 0, 0, 4, 0, 0, 0, 6, 0, 0, 0, 7, 0, 8, 0,
            9, 0, 0, 0, 10, 0, 0, 0, 11, 0, 0, 0,
        ]
    );
    assert_eq!(
        PcWarpUseNpcSuccess0104::decode(&warp_success.encode()),
        Ok(warp_success)
    );

    let warp_failure = PcWarpUseNpcFailure0104 { error_code: -12 };
    assert_eq!(warp_failure.encode(), vec![0xf4, 0xff, 0xff, 0xff]);
    assert_eq!(
        PcWarpUseNpcFailure0104::decode(&warp_failure.encode()),
        Ok(warp_failure)
    );

    let goto_success = PcGotoSuccess0104 {
        position: [0x1234_5678, -2, 3],
    };
    assert_eq!(
        goto_success.encode(),
        vec![0x78, 0x56, 0x34, 0x12, 0xfe, 0xff, 0xff, 0xff, 3, 0, 0, 0,]
    );
    assert_eq!(
        PcGotoSuccess0104::decode(&goto_success.encode()),
        Ok(goto_success)
    );

    let task_request = PcTaskStopRequest0104 {
        task_id: 0x1234_5678,
    };
    let task_success = PcTaskStopSuccess0104 {
        task_id: 0x1234_5678,
    };
    let task_failure = PcTaskStopFailure0104 { error_code: -3 };
    assert_eq!(task_request.encode(), vec![0x78, 0x56, 0x34, 0x12]);
    assert_eq!(task_success.encode(), vec![0x78, 0x56, 0x34, 0x12]);
    assert_eq!(task_failure.encode(), vec![0xfd, 0xff, 0xff, 0xff]);
    assert_eq!(
        PcTaskStopRequest0104::decode(&task_request.encode()),
        Ok(task_request)
    );
    assert_eq!(
        PcTaskStopSuccess0104::decode(&task_success.encode()),
        Ok(task_success)
    );
    assert_eq!(
        PcTaskStopFailure0104::decode(&task_failure.encode()),
        Ok(task_failure)
    );

    let special_state = PcSpecialStateSwitchRequest0104 {
        pc_id: 0x0102_0304,
        special_state_flag: 16,
    };
    assert_eq!(special_state.encode(), vec![4, 3, 2, 1, 16, 0, 0, 0]);
    let mut nonzero_special_state_padding = special_state.encode();
    nonzero_special_state_padding[5..8].copy_from_slice(&[0xaa, 0xbb, 0xcc]);
    assert_eq!(
        PcSpecialStateSwitchRequest0104::decode(&nonzero_special_state_padding),
        Ok(special_state)
    );

    let interaction = NpcInteractionRequest0104 {
        npc_id: 2_661,
        flag: 1,
    };
    assert_eq!(interaction.encode(), vec![0x65, 0x0a, 0, 0, 1, 0, 0, 0]);
    assert_eq!(
        NpcInteractionRequest0104::decode(&interaction.encode()),
        Ok(interaction)
    );

    assert_strict_size!(PcChangeMentorRequest0104);
    assert_strict_size!(PcChangeMentorSuccess0104);
    assert_strict_size!(PcChangeMentorFailure0104);
    assert_strict_size!(PcWarpUseNpcRequest0104);
    assert_strict_size!(PcWarpUseNpcSuccess0104);
    assert_strict_size!(PcWarpUseNpcFailure0104);
    assert_strict_size!(PcGotoSuccess0104);
    assert_strict_size!(PcTaskStopRequest0104);
    assert_strict_size!(PcTaskStopSuccess0104);
    assert_strict_size!(PcTaskStopFailure0104);
    assert_strict_size!(PcSpecialStateSwitchRequest0104);
    assert_strict_size!(NpcInteractionRequest0104);
}

#[test]
fn present_npc_types_0104_ids_sizes_and_golden_payloads_are_exact() {
    assert_eq!(packet::P_CL2FE_REQ_PRESENT_NPC_TYPES, 318_767_271);
    assert_eq!(packet::P_CL2FE_REQ_PRESENT_NPC_TYPES, 0x1300_00a7);
    assert_eq!(packet::P_FE2CL_REP_PRESENT_NPC_TYPES, 822_083_895);
    assert_eq!(packet::P_FE2CL_REP_PRESENT_NPC_TYPES, 0x3100_0137);

    let request = PresentNpcTypesRequest0104 {
        last_sync_time: 0x0102_0304_0506_0708,
    };
    let request_golden = vec![8, 7, 6, 5, 4, 3, 2, 1];
    assert_eq!(PresentNpcTypesRequest0104::SIZE, 8);
    assert_eq!(request.encode(), request_golden);
    assert_eq!(
        PresentNpcTypesRequest0104::decode(&request_golden),
        Ok(request)
    );
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PRESENT_NPC_TYPES),
        Some(PresentNpcTypesRequest0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PRESENT_NPC_TYPES),
        None,
        "the reply owns a variable iCnt × int32_t tail"
    );

    let reply = PresentNpcTypesReply0104 {
        clear: -7,
        npc_types: vec![2_671, -1, 0x1234_5678],
    };
    let reply_golden = vec![
        0xf9, 0xff, 0xff, 0xff, 3, 0, 0, 0, 0x6f, 0x0a, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x78,
        0x56, 0x34, 0x12,
    ];
    assert_eq!(reply.encode(), Ok(reply_golden.clone()));
    assert_eq!(
        PresentNpcTypesReply0104::decode(&reply_golden),
        Ok(reply.clone())
    );
    assert!(reply.clears_existing());
    assert!(
        !PresentNpcTypesReply0104 {
            clear: 0,
            npc_types: Vec::new(),
        }
        .clears_existing()
    );

    assert_eq!(
        decode_present_npc_types_packet_0104(
            packet::P_CL2FE_REQ_PRESENT_NPC_TYPES,
            &request_golden,
        ),
        Ok(Some(PresentNpcTypesPacket0104::Request(request)))
    );
    assert_eq!(
        decode_present_npc_types_packet_0104(
            packet::P_FE2CL_REP_PRESENT_NPC_TYPES,
            &reply_golden,
        ),
        Ok(Some(PresentNpcTypesPacket0104::Reply(reply)))
    );
    assert_eq!(
        decode_present_npc_types_packet_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn present_npc_types_0104_counted_tail_fails_closed_and_is_lossless() {
    for actual in [7, 9] {
        assert!(matches!(
            decode_present_npc_types_packet_0104(
                packet::P_CL2FE_REQ_PRESENT_NPC_TYPES,
                &vec![0; actual],
            ),
            Err(PresentNpcTypesDecodeError0104::Fixed(
                PayloadError::WrongSize {
                    expected: PresentNpcTypesRequest0104::SIZE,
                    actual: error_actual,
                }
            )) if error_actual == actual
        ));
    }

    assert_eq!(
        PresentNpcTypesReply0104::decode(&[0; 7]),
        Err(CountedPayloadError0104::MissingHeader {
            expected_at_least: PresentNpcTypesReply0104::HEADER_SIZE,
            actual: 7,
        })
    );

    let mut negative = vec![0; PresentNpcTypesReply0104::HEADER_SIZE];
    write_i32(&mut negative, 4, -1);
    assert_eq!(
        PresentNpcTypesReply0104::decode(&negative),
        Err(CountedPayloadError0104::NegativeCount { count: -1 })
    );

    let mut truncated = vec![0; 12];
    write_i32(&mut truncated, 4, 2);
    assert_eq!(
        PresentNpcTypesReply0104::decode(&truncated),
        Err(CountedPayloadError0104::WrongSize {
            expected: 16,
            actual: 12,
        })
    );

    let mut trailing = vec![0; 16];
    write_i32(&mut trailing, 4, 1);
    assert_eq!(
        PresentNpcTypesReply0104::decode(&trailing),
        Err(CountedPayloadError0104::WrongSize {
            expected: 12,
            actual: 16,
        })
    );

    let mut excessive = vec![0; PresentNpcTypesReply0104::HEADER_SIZE];
    write_i32(
        &mut excessive,
        4,
        PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET as i32 + 1,
    );
    assert_eq!(
        PresentNpcTypesReply0104::decode(&excessive),
        Err(CountedPayloadError0104::CountTooLarge {
            count: PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET + 1,
            maximum: PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET,
        })
    );
    assert_eq!(PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET, 1_020);
    assert_eq!(
        PresentNpcTypesReply0104 {
            clear: 1,
            npc_types: vec![0; PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET + 1],
        }
        .encode(),
        Err(CountedPayloadError0104::CountTooLarge {
            count: PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET + 1,
            maximum: PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET,
        })
    );

    let maximum = PresentNpcTypesReply0104 {
        clear: 1,
        npc_types: (0..PresentNpcTypesReply0104::MAX_NPC_TYPES_PER_PACKET as i32).collect(),
    };
    let maximum_wire = maximum.encode().unwrap();
    assert_eq!(maximum_wire.len(), OPENFUSION_PAYLOAD_CAPACITY_0104);
    assert_eq!(PresentNpcTypesReply0104::decode(&maximum_wire), Ok(maximum));

    let repeated = PresentNpcTypesReply0104 {
        clear: 0,
        npc_types: vec![7, 7, -1],
    };
    assert_eq!(
        PresentNpcTypesReply0104::decode(&repeated.encode().unwrap()),
        Ok(repeated),
        "transport preserves order and duplicates for transactional application validation"
    );
}
