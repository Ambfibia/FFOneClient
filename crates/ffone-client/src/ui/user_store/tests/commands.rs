use super::*;

#[test]
fn every_request_has_exact_beta0104_id_size_and_bytes() {
    let sample = item(7, 0x1234, 0x1020_3040);
    let packets = [
        UserStorePacket0104::ready(-3),
        UserStorePacket0104::cancel(44),
        UserStorePacket0104::register_item(2, 9, sample, 500_001),
        UserStorePacket0104::unregister_item(4),
        UserStorePacket0104::sale_start(8),
        UserStorePacket0104::item_list(77),
        UserStorePacket0104::item_buy(77, 3, 49),
    ];
    assert_eq!(packets[0].packet_id, STREETSTALL_REQ_READY);
    assert_eq!(packets[0].payload, (-3_i32).to_le_bytes());
    assert_eq!(packets[1].packet_id, STREETSTALL_REQ_CANCEL);
    assert_eq!(packets[1].payload, 44_i32.to_le_bytes());
    assert_eq!(packets[2].packet_id, STREETSTALL_REQ_REGISTER_ITEM);
    assert_eq!(packets[2].payload.len(), STREETSTALL_REGISTER_REQUEST_SIZE);
    assert_eq!(read_i32(&packets[2].payload, 0), 2);
    assert_eq!(read_i32(&packets[2].payload, 4), 9);
    assert_eq!(decode_item(&packets[2].payload[8..20]), sample);
    assert_eq!(read_i32(&packets[2].payload, 20), 500_001);
    assert_eq!(packets[3].packet_id, STREETSTALL_REQ_UNREGISTER_ITEM);
    assert_eq!(packets[3].payload, 4_i32.to_le_bytes());
    assert_eq!(packets[4].packet_id, STREETSTALL_REQ_SALE_START);
    assert_eq!(packets[4].payload, 8_i32.to_le_bytes());
    assert_eq!(packets[5].packet_id, STREETSTALL_REQ_ITEM_LIST);
    assert_eq!(packets[5].payload, 77_i32.to_le_bytes());
    assert_eq!(packets[6].packet_id, STREETSTALL_REQ_ITEM_BUY);
    assert_eq!(packets[6].payload.len(), STREETSTALL_ITEM_BUY_REQUEST_SIZE);
    assert_eq!(read_i32(&packets[6].payload, 0), 77);
    assert_eq!(read_i32(&packets[6].payload, 4), 3);
    assert_eq!(read_i32(&packets[6].payload, 8), 49);
}

#[test]
fn every_fixed_reply_requires_exact_size() {
    let packet_ids = [
        STREETSTALL_REP_READY_SUCCESS,
        STREETSTALL_REP_READY_FAIL,
        STREETSTALL_REP_CANCEL_SUCCESS,
        STREETSTALL_REP_CANCEL_FAIL,
        STREETSTALL_REP_REGISTER_ITEM_SUCCESS,
        STREETSTALL_REP_REGISTER_ITEM_FAIL,
        STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS,
        STREETSTALL_REP_UNREGISTER_ITEM_FAIL,
        STREETSTALL_REP_SALE_START_SUCCESS,
        STREETSTALL_REP_SALE_START_FAIL,
        STREETSTALL_REP_ITEM_LIST_FAIL,
        STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER,
        STREETSTALL_REP_ITEM_BUY_SUCCESS_SELLER,
        STREETSTALL_REP_ITEM_BUY_FAIL,
    ];
    for packet_id in packet_ids {
        let exact = fixed_reply_payload(packet_id);
        assert!(decode_user_store_reply_0104(packet_id, &exact).is_ok());
        assert!(decode_user_store_reply_0104(packet_id, &exact[..exact.len() - 1]).is_err());
        let mut long = exact;
        long.push(0);
        assert!(decode_user_store_reply_0104(packet_id, &long).is_err());
    }
}
