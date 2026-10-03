use super::*;

#[test]
fn npc_skill_hit_authority_decodes_every_proven_openfusion_result_family() {
    assert_eq!(NpcSkillHitPrefix0104::SIZE, 28);
    assert_eq!(NpcSkillHit0104::MAX_TARGETS, 101);
    assert_eq!(fixed_payload_size(packet::P_FE2CL_NPC_SKILL_HIT), None);

    let base = NpcSkillHitPrefix0104 {
        npc_id: 314,
        skill_id: 27,
        pack_padding: [0xa5, 0x5a],
        position: [-100, 200, 300],
        skill_type: 8,
        target_count: 2,
    };
    let prefix = base.encode_prefix();
    assert_eq!(&prefix[0..4], &314_i32.to_le_bytes());
    assert_eq!(&prefix[4..6], &27_i16.to_le_bytes());
    assert_eq!(&prefix[6..8], &[0xa5, 0x5a]);
    assert_eq!(&prefix[8..12], &(-100_i32).to_le_bytes());
    assert_eq!(&prefix[12..16], &200_i32.to_le_bytes());
    assert_eq!(&prefix[16..20], &300_i32.to_le_bytes());
    assert_eq!(&prefix[20..24], &8_i32.to_le_bytes());
    assert_eq!(&prefix[24..28], &2_i32.to_le_bytes());

    let supported = [
        (26, 12),
        (2, 16),
        (7, 16),
        (10, 16),
        (11, 16),
        (12, 16),
        (14, 16),
        (15, 16),
        (16, 16),
        (17, 16),
        (18, 16),
        (19, 16),
        (20, 16),
        (25, 16),
        (31, 16),
        (32, 16),
        (33, 16),
        (34, 16),
        (35, 16),
        (1, 20),
        (22, 20),
        (38, 20),
        (39, 20),
        (27, 24),
        (28, 24),
        (3, 32),
        (4, 32),
        (5, 32),
        (8, 32),
        (30, 36),
        (21, 40),
    ];
    for (skill_type, record_size) in supported {
        assert_eq!(npc_skill_result_size_0104(skill_type), Some(record_size));
        let mut expected_prefix = base;
        expected_prefix.skill_type = skill_type;
        let mut wire = expected_prefix.encode_prefix();
        wire.resize(NpcSkillHitPrefix0104::SIZE + 2 * record_size, 0x5a);
        let decoded = NpcSkillHit0104::decode(&wire).unwrap();
        assert_eq!(decoded.prefix(), expected_prefix);
        assert_eq!(decoded.result_record_size(), record_size);
        assert_eq!(decoded.result_bytes(), vec![0x5a; 2 * record_size]);
        assert_eq!(decoded.results().len(), 2);
        assert_eq!(
            decode_npc_skill_authority_0104(packet::P_FE2CL_NPC_SKILL_HIT, &wire),
            Ok(Some(NpcSkillAuthorityPacket0104::SkillHit(decoded)))
        );
    }

    let no_op = NpcSkillHitPrefix0104 {
        skill_type: 29,
        target_count: 0,
        ..base
    };
    let decoded = NpcSkillHit0104::decode(&no_op.encode_prefix()).unwrap();
    assert_eq!(npc_skill_result_size_0104(29), Some(0));
    assert_eq!(decoded.prefix(), no_op);
    assert_eq!(decoded.result_record_size(), 0);
    assert!(decoded.result_bytes().is_empty());
    assert!(decoded.results().is_empty());
}

#[test]
fn npc_corruption_hit_authority_preserves_exact_pack4_result() {
    assert_eq!(NpcSkillCorruptionHitPrefix0104::SIZE, 24);
    assert_eq!(NpcSkillCorruptionResult0104::SIZE, 40);
    assert_eq!(NpcSkillCorruptionHit0104::MAX_TARGETS, 101);
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT),
        None
    );

    let prefix = NpcSkillCorruptionHitPrefix0104 {
        npc_id: 314,
        skill_id: 52,
        style: 2,
        position: [-100, 200, 300],
        target_count: 1,
    };
    let prefix_bytes = prefix.encode_prefix();
    assert_eq!(&prefix_bytes[0..4], &314_i32.to_le_bytes());
    assert_eq!(&prefix_bytes[4..6], &52_i16.to_le_bytes());
    assert_eq!(&prefix_bytes[6..8], &2_i16.to_le_bytes());
    assert_eq!(&prefix_bytes[8..12], &(-100_i32).to_le_bytes());
    assert_eq!(&prefix_bytes[12..16], &200_i32.to_le_bytes());
    assert_eq!(&prefix_bytes[16..20], &300_i32.to_le_bytes());
    assert_eq!(&prefix_bytes[20..24], &1_i32.to_le_bytes());

    let mut result_bytes = vec![0; NpcSkillCorruptionResult0104::SIZE];
    write_i32(&mut result_bytes, 0, 1);
    write_i32(&mut result_bytes, 4, 77);
    write_i32(&mut result_bytes, 8, 0);
    write_i32(&mut result_bytes, 12, 450);
    write_i32(&mut result_bytes, 16, -25);
    result_bytes[20] = 16;
    result_bytes[21] = 0xa5;
    write_test_i16(&mut result_bytes, 22, 2);
    write_i32(&mut result_bytes, 24, 1);
    write_test_i16(&mut result_bytes, 28, 17);
    write_test_i16(&mut result_bytes, 30, 0);
    write_i32(&mut result_bytes, 32, 0x400);
    write_i32(&mut result_bytes, 36, 0x200);

    let mut wire = prefix_bytes;
    wire.extend_from_slice(&result_bytes);
    let decoded = NpcSkillCorruptionHit0104::decode(&wire).unwrap();
    assert_eq!(decoded.prefix(), prefix);
    assert_eq!(decoded.result_bytes(), result_bytes);
    assert_eq!(
        decoded.results(),
        &[NpcSkillCorruptionResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 1,
                id: 77,
            },
            protected: 0,
            damage: 450,
            hp: -25,
            hit_flag: 16,
            pack_padding: [0xa5],
            active_nano_slot: 2,
            nano_deactivated: 1,
            nano_id: 17,
            nano_stamina: 0,
            condition_bit_flag: 0x400,
            condition_status_deleted: 0x200,
        }]
    );
    assert_eq!(
        decode_npc_skill_authority_0104(packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT, &wire,),
        Ok(Some(NpcSkillAuthorityPacket0104::CorruptionHit(decoded)))
    );
}

#[test]
fn npc_skill_authority_rejects_malformed_counts_sizes_and_unknown_families() {
    assert!(matches!(
        NpcSkillHit0104::decode(&[0; NpcSkillHitPrefix0104::SIZE - 1]),
        Err(NpcSkillAuthorityDecodeError0104::Fixed(
            PayloadError::WrongSize {
                expected: NpcSkillHitPrefix0104::SIZE,
                actual: 27,
            }
        ))
    ));
    assert!(matches!(
        NpcSkillCorruptionHit0104::decode(&[0; NpcSkillCorruptionHitPrefix0104::SIZE - 1]),
        Err(NpcSkillAuthorityDecodeError0104::Fixed(
            PayloadError::WrongSize {
                expected: NpcSkillCorruptionHitPrefix0104::SIZE,
                actual: 23,
            }
        ))
    ));

    let hit = NpcSkillHitPrefix0104 {
        npc_id: 314,
        skill_id: 27,
        pack_padding: [0; 2],
        position: [0; 3],
        skill_type: 1,
        target_count: 1,
    };
    for actual in [47, 49] {
        let mut wire = hit.encode_prefix();
        wire.resize(actual, 0);
        assert!(matches!(
            NpcSkillHit0104::decode(&wire),
            Err(NpcSkillAuthorityDecodeError0104::Fixed(
                PayloadError::WrongSize {
                    expected: 48,
                    actual: rejected,
                }
            )) if rejected == actual
        ));
    }

    let corruption = NpcSkillCorruptionHitPrefix0104 {
        npc_id: 314,
        skill_id: 52,
        style: 1,
        position: [0; 3],
        target_count: 1,
    };
    for actual in [63, 65] {
        let mut wire = corruption.encode_prefix();
        wire.resize(actual, 0);
        assert!(matches!(
            NpcSkillCorruptionHit0104::decode(&wire),
            Err(NpcSkillAuthorityDecodeError0104::Fixed(
                PayloadError::WrongSize {
                    expected: 64,
                    actual: rejected,
                }
            )) if rejected == actual
        ));
    }

    let mut negative_hit = hit;
    negative_hit.target_count = -1;
    assert!(matches!(
        NpcSkillHit0104::decode(&negative_hit.encode_prefix()),
        Err(NpcSkillAuthorityDecodeError0104::NegativeTargetCount {
            packet: "NPC_SKILL_HIT",
            target_count: -1,
        })
    ));
    let mut empty_corruption = corruption;
    empty_corruption.target_count = 0;
    assert!(matches!(
        NpcSkillCorruptionHit0104::decode(&empty_corruption.encode_prefix()),
        Err(NpcSkillAuthorityDecodeError0104::EmptyTargetResults {
            packet: "NPC_SKILL_CORRUPTION_HIT",
        })
    ));

    let mut unknown = hit;
    unknown.skill_type = 6;
    unknown.target_count = 0;
    assert_eq!(
        NpcSkillHit0104::decode(&unknown.encode_prefix()),
        Err(NpcSkillAuthorityDecodeError0104::UnsupportedSkillType { skill_type: 6 })
    );
    let mut invalid_no_op = hit;
    invalid_no_op.skill_type = 29;
    assert_eq!(
        NpcSkillHit0104::decode(&invalid_no_op.encode_prefix()),
        Err(NpcSkillAuthorityDecodeError0104::NoResultSkillHasTargets {
            skill_type: 29,
            target_count: 1,
        })
    );

    let mut over_cap_hit = hit;
    over_cap_hit.target_count = 102;
    assert!(matches!(
        NpcSkillHit0104::decode(&over_cap_hit.encode_prefix()),
        Err(NpcSkillAuthorityDecodeError0104::TargetCountTooLarge {
            packet: "NPC_SKILL_HIT",
            target_count: 102,
            maximum: 101,
        })
    ));
    let mut over_cap_corruption = corruption;
    over_cap_corruption.target_count = 102;
    assert!(matches!(
        NpcSkillCorruptionHit0104::decode(&over_cap_corruption.encode_prefix()),
        Err(NpcSkillAuthorityDecodeError0104::TargetCountTooLarge {
            packet: "NPC_SKILL_CORRUPTION_HIT",
            target_count: 102,
            maximum: 101,
        })
    ));

    let empty_supported = NpcSkillHitPrefix0104 {
        target_count: 0,
        ..hit
    };
    assert!(NpcSkillHit0104::decode(&empty_supported.encode_prefix()).is_ok());
    assert_eq!(
        decode_npc_skill_authority_0104(0xdead_beef, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn special_weapon_fire_requests_match_clean_pack4_layouts() {
    let rocket = PcRocketStyleFireRequest0104 {
        skill_id: 0,
        position: [100, 200, 300],
        destination: [400, 500, 600],
    };
    assert_eq!(
        rocket.encode(),
        [0, 100, 200, 300, 400, 500, 600]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        PcRocketStyleFireRequest0104::decode(&rocket.encode()),
        Ok(rocket)
    );

    let grenade = PcGrenadeStyleFireRequest0104 {
        skill_id: 0,
        destination: [-100, 200, -300],
    };
    assert_eq!(
        grenade.encode(),
        [0, -100, 200, -300]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        PcGrenadeStyleFireRequest0104::decode(&grenade.encode()),
        Ok(grenade)
    );
}

#[test]
fn special_weapon_fire_replies_preserve_weapon_identity_and_routes() {
    let bullet = PcBullet0104 {
        attack_type: 1,
        id: 365,
        charged: 0,
    };
    let local_rocket = PcRocketStyleFireSuccess0104 {
        skill_id: 0,
        position: [100, 200, 300],
        destination: [400, 500, 600],
        bullet_id: 7,
        pack_padding: [0xa5, 0x5a, 0xff],
        bullet,
        weapon_battery: 89,
        nano_deactivated: 0,
        nano_id: 12,
        nano_stamina: 149,
    };
    assert_eq!(local_rocket.encode().len(), 56);
    assert_eq!(
        decode_pc_warhead_fire_packet_0104(
            packet::P_FE2CL_REP_PC_ROCKET_STYLE_FIRE_SUCC,
            &local_rocket.encode(),
        ),
        Ok(Some(PcWarheadFirePacket0104::LocalRocket(local_rocket)))
    );

    let remote_rocket = PcRocketStyleFire0104 {
        pc_id: 42,
        position: [100, 200, 300],
        destination: [400, 500, 600],
        bullet_id: 7,
        pack_padding: [0xa5, 0x5a, 0xff],
        bullet,
        nano_deactivated: 0,
    };
    assert_eq!(remote_rocket.encode().len(), 48);
    assert_eq!(
        decode_pc_warhead_fire_packet_0104(
            packet::P_FE2CL_PC_ROCKET_STYLE_FIRE,
            &remote_rocket.encode(),
        ),
        Ok(Some(PcWarheadFirePacket0104::RemoteRocket(remote_rocket)))
    );

    let local_grenade = PcGrenadeStyleFireSuccess0104 {
        skill_id: 0,
        destination: [-100, 200, -300],
        bullet_id: 3,
        pack_padding: [1, 2, 3],
        bullet,
        weapon_battery: 88,
        nano_deactivated: 0,
        nano_id: 12,
        nano_stamina: 148,
    };
    assert_eq!(local_grenade.encode().len(), 44);
    assert_eq!(
        decode_pc_warhead_fire_packet_0104(
            packet::P_FE2CL_REP_PC_GRENADE_STYLE_FIRE_SUCC,
            &local_grenade.encode(),
        ),
        Ok(Some(PcWarheadFirePacket0104::LocalGrenade(local_grenade)))
    );

    let remote_grenade = PcGrenadeStyleFire0104 {
        pc_id: 42,
        destination: [-100, 200, -300],
        bullet_id: 3,
        pack_padding: [1, 2, 3],
        bullet,
        nano_deactivated: 0,
    };
    assert_eq!(remote_grenade.encode().len(), 36);
    assert_eq!(
        decode_pc_warhead_fire_packet_0104(
            packet::P_FE2CL_PC_GRENADE_STYLE_FIRE,
            &remote_grenade.encode(),
        ),
        Ok(Some(PcWarheadFirePacket0104::RemoteGrenade(remote_grenade)))
    );
}

#[test]
fn npc_combat_decoder_is_loss_aware_at_its_boundary() {
    let unknown = [0xaa, 0xbb, 0xcc];
    assert_eq!(
        decode_npc_combat_packet_0104(0x3100_0abc, &unknown),
        Ok(None)
    );

    assert!(matches!(
        decode_npc_combat_packet_0104(packet::P_FE2CL_NPC_MOVE, &[0; 23]),
        Err(NpcCombatDecodeError0104::Fixed(PayloadError::WrongSize {
            expected: 24,
            actual: 23,
        }))
    ));

    let mut malformed_results = vec![0; 8 + AttackResult0104::SIZE];
    write_i32(&mut malformed_results, 4, 2);
    assert!(matches!(
        decode_npc_combat_packet_0104(packet::P_FE2CL_NPC_ATTACK_PCS, &malformed_results),
        Err(NpcCombatDecodeError0104::Counted(
            CountedPayloadError0104::WrongSize { .. }
        ))
    ));
}

#[test]
fn around_del_npc_counted_ids_round_trip() {
    let packet = AroundDelNpc0104 {
        npc_ids: vec![17, 23, 42],
    };
    let encoded = packet.encode().unwrap();
    assert_eq!(AroundDelNpc0104::decode(&encoded), Ok(packet.clone()));
    assert_eq!(
        decode_npc_combat_packet_0104(packet::P_FE2CL_AROUND_DEL_NPC, &encoded),
        Ok(Some(NpcCombatPacket0104::AroundDelNpc(packet)))
    );
}

#[test]
fn skill_buff_packets_match_clean_0104_pack4_layouts() {
    assert_eq!(packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT, 822_083_680);
    assert_eq!(packet::P_FE2CL_PC_BUFF_UPDATE, 822_083_807);
    assert_eq!(packet::P_FE2CL_PC_CASH_BUFF_UPDATE, 822_083_864);

    let time_buff = TimeBuff0104 {
        time_limit: 0x0102_0304_0506_0708,
        time_duration: 0x1112_1314_1516_1718,
        time_repeat: -3,
        value: 450,
        confirm_number: 27,
    };
    let time_bytes = time_buff.encode();
    assert_eq!(&time_bytes[0..8], &time_buff.time_limit.to_le_bytes());
    assert_eq!(&time_bytes[8..16], &time_buff.time_duration.to_le_bytes());
    assert_eq!(&time_bytes[16..20], &(-3i32).to_le_bytes());
    assert_eq!(TimeBuff0104::decode(&time_bytes), Ok(time_buff));

    let regular = PcBuffUpdate0104 {
        buff_id: 17,
        update_kind: 3,
        buff_type: 2,
        time_buff,
        condition_bit_flag: 0x0001_0080,
    };
    let regular_bytes = regular.encode();
    assert_eq!(&regular_bytes[12..40], time_bytes);
    assert_eq!(PcBuffUpdate0104::decode(&regular_bytes), Ok(regular));
    assert_eq!(
        decode_skill_buff_packet_0104(packet::P_FE2CL_PC_BUFF_UPDATE, &regular_bytes),
        Ok(Some(SkillBuffPacket0104::Pc(regular)))
    );

    let cash = PcCashBuffUpdate0104 {
        buff_id: 14,
        update_kind: 1,
        time_buff,
        condition_bit_flag: 0x2000,
    };
    let cash_bytes = cash.encode();
    assert_eq!(&cash_bytes[8..36], time_bytes);
    assert_eq!(PcCashBuffUpdate0104::decode(&cash_bytes), Ok(cash));
    assert_eq!(
        decode_skill_buff_packet_0104(packet::P_FE2CL_PC_CASH_BUFF_UPDATE, &cash_bytes),
        Ok(Some(SkillBuffPacket0104::Cash(cash)))
    );

    let timeout = CharTimeBuffTimeout0104 {
        character_type: 3,
        character_id: 912,
        condition_bit_flag: 0x400,
    };
    let timeout_bytes = timeout.encode();
    assert_eq!(
        decode_skill_buff_packet_0104(packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT, &timeout_bytes),
        Ok(Some(SkillBuffPacket0104::Timeout(timeout)))
    );
    assert_eq!(
        decode_skill_buff_packet_0104(0x3100_0abc, &[1, 2]),
        Ok(None)
    );
    assert!(matches!(
        decode_skill_buff_packet_0104(packet::P_FE2CL_PC_BUFF_UPDATE, &[0; 43]),
        Err(PayloadError::WrongSize {
            expected: 44,
            actual: 43
        })
    ));

    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT),
        Some(12)
    );
    assert_eq!(fixed_payload_size(packet::P_FE2CL_PC_BUFF_UPDATE), Some(44));
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_PC_CASH_BUFF_UPDATE),
        Some(40)
    );
}

#[test]
fn group_roster_decodes_exact_pack4_layout_and_utf16_names() {
    assert_eq!(GroupPcMemberInfo0104::SIZE, 112);
    assert_eq!(GroupNpcMemberInfo0104::SIZE, 32);

    let mut payload =
        vec![0u8; 12 + GroupPcMemberInfo0104::SIZE + GroupNpcMemberInfo0104::SIZE];
    write_i32(&mut payload, 0, 77);
    write_i32(&mut payload, 4, 1);
    write_i32(&mut payload, 8, 1);

    let pc = 12;
    write_i32(&mut payload, pc, 101);
    write_i64(&mut payload, pc + 4, 0x0102_0304_0506_0708);
    payload[pc + 12] = 1;
    write_utf16(
        &mut payload,
        pc + 14,
        &FixedUtf16::<9>::from_str("Декстер").unwrap(),
    );
    write_utf16(
        &mut payload,
        pc + 32,
        &FixedUtf16::<17>::from_str("Тест").unwrap(),
    );
    payload[pc + 66] = 2;
    write_test_i16(&mut payload, pc + 68, 36);
    for (offset, value) in [
        (72, 875),
        (76, 1000),
        (80, 1),
        (84, 14),
        (88, 111),
        (92, 222),
        (96, 333),
        (100, 1),
    ] {
        write_i32(&mut payload, pc + offset, value);
    }
    write_test_i16(&mut payload, pc + 104, 7);
    write_test_i16(&mut payload, pc + 106, 8);
    write_test_i16(&mut payload, pc + 108, 9);

    let npc = pc + GroupPcMemberInfo0104::SIZE;
    for (index, value) in [201, 501, 750, 1, 14, -10, -20, -30]
        .into_iter()
        .enumerate()
    {
        write_i32(&mut payload, npc + index * 4, value);
    }

    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_GROUP_MEMBER_INFO,
        flags: 0,
        checksum: 0,
        payload,
    };
    let Some(GroupPacket0104::Roster(roster)) = decode_group_packet_0104(&frame).unwrap()
    else {
        panic!("expected group roster");
    };
    assert_eq!(roster.context_id, 77);
    assert_eq!(roster.npc_id, None);
    assert_eq!(roster.pc_members.len(), 1);
    assert_eq!(roster.npc_members.len(), 1);

    let member = &roster.pc_members[0];
    assert_eq!(member.pc_id, 101);
    assert_eq!(member.pc_uid, 0x0102_0304_0506_0708);
    assert_eq!(member.first_name.to_string_lossy(), "Декстер");
    assert_eq!(member.last_name.to_string_lossy(), "Тест");
    assert_eq!(member.level, 36);
    assert_eq!(member.hp, 875);
    assert_eq!(member.max_hp, 1000);
    assert_eq!(member.position, [111, 222, 333]);
    assert_eq!(
        member.nano,
        Nano0104 {
            id: 7,
            skill_id: 8,
            stamina: 9,
        }
    );
    assert_eq!(roster.npc_members[0].npc_id, 201);
    assert_eq!(roster.npc_members[0].hp, 750);
    assert_eq!(roster.npc_members[0].position, [-10, -20, -30]);
}

#[test]
fn npc_group_success_uses_two_id_header_before_counts() {
    let mut payload = vec![0u8; 16];
    write_i32(&mut payload, 0, 41);
    write_i32(&mut payload, 4, 99);
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NPC_GROUP_INVITE_SUCC,
        flags: 0,
        checksum: 0,
        payload,
    };
    assert_eq!(
        decode_group_packet_0104(&frame),
        Ok(Some(GroupPacket0104::Roster(GroupRoster0104 {
            context_id: 41,
            npc_id: Some(99),
            pc_members: vec![],
            npc_members: vec![],
        })))
    );
}

#[test]
fn group_roster_rejects_short_trailing_negative_and_oversized_counts() {
    let frame = |payload: Vec<u8>| DecodedFrame {
        packet_type: packet::P_FE2CL_PC_GROUP_JOIN,
        flags: 0,
        checksum: 0,
        payload,
    };

    assert_eq!(
        decode_group_packet_0104(&frame(vec![0; 11])),
        Err(CountedPayloadError0104::MissingHeader {
            expected_at_least: 12,
            actual: 11,
        })
    );
    assert_eq!(
        decode_group_packet_0104(&frame(vec![0; 13])),
        Err(CountedPayloadError0104::WrongSize {
            expected: 12,
            actual: 13,
        })
    );

    let mut negative = vec![0; 12];
    write_i32(&mut negative, 4, -1);
    assert_eq!(
        decode_group_packet_0104(&frame(negative)),
        Err(CountedPayloadError0104::NegativeCount { count: -1 })
    );

    for (offset, count, maximum) in [(4, 5, 4), (8, 6, 5)] {
        let mut payload = vec![0; 12];
        write_i32(&mut payload, offset, count as i32);
        assert_eq!(
            decode_group_packet_0104(&frame(payload)),
            Err(CountedPayloadError0104::CountTooLarge { count, maximum })
        );
    }
}

#[test]
fn group_leave_success_is_exact_and_unknown_packets_are_ignored() {
    let leave = |payload: Vec<u8>| DecodedFrame {
        packet_type: packet::P_FE2CL_PC_GROUP_LEAVE_SUCC,
        flags: 0,
        checksum: 0,
        payload,
    };
    assert_eq!(
        decode_group_packet_0104(&leave(vec![0])),
        Ok(Some(GroupPacket0104::LeaveSuccess))
    );
    assert_eq!(
        decode_group_packet_0104(&leave(vec![])),
        Err(CountedPayloadError0104::WrongSize {
            expected: 1,
            actual: 0,
        })
    );
    assert_eq!(
        decode_group_packet_0104(&leave(vec![0, 0])),
        Err(CountedPayloadError0104::WrongSize {
            expected: 1,
            actual: 2,
        })
    );

    let unknown = DecodedFrame {
        packet_type: 0x3100_0abc,
        flags: 0,
        checksum: 0,
        payload: vec![0xff; 3],
    };
    assert_eq!(decode_group_packet_0104(&unknown), Ok(None));
}

#[test]
fn freechat_0104_unicode_payloads_have_exact_source_layouts() {
    assert_eq!(packet::P_CL2FE_REQ_PC_FREECHAT, 0x1300_0007);
    assert_eq!(packet::P_FE2CL_REP_PC_FREECHAT_SUCC, 0x3100_0012);
    assert_eq!(FreeChatRequest0104::SIZE, 260);
    assert_eq!(FreeChatSuccess0104::SIZE, 264);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_FREECHAT),
        Some(260)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_FREECHAT_SUCC),
        Some(264)
    );

    let message = FixedUtf16::<128>::from_str("Привет, мир 🌌").unwrap();
    let request = FreeChatRequest0104 {
        message: message.clone(),
        emote_code: -17,
    };
    let request_bytes = request.encode();
    assert_eq!(request_bytes.len(), 260);
    assert_eq!(&request_bytes[256..260], &(-17i32).to_le_bytes());
    assert_eq!(FreeChatRequest0104::decode(&request_bytes), Ok(request));

    let success = FreeChatSuccess0104 {
        pc_id: 0x0102_0304,
        message,
        emote_code: 42,
    };
    let success_bytes = success.encode();
    assert_eq!(success_bytes.len(), 264);
    assert_eq!(&success_bytes[..4], &0x0102_0304i32.to_le_bytes());
    assert_eq!(&success_bytes[260..264], &42i32.to_le_bytes());
    assert_eq!(
        FreeChatSuccess0104::decode(&success_bytes),
        Ok(success.clone())
    );
    assert_eq!(
        decode_freechat_packet_0104(packet::P_FE2CL_REP_PC_FREECHAT_SUCC, &success_bytes),
        Ok(Some(FreeChatPacket0104::Success(success)))
    );
}

#[test]
fn freechat_0104_decoder_is_strict_and_preserves_unknown_packets() {
    assert_eq!(
        decode_freechat_packet_0104(0x3100_0abc, &[0xaa, 0xbb, 0xcc]),
        Ok(None)
    );
    assert_eq!(
        decode_freechat_packet_0104(packet::P_CL2FE_REQ_PC_FREECHAT, &[0; 259]),
        Err(PayloadError::WrongSize {
            expected: 260,
            actual: 259,
        })
    );
    assert_eq!(
        decode_freechat_packet_0104(packet::P_FE2CL_REP_PC_FREECHAT_SUCC, &[0; 265]),
        Err(PayloadError::WrongSize {
            expected: 264,
            actual: 265,
        })
    );
}
