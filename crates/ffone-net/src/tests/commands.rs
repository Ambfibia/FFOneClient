use super::*;

#[test]
fn fake_openfusion_chat_command_response_round_trips_as_server_message_0104() {
    const COMMAND: &str = "/help";
    const RESPONSE: &str = "Commands available to you: help, access, population";

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x8877_6655_4433_2211;
    let fe_key = 0x1122_3344_5566_7788;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(frame.packet_type, packet::P_CL2FE_REQ_PC_FREECHAT);
        let request = FreeChatRequest0104::decode(&frame.payload).unwrap();
        assert_eq!(request.message.to_string_lossy(), COMMAND);

        send_server(
            &mut stream,
            packet::P_FE2CL_PC_MOTD_LOGIN,
            &ffone_protocol::ServerMessage0104 {
                message_type: 1,
                message: FixedUtf16::from_str(RESPONSE).unwrap(),
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
        .send_freechat(&FreeChatRequest0104 {
            message: FixedUtf16::from_str(COMMAND).unwrap(),
            emote_code: 0,
        })
        .unwrap();

    let frame = receiver.read_next().unwrap();
    let response =
        ffone_protocol::decode_server_message_0104(frame.packet_type, &frame.payload)
            .unwrap()
            .expect("the command reply must use P_FE2CL_PC_MOTD_LOGIN");
    assert_eq!(response.message_type, 1);
    assert_eq!(response.message.to_string_lossy(), RESPONSE);

    server.join().unwrap();
}

#[test]
fn fake_server_nano_tune_io_routes_exact_request_success_failure_and_passthrough() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x1020_3040_5060_7080;
    let e_key = 0x8070_6050_4030_2010;
    let request = NanoTuneRequest0104 {
        nano_id: 36,
        tune_id: 144,
        // Free tuning retains the fixed ten-slot tail and sends zeros.
        needed_item_slots: [0; NANO_TUNE_ITEM_SLOT_COUNT_0104],
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
    let failure = NanoTuneFailure0104 {
        pc_id: TEST_PLAYER_ID,
        error_code: 5,
    };
    let malformed_payload = vec![0x5a; NanoTuneSuccess0104::SIZE - 1];
    let unknown_payload = vec![0xde, 0xad, 0xbe, 0xef];
    let server_malformed = malformed_payload.clone();
    let server_unknown = unknown_payload.clone();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let outbound = read_client(&mut stream, e_key, 0);
        assert_eq!(outbound.packet_type, packet::P_CL2FE_REQ_NANO_TUNE);
        assert_eq!(outbound.payload.len(), NanoTuneRequest0104::SIZE);
        assert_eq!(NanoTuneRequest0104::decode(&outbound.payload), Ok(request));

        send_server(
            &mut stream,
            packet::P_FE2CL_REP_NANO_TUNE_SUCC,
            &success,
            fe_key,
        );
        send_server(
            &mut stream,
            packet::P_FE2CL_REP_NANO_TUNE_FAIL,
            &failure,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_REP_NANO_TUNE_SUCC,
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

    sender.send_nano_tune(&request).unwrap();

    let decoded_success = receiver.read_next_nano_tune().unwrap();
    assert!(matches!(
        &decoded_success,
        NanoTuneGameplayFrame0104::Decoded {
            packet: NanoTunePacket0104::Success(actual),
            ..
        } if actual == &success
    ));
    assert_eq!(decoded_success.raw_frame().payload, success.encode());

    let decoded_failure = receiver.read_next_nano_tune().unwrap();
    assert!(matches!(
        &decoded_failure,
        NanoTuneGameplayFrame0104::Decoded {
            packet: NanoTunePacket0104::Failure(actual),
            ..
        } if actual == &failure
    ));
    assert_eq!(decoded_failure.raw_frame().payload, failure.encode());

    let malformed = receiver.read_next_nano_tune().unwrap();
    assert!(matches!(
        &malformed,
        NanoTuneGameplayFrame0104::Malformed {
            error: PayloadError::WrongSize {
                expected: NanoTuneSuccess0104::SIZE,
                actual,
            },
            ..
        } if *actual == NanoTuneSuccess0104::SIZE - 1
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);

    let passthrough = receiver.read_next_nano_tune().unwrap();
    assert!(matches!(
        &passthrough,
        NanoTuneGameplayFrame0104::Passthrough(frame)
            if frame.packet_type == 0x3100_0abc
    ));
    assert_eq!(passthrough.raw_frame().payload, unknown_payload);

    server.join().unwrap();
}
