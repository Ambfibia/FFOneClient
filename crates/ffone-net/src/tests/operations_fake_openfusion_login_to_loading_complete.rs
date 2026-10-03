use super::*;

pub(super) fn send_server_bytes(stream: &mut TcpStream, packet_type: u32, payload: &[u8], key: u64) {
    let frame = encode_server_frame(packet_type, payload, key).unwrap();
    stream.write_all(&frame).unwrap();
}

pub(super) fn send_server<P: WirePayload>(
    stream: &mut TcpStream,
    packet_type: u32,
    payload: &P,
    key: u64,
) {
    send_server_bytes(stream, packet_type, &payload.encode(), key);
}

pub(super) fn put_utf16<const N: usize>(bytes: &mut [u8], offset: usize, value: &FixedUtf16<N>) {
    for (index, unit) in value.as_units().iter().enumerate() {
        let start = offset + index * 2;
        bytes[start..start + 2].copy_from_slice(&unit.to_le_bytes());
    }
}

#[test]
fn fake_openfusion_character_creation_and_delete_preserve_login_sequence() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let uid = TEST_UID;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let login_frame = read_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(login_frame.packet_type, packet::P_CL2LS_REQ_LOGIN);
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_LOGIN_SUCC,
            &LoginSuccess {
                character_count: 0,
                selected_slot: 1,
                payment_flag: 1,
                packing_byte: 0,
                server_time: TEST_SERVER_TIME,
                id: FixedUtf16::from_str("creator").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );
        let e_key = derive_login_e_key(TEST_SERVER_TIME, 0, 1);

        let check_frame = read_client(&mut stream, e_key, 1);
        assert_eq!(check_frame.packet_type, packet::P_CL2LS_REQ_CHECK_CHAR_NAME);
        let check = CharacterNameCheckRequest0104::decode(&check_frame.payload).unwrap();
        assert_eq!(check.first_name.to_string_lossy(), "Proto");
        assert_eq!(check.last_name.to_string_lossy(), "Hero");

        let heartbeat_payload = 0x1122_3344i32.to_le_bytes();
        send_server_bytes(
            &mut stream,
            packet::P_LS2CL_REQ_LIVE_CHECK,
            &heartbeat_payload,
            e_key,
        );
        let heartbeat = read_client(&mut stream, e_key, 2);
        assert_eq!(heartbeat.packet_type, packet::P_CL2LS_REP_LIVE_CHECK);
        assert_eq!(heartbeat.payload, heartbeat_payload);
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHECK_CHAR_NAME_SUCC,
            &CharacterNameCheckSuccess0104 {
                first_name: check.first_name,
                last_name: check.last_name,
            },
            e_key,
        );

        let save_frame = read_client(&mut stream, e_key, 3);
        assert_eq!(save_frame.packet_type, packet::P_CL2LS_REQ_SAVE_CHAR_NAME);
        let save = CharacterNameSaveRequest0104::decode(&save_frame.payload).unwrap();
        assert_eq!(save.slot, 2);
        assert_eq!(save.gender, 1);
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_SAVE_CHAR_NAME_SUCC,
            &CharacterNameSaveSuccess0104 {
                pc_uid: uid,
                slot: save.slot,
                gender: save.gender,
                first_name: save.first_name,
                last_name: save.last_name,
            },
            e_key,
        );

        let create_frame = read_client(&mut stream, e_key, 4);
        assert_eq!(create_frame.packet_type, packet::P_CL2LS_REQ_CHAR_CREATE);
        let create = CharacterCreateRequest0104::decode(&create_frame.payload).unwrap();
        assert_eq!(create.style.pc_uid, uid);
        assert_eq!(create.style.gender, 2);
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_CREATE_SUCC,
            &CharacterCreateSuccess0104 {
                level: 1,
                style: create.style,
                style2: PcStyle2Flags0104 {
                    appearance_flag: 1,
                    tutorial_flag: 0,
                    payzone_flag: 0,
                },
                equipped: create.equipped,
            },
            e_key,
        );

        let delete_frame = read_client(&mut stream, e_key, 5);
        assert_eq!(delete_frame.packet_type, packet::P_CL2LS_REQ_CHAR_DELETE);
        assert_eq!(
            CharacterDeleteRequest0104::decode(&delete_frame.payload)
                .unwrap()
                .pc_uid,
            uid
        );
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_DELETE_SUCC,
            &CharacterDeleteSuccess0104 { slot: 2 },
            e_key,
        );
    });

    let mut session = LoginSession::connect_password(address, "creator", "secret").unwrap();
    assert!(session.characters().is_empty());

    let check = CharacterNameCheckRequest0104::new("Proto", "Hero", 0, 0, 0).unwrap();
    let checked = session.check_character_name(&check).unwrap();
    // The 0104 client always reserves a name as male, then permits the
    // appearance screen to switch to female before final creation.
    let save = CharacterNameSaveRequest0104::from_check(2, 1, &check, &checked);
    let saved = session.save_character_name(&save).unwrap();
    assert_eq!(saved.pc_uid, uid);

    let create = CharacterCreateRequest0104 {
        style: PcStyle0104 {
            pc_uid: uid,
            name_check: 0,
            first_name: saved.first_name,
            last_name: saved.last_name,
            gender: 2,
            face_style: 6,
            hair_style: 25,
            hair_color: 1,
            skin_color: 1,
            eye_color: 1,
            height: 0,
            body: 0,
            class: 0,
        },
        equipped: OnItem0104 {
            upper_body_id: 100,
            lower_body_id: 200,
            foot_id: 300,
            ..Default::default()
        },
        selected_indices: OnItemIndex0104 {
            upper_body_index: 1,
            lower_body_index: 2,
            foot_index: 3,
            face_style_index: 0,
            hair_style_index: 0,
        },
    };
    let created = session.create_character(&create).unwrap();
    assert_eq!(created.style.pc_uid, uid);
    assert_eq!(session.characters().len(), 1);
    assert_eq!(session.characters()[0].slot(), 2);
    assert_eq!(session.characters()[0].pc_uid(), uid);
    assert_eq!(session.characters()[0].style().appearance_flag, 1);
    assert_eq!(
        session.characters()[0]
            .equipped_item(ffone_protocol::CharacterEquipSlot0104::UpperBody)
            .item_id,
        100
    );

    assert_eq!(
        session.delete_character(uid).unwrap(),
        CharacterDeleteSuccess0104 { slot: 2 }
    );
    assert!(session.characters().is_empty());
    server.join().unwrap();
}

#[test]
fn fake_openfusion_duplicate_exit_is_exact_and_fire_and_forget() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(frame.packet_type, packet::P_CL2LS_REQ_PC_EXIT_DUPLICATE);
        let request = DuplicateExitRequest0104::decode(&frame.payload).unwrap();
        assert_eq!(request.id.to_string_lossy(), "duplicate-user");
        assert_eq!(request.password.to_string_lossy(), "duplicate-secret");
    });

    LoginSession::request_duplicate_exit(address, "duplicate-user", "duplicate-secret")
        .unwrap();
    server.join().unwrap();
}

#[test]
fn fake_openfusion_character_rename_updates_the_retained_roster() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let login_frame = read_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(login_frame.packet_type, packet::P_CL2LS_REQ_LOGIN);
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_LOGIN_SUCC,
            &LoginSuccess {
                character_count: 1,
                selected_slot: 2,
                payment_flag: 1,
                packing_byte: 0,
                server_time: TEST_SERVER_TIME,
                id: FixedUtf16::from_str("rename-user").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );
        let e_key = derive_login_e_key(TEST_SERVER_TIME, 1, 2);
        let mut character = CharacterInfo0104::zeroed();
        character.as_bytes_mut()[0] = 2;
        character.as_bytes_mut()[4..12].copy_from_slice(&TEST_UID.to_le_bytes());
        character.as_bytes_mut()[66] = 1;
        put_utf16(
            character.as_bytes_mut(),
            14,
            &FixedUtf16::<9>::from_str("Old").unwrap(),
        );
        put_utf16(
            character.as_bytes_mut(),
            32,
            &FixedUtf16::<17>::from_str("Name").unwrap(),
        );
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_INFO,
            &character,
            e_key,
        );

        let frame = read_client(&mut stream, e_key, 1);
        assert_eq!(frame.packet_type, packet::P_CL2LS_REQ_CHANGE_CHAR_NAME);
        let request = CharacterNameChangeRequest0104::decode(&frame.payload).unwrap();
        assert_eq!(request.pc_uid, TEST_UID);
        assert_eq!(request.slot, 2);
        assert_eq!(request.gender, 2);
        assert_eq!(request.first_name.to_string_lossy(), "New");
        assert_eq!(request.last_name.to_string_lossy(), "Identity");
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHANGE_CHAR_NAME_SUCC,
            &CharacterNameChangeSuccess0104 {
                pc_uid: request.pc_uid,
                slot: request.slot,
                first_name: request.first_name,
                last_name: request.last_name,
            },
            e_key,
        );
    });

    let mut session =
        LoginSession::connect_password(address, "rename-user", "rename-secret").unwrap();
    let request = CharacterNameChangeRequest0104 {
        pc_uid: TEST_UID,
        slot: 2,
        gender: 2,
        first_name_code: 0,
        last_name_code: 0,
        middle_name_code: 0,
        first_name: FixedUtf16::from_str("New").unwrap(),
        last_name: FixedUtf16::from_str("Identity").unwrap(),
    };
    let reply = session.change_character_name(&request).unwrap();
    assert_eq!(reply.first_name.to_string_lossy(), "New");
    assert_eq!(reply.last_name.to_string_lossy(), "Identity");
    assert_eq!(
        session.characters()[0].first_name().to_string_lossy(),
        "New"
    );
    assert_eq!(
        session.characters()[0].last_name().to_string_lossy(),
        "Identity"
    );
    assert_eq!(session.characters()[0].style().gender, 2);
    server.join().unwrap();
}

pub(super) fn serve_clean_login_roster(stream: &mut TcpStream, account: &str) -> u64 {
    let login_frame = read_client(stream, DEFAULT_KEY, 0);
    assert_eq!(login_frame.packet_type, packet::P_CL2LS_REQ_LOGIN);
    send_server(
        stream,
        packet::P_LS2CL_REP_LOGIN_SUCC,
        &LoginSuccess {
            character_count: 1,
            selected_slot: 0,
            payment_flag: 1,
            packing_byte: 0,
            server_time: TEST_SERVER_TIME,
            id: FixedUtf16::from_str(account).unwrap(),
            open_beta_flag: 0,
        },
        DEFAULT_KEY,
    );
    let login_e_key = derive_login_e_key(TEST_SERVER_TIME, 1, 0);
    let mut character = CharacterInfo0104::zeroed();
    character.as_bytes_mut()[4..12].copy_from_slice(&TEST_UID.to_le_bytes());
    send_server(
        stream,
        packet::P_LS2CL_REP_CHAR_INFO,
        &character,
        login_e_key,
    );
    login_e_key
}

#[test]
fn clean_login_server_char_select_succ_is_answered_with_shard_select() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let login_e_key = serve_clean_login_roster(&mut stream, "clean-user");

        let select = read_client(&mut stream, login_e_key, 1);
        assert_eq!(select.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
        // The original login server acknowledges the character first and
        // only then waits for the shard choice.
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_SELECT_SUCC,
            &ffone_protocol::wire_0104::LsCharSelectSuccess0104,
            login_e_key,
        );

        let shard_select = read_client(&mut stream, login_e_key, 2);
        assert_eq!(shard_select.packet_type, packet::P_CL2LS_REQ_SHARD_SELECT);
        assert_eq!(
            shard_select.payload,
            vec![CLEAN_WEB_PLAYER_SHARD_NUM_0104 as u8]
        );

        let mut server_ip = [0u8; 16];
        server_ip[..9].copy_from_slice(b"127.0.0.1");
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_SHARD_SELECT_SUCC,
            &ShardSelectSuccess {
                server_ip,
                server_port: 23_001,
                enter_serial_key: TEST_SERIAL,
            },
            login_e_key,
        );
    });

    let mut login = LoginSession::connect_password(address, "clean-user", "secret").unwrap();
    let ticket = login.select_character(TEST_UID).unwrap();
    assert_eq!(ticket.pc_uid(), TEST_UID);
    assert_eq!(ticket.endpoint().port(), 23_001);
    server.join().unwrap();
}

#[test]
fn clean_login_server_char_select_fail_is_a_typed_rejection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let login_e_key = serve_clean_login_roster(&mut stream, "clean-user");
        let select = read_client(&mut stream, login_e_key, 1);
        assert_eq!(select.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_SELECT_FAIL,
            &LsCharSelectFailure0104 { error_code: 7 },
            login_e_key,
        );
    });

    let mut login = LoginSession::connect_password(address, "clean-user", "secret").unwrap();
    let error = login.select_character(TEST_UID).unwrap_err();
    assert!(
        matches!(error, NetError::CharacterSelectRejected { error_code: 7 }),
        "{error}"
    );
    server.join().unwrap();
}

#[test]
fn fake_openfusion_login_to_loading_complete() {
    let shard_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let shard_address = shard_listener.local_addr().unwrap();
    let login_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let login_address = login_listener.local_addr().unwrap();
    let fe_key = derive_frontend_key(44);

    let shard_server = thread::spawn(move || {
        let (mut stream, _) = shard_listener.accept().unwrap();
        let enter_frame = read_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(enter_frame.packet_type, packet::P_CL2FE_REQ_PC_ENTER);
        let enter = PcEnterRequest::decode(&enter_frame.payload).unwrap();
        assert_eq!(enter.id.to_string_lossy(), "test-user");
        assert_eq!(enter.enter_serial_key, TEST_SERIAL);

        let mut load = PcLoadData0104::zeroed();
        load.as_bytes_mut()
            [PcLoadData0104::FUSION_MATTER_OFFSET..PcLoadData0104::FUSION_MATTER_OFFSET + 4]
            .copy_from_slice(&TEST_FUSION_MATTER.to_le_bytes());
        load.as_bytes_mut()
            [PcLoadData0104::MAP_NUMBER_OFFSET..PcLoadData0104::MAP_NUMBER_OFFSET + 4]
            .copy_from_slice(&5i32.to_le_bytes());
        let enter_success = PcEnterSuccess {
            id: TEST_PLAYER_ID,
            load,
            server_time: TEST_SERVER_TIME,
        };
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_NANO_BOOK_SUBSET,
            &PRE_ENTER_NANO,
            fe_key,
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_PC_ENTER_SUCC,
            &enter_success,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_NANO_BOOK_SUBSET,
            &POST_ENTER_NANO,
            fe_key,
        );

        let shard_e_key =
            derive_shard_e_key(TEST_SERVER_TIME, TEST_PLAYER_ID, TEST_FUSION_MATTER);
        let loading_frame = read_client(&mut stream, shard_e_key, 1);
        assert_eq!(
            loading_frame.packet_type,
            packet::P_CL2FE_REQ_PC_LOADING_COMPLETE
        );
        assert_eq!(
            PcLoadingCompleteRequest::decode(&loading_frame.payload)
                .unwrap()
                .pc_id,
            0
        );

        // OpenFusion chunk registration sends all four initial AROUND bucket kinds before
        // acknowledging loading complete. Zero-count buckets still have their native struct
        // header sizes (except SHINY, whose header is only the count).
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_PC_AROUND,
            &vec![0; PcAppearance0104::SIZE],
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_NPC_AROUND,
            &vec![0; NpcAppearance0104::SIZE],
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_TRANSPORTATION_AROUND,
            &vec![0; TransportationAppearance0104::SIZE],
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_SHINY_AROUND,
            &vec![0; 4],
            fe_key,
        );

        // Verify inbound FE / outbound E asymmetry and automatic heartbeat response.
        let shard_heartbeat_payload = 0x5566_7788i32.to_le_bytes();
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REQ_LIVE_CHECK,
            &shard_heartbeat_payload,
            fe_key,
        );
        let heartbeat = read_client(&mut stream, shard_e_key, 2);
        assert_eq!(heartbeat.packet_type, packet::P_CL2FE_REP_LIVE_CHECK);
        assert_eq!(heartbeat.payload, shard_heartbeat_payload);
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_PC_LOADING_COMPLETE_SUCC,
            &PcLoadingCompleteSuccess { pc_id: 0 },
            fe_key,
        );

        let move_frame = read_client(&mut stream, shard_e_key, 3);
        assert_eq!(move_frame.packet_type, packet::P_CL2FE_REQ_PC_MOVE);
        assert_eq!(
            PcMoveRequest0104::decode(&move_frame.payload)
                .unwrap()
                .position,
            [100, 200, 300]
        );
        let stop_frame = read_client(&mut stream, shard_e_key, 4);
        assert_eq!(stop_frame.packet_type, packet::P_CL2FE_REQ_PC_STOP);
        assert_eq!(
            PcStopRequest0104::decode(&stop_frame.payload)
                .unwrap()
                .position,
            [110, 210, 310]
        );
        let jump_frame = read_client(&mut stream, shard_e_key, 5);
        assert_eq!(jump_frame.packet_type, packet::P_CL2FE_REQ_PC_JUMP);
        assert_eq!(
            PcJumpRequest0104::decode(&jump_frame.payload)
                .unwrap()
                .velocity,
            [400, 500, 600]
        );
        let gameplay_heartbeat_payload = 0x6677_8899u32.to_le_bytes();
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REQ_LIVE_CHECK,
            &gameplay_heartbeat_payload,
            fe_key,
        );
        let gameplay_heartbeat = read_client(&mut stream, shard_e_key, 6);
        assert_eq!(
            gameplay_heartbeat.packet_type,
            packet::P_CL2FE_REP_LIVE_CHECK
        );
        assert_eq!(gameplay_heartbeat.payload, gameplay_heartbeat_payload);
        send_server_bytes(&mut stream, 0x3100_0abc, &[9, 8, 7], fe_key);
    });

    let login_server = thread::spawn(move || {
        let (mut stream, _) = login_listener.accept().unwrap();
        let login_frame = read_client(&mut stream, DEFAULT_KEY, 0);
        assert_eq!(login_frame.packet_type, packet::P_CL2LS_REQ_LOGIN);
        let request = LoginRequest::decode(&login_frame.payload).unwrap();
        assert_eq!(request.id.to_string_lossy(), "test-user");
        assert_eq!(request.password.to_string_lossy(), "test-password");
        assert_eq!(
            (
                request.client_version_a,
                request.client_version_b,
                request.client_version_c
            ),
            (1, 0, 44)
        );

        let login_heartbeat_payload = 0x1122_3344i32.to_le_bytes();
        send_server_bytes(
            &mut stream,
            packet::P_LS2CL_REQ_LIVE_CHECK,
            &login_heartbeat_payload,
            DEFAULT_KEY,
        );
        let heartbeat = read_client(&mut stream, DEFAULT_KEY, 1);
        assert_eq!(heartbeat.packet_type, packet::P_CL2LS_REP_LIVE_CHECK);
        assert_eq!(heartbeat.payload, login_heartbeat_payload);

        let login_success = LoginSuccess {
            character_count: 1,
            selected_slot: 0,
            payment_flag: 1,
            packing_byte: 0,
            server_time: TEST_SERVER_TIME,
            id: FixedUtf16::from_str("test-user").unwrap(),
            open_beta_flag: 0,
        };
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_LOGIN_SUCC,
            &login_success,
            DEFAULT_KEY,
        );
        let login_e_key = derive_login_e_key(TEST_SERVER_TIME, 1, 0);
        let mut character = CharacterInfo0104::zeroed();
        character.as_bytes_mut()[0] = 0;
        character.as_bytes_mut()[2..4].copy_from_slice(&36i16.to_le_bytes());
        character.as_bytes_mut()[4..12].copy_from_slice(&TEST_UID.to_le_bytes());
        put_utf16(
            character.as_bytes_mut(),
            14,
            &FixedUtf16::<9>::from_str("Test").unwrap(),
        );
        put_utf16(
            character.as_bytes_mut(),
            32,
            &FixedUtf16::<17>::from_str("Player").unwrap(),
        );
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_INFO,
            &character,
            login_e_key,
        );

        let select_frame = read_client(&mut stream, login_e_key, 2);
        assert_eq!(select_frame.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
        assert_eq!(
            CharacterSelectRequest::decode(&select_frame.payload)
                .unwrap()
                .pc_uid,
            TEST_UID
        );
        let ip_text = shard_address.ip().to_string();
        let mut server_ip = [0u8; 16];
        server_ip[..ip_text.len()].copy_from_slice(ip_text.as_bytes());
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_SHARD_SELECT_SUCC,
            &ShardSelectSuccess {
                server_ip,
                server_port: i32::from(shard_address.port()),
                enter_serial_key: TEST_SERIAL,
            },
            login_e_key,
        );
    });

    let mut login =
        LoginSession::connect_password(login_address, "test-user", "test-password").unwrap();
    assert_eq!(login.characters().len(), 1);
    assert_eq!(login.first_character_uid().unwrap(), TEST_UID);
    let ticket = login.select_character(TEST_UID).unwrap();
    assert_eq!(ticket.endpoint(), shard_address);
    drop(login);

    let mut shard = ShardSession::connect(ticket).unwrap();
    assert_eq!(shard.player_id(), TEST_PLAYER_ID);
    assert_eq!(shard.server_time(), TEST_SERVER_TIME);
    assert_eq!(shard.load_data().fusion_matter(), TEST_FUSION_MATTER);
    assert_eq!(shard.load_data().map_number(), 5);
    assert_eq!(shard.entry_prelude().len(), 1);
    assert_eq!(
        shard.entry_prelude()[0].packet_type,
        packet::P_FE2CL_REP_NANO_BOOK_SUBSET
    );
    assert_eq!(shard.entry_prelude()[0].payload, PRE_ENTER_NANO);
    let loaded = shard.complete_loading().unwrap();
    assert_eq!(loaded.response.pc_id, 0);
    assert_eq!(loaded.prelude.len(), 6);
    assert_eq!(loaded.prelude[0].payload, PRE_ENTER_NANO);
    assert_eq!(loaded.prelude[1].payload, POST_ENTER_NANO);

    let bootstrap = loaded.into_world_bootstrap();
    assert_eq!(bootstrap.response.pc_id, 0);
    assert_eq!(bootstrap.packets.len(), 6);
    assert!(!bootstrap.has_decode_errors());
    assert!(matches!(
        &bootstrap.packets[0],
        WorldBootstrapPacket::Passthrough(frame)
            if frame.payload == PRE_ENTER_NANO
    ));
    assert!(matches!(
        &bootstrap.packets[1],
        WorldBootstrapPacket::Passthrough(frame)
            if frame.payload == POST_ENTER_NANO
    ));
    assert!(matches!(
        &bootstrap.packets[2],
        WorldBootstrapPacket::InitialEntities {
            entities: InitialAroundPacket0104::Players(entries),
            ..
        } if entries.is_empty()
    ));
    assert!(matches!(
        &bootstrap.packets[3],
        WorldBootstrapPacket::InitialEntities {
            entities: InitialAroundPacket0104::Npcs(entries),
            ..
        } if entries.is_empty()
    ));
    assert!(matches!(
        &bootstrap.packets[4],
        WorldBootstrapPacket::InitialEntities {
            entities: InitialAroundPacket0104::Transportation(entries),
            ..
        } if entries.is_empty()
    ));
    assert!(matches!(
        &bootstrap.packets[5],
        WorldBootstrapPacket::InitialEntities {
            entities: InitialAroundPacket0104::Shinies(entries),
            ..
        } if entries.is_empty()
    ));

    let gameplay = shard.into_gameplay().unwrap();
    assert_eq!(gameplay.player_id(), TEST_PLAYER_ID);
    assert_eq!(gameplay.load_data().map_number(), 5);
    let (sender, mut receiver) = gameplay.into_parts();
    sender
        .send_move(&PcMoveRequest0104 {
            client_time: 0,
            position: [100, 200, 300],
            velocity: [1.0, 2.0, 3.0],
            angle: 90,
            key_value: 1,
            speed: 500,
        })
        .unwrap();
    sender
        .send_stop(&PcStopRequest0104 {
            client_time: 0,
            position: [110, 210, 310],
        })
        .unwrap();
    sender
        .send_jump(&PcJumpRequest0104 {
            client_time: 0,
            position: [110, 210, 310],
            velocity: [400, 500, 600],
            angle: 90,
            key_value: 1,
            speed: 500,
        })
        .unwrap();
    let gameplay_frame = receiver.read_next().unwrap();
    assert_eq!(gameplay_frame.packet_type, 0x3100_0abc);
    assert_eq!(gameplay_frame.payload, [9, 8, 7]);

    login_server.join().unwrap();
    shard_server.join().unwrap();
}
