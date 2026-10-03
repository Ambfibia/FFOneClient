use super::*;

pub(super) fn send_tutorial_server<P: WirePayload>(
    stream: &mut TcpStream,
    packet_type: u32,
    payload: &P,
    key: u64,
) {
    let frame = encode_server_frame(packet_type, &payload.encode(), key).unwrap();
    stream.write_all(&frame).unwrap();
}

#[test]
fn character_creation_worker_retries_rejections_and_keeps_delayed_reservation_atomic() {
    use ffone_protocol::{
        CharacterNameCheckFailure0104, PcStyle2Flags0104, PcStyle0104, ShardSelectFailure,
    };
    let (idle_done_tx, idle_done_rx) = mpsc::channel();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        assert_eq!(
            read_tutorial_client(&mut stream, DEFAULT_KEY, 0).packet_type,
            packet::P_CL2LS_REQ_LOGIN
        );
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_LOGIN_SUCC,
            &LoginSuccess {
                character_count: 0,
                selected_slot: 1,
                payment_flag: 1,
                packing_byte: 0,
                server_time: TUTORIAL_TEST_SERVER_TIME,
                id: FixedUtf16::from_str("creator").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );
        let key = derive_login_e_key(TUTORIAL_TEST_SERVER_TIME, 0, 1);
        assert_eq!(
            read_tutorial_client(&mut stream, key, 1).packet_type,
            packet::P_CL2LS_REQ_CHECK_CHAR_NAME
        );
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_CHECK_CHAR_NAME_FAIL,
            &CharacterNameCheckFailure0104 { error_code: 1 },
            key,
        );
        let check = read_tutorial_client(&mut stream, key, 2);
        assert_eq!(check.packet_type, packet::P_CL2LS_REQ_CHECK_CHAR_NAME);
        let check = CharacterNameCheckRequest0104::decode(&check.payload).unwrap();
        thread::sleep(Duration::from_millis(40));
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_CHECK_CHAR_NAME_SUCC,
            &CharacterNameCheckSuccess0104 {
                first_name: check.first_name,
                last_name: check.last_name,
            },
            key,
        );
        // No Bevy event drain or extra SaveCharacterName command is needed.
        let save = read_tutorial_client(&mut stream, key, 3);
        assert_eq!(save.packet_type, packet::P_CL2LS_REQ_SAVE_CHAR_NAME);
        let save = CharacterNameSaveRequest0104::decode(&save.payload).unwrap();
        assert_eq!((save.slot, save.gender), (2, 1));
        thread::sleep(Duration::from_millis(40));
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_SAVE_CHAR_NAME_SUCC,
            &CharacterNameSaveSuccess0104 {
                pc_uid: TUTORIAL_TEST_UID,
                slot: save.slot,
                gender: save.gender,
                first_name: save.first_name,
                last_name: save.last_name,
            },
            key,
        );
        // No UI commands until all keepalives are answered. Split the wire
        // length and body to cover TCP fragmentation without losing framing.
        for sequence in 4..7 {
            let payload = (sequence as i32).to_le_bytes();
            let wire = encode_server_frame(packet::P_LS2CL_REQ_LIVE_CHECK, &payload, key).unwrap();
            stream.write_all(&wire[..2]).unwrap();
            thread::sleep(Duration::from_millis(150));
            stream.write_all(&wire[2..]).unwrap();
            let reply = read_tutorial_client(&mut stream, key, sequence);
            assert_eq!(reply.packet_type, packet::P_CL2LS_REP_LIVE_CHECK);
            assert_eq!(reply.payload, payload);
        }
        idle_done_tx.send(()).unwrap();
        let create = read_tutorial_client(&mut stream, key, 7);
        assert_eq!(create.packet_type, packet::P_CL2LS_REQ_CHAR_CREATE);
        // OpenFusion's invalidCharacter uses SHARD_SELECT_FAIL even here.
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_SHARD_SELECT_FAIL,
            &ShardSelectFailure { error_code: 2 },
            key,
        );
        let retry = read_tutorial_client(&mut stream, key, 8);
        assert_eq!(retry.packet_type, packet::P_CL2LS_REQ_CHAR_CREATE);
        assert_eq!(
            retry.payload, create.payload,
            "retry preserves the reserved identity and appearance"
        );
        let request = CharacterCreateRequest0104::decode(&retry.payload).unwrap();
        assert_eq!(
            (
                request.style.gender,
                request.style.skin_color,
                request.style.hair_color,
                request.style.eye_color
            ),
            (2, 36, 54, 10)
        );
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_CREATE_SUCC,
            &CharacterCreateSuccess0104 {
                level: 1,
                style: request.style,
                style2: PcStyle2Flags0104 {
                    appearance_flag: 1,
                    tutorial_flag: 0,
                    payzone_flag: 0,
                },
                equipped: request.equipped,
            },
            key,
        );
    });
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));
    command_tx
        .send(NetworkCommand::Login {
            login_address: address.to_string(),
            username: "creator".into(),
            password: "fixture-only".into(),
        })
        .unwrap();
    let receive = || event_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(receive(), NetworkEvent::Connecting));
    assert!(matches!(receive(), NetworkEvent::LoginMetadata { .. }));
    assert!(matches!(receive(), NetworkEvent::Characters(roster) if roster.is_empty()));
    let reserve = || NetworkCommand::ReserveCharacterName {
        request: CharacterNameCheckRequest0104::new("Test", "Creator", 0, 0, 0).unwrap(),
        slot: 2,
        gender: 1,
    };
    command_tx.send(reserve()).unwrap();
    assert!(matches!(
        receive(),
        NetworkEvent::CharacterOperationRejected {
            stage: CharacterOperationStage::NameCheck,
            error_code: 1
        }
    ));
    command_tx.send(reserve()).unwrap();
    let NetworkEvent::CharacterNameSaved(saved) = receive() else {
        panic!("reservation must finish without a UI roundtrip")
    };
    idle_done_rx.recv_timeout(Duration::from_secs(3)).expect("idle creation must answer heartbeats without a UI command");
    let request = CharacterCreateRequest0104 {
        style: PcStyle0104 {
            pc_uid: saved.pc_uid,
            first_name: saved.first_name,
            last_name: saved.last_name,
            gender: 2,
            skin_color: 36,
            hair_color: 54,
            eye_color: 10,
            face_style: 6,
            hair_style: 25,
            name_check: 0,
            height: 0,
            body: 0,
            class: 0,
        },
        equipped: Default::default(),
        selected_indices: Default::default(),
    };
    command_tx
        .send(NetworkCommand::CreateCharacter(request.clone()))
        .unwrap();
    assert!(matches!(
        receive(),
        NetworkEvent::CharacterOperationRejected {
            stage: CharacterOperationStage::Appearance,
            error_code: 2
        }
    ));
    command_tx
        .send(NetworkCommand::CreateCharacter(request))
        .unwrap();
    assert!(
        matches!(receive(), NetworkEvent::CharacterCreated { slot: 2, response } if response.style.pc_uid == TUTORIAL_TEST_UID)
    );
    assert!(
        matches!(receive(), NetworkEvent::Characters(roster) if roster.len() == 1 && roster[0].pc_uid == TUTORIAL_TEST_UID && roster[0].style.appearance_flag == 1)
    );
    server.join().unwrap();
    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn login_debug_output_redacts_password() {
    let command = NetworkCommand::Login {
        login_address: "127.0.0.1:23000".to_owned(),
        username: "Ambfibia".to_owned(),
        password: "do-not-log-this".to_owned(),
    };
    let debug = format!("{command:?}");
    assert!(debug.contains("Ambfibia"));
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("do-not-log-this"));

    let duplicate = NetworkCommand::ExitDuplicateSession {
        login_address: "127.0.0.1:23000".to_owned(),
        username: "Ambfibia".to_owned(),
        password: "also-do-not-log-this".to_owned(),
    };
    let debug = format!("{duplicate:?}");
    assert!(debug.contains("Ambfibia"));
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("also-do-not-log-this"));
}

#[test]
fn incoming_fixed_frames_fail_closed_while_unknown_frames_remain_lossless() {
    let malformed = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_MOVE,
        flags: 7,
        checksum: 7,
        payload: vec![0; 3],
    };
    assert!(matches!(
        NetworkEvent::incoming_frame_0104(NetworkStream0104::Shard, malformed.clone()),
        NetworkEvent::MalformedFrame0104 {
            stream: NetworkStream0104::Shard,
            frame,
            expected_payload_size,
        } if frame == malformed && expected_payload_size == ffone_protocol::PcMove0104::SIZE
    ));

    let valid = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_MOVE,
        flags: 9,
        checksum: 9,
        payload: vec![0; ffone_protocol::PcMove0104::SIZE],
    };
    assert!(matches!(
        NetworkEvent::incoming_frame_0104(NetworkStream0104::Shard, valid.clone()),
        NetworkEvent::Frame(frame) if frame == valid
    ));

    let malformed_server_message = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_MOTD_LOGIN,
        flags: 10,
        checksum: 10,
        payload: vec![0; ffone_protocol::ServerMessage0104::SIZE - 1],
    };
    assert!(matches!(
        NetworkEvent::incoming_frame_0104(
            NetworkStream0104::Shard,
            malformed_server_message.clone(),
        ),
        NetworkEvent::MalformedFrame0104 {
            stream: NetworkStream0104::Shard,
            frame,
            expected_payload_size,
        } if frame == malformed_server_message
            && expected_payload_size == ffone_protocol::ServerMessage0104::SIZE
    ));

    let malformed_gm_value = DecodedFrame {
        packet_type: packet::P_FE2CL_GM_REP_PC_SET_VALUE,
        flags: 12,
        checksum: 12,
        payload: vec![0; ffone_protocol::GmSetValueReply0104::SIZE - 1],
    };
    assert!(matches!(
        NetworkEvent::incoming_frame_0104(
            NetworkStream0104::Shard,
            malformed_gm_value.clone(),
        ),
        NetworkEvent::MalformedFrame0104 {
            stream: NetworkStream0104::Shard,
            frame,
            expected_payload_size,
        } if frame == malformed_gm_value
            && expected_payload_size == ffone_protocol::GmSetValueReply0104::SIZE
    ));

    let goto = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_GOTO_SUCC,
        flags: 13,
        checksum: 13,
        payload: PcGotoSuccess0104 {
            position: [54_700, 65_500, -10_540],
        }
        .encode(),
    };
    assert!(matches!(
        NetworkEvent::incoming_frame_0104(NetworkStream0104::Shard, goto.clone()),
        NetworkEvent::Frame(frame) if frame == goto
    ));
    let malformed_goto = DecodedFrame {
        payload: vec![0; PcGotoSuccess0104::SIZE - 1],
        ..goto
    };
    assert!(matches!(
        NetworkEvent::incoming_frame_0104(
            NetworkStream0104::Shard,
            malformed_goto.clone(),
        ),
        NetworkEvent::MalformedFrame0104 {
            stream: NetworkStream0104::Shard,
            frame,
            expected_payload_size,
        } if frame == malformed_goto && expected_payload_size == PcGotoSuccess0104::SIZE
    ));

    let unknown = DecodedFrame {
        packet_type: 0xfe00_0104,
        flags: 11,
        checksum: 11,
        payload: vec![1, 2, 3, 4, 5],
    };
    assert!(matches!(
        NetworkEvent::incoming_frame_0104(NetworkStream0104::Login, unknown.clone()),
        NetworkEvent::LoginFrame(frame) if frame == unknown
    ));
}

#[test]
fn complete_tutorial_without_login_is_terminal_and_typed() {
    let command = NetworkCommand::CompleteTutorial {
        pc_uid: TUTORIAL_TEST_UID,
    };
    let debug = format!("{command:?}");
    assert!(debug.contains("CompleteTutorial"));
    assert!(debug.contains(&TUTORIAL_TEST_UID.to_string()));

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));

    command_tx.send(command).unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::TutorialExitFailed { pc_uid, error })
            if pc_uid == TUTORIAL_TEST_UID
                && error == "tutorial completion requires a successful login"
    ));

    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn retained_login_reselects_character_after_tutorial_completion() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let login = read_tutorial_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(login.packet_type, packet::P_CL2LS_REQ_LOGIN);

        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_LOGIN_SUCC,
            &LoginSuccess {
                character_count: 1,
                selected_slot: 0,
                payment_flag: 1,
                packing_byte: 0,
                server_time: TUTORIAL_TEST_SERVER_TIME,
                id: FixedUtf16::from_str("retained-login").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );
        let login_e_key = derive_login_e_key(TUTORIAL_TEST_SERVER_TIME, 1, 0);
        let mut character = CharacterInfo0104::zeroed();
        character.as_bytes_mut()[4..12].copy_from_slice(&TUTORIAL_TEST_UID.to_le_bytes());
        character.as_bytes_mut()[81] = 0;
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_INFO,
            &character,
            login_e_key,
        );

        let initial_select = read_tutorial_client(&mut stream, login_e_key, 1);
        assert_eq!(initial_select.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
        send_tutorial_shard_selection(&mut stream, login_e_key, 23_001);

        let save = read_tutorial_client(&mut stream, login_e_key, 2);
        assert_eq!(save.packet_type, packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR);
        assert_eq!(
            CharacterTutorialSaveRequest0104::decode(&save.payload),
            Ok(CharacterTutorialSaveRequest0104 {
                pc_uid: TUTORIAL_TEST_UID,
                tutorial_flag: 1,
            })
        );

        let open_world_select = read_tutorial_client(&mut stream, login_e_key, 3);
        assert_eq!(
            CharacterSelectRequest::decode(&open_world_select.payload),
            Ok(CharacterSelectRequest {
                pc_uid: TUTORIAL_TEST_UID,
            })
        );
        send_tutorial_shard_selection(&mut stream, login_e_key, 23_002);
    });

    let mut login =
        LoginSession::connect_password(address, "retained-login", "secret").unwrap();
    let characters = login.characters().to_vec();
    let initial_ticket = login.select_character(TUTORIAL_TEST_UID).unwrap();
    assert_eq!(initial_ticket.endpoint().port(), 23_001);

    let connection = login.into_world_connection().unwrap();
    let (sender, mut receiver) = connection.into_parts();
    let shutting_down = Arc::new(AtomicBool::new(false));
    let (selection_sender, selection_events) = mpsc::channel();
    let (event_sender, _event_receiver) = mpsc::sync_channel(8);
    spawn_login_world_reader(&event_sender, &shutting_down, selection_sender, move || {
        receiver.read_next()
    });
    let mut retained = LoginWorldHandle {
        sender,
        shutting_down,
        selection_events,
        characters,
    };

    let (style, open_world_ticket) = request_retained_shard_ticket(
        &mut retained,
        TUTORIAL_TEST_UID,
        CharacterEntryRoute::TutorialCompletion,
    )
    .unwrap();
    assert_eq!(style.tutorial_flag, 1);
    assert_eq!(open_world_ticket.pc_uid(), TUTORIAL_TEST_UID);
    assert_eq!(open_world_ticket.endpoint().port(), 23_002);
    assert_eq!(retained.characters[0].style().tutorial_flag, 1);

    retained.stop();
    server.join().unwrap();
}

#[test]
fn retained_login_answers_clean_char_select_succ_with_shard_select() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let login = read_tutorial_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(login.packet_type, packet::P_CL2LS_REQ_LOGIN);

        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_LOGIN_SUCC,
            &LoginSuccess {
                character_count: 1,
                selected_slot: 0,
                payment_flag: 1,
                packing_byte: 0,
                server_time: TUTORIAL_TEST_SERVER_TIME,
                id: FixedUtf16::from_str("clean-login").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );
        let login_e_key = derive_login_e_key(TUTORIAL_TEST_SERVER_TIME, 1, 0);
        let mut character = CharacterInfo0104::zeroed();
        character.as_bytes_mut()[4..12].copy_from_slice(&TUTORIAL_TEST_UID.to_le_bytes());
        character.as_bytes_mut()[81] = 0;
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_INFO,
            &character,
            login_e_key,
        );

        let initial_select = read_tutorial_client(&mut stream, login_e_key, 1);
        assert_eq!(initial_select.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
        send_tutorial_shard_selection(&mut stream, login_e_key, 23_001);

        let save = read_tutorial_client(&mut stream, login_e_key, 2);
        assert_eq!(save.packet_type, packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR);

        let open_world_select = read_tutorial_client(&mut stream, login_e_key, 3);
        assert_eq!(
            open_world_select.packet_type,
            packet::P_CL2LS_REQ_CHAR_SELECT
        );
        // Original login-server route: acknowledge first, then expect the
        // shard choice on the retained socket before the ticket.
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_SELECT_SUCC,
            &ffone_protocol::wire_0104::LsCharSelectSuccess0104,
            login_e_key,
        );
        let shard_select = read_tutorial_client(&mut stream, login_e_key, 4);
        assert_eq!(shard_select.packet_type, packet::P_CL2LS_REQ_SHARD_SELECT);
        assert_eq!(
            shard_select.payload,
            vec![ffone_net::CLEAN_WEB_PLAYER_SHARD_NUM_0104 as u8]
        );
        send_tutorial_shard_selection(&mut stream, login_e_key, 23_002);
    });

    let mut login = LoginSession::connect_password(address, "clean-login", "secret").unwrap();
    let characters = login.characters().to_vec();
    let initial_ticket = login.select_character(TUTORIAL_TEST_UID).unwrap();
    assert_eq!(initial_ticket.endpoint().port(), 23_001);

    let connection = login.into_world_connection().unwrap();
    let (sender, mut receiver) = connection.into_parts();
    let shutting_down = Arc::new(AtomicBool::new(false));
    let (selection_sender, selection_events) = mpsc::channel();
    let (event_sender, _event_receiver) = mpsc::sync_channel(8);
    spawn_login_world_reader(&event_sender, &shutting_down, selection_sender, move || {
        receiver.read_next()
    });
    let mut retained = LoginWorldHandle {
        sender,
        shutting_down,
        selection_events,
        characters,
    };

    let (_, open_world_ticket) = request_retained_shard_ticket(
        &mut retained,
        TUTORIAL_TEST_UID,
        CharacterEntryRoute::TutorialCompletion,
    )
    .unwrap();
    assert_eq!(open_world_ticket.endpoint().port(), 23_002);

    retained.stop();
    server.join().unwrap();
}

#[test]
fn group_leave_worker_routes_through_the_gameplay_sender_guard() {
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));

    command_tx
        .send(NetworkCommand::LeaveGroup(GroupLeaveRequest0104 {
            unused: 0,
        }))
        .unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Error(message))
            if message == "gameplay packet requested before entering the world"
    ));

    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn tutorial_completion_ignores_contaminated_saved_positions_and_enters_sector_v() {
    assert_eq!(
        character_entry_location(
            CharacterEntryRoute::TutorialCompletion,
            CharacterEntryLocation0104::Saved,
        ),
        CharacterEntryLocation0104::TutorialCompletionSectorV
    );
    let (position, angle, request) = character_entry_pose(
        [54_700, 65_500, -10_540],
        270,
        CharacterEntryLocation0104::TutorialCompletionSectorV,
    );
    assert_eq!(position, TUTORIAL_COMPLETION_SECTOR_V_POSITION_0104);
    assert_eq!(angle, TUTORIAL_COMPLETION_SECTOR_V_ANGLE_0104);
    assert_eq!(
        request.map(|request| (request.position, request.angle)),
        Some((
            TUTORIAL_COMPLETION_SECTOR_V_POSITION_0104,
            TUTORIAL_COMPLETION_SECTOR_V_ANGLE_0104,
        )),
        "the canonical Sector V pose must reach the shard before loading completes"
    );

    let scripted = CharacterEntryLocation0104::Scripted {
        position: [54_700, 65_500, -10_540],
        angle: 270,
    };
    assert_eq!(
        character_entry_location(CharacterEntryRoute::Selection, scripted),
        scripted,
        "ordinary selection must preserve its caller-owned entry policy"
    );
}

#[test]
fn world_ready_is_published_before_and_preserves_freechat_frames() {
    let (events, receiver) = mpsc::sync_channel(32);
    let shutting_down = Arc::new(AtomicBool::new(false));
    let reader_finished = Arc::new(GameplayReaderFinished::default());
    let world = WorldReady {
        pc_uid: 8,
        player_id: 81,
        server_time: 0x1122_3344_5566_7788,
        map_number: 0,
        hp: 1000,
        position: [632_032, 187_177, -5_500],
        angle: 90,
        login_style: CharacterStyle0104 {
            name_check: 1,
            gender: 1,
            face_style: 0,
            hair_style: 0,
            hair_color: 0,
            skin_color: 0,
            eye_color: 0,
            height: 0,
            body: 0,
            class: 0,
            appearance_flag: 1,
            tutorial_flag: 1,
            payzone_flag: 0,
        },
        load: PcLoadData0104::zeroed(),
        bootstrap: WorldBootstrap {
            response: PcLoadingCompleteSuccess { pc_id: 81 },
            packets: Vec::new(),
        },
    };
    let payload = FreeChatSuccess0104 {
        pc_id: 81,
        message: FixedUtf16::from_str("Привет 🌌").unwrap(),
        emote_code: 12,
    }
    .encode();
    let normal_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_FREECHAT_SUCC,
        flags: 0x123,
        checksum: 0x456,
        payload,
    };
    let group_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC,
        flags: 0x234,
        checksum: 0x567,
        payload: AllGroupFreeChatSuccess0104 {
            sender_pc_id: 91,
            message: FixedUtf16::from_str(
                "\u{0413}\u{0440}\u{0443}\u{043f}\u{043f}\u{0430} \u{1f680}",
            )
            .unwrap(),
            emote_code: 18,
        }
        .encode(),
    };
    let buddy_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC,
        flags: 0x345,
        checksum: 0x678,
        payload: BuddyFreeChatSuccess0104 {
            from_pc_uid: 0x0102_0304_0506_0708,
            to_pc_uid: 0x1112_1314_1516_1718,
            message: FixedUtf16::from_str(
                "\u{0414}\u{0440}\u{0443}\u{0433} \u{0432} \u{0441}\u{0435}\u{0442}\u{0438}",
            )
            .unwrap(),
            emote_code: 29,
        }
        .encode(),
    };
    let expected_frames = [
        normal_frame.clone(),
        group_frame.clone(),
        buddy_frame.clone(),
    ];
    let mut frames = vec![normal_frame, group_frame, buddy_frame].into_iter();

    assert!(publish_world_then_spawn_gameplay_reader(
        &events,
        &shutting_down,
        &reader_finished,
        world,
        move || match frames.next() {
            Some(frame) => Ok(frame),
            None => Err(ffone_net::NetError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "test reader complete",
            ))),
        },
    ));

    match receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
        NetworkEvent::WorldReady(actual) => {
            assert_eq!(actual.server_time, 0x1122_3344_5566_7788);
            assert_eq!(actual.login_style.tutorial_flag, 1);
            assert_eq!(
                actual.load.style_flags(),
                [0, 0, 0],
                "the OpenFusion shard load may omit PCStyle2 without erasing the login flags"
            );
        }
        other => panic!("expected WorldReady before live frames, got {other:?}"),
    }
    for expected in expected_frames {
        match receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            NetworkEvent::Frame(actual) => assert_eq!(actual, expected),
            other => panic!("expected live frame after WorldReady, got {other:?}"),
        }
    }
    shutting_down.store(true, Ordering::Release);
}

#[test]
fn graceful_exit_reader_delivers_success_then_suppresses_expected_eof() {
    let (events, receiver) = mpsc::sync_channel(32);
    let shutting_down = Arc::new(AtomicBool::new(true));
    let reader_finished = Arc::new(GameplayReaderFinished::default());
    let success = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_EXIT_SUCC,
        flags: 0x123,
        checksum: 0x456,
        payload: PcExitSuccess0104 {
            pc_id: 81,
            exit_code: 1,
        }
        .encode(),
    };
    let expected = success.clone();
    let mut frames = Some(success);

    spawn_gameplay_reader(
        &events,
        &shutting_down,
        &reader_finished,
        move || match frames.take() {
            Some(frame) => Ok(frame),
            None => Err(ffone_net::NetError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "server closed after PC exit success",
            ))),
        },
    );

    match receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
        NetworkEvent::Frame(actual) => assert_eq!(actual, expected),
        other => panic!("expected PC exit success frame, got {other:?}"),
    }
    assert!(matches!(
        receiver.recv_timeout(Duration::from_millis(100)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    assert!(reader_finished.wait(Duration::from_secs(1)));
}

#[test]
fn retained_login_transport_failure_marks_session_unusable() {
    let (events, receiver) = mpsc::sync_channel(8);
    let (selection_sender, selection_receiver) = mpsc::channel();
    let shutting_down = Arc::new(AtomicBool::new(false));
    spawn_login_world_reader(&events, &shutting_down, selection_sender, || {
        Err(ffone_net::NetError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "retained login closed",
        )))
    });
    assert!(matches!(
        selection_receiver.recv_timeout(Duration::from_secs(1)),
        Ok(RetainedLoginSelection::TransportFailed(_))
    ));
    assert!(shutting_down.load(Ordering::Acquire));
    assert!(matches!(
        receiver.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Disconnected {
            reason: DisconnectReason0104::LoginTransportFailed(_)
        })
    ));
}
