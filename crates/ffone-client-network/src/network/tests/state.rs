use super::*;

pub(super) fn send_tutorial_shard_selection(stream: &mut TcpStream, key: u64, port: i32) {
    let mut server_ip = [0u8; 16];
    server_ip[..9].copy_from_slice(b"127.0.0.1");
    send_tutorial_server(
        stream,
        packet::P_LS2CL_REP_SHARD_SELECT_SUCC,
        &ShardSelectSuccess {
            server_ip,
            server_port: port,
            enter_serial_key: 0x1020_3040_5060_7080,
        },
        key,
    );
}

#[test]
fn inventory_event_access_is_typed_strict_and_lossless() {
    let move_success = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 6,
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
            item_id: 506,
            option: 1,
            time_limit: 900,
        },
    };
    let move_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_ITEM_MOVE_SUCC,
        flags: 0x123,
        checksum: 0x456,
        payload: move_success.encode(),
    };
    let classified = NetworkEvent::Frame(move_frame.clone())
        .inventory_frame_0104()
        .unwrap();
    assert!(matches!(
        &classified,
        InventoryGameplayFrame0104::Decoded {
            frame,
            packet: InventoryPacket0104::ItemMoveSuccess(actual),
        } if frame == &move_frame && actual == &move_success
    ));
    assert_eq!(classified.raw_frame(), &move_frame);

    let malformed_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_EQUIP_CHANGE,
        flags: 0x234,
        checksum: 0x567,
        payload: vec![0x5a; EquipChangePacket0104::SIZE - 1],
    };
    let malformed = NetworkEvent::Frame(malformed_frame.clone())
        .inventory_frame_0104()
        .unwrap();
    assert!(matches!(
        &malformed,
        InventoryGameplayFrame0104::Malformed { frame, .. }
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
        .inventory_frame_0104()
        .unwrap();
    assert!(matches!(
        &passthrough,
        InventoryGameplayFrame0104::Passthrough(frame) if frame == &unknown_frame
    ));
    assert_eq!(passthrough.raw_frame(), &unknown_frame);
    assert!(
        NetworkEvent::LoginFrame(unknown_frame)
            .inventory_frame_0104()
            .is_none()
    );
}
