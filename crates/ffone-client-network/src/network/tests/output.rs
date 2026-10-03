use super::*;

#[test]
fn complete_tutorial_writes_save_then_select_without_waiting_for_save_reply() {
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
                id: FixedUtf16::from_str("tutorial-worker").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );
        let login_e_key = derive_login_e_key(TUTORIAL_TEST_SERVER_TIME, 1, 0);
        let mut character = CharacterInfo0104::zeroed();
        character.as_bytes_mut()[4..12].copy_from_slice(&TUTORIAL_TEST_UID.to_le_bytes());
        send_tutorial_server(
            &mut stream,
            packet::P_LS2CL_REP_CHAR_INFO,
            &character,
            login_e_key,
        );

        let save = read_tutorial_client(&mut stream, login_e_key, 1);
        assert_eq!(save.packet_type, packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR);
        assert_eq!(
            CharacterTutorialSaveRequest0104::decode(&save.payload),
            Ok(CharacterTutorialSaveRequest0104 {
                pc_uid: TUTORIAL_TEST_UID,
                tutorial_flag: 1,
            })
        );

        // OpenFusion deliberately sends no SAVE_CHAR_TUTOR success response.
        // The next client frame must already be CHAR_SELECT.
        let select = read_tutorial_client(&mut stream, login_e_key, 2);
        assert_eq!(select.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
        assert_eq!(
            CharacterSelectRequest::decode(&select.payload),
            Ok(CharacterSelectRequest {
                pc_uid: TUTORIAL_TEST_UID,
            })
        );
    });

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));
    command_tx
        .send(NetworkCommand::Login {
            login_address: address.to_string(),
            username: "tutorial-worker".to_owned(),
            password: "secret".to_owned(),
        })
        .unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Connecting)
    ));
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::LoginMetadata { payment_flag: 1 })
    ));
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Characters(characters))
            if characters.len() == 1 && characters[0].pc_uid == TUTORIAL_TEST_UID
    ));

    command_tx
        .send(NetworkCommand::CompleteTutorial {
            pc_uid: TUTORIAL_TEST_UID,
        })
        .unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::EnteringWorld { pc_uid }) if pc_uid == TUTORIAL_TEST_UID
    ));
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(2)),
        Ok(NetworkEvent::TutorialExitFailed { pc_uid, error })
            if pc_uid == TUTORIAL_TEST_UID && error.contains("network I/O failed")
    ));

    server.join().unwrap();
    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}
