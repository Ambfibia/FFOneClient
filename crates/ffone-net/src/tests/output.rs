use super::*;

#[test]
fn fake_openfusion_tutorial_save_is_exact_and_expects_no_response() {
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
                selected_slot: 0,
                payment_flag: 1,
                packing_byte: 0,
                server_time: TEST_SERVER_TIME,
                id: FixedUtf16::from_str("tutorial-user").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );

        let login_e_key = derive_login_e_key(TEST_SERVER_TIME, 1, 0);
        let mut character = CharacterInfo0104::zeroed();
        character.as_bytes_mut()[4..12].copy_from_slice(&TEST_UID.to_le_bytes());
        send_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_INFO,
            &character,
            login_e_key,
        );

        let actual_wire = read_wire(&mut stream);
        let expected_payload = [
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00,
        ];
        assert_eq!(
            actual_wire,
            encode_client_frame(
                packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR,
                &expected_payload,
                login_e_key,
                1,
            )
            .unwrap()
        );
        let tutorial_save =
            decode_client_frame(&actual_wire, login_e_key, 1).expect("exact client frame");
        assert_eq!(
            tutorial_save.packet_type,
            packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR
        );
        assert_eq!(
            CharacterTutorialSaveRequest0104::decode(&tutorial_save.payload),
            Ok(CharacterTutorialSaveRequest0104 {
                pc_uid: TEST_UID,
                tutorial_flag: 1,
            })
        );
        // OpenFusion's success path returns immediately without writing a response.

        let select_frame = read_client(&mut stream, login_e_key, 2);
        assert_eq!(select_frame.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
        assert_eq!(
            CharacterSelectRequest::decode(&select_frame.payload),
            Ok(CharacterSelectRequest { pc_uid: TEST_UID })
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

    let mut login = LoginSession::connect_password(address, "tutorial-user", "secret").unwrap();
    login
        .send_tutorial_completion(&CharacterTutorialSaveRequest0104 {
            pc_uid: TEST_UID,
            tutorial_flag: 1,
        })
        .unwrap();
    assert_eq!(login.characters()[0].style().tutorial_flag, 1);
    let ticket = login.select_character(TEST_UID).unwrap();
    assert_eq!(ticket.pc_uid(), TEST_UID);

    server.join().unwrap();
}
