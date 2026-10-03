use super::*;

#[test]
fn inventory_0104_decoder_is_strict_and_unknown_packets_stay_unclaimed() {
    for actual in [
        ItemMoveSuccessPacket0104::SIZE - 1,
        ItemMoveSuccessPacket0104::SIZE + 1,
    ] {
        assert_eq!(
            decode_inventory_packet_0104(packet::P_FE2CL_PC_ITEM_MOVE_SUCC, &vec![0; actual],),
            Err(PayloadError::WrongSize {
                expected: ItemMoveSuccessPacket0104::SIZE,
                actual,
            })
        );
    }
    for actual in [
        EquipChangePacket0104::SIZE - 1,
        EquipChangePacket0104::SIZE + 1,
    ] {
        assert_eq!(
            decode_inventory_packet_0104(packet::P_FE2CL_PC_EQUIP_CHANGE, &vec![0; actual],),
            Err(PayloadError::WrongSize {
                expected: EquipChangePacket0104::SIZE,
                actual,
            })
        );
    }
    assert_eq!(
        decode_inventory_packet_0104(0x3100_0abc, &[0xde, 0xad]),
        Ok(None)
    );
}

#[test]
fn buddy_lifecycle_0104_names_state_and_variable_list_are_strict_and_lossless() {
    let buddy = BuddyBaseInfo0104 {
        pc_id: 81,
        pc_uid: 0x0102_0304_0506_0708,
        blocked: 0,
        free_chat: 1,
        pc_state: 2,
        first_name: FixedUtf16::from_str("\u{0414}\u{044d}\u{043a}\u{0441}").unwrap(),
        last_name: FixedUtf16::from_str(
            "\u{041c}\u{0430}\u{043d}\u{0434}\u{0430}\u{0440}\u{043a}",
        )
        .unwrap(),
        gender: 1,
        name_check_flag: 1,
    };
    let buddy_bytes = buddy.encode();
    assert_eq!(buddy_bytes.len(), 72);
    assert_eq!(buddy_bytes[15], 0);
    assert_eq!(&buddy_bytes[70..72], &[0, 0]);
    assert_eq!(BuddyBaseInfo0104::decode(&buddy_bytes), Ok(buddy.clone()));

    let accept = BuddyAcceptSuccess0104 {
        buddy_slot: 7,
        buddy: buddy.clone(),
    };
    let accept_bytes = accept.encode();
    assert_eq!(accept_bytes.len(), 76);
    assert_eq!(&accept_bytes[1..4], &[0, 0, 0]);
    assert_eq!(&accept_bytes[4..76], buddy_bytes.as_slice());
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC,
            &accept_bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::AcceptSuccess(accept)))
    );

    let incoming = BuddyIncomingRequest0104 {
        request_id: 91,
        buddy_id: 81,
        first_name: buddy.first_name.clone(),
        last_name: buddy.last_name.clone(),
    };
    let incoming_bytes = incoming.encode();
    assert_eq!(incoming_bytes.len(), 60);
    assert_eq!(
        BuddyIncomingRequest0104::decode(&incoming_bytes),
        Ok(incoming)
    );

    let state = BuddyStateSuccess0104 {
        buddy_ids: std::array::from_fn(|index| 1_000 + index as i32),
        buddy_states: std::array::from_fn(|index| (index % 3) as u8),
    };
    let state_bytes = state.encode();
    assert_eq!(state_bytes.len(), 252);
    assert_eq!(&state_bytes[248..250], &[0, 1]);
    assert_eq!(&state_bytes[250..252], &[0, 0]);
    assert_eq!(BuddyStateSuccess0104::decode(&state_bytes), Ok(state));

    let prefix = BuddyListInfoPrefix0104 {
        pc_id: 81,
        pc_uid: 0x3132_3334_3536_3738,
        list_num: 0,
        buddy_count: 1,
        pack_padding: [0xaa, 0xbb],
    };
    let mut variable = prefix.encode_prefix();
    variable.extend_from_slice(&buddy_bytes);
    assert_eq!(
        BuddyListInfoPrefix0104::decode_prefix(&variable),
        Ok(prefix)
    );
    let list = BuddyListInfo0104 {
        pc_id: prefix.pc_id,
        pc_uid: prefix.pc_uid,
        list_num: prefix.list_num,
        pack_padding: prefix.pack_padding,
        buddies: vec![buddy.clone()],
    };
    assert_eq!(list.buddy_count(), 1);
    assert_eq!(list.encode(), Ok(variable.clone()));
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC,
            &variable,
        ),
        Ok(Some(BuddyLifecyclePacket0104::ListInfo(list)))
    );
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC,
            &variable[..15],
        ),
        Err(PayloadError::WrongSize {
            expected: BuddyListInfoPrefix0104::SIZE,
            actual: 15,
        })
    );
    assert_eq!(
        BuddyListInfo0104::decode(&variable[..variable.len() - 1]),
        Err(PayloadError::WrongSize {
            expected: BuddyListInfoPrefix0104::SIZE + BuddyBaseInfo0104::SIZE,
            actual: BuddyListInfoPrefix0104::SIZE + BuddyBaseInfo0104::SIZE - 1,
        })
    );
    let mut trailing = variable.clone();
    trailing.push(0xcc);
    assert_eq!(
        BuddyListInfo0104::decode(&trailing),
        Err(PayloadError::WrongSize {
            expected: BuddyListInfoPrefix0104::SIZE + BuddyBaseInfo0104::SIZE,
            actual: BuddyListInfoPrefix0104::SIZE + BuddyBaseInfo0104::SIZE + 1,
        })
    );

    let mut invalid_list_num = BuddyListInfoPrefix0104 {
        pc_id: 81,
        pc_uid: prefix.pc_uid,
        list_num: -1,
        buddy_count: 0,
        pack_padding: [0, 0],
    }
    .encode_prefix();
    assert_eq!(
        BuddyListInfo0104::decode(&invalid_list_num),
        Err(PayloadError::ValueOutOfRange {
            field: "buddy list_num",
            value: -1,
            minimum: 0,
            maximum: 50,
        })
    );
    invalid_list_num[12] = 51;
    assert_eq!(
        BuddyListInfo0104::decode(&invalid_list_num),
        Err(PayloadError::ValueOutOfRange {
            field: "buddy list_num",
            value: 51,
            minimum: 0,
            maximum: 50,
        })
    );
    let invalid_count = BuddyListInfoPrefix0104 {
        pc_id: 81,
        pc_uid: prefix.pc_uid,
        list_num: 0,
        buddy_count: -1,
        pack_padding: [0, 0],
    }
    .encode_prefix();
    assert_eq!(
        BuddyListInfo0104::decode(&invalid_count),
        Err(PayloadError::ValueOutOfRange {
            field: "buddy count",
            value: -1,
            minimum: 0,
            maximum: 50,
        })
    );
    let mut invalid_count_high = invalid_count;
    invalid_count_high[13] = 51;
    assert_eq!(
        BuddyListInfo0104::decode(&invalid_count_high),
        Err(PayloadError::ValueOutOfRange {
            field: "buddy count",
            value: 51,
            minimum: 0,
            maximum: 50,
        })
    );
    let mut crossing = BuddyListInfoPrefix0104 {
        pc_id: 81,
        pc_uid: prefix.pc_uid,
        list_num: 50,
        buddy_count: 1,
        pack_padding: [0, 0],
    }
    .encode_prefix();
    crossing.extend_from_slice(&buddy_bytes);
    assert_eq!(
        BuddyListInfo0104::decode(&crossing),
        Err(PayloadError::ValueOutOfRange {
            field: "buddy list_num + count",
            value: 51,
            minimum: 0,
            maximum: 50,
        })
    );
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE,
            &[0; BuddyFreeChatRequest0104::SIZE],
        ),
        Ok(None),
        "Buddy FreeChat remains a separate ABI"
    );
}

#[test]
fn reward_item_0104_is_strict_variable_post_state() {
    let reply = RewardItemReply0104 {
        candy: 11,
        fusion_matter: 22,
        nano_battery: 33,
        weapon_battery: 44,
        pack_padding: [0xa1, 0xb2, 0xc3],
        fatigue: 55,
        fatigue_level: 66,
        npc_type_id: 2676,
        task_id: 2248,
        items: vec![
            ItemReward0104 {
                item: ItemBase0104 {
                    item_type: 8,
                    item_id: 120,
                    option: 3,
                    time_limit: 0,
                },
                inventory_location: 2,
                slot: 4,
            },
            ItemReward0104 {
                item: ItemBase0104 {
                    item_type: 9,
                    item_id: 531,
                    option: 1,
                    time_limit: 123,
                },
                inventory_location: 1,
                slot: 5,
            },
        ],
    };
    let encoded = reply.encode().unwrap();
    assert_eq!(encoded.len(), RewardItemReply0104::HEADER_SIZE + 40);
    assert_eq!(encoded[16], 2);
    assert_eq!(&encoded[17..20], &[0xa1, 0xb2, 0xc3]);
    assert_eq!(RewardItemReply0104::decode(&encoded), Ok(reply));

    let mut truncated = encoded.clone();
    truncated.pop();
    assert!(matches!(
        RewardItemReply0104::decode(&truncated),
        Err(PayloadError::WrongSize { .. })
    ));
    let mut negative_count = vec![0; RewardItemReply0104::HEADER_SIZE];
    negative_count[16] = 0xff;
    assert!(matches!(
        RewardItemReply0104::decode(&negative_count),
        Err(PayloadError::ValueOutOfRange {
            field: "reward item count",
            ..
        })
    ));
}
