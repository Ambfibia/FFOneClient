use super::*;

#[test]
fn every_client_request_has_the_clean_packet_id_and_abi_size() {
    let requests = [
        (EmailRequest::UpdateCheck, 0x1300_007B, 0),
        (EmailRequest::Read { email_index: 1 }, 0x1300_007C, 8),
        (EmailRequest::PageList { page: 1 }, 0x1300_007D, 1),
        (
            EmailRequest::Delete {
                email_indices: [0; 5],
            },
            0x1300_007E,
            40,
        ),
        (
            EmailRequest::Send {
                recipient_pc_uid: 1,
                subject: String::new(),
                content: String::new(),
                items: [EmailOutgoingItem::default(); 4],
                cash: 0,
            },
            0x1300_007F,
            1_164,
        ),
        (
            EmailRequest::ReceiveItem {
                email_index: 1,
                inventory_slot: 2,
                email_item_slot: 3,
            },
            0x1300_0080,
            16,
        ),
        (EmailRequest::ReceiveCash { email_index: 1 }, 0x1300_0081, 8),
        (
            EmailRequest::ReceiveAllItems { email_index: 1 },
            0x1300_0089,
            8,
        ),
    ];
    for (request, id, size) in requests {
        assert_eq!(request.packet_id(), id);
        assert_eq!(request.body_size(), size);
    }
}

#[test]
fn reply_packet_ids_and_sizes_cover_the_full_email_family() {
    let reply = EmailReply::ReadSuccess(EmailReadMessage::default());
    assert_eq!(reply.packet_id(), EMAIL_REP_READ_SUCCESS_ID);
    assert_eq!(reply.body_size(), 1_084);
    let reply = EmailReply::PageListSuccess {
        page: 1,
        messages: vec![],
    };
    assert_eq!(reply.packet_id(), EMAIL_REP_PAGE_LIST_SUCCESS_ID);
    assert_eq!(reply.body_size(), 1_024);
    let reply = EmailReply::SendSuccess {
        recipient_pc_uid: 1,
        authoritative_cash: 0,
        items: [EmailOutgoingItem::default(); 4],
    };
    assert_eq!(reply.packet_id(), EMAIL_REP_SEND_SUCCESS_ID);
    assert_eq!(reply.body_size(), 76);
    let reply = EmailReply::ReceiveAllItemsFailure {
        email_index: 1,
        error_code: 5,
    };
    assert_eq!(reply.packet_id(), EMAIL_REP_RECEIVE_ALL_FAILURE_ID);
    assert_eq!(reply.body_size(), 12);
}

#[test]
fn compose_enforces_50_plus_20_per_item_and_fixed_four_item_wire_array() {
    let mut draft = EmailComposeDraft {
        recipient_pc_uid: 77,
        recipient_name: "Dexter".to_owned(),
        cash: 100,
        ..default()
    };
    draft.subject = "".to_owned();
    draft.content = "Meet me at Tech Square.".to_owned();
    draft.attachments[0] = Some(EmailOutgoingItem {
        inventory_slot: 9,
        item: EmailWireItem {
            item_type: 7,
            item_id: 123,
            option: 4,
            time_limit: 0,
        },
    });
    assert_eq!(draft.postage(), 70);
    let request = draft.request(170).unwrap();
    let EmailRequest::Send {
        subject,
        items,
        cash,
        ..
    } = request
    else {
        panic!("expected send request")
    };
    assert_eq!(subject, "No subject.");
    assert_eq!(items[0].inventory_slot, 9);
    assert_eq!(items[1], EmailOutgoingItem::default());
    assert_eq!(cash, 100);
}

#[test]
fn compose_text_limits_match_the_clean_controls_not_the_larger_subject_wire_field() {
    let mut draft = EmailComposeDraft::default();
    draft.set_subject("abcdefghijklmnopqrstuvwxyz");
    draft.set_content("x".repeat(600));
    assert_eq!(utf16_units(&draft.subject), 24);
    assert_eq!(utf16_units(&draft.content), 512);
    assert_eq!(EMAIL_SUBJECT_WIRE_UNITS, 32);
    assert_eq!(EMAIL_CONTENT_WIRE_UNITS, 512);
    draft.recipient_pc_uid = 1;
    assert!(draft.request(EMAIL_BASE_POSTAGE).is_ok());

    draft.set_subject("😀".repeat(20));
    assert_eq!(utf16_units(&draft.subject), 24);
}

#[test]
fn input_boundary_disables_every_email_control_during_packet_or_global_modal() {
    let (mut model, _, _, _) = opened_player_model();
    assert!(model.input_enabled());
    model.send_in_flight = true;
    assert!(!model.input_enabled());
    model.send_in_flight = false;
    model.help_active = true;
    assert!(!model.input_enabled());
    model.help_active = false;
    model.system_popup_active = true;
    assert!(!model.input_enabled());
    assert!(!model.input_boundary().escape_close_gate_enabled);
}
