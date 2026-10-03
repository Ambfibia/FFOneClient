use super::*;

pub(super) fn frame(packet_type: u32, payload: Vec<u8>) -> DecodedFrame {
    DecodedFrame {
        packet_type,
        flags: 0x1234,
        checksum: 0x4321,
        payload,
    }
}

pub(super) fn page_success_frame(page: i8, email_index: i64) -> DecodedFrame {
    let mut body = vec![0; EMAIL_REP_PAGE_LIST_SUCCESS_SIZE];
    body[0] = page as u8;
    let record = EMAIL_SUMMARY_FIRST_OFFSET;
    body[record..record + 8].copy_from_slice(&email_index.to_le_bytes());
    body[record + 8..record + 16].copy_from_slice(&55_i64.to_le_bytes());
    write_utf16_at(&mut body, record + 16, EMAIL_FIRST_NAME_UNITS, "Blossom");
    write_utf16_at(&mut body, record + 34, EMAIL_LAST_NAME_UNITS, "Utonium");
    write_utf16_at(&mut body, record + 68, EMAIL_SUBJECT_UNITS, "Hello");
    frame(EMAIL_REP_PAGE_LIST_SUCCESS_ID, body)
}

pub(super) fn read_success_frame(email_index: i64, item: Option<EmailWireItem>) -> DecodedFrame {
    let mut body = vec![0; EMAIL_REP_READ_SUCCESS_SIZE];
    body[0..8].copy_from_slice(&email_index.to_le_bytes());
    write_utf16_at(&mut body, 8, EMAIL_CONTENT_UNITS, "Server body");
    if let Some(item) = item {
        let offset = 8 + EMAIL_CONTENT_UNITS * 2;
        body[offset..offset + 2].copy_from_slice(&item.item_type.to_le_bytes());
        body[offset + 2..offset + 4].copy_from_slice(&item.item_id.to_le_bytes());
        body[offset + 4..offset + 8].copy_from_slice(&item.option.to_le_bytes());
        body[offset + 8..offset + 12].copy_from_slice(&item.time_limit.to_le_bytes());
    }
    frame(EMAIL_REP_READ_SUCCESS_ID, body)
}

pub(super) fn send_success_frame(request: &EmailRequest, taros_after: i32) -> DecodedFrame {
    let EmailRequest::Send {
        recipient_pc_uid,
        items,
        ..
    } = request
    else {
        panic!("send request")
    };
    let mut body = vec![0; EMAIL_REP_SEND_SUCCESS_SIZE];
    body[0..8].copy_from_slice(&recipient_pc_uid.to_le_bytes());
    body[8..12].copy_from_slice(&taros_after.to_le_bytes());
    for (index, outgoing) in items.iter().enumerate() {
        let offset = 12 + index * EMAIL_OUTGOING_ITEM_SIZE;
        body[offset..offset + 4].copy_from_slice(&outgoing.inventory_slot.to_le_bytes());
        body[offset + 4..offset + 6].copy_from_slice(&outgoing.item.item_type.to_le_bytes());
        body[offset + 6..offset + 8].copy_from_slice(&outgoing.item.item_id.to_le_bytes());
        body[offset + 8..offset + 12].copy_from_slice(&outgoing.item.option.to_le_bytes());
        body[offset + 12..offset + 16].copy_from_slice(&outgoing.item.time_limit.to_le_bytes());
    }
    frame(EMAIL_REP_SEND_SUCCESS_ID, body)
}

#[test]
fn unknown_frames_and_enqueue_rejections_preserve_every_frame_field() {
    let (mut harness, _) = open_harness(None, EmailItemFeaturePolicy0104::default(), false);
    let foreign = frame(0x3100_0061, vec![9, 8, 7, 6]);
    assert_eq!(
        harness.production.route_frame(
            foreign.clone(),
            &mut harness.model,
            &mut harness.actions,
            &mut harness.transport,
            &mut harness.audio,
            &mut harness.network,
        ),
        EmailFrameDisposition0104::Passthrough(foreign.clone())
    );
    assert_eq!(
        EmailProductionRuntime0104::enqueue_frame(&mut harness.inbox, foreign.clone()),
        Err(foreign)
    );
    assert!(harness.inbox.0.is_empty());
}
