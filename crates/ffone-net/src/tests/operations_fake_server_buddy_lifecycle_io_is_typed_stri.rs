use super::*;

#[test]
fn fake_server_buddy_lifecycle_io_is_typed_strict_and_lossless() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x1122_3344_5566_7788;
    let e_key = 0x8877_6655_4433_2211;
    let buddy_uid = 0x0102_0304_0506_0708;
    let state = BuddyStateSuccess0104 {
        buddy_ids: std::array::from_fn(|index| 500 + index as i32),
        buddy_states: std::array::from_fn(|index| (index % 2) as u8),
    };
    let malformed_payload = vec![0x5a; BuddyStateSuccess0104::SIZE - 1];
    let list = BuddyListInfo0104 {
        pc_id: 77,
        pc_uid: 0x1112_1314_1516_1718,
        list_num: 0,
        pack_padding: [0xaa, 0xbb],
        buddies: vec![BuddyBaseInfo0104 {
            pc_id: 81,
            pc_uid: buddy_uid,
            blocked: 0,
            free_chat: 1,
            pc_state: 2,
            first_name: FixedUtf16::from_str("Dexter").unwrap(),
            last_name: FixedUtf16::from_str("Mandark").unwrap(),
            gender: 1,
            name_check_flag: 1,
        }],
    };
    let variable_list = list.encode().unwrap();
    let malformed_list = variable_list[..variable_list.len() - 1].to_vec();
    let find_name_success = BuddyFindNameSuccess0104 {
        first_name: FixedUtf16::from_str("Dexter").unwrap(),
        last_name: FixedUtf16::from_str("Mandark").unwrap(),
        pc_uid: buddy_uid,
        name_check_flag: 1,
    };
    let find_name_accept_failure = BuddyFindNameAcceptFailure0104 {
        first_name: FixedUtf16::from_str("Dexter").unwrap(),
        last_name: FixedUtf16::from_str("Mandark").unwrap(),
        pc_uid: buddy_uid,
        name_check_flag: 1,
        error_code: 6,
    };
    let warp_success = BuddyWarpOtherShardSuccess0104 {
        buddy_pc_uid: buddy_uid,
        shard_num: 3,
        channel_num: 27,
    };

    let server_state = state.clone();
    let server_malformed = malformed_payload.clone();
    let server_variable = variable_list.clone();
    let server_malformed_list = malformed_list.clone();
    let server_find_name_success = find_name_success.clone();
    let server_find_name_accept_failure = find_name_accept_failure.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        let request = read_client(&mut stream, e_key, 0);
        assert_eq!(request.packet_type, packet::P_CL2FE_REQ_REQUEST_MAKE_BUDDY);
        assert_eq!(
            BuddyMakeRequest0104::decode(&request.payload),
            Ok(BuddyMakeRequest0104 {
                buddy_id: 81,
                buddy_pc_uid: buddy_uid,
            })
        );

        let accept = read_client(&mut stream, e_key, 1);
        assert_eq!(accept.packet_type, packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY);
        assert_eq!(&accept.payload[1..4], &[0, 0, 0]);
        assert_eq!(
            BuddyAcceptRequest0104::decode(&accept.payload),
            Ok(BuddyAcceptRequest0104 {
                accept_flag: 1,
                buddy_id: 81,
                buddy_pc_uid: buddy_uid,
            })
        );

        let refresh = read_client(&mut stream, e_key, 2);
        assert_eq!(refresh.packet_type, packet::P_CL2FE_REQ_GET_BUDDY_STATE);
        assert_eq!(refresh.payload, vec![0]);

        let block = read_client(&mut stream, e_key, 3);
        assert_eq!(block.packet_type, packet::P_CL2FE_REQ_SET_BUDDY_BLOCK);
        assert_eq!(&block.payload[9..12], &[0, 0, 0]);
        assert_eq!(
            BuddySetBlockRequest0104::decode(&block.payload),
            Ok(BuddySetBlockRequest0104 {
                buddy_pc_uid: buddy_uid,
                buddy_slot: 7,
            })
        );

        let remove = read_client(&mut stream, e_key, 4);
        assert_eq!(remove.packet_type, packet::P_CL2FE_REQ_REMOVE_BUDDY);
        assert_eq!(&remove.payload[9..12], &[0, 0, 0]);
        assert_eq!(
            BuddyRemoveRequest0104::decode(&remove.payload),
            Ok(BuddyRemoveRequest0104 {
                buddy_pc_uid: buddy_uid,
                buddy_slot: 7,
            })
        );

        let warp = read_client(&mut stream, e_key, 5);
        assert_eq!(warp.packet_type, packet::P_CL2FE_REQ_PC_BUDDY_WARP);
        assert_eq!(&warp.payload[9..12], &[0, 0, 0]);
        assert_eq!(
            BuddyWarpRequest0104::decode(&warp.payload),
            Ok(BuddyWarpRequest0104 {
                buddy_pc_uid: buddy_uid,
                buddy_slot: 7,
            })
        );

        let find_name = read_client(&mut stream, e_key, 6);
        assert_eq!(
            find_name.packet_type,
            packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY
        );
        let find_name = BuddyFindNameRequest0104::decode(&find_name.payload).unwrap();
        assert_eq!(find_name.first_name.to_string_lossy(), "Dexter");
        assert_eq!(find_name.last_name.to_string_lossy(), "Mandark");

        let find_name_accept = read_client(&mut stream, e_key, 7);
        assert_eq!(
            find_name_accept.packet_type,
            packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY
        );
        let find_name_accept =
            BuddyFindNameAcceptRequest0104::decode(&find_name_accept.payload).unwrap();
        assert_eq!(find_name_accept.accept_flag, 1);
        assert_eq!(find_name_accept.buddy_pc_uid, buddy_uid);
        assert_eq!(find_name_accept.first_name.to_string_lossy(), "Dexter");
        assert_eq!(find_name_accept.last_name.to_string_lossy(), "Mandark");

        send_server(
            &mut stream,
            packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC,
            &BuddyMakeSuccess0104 {
                request_id: 77,
                buddy_id: 81,
                buddy_pc_uid: buddy_uid,
            },
            fe_key,
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC,
            &server_state,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC,
            &server_malformed,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC,
            &server_variable,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC,
            &server_malformed_list,
            fe_key,
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC,
            &server_find_name_success,
            fe_key,
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL,
            &server_find_name_accept_failure,
            fe_key,
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC,
            &warp_success,
            fe_key,
        );
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream.try_clone().unwrap()).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    let mut receiver = GameplayReceiver {
        io: TcpFrameIo::from_stream(stream).unwrap(),
        inbound_fe_key: fe_key,
        sender: sender.clone(),
    };

    sender
        .send_buddy_request(&BuddyMakeRequest0104 {
            buddy_id: 81,
            buddy_pc_uid: buddy_uid,
        })
        .unwrap();
    sender
        .send_buddy_accept(&BuddyAcceptRequest0104 {
            accept_flag: 1,
            buddy_id: 81,
            buddy_pc_uid: buddy_uid,
        })
        .unwrap();
    sender
        .send_buddy_state_refresh(&BuddyStateRequest0104 { unused: 0 })
        .unwrap();
    sender
        .send_buddy_block(&BuddySetBlockRequest0104 {
            buddy_pc_uid: buddy_uid,
            buddy_slot: 7,
        })
        .unwrap();
    sender
        .send_buddy_remove(&BuddyRemoveRequest0104 {
            buddy_pc_uid: buddy_uid,
            buddy_slot: 7,
        })
        .unwrap();
    sender
        .send_buddy_warp(&BuddyWarpRequest0104 {
            buddy_pc_uid: buddy_uid,
            buddy_slot: 7,
        })
        .unwrap();
    sender
        .send_buddy_find_name(&BuddyFindNameRequest0104 {
            first_name: FixedUtf16::from_str("Dexter").unwrap(),
            last_name: FixedUtf16::from_str("Mandark").unwrap(),
        })
        .unwrap();
    sender
        .send_buddy_find_name_accept(&BuddyFindNameAcceptRequest0104 {
            accept_flag: 1,
            buddy_pc_uid: buddy_uid,
            first_name: FixedUtf16::from_str("Dexter").unwrap(),
            last_name: FixedUtf16::from_str("Mandark").unwrap(),
        })
        .unwrap();

    let make_success = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &make_success,
        BuddyLifecycleGameplayFrame0104::Decoded {
            packet: BuddyLifecyclePacket0104::MakeSuccess(BuddyMakeSuccess0104 {
                request_id: 77,
                buddy_id: 81,
                buddy_pc_uid,
            }),
            ..
        } if *buddy_pc_uid == buddy_uid
    ));
    assert_eq!(
        make_success.raw_frame().packet_type,
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC
    );

    let decoded_state = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &decoded_state,
        BuddyLifecycleGameplayFrame0104::Decoded {
            packet: BuddyLifecyclePacket0104::StateSuccess(actual),
            ..
        } if actual == &state
    ));

    let malformed = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &malformed,
        BuddyLifecycleGameplayFrame0104::Malformed {
            error: PayloadError::WrongSize {
                expected: BuddyStateSuccess0104::SIZE,
                actual,
            },
            ..
        } if *actual == BuddyStateSuccess0104::SIZE - 1
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);

    let variable = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &variable,
        BuddyLifecycleGameplayFrame0104::Decoded {
            packet: BuddyLifecyclePacket0104::ListInfo(actual),
            ..
        } if actual == &list
    ));
    assert_eq!(variable.raw_frame().payload, variable_list);

    let malformed_variable = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &malformed_variable,
        BuddyLifecycleGameplayFrame0104::Malformed {
            error: PayloadError::WrongSize {
                expected,
                actual,
            },
            ..
        } if *expected == BuddyListInfo0104::HEADER_SIZE + BuddyBaseInfo0104::SIZE
            && *actual == *expected - 1
    ));
    assert_eq!(malformed_variable.raw_frame().payload, malformed_list);

    let find_name = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &find_name,
        BuddyLifecycleGameplayFrame0104::Decoded {
            packet: BuddyLifecyclePacket0104::FindNameSuccess(actual),
            ..
        } if actual == &find_name_success
    ));

    let find_name_accept = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &find_name_accept,
        BuddyLifecycleGameplayFrame0104::Decoded {
            packet: BuddyLifecyclePacket0104::FindNameAcceptFailure(actual),
            ..
        } if actual == &find_name_accept_failure
    ));

    let warp = receiver.read_next_buddy_lifecycle().unwrap();
    assert!(matches!(
        &warp,
        BuddyLifecycleGameplayFrame0104::Decoded {
            packet: BuddyLifecyclePacket0104::WarpOtherShardSuccess(actual),
            ..
        } if actual == &warp_success
    ));

    server.join().unwrap();
}

#[test]
fn gameplay_sender_escort_group_uses_exact_ids_and_four_byte_bodies() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x7766_5544_3322_1100;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        for (sequence, packet_type) in [0x1300_0084, 0x1300_0085].into_iter().enumerate() {
            let frame = read_client(&mut stream, e_key, sequence as u16);
            assert_eq!(frame.packet_type, packet_type);
            assert_eq!(frame.payload, 900_001_i32.to_le_bytes());
        }
    });
    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender.send_escort_invite(900_001).unwrap();
    sender.send_escort_kick(900_001).unwrap();
    server.join().unwrap();
}

#[test]
fn fake_openfusion_gm_speed_round_trips_the_exact_set_value_packets() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x8877_6655_4433_2211;
    let fe_key = 0x1122_3344_5566_7788;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(frame.packet_type, packet::P_CL2FE_GM_REQ_PC_SET_VALUE);
        assert_eq!(frame.payload.len(), GmSetValueRequest0104::SIZE);
        assert_eq!(
            GmSetValueRequest0104::decode(&frame.payload),
            Ok(GmSetValueRequest0104::speed(TEST_PLAYER_ID, 1_200))
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_GM_REP_PC_SET_VALUE,
            &ffone_protocol::GmSetValueReply0104 {
                pc_id: TEST_PLAYER_ID,
                value_type: ffone_protocol::GM_SET_VALUE_SPEED_0104,
                value: 1_200,
            },
            fe_key,
        );
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream.try_clone().unwrap()).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    let mut receiver = GameplayReceiver {
        io: TcpFrameIo::from_stream(stream).unwrap(),
        inbound_fe_key: fe_key,
        sender: sender.clone(),
    };
    sender
        .send_gm_set_value(&GmSetValueRequest0104::speed(TEST_PLAYER_ID, 1_200))
        .unwrap();
    let frame = receiver.read_next().unwrap();
    assert_eq!(frame.packet_type, packet::P_FE2CL_GM_REP_PC_SET_VALUE);
    assert_eq!(
        ffone_protocol::decode_gm_set_value_reply_0104(frame.packet_type, &frame.payload),
        Ok(Some(ffone_protocol::GmSetValueReply0104 {
            pc_id: TEST_PLAYER_ID,
            value_type: ffone_protocol::GM_SET_VALUE_SPEED_0104,
            value: 1_200,
        }))
    );

    server.join().unwrap();
}

#[test]
fn gameplay_sender_all_group_freechat_has_no_invented_target_field() {
    const MESSAGE: &str = "\u{0413}\u{0440}\u{0443}\u{043f}\u{043f}\u{0430} \u{1f680}";

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x7766_5544_3322_1100;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(
            frame.packet_type,
            packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE
        );
        assert_eq!(frame.payload.len(), AllGroupFreeChatRequest0104::SIZE);
        let request = AllGroupFreeChatRequest0104::decode(&frame.payload).unwrap();
        assert_eq!(request.message.to_string_lossy(), MESSAGE);
        assert_eq!(request.emote_code, 19);
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender
        .send_all_group_freechat(&AllGroupFreeChatRequest0104 {
            message: FixedUtf16::from_str(MESSAGE).unwrap(),
            emote_code: 19,
        })
        .unwrap();

    server.join().unwrap();
}

#[test]
fn gameplay_sender_buddy_freechat_preserves_exact_target_uid_and_slot() {
    const MESSAGE: &str =
        "\u{0414}\u{0440}\u{0443}\u{0433} \u{0432} \u{0441}\u{0435}\u{0442}\u{0438} \u{1f680}";

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x6655_4433_2211_0099;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(
            frame.packet_type,
            packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE
        );
        assert_eq!(frame.payload.len(), BuddyFreeChatRequest0104::SIZE);
        let request = BuddyFreeChatRequest0104::decode(&frame.payload).unwrap();
        assert_eq!(request.message.to_string_lossy(), MESSAGE);
        assert_eq!(request.emote_code, -11);
        assert_eq!(request.buddy_pc_uid, 0x0102_0304_0506_0708);
        assert_eq!(request.buddy_slot, -3);
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender
        .send_buddy_freechat(&BuddyFreeChatRequest0104 {
            message: FixedUtf16::from_str(MESSAGE).unwrap(),
            emote_code: -11,
            buddy_pc_uid: 0x0102_0304_0506_0708,
            buddy_slot: -3,
        })
        .unwrap();

    server.join().unwrap();
}

#[test]
fn fake_server_present_npc_types_io_is_typed_strict_and_lossless() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x1122_3344_5566_7788;
    let e_key = 0x8877_6655_4433_2211;
    let request = PresentNpcTypesRequest0104 {
        last_sync_time: 0x0102_0304_0506_0708,
    };
    let reply = PresentNpcTypesReply0104 {
        clear: -7,
        npc_types: vec![2_671, 3_145, -1],
    };
    let malformed_payload = {
        let mut payload = vec![0; 12];
        payload[4..8].copy_from_slice(&2i32.to_le_bytes());
        payload[8..12].copy_from_slice(&2_671i32.to_le_bytes());
        payload
    };
    let unknown_payload = vec![0xde, 0xad, 0xbe, 0xef];
    let server_reply = reply.clone();
    let server_malformed = malformed_payload.clone();
    let server_unknown = unknown_payload.clone();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(frame.packet_type, packet::P_CL2FE_REQ_PRESENT_NPC_TYPES);
        assert_eq!(frame.payload, vec![8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(
            PresentNpcTypesRequest0104::decode(&frame.payload),
            Ok(request)
        );

        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PRESENT_NPC_TYPES,
            &server_reply.encode().unwrap(),
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PRESENT_NPC_TYPES,
            &server_malformed,
            fe_key,
        );
        send_server_bytes(&mut stream, 0x3100_0abc, &server_unknown, fe_key);
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream.try_clone().unwrap()).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    let mut receiver = GameplayReceiver {
        io: TcpFrameIo::from_stream(stream).unwrap(),
        inbound_fe_key: fe_key,
        sender: sender.clone(),
    };

    sender.send_present_npc_types(&request).unwrap();

    let decoded = receiver.read_next_present_npc_types().unwrap();
    assert!(matches!(
        &decoded,
        PresentNpcTypesGameplayFrame0104::Decoded {
            packet: PresentNpcTypesPacket0104::Reply(actual),
            ..
        } if actual == &reply && actual.clears_existing()
    ));
    assert_eq!(
        decoded.raw_frame().payload,
        reply.encode().expect("known valid reply")
    );

    let malformed = receiver.read_next_present_npc_types().unwrap();
    assert!(matches!(
        &malformed,
        PresentNpcTypesGameplayFrame0104::Malformed {
            error: PresentNpcTypesDecodeError0104::Counted(
                CountedPayloadError0104::WrongSize {
                    expected: 16,
                    actual: 12,
                }
            ),
            ..
        }
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);

    let passthrough = receiver.read_next_present_npc_types().unwrap();
    assert!(matches!(
        &passthrough,
        PresentNpcTypesGameplayFrame0104::Passthrough(frame)
            if frame.packet_type == 0x3100_0abc
    ));
    assert_eq!(passthrough.raw_frame().payload, unknown_payload);

    server.join().unwrap();
}
