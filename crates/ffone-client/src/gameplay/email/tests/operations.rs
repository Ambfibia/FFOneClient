use super::*;

pub(super) fn sample_item(seed: i32) -> EmailWireItem {
    EmailWireItem {
        item_type: seed as i16,
        item_id: (seed + 1) as i16,
        option: seed + 2,
        time_limit: seed + 3,
    }
}

pub(super) fn sample_outgoing(seed: i32) -> EmailOutgoingItem {
    EmailOutgoingItem {
        inventory_slot: seed,
        item: sample_item(seed + 10),
    }
}

pub(super) fn base_item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 77,
    }
}

pub(super) fn empty_base_item() -> ItemBase0104 {
    ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    }
}

pub(super) fn authority_with(item: Option<ItemBase0104>) -> EmailInventoryAuthority0104 {
    let mut slots = [empty_base_item(); EMAIL_INVENTORY_SLOT_COUNT];
    if let Some(item) = item {
        slots[0] = item;
    }
    EmailInventoryAuthority0104 {
        owner_pc_id: 42,
        slots,
    }
}

pub(super) fn open_harness(
    item: Option<ItemBase0104>,
    item_policy: EmailItemFeaturePolicy0104,
    cursor_was_locked: bool,
) -> (ProductionHarness, EmailProductionOutput0104) {
    let inventory = authority_with(item);
    let mut harness = ProductionHarness {
        production: EmailProductionRuntime0104::default(),
        model: EmailUiModel::default(),
        actions: EmailUiOutbox::default(),
        audio: EmailUiAudioOutbox::default(),
        transport: EmailTransportOutbox::default(),
        network: EmailNetworkRuntime0104::default(),
        inbox: EmailNetworkInbox0104::default(),
        inventory,
    };
    let output = harness
        .production
        .open(
            EmailOpenContext0104 {
                player: EmailPlayerAuthority0104 {
                    owner_pc_id: 42,
                    taros: 1_000,
                    current_local_time: EmailSystemTime {
                        year: 2010,
                        month: 1,
                        day: 4,
                        ..EmailSystemTime::default()
                    },
                },
                cursor_was_locked,
                item_policy,
            },
            vec![EmailGuideMessage {
                sender_name: "Dexter".to_owned(),
                subject: "Guide".to_owned(),
                content: "Welcome".to_owned(),
                ..EmailGuideMessage::default()
            }],
            vec![EmailBuddy {
                pc_uid: 55,
                first_name: "Blossom".to_owned(),
                last_name: "Utonium".to_owned(),
                name_check_flag: 1,
            }],
            &harness.inventory,
            &TestCatalog,
            &mut harness.model,
            &mut harness.actions,
            &mut harness.audio,
            &harness.network,
            &harness.inbox,
            &harness.transport,
        )
        .expect("open Email production harness");
    (harness, output)
}

pub(super) fn prepare_send(harness: &mut ProductionHarness, item: ItemBase0104) -> EmailRequest {
    harness.model.screen = EmailScreen::Compose;
    harness.model.opening_elapsed_seconds = 99.0;
    harness.model.draft.recipient_pc_uid = 55;
    harness.model.draft.recipient_name = "BlossomUtonium".to_owned();
    harness.model.draft.subject = "Subject".to_owned();
    harness.model.draft.content = "Body".to_owned();
    harness.model.draft.attachments[0] = Some(EmailOutgoingItem {
        inventory_slot: 0,
        item: email_item_from_base(item),
    });
    let request = harness
        .model
        .draft
        .request(harness.model.available_cash)
        .expect("valid compose draft");
    harness.transport.push(request.clone());
    harness.model.send_in_flight = true;
    harness.model.mail_send_in_flight = true;
    request
}

#[test]
fn production_open_projects_feeds_and_clean_close_restores_saved_cursor() {
    let item = base_item(7, 12, 3);
    let (mut harness, opened) =
        open_harness(Some(item), EmailItemFeaturePolicy0104::default(), true);
    assert_eq!(harness.production.mode_lease().unwrap().game_mode, 18);
    assert_eq!(harness.model.folder, EmailFolder::Guide);
    assert_eq!(harness.model.buddies.len(), 1);
    assert_eq!(
        harness.model.inventory[0].as_ref().unwrap().item.item_id,
        12
    );
    assert_eq!(
        harness.model.inventory[0]
            .as_ref()
            .unwrap()
            .count_label
            .as_deref(),
        Some("3")
    );
    assert_eq!(
        opened.actions,
        vec![
            EmailUiAction::StartUiModeSound,
            EmailUiAction::SetCursorLocked(false),
            EmailUiAction::RefreshGuideEmail {
                event_group: 15,
                event_function: 7,
            },
            EmailUiAction::SetInventoryMailMode {
                event_group: 11,
                event_function: 0,
                value: 4,
            },
        ]
    );

    let gate = harness
        .production
        .request_close(
            EmailCloseSource::EmailKey,
            &mut harness.model,
            &mut harness.actions,
            &mut harness.audio,
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        gate.actions.as_slice(),
        [EmailUiAction::QueryComputressExitGate { .. }]
    ));
    let closed = harness
        .production
        .resolve_computress_gate(
            false,
            &mut harness.model,
            &mut harness.actions,
            &mut harness.audio,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        closed.actions,
        vec![
            EmailUiAction::SetInventoryMailMode {
                event_group: 11,
                event_function: 0,
                value: 10,
            },
            EmailUiAction::SetCursorLocked(true),
            EmailUiAction::ExitMode {
                event_group: 2,
                event_function: 1,
            },
            EmailUiAction::StopUiModeSound,
        ]
    );
    assert!(!harness.model.visible);
    assert!(!harness.production.modal_active());
}

#[test]
fn absent_artwork_does_not_change_attachment_authorization() {
    struct NoArtwork;
    impl EmailItemCatalog0104 for NoArtwork {
        fn resolve(&self, item: ItemBase0104) -> Option<EmailItemCatalogMetadata0104> {
            let mut metadata = TestCatalog.resolve(item)?;
            metadata.icon_path = None;
            Some(metadata)
        }
    }
    let policy = EmailItemFeaturePolicy0104::default();
    assert!(validate_attachment_eligibility(0, base_item(7, 1, 1), policy, &NoArtwork).is_ok());
    assert!(matches!(validate_attachment_eligibility(0, base_item(7, 903, 1), policy, &NoArtwork),
        Err(EmailProductionError0104::AttachmentRejected { reason: EmailAttachmentRejection0104::NotTradeable, .. })));
    assert!(matches!(validate_attachment_eligibility(0, base_item(7, 900, 1), policy, &NoArtwork),
        Err(EmailProductionError0104::MissingCatalogItem { .. })));
}

#[test]
fn buddy_feed_preserves_order_and_rejects_ambiguous_pcuid() {
    let buddies = vec![
        EmailBuddy {
            pc_uid: 77,
            first_name: "A".to_owned(),
            ..EmailBuddy::default()
        },
        EmailBuddy {
            pc_uid: 77,
            first_name: "B".to_owned(),
            ..EmailBuddy::default()
        },
    ];
    assert_eq!(
        project_email_buddies_0104(&buddies),
        Err(EmailProductionError0104::DuplicateBuddy {
            first_index: 0,
            second_index: 1,
            pc_uid: 77,
        })
    );
}

#[test]
fn buddy_ui_adapter_filters_blocked_slots_and_preserves_authority_order() {
    let mut model = BuddyUiModel::default();
    model
        .set_entry(
            3,
            Some(crate::buddy_ui::BuddyEntry {
                pc_uid: 33,
                first_name: "Third".to_owned(),
                name_check_flag: 1,
                ..crate::buddy_ui::BuddyEntry::default()
            }),
        )
        .unwrap();
    model
        .set_entry(
            1,
            Some(crate::buddy_ui::BuddyEntry {
                pc_uid: 11,
                first_name: "First".to_owned(),
                name_check_flag: 1,
                ..crate::buddy_ui::BuddyEntry::default()
            }),
        )
        .unwrap();
    model
        .set_entry(
            2,
            Some(crate::buddy_ui::BuddyEntry {
                pc_uid: 22,
                blocked: true,
                ..crate::buddy_ui::BuddyEntry::default()
            }),
        )
        .unwrap();
    let projected = email_buddies_from_buddy_ui_0104(&model).unwrap();
    assert_eq!(
        projected
            .iter()
            .map(|buddy| buddy.pc_uid)
            .collect::<Vec<_>>(),
        vec![11, 33]
    );
}

#[test]
fn malformed_and_mismatched_owned_frames_keep_full_controller_correlation() {
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

    let malformed = frame(EMAIL_REP_PAGE_LIST_FAILURE_ID, vec![0; 7]);
    assert!(matches!(
        harness.production.route_frame(
            malformed.clone(),
            &mut harness.model,
            &mut harness.actions,
            &mut harness.transport,
            &mut harness.audio,
            &mut harness.network,
        ),
        EmailFrameDisposition0104::Rejected { frame, .. } if frame == malformed
    ));
    assert_eq!(
        harness.production.pending_request(),
        Some(&EmailRequest::PageList { page: 1 })
    );
    assert_eq!(
        harness.network.0.pending(),
        Some(&EmailPending0104::PageList { page: 1 })
    );

    let mut wrong_body = vec![0; EMAIL_REP_PAGE_LIST_FAILURE_SIZE];
    wrong_body[0] = 2;
    let mismatched = frame(EMAIL_REP_PAGE_LIST_FAILURE_ID, wrong_body);
    assert!(matches!(
        harness.production.route_frame(
            mismatched.clone(),
            &mut harness.model,
            &mut harness.actions,
            &mut harness.transport,
            &mut harness.audio,
            &mut harness.network,
        ),
        EmailFrameDisposition0104::Rejected {
            frame,
            error: EmailProductionError0104::ReplyDoesNotMatchRequest { .. }
        } if frame == mismatched
    ));
    assert_eq!(
        harness.production.pending_request(),
        Some(&EmailRequest::PageList { page: 1 })
    );
}

#[test]
fn attachment_rules_fail_closed_and_preserve_clean_override_order() {
    assert!(matches!(
        validate_attachment_eligibility(
            0,
            base_item(7, 902, 1),
            EmailItemFeaturePolicy0104::default(),
            &TestCatalog,
        ),
        Err(EmailProductionError0104::MissingTradeMetadata { .. })
    ));
    assert!(matches!(
        validate_attachment_eligibility(
            0,
            base_item(7, 904, 1),
            EmailItemFeaturePolicy0104::default(),
            &TestCatalog,
        ),
        Err(EmailProductionError0104::MissingGeneralSubtype { .. })
    ));
    assert!(matches!(
        validate_attachment_eligibility(
            0,
            base_item(7, 905, 1),
            EmailItemFeaturePolicy0104::default(),
            &TestCatalog,
        ),
        Err(EmailProductionError0104::AttachmentRejected {
            reason: EmailAttachmentRejection0104::GeneralEmailSubtype,
            ..
        })
    ));
    let combined_and_enchanted = base_item(0, 1, (2 << 16) | 2);
    assert!(matches!(
        validate_attachment_eligibility(
            0,
            combined_and_enchanted,
            EmailItemFeaturePolicy0104 {
                combine_enabled: true,
                korean_enchant_enabled: false,
            },
            &TestCatalog,
        ),
        Err(EmailProductionError0104::AttachmentRejected {
            reason: EmailAttachmentRejection0104::CombinedLook,
            ..
        })
    ));
    assert!(
        validate_attachment_eligibility(
            0,
            combined_and_enchanted,
            EmailItemFeaturePolicy0104 {
                combine_enabled: true,
                korean_enchant_enabled: true,
            },
            &TestCatalog,
        )
        .is_ok()
    );
}

#[test]
fn correlated_failure_surfaces_keyed_system_message_without_authority_commit() {
    let item = base_item(0, 1, 2);
    let (mut harness, _) =
        open_harness(Some(item), EmailItemFeaturePolicy0104::default(), false);
    let request = prepare_send(&mut harness, item);
    harness
        .production
        .dispatch_next_request(
            &harness.model,
            &mut harness.transport,
            &mut harness.network,
            &TestCatalog,
        )
        .unwrap();
    let EmailRequest::Send {
        recipient_pc_uid, ..
    } = request
    else {
        unreachable!()
    };
    let mut body = vec![0; EMAIL_REP_SEND_FAILURE_SIZE];
    body[0..8].copy_from_slice(&recipient_pc_uid.to_le_bytes());
    body[8..12].copy_from_slice(&4_i32.to_le_bytes());
    let EmailFrameDisposition0104::Applied { output, .. } = harness.production.route_frame(
        frame(EMAIL_REP_SEND_FAILURE_ID, body),
        &mut harness.model,
        &mut harness.actions,
        &mut harness.transport,
        &mut harness.audio,
        &mut harness.network,
    ) else {
        panic!("send failure must apply")
    };
    assert!(output.commit.is_none());
    assert!(output.actions.iter().any(|action| matches!(
        action,
        EmailUiAction::SystemMessage { localized, .. }
            if localized.key == "ui.email.error.send.improper_language"
    )));
    assert!(!harness.model.mail_send_in_flight);
}

#[test]
fn receive_item_acknowledgement_never_invents_missing_item_bytes() {
    let (mut harness, _) = open_harness(None, EmailItemFeaturePolicy0104::default(), false);
    let attached = EmailWireItem {
        item_type: 7,
        item_id: 12,
        option: 3,
        time_limit: 77,
    };
    harness.model.folder = EmailFolder::Player;
    harness.model.player_messages = vec![EmailSummary {
        email_index: 88,
        ..EmailSummary::default()
    }];
    harness.model.selected_row = Some(0);
    harness.model.read_message = Some(EmailReadMessage {
        email_index: 88,
        items: [
            attached,
            EmailWireItem::default(),
            EmailWireItem::default(),
            EmailWireItem::default(),
        ],
        ..EmailReadMessage::default()
    });
    let request = EmailRequest::ReceiveItem {
        email_index: 88,
        inventory_slot: 0,
        email_item_slot: 1,
    };
    harness.transport.push(request);
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

    let mut body = vec![0; EMAIL_REP_RECEIVE_ITEM_SUCCESS_SIZE];
    body[0..8].copy_from_slice(&88_i64.to_le_bytes());
    body[8..12].copy_from_slice(&0_i32.to_le_bytes());
    body[12..16].copy_from_slice(&1_i32.to_le_bytes());
    let EmailFrameDisposition0104::Applied { output, .. } = harness.production.route_frame(
        frame(EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID, body),
        &mut harness.model,
        &mut harness.actions,
        &mut harness.transport,
        &mut harness.audio,
        &mut harness.network,
    ) else {
        panic!("receive item success must apply")
    };
    let commit = output.commit.unwrap();
    assert!(commit.inventory_writes().next().is_none());
    assert!(commit.inventory_refresh_required());
    assert_eq!(
        commit.source(),
        EmailAuthoritySource0104::ReceiveItemSuccess {
            email_index: 88,
            inventory_slot: 0,
            email_item_slot: 1,
        }
    );
    assert!(harness.model.read_message.as_ref().unwrap().items[0].is_empty());
}

#[test]
fn send_layout_matches_pack_four_and_by_val_tstr_marshalling() {
    let long_content = "x".repeat(EMAIL_CONTENT_UNITS + 20);
    let request = EmailRequest::Send {
        recipient_pc_uid: 0x0102_0304_0506_0708,
        subject: "Hi".to_owned(),
        content: long_content,
        items: [
            sample_outgoing(1),
            sample_outgoing(2),
            sample_outgoing(3),
            sample_outgoing(4),
        ],
        cash: 0x1122_3344,
    };
    let packet = encode_email_request_0104(&request).unwrap();
    assert_eq!(&packet.body[0..8], &0x0102_0304_0506_0708_i64.to_le_bytes());
    assert_eq!(&packet.body[8..14], &[b'H', 0, b'i', 0, 0, 0]);
    assert_eq!(
        &packet.body[8 + EMAIL_SUBJECT_UNITS * 2 - 2..8 + EMAIL_SUBJECT_UNITS * 2],
        &[0, 0]
    );
    let content_offset = 8 + EMAIL_SUBJECT_UNITS * 2;
    assert_eq!(
        &packet.body[content_offset + EMAIL_CONTENT_UNITS * 2 - 2
            ..content_offset + EMAIL_CONTENT_UNITS * 2],
        &[0, 0]
    );
    let first_item_offset = content_offset + EMAIL_CONTENT_UNITS * 2;
    assert_eq!(
        read_outgoing_item(&packet.body, first_item_offset),
        sample_outgoing(1)
    );
    assert_eq!(read_i32(&packet.body, packet.body.len() - 4), 0x1122_3344);
}

#[test]
fn page_list_decoder_uses_the_exact_padding_and_204_byte_record_offsets() {
    let mut body = vec![0; EMAIL_REP_PAGE_LIST_SUCCESS_SIZE];
    body[0] = 3;
    let record = EMAIL_SUMMARY_FIRST_OFFSET;
    body[record..record + 8].copy_from_slice(&44_i64.to_le_bytes());
    body[record + 8..record + 16].copy_from_slice(&55_i64.to_le_bytes());
    write_utf16_at(&mut body, record + 16, EMAIL_FIRST_NAME_UNITS, "Dexter");
    write_utf16_at(&mut body, record + 34, EMAIL_LAST_NAME_UNITS, "Monkey");
    write_utf16_at(&mut body, record + 68, EMAIL_SUBJECT_UNITS, "Welcome");
    body[record + 132..record + 136].copy_from_slice(&1_i32.to_le_bytes());
    for (index, value) in [2010, 1, 5, 4, 3, 2, 1, 99].into_iter().enumerate() {
        body[record + 136 + index * 4..record + 140 + index * 4]
            .copy_from_slice(&value_i32(value).to_le_bytes());
    }
    body[record + 200..record + 204].copy_from_slice(&1_i32.to_le_bytes());

    let Some(EmailReply::PageListSuccess { page, messages }) =
        decode_email_reply_0104(EMAIL_REP_PAGE_LIST_SUCCESS_ID, &body).unwrap()
    else {
        panic!("page reply");
    };
    assert_eq!(page, 3);
    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0].email_index, 44);
    assert_eq!(messages[0].from_pc_uid, 55);
    assert_eq!(messages[0].first_name, "Dexter");
    assert_eq!(messages[0].last_name, "Monkey");
    assert_eq!(messages[0].subject, "Welcome");
    assert_eq!(messages[0].send_time.year, 2010);
    assert_eq!(messages[0].send_time.milliseconds, 99);
    assert_eq!(messages[0].item_cash_flag, 1);
    assert_eq!(messages[1], EmailSummary::default());
}

#[test]
fn malformed_and_mismatched_replies_do_not_consume_correlation() {
    let mut runtime = EmailTransportRuntime0104::default();
    runtime
        .begin(&EmailRequest::Read { email_index: 77 })
        .unwrap();
    assert!(matches!(
        runtime.accept(EMAIL_REP_READ_SUCCESS_ID, &[0; 3]),
        Err(EmailRuntimeError0104::Decode(_))
    ));
    assert_eq!(
        runtime.pending(),
        Some(&EmailPending0104::Read { email_index: 77 })
    );

    let mut wrong = vec![0; EMAIL_REP_READ_FAILURE_SIZE];
    wrong[0..8].copy_from_slice(&78_i64.to_le_bytes());
    assert!(matches!(
        runtime.accept(EMAIL_REP_READ_FAILURE_ID, &wrong),
        Err(EmailRuntimeError0104::ReplyDoesNotMatchPending { .. })
    ));
    assert_eq!(
        runtime.pending(),
        Some(&EmailPending0104::Read { email_index: 77 })
    );

    wrong[0..8].copy_from_slice(&77_i64.to_le_bytes());
    assert!(matches!(
        runtime.accept(EMAIL_REP_READ_FAILURE_ID, &wrong),
        Ok(Some(EmailRuntimeDelivery0104::Correlated(
            EmailReply::ReadFailure {
                email_index: 77,
                error_code: 0
            }
        )))
    ));
    assert_eq!(runtime.pending(), None);
}

#[test]
fn invalid_utf16_fails_closed_without_panicking() {
    let mut body = vec![0; EMAIL_REP_READ_SUCCESS_SIZE];
    body[8..10].copy_from_slice(&0xd800_u16.to_le_bytes());
    assert_eq!(
        decode_email_reply_0104(EMAIL_REP_READ_SUCCESS_ID, &body),
        Err(EmailDecodeError0104::InvalidUtf16 {
            packet_id: EMAIL_REP_READ_SUCCESS_ID,
            field: "read-success content",
        })
    );
}

pub(super) const fn value_i32(value: i32) -> i32 {
    value
}
