use super::*;

#[test]
fn bridge_command_queue_is_bounded_and_reports_backpressure() {
    let (commands, _command_rx) = mpsc::sync_channel(NetworkBridge::COMMAND_QUEUE_CAPACITY);
    let (_events, event_rx) = mpsc::sync_channel(1);
    let bridge = NetworkBridge {
        commands,
        events: Mutex::new(event_rx),
    };
    for _ in 0..NetworkBridge::COMMAND_QUEUE_CAPACITY {
        bridge.send(NetworkCommand::RefreshCharacters).unwrap();
    }
    let error = bridge
        .send(NetworkCommand::RefreshCharacters)
        .expect_err("the bounded queue must reject one extra command");
    assert!(error.contains("queue is full"));
    assert!(error.contains(&NetworkBridge::COMMAND_QUEUE_CAPACITY.to_string()));
}

#[test]
fn quick_slot_commands_expose_register_and_use_intent() {
    let register = NetworkCommand::RegisterQuickSlot(QuickSlotRegisterRequest0104 {
        slot_num: 6,
        item_type: 10,
        item_id: 304,
    });
    let use_item = NetworkCommand::UseItem(ItemUseRequest0104 {
        item_location: 3,
        slot_num: 6,
        nano_slot: -1,
    });
    let open_chest = NetworkCommand::OpenChest(ItemChestOpenRequest0104 {
        item_location: 1,
        slot_num: 7,
        chest_item: ffone_protocol::ItemBase0104 {
            item_type: 9,
            item_id: 42,
            option: 1,
            time_limit: 0,
        },
    });

    let register_debug = format!("{register:?}");
    assert!(register_debug.contains("RegisterQuickSlot"));
    assert!(register_debug.contains("slot_num: 6"));
    assert!(register_debug.contains("item_id: 304"));

    let use_debug = format!("{use_item:?}");
    assert!(use_debug.contains("UseItem"));
    assert!(use_debug.contains("item_location: 3"));
    assert!(use_debug.contains("nano_slot: -1"));

    let open_debug = format!("{open_chest:?}");
    assert!(open_debug.contains("OpenChest"));
    assert!(open_debug.contains("item_type: 9"));
    assert!(open_debug.contains("slot_num: 7"));
}

#[test]
fn regen_command_debug_and_worker_routing_are_explicit() {
    let request = PcRegenRequest0104 {
        regen_type: 6,
        e_il: 1,
        index: 23,
    };
    let command = NetworkCommand::Regen(request);
    let debug = format!("{command:?}");
    assert!(debug.contains("Regen"));
    assert!(debug.contains("regen_type: 6"));
    assert!(debug.contains("e_il: 1"));
    assert!(debug.contains("index: 23"));

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));

    command_tx.send(NetworkCommand::Regen(request)).unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Error(message))
            if message == "gameplay packet requested before entering the world"
    ));

    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn present_npc_types_event_access_is_typed_strict_and_lossless() {
    let reply = PresentNpcTypesReply0104 {
        clear: -3,
        npc_types: vec![2_671, 3_145, -1],
    };
    let valid_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PRESENT_NPC_TYPES,
        flags: 0x123,
        checksum: 0x456,
        payload: reply.encode().unwrap(),
    };
    let decoded = NetworkEvent::Frame(valid_frame.clone())
        .present_npc_types_frame_0104()
        .unwrap();
    assert!(matches!(
        &decoded,
        PresentNpcTypesGameplayFrame0104::Decoded {
            frame,
            packet: PresentNpcTypesPacket0104::Reply(actual),
        } if frame == &valid_frame && actual == &reply && actual.clears_existing()
    ));
    assert_eq!(decoded.raw_frame(), &valid_frame);

    let malformed_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PRESENT_NPC_TYPES,
        flags: 0x234,
        checksum: 0x567,
        payload: {
            let mut payload = vec![0; 12];
            payload[4..8].copy_from_slice(&2i32.to_le_bytes());
            payload[8..12].copy_from_slice(&2_671i32.to_le_bytes());
            payload
        },
    };
    let malformed = NetworkEvent::Frame(malformed_frame.clone())
        .present_npc_types_frame_0104()
        .unwrap();
    assert!(matches!(
        &malformed,
        PresentNpcTypesGameplayFrame0104::Malformed { frame, .. }
            if frame == &malformed_frame
    ));
    assert_eq!(malformed.raw_frame(), &malformed_frame);

    let unknown_frame = DecodedFrame {
        packet_type: 0x3100_0abc,
        flags: 0x345,
        checksum: 0x678,
        payload: vec![0xde, 0xad, 0xbe, 0xef],
    };
    let passthrough = NetworkEvent::Frame(unknown_frame.clone())
        .present_npc_types_frame_0104()
        .unwrap();
    assert!(matches!(
        &passthrough,
        PresentNpcTypesGameplayFrame0104::Passthrough(frame)
            if frame == &unknown_frame
    ));
    assert_eq!(passthrough.raw_frame(), &unknown_frame);

    assert!(
        NetworkEvent::LoginFrame(valid_frame)
            .present_npc_types_frame_0104()
            .is_none()
    );
}

#[test]
fn quick_slot_event_access_is_typed_strict_and_lossless() {
    let roster = QuickSlotInfo0104 {
        slots: std::array::from_fn(|index| QuickSlotEntry0104 {
            item_type: 10 + index as i16,
            item_id: 500 + index as i16,
        }),
    };
    let roster_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_QUICK_SLOT_INFO,
        flags: 0x123,
        checksum: 0x456,
        payload: roster.encode(),
    };
    let event = NetworkEvent::Frame(roster_frame.clone());
    let classified = event.quick_slot_frame_0104().unwrap();
    assert!(matches!(
        &classified,
        QuickSlotGameplayFrame0104::Decoded {
            frame,
            packet: QuickSlotPacket0104::Info(actual),
        } if frame == &roster_frame && actual == &roster
    ));
    assert_eq!(classified.raw_frame(), &roster_frame);

    let malformed_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_QUICK_SLOT_INFO,
        flags: 0x234,
        checksum: 0x567,
        payload: vec![0x5a; QuickSlotInfo0104::SIZE - 1],
    };
    let malformed = NetworkEvent::Frame(malformed_frame.clone())
        .quick_slot_frame_0104()
        .unwrap();
    assert!(matches!(
        &malformed,
        QuickSlotGameplayFrame0104::Malformed { frame, .. }
            if frame == &malformed_frame
    ));
    assert_eq!(malformed.raw_frame(), &malformed_frame);

    let mut variable_payload = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 3,
        slot_num: 6,
        remaining_item: ItemBase0104 {
            item_type: 10,
            item_id: 506,
            option: 1,
            time_limit: 900,
        },
        skill_id: 22,
        pack_padding: [0, 0],
        skill_type: 3,
        target_count: 1,
    }
    .encode_prefix();
    variable_payload.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
    let variable_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
        flags: 0x345,
        checksum: 0x678,
        payload: variable_payload,
    };
    let variable = NetworkEvent::Frame(variable_frame.clone())
        .quick_slot_frame_0104()
        .unwrap();
    assert!(matches!(
        &variable,
        QuickSlotGameplayFrame0104::Passthrough(frame) if frame == &variable_frame
    ));
    assert_eq!(variable.raw_frame(), &variable_frame);

    let malformed_item_use = NetworkEvent::Frame(variable_frame.clone())
        .item_use_frame_0104()
        .unwrap();
    assert!(matches!(
        &malformed_item_use,
        ItemUseGameplayFrame0104::Malformed { frame, .. } if frame == &variable_frame
    ));
    assert_eq!(malformed_item_use.raw_frame(), &variable_frame);

    let valid_prefix = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 1,
        slot_num: 6,
        remaining_item: ItemBase0104 {
            item_type: 7,
            item_id: 506,
            option: 1,
            time_limit: 900,
        },
        skill_id: 144,
        pack_padding: [0, 0],
        skill_type: 33,
        target_count: 1,
    };
    let mut valid_payload = valid_prefix.encode_prefix();
    valid_payload.resize(
        ItemUseSuccessPrefix0104::SIZE + SkillResultBuff0104::SIZE,
        0,
    );
    let valid_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
        flags: 0x456,
        checksum: 0x789,
        payload: valid_payload,
    };
    let valid = NetworkEvent::Frame(valid_frame.clone())
        .item_use_frame_0104()
        .unwrap();
    assert!(matches!(
        &valid,
        ItemUseGameplayFrame0104::Decoded {
            frame,
            packet: ItemUsePacket0104::Success(packet),
        } if frame == &valid_frame
            && packet.prefix() == valid_prefix
            && matches!(packet.results(), ItemUseSkillResults0104::Buff(records) if records.len() == 1)
    ));
    assert_eq!(valid.raw_frame(), &valid_frame);

    assert!(
        NetworkEvent::LoginFrame(variable_frame.clone())
            .quick_slot_frame_0104()
            .is_none()
    );
    assert!(
        NetworkEvent::LoginFrame(variable_frame)
            .item_use_frame_0104()
            .is_none()
    );
}

#[test]
fn vendor_commands_and_event_access_are_typed_strict_and_lossless() {
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 42,
        option: 3,
        time_limit: 0,
    };
    let commands = [
        NetworkCommand::VendorStart(VendorStartRequest0104 {
            npc_id: 9_001,
            vendor_id: 650,
        }),
        NetworkCommand::VendorTableUpdate(VendorTableUpdateRequest0104 {
            npc_id: 9_001,
            vendor_id: 650,
        }),
        NetworkCommand::VendorBuy(VendorItemBuyRequest0104 {
            npc_id: 9_001,
            vendor_id: 9_001,
            list_id: 41,
            item,
            inventory_slot: 4,
        }),
        NetworkCommand::VendorSell(VendorItemSellRequest0104 {
            inventory_slot: 5,
            item_count: 2,
        }),
        NetworkCommand::VendorRestore(VendorItemRestoreBuyRequest0104 {
            npc_id: 9_001,
            vendor_id: 9_001,
            list_id: 3,
            item,
            inventory_slot: 6,
        }),
        NetworkCommand::VendorBatteryBuy(VendorBatteryBuyRequest0104 {
            npc_id: 9_001,
            vendor_id: 9_001,
            list_id: 41,
            item,
        }),
        NetworkCommand::DeleteInventoryItem(PcItemDeleteRequest0104 {
            item_location: 1,
            slot_num: 7,
        }),
        NetworkCommand::DisassembleInventoryItem(PcDisassembleItemRequest0104 { item_slot: 8 }),
    ];
    let debug = commands
        .iter()
        .map(|command| format!("{command:?}"))
        .collect::<Vec<_>>();
    for intent in [
        "VendorStart",
        "VendorTableUpdate",
        "VendorBuy",
        "VendorSell",
        "VendorRestore",
        "VendorBatteryBuy",
        "DeleteInventoryItem",
        "DisassembleInventoryItem",
    ] {
        assert!(debug.iter().any(|value| value.contains(intent)));
    }

    let success = VendorStartSuccess0104 {
        npc_id: 9_001,
        vendor_id: 650,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_VENDOR_START_SUCC,
        flags: 0x123,
        checksum: 0x456,
        payload: success.encode(),
    };
    let classified = NetworkEvent::Frame(frame.clone())
        .vendor_frame_0104()
        .unwrap();
    assert!(matches!(
        &classified,
        VendorGameplayFrame0104::Decoded {
            frame: actual_frame,
            packet: VendorPacket0104::StartSuccess(actual),
        } if actual_frame == &frame && actual == &success
    ));
    assert_eq!(classified.raw_frame(), &frame);

    let malformed_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_VENDOR_START_SUCC,
        flags: 0x234,
        checksum: 0x567,
        payload: vec![0x5a; VendorStartSuccess0104::SIZE - 1],
    };
    let malformed = NetworkEvent::Frame(malformed_frame.clone())
        .vendor_frame_0104()
        .unwrap();
    assert!(matches!(
        &malformed,
        VendorGameplayFrame0104::Malformed { frame, .. } if frame == &malformed_frame
    ));
    assert_eq!(malformed.raw_frame(), &malformed_frame);

    let unknown_frame = DecodedFrame {
        packet_type: 0x3100_0abc,
        flags: 0x345,
        checksum: 0x678,
        payload: vec![0xde, 0xad],
    };
    let passthrough = NetworkEvent::Frame(unknown_frame.clone())
        .vendor_frame_0104()
        .unwrap();
    assert!(matches!(
        &passthrough,
        VendorGameplayFrame0104::Passthrough(frame) if frame == &unknown_frame
    ));
    assert_eq!(passthrough.raw_frame(), &unknown_frame);
    assert!(
        NetworkEvent::LoginFrame(unknown_frame)
            .vendor_frame_0104()
            .is_none()
    );
}

#[test]
fn buddy_lifecycle_commands_expose_exact_target_intent() {
    let uid = 0x0102_0304_0506_0708;
    let commands = [
        NetworkCommand::RequestBuddy(BuddyMakeRequest0104 {
            buddy_id: 81,
            buddy_pc_uid: uid,
        }),
        NetworkCommand::RequestBuddyByName(BuddyFindNameRequest0104 {
            first_name: FixedUtf16::from_str("Dexter").unwrap(),
            last_name: FixedUtf16::from_str("Mandark").unwrap(),
        }),
        NetworkCommand::AcceptBuddyByName(BuddyFindNameAcceptRequest0104 {
            accept_flag: 1,
            buddy_pc_uid: uid,
            first_name: FixedUtf16::from_str("Dexter").unwrap(),
            last_name: FixedUtf16::from_str("Mandark").unwrap(),
        }),
        NetworkCommand::AcceptBuddy(BuddyAcceptRequest0104 {
            accept_flag: 1,
            buddy_id: 81,
            buddy_pc_uid: uid,
        }),
        NetworkCommand::RefreshBuddyState(BuddyStateRequest0104 { unused: 0 }),
        NetworkCommand::BlockBuddy(BuddySetBlockRequest0104 {
            buddy_pc_uid: uid,
            buddy_slot: 7,
        }),
        NetworkCommand::RemoveBuddy(BuddyRemoveRequest0104 {
            buddy_pc_uid: uid,
            buddy_slot: 7,
        }),
        NetworkCommand::WarpToBuddy(BuddyWarpRequest0104 {
            buddy_pc_uid: uid,
            buddy_slot: 7,
        }),
        NetworkCommand::LeaveGroup(GroupLeaveRequest0104 { unused: 0 }),
    ];
    let debug = commands
        .iter()
        .map(|command| format!("{command:?}"))
        .collect::<Vec<_>>();
    for intent in [
        "RequestBuddy",
        "RequestBuddyByName",
        "AcceptBuddyByName",
        "AcceptBuddy",
        "RefreshBuddyState",
        "BlockBuddy",
        "RemoveBuddy",
        "WarpToBuddy",
        "LeaveGroup",
    ] {
        assert!(debug.iter().any(|value| value.contains(intent)));
    }
    assert!(debug[0].contains("buddy_id: 81"));
    assert!(debug[1].contains("Dexter"));
    assert!(debug[2].contains("accept_flag: 1"));
    assert!(debug[3].contains("accept_flag: 1"));
    assert!(debug[5].contains("buddy_slot: 7"));
    assert!(debug[7].contains("72623859790382856"));
    assert!(debug[8].contains("unused: 0"));
}

#[test]
fn exit_world_command_debug_and_worker_routing_are_explicit() {
    assert_eq!(format!("{:?}", NetworkCommand::ExitWorld), "ExitWorld");

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));

    command_tx.send(NetworkCommand::ExitWorld).unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Error(message))
            if message == "gameplay packet requested before entering the world"
    ));

    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn buddy_lifecycle_event_access_is_typed_strict_and_lossless() {
    let state = BuddyStateSuccess0104 {
        buddy_ids: std::array::from_fn(|index| 500 + index as i32),
        buddy_states: std::array::from_fn(|index| (index % 2) as u8),
    };
    let state_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC,
        flags: 0x123,
        checksum: 0x456,
        payload: state.encode(),
    };
    let decoded = NetworkEvent::Frame(state_frame.clone())
        .buddy_lifecycle_frame_0104()
        .unwrap();
    assert!(matches!(
        &decoded,
        BuddyLifecycleGameplayFrame0104::Decoded {
            frame,
            packet: BuddyLifecyclePacket0104::StateSuccess(actual),
        } if frame == &state_frame && actual == &state
    ));
    assert_eq!(decoded.raw_frame(), &state_frame);

    let malformed_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC,
        flags: 0x234,
        checksum: 0x567,
        payload: vec![0x5a; BuddyStateSuccess0104::SIZE - 1],
    };
    let malformed = NetworkEvent::Frame(malformed_frame.clone())
        .buddy_lifecycle_frame_0104()
        .unwrap();
    assert!(matches!(
        &malformed,
        BuddyLifecycleGameplayFrame0104::Malformed { frame, .. }
            if frame == &malformed_frame
    ));
    assert_eq!(malformed.raw_frame(), &malformed_frame);

    let list = BuddyListInfo0104 {
        pc_id: 77,
        pc_uid: 0x1112_1314_1516_1718,
        list_num: 0,
        pack_padding: [0xaa, 0xbb],
        buddies: vec![BuddyBaseInfo0104 {
            pc_id: 81,
            pc_uid: 0x0102_0304_0506_0708,
            blocked: 0,
            free_chat: 1,
            pc_state: 2,
            first_name: FixedUtf16::from_str("Dexter").unwrap(),
            last_name: FixedUtf16::from_str("Mandark").unwrap(),
            gender: 1,
            name_check_flag: 1,
        }],
    };
    let variable_payload = list.encode().unwrap();
    let variable_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC,
        flags: 0x345,
        checksum: 0x678,
        payload: variable_payload.clone(),
    };
    let variable = NetworkEvent::Frame(variable_frame.clone())
        .buddy_lifecycle_frame_0104()
        .unwrap();
    assert!(matches!(
        &variable,
        BuddyLifecycleGameplayFrame0104::Decoded {
            frame,
            packet: BuddyLifecyclePacket0104::ListInfo(actual),
        } if frame == &variable_frame && actual == &list
    ));
    assert_eq!(variable.raw_frame(), &variable_frame);

    let malformed_list_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC,
        flags: 0x456,
        checksum: 0x789,
        payload: variable_payload[..variable_payload.len() - 1].to_vec(),
    };
    let malformed_list = NetworkEvent::Frame(malformed_list_frame.clone())
        .buddy_lifecycle_frame_0104()
        .unwrap();
    assert!(matches!(
        &malformed_list,
        BuddyLifecycleGameplayFrame0104::Malformed { frame, .. }
            if frame == &malformed_list_frame
    ));
    assert_eq!(malformed_list.raw_frame(), &malformed_list_frame);

    assert!(
        NetworkEvent::LoginFrame(variable_frame)
            .buddy_lifecycle_frame_0104()
            .is_none()
    );
}

#[test]
fn freechat_command_debug_identifies_the_outbound_message() {
    let command = NetworkCommand::SendFreeChat(FreeChatRequest0104 {
        message: FixedUtf16::from_str("Hello, world").unwrap(),
        emote_code: 7,
    });
    let debug = format!("{command:?}");
    assert!(debug.contains("SendFreeChat"));
    assert!(debug.contains("Hello, world"));
}

#[test]
fn gm_speed_command_preserves_player_type_and_value() {
    let command = NetworkCommand::SetGmValue(GmSetValueRequest0104::speed(81, 1_200));
    let debug = format!("{command:?}");
    assert!(debug.contains("SetGmValue"));
    assert!(debug.contains("pc_id: 81"));
    assert!(debug.contains("value_type: 6"));
    assert!(debug.contains("value: 1200"));
}

#[test]
fn all_group_freechat_command_has_no_target_semantics() {
    let command = NetworkCommand::SendAllGroupFreeChat(AllGroupFreeChatRequest0104 {
        message: FixedUtf16::from_str("Squad ready").unwrap(),
        emote_code: 17,
    });
    let debug = format!("{command:?}");
    assert!(debug.contains("SendAllGroupFreeChat"));
    assert!(debug.contains("Squad ready"));
}

#[test]
fn buddy_freechat_command_preserves_target_uid_and_slot() {
    let command = NetworkCommand::SendBuddyFreeChat(BuddyFreeChatRequest0104 {
        message: FixedUtf16::from_str("Buddy ready").unwrap(),
        emote_code: 17,
        buddy_pc_uid: 0x0102_0304_0506_0708,
        buddy_slot: 6,
    });
    let debug = format!("{command:?}");
    assert!(debug.contains("SendBuddyFreeChat"));
    assert!(debug.contains("Buddy ready"));
    assert!(debug.contains("72623859790382856"));
    assert!(debug.contains("buddy_slot: 6"));
}

#[test]
fn nano_tune_event_correlation_routes_authoritative_success_and_failure_fail_closed() {
    let pending = NanoTunePending0104 {
        request_token: 41,
        player_id: 77,
        nano_id: 36,
        skill_id: 144,
    };
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
    let success_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_TUNE_SUCC,
        flags: 0x123,
        checksum: 0x456,
        payload: success.encode(),
    };
    let correlated = NetworkEvent::Frame(success_frame.clone())
        .correlate_nano_tune_0104(pending)
        .expect("known Nano tune packet")
        .expect("matching reply");
    assert_eq!(correlated.request_token, 41);
    assert_eq!(correlated.packet_id(), packet::P_FE2CL_REP_NANO_TUNE_SUCC);
    assert_eq!(correlated.payload_size(), NanoTuneSuccess0104::SIZE);
    assert_eq!(correlated.frame, success_frame);
    assert_eq!(correlated.packet, NanoTunePacket0104::Success(success));

    let failure = NanoTuneFailure0104 {
        pc_id: 77,
        error_code: 5,
    };
    let failure_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_TUNE_FAIL,
        flags: 0x234,
        checksum: 0x567,
        payload: failure.encode(),
    };
    let correlated_failure = NetworkEvent::Frame(failure_frame.clone())
        .correlate_nano_tune_0104(pending)
        .expect("known Nano tune packet")
        .expect("matching failure identity");
    assert_eq!(correlated_failure.request_token, 41);
    assert_eq!(correlated_failure.frame, failure_frame);
    assert_eq!(
        correlated_failure.packet,
        NanoTunePacket0104::Failure(failure)
    );

    let mismatched_skill = NanoTuneSuccess0104 {
        skill_id: 145,
        ..success
    };
    let mismatched_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_TUNE_SUCC,
        flags: 0x345,
        checksum: 0x678,
        payload: mismatched_skill.encode(),
    };
    let mismatch = NetworkEvent::Frame(mismatched_frame.clone())
        .correlate_nano_tune_0104(pending)
        .expect("known Nano tune packet")
        .expect_err("mismatched authoritative skill must fail closed");
    assert!(matches!(
        mismatch,
        NanoTuneCorrelationError0104::Mismatch {
            frame,
            fault: NanoTuneCorrelationFault0104::SkillId {
                expected: 144,
                actual: 145,
            },
        } if frame == mismatched_frame
    ));

    let mismatched_player = NanoTuneFailure0104 {
        pc_id: 78,
        ..failure
    };
    let mismatched_player_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_TUNE_FAIL,
        flags: 0x456,
        checksum: 0x789,
        payload: mismatched_player.encode(),
    };
    assert!(matches!(
        NetworkEvent::Frame(mismatched_player_frame.clone())
            .correlate_nano_tune_0104(pending),
        Some(Err(NanoTuneCorrelationError0104::Mismatch {
            frame,
            fault: NanoTuneCorrelationFault0104::PlayerId {
                expected: 77,
                actual: 78,
            },
        })) if frame == mismatched_player_frame
    ));

    let malformed_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_TUNE_SUCC,
        flags: 0x567,
        checksum: 0x89a,
        payload: vec![0x5a; NanoTuneSuccess0104::SIZE - 1],
    };
    assert!(matches!(
        NetworkEvent::Frame(malformed_frame.clone()).correlate_nano_tune_0104(pending),
        Some(Err(NanoTuneCorrelationError0104::Malformed {
            frame,
            error: PayloadError::WrongSize {
                expected: NanoTuneSuccess0104::SIZE,
                actual,
            },
        })) if frame == malformed_frame && actual == NanoTuneSuccess0104::SIZE - 1
    ));

    let unrelated = DecodedFrame {
        packet_type: 0x3100_0abc,
        flags: 0x678,
        checksum: 0x9ab,
        payload: vec![0xde, 0xad],
    };
    assert!(
        NetworkEvent::Frame(unrelated)
            .correlate_nano_tune_0104(pending)
            .is_none()
    );
    assert!(
        NetworkEvent::LoginFrame(success_frame)
            .correlate_nano_tune_0104(pending)
            .is_none()
    );
}
