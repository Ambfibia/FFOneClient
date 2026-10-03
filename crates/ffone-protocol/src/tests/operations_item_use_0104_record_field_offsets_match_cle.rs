use super::*;

#[test]
fn server_message_0104_matches_clean_pack2_layout_and_is_strict() {
    assert_eq!(packet::P_FE2CL_PC_MOTD_LOGIN, 0x3100_00d1);
    assert_eq!(ServerMessage0104::SIZE, 1026);
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_PC_MOTD_LOGIN),
        Some(ServerMessage0104::SIZE)
    );

    let message = ServerMessage0104 {
        message_type: 1,
        message: FixedUtf16::from_str("Commands available: /help /population").unwrap(),
    };
    let bytes = message.encode();
    assert_eq!(bytes.len(), 1026);
    assert_eq!(bytes[0], 1);
    assert_eq!(bytes[1], 0);
    assert_eq!(&bytes[2..4], &('C' as u16).to_le_bytes());
    assert_eq!(ServerMessage0104::decode(&bytes), Ok(message.clone()));
    assert_eq!(
        decode_server_message_0104(packet::P_FE2CL_PC_MOTD_LOGIN, &bytes),
        Ok(Some(message))
    );
    assert_eq!(decode_server_message_0104(0x3100_0abc, &[1, 2]), Ok(None));
    assert_eq!(
        decode_server_message_0104(packet::P_FE2CL_PC_MOTD_LOGIN, &bytes[..1025]),
        Err(PayloadError::WrongSize {
            expected: 1026,
            actual: 1025,
        })
    );
}

#[test]
fn buddy_freechat_0104_payloads_preserve_exact_uid_slot_and_offsets() {
    const MESSAGE: &str =
        "\u{0414}\u{0440}\u{0443}\u{0433} \u{0432} \u{0441}\u{0435}\u{0442}\u{0438} \u{1f680}";

    assert_eq!(packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE, 0x1300_0037);
    assert_eq!(
        packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC,
        0x3100_0069
    );
    assert_eq!(BuddyFreeChatRequest0104::SIZE, 272);
    assert_eq!(BuddyFreeChatSuccess0104::SIZE, 276);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE),
        Some(272)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC),
        Some(276)
    );

    let message = FixedUtf16::<128>::from_str(MESSAGE).unwrap();
    let request = BuddyFreeChatRequest0104 {
        message: message.clone(),
        emote_code: -23,
        buddy_pc_uid: 0x0102_0304_0506_0708,
        buddy_slot: -7,
    };
    let request_bytes = request.encode();
    assert_eq!(request_bytes.len(), 272);
    assert_eq!(&request_bytes[256..260], &(-23i32).to_le_bytes());
    assert_eq!(
        &request_bytes[260..268],
        &0x0102_0304_0506_0708i64.to_le_bytes()
    );
    assert_eq!(request_bytes[268], (-7i8) as u8);
    assert_eq!(&request_bytes[269..272], &[0, 0, 0]);
    assert_eq!(
        BuddyFreeChatRequest0104::decode(&request_bytes),
        Ok(request.clone())
    );
    assert_eq!(
        decode_buddy_freechat_packet_0104(
            packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE,
            &request_bytes,
        ),
        Ok(Some(BuddyFreeChatPacket0104::Request(request)))
    );

    let success = BuddyFreeChatSuccess0104 {
        from_pc_uid: 0x0102_0304_0506_0708,
        to_pc_uid: 0x1112_1314_1516_1718,
        message,
        emote_code: 41,
    };
    let success_bytes = success.encode();
    assert_eq!(success_bytes.len(), 276);
    assert_eq!(
        &success_bytes[0..8],
        &0x0102_0304_0506_0708i64.to_le_bytes()
    );
    assert_eq!(
        &success_bytes[8..16],
        &0x1112_1314_1516_1718i64.to_le_bytes()
    );
    assert_eq!(&success_bytes[272..276], &41i32.to_le_bytes());
    assert_eq!(
        BuddyFreeChatSuccess0104::decode(&success_bytes),
        Ok(success.clone())
    );
    assert_eq!(success.message.to_string_lossy(), MESSAGE);
    assert_eq!(
        decode_buddy_freechat_packet_0104(
            packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC,
            &success_bytes,
        ),
        Ok(Some(BuddyFreeChatPacket0104::Success(success)))
    );
}

#[test]
fn buddy_freechat_0104_decoder_is_strict_and_ignores_other_chat_families() {
    assert_eq!(
        decode_buddy_freechat_packet_0104(
            packet::P_CL2FE_REQ_PC_FREECHAT,
            &[0; FreeChatRequest0104::SIZE],
        ),
        Ok(None)
    );
    assert_eq!(
        decode_buddy_freechat_packet_0104(
            packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE,
            &[0; AllGroupFreeChatRequest0104::SIZE],
        ),
        Ok(None)
    );
    assert_eq!(
        decode_buddy_freechat_packet_0104(
            packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE,
            &[0; 271],
        ),
        Err(PayloadError::WrongSize {
            expected: 272,
            actual: 271,
        })
    );
    assert_eq!(
        decode_buddy_freechat_packet_0104(
            packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC,
            &[0; 277],
        ),
        Err(PayloadError::WrongSize {
            expected: 276,
            actual: 277,
        })
    );
    assert_eq!(
        decode_buddy_freechat_packet_0104(0x3100_0abc, &[0xaa, 0xbb, 0xcc]),
        Ok(None)
    );
}

#[test]
fn all_group_freechat_0104_unicode_payloads_have_exact_source_layouts() {
    const MESSAGE: &str = "\u{0413}\u{0440}\u{0443}\u{043f}\u{043f}\u{0430} \u{1f680}";

    assert_eq!(
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE,
        0x1300_0063
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC,
        0x3100_00b5
    );
    assert_eq!(AllGroupFreeChatRequest0104::SIZE, 260);
    assert_eq!(AllGroupFreeChatSuccess0104::SIZE, 264);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE),
        Some(260)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC),
        Some(264)
    );

    let message = FixedUtf16::<128>::from_str(MESSAGE).unwrap();
    let request = AllGroupFreeChatRequest0104 {
        message: message.clone(),
        emote_code: -9,
    };
    let request_bytes = request.encode();
    assert_eq!(request_bytes.len(), 260);
    assert_eq!(&request_bytes[256..260], &(-9i32).to_le_bytes());
    assert_eq!(
        AllGroupFreeChatRequest0104::decode(&request_bytes),
        Ok(request)
    );

    let success = AllGroupFreeChatSuccess0104 {
        sender_pc_id: 0x1020_3040,
        message,
        emote_code: 31,
    };
    let success_bytes = success.encode();
    assert_eq!(success_bytes.len(), 264);
    assert_eq!(&success_bytes[..4], &0x1020_3040i32.to_le_bytes());
    assert_eq!(&success_bytes[260..264], &31i32.to_le_bytes());
    assert_eq!(
        AllGroupFreeChatSuccess0104::decode(&success_bytes),
        Ok(success.clone())
    );
    assert_eq!(success.message.to_string_lossy(), MESSAGE);
    assert_eq!(
        decode_all_group_freechat_packet_0104(
            packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC,
            &success_bytes,
        ),
        Ok(Some(AllGroupFreeChatPacket0104::Success(success)))
    );
}

#[test]
fn all_group_freechat_0104_decoder_is_strict_and_ignores_other_chat_families() {
    assert_eq!(
        decode_all_group_freechat_packet_0104(
            packet::P_CL2FE_REQ_PC_FREECHAT,
            &[0; FreeChatRequest0104::SIZE],
        ),
        Ok(None)
    );
    assert_eq!(
        decode_all_group_freechat_packet_0104(
            packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE,
            &[0; 259],
        ),
        Err(PayloadError::WrongSize {
            expected: 260,
            actual: 259,
        })
    );
    assert_eq!(
        decode_all_group_freechat_packet_0104(
            packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC,
            &[0; 265],
        ),
        Err(PayloadError::WrongSize {
            expected: 264,
            actual: 265,
        })
    );
}

#[test]
fn quick_slot_0104_roster_and_registration_use_exact_pack_layouts() {
    let info = QuickSlotInfo0104 {
        slots: std::array::from_fn(|index| QuickSlotEntry0104 {
            item_type: 100 + index as i16,
            item_id: -200 - index as i16,
        }),
    };
    let bytes = info.encode();
    assert_eq!(bytes.len(), 32);
    assert_eq!(&bytes[0..2], &100i16.to_le_bytes());
    assert_eq!(&bytes[2..4], &(-200i16).to_le_bytes());
    assert_eq!(&bytes[28..30], &107i16.to_le_bytes());
    assert_eq!(&bytes[30..32], &(-207i16).to_le_bytes());
    assert_eq!(QuickSlotInfo0104::decode(&bytes), Ok(info));
    for actual in [31, 33] {
        assert_eq!(
            QuickSlotInfo0104::decode(&vec![0; actual]),
            Err(PayloadError::WrongSize {
                expected: QuickSlotInfo0104::SIZE,
                actual,
            })
        );
    }

    let request = QuickSlotRegisterRequest0104 {
        slot_num: 7,
        item_type: 10,
        item_id: 321,
    };
    assert_eq!(request.encode(), vec![7, 0, 0, 0, 10, 0, 0x41, 0x01]);
    assert_eq!(
        decode_quick_slot_packet_0104(
            packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT,
            &request.encode(),
        ),
        Ok(Some(QuickSlotPacket0104::RegisterRequest(request)))
    );

    let success = QuickSlotRegisterSuccess0104 {
        slot_num: request.slot_num,
        item_type: request.item_type,
        item_id: request.item_id,
    };
    assert_eq!(
        decode_quick_slot_packet_0104(
            packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC,
            &success.encode(),
        ),
        Ok(Some(QuickSlotPacket0104::RegisterSuccess(success)))
    );
    assert_eq!(
        decode_quick_slot_packet_0104(
            packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL,
            &(-17i32).to_le_bytes(),
        ),
        Ok(Some(QuickSlotPacket0104::RegisterFailure(
            QuickSlotRegisterFailure0104 { error_code: -17 }
        )))
    );
    assert_eq!(
        decode_quick_slot_packet_0104(packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC, &[0; 7],),
        Err(PayloadError::WrongSize {
            expected: QuickSlotRegisterSuccess0104::SIZE,
            actual: 7,
        })
    );
}

#[test]
fn item_use_0104_buff_success_and_damage_broadcast_are_typed_losslessly() {
    let success = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 1,
        slot_num: 5,
        remaining_item: ItemBase0104 {
            item_type: 7,
            item_id: 321,
            option: 9,
            time_limit: 1_234,
        },
        skill_id: -8,
        pack_padding: [0xaa, 0xbb],
        skill_type: 33,
        target_count: 2,
    };
    let mut success_payload = success.encode_prefix();
    assert_eq!(success_payload.len(), 36);
    assert_eq!(&success_payload[12..14], &7i16.to_le_bytes());
    assert_eq!(&success_payload[14..16], &321i16.to_le_bytes());
    assert_eq!(&success_payload[16..20], &9i32.to_le_bytes());
    assert_eq!(&success_payload[20..24], &1_234i32.to_le_bytes());
    assert_eq!(&success_payload[24..26], &(-8i16).to_le_bytes());
    assert_eq!(&success_payload[26..28], &[0xaa, 0xbb]);
    let buffs = [
        SkillResultBuff0104 {
            character_type: 1,
            character_id: 77,
            protected: 0,
            condition_bit_flag: 0x1020_3040,
        },
        SkillResultBuff0104 {
            character_type: 2,
            character_id: 88,
            protected: 1,
            condition_bit_flag: -7,
        },
    ];
    for buff in buffs {
        append_skill_result_buff(&mut success_payload, buff);
    }
    assert_eq!(
        ItemUseSuccessPrefix0104::decode_prefix(&success_payload),
        Ok(success)
    );
    assert_eq!(
        ItemUseSuccessPacket0104::decode(&success_payload),
        Ok(ItemUseSuccessPacket0104 {
            prefix: success,
            results: ItemUseSkillResults0104::Buff(buffs.to_vec()),
        })
    );
    assert_eq!(
        decode_item_use_packet_0104(packet::P_FE2CL_REP_PC_ITEM_USE_SUCC, &success_payload,),
        Ok(Some(ItemUsePacket0104::Success(ItemUseSuccessPacket0104 {
            prefix: success,
            results: ItemUseSkillResults0104::Buff(buffs.to_vec()),
        })))
    );
    assert_eq!(
        decode_quick_slot_packet_0104(packet::P_FE2CL_REP_PC_ITEM_USE_SUCC, &success_payload,),
        Ok(None),
        "the legacy QuickSlot classifier remains fixed-layout only"
    );
    assert_eq!(
        ItemUseSuccessPrefix0104::decode_prefix(&success_payload[..35]),
        Err(PayloadError::WrongSize {
            expected: ItemUseSuccessPrefix0104::SIZE,
            actual: 35,
        })
    );

    let broadcast = ItemUseBroadcastPrefix0104 {
        pc_id: 81,
        skill_id: 22,
        pack_padding: [0xcc, 0xdd],
        skill_type: 1,
        target_count: 1,
    };
    let mut broadcast_payload = broadcast.encode_prefix();
    assert_eq!(&broadcast_payload[4..6], &22i16.to_le_bytes());
    assert_eq!(&broadcast_payload[6..8], &[0xcc, 0xdd]);
    let damage = SkillResultDamage0104 {
        character_type: 4,
        character_id: 99,
        protected: 1,
        damage: 250,
        hp: 750,
    };
    append_skill_result_damage(&mut broadcast_payload, damage);
    assert_eq!(
        ItemUseBroadcastPrefix0104::decode_prefix(&broadcast_payload),
        Ok(broadcast)
    );
    assert_eq!(
        ItemUseBroadcastPacket0104::decode(&broadcast_payload),
        Ok(ItemUseBroadcastPacket0104 {
            prefix: broadcast,
            results: ItemUseSkillResults0104::Damage(vec![damage]),
        })
    );
    assert_eq!(
        decode_quick_slot_packet_0104(packet::P_FE2CL_PC_ITEM_USE, &broadcast_payload),
        Ok(None)
    );
    assert_eq!(
        ItemUseBroadcastPrefix0104::decode_prefix(&broadcast_payload[..15]),
        Err(PayloadError::WrongSize {
            expected: ItemUseBroadcastPrefix0104::SIZE,
            actual: 15,
        })
    );
    assert_eq!(
        decode_quick_slot_packet_0104(0x3100_0abc, &[1, 2, 3]),
        Ok(None)
    );
}

#[test]
fn item_use_0104_record_field_offsets_match_clean_pack4_layouts() {
    let payload_for = |skill_type: i32, tail: Vec<u8>| {
        let mut payload = ItemUseBroadcastPrefix0104 {
            pc_id: 77,
            skill_id: 9,
            pack_padding: [0, 0],
            skill_type,
            target_count: 1,
        }
        .encode_prefix();
        payload.extend_from_slice(&tail);
        payload
    };

    let mut heal_stamina = vec![0; SkillResultHealStamina0104::SIZE];
    write_i32(&mut heal_stamina, 0, 1);
    write_i32(&mut heal_stamina, 4, 77);
    heal_stamina[8..10].copy_from_slice(&123i16.to_le_bytes());
    heal_stamina[10..12].copy_from_slice(&4i16.to_le_bytes());
    heal_stamina[12..14].copy_from_slice(&5i16.to_le_bytes());
    heal_stamina[14..16].copy_from_slice(&6i16.to_le_bytes());
    assert!(matches!(
        ItemUseBroadcastPacket0104::decode(&payload_for(6, heal_stamina)),
        Ok(ItemUseBroadcastPacket0104 {
            results: ItemUseSkillResults0104::HealStamina(records),
            ..
        }) if records == vec![SkillResultHealStamina0104 {
            character_type: 1,
            character_id: 77,
            healed_nano_stamina: 123,
            nano: Nano0104 { id: 4, skill_id: 5, stamina: 6 },
        }]
    ));

    let mut debuff = vec![0; SkillResultDamageDebuff0104::SIZE];
    for (offset, value) in [(0, 2), (4, 88), (8, 1), (12, 50), (16, 450)] {
        write_i32(&mut debuff, offset, value);
    }
    debuff[20..22].copy_from_slice(&(-7i16).to_le_bytes());
    debuff[22..24].copy_from_slice(&[0xaa, 0xbb]);
    write_i32(&mut debuff, 24, 1);
    write_i32(&mut debuff, 28, 0x1020_3040);
    assert!(matches!(
        ItemUseBroadcastPacket0104::decode(&payload_for(3, debuff)),
        Ok(ItemUseBroadcastPacket0104 {
            results: ItemUseSkillResults0104::DamageDebuff(records),
            ..
        }) if records == vec![SkillResultDamageDebuff0104 {
            character_type: 2,
            character_id: 88,
            protected: 1,
            damage: 50,
            hp: 450,
            stamina: -7,
            pack_padding: [0xaa, 0xbb],
            nano_deactivated: 1,
            condition_bit_flag: 0x1020_3040,
        }]
    ));

    let mut battery = vec![0; SkillResultBatteryDrain0104::SIZE];
    for (offset, value) in [
        (0, 1),
        (4, 77),
        (8, 0),
        (12, 10),
        (16, 90),
        (20, 20),
        (24, 80),
    ] {
        write_i32(&mut battery, offset, value);
    }
    battery[28..30].copy_from_slice(&30i16.to_le_bytes());
    battery[30..32].copy_from_slice(&[0xcc, 0xdd]);
    write_i32(&mut battery, 32, 0);
    write_i32(&mut battery, 36, -1);
    assert!(matches!(
        ItemUseBroadcastPacket0104::decode(&payload_for(21, battery)),
        Ok(ItemUseBroadcastPacket0104 {
            results: ItemUseSkillResults0104::BatteryDrain(records),
            ..
        }) if records == vec![SkillResultBatteryDrain0104 {
            character_type: 1,
            character_id: 77,
            protected: 0,
            drained_weapon_battery: 10,
            weapon_battery: 90,
            drained_nano_battery: 20,
            nano_battery: 80,
            stamina: 30,
            pack_padding: [0xcc, 0xdd],
            nano_deactivated: 0,
            condition_bit_flag: -1,
        }]
    ));

    let mut movement = vec![0; SkillResultMove0104::SIZE];
    for (offset, value) in [(0, 1), (4, 77), (8, 14), (12, 100), (16, 200), (20, 300)] {
        write_i32(&mut movement, offset, value);
    }
    assert!(matches!(
        ItemUseBroadcastPacket0104::decode(&payload_for(27, movement)),
        Ok(ItemUseBroadcastPacket0104 {
            results: ItemUseSkillResults0104::Move(records),
            ..
        }) if records == vec![SkillResultMove0104 {
            character_type: 1,
            character_id: 77,
            map_number: 14,
            position: [100, 200, 300],
        }]
    ));

    let mut resurrect = vec![0; SkillResultResurrect0104::SIZE];
    write_i32(&mut resurrect, 0, 1);
    write_i32(&mut resurrect, 4, 77);
    write_i32(&mut resurrect, 8, 500);
    assert!(matches!(
        ItemUseBroadcastPacket0104::decode(&payload_for(26, resurrect)),
        Ok(ItemUseBroadcastPacket0104 {
            results: ItemUseSkillResults0104::Resurrect(records),
            ..
        }) if records == vec![SkillResultResurrect0104 {
            character_type: 1,
            character_id: 77,
            regenerated_hp: 500,
        }]
    ));
}

#[test]
fn item_use_0104_bloodsucking_tail_has_leading_heal_then_damage_array() {
    let prefix = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 1,
        slot_num: 9,
        remaining_item: ItemBase0104 {
            item_type: 7,
            item_id: 120,
            option: 2,
            time_limit: 0,
        },
        skill_id: 10,
        pack_padding: [0, 0],
        skill_type: 30,
        target_count: 2,
    };
    let self_heal = SkillResultHealHp0104 {
        character_type: 1,
        character_id: 77,
        healed_hp: 50,
        hp: 500,
    };
    let targets = [
        SkillResultDamage0104 {
            character_type: 2,
            character_id: 100,
            protected: 0,
            damage: 25,
            hp: 75,
        },
        SkillResultDamage0104 {
            character_type: 2,
            character_id: 101,
            protected: 1,
            damage: 0,
            hp: 100,
        },
    ];
    let mut payload = prefix.encode_prefix();
    append_skill_result_heal_hp(&mut payload, self_heal);
    for damage in targets {
        append_skill_result_damage(&mut payload, damage);
    }

    assert_eq!(payload.len(), 92);
    assert_eq!(
        ItemUseSuccessPacket0104::decode(&payload),
        Ok(ItemUseSuccessPacket0104 {
            prefix,
            results: ItemUseSkillResults0104::Bloodsucking {
                self_heal,
                targets: targets.to_vec(),
            },
        })
    );
}

#[test]
fn item_use_0104_zero_unknown_and_unproven_positive_variants_fail_closed() {
    let zero = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 1,
        slot_num: 5,
        remaining_item: ItemBase0104 {
            item_type: 7,
            item_id: 321,
            option: 8,
            time_limit: 0,
        },
        skill_id: 0,
        pack_padding: [0, 0],
        skill_type: 1_004,
        target_count: 0,
    };
    assert_eq!(
        ItemUseSuccessPacket0104::decode(&zero.encode_prefix()),
        Ok(ItemUseSuccessPacket0104 {
            prefix: zero,
            results: ItemUseSkillResults0104::None,
        })
    );
    let mut extra_zero = zero.encode_prefix();
    extra_zero.push(0);
    assert!(matches!(
        ItemUseSuccessPacket0104::decode(&extra_zero),
        Err(ItemUseDecodeError0104::Fixed(PayloadError::WrongSize {
            expected: ItemUseSuccessPrefix0104::SIZE,
            ..
        }))
    ));

    for skill_type in [7, 9, 13, 22, 24, 29, 36, 38, 39, 777] {
        let unsupported = ItemUseBroadcastPrefix0104 {
            pc_id: 1,
            skill_id: 2,
            pack_padding: [0, 0],
            skill_type,
            target_count: 1,
        }
        .encode_prefix();
        assert_eq!(
            ItemUseBroadcastPacket0104::decode(&unsupported),
            Err(ItemUseDecodeError0104::UnsupportedPositiveTargetSkillType {
                skill_type,
                target_count: 1,
            })
        );
    }

    let negative = ItemUseBroadcastPrefix0104 {
        pc_id: 1,
        skill_id: 2,
        pack_padding: [0, 0],
        skill_type: 1,
        target_count: -1,
    }
    .encode_prefix();
    assert_eq!(
        ItemUseBroadcastPacket0104::decode(&negative),
        Err(ItemUseDecodeError0104::NegativeTargetCount { target_count: -1 })
    );
    assert_eq!(
        decode_item_use_packet_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn item_use_0104_failure_is_typed_by_the_dedicated_classifier() {
    let failure = ItemUseFailure0104 { error_code: -42 };
    assert_eq!(
        decode_item_use_packet_0104(packet::P_FE2CL_REP_PC_ITEM_USE_FAIL, &failure.encode(),),
        Ok(Some(ItemUsePacket0104::Failure(failure)))
    );
    assert_eq!(
        decode_item_use_packet_0104(packet::P_FE2CL_REP_PC_ITEM_USE_FAIL, &[0; 3]),
        Err(ItemUseDecodeError0104::Fixed(PayloadError::WrongSize {
            expected: ItemUseFailure0104::SIZE,
            actual: 3,
        }))
    );
}

#[test]
fn bank_0104_decoder_rejects_non_exact_bodies_and_unknown_ids() {
    for actual in [
        PcBankOpenSuccess0104::SIZE - 1,
        PcBankOpenSuccess0104::SIZE + 1,
    ] {
        assert_eq!(
            decode_pc_bank_reply_0104(packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC, &vec![0; actual],),
            Err(PayloadError::WrongSize {
                expected: PcBankOpenSuccess0104::SIZE,
                actual,
            })
        );
    }
    assert_eq!(
        decode_pc_bank_reply_0104(packet::P_FE2CL_REP_PC_BANK_CLOSE_FAIL, &[0; 3]),
        Err(PayloadError::WrongSize {
            expected: PcBankFailure0104::SIZE,
            actual: 3,
        })
    );
    assert_eq!(
        decode_pc_bank_reply_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}
