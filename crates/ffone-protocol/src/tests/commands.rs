use super::*;

#[test]
fn login_request_has_exact_pack4_offsets() {
    let packet = LoginRequest::password_login("Ambfibia", "ambfibia12", 1, 0, 44).unwrap();
    let bytes = packet.encode();
    assert_eq!(bytes.len(), 468);
    assert_eq!(&bytes[..18], b"A\0m\0b\0f\0i\0b\0i\0a\0\0\0");
    assert_eq!(read_i32(&bytes, 132), 1);
    assert_eq!(read_i32(&bytes, 136), 0);
    assert_eq!(read_i32(&bytes, 140), 44);
    assert_eq!(read_i32(&bytes, 144), 1);
    assert_eq!(bytes[467], 0);
    assert_eq!(LoginRequest::decode(&bytes).unwrap(), packet);
}

#[test]
fn attack_request_and_results_use_exact_counted_trailers() {
    let request = PcAttackNpcsRequest0104 {
        npc_ids: vec![10, 20, 30],
    };
    assert_eq!(
        request.encode().unwrap(),
        vec![3, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0]
    );
    assert_eq!(
        PcAttackNpcsRequest0104::decode(&request.encode().unwrap()),
        Ok(request)
    );
    assert_eq!(
        PcAttackNpcsRequest0104 {
            npc_ids: vec![1, 2, 3, 4]
        }
        .encode(),
        Err(CountedPayloadError0104::CountTooLarge {
            count: 4,
            maximum: 3,
        })
    );

    let result_a = AttackResult0104 {
        entity_type: 2,
        id: 101,
        protected: 0,
        damage: 25,
        hp: 75,
        hit_flag: 1,
    };
    let result_b = AttackResult0104 {
        entity_type: 2,
        id: 202,
        protected: 1,
        damage: -2,
        hp: 300,
        hit_flag: 2,
    };
    let broadcast = PcAttackNpcs0104 {
        pc_id: 77,
        results: vec![result_a, result_b],
    };
    let encoded = broadcast.encode().unwrap();
    assert_eq!(encoded.len(), 8 + 2 * AttackResult0104::SIZE);
    assert_eq!(&encoded[4..8], &2i32.to_le_bytes());
    assert_eq!(&encoded[8..32], result_a.encode());
    assert_eq!(&encoded[32..56], result_b.encode());
    assert_eq!(PcAttackNpcs0104::decode(&encoded), Ok(broadcast));
}

#[test]
fn pvp_attack_request_and_results_use_clean_counted_trailers() {
    assert_eq!(packet::P_CL2FE_REQ_PC_ATTACK_CHARS, 0x1300_0090);
    assert_eq!(packet::P_FE2CL_PC_ATTACK_CHARS_SUCC, 0x3100_0102);
    assert_eq!(packet::P_FE2CL_PC_ATTACK_CHARS, 0x3100_0103);
    assert_eq!(packet::P_FE2CL_NPC_ATTACK_CHARS, 0x3100_0104);
    assert_eq!(packet::P_FE2CL_CHARACTER_ATTACK_CHARACTERS, 0x3100_0085);
    for id in [
        packet::P_FE2CL_PC_ATTACK_CHARS_SUCC,
        packet::P_FE2CL_PC_ATTACK_CHARS,
        packet::P_FE2CL_NPC_ATTACK_CHARS,
        packet::P_FE2CL_CHARACTER_ATTACK_CHARACTERS,
    ] {
        assert_eq!(
            fixed_payload_size(id),
            None,
            "{id:#x} carries a counted trailer"
        );
    }

    // cnAvatarAttack writes Status.iID first and the entity kind second.
    let request = PcAttackCharsRequest0104 {
        targets: vec![
            PcAttackCharsTarget0104 {
                id: 10,
                entity_type: PcAttackCharsTarget0104::NPC_ENTITY_TYPE,
            },
            PcAttackCharsTarget0104 {
                id: 20,
                entity_type: PcAttackCharsTarget0104::PC_ENTITY_TYPE,
            },
        ],
    };
    let wire = request.encode().unwrap();
    assert_eq!(
        wire,
        vec![2, 0, 0, 0, 10, 0, 0, 0, 4, 0, 0, 0, 20, 0, 0, 0, 1, 0, 0, 0]
    );
    assert_eq!(PcAttackCharsRequest0104::decode(&wire), Ok(request));
    assert_eq!(
        registered_request_layout_0104(packet::P_CL2FE_REQ_PC_ATTACK_CHARS),
        Some(RegisteredRequestLayout0104::Counted {
            base_size: 4,
            count_offset: 0,
            element_size: 8,
            maximum_count: PcAttackCharsRequest0104::MAX_TARGETS,
        })
    );
    assert!(matches!(
        PcAttackCharsRequest0104::decode(&wire[..12]),
        Err(CountedPayloadError0104::WrongSize { .. })
    ));

    let npc_hit = AttackResult0104 {
        entity_type: 4,
        id: 101,
        protected: 0,
        damage: 25,
        hp: 75,
        hit_flag: 1,
    };
    let pc_hit = AttackResult0104 {
        entity_type: 1,
        id: 7,
        protected: 0,
        damage: 40,
        hp: 960,
        hit_flag: 2,
    };

    let success = PcAttackCharsSuccess0104 {
        battery_w: 1234,
        results: vec![npc_hit, pc_hit],
    };
    let encoded = success.encode().unwrap();
    assert_eq!(encoded.len(), 8 + 2 * AttackResult0104::SIZE);
    assert_eq!(&encoded[0..4], &1234i32.to_le_bytes());
    assert_eq!(&encoded[4..8], &2i32.to_le_bytes());
    assert_eq!(
        PcAttackCharsSuccess0104::decode(&encoded),
        Ok(success.clone())
    );
    assert_eq!(
        decode_npc_combat_packet_0104(packet::P_FE2CL_PC_ATTACK_CHARS_SUCC, &encoded),
        Ok(Some(NpcCombatPacket0104::PcAttackCharsSuccess(success)))
    );

    let broadcast = PcAttackChars0104 {
        pc_id: 77,
        results: vec![npc_hit],
    };
    assert_eq!(
        PcAttackChars0104::decode(&broadcast.encode().unwrap()),
        Ok(broadcast)
    );

    let npc_attack = NpcAttackChars0104 {
        npc_id: 314,
        results: vec![pc_hit, npc_hit],
    };
    assert_eq!(
        NpcAttackChars0104::decode(&npc_attack.encode().unwrap()),
        Ok(npc_attack)
    );

    let character_attack = CharacterAttackCharacters0104 {
        entity_type: 4,
        character_id: 555,
        results: vec![npc_hit],
    };
    let encoded = character_attack.encode().unwrap();
    assert_eq!(encoded.len(), 12 + AttackResult0104::SIZE);
    assert_eq!(&encoded[0..4], &4i32.to_le_bytes());
    assert_eq!(&encoded[4..8], &555i32.to_le_bytes());
    assert_eq!(&encoded[8..12], &1i32.to_le_bytes());
    assert_eq!(&encoded[12..36], npc_hit.encode());
    assert_eq!(
        CharacterAttackCharacters0104::decode(&encoded),
        Ok(character_attack)
    );
    assert!(matches!(
        CharacterAttackCharacters0104::decode(&encoded[..30]),
        Err(CountedPayloadError0104::WrongSize { .. })
    ));
    let mut negative = encoded.clone();
    write_i32(&mut negative, 8, -1);
    assert_eq!(
        CharacterAttackCharacters0104::decode(&negative),
        Err(CountedPayloadError0104::NegativeCount { count: -1 })
    );
}

#[test]
fn gm_speed_set_value_0104_matches_clean_pack4_request_and_reply() {
    assert_eq!(packet::P_CL2FE_GM_REQ_PC_SET_VALUE, 0x1300_006b);
    assert_eq!(packet::P_FE2CL_GM_REP_PC_SET_VALUE, 0x3100_00c5);
    assert_eq!(GM_SET_VALUE_SPEED_0104, 6);
    assert_eq!(GmSetValueRequest0104::SIZE, 12);
    assert_eq!(GmSetValueReply0104::SIZE, 12);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_GM_REQ_PC_SET_VALUE),
        Some(12)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_GM_REP_PC_SET_VALUE),
        Some(12)
    );

    let request = GmSetValueRequest0104::speed(0x0102_0304, 1_200);
    let request_bytes = request.encode();
    assert_eq!(&request_bytes[0..4], &0x0102_0304i32.to_le_bytes());
    assert_eq!(&request_bytes[4..8], &6i32.to_le_bytes());
    assert_eq!(&request_bytes[8..12], &1_200i32.to_le_bytes());
    assert_eq!(GmSetValueRequest0104::decode(&request_bytes), Ok(request));

    let reply = GmSetValueReply0104 {
        pc_id: 0x0102_0304,
        value_type: GM_SET_VALUE_SPEED_0104,
        value: 1_200,
    };
    let reply_bytes = reply.encode();
    assert_eq!(GmSetValueReply0104::decode(&reply_bytes), Ok(reply));
    assert_eq!(
        decode_gm_set_value_reply_0104(packet::P_FE2CL_GM_REP_PC_SET_VALUE, &reply_bytes),
        Ok(Some(reply))
    );
    assert_eq!(decode_gm_set_value_reply_0104(0x3100_0abc, &[1]), Ok(None));
    assert_eq!(
        decode_gm_set_value_reply_0104(packet::P_FE2CL_GM_REP_PC_SET_VALUE, &reply_bytes[..11],),
        Err(PayloadError::WrongSize {
            expected: 12,
            actual: 11,
        })
    );
}

#[test]
fn quick_slot_0104_item_use_request_and_failure_are_strict() {
    let request = ItemUseRequest0104 {
        item_location: 3,
        slot_num: 0x1020_3040,
        nano_slot: -9,
    };
    let bytes = request.encode();
    assert_eq!(bytes.len(), 12);
    assert_eq!(&bytes[0..4], &3i32.to_le_bytes());
    assert_eq!(&bytes[4..8], &0x1020_3040i32.to_le_bytes());
    assert_eq!(&bytes[8..10], &(-9i16).to_le_bytes());
    assert_eq!(&bytes[10..12], &[0, 0], "pack(4) tail padding");
    assert_eq!(ItemUseRequest0104::decode(&bytes), Ok(request));
    assert_eq!(
        decode_quick_slot_packet_0104(packet::P_CL2FE_REQ_ITEM_USE, &bytes),
        Ok(Some(QuickSlotPacket0104::ItemUseRequest(request)))
    );
    assert_eq!(
        ItemUseRequest0104::decode(&bytes[..11]),
        Err(PayloadError::WrongSize {
            expected: ItemUseRequest0104::SIZE,
            actual: 11,
        })
    );

    let failure = ItemUseFailure0104 { error_code: 42 };
    assert_eq!(
        decode_quick_slot_packet_0104(packet::P_FE2CL_REP_PC_ITEM_USE_FAIL, &failure.encode(),),
        Ok(Some(QuickSlotPacket0104::ItemUseFailure(failure)))
    );
    assert_eq!(
        ItemUseFailure0104::decode(&[0; 5]),
        Err(PayloadError::WrongSize {
            expected: ItemUseFailure0104::SIZE,
            actual: 5,
        })
    );
}

#[test]
fn item_chest_open_0104_request_and_replies_are_strict() {
    let request = ItemChestOpenRequest0104 {
        item_location: 1,
        slot_num: 17,
        chest_item: ItemBase0104 {
            item_type: 9,
            item_id: 424,
            option: 3,
            time_limit: 0,
        },
    };
    let bytes = request.encode();
    assert_eq!(bytes.len(), ItemChestOpenRequest0104::SIZE);
    assert_eq!(&bytes[0..4], &1i32.to_le_bytes());
    assert_eq!(&bytes[4..8], &17i32.to_le_bytes());
    assert_eq!(&bytes[8..10], &9i16.to_le_bytes());
    assert_eq!(&bytes[10..12], &424i16.to_le_bytes());
    assert_eq!(&bytes[12..16], &3i32.to_le_bytes());
    assert_eq!(&bytes[16..20], &0i32.to_le_bytes());
    assert_eq!(ItemChestOpenRequest0104::decode(&bytes), Ok(request));
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_ITEM_CHEST_OPEN),
        Some(ItemChestOpenRequest0104::SIZE)
    );

    let success = ItemChestOpenSuccess0104 { slot_num: 17 };
    assert_eq!(
        ItemChestOpenSuccess0104::decode(&success.encode()),
        Ok(success)
    );
    let failure = ItemChestOpenFailure0104 {
        slot_num: 17,
        error_code: 2,
    };
    assert_eq!(
        ItemChestOpenFailure0104::decode(&failure.encode()),
        Ok(failure)
    );
}

#[test]
fn group_leave_request_0104_matches_clean_one_byte_abi() {
    assert_eq!(packet::P_CL2FE_REQ_PC_GROUP_LEAVE, 318_767_183);
    assert_eq!(packet::P_CL2FE_REQ_PC_GROUP_LEAVE, 0x1300_004f);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_GROUP_LEAVE),
        Some(GroupLeaveRequest0104::SIZE)
    );

    let request = GroupLeaveRequest0104 { unused: 0 };
    assert_eq!(request.encode(), vec![0]);
    assert_eq!(GroupLeaveRequest0104::decode(&[0]), Ok(request));
    assert_eq!(
        GroupLeaveRequest0104::decode(&[]),
        Err(PayloadError::WrongSize {
            expected: 1,
            actual: 0,
        })
    );
    assert_eq!(
        GroupLeaveRequest0104::decode(&[0, 0]),
        Err(PayloadError::WrongSize {
            expected: 1,
            actual: 2,
        })
    );
}

#[test]
fn buddy_lifecycle_0104_request_success_and_warp_layouts_are_exact() {
    let success = BuddyMakeSuccess0104 {
        request_id: 77,
        buddy_id: 81,
        buddy_pc_uid: 0x0102_0304_0506_0708,
    };
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC,
            &success.encode(),
        ),
        Ok(Some(BuddyLifecyclePacket0104::MakeSuccess(success)))
    );

    let other_shard = BuddyWarpOtherShardSuccess0104 {
        buddy_pc_uid: 0x1112_1314_1516_1718,
        shard_num: -3,
        channel_num: 27,
    };
    let bytes = other_shard.encode();
    assert_eq!(bytes.len(), 16);
    assert_eq!(bytes[8], (-3i8) as u8);
    assert_eq!(&bytes[9..12], &[0, 0, 0]);
    assert_eq!(&bytes[12..16], &27i32.to_le_bytes());
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC,
            &bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::WarpOtherShardSuccess(
            other_shard
        )))
    );
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC,
            &[0x5a],
        ),
        Ok(Some(BuddyLifecyclePacket0104::WarpSameShardSuccess(
            BuddyWarpSameShardSuccess0104 { unused: 0x5a }
        )))
    );
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC,
            &[0; 15],
        ),
        Err(PayloadError::WrongSize {
            expected: 16,
            actual: 15,
        })
    );
}

#[test]
fn vendor_0104_decoder_is_strict_for_every_known_reply_and_passes_unknown() {
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 42,
        option: 3,
        time_limit: 0,
    };
    let table_item = ItemVendor0104 {
        vendor_id: 650,
        buy_cost: 700.25,
        item,
        sort_num: 8,
    };
    let bodies = vec![
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC,
            VendorItemBuySuccess0104 {
                candy: 9_000,
                inventory_slot: 4,
                item,
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_FAIL,
            VendorFailure0104 { error_code: 1 }.encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC,
            VendorItemSellSuccess0104 {
                candy: 9_500,
                inventory_slot: 5,
                item,
                item_stay: ItemBase0104 { option: 2, ..item },
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_FAIL,
            VendorFailure0104 { error_code: 2 }.encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC,
            PcItemDeleteSuccess0104 {
                item_location: 1,
                slot_num: 6,
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_START_SUCC,
            VendorStartSuccess0104 {
                npc_id: 9001,
                vendor_id: 650,
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_START_FAIL,
            VendorFailure0104 { error_code: 3 }.encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC,
            VendorTableUpdateSuccess0104 {
                items: [table_item; VENDOR_TABLE_ITEM_COUNT_0104],
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_FAIL,
            VendorFailure0104 { error_code: 4 }.encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC,
            VendorItemRestoreBuySuccess0104 {
                candy: 8_000,
                inventory_slot: 7,
                item,
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_FAIL,
            VendorFailure0104 { error_code: 5 }.encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC,
            VendorBatteryBuySuccess0104 {
                candy: 7_000,
                weapon_battery: 100,
                nano_battery: 200,
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_FAIL,
            VendorFailure0104 { error_code: 6 }.encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC,
            PcDisassembleItemSuccess0104 {
                new_item_slot: 8,
                new_item: item,
            }
            .encode(),
        ),
        (
            packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL,
            PcDisassembleItemFailure0104 {
                error_code: 7,
                item_slot: 8,
            }
            .encode(),
        ),
    ];

    for (packet_id, body) in bodies {
        assert!(
            decode_vendor_packet_0104(packet_id, &body)
                .unwrap()
                .is_some()
        );
        for malformed in [body[..body.len() - 1].to_vec(), {
            let mut trailing = body.clone();
            trailing.push(0);
            trailing
        }] {
            assert_eq!(
                decode_vendor_packet_0104(packet_id, &malformed),
                Err(PayloadError::WrongSize {
                    expected: body.len(),
                    actual: malformed.len(),
                })
            );
        }
    }
    assert_eq!(
        decode_vendor_packet_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn nano_world_action_0104_ids_sizes_offsets_and_round_trips_match_openfusion_abi() {
    assert_eq!(packet::P_CL2FE_REQ_NANO_EQUIP, 0x1300_000d);
    assert_eq!(packet::P_CL2FE_REQ_NANO_UNEQUIP, 0x1300_000e);
    assert_eq!(packet::P_CL2FE_REQ_NANO_ACTIVE, 0x1300_000f);
    assert_eq!(packet::P_CL2FE_REQ_NANO_SKILL_USE, 0x1300_0011);
    assert_eq!(packet::P_CL2FE_REQ_PC_VEHICLE_ON, 0x1300_009f);
    assert_eq!(packet::P_CL2FE_REQ_PC_VEHICLE_OFF, 0x1300_00a0);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_ON_SUCC, 0x3100_0122);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_ON_FAIL, 0x3100_0123);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_OFF_SUCC, 0x3100_0124);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_OFF_FAIL, 0x3100_0125);

    let equip = NanoEquipRequest0104 {
        nano_id: 36,
        nano_slot: 2,
    };
    let unequip = NanoUnequipRequest0104 { nano_slot: 1 };
    let active = NanoActiveRequest0104 { nano_slot: -1 };
    let vehicle_on = PcVehicleOnRequest0104 { unused: 0x5a };
    let vehicle_off = PcVehicleOffRequest0104 { unused: 0xa5 };

    for (packet_id, expected) in [
        (packet::P_CL2FE_REQ_NANO_EQUIP, NanoEquipRequest0104::SIZE),
        (
            packet::P_CL2FE_REQ_NANO_UNEQUIP,
            NanoUnequipRequest0104::SIZE,
        ),
        (packet::P_CL2FE_REQ_NANO_ACTIVE, NanoActiveRequest0104::SIZE),
        (
            packet::P_CL2FE_REQ_PC_VEHICLE_ON,
            PcVehicleOnRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_VEHICLE_OFF,
            PcVehicleOffRequest0104::SIZE,
        ),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(expected));
        assert_eq!(
            registered_request_layout_0104(packet_id),
            Some(RegisteredRequestLayout0104::Fixed { size: expected })
        );
    }
    assert_eq!(NanoEquipRequest0104::SIZE, 4);
    assert_eq!(NanoUnequipRequest0104::SIZE, 2);
    assert_eq!(NanoActiveRequest0104::SIZE, 2);
    assert_eq!(PcVehicleOnRequest0104::SIZE, 1);
    assert_eq!(PcVehicleOffRequest0104::SIZE, 1);
    assert_eq!(NanoEquipRequest0104::decode(&equip.encode()), Ok(equip));
    assert_eq!(
        NanoUnequipRequest0104::decode(&unequip.encode()),
        Ok(unequip)
    );
    assert_eq!(NanoActiveRequest0104::decode(&active.encode()), Ok(active));
    assert_eq!(
        PcVehicleOnRequest0104::decode(&vehicle_on.encode()),
        Ok(vehicle_on)
    );
    assert_eq!(
        PcVehicleOffRequest0104::decode(&vehicle_off.encode()),
        Ok(vehicle_off)
    );
    for (packet_id, size) in [
        (packet::P_FE2CL_PC_VEHICLE_ON_SUCC, 1),
        (packet::P_FE2CL_PC_VEHICLE_ON_FAIL, 4),
        (packet::P_FE2CL_PC_VEHICLE_OFF_SUCC, 1),
        (packet::P_FE2CL_PC_VEHICLE_OFF_FAIL, 4),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(size));
    }

    let skill = NanoSkillUseRequest0104 {
        bullet_id: -3,
        arg1: 0x1020_3040,
        arg2: -2,
        arg3: 0x5060_7080,
        target_ids: vec![81, -1, 0x1234_5678],
    };
    let wire = skill.encode().unwrap();
    assert_eq!(NanoSkillUseRequest0104::HEADER_SIZE, 20);
    assert_eq!(NanoSkillUseRequest0104::MAX_TARGETS, 1_017);
    assert_eq!(wire.len(), 32);
    assert_eq!(wire[0], (-3_i8) as u8);
    assert_eq!(&wire[1..4], &[0, 0, 0]);
    assert_eq!(&wire[4..8], &0x1020_3040_i32.to_le_bytes());
    assert_eq!(&wire[8..12], &(-2_i32).to_le_bytes());
    assert_eq!(&wire[12..16], &0x5060_7080_i32.to_le_bytes());
    assert_eq!(&wire[16..20], &3_i32.to_le_bytes());
    assert_eq!(&wire[20..24], &81_i32.to_le_bytes());
    assert_eq!(&wire[24..28], &(-1_i32).to_le_bytes());
    assert_eq!(&wire[28..32], &0x1234_5678_i32.to_le_bytes());
    assert_eq!(NanoSkillUseRequest0104::decode(&wire), Ok(skill.clone()));
    assert_eq!(
        registered_request_layout_0104(packet::P_CL2FE_REQ_NANO_SKILL_USE),
        Some(RegisteredRequestLayout0104::Counted {
            base_size: NanoSkillUseRequest0104::HEADER_SIZE,
            count_offset: 16,
            element_size: 4,
            maximum_count: NanoSkillUseRequest0104::MAX_TARGETS,
        })
    );
    assert!(
        RegisteredGameplayRequest0104::new(packet::P_CL2FE_REQ_NANO_SKILL_USE, wire).is_ok()
    );
}
