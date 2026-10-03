use super::*;
use ffone_protocol::{
    CharacterNameCheckSuccess0104, CharacterNameSaveSuccess0104, PcEnterSuccess, PcStyle2Flags0104,
    PcStyle0104, derive_frontend_key, derive_shard_e_key,
};

fn create_on_server(stream: &mut TcpStream, key: u64, sequence: &mut u16, slot: i8, uid: i64) {
    let check = read_tutorial_client(stream, key, *sequence);
    *sequence += 1;
    assert_eq!(check.packet_type, packet::P_CL2LS_REQ_CHECK_CHAR_NAME);
    let check = CharacterNameCheckRequest0104::decode(&check.payload).unwrap();
    send_tutorial_server(
        stream,
        packet::P_LS2CL_REP_CHECK_CHAR_NAME_SUCC,
        &CharacterNameCheckSuccess0104 {
            first_name: check.first_name,
            last_name: check.last_name,
        },
        key,
    );
    let save = read_tutorial_client(stream, key, *sequence);
    *sequence += 1;
    assert_eq!(save.packet_type, packet::P_CL2LS_REQ_SAVE_CHAR_NAME);
    let save = CharacterNameSaveRequest0104::decode(&save.payload).unwrap();
    assert_eq!(save.slot, slot);
    send_tutorial_server(
        stream,
        packet::P_LS2CL_REP_SAVE_CHAR_NAME_SUCC,
        &CharacterNameSaveSuccess0104 {
            pc_uid: uid,
            slot,
            gender: save.gender,
            first_name: save.first_name,
            last_name: save.last_name,
        },
        key,
    );
    let create = read_tutorial_client(stream, key, *sequence);
    *sequence += 1;
    assert_eq!(create.packet_type, packet::P_CL2LS_REQ_CHAR_CREATE);
    let create = CharacterCreateRequest0104::decode(&create.payload).unwrap();
    assert_eq!(create.style.pc_uid, uid);
    send_tutorial_server(
        stream,
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
        key,
    );
}

#[test]
fn character_session_creates_second_character_after_world_return_with_known_location() {
    let login_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = login_listener.local_addr().unwrap();
    let shard_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let shard_port = shard_listener.local_addr().unwrap().port();
    let (done_tx, done_rx) = mpsc::channel();
    let (heartbeat_tx, heartbeat_rx) = mpsc::channel();
    let login_server = thread::spawn(move || {
        let (mut stream, _) = login_listener.accept().unwrap();
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
                id: FixedUtf16::from_str("two-characters").unwrap(),
                open_beta_flag: 0,
            },
            DEFAULT_KEY,
        );
        let key = derive_login_e_key(TUTORIAL_TEST_SERVER_TIME, 0, 1);
        let mut sequence = 1;
        for slot in 1..=2 {
            create_on_server(&mut stream, key, &mut sequence, slot, i64::from(slot));
            let select = read_tutorial_client(&mut stream, key, sequence);
            sequence += 1;
            assert_eq!(select.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
            assert_eq!(
                CharacterSelectRequest::decode(&select.payload)
                    .unwrap()
                    .pc_uid,
                i64::from(slot)
            );
            send_tutorial_shard_selection(&mut stream, key, i32::from(shard_port));
            if slot == 1 {
                let save = read_tutorial_client(&mut stream, key, sequence);
                sequence += 1;
                assert_eq!(save.packet_type, packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR);
                assert_eq!(
                    CharacterTutorialSaveRequest0104::decode(&save.payload)
                        .unwrap()
                        .pc_uid,
                    1
                );
                let select = read_tutorial_client(&mut stream, key, sequence);
                sequence += 1;
                assert_eq!(select.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
                send_tutorial_shard_selection(&mut stream, key, i32::from(shard_port));
                // Idle world heartbeat and second creation share the same sequence.
                let wire =
                    encode_server_frame(packet::P_LS2CL_REQ_LIVE_CHECK, &42i32.to_le_bytes(), key)
                        .unwrap();
                stream.write_all(&wire).unwrap();
                let reply = read_tutorial_client(&mut stream, key, sequence);
                sequence += 1;
                assert_eq!(reply.packet_type, packet::P_CL2LS_REP_LIVE_CHECK);
                assert_eq!(reply.payload, 42i32.to_le_bytes());
                heartbeat_tx.send(()).unwrap();
            }
        }
        done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    });
    let shard_server = thread::spawn(move || {
        for entry in 0..3 {
            let (mut stream, _) = shard_listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            assert_eq!(
                read_tutorial_client(&mut stream, DEFAULT_KEY, 0).packet_type,
                packet::P_CL2FE_REQ_PC_ENTER
            );
            let mut load = PcLoadData0104::zeroed();
            for (offset, coordinate) in [
                PcLoadData0104::X_OFFSET,
                PcLoadData0104::Y_OFFSET,
                PcLoadData0104::Z_OFFSET,
            ]
            .into_iter()
            .zip([632_032i32, 187_177, -5_500])
            {
                load.as_bytes_mut()[offset..offset + 4].copy_from_slice(&coordinate.to_le_bytes());
            }
            let player_id = 81;
            let fe_key = derive_frontend_key(44);
            send_tutorial_server(
                &mut stream,
                packet::P_FE2CL_REP_PC_ENTER_SUCC,
                &PcEnterSuccess {
                    id: player_id,
                    load,
                    server_time: TUTORIAL_TEST_SERVER_TIME,
                },
                fe_key,
            );
            let key = derive_shard_e_key(TUTORIAL_TEST_SERVER_TIME, player_id, 0);
            let mut sequence = 1;
            if entry == 1 {
                assert_eq!(
                    read_tutorial_client(&mut stream, key, sequence).packet_type,
                    packet::P_CL2FE_REQ_PC_MOVE
                );
                sequence += 1;
            }
            assert_eq!(
                read_tutorial_client(&mut stream, key, sequence).packet_type,
                packet::P_CL2FE_REQ_PC_LOADING_COMPLETE
            );
            sequence += 1;
            send_tutorial_server(
                &mut stream,
                packet::P_FE2CL_REP_PC_LOADING_COMPLETE_SUCC,
                &PcLoadingCompleteSuccess { pc_id: 0 },
                fe_key,
            );
            if entry == 0 {
                assert_eq!(
                    read_tutorial_client(&mut stream, key, sequence).packet_type,
                    packet::P_CL2FE_REQ_PC_EXIT
                );
                send_tutorial_server(
                    &mut stream,
                    packet::P_FE2CL_REP_PC_EXIT_SUCC,
                    &PcExitSuccess0104 {
                        pc_id: player_id,
                        exit_code: 1,
                    },
                    fe_key,
                );
            } else {
                assert_eq!(stream.read(&mut [0]).unwrap(), 0);
            }
        }
    });

    let (commands, command_rx) = mpsc::channel();
    let (events, event_rx) = mpsc::sync_channel(64);
    let worker = thread::spawn(move || network_worker(command_rx, events));
    let receive = || event_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    commands
        .send(NetworkCommand::Login {
            login_address: address.to_string(),
            username: "two-characters".into(),
            password: "fixture-only".into(),
        })
        .unwrap();
    assert!(matches!(receive(), NetworkEvent::Connecting));
    assert!(matches!(receive(), NetworkEvent::LoginMetadata { .. }));
    assert!(matches!(receive(), NetworkEvent::Characters(roster) if roster.is_empty()));
    for slot in 1..=2 {
        commands
            .send(NetworkCommand::ReserveCharacterName {
                request: CharacterNameCheckRequest0104::new("Test", "Hero", 0, 0, 0).unwrap(),
                slot,
                gender: 1,
            })
            .unwrap();
        let NetworkEvent::CharacterNameSaved(saved) = receive() else {
            panic!("name reservation failed")
        };
        commands
            .send(NetworkCommand::CreateCharacter(
                CharacterCreateRequest0104 {
                    style: PcStyle0104 {
                        pc_uid: saved.pc_uid,
                        first_name: saved.first_name,
                        last_name: saved.last_name,
                        gender: 1,
                        name_check: 1,
                        face_style: 0,
                        hair_style: 0,
                        hair_color: 0,
                        skin_color: 0,
                        eye_color: 0,
                        height: 0,
                        body: 0,
                        class: 0,
                    },
                    equipped: Default::default(),
                    selected_indices: Default::default(),
                },
            ))
            .unwrap();
        assert!(
            matches!(receive(), NetworkEvent::CharacterCreated { slot: actual, .. } if actual == slot)
        );
        assert!(
            matches!(receive(), NetworkEvent::Characters(roster) if roster.len() == slot as usize)
        );
        commands
            .send(NetworkCommand::SelectCharacter {
                pc_uid: saved.pc_uid,
                location: CharacterEntryLocation0104::Saved,
            })
            .unwrap();
        assert!(matches!(receive(), NetworkEvent::EnteringWorld { .. }));
        assert!(matches!(receive(), NetworkEvent::WorldReady(_)));
        if slot == 1 {
            commands
                .send(NetworkCommand::CompleteTutorial {
                    pc_uid: saved.pc_uid,
                })
                .unwrap();
            assert!(
                matches!(receive(), NetworkEvent::Frame(frame) if frame.packet_type == packet::P_FE2CL_REP_PC_EXIT_SUCC)
            );
            assert!(matches!(receive(), NetworkEvent::EnteringWorld { .. }));
            assert!(
                matches!(receive(), NetworkEvent::WorldReady(world) if world.login_style.tutorial_flag == 1)
            );
            heartbeat_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        commands
            .send(NetworkCommand::ReturnToCharacterSelection)
            .unwrap();
        let NetworkEvent::ReturnedToCharacterSelection(roster) = receive() else {
            panic!("menu return failed")
        };
        assert_eq!(roster.len(), slot as usize);
        assert_eq!(
            roster[0].position,
            TUTORIAL_COMPLETION_SECTOR_V_POSITION_0104
        );
        assert_eq!(roster[0].style.tutorial_flag, 1);
        assert_eq!(roster[slot as usize - 1].slot, slot);
        if slot == 2 {
            assert_eq!(roster[1].position, [632_032, 187_177, -5_500]);
        }
    }
    commands.send(NetworkCommand::RefreshCharacters).unwrap();
    assert!(matches!(receive(), NetworkEvent::Characters(roster) if roster.len() == 2));
    done_tx.send(()).unwrap();
    assert!(matches!(
        receive(),
        NetworkEvent::Disconnected {
            reason: DisconnectReason0104::LoginTransportFailed(_)
        }
    ));
    commands.send(NetworkCommand::RefreshCharacters).unwrap();
    assert!(
        matches!(receive(), NetworkEvent::Error(error) if error.contains("requires a successful login"))
    );
    commands.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
    login_server.join().unwrap();
    shard_server.join().unwrap();
}
