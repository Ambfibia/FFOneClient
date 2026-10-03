use super::*;

#[test]
fn fake_server_inventory_io_is_typed_strict_and_lossless() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let fe_key = 0x1020_3040_5060_7080;
    let move_success = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 4,
        from_slot_item: ItemBase0104 {
            item_type: 7,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        to_location: 0,
        to_slot_num: 7,
        to_slot_item: ItemBase0104 {
            item_type: 0,
            item_id: 304,
            option: 2,
            time_limit: 900,
        },
    };
    let malformed_payload = vec![0x5a; EquipChangePacket0104::SIZE - 1];
    let unknown_payload = vec![0xde, 0xad, 0xbe, 0xef];
    let server_move = move_success;
    let server_malformed = malformed_payload.clone();
    let server_unknown = unknown_payload.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        send_server(
            &mut stream,
            packet::P_FE2CL_PC_ITEM_MOVE_SUCC,
            &server_move,
            fe_key,
        );
        send_server_bytes(
            &mut stream,
            packet::P_FE2CL_PC_EQUIP_CHANGE,
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

    let decoded = receiver.read_next_inventory().unwrap();
    assert!(matches!(
        &decoded,
        InventoryGameplayFrame0104::Decoded {
            packet: InventoryPacket0104::ItemMoveSuccess(actual),
            ..
        } if actual == &move_success
    ));
    assert_eq!(decoded.raw_frame().payload, move_success.encode());

    let malformed = receiver.read_next_inventory().unwrap();
    assert!(matches!(
        &malformed,
        InventoryGameplayFrame0104::Malformed {
            error: PayloadError::WrongSize {
                expected: EquipChangePacket0104::SIZE,
                actual,
            },
            ..
        } if *actual == EquipChangePacket0104::SIZE - 1
    ));
    assert_eq!(malformed.raw_frame().payload, malformed_payload);

    let passthrough = receiver.read_next_inventory().unwrap();
    assert!(matches!(
        &passthrough,
        InventoryGameplayFrame0104::Passthrough(frame)
            if frame.packet_type == 0x3100_0abc
    ));
    assert_eq!(passthrough.raw_frame().payload, unknown_payload);

    server.join().unwrap();
}
