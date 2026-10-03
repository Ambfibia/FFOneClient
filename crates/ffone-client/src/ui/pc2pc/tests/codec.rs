use super::*;

#[test]
fn packet_ids_and_wire_item_size_match_clean_0104_abi() {
    let identity = identity();
    let confirm = Pc2pcIntent0104::Confirm(Pc2pcConfirmIntent0104 {
        envelope: Pc2pcEnvelope0104::local(identity),
    });
    let cancel = Pc2pcIntent0104::Cancel(Pc2pcCancelIntent0104 {
        envelope: Pc2pcEnvelope0104::local(identity),
    });
    assert_eq!(confirm.packet_id(), 318_767_143);
    assert_eq!(cancel.packet_id(), 318_767_144);
    assert_eq!(Pc2pcTradeItem0104::SIZE, 16);
    assert_eq!(12 + Pc2pcTradeItem0104::SIZE, 28);
    assert_eq!(
        12 + PC2PC_PROTOCOL_TRADE_ITEM_COUNT * Pc2pcTradeItem0104::SIZE
            + 4
            + PC2PC_PROTOCOL_TRADE_ITEM_COUNT * Pc2pcTradeItem0104::SIZE,
        400
    );

    let register = Pc2pcIntent0104::RegisterItem(Pc2pcRegisterItemIntent0104 {
        envelope: Pc2pcEnvelope0104::local(identity),
        item: Pc2pcTradeItem0104 {
            item_type: 7,
            item_id: 91,
            option: 3,
            inventory_slot: 12,
            offer_slot: 2,
        },
    })
    .encode_registered()
    .unwrap();
    assert_eq!(register.packet_type(), 0x1300_002a);
    assert_eq!(register.payload().len(), 28);
    assert_eq!(&register.payload()[12..16], &[7, 0, 91, 0]);
}

pub(super) fn offer_reply_frame(
    packet_type: u32,
    requester_pc_id: i32,
    from_pc_id: i32,
    to_pc_id: i32,
    abort_code: Option<i16>,
) -> DecodedFrame {
    let mut payload = vec![0; if abort_code.is_some() { 16 } else { 12 }];
    payload[0..4].copy_from_slice(&requester_pc_id.to_le_bytes());
    payload[4..8].copy_from_slice(&from_pc_id.to_le_bytes());
    payload[8..12].copy_from_slice(&to_pc_id.to_le_bytes());
    if let Some(error_code) = abort_code {
        payload[12..14].copy_from_slice(&error_code.to_le_bytes());
    }
    DecodedFrame {
        packet_type,
        flags: 0,
        checksum: 0,
        payload,
    }
}
