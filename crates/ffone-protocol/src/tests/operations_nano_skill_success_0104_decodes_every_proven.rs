use super::*;

#[test]
fn vendor_0104_ids_sizes_and_pack4_offsets_match_clean_abi() {
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_BUY, 0x1300_0017);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_SELL, 0x1300_0018);
    assert_eq!(packet::P_CL2FE_REQ_PC_ITEM_DELETE, 0x1300_0019);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_START, 0x1300_0030);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE, 0x1300_0031);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY, 0x1300_0032);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY, 0x1300_004a);
    assert_eq!(packet::P_CL2FE_REQ_PC_DISASSEMBLE_ITEM, 0x1300_00a2);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC, 0x3100_005c);
    assert_eq!(packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL, 0x3100_012b);

    let item = ItemBase0104 {
        item_type: 7,
        item_id: 0x1234,
        option: 0x1020_3040,
        time_limit: -99,
    };
    let request = VendorItemBuyRequest0104 {
        npc_id: 0x0102_0304,
        vendor_id: 0x1112_1314,
        list_id: -2,
        item,
        inventory_slot: 49,
    };
    let wire = request.encode();
    assert_eq!(wire.len(), 28);
    assert_eq!(&wire[0..4], &0x0102_0304_i32.to_le_bytes());
    assert_eq!(&wire[4..8], &0x1112_1314_i32.to_le_bytes());
    assert_eq!(wire[8], 0xfe);
    assert_eq!(&wire[9..12], &[0, 0, 0]);
    assert_eq!(&wire[12..14], &7_i16.to_le_bytes());
    assert_eq!(&wire[14..16], &0x1234_i16.to_le_bytes());
    assert_eq!(&wire[16..20], &0x1020_3040_i32.to_le_bytes());
    assert_eq!(&wire[20..24], &(-99_i32).to_le_bytes());
    assert_eq!(&wire[24..28], &49_i32.to_le_bytes());
    assert_eq!(VendorItemBuyRequest0104::decode(&wire), Ok(request));

    let restore = VendorItemRestoreBuyRequest0104 {
        npc_id: request.npc_id,
        vendor_id: request.vendor_id,
        list_id: request.list_id,
        item,
        inventory_slot: request.inventory_slot,
    };
    assert_eq!(restore.encode(), wire);

    let battery = VendorBatteryBuyRequest0104 {
        npc_id: request.npc_id,
        vendor_id: request.vendor_id,
        list_id: request.list_id,
        item,
    };
    let battery_wire = battery.encode();
    assert_eq!(battery_wire.len(), 24);
    assert_eq!(&battery_wire[9..12], &[0, 0, 0]);
    assert_eq!(
        VendorBatteryBuyRequest0104::decode(&battery_wire),
        Ok(battery)
    );

    let vendor_item = ItemVendor0104 {
        vendor_id: 650,
        buy_cost: 12.5,
        item,
        sort_num: 8,
    };
    let vendor_wire = vendor_item.encode();
    let mut item_wire = [0; ItemBase0104::SIZE];
    item.encode_into(&mut item_wire);
    assert_eq!(vendor_wire.len(), 24);
    assert_eq!(&vendor_wire[0..4], &650_i32.to_le_bytes());
    assert_eq!(&vendor_wire[4..8], &12.5_f32.to_le_bytes());
    assert_eq!(&vendor_wire[8..20], &item_wire);
    assert_eq!(&vendor_wire[20..24], &8_i32.to_le_bytes());
    assert_eq!(ItemVendor0104::decode(&vendor_wire), Ok(vendor_item));

    for (packet_id, size) in [
        (
            packet::P_CL2FE_REQ_PC_VENDOR_ITEM_BUY,
            VendorItemBuyRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_VENDOR_ITEM_SELL,
            VendorItemSellRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_ITEM_DELETE,
            PcItemDeleteRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_VENDOR_START,
            VendorStartRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE,
            VendorTableUpdateRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY,
            VendorItemRestoreBuyRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY,
            VendorBatteryBuyRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_DISASSEMBLE_ITEM,
            PcDisassembleItemRequest0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC,
            VendorItemBuySuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC,
            VendorItemSellSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC,
            PcItemDeleteSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_START_SUCC,
            VendorStartSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC,
            VendorTableUpdateSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC,
            VendorItemRestoreBuySuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC,
            VendorBatteryBuySuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC,
            PcDisassembleItemSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL,
            PcDisassembleItemFailure0104::SIZE,
        ),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(size));
    }
    for packet_id in [
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_FAIL,
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_FAIL,
        packet::P_FE2CL_REP_PC_VENDOR_START_FAIL,
        packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_FAIL,
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_FAIL,
        packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_FAIL,
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(VendorFailure0104::SIZE));
    }
}

#[test]
fn fixed_utf16_reserves_a_null_terminator() {
    assert!(FixedUtf16::<4>::from_str("abc").is_ok());
    assert_eq!(
        FixedUtf16::<4>::from_str("abcd"),
        Err(PayloadError::Utf16TooLong {
            capacity: 4,
            actual: 4
        })
    );
}

#[test]
fn pc_nano_create_0104_ids_sizes_offsets_and_round_trips_match_clean_abi() {
    assert_eq!(packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC, 822_083_667);
    assert_eq!(packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL, 822_083_668);
    assert_eq!(PcNanoCreateSuccess0104::SIZE, 28);
    assert_eq!(PcNanoCreateFailure0104::SIZE, 8);

    let success = PcNanoCreateSuccess0104 {
        fusion_matter: 12_345,
        quest_item_slot: 17,
        quest_item: ItemBase0104 {
            item_type: 7,
            item_id: 912,
            option: 3,
            time_limit: 4_567,
        },
        nano: Nano0104 {
            id: 36,
            skill_id: 0,
            stamina: 150,
        },
        player_level: 35,
    };
    let wire = success.encode();
    assert_eq!(&wire[0..4], &12_345_i32.to_le_bytes());
    assert_eq!(&wire[4..8], &17_i32.to_le_bytes());
    assert_eq!(&wire[8..10], &7_i16.to_le_bytes());
    assert_eq!(&wire[10..12], &912_i16.to_le_bytes());
    assert_eq!(&wire[20..22], &36_i16.to_le_bytes());
    assert_eq!(&wire[22..24], &0_i16.to_le_bytes());
    assert_eq!(&wire[24..26], &150_i16.to_le_bytes());
    assert_eq!(&wire[26..28], &35_i16.to_le_bytes());
    assert_eq!(PcNanoCreateSuccess0104::decode(&wire), Ok(success));
    assert_eq!(
        decode_pc_nano_create_packet_0104(packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC, &wire),
        Ok(Some(PcNanoCreatePacket0104::Success(success)))
    );

    let failure = PcNanoCreateFailure0104 {
        pc_id: 77,
        error_code: -5,
    };
    let failure_wire = failure.encode();
    assert_eq!(PcNanoCreateFailure0104::decode(&failure_wire), Ok(failure));
    assert_eq!(
        decode_pc_nano_create_packet_0104(
            packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL,
            &failure_wire
        ),
        Ok(Some(PcNanoCreatePacket0104::Failure(failure)))
    );

    for (packet_id, expected) in [
        (
            packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC,
            PcNanoCreateSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL,
            PcNanoCreateFailure0104::SIZE,
        ),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(expected));
        for actual in [expected - 1, expected + 1] {
            assert_eq!(
                decode_pc_nano_create_packet_0104(packet_id, &vec![0x5a; actual]),
                Err(PayloadError::WrongSize { expected, actual })
            );
        }
    }
    assert_eq!(
        decode_pc_nano_create_packet_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn nano_tune_0104_ids_sizes_offsets_and_round_trips_match_clean_abi() {
    assert_eq!(packet::P_CL2FE_REQ_NANO_TUNE, 318_767_120);
    assert_eq!(packet::P_FE2CL_REP_NANO_TUNE_SUCC, 822_083_625);
    assert_eq!(packet::P_FE2CL_REP_NANO_TUNE_FAIL, 822_083_669);
    assert_eq!(NanoTuneRequest0104::SIZE, 44);
    assert_eq!(NanoTuneSuccess0104::SIZE, 168);
    assert_eq!(NanoTuneFailure0104::SIZE, 8);

    let request = NanoTuneRequest0104 {
        nano_id: 36,
        tune_id: 0x1234,
        needed_item_slots: [0, 1, -1, 3, 4, 5, 6, 7, 8, 9],
    };
    let request_wire = request.encode();
    assert_eq!(&request_wire[0..2], &36_i16.to_le_bytes());
    assert_eq!(&request_wire[2..4], &0x1234_i16.to_le_bytes());
    assert_eq!(&request_wire[4..8], &0_i32.to_le_bytes());
    assert_eq!(&request_wire[12..16], &(-1_i32).to_le_bytes());
    assert_eq!(&request_wire[40..44], &9_i32.to_le_bytes());
    assert_eq!(NanoTuneRequest0104::decode(&request_wire), Ok(request));

    let items = std::array::from_fn(|index| ItemBase0104 {
        item_type: index as i16 - 2,
        item_id: 500 + index as i16,
        option: 1_000 + index as i32,
        time_limit: 2_000 + index as i32,
    });
    let success = NanoTuneSuccess0104 {
        nano_id: 36,
        skill_id: 144,
        fusion_matter: 12_345,
        item_slots: [9, 8, 7, 6, 5, 4, 3, 2, 1, -1],
        items,
    };
    let success_wire = success.encode();
    assert_eq!(&success_wire[0..2], &36_i16.to_le_bytes());
    assert_eq!(&success_wire[2..4], &144_i16.to_le_bytes());
    assert_eq!(&success_wire[4..8], &12_345_i32.to_le_bytes());
    assert_eq!(&success_wire[8..12], &9_i32.to_le_bytes());
    assert_eq!(&success_wire[44..48], &(-1_i32).to_le_bytes());
    let mut first_item_wire = [0; ItemBase0104::SIZE];
    items[0].encode_into(&mut first_item_wire);
    let mut last_item_wire = [0; ItemBase0104::SIZE];
    items[9].encode_into(&mut last_item_wire);
    assert_eq!(&success_wire[48..60], &first_item_wire);
    assert_eq!(&success_wire[156..168], &last_item_wire);
    assert_eq!(NanoTuneSuccess0104::decode(&success_wire), Ok(success));

    let failure = NanoTuneFailure0104 {
        pc_id: 77,
        error_code: -5,
    };
    let failure_wire = failure.encode();
    assert_eq!(&failure_wire[0..4], &77_i32.to_le_bytes());
    assert_eq!(&failure_wire[4..8], &(-5_i32).to_le_bytes());
    assert_eq!(NanoTuneFailure0104::decode(&failure_wire), Ok(failure));

    for (packet_id, expected) in [
        (packet::P_CL2FE_REQ_NANO_TUNE, NanoTuneRequest0104::SIZE),
        (
            packet::P_FE2CL_REP_NANO_TUNE_SUCC,
            NanoTuneSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_NANO_TUNE_FAIL,
            NanoTuneFailure0104::SIZE,
        ),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(expected));
    }
}

#[test]
fn nano_skill_use_0104_rejects_malformed_or_oversized_counted_tails() {
    assert!(matches!(
        NanoSkillUseRequest0104::decode(&[0; 19]),
        Err(CountedPayloadError0104::MissingHeader {
            expected_at_least: 20,
            actual: 19,
        })
    ));

    let mut negative = vec![0; NanoSkillUseRequest0104::HEADER_SIZE];
    negative[16..20].copy_from_slice(&(-1_i32).to_le_bytes());
    assert!(matches!(
        NanoSkillUseRequest0104::decode(&negative),
        Err(CountedPayloadError0104::NegativeCount { count: -1 })
    ));

    let too_many = NanoSkillUseRequest0104 {
        bullet_id: 0,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        target_ids: vec![0; NanoSkillUseRequest0104::MAX_TARGETS + 1],
    };
    assert!(matches!(
        too_many.encode(),
        Err(CountedPayloadError0104::CountTooLarge {
            count: 1_018,
            maximum: 1_017,
        })
    ));
}

#[test]
fn nano_active_success_0104_matches_exact_id_layout_and_strict_decoder() {
    assert_eq!(packet::P_FE2CL_REP_NANO_ACTIVE_SUCC, 0x3100_0028);
    let success = NanoActiveSuccess0104 {
        active_nano_slot: 2,
        pack_padding: [0xa5, 0x5a],
        condition_status_add: 1,
    };
    let wire = success.encode();
    assert_eq!(NanoActiveSuccess0104::SIZE, 8);
    assert_eq!(&wire[0..2], &2_i16.to_le_bytes());
    assert_eq!(&wire[2..4], &[0xa5, 0x5a]);
    assert_eq!(&wire[4..8], &1_i32.to_le_bytes());
    assert_eq!(NanoActiveSuccess0104::decode(&wire), Ok(success));
    assert_eq!(
        decode_nano_active_success_0104(packet::P_FE2CL_REP_NANO_ACTIVE_SUCC, &wire),
        Ok(Some(success))
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_NANO_ACTIVE_SUCC),
        Some(NanoActiveSuccess0104::SIZE)
    );
    for actual in [
        NanoActiveSuccess0104::SIZE - 1,
        NanoActiveSuccess0104::SIZE + 1,
    ] {
        assert_eq!(
            decode_nano_active_success_0104(
                packet::P_FE2CL_REP_NANO_ACTIVE_SUCC,
                &vec![0; actual],
            ),
            Err(PayloadError::WrongSize {
                expected: NanoActiveSuccess0104::SIZE,
                actual,
            })
        );
    }
    for invalid_slot in [-2_i16, 3_i16] {
        let mut malformed = wire.clone();
        malformed[0..2].copy_from_slice(&invalid_slot.to_le_bytes());
        assert_eq!(
            decode_nano_active_success_0104(packet::P_FE2CL_REP_NANO_ACTIVE_SUCC, &malformed,),
            Err(PayloadError::ValueOutOfRange {
                field: "active nano slot",
                value: i32::from(invalid_slot),
                minimum: -1,
                maximum: 2,
            })
        );
    }
    assert_eq!(
        decode_nano_active_success_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn nano_skill_success_0104_validates_exact_header_and_every_openfusion_tail_width() {
    assert_eq!(packet::P_FE2CL_NANO_SKILL_USE_SUCC, 0x3100_002b);
    assert_eq!(packet::P_FE2CL_NANO_SKILL_USE, 0x3100_002c);
    assert_eq!(NanoSkillUseSuccessPrefix0104::SIZE, 36);
    assert_eq!(NanoSkillUseSuccess0104::MAX_TARGETS, 101);
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_NANO_SKILL_USE_SUCC),
        None,
        "the eST-dependent success packet must never enter the fixed registry"
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_NANO_SKILL_USE),
        None,
        "the eST-dependent viewer packet must never enter the fixed registry"
    );

    let prefix = NanoSkillUseSuccessPrefix0104 {
        pc_id: 0x1020_3040,
        bullet_id: -3,
        pack_padding: [0xa5],
        skill_id: 8,
        arg1: -1,
        arg2: 0x5060_7080,
        arg3: -2,
        nano_deactivated: 1,
        nano_id: 17,
        nano_stamina: 75,
        skill_type: 8,
        target_count: 2,
    };
    let prefix_wire = prefix.encode_prefix();
    assert_eq!(&prefix_wire[0..4], &0x1020_3040_i32.to_le_bytes());
    assert_eq!(prefix_wire[4], (-3_i8) as u8);
    assert_eq!(prefix_wire[5], 0xa5);
    assert_eq!(&prefix_wire[6..8], &8_i16.to_le_bytes());
    assert_eq!(&prefix_wire[8..12], &(-1_i32).to_le_bytes());
    assert_eq!(&prefix_wire[12..16], &0x5060_7080_i32.to_le_bytes());
    assert_eq!(&prefix_wire[16..20], &(-2_i32).to_le_bytes());
    assert_eq!(&prefix_wire[20..24], &1_i32.to_le_bytes());
    assert_eq!(&prefix_wire[24..26], &17_i16.to_le_bytes());
    assert_eq!(&prefix_wire[26..28], &75_i16.to_le_bytes());
    assert_eq!(&prefix_wire[28..32], &8_i32.to_le_bytes());
    assert_eq!(&prefix_wire[32..36], &2_i32.to_le_bytes());

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
        assert_eq!(nano_skill_result_size_0104(skill_type), Some(record_size));
        let mut expected_prefix = prefix;
        expected_prefix.skill_type = skill_type;
        let mut wire = expected_prefix.encode_prefix();
        wire.resize(NanoSkillUseSuccessPrefix0104::SIZE + 2 * record_size, 0x5a);
        let decoded = NanoSkillUseSuccess0104::decode(&wire).unwrap();
        assert_eq!(decoded.prefix(), expected_prefix);
        assert_eq!(decoded.result_record_size(), record_size);
        assert_eq!(decoded.result_bytes(), vec![0x5a; 2 * record_size]);
        assert_eq!(decoded.results().len(), 2);
        assert_eq!(
            decode_nano_skill_use_packet_0104(packet::P_FE2CL_NANO_SKILL_USE, &wire,),
            Ok(Some(NanoSkillUsePacket0104 {
                delivery: NanoSkillUseDelivery0104::RemoteUse,
                result: decoded.clone(),
            }))
        );
        assert_eq!(
            decode_nano_skill_use_success_0104(packet::P_FE2CL_NANO_SKILL_USE_SUCC, &wire,),
            Ok(Some(decoded))
        );
    }
}

#[test]
fn nano_skill_success_0104_decodes_every_proven_result_family_at_pack4_offsets() {
    let decode = |skill_type: i32, record: Vec<u8>| {
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
            target_count: 1,
        };
        let mut wire = prefix.encode_prefix();
        wire.extend_from_slice(&record);
        NanoSkillUseSuccess0104::decode(&wire).unwrap().results()[0]
    };

    let mut damage = vec![0; 20];
    write_i32(&mut damage, 0, 4);
    write_i32(&mut damage, 4, 9001);
    write_i32(&mut damage, 8, 1);
    write_i32(&mut damage, 12, 125);
    write_i32(&mut damage, 16, 875);
    assert_eq!(
        decode(1, damage),
        NanoSkillResult0104::Damage(NanoSkillDamageResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 4,
                id: 9001,
            },
            protected: 1,
            damage: 125,
            hp: 875,
        })
    );

    let mut heal = vec![0; 16];
    write_i32(&mut heal, 0, 1);
    write_i32(&mut heal, 4, 88);
    write_i32(&mut heal, 8, 60);
    write_i32(&mut heal, 12, 940);
    assert_eq!(
        decode(34, heal),
        NanoSkillResult0104::HealHp(NanoSkillHealHpResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 1,
                id: 88,
            },
            healed_hp: 60,
            hp: 940,
        })
    );

    let mut debuff = vec![0; 32];
    write_i32(&mut debuff, 0, 2);
    write_i32(&mut debuff, 4, 41);
    write_i32(&mut debuff, 8, 0);
    write_i32(&mut debuff, 12, 25);
    write_i32(&mut debuff, 16, 700);
    write_test_i16(&mut debuff, 20, 55);
    debuff[22..24].copy_from_slice(&[0xab, 0xcd]);
    write_i32(&mut debuff, 24, 1);
    write_i32(&mut debuff, 28, 0x400);
    assert_eq!(
        decode(8, debuff),
        NanoSkillResult0104::DamageDebuff(NanoSkillDamageDebuffResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 2,
                id: 41,
            },
            protected: 0,
            damage: 25,
            hp: 700,
            nano_stamina: 55,
            pack_padding: [0xab, 0xcd],
            nano_deactivated: 1,
            condition_bit_flag: 0x400,
        })
    );

    let mut buff = vec![0; 16];
    write_i32(&mut buff, 0, 4);
    write_i32(&mut buff, 4, 42);
    write_i32(&mut buff, 8, 0);
    write_i32(&mut buff, 12, 0x800);
    assert_eq!(
        decode(10, buff),
        NanoSkillResult0104::Buff(NanoSkillBuffResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 4,
                id: 42,
            },
            protected: 0,
            condition_bit_flag: 0x800,
        })
    );

    let mut drain = vec![0; 40];
    write_i32(&mut drain, 0, 1);
    write_i32(&mut drain, 4, 88);
    write_i32(&mut drain, 8, 0);
    write_i32(&mut drain, 12, 10);
    write_i32(&mut drain, 16, 90);
    write_i32(&mut drain, 20, 20);
    write_i32(&mut drain, 24, 180);
    write_test_i16(&mut drain, 28, 45);
    drain[30..32].copy_from_slice(&[0x12, 0x34]);
    write_i32(&mut drain, 32, 1);
    write_i32(&mut drain, 36, 0x1000);
    assert_eq!(
        decode(21, drain),
        NanoSkillResult0104::BatteryDrain(NanoSkillBatteryDrainResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 1,
                id: 88,
            },
            protected: 0,
            drained_weapon_battery: 10,
            weapon_battery: 90,
            drained_nano_battery: 20,
            nano_battery: 180,
            nano_stamina: 45,
            pack_padding: [0x12, 0x34],
            nano_deactivated: 1,
            condition_bit_flag: 0x1000,
        })
    );

    let mut movement = vec![0; 24];
    write_i32(&mut movement, 0, 1);
    write_i32(&mut movement, 4, 88);
    write_i32(&mut movement, 8, 7);
    write_i32(&mut movement, 12, -100);
    write_i32(&mut movement, 16, 200);
    write_i32(&mut movement, 20, 300);
    assert_eq!(
        decode(28, movement),
        NanoSkillResult0104::Move(NanoSkillMoveResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 1,
                id: 88,
            },
            map_number: 7,
            position: [-100, 200, 300],
        })
    );

    let mut resurrect = vec![0; 12];
    write_i32(&mut resurrect, 0, 1);
    write_i32(&mut resurrect, 4, 88);
    write_i32(&mut resurrect, 8, 500);
    assert_eq!(
        decode(26, resurrect),
        NanoSkillResult0104::Resurrect(NanoSkillResurrectResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 1,
                id: 88,
            },
            hp: 500,
        })
    );

    let mut leech = vec![0; 36];
    write_i32(&mut leech, 0, 4);
    write_i32(&mut leech, 4, 9001);
    write_i32(&mut leech, 8, 40);
    write_i32(&mut leech, 12, 640);
    write_i32(&mut leech, 16, 4);
    write_i32(&mut leech, 20, 9001);
    write_i32(&mut leech, 24, 0);
    write_i32(&mut leech, 28, 80);
    write_i32(&mut leech, 32, 120);
    assert_eq!(
        decode(30, leech),
        NanoSkillResult0104::Leech(NanoSkillLeechResult0104 {
            heal: NanoSkillHealHpResult0104 {
                target: NanoSkillTarget0104 {
                    entity_type: 4,
                    id: 9001,
                },
                healed_hp: 40,
                hp: 640,
            },
            damage: NanoSkillDamageResult0104 {
                target: NanoSkillTarget0104 {
                    entity_type: 4,
                    id: 9001,
                },
                protected: 0,
                damage: 80,
                hp: 120,
            },
        })
    );
}
