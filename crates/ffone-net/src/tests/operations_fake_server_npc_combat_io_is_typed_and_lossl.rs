use super::*;

#[test]
fn world_bootstrap_golden_preserves_order_raw_frames_and_unknown_packets() {
    let unknown = DecodedFrame {
        packet_type: 0x3100_0abc,
        flags: 0x123,
        checksum: 0x456,
        payload: vec![0xde, 0xad, 0xbe, 0xef],
    };

    let mut npc_payload = vec![0u8; NpcAppearance0104::SIZE * 2];
    npc_payload[..4].copy_from_slice(&1i32.to_le_bytes());
    let entry = NpcAppearance0104::SIZE;
    for (index, value) in [41i32, 1234, 900, 0x55, 10, 20, 30, 180, 7]
        .into_iter()
        .enumerate()
    {
        let offset = entry + index * 4;
        npc_payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    let npc_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_NPC_AROUND,
        flags: 0x111,
        checksum: 0x222,
        payload: npc_payload,
    };

    let shiny_zero = DecodedFrame {
        packet_type: packet::P_FE2CL_SHINY_AROUND,
        flags: 0,
        checksum: 0,
        payload: vec![0; 4],
    };
    let bootstrap = LoadingComplete {
        response: PcLoadingCompleteSuccess { pc_id: 77 },
        prelude: vec![unknown.clone(), npc_frame.clone(), shiny_zero.clone()],
    }
    .into_world_bootstrap();

    assert_eq!(bootstrap.response.pc_id, 77);
    assert_eq!(
        bootstrap
            .packets
            .iter()
            .map(|packet| packet.raw_frame().packet_type)
            .collect::<Vec<_>>(),
        vec![
            unknown.packet_type,
            packet::P_FE2CL_NPC_AROUND,
            packet::P_FE2CL_SHINY_AROUND,
        ]
    );
    assert_eq!(
        bootstrap.packets[0],
        WorldBootstrapPacket::Passthrough(unknown)
    );
    match &bootstrap.packets[1] {
        WorldBootstrapPacket::InitialEntities { frame, entities } => {
            assert_eq!(frame, &npc_frame);
            assert_eq!(
                entities,
                &InitialAroundPacket0104::Npcs(vec![NpcAppearance0104 {
                    npc_id: 41,
                    npc_type: 1234,
                    hp: 900,
                    condition_bit_flag: 0x55,
                    position: [10, 20, 30],
                    angle: 180,
                    barker_type: 7,
                }])
            );
        }
        other => panic!("expected decoded NPC bucket, got {other:?}"),
    }
    assert!(matches!(
        &bootstrap.packets[2],
        WorldBootstrapPacket::InitialEntities {
            frame,
            entities: InitialAroundPacket0104::Shinies(entries),
        } if frame == &shiny_zero && entries.is_empty()
    ));
    assert!(!bootstrap.has_decode_errors());
    assert_eq!(bootstrap.decode_errors().count(), 0);
}

#[test]
fn world_bootstrap_retains_malformed_around_and_continues_decoding() {
    let malformed = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_AROUND,
        flags: 1,
        checksum: 2,
        payload: vec![0xaa, 0xbb, 0xcc],
    };
    let unknown = DecodedFrame {
        packet_type: 0x3100_0bad,
        flags: 3,
        checksum: 4,
        payload: vec![5, 6, 7],
    };
    let valid_after_error = DecodedFrame {
        packet_type: packet::P_FE2CL_SHINY_AROUND,
        flags: 8,
        checksum: 9,
        payload: vec![0; 4],
    };

    let bootstrap = LoadingComplete {
        response: PcLoadingCompleteSuccess { pc_id: 0 },
        prelude: vec![
            malformed.clone(),
            unknown.clone(),
            valid_after_error.clone(),
        ],
    }
    .into_world_bootstrap();

    assert_eq!(bootstrap.packets.len(), 3);
    assert!(bootstrap.has_decode_errors());
    assert!(matches!(
        &bootstrap.packets[0],
        WorldBootstrapPacket::MalformedInitialEntities { frame, error }
            if frame == &malformed
                && error == &AroundDecodeError::MissingCount { actual: 3 }
    ));
    assert_eq!(
        bootstrap.packets[1],
        WorldBootstrapPacket::Passthrough(unknown)
    );
    assert!(matches!(
        &bootstrap.packets[2],
        WorldBootstrapPacket::InitialEntities {
            frame,
            entities: InitialAroundPacket0104::Shinies(entries),
        } if frame == &valid_after_error && entries.is_empty()
    ));

    let errors = bootstrap.decode_errors().collect::<Vec<_>>();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, 0);
    assert_eq!(errors[0].1, &malformed);
    assert_eq!(errors[0].2, &AroundDecodeError::MissingCount { actual: 3 });
}

#[test]
fn pc_enter_failure_falls_back_to_default_key() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let enter = read_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(enter.packet_type, packet::P_CL2FE_REQ_PC_ENTER);
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_PC_ENTER_FAIL,
            &PcEnterFailure { error_code: 42 },
            DEFAULT_KEY,
        );
    });
    let ticket = ShardTicket {
        endpoint,
        login_id: FixedUtf16::from_str("test-user").unwrap(),
        pc_uid: TEST_UID,
        enter_serial_key: TEST_SERIAL,
        fe_key: derive_frontend_key(44),
    };
    match ShardSession::connect(ticket) {
        Err(NetError::PcEnterRejected { error_code: 42 }) => {}
        Err(other) => panic!("unexpected error: {other}"),
        Ok(_) => panic!("default-key failure was accepted as success"),
    }
    server.join().unwrap();
}

#[test]
fn fake_server_npc_combat_io_is_typed_and_lossless() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x1122_3344_5566_7788;
    let e_key = 0x8877_6655_4433_2211;
    let malformed_payload = vec![0xaa, 0xbb, 0xcc];
    let unknown_payload = vec![9, 8, 7, 6];
    let server_malformed = malformed_payload.clone();
    let server_unknown = unknown_payload.clone();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        let attack = read_client(&mut stream, e_key, 0);
        assert_eq!(attack.packet_type, packet::P_CL2FE_REQ_PC_ATTACK_NPCS);
        assert_eq!(
            PcAttackNpcsRequest0104::decode(&attack.payload).unwrap(),
            PcAttackNpcsRequest0104 {
                npc_ids: vec![41, 99]
            }
        );
        let rocket = read_client(&mut stream, e_key, 1);
        assert_eq!(rocket.packet_type, packet::P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE);
        assert_eq!(
            PcRocketStyleFireRequest0104::decode(&rocket.payload).unwrap(),
            PcRocketStyleFireRequest0104 {
                skill_id: 0,
                position: [1, 2, 3],
                destination: [4, 5, 6],
            }
        );
        let grenade = read_client(&mut stream, e_key, 2);
        assert_eq!(
            grenade.packet_type,
            packet::P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE
        );
        assert_eq!(
            PcGrenadeStyleFireRequest0104::decode(&grenade.payload).unwrap(),
            PcGrenadeStyleFireRequest0104 {
                skill_id: 0,
                destination: [7, 8, 9],
            }
        );
        let begin = read_client(&mut stream, e_key, 3);
        assert_eq!(begin.packet_type, packet::P_CL2FE_REQ_PC_COMBAT_BEGIN);
        assert_eq!(begin.payload, 77i32.to_le_bytes());
        let end = read_client(&mut stream, e_key, 4);
        assert_eq!(end.packet_type, packet::P_CL2FE_REQ_PC_COMBAT_END);
        assert_eq!(end.payload, 77i32.to_le_bytes());

        let heartbeat_payload = 0x1020_3040i32.to_le_bytes();
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REQ_LIVE_CHECK,
            &heartbeat_payload,
            fe_key,
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_NPC_MOVE,
            &NpcMove0104 {
                npc_id: 41,
                destination: [100, 200, 300],
                speed: 450,
                move_style: 1,
            },
            fe_key,
        );
        let heartbeat = read_client(&mut stream, e_key, 5);
        assert_eq!(heartbeat.packet_type, packet::P_CL2FE_REP_LIVE_CHECK);
        assert_eq!(heartbeat.payload, heartbeat_payload);

        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_NPC_ATTACK_PCS,
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

    sender
        .send_attack_npcs(&PcAttackNpcsRequest0104 {
            npc_ids: vec![41, 99],
        })
        .unwrap();
    sender
        .send_rocket_style_fire(&PcRocketStyleFireRequest0104 {
            skill_id: 0,
            position: [1, 2, 3],
            destination: [4, 5, 6],
        })
        .unwrap();
    sender
        .send_grenade_style_fire(&PcGrenadeStyleFireRequest0104 {
            skill_id: 0,
            destination: [7, 8, 9],
        })
        .unwrap();
    sender.send_combat_begin(77).unwrap();
    sender.send_combat_end(77).unwrap();

    assert!(matches!(
        receiver.read_next_npc_combat().unwrap(),
        NpcCombatGameplayFrame0104::Decoded {
            packet: NpcCombatPacket0104::NpcMove(NpcMove0104 {
                npc_id: 41,
                destination: [100, 200, 300],
                speed: 450,
                move_style: 1,
            }),
            ..
        }
    ));

    let malformed = receiver.read_next_npc_combat().unwrap();
    assert!(matches!(
        malformed,
        NpcCombatGameplayFrame0104::Malformed { .. }
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);
    let passthrough = receiver.read_next_npc_combat().unwrap();
    assert!(matches!(
        &passthrough,
        NpcCombatGameplayFrame0104::Passthrough(frame)
            if frame.packet_type == 0x3100_0abc
    ));
    assert_eq!(passthrough.raw_frame().payload, unknown_payload);

    server.join().unwrap();
}

#[test]
fn fake_server_quick_slot_io_is_typed_strict_and_lossless() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x0123_4567_89ab_cdef;
    let e_key = 0xfedc_ba98_7654_3210;
    let roster = QuickSlotInfo0104 {
        slots: std::array::from_fn(|index| QuickSlotEntry0104 {
            item_type: 10 + index as i16,
            item_id: 300 + index as i16,
        }),
    };
    let malformed_payload = vec![0x5a; QuickSlotInfo0104::SIZE - 1];
    let mut variable_payload = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 3,
        slot_num: 4,
        remaining_item: ItemBase0104 {
            item_type: 10,
            item_id: 304,
            option: 2,
            time_limit: 900,
        },
        skill_id: 22,
        pack_padding: [0, 0],
        skill_type: 3,
        target_count: 1,
    }
    .encode_prefix();
    variable_payload.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef, 1, 2, 3, 4]);

    let server_roster = roster;
    let server_malformed = malformed_payload.clone();
    let server_variable = variable_payload.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        let register = read_client(&mut stream, e_key, 0);
        assert_eq!(
            register.packet_type,
            packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT
        );
        assert_eq!(
            QuickSlotRegisterRequest0104::decode(&register.payload),
            Ok(QuickSlotRegisterRequest0104 {
                slot_num: 4,
                item_type: 10,
                item_id: 304,
            })
        );

        let use_item = read_client(&mut stream, e_key, 1);
        assert_eq!(use_item.packet_type, packet::P_CL2FE_REQ_ITEM_USE);
        assert_eq!(&use_item.payload[10..12], &[0, 0]);
        assert_eq!(
            ItemUseRequest0104::decode(&use_item.payload),
            Ok(ItemUseRequest0104 {
                item_location: 3,
                slot_num: 4,
                nano_slot: -1,
            })
        );

        send_server(
            &mut stream,
            packet::P_FE2CL_PC_QUICK_SLOT_INFO,
            &server_roster,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_PC_QUICK_SLOT_INFO,
            &server_malformed,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
            &server_variable,
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
        .send_register_quick_slot(&QuickSlotRegisterRequest0104 {
            slot_num: 4,
            item_type: 10,
            item_id: 304,
        })
        .unwrap();
    sender
        .send_item_use(&ItemUseRequest0104 {
            item_location: 3,
            slot_num: 4,
            nano_slot: -1,
        })
        .unwrap();

    let decoded = receiver.read_next_quick_slot().unwrap();
    assert!(matches!(
        &decoded,
        QuickSlotGameplayFrame0104::Decoded {
            packet: QuickSlotPacket0104::Info(actual),
            ..
        } if actual == &roster
    ));
    assert_eq!(decoded.raw_frame().payload, roster.encode());

    let malformed = receiver.read_next_quick_slot().unwrap();
    assert!(matches!(
        &malformed,
        QuickSlotGameplayFrame0104::Malformed {
            error: PayloadError::WrongSize {
                expected: QuickSlotInfo0104::SIZE,
                actual,
            },
            ..
        } if *actual == QuickSlotInfo0104::SIZE - 1
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);

    let variable = receiver.read_next_quick_slot().unwrap();
    assert!(matches!(
        &variable,
        QuickSlotGameplayFrame0104::Passthrough(frame)
            if frame.packet_type == packet::P_FE2CL_REP_PC_ITEM_USE_SUCC
    ));
    assert_eq!(variable.raw_frame().payload, variable_payload);

    server.join().unwrap();
}

#[test]
fn item_use_gameplay_classifier_is_typed_strict_and_lossless() {
    let prefix = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 1,
        slot_num: 4,
        remaining_item: ItemBase0104 {
            item_type: 7,
            item_id: 304,
            option: 1,
            time_limit: 900,
        },
        skill_id: 144,
        pack_padding: [0, 0],
        skill_type: 33,
        target_count: 1,
    };
    let mut payload = prefix.encode_prefix();
    payload.resize(
        ItemUseSuccessPrefix0104::SIZE + ffone_protocol::SkillResultBuff0104::SIZE,
        0,
    );
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
        flags: 0x123,
        checksum: 0x456,
        payload: payload.clone(),
    };
    let decoded = ItemUseGameplayFrame0104::decode(frame.clone());
    assert!(matches!(
        &decoded,
        ItemUseGameplayFrame0104::Decoded {
            frame: retained,
            packet: ItemUsePacket0104::Success(packet),
        } if retained == &frame
            && packet.prefix() == prefix
            && matches!(packet.results(), ItemUseSkillResults0104::Buff(records) if records.len() == 1)
    ));
    assert_eq!(decoded.raw_frame(), &frame);

    let unsupported_prefix = ItemUseSuccessPrefix0104 {
        skill_type: 7,
        ..prefix
    };
    let malformed_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
        flags: 0x234,
        checksum: 0x567,
        payload: unsupported_prefix.encode_prefix(),
    };
    let malformed = ItemUseGameplayFrame0104::decode(malformed_frame.clone());
    assert!(matches!(
        &malformed,
        ItemUseGameplayFrame0104::Malformed {
            frame: retained,
            error: ItemUseDecodeError0104::UnsupportedPositiveTargetSkillType {
                skill_type: 7,
                target_count: 1,
            },
        } if retained == &malformed_frame
    ));
    assert_eq!(malformed.raw_frame(), &malformed_frame);

    let unknown_frame = DecodedFrame {
        packet_type: 0x3100_0abc,
        flags: 0x345,
        checksum: 0x678,
        payload: vec![0xde, 0xad],
    };
    let passthrough = ItemUseGameplayFrame0104::decode(unknown_frame.clone());
    assert!(matches!(
        &passthrough,
        ItemUseGameplayFrame0104::Passthrough(retained) if retained == &unknown_frame
    ));
    assert_eq!(passthrough.raw_frame(), &unknown_frame);
}

#[test]
fn fake_server_item_use_io_preserves_typed_and_malformed_frames() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x1020_3040_5060_7080;

    let success_prefix = ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 1,
        slot_num: 4,
        remaining_item: ItemBase0104 {
            item_type: 7,
            item_id: 304,
            option: 1,
            time_limit: 900,
        },
        skill_id: 144,
        pack_padding: [0, 0],
        skill_type: 33,
        target_count: 1,
    };
    let mut success_payload = success_prefix.encode_prefix();
    success_payload.resize(
        ItemUseSuccessPrefix0104::SIZE + ffone_protocol::SkillResultBuff0104::SIZE,
        0,
    );
    let malformed_payload = ItemUseSuccessPrefix0104 {
        skill_type: 7,
        ..success_prefix
    }
    .encode_prefix();
    let broadcast_prefix = ffone_protocol::ItemUseBroadcastPrefix0104 {
        pc_id: 88,
        skill_id: 10,
        pack_padding: [0, 0],
        skill_type: 1,
        target_count: 1,
    };
    let mut broadcast_payload = broadcast_prefix.encode_prefix();
    broadcast_payload.resize(
        ffone_protocol::ItemUseBroadcastPrefix0104::SIZE
            + ffone_protocol::SkillResultDamage0104::SIZE,
        0,
    );

    let server_success = success_payload.clone();
    let server_malformed = malformed_payload.clone();
    let server_broadcast = broadcast_payload.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
            &server_success,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
            &server_malformed,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_PC_ITEM_USE,
            &server_broadcast,
            fe_key,
        );
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream.try_clone().unwrap()).unwrap(),
            outbound_e_key: 0,
        })),
    };
    let mut receiver = GameplayReceiver {
        io: TcpFrameIo::from_stream(stream).unwrap(),
        inbound_fe_key: fe_key,
        sender,
    };

    let success = receiver.read_next_item_use().unwrap();
    assert!(matches!(
        &success,
        ItemUseGameplayFrame0104::Decoded {
            packet: ItemUsePacket0104::Success(packet),
            ..
        } if packet.prefix() == success_prefix
    ));
    assert_eq!(success.raw_frame().payload, success_payload);

    let malformed = receiver.read_next_item_use().unwrap();
    assert!(matches!(
        &malformed,
        ItemUseGameplayFrame0104::Malformed {
            error: ItemUseDecodeError0104::UnsupportedPositiveTargetSkillType {
                skill_type: 7,
                target_count: 1,
            },
            ..
        }
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);

    let broadcast = receiver.read_next_item_use().unwrap();
    assert!(matches!(
        &broadcast,
        ItemUseGameplayFrame0104::Decoded {
            packet: ItemUsePacket0104::Broadcast(packet),
            ..
        } if packet.prefix() == broadcast_prefix
    ));
    assert_eq!(broadcast.raw_frame().payload, broadcast_payload);

    server.join().unwrap();
}

#[test]
fn fake_server_vendor_io_is_typed_strict_and_lossless() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x2211_4433_6655_8877;
    let table = VendorTableUpdateSuccess0104 {
        items: [ItemVendor0104 {
            vendor_id: 650,
            buy_cost: 425.5,
            item: ItemBase0104 {
                item_type: 4,
                item_id: 104,
                option: 1,
                time_limit: 0,
            },
            sort_num: 8,
        }; VENDOR_TABLE_ITEM_COUNT_0104],
    };
    let malformed_payload = vec![0x5a; VendorStartSuccess0104::SIZE - 1];
    let unknown_payload = vec![0xde, 0xad, 0xbe, 0xef];
    let server_table = table;
    let server_malformed = malformed_payload.clone();
    let server_unknown = unknown_payload.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC,
            &server_table,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_PC_VENDOR_START_SUCC,
            &server_malformed,
            fe_key,
        );
        send_server_bytes(&mut stream, 0x3100_0abc, &server_unknown, fe_key);
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream.try_clone().unwrap()).unwrap(),
            outbound_e_key: 0,
        })),
    };
    let mut receiver = GameplayReceiver {
        io: TcpFrameIo::from_stream(stream).unwrap(),
        inbound_fe_key: fe_key,
        sender,
    };

    let decoded = receiver.read_next_vendor().unwrap();
    assert!(matches!(
        &decoded,
        VendorGameplayFrame0104::Decoded {
            packet: VendorPacket0104::TableSuccess(actual),
            ..
        } if actual == &table
    ));
    assert_eq!(decoded.raw_frame().payload, table.encode());

    let malformed = receiver.read_next_vendor().unwrap();
    assert!(matches!(
        &malformed,
        VendorGameplayFrame0104::Malformed {
            error: PayloadError::WrongSize {
                expected: VendorStartSuccess0104::SIZE,
                actual,
            },
            ..
        } if *actual == VendorStartSuccess0104::SIZE - 1
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);

    let passthrough = receiver.read_next_vendor().unwrap();
    assert!(matches!(
        &passthrough,
        VendorGameplayFrame0104::Passthrough(frame)
            if frame.packet_type == 0x3100_0abc
    ));
    assert_eq!(passthrough.raw_frame().payload, unknown_payload);

    server.join().unwrap();
}
