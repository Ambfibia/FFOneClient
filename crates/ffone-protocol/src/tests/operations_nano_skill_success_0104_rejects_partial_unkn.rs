use super::*;

#[test]
fn nano_skill_success_0104_rejects_partial_unknown_wrong_size_and_over_cap_tails() {
    assert!(matches!(
        NanoSkillUseSuccess0104::decode(&[0; NanoSkillUseSuccessPrefix0104::SIZE - 1]),
        Err(NanoSkillUseDecodeError0104::Fixed(
            PayloadError::WrongSize {
                expected: NanoSkillUseSuccessPrefix0104::SIZE,
                actual: 35,
            }
        ))
    ));

    let base = NanoSkillUseSuccessPrefix0104 {
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
        skill_type: 8,
        target_count: 2,
    };
    for actual in [99, 101] {
        let mut wire = base.encode_prefix();
        wire.resize(actual, 0);
        assert!(matches!(
            NanoSkillUseSuccess0104::decode(&wire),
            Err(NanoSkillUseDecodeError0104::Fixed(
                PayloadError::WrongSize {
                    expected: 100,
                    actual: rejected,
                }
            )) if rejected == actual
        ));
    }

    let mut negative = base;
    negative.target_count = -1;
    assert!(matches!(
        NanoSkillUseSuccess0104::decode(&negative.encode_prefix()),
        Err(NanoSkillUseDecodeError0104::NegativeTargetCount { target_count: -1 })
    ));

    let mut empty = base;
    empty.target_count = 0;
    assert_eq!(
        NanoSkillUseSuccess0104::decode(&empty.encode_prefix()),
        Err(NanoSkillUseDecodeError0104::EmptyTargetResults)
    );

    let mut unsupported = base;
    for skill_type in [0, 6, 9, 13, 23, 24, 29, 36, 37, 777] {
        unsupported.skill_type = skill_type;
        assert_eq!(
            NanoSkillUseSuccess0104::decode(&unsupported.encode_prefix()),
            Err(NanoSkillUseDecodeError0104::UnsupportedSkillType { skill_type })
        );
    }
    unsupported.skill_type = 6;
    assert!(matches!(
        NanoSkillUseSuccess0104::decode(&unsupported.encode_prefix()),
        Err(NanoSkillUseDecodeError0104::UnsupportedSkillType { skill_type: 6 })
    ));

    let mut over_cap = base;
    over_cap.skill_type = 26;
    over_cap.target_count = 102;
    let mut over_cap_wire = over_cap.encode_prefix();
    over_cap_wire.resize(NanoSkillUseSuccessPrefix0104::SIZE + 102 * 12, 0);
    assert!(matches!(
        NanoSkillUseSuccess0104::decode(&over_cap_wire),
        Err(NanoSkillUseDecodeError0104::TargetCountTooLarge {
            target_count: 102,
            maximum: 101,
        })
    ));

    let mut maximum = base;
    maximum.skill_type = 21;
    maximum.target_count = NanoSkillUseSuccess0104::MAX_TARGETS as i32;
    let mut maximum_wire = maximum.encode_prefix();
    maximum_wire.resize(
        NanoSkillUseSuccessPrefix0104::SIZE + NanoSkillUseSuccess0104::MAX_TARGETS * 40,
        0,
    );
    assert!(NanoSkillUseSuccess0104::decode(&maximum_wire).is_ok());
    assert_eq!(maximum_wire.len(), 4_076);

    assert_eq!(
        decode_nano_skill_use_success_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
    assert_eq!(
        decode_nano_skill_use_success_0104(packet::P_FE2CL_NANO_SKILL_USE, &[0xde, 0xad]),
        Ok(None),
        "the compatibility decoder must leave remote packets untouched"
    );
    assert!(matches!(
        decode_nano_skill_use_packet_0104(packet::P_FE2CL_NANO_SKILL_USE, &[0xde, 0xad]),
        Err(NanoSkillUseDecodeError0104::Fixed(
            PayloadError::WrongSize { .. }
        ))
    ));
}

#[test]
fn nano_tune_0104_decoder_is_strict_and_passes_unknown_frames() {
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 42,
        option: 3,
        time_limit: 0,
    };
    let success = NanoTuneSuccess0104 {
        nano_id: 36,
        skill_id: 144,
        fusion_matter: 12_345,
        item_slots: [0, 1, 2, 3, 4, 5, 6, 7, 8, -1],
        items: [item; NANO_TUNE_ITEM_SLOT_COUNT_0104],
    };
    let failure = NanoTuneFailure0104 {
        pc_id: 77,
        error_code: 5,
    };
    assert_eq!(
        decode_nano_tune_packet_0104(packet::P_FE2CL_REP_NANO_TUNE_SUCC, &success.encode(),),
        Ok(Some(NanoTunePacket0104::Success(success)))
    );
    assert_eq!(
        decode_nano_tune_packet_0104(packet::P_FE2CL_REP_NANO_TUNE_FAIL, &failure.encode(),),
        Ok(Some(NanoTunePacket0104::Failure(failure)))
    );
    for (packet_id, expected) in [
        (
            packet::P_FE2CL_REP_NANO_TUNE_SUCC,
            NanoTuneSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_NANO_TUNE_FAIL,
            NanoTuneFailure0104::SIZE,
        ),
    ] {
        for actual in [expected - 1, expected + 1] {
            assert_eq!(
                decode_nano_tune_packet_0104(packet_id, &vec![0x5a; actual]),
                Err(PayloadError::WrongSize { expected, actual })
            );
        }
    }
    assert_eq!(
        decode_nano_tune_packet_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn mission_0104_packets_round_trip_at_clean_pack4_sizes() {
    let start = PcTaskStartRequest0104 {
        task_id: 2248,
        npc_id: 71,
        escort_npc_id: -1,
    };
    assert_eq!(PcTaskStartRequest0104::SIZE, 12);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_TASK_START),
        Some(12)
    );
    assert_eq!(PcTaskStartRequest0104::decode(&start.encode()), Ok(start));

    let end = PcTaskEndRequest0104 {
        task_id: 2249,
        npc_id: 72,
        reward_box_1: 2,
        reward_box_2: -1,
        pack_padding: [0xa5, 0x5a],
        escort_npc_id: 73,
    };
    let end_bytes = end.encode();
    assert_eq!(PcTaskEndRequest0104::SIZE, 16);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_TASK_END),
        Some(16)
    );
    assert_eq!(&end_bytes[8..12], &[2, 0xff, 0xa5, 0x5a]);
    assert_eq!(PcTaskEndRequest0104::decode(&end_bytes), Ok(end));

    let start_success = PcTaskStartSuccess0104 {
        task_id: 2248,
        remaining_time: 300,
    };
    let failure = PcTaskFailure0104 {
        task_id: 2248,
        error_code: 11,
    };
    let end_success = PcTaskEndSuccess0104 { task_id: 2248 };
    let kill = PcKillQuestNpcsSuccess0104 { npc_type_id: 2676 };
    let current = PcSetCurrentMissionId0104 { mission_id: 937 };
    assert_eq!(
        PcTaskStartSuccess0104::decode(&start_success.encode()),
        Ok(start_success)
    );
    assert_eq!(PcTaskFailure0104::decode(&failure.encode()), Ok(failure));
    assert_eq!(
        PcTaskEndSuccess0104::decode(&end_success.encode()),
        Ok(end_success)
    );
    assert_eq!(PcKillQuestNpcsSuccess0104::decode(&kill.encode()), Ok(kill));
    assert_eq!(
        PcSetCurrentMissionId0104::decode(&current.encode()),
        Ok(current)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_TASK_START_SUCC),
        Some(8)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_TASK_END_SUCC),
        Some(4)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_KILL_QUEST_NPCS_SUCC),
        Some(4)
    );
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID),
        Some(4)
    );
}
