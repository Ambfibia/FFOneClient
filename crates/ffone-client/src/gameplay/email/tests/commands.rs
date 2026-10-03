use super::*;

#[test]
fn unsolicited_new_email_effect_does_not_unlock_an_unrelated_pending_request() {
    let (mut harness, _) = open_harness(None, EmailItemFeaturePolicy0104::default(), false);
    harness
        .production
        .switch_folder(
            EmailFolder::Player,
            &mut harness.model,
            &mut harness.transport,
            &mut harness.actions,
            &mut harness.audio,
        )
        .unwrap();
    harness
        .production
        .dispatch_next_request(
            &harness.model,
            &mut harness.transport,
            &mut harness.network,
            &TestCatalog,
        )
        .unwrap();
    assert!(harness.model.send_in_flight);

    let EmailFrameDisposition0104::Applied { output, .. } = harness.production.route_frame(
        frame(EMAIL_REP_NEW_ID, 3_i32.to_le_bytes().to_vec()),
        &mut harness.model,
        &mut harness.actions,
        &mut harness.transport,
        &mut harness.audio,
        &mut harness.network,
    ) else {
        panic!("new-email push must apply")
    };
    assert_eq!(output.actions, vec![EmailUiAction::NewEmailCount(3)]);
    assert!(harness.model.send_in_flight);
    assert_eq!(
        harness.production.pending_request(),
        Some(&EmailRequest::PageList { page: 1 })
    );
    assert_eq!(
        harness.network.0.pending(),
        Some(&EmailPending0104::PageList { page: 1 })
    );
}

#[test]
fn send_success_emits_atomic_server_commit_and_consumes_legacy_mutation_action() {
    let item = base_item(0, 1, 2);
    let (mut harness, _) =
        open_harness(Some(item), EmailItemFeaturePolicy0104::default(), false);
    let request = prepare_send(&mut harness, item);
    let outbound = harness
        .production
        .dispatch_next_request(
            &harness.model,
            &mut harness.transport,
            &mut harness.network,
            &TestCatalog,
        )
        .unwrap()
        .request
        .unwrap();
    assert_eq!(outbound.packet_type(), EMAIL_REQ_SEND_ID);

    let reply = send_success_frame(&request, 700);
    let EmailFrameDisposition0104::Applied { output, .. } = harness.production.route_frame(
        reply,
        &mut harness.model,
        &mut harness.actions,
        &mut harness.transport,
        &mut harness.audio,
        &mut harness.network,
    ) else {
        panic!("send success must apply")
    };
    let commit = output.commit.unwrap();
    assert_eq!(commit.owner_pc_id(), 42);
    assert_eq!(commit.source(), EmailAuthoritySource0104::SendSuccess);
    assert_eq!(commit.taros_after(), Some(700));
    assert!(commit.inventory_refresh_required());
    assert_eq!(
        commit.inventory_writes().collect::<Vec<_>>(),
        vec![EmailInventoryWrite0104 {
            inventory_slot: 0,
            item: ItemBase0104 { item_id: 0, ..item },
        }]
    );
    assert!(
        !output
            .actions
            .iter()
            .any(|action| matches!(action, EmailUiAction::ApplySendSuccessItems(_)))
    );
    assert_eq!(harness.model.available_cash, 700);
    assert!(
        harness
            .production
            .session()
            .unwrap()
            .inventory_projection_stale()
    );

    let refreshed = authority_with(None);
    harness
        .production
        .refresh_inventory_projection(
            EmailPlayerAuthority0104 {
                owner_pc_id: 42,
                taros: 700,
                current_local_time: EmailSystemTime::default(),
            },
            &refreshed,
            &TestCatalog,
            &mut harness.model,
        )
        .unwrap();
    assert!(
        !harness
            .production
            .session()
            .unwrap()
            .inventory_projection_stale()
    );
}

#[test]
fn receive_taros_commit_uses_only_authoritative_reply_value() {
    let (mut harness, _) = open_harness(None, EmailItemFeaturePolicy0104::default(), false);
    harness.model.folder = EmailFolder::Player;
    harness.model.player_messages = vec![EmailSummary {
        email_index: 99,
        ..EmailSummary::default()
    }];
    harness.model.selected_row = Some(0);
    harness.model.read_message = Some(EmailReadMessage {
        email_index: 99,
        cash: 123,
        ..EmailReadMessage::default()
    });
    harness
        .transport
        .push(EmailRequest::ReceiveCash { email_index: 99 });
    harness.model.send_in_flight = true;
    harness
        .production
        .dispatch_next_request(
            &harness.model,
            &mut harness.transport,
            &mut harness.network,
            &TestCatalog,
        )
        .unwrap();
    let mut body = vec![0; EMAIL_REP_RECEIVE_CASH_SUCCESS_SIZE];
    body[0..8].copy_from_slice(&99_i64.to_le_bytes());
    body[8..12].copy_from_slice(&1_321_i32.to_le_bytes());
    let EmailFrameDisposition0104::Applied { output, .. } = harness.production.route_frame(
        frame(EMAIL_REP_RECEIVE_CASH_SUCCESS_ID, body),
        &mut harness.model,
        &mut harness.actions,
        &mut harness.transport,
        &mut harness.audio,
        &mut harness.network,
    ) else {
        panic!("receive cash success must apply")
    };
    let commit = output.commit.unwrap();
    assert_eq!(commit.taros_after(), Some(1_321));
    assert_eq!(harness.model.available_cash, 1_321);
    assert_eq!(
        harness.production.session().unwrap().context().player.taros,
        1_321
    );
}

#[test]
fn every_request_encodes_the_exact_clean_id_size_and_little_endian_fields() {
    let requests = [
        EmailRequest::UpdateCheck,
        EmailRequest::Read {
            email_index: 0x0102_0304_0506_0708,
        },
        EmailRequest::PageList { page: -2 },
        EmailRequest::Delete {
            email_indices: [1, 2, 3, 4, 5],
        },
        EmailRequest::Send {
            recipient_pc_uid: 77,
            subject: "Subject".to_owned(),
            content: "Body".to_owned(),
            items: array::from_fn(|index| sample_outgoing(index as i32)),
            cash: 1234,
        },
        EmailRequest::ReceiveItem {
            email_index: 91,
            inventory_slot: 7,
            email_item_slot: 3,
        },
        EmailRequest::ReceiveCash { email_index: 92 },
        EmailRequest::ReceiveAllItems { email_index: 93 },
    ];
    let expected = [
        (EMAIL_REQ_UPDATE_CHECK_ID, EMAIL_REQ_UPDATE_CHECK_SIZE),
        (EMAIL_REQ_READ_ID, EMAIL_REQ_READ_SIZE),
        (EMAIL_REQ_PAGE_LIST_ID, EMAIL_REQ_PAGE_LIST_SIZE),
        (EMAIL_REQ_DELETE_ID, EMAIL_REQ_DELETE_SIZE),
        (EMAIL_REQ_SEND_ID, EMAIL_REQ_SEND_SIZE),
        (EMAIL_REQ_RECEIVE_ITEM_ID, EMAIL_REQ_RECEIVE_ITEM_SIZE),
        (EMAIL_REQ_RECEIVE_CASH_ID, EMAIL_REQ_RECEIVE_CASH_SIZE),
        (EMAIL_REQ_RECEIVE_ALL_ID, EMAIL_REQ_RECEIVE_ALL_SIZE),
    ];

    for (request, (packet_id, size)) in requests.iter().zip(expected) {
        let packet = encode_email_request_0104(request).expect("encode");
        assert_eq!(packet.packet_id, packet_id);
        assert_eq!(packet.body.len(), size);
    }

    assert!(
        encode_email_request_0104(&EmailRequest::UpdateCheck)
            .unwrap()
            .body
            .is_empty()
    );
    assert_eq!(
        &encode_email_request_0104(&requests[1]).unwrap().body,
        &0x0102_0304_0506_0708_i64.to_le_bytes()
    );
    assert_eq!(
        encode_email_request_0104(&requests[2]).unwrap().body,
        vec![0xfe]
    );
}

#[test]
fn interior_nul_is_rejected_before_a_pending_request_is_installed() {
    let request = EmailRequest::Send {
        recipient_pc_uid: 1,
        subject: "bad\0subject".to_owned(),
        content: String::new(),
        items: [EmailOutgoingItem::default(); EMAIL_ATTACHMENT_COUNT],
        cash: 0,
    };
    let mut runtime = EmailTransportRuntime0104::default();
    assert_eq!(
        runtime.begin(&request),
        Err(EmailRuntimeError0104::Encode(
            EmailEncodeError0104::InteriorNul { field: "subject" }
        ))
    );
    assert_eq!(runtime.pending(), None);
}

#[test]
fn every_known_reply_rejects_both_short_and_long_frames() {
    let packets = [
        (EMAIL_REP_NEW_ID, EMAIL_REP_NEW_SIZE),
        (EMAIL_REP_READ_SUCCESS_ID, EMAIL_REP_READ_SUCCESS_SIZE),
        (EMAIL_REP_READ_FAILURE_ID, EMAIL_REP_READ_FAILURE_SIZE),
        (
            EMAIL_REP_PAGE_LIST_SUCCESS_ID,
            EMAIL_REP_PAGE_LIST_SUCCESS_SIZE,
        ),
        (
            EMAIL_REP_PAGE_LIST_FAILURE_ID,
            EMAIL_REP_PAGE_LIST_FAILURE_SIZE,
        ),
        (EMAIL_REP_DELETE_SUCCESS_ID, EMAIL_REP_DELETE_SUCCESS_SIZE),
        (EMAIL_REP_DELETE_FAILURE_ID, EMAIL_REP_DELETE_FAILURE_SIZE),
        (EMAIL_REP_SEND_SUCCESS_ID, EMAIL_REP_SEND_SUCCESS_SIZE),
        (EMAIL_REP_SEND_FAILURE_ID, EMAIL_REP_SEND_FAILURE_SIZE),
        (
            EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID,
            EMAIL_REP_RECEIVE_ITEM_SUCCESS_SIZE,
        ),
        (
            EMAIL_REP_RECEIVE_ITEM_FAILURE_ID,
            EMAIL_REP_RECEIVE_ITEM_FAILURE_SIZE,
        ),
        (
            EMAIL_REP_RECEIVE_CASH_SUCCESS_ID,
            EMAIL_REP_RECEIVE_CASH_SUCCESS_SIZE,
        ),
        (
            EMAIL_REP_RECEIVE_CASH_FAILURE_ID,
            EMAIL_REP_RECEIVE_CASH_FAILURE_SIZE,
        ),
        (
            EMAIL_REP_RECEIVE_ALL_SUCCESS_ID,
            EMAIL_REP_RECEIVE_ALL_SUCCESS_SIZE,
        ),
        (
            EMAIL_REP_RECEIVE_ALL_FAILURE_ID,
            EMAIL_REP_RECEIVE_ALL_FAILURE_SIZE,
        ),
    ];

    for (packet_id, size) in packets {
        for actual in [size.saturating_sub(1), size + 1] {
            assert_eq!(
                decode_email_reply_0104(packet_id, &vec![0; actual]),
                Err(EmailDecodeError0104::UnexpectedBodySize {
                    packet_id,
                    expected: size,
                    actual,
                })
            );
        }
    }
    assert_eq!(
        decode_email_reply_0104(0xdead_beef, &[1, 2, 3]).unwrap(),
        None
    );
}
