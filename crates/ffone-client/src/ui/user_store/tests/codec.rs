use super::*;

pub(super) fn fixed_reply_payload(packet_id: u32) -> Vec<u8> {
    let size = match packet_id {
        STREETSTALL_REP_READY_SUCCESS => STREETSTALL_READY_SUCCESS_SIZE,
        STREETSTALL_REP_READY_FAIL
        | STREETSTALL_REP_CANCEL_FAIL
        | STREETSTALL_REP_REGISTER_ITEM_FAIL
        | STREETSTALL_REP_UNREGISTER_ITEM_FAIL
        | STREETSTALL_REP_SALE_START_FAIL
        | STREETSTALL_REP_ITEM_LIST_FAIL
        | STREETSTALL_REP_ITEM_BUY_FAIL => STREETSTALL_FAILURE_SIZE,
        STREETSTALL_REP_CANCEL_SUCCESS => STREETSTALL_CANCEL_SUCCESS_SIZE,
        STREETSTALL_REP_REGISTER_ITEM_SUCCESS => STREETSTALL_REGISTER_SUCCESS_SIZE,
        STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS => STREETSTALL_UNREGISTER_SUCCESS_SIZE,
        STREETSTALL_REP_SALE_START_SUCCESS => STREETSTALL_SALE_START_SUCCESS_SIZE,
        STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER | STREETSTALL_REP_ITEM_BUY_SUCCESS_SELLER => {
            STREETSTALL_ITEM_BUY_SUCCESS_SIZE
        }
        _ => panic!("not a fixed-size reply"),
    };
    let mut payload = vec![0_u8; size];
    if packet_id == STREETSTALL_REP_READY_SUCCESS {
        write_i32(&mut payload, 0, 0);
        write_i32(&mut payload, 4, 5);
        payload[8..12].copy_from_slice(&0.05_f32.to_le_bytes());
    }
    payload
}

pub(super) fn list_payload(target: i32, records: &[(i32, ItemBase0104, i32)]) -> Vec<u8> {
    let mut payload = vec![
        0_u8;
        STREETSTALL_ITEM_LIST_HEADER_SIZE
            + records.len() * STREETSTALL_ITEM_LIST_RECORD_SIZE
    ];
    write_i32(&mut payload, 0, target);
    write_i32(&mut payload, 4, records.len() as i32);
    for (index, (slot, item, price)) in records.iter().copied().enumerate() {
        let offset =
            STREETSTALL_ITEM_LIST_HEADER_SIZE + index * STREETSTALL_ITEM_LIST_RECORD_SIZE;
        write_i32(&mut payload, offset, slot);
        encode_item(item, &mut payload[offset + 4..offset + 16]);
        write_i32(&mut payload, offset + 16, price);
    }
    payload
}

#[test]
fn list_reply_is_strict_and_retains_packet_holes() {
    let records = [(0, item(0, 10, 1), 50), (3, item(7, 20, 4), -1)];
    let payload = list_payload(TARGET, &records);
    let decoded = decode_user_store_reply_0104(STREETSTALL_REP_ITEM_LIST, &payload)
        .expect("valid sparse list");
    assert_eq!(
        decoded,
        UserStoreReply0104::ItemListSuccess(UserStoreItemListSuccess0104 {
            target_pc_id: TARGET,
            records: records
                .into_iter()
                .map(|(list_slot, item, price)| UserStoreListingRecord0104 {
                    list_slot,
                    item,
                    price,
                })
                .collect(),
        })
    );
    assert!(decode_user_store_reply_0104(STREETSTALL_REP_ITEM_LIST, &payload[..7]).is_err());
    let mut long = payload.clone();
    long.push(0);
    assert!(decode_user_store_reply_0104(STREETSTALL_REP_ITEM_LIST, &long).is_err());
    let mut negative = vec![0_u8; 8];
    write_i32(&mut negative, 4, -1);
    assert_eq!(
        decode_user_store_reply_0104(STREETSTALL_REP_ITEM_LIST, &negative),
        Err(UserStoreCodecError0104::NegativeListCount(-1))
    );
    let mut over = vec![0_u8; 8];
    write_i32(&mut over, 4, 6);
    assert_eq!(
        decode_user_store_reply_0104(STREETSTALL_REP_ITEM_LIST, &over),
        Err(UserStoreCodecError0104::ListCountOverMaximum(6))
    );
    let duplicate = list_payload(TARGET, &[(1, records[0].1, 1), (1, records[1].1, 2)]);
    assert_eq!(
        decode_user_store_reply_0104(STREETSTALL_REP_ITEM_LIST, &duplicate),
        Err(UserStoreCodecError0104::DuplicateListSlot(1))
    );
    let invalid = list_payload(TARGET, &[(5, records[0].1, 1)]);
    assert_eq!(
        decode_user_store_reply_0104(STREETSTALL_REP_ITEM_LIST, &invalid),
        Err(UserStoreCodecError0104::InvalidListSlot(5))
    );
}
