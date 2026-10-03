use super::*;

pub(super) fn runtime(inventory: &[(usize, ItemBase0104)]) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::zeroed();
    for &(slot, value) in inventory {
        write_item_base(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    write_item_base(
        load.as_bytes_mut(),
        PcLoadData0104::EQUIPMENT_OFFSET,
        item(0, 77, 0),
    );
    InventoryRuntime0104::from_pc_load(LOCAL_ID, &load)
}

#[test]
fn register_and_cash_intents_do_not_mutate_inventory_taros_or_offer() {
    let mut model = model_with(500, &[(0, item(7, 90, 12))]);
    let before = model.snapshot.clone().unwrap();
    let register = model
        .request_register_item(Pc2pcModalState::default(), 0, 2, Some(4))
        .unwrap();
    assert_eq!(
        register.item,
        Pc2pcTradeItem0104 {
            item_type: 7,
            item_id: 90,
            option: 4,
            inventory_slot: 0,
            offer_slot: 2,
        }
    );
    assert_eq!(model.snapshot.as_ref().unwrap(), &before);
    assert_eq!(model.outbox.len(), 1);
    assert_eq!(model.outbox.0.front().unwrap().packet_id(), 0x1300_002a);

    assert_eq!(
        model.request_register_taros(
            Pc2pcModalState::default(),
            capabilities_with_popup(),
            200,
        ),
        Err(Pc2pcActionError0104::RequestPending)
    );

    let mut cash_model = model_with(500, &[(0, item(7, 90, 12))]);
    let cash = cash_model
        .request_register_taros(Pc2pcModalState::default(), capabilities_with_popup(), 200)
        .unwrap();
    assert_eq!(cash.taros, 200);
    let snapshot = cash_model.snapshot.as_ref().unwrap();
    assert_eq!(snapshot.local_wallet_taros(), 500);
    assert_eq!(snapshot.local_offer_taros(), 0);
}

#[test]
fn local_chat_echo_requires_exact_pending_text_and_preserves_state_on_mismatch() {
    let mut model = model_with(500, &[]);
    let capabilities = Pc2pcBackendCapabilities {
        free_chat_backend: true,
        ..default()
    };
    let intent = model
        .request_chat(
            Pc2pcModalState::default(),
            capabilities,
            Some(true),
            "exact text",
        )
        .unwrap();
    let pending = model.state.pending.clone();
    let result = model.apply_server_outcome(
        Pc2pcServerOutcome0104::ChatMessage {
            envelope: local_envelope(),
            text: "different text".to_owned(),
            emote_code: intent.emote_code,
        },
        capabilities,
        &Catalog,
        &AllowEquip,
    );
    assert!(matches!(
        result,
        Err(Pc2pcCorrelationError0104::ChatMismatch { .. })
    ));
    assert_eq!(model.state.pending, pending);
    assert!(model.chat.is_empty());

    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::ChatMessage {
                envelope: intent.envelope,
                text: intent.text,
                emote_code: intent.emote_code,
            },
            capabilities,
            &Catalog,
            &AllowEquip,
        )
        .unwrap();
    assert!(model.state.pending.is_none());
    assert_eq!(model.chat.len(), 1);
}

#[test]
fn primary_inventory_skin_font_and_padding_metrics_stay_exact() {
    assert_eq!(PC2PC_INVENTORY_SKIN_PATH_ID, 1_366);
    assert_eq!(PC2PC_JEFFE_12_SOURCE_FONT_PATH_ID, 977);
    assert_eq!(PC2PC_JEFFE_12_FONT_SIZE, 12.0);
    assert_eq!(PC2PC_JEFFE_12_LINE_HEIGHT, 13.560_000_42);
    assert_eq!(PC2PC_JEFFE_14_SOURCE_FONT_PATH_ID, 933);
    assert_eq!(PC2PC_JEFFE_14_FONT_SIZE, 12.0);
    assert_eq!(PC2PC_JEFFE_14_LINE_HEIGHT, 11.300_000_19);
    assert_eq!(PC2PC_JEFFE_16_SOURCE_FONT_PATH_ID, 1_008);
    assert_eq!(PC2PC_JEFFE_16_FONT_SIZE, 14.0);
    assert_eq!(PC2PC_JEFFE_16_LINE_HEIGHT, 13.560_000_42);
    assert_eq!(PC2PC_CHALET_SMALL_SOURCE_FONT_PATH_ID, 949);
    assert_eq!(PC2PC_CHALET_SMALL_FONT_SIZE, 12.0);
    assert_eq!(PC2PC_CHALET_SMALL_LINE_HEIGHT, 13.560_000_42);
    assert_eq!(PC2PC_LABEL_PADDING_TOP, 3.0);
    assert_eq!(PC2PC_LABEL_PADDING_BOTTOM, 3.0);
    assert_eq!(PC2PC_BUTTON_PADDING_LEFT, 6.0);
    assert_eq!(PC2PC_BUTTON_PADDING_RIGHT, 6.0);
    assert_eq!(PC2PC_BUTTON_PADDING_TOP, 3.0);
    assert_eq!(PC2PC_BUTTON_PADDING_BOTTOM, 3.0);
    assert_eq!(PC2PC_READY_NAME_GAP, 5.0);
}

#[test]
fn offer_runtime_correlates_outgoing_and_incoming_sessions_fail_closed() {
    let mut outgoing = Pc2pcOfferRuntime0104::default();
    let request = outgoing.request_outgoing(LOCAL_ID, REMOTE_ID).unwrap();
    assert_eq!(outgoing.pop_request(), Some(request));
    assert!(matches!(
        outgoing.pending(),
        Some(Pc2pcPendingOffer0104::Outgoing(_))
    ));

    let wrong = Pc2pcOfferReply0104 {
        envelope: Pc2pcEnvelope0104 {
            pair: Pc2pcPair0104::new(LOCAL_ID, REMOTE_ID + 1).unwrap(),
            requester_pc_id: REMOTE_ID + 1,
        },
        kind: Pc2pcOfferReplyKind0104::Accepted,
    };
    assert!(matches!(
        outgoing.apply_reply(LOCAL_ID, wrong),
        Err(Pc2pcOfferFlowError0104::PairMismatch { .. })
    ));
    assert!(outgoing.accepted().is_none());
    assert!(outgoing.pending().is_some());

    let accepted = Pc2pcOfferReply0104 {
        envelope: Pc2pcEnvelope0104 {
            pair: Pc2pcPair0104::new(LOCAL_ID, REMOTE_ID).unwrap(),
            requester_pc_id: REMOTE_ID,
        },
        kind: Pc2pcOfferReplyKind0104::Accepted,
    };
    outgoing.apply_reply(LOCAL_ID, accepted).unwrap();
    assert_eq!(outgoing.take_accepted().unwrap().remote_pc_id, REMOTE_ID);
    assert!(outgoing.pending().is_none());

    let mut incoming = Pc2pcOfferRuntime0104::default();
    incoming
        .apply_reply(
            LOCAL_ID,
            Pc2pcOfferReply0104 {
                envelope: Pc2pcEnvelope0104 {
                    pair: Pc2pcPair0104::new(REMOTE_ID, LOCAL_ID).unwrap(),
                    requester_pc_id: REMOTE_ID,
                },
                kind: Pc2pcOfferReplyKind0104::Offered,
            },
        )
        .unwrap();
    let accept = incoming.accept_incoming().unwrap();
    assert_eq!(accept.kind, Pc2pcOfferRequestKind0104::Accept);
    assert_eq!(accept.envelope.requester_pc_id, LOCAL_ID);
    assert_eq!(incoming.pop_request(), Some(accept));
    assert!(matches!(
        incoming.accept_incoming(),
        Err(Pc2pcOfferFlowError0104::TransitionAlreadyQueued {
            kind: Pc2pcOfferRequestKind0104::Accept
        })
    ));
    assert!(matches!(
        incoming.refuse_incoming(),
        Err(Pc2pcOfferFlowError0104::TransitionAlreadyQueued {
            kind: Pc2pcOfferRequestKind0104::Accept
        })
    ));

    let mut cancelling = Pc2pcOfferRuntime0104::default();
    cancelling.request_outgoing(LOCAL_ID, REMOTE_ID).unwrap();
    cancelling.pop_request();
    let cancel = cancelling.cancel_outgoing().unwrap();
    assert!(cancelling.pending().is_some());
    assert_eq!(cancelling.pop_request(), Some(cancel));
    assert!(matches!(
        cancelling.cancel_outgoing(),
        Err(Pc2pcOfferFlowError0104::TransitionAlreadyQueued {
            kind: Pc2pcOfferRequestKind0104::Cancel
        })
    ));
    cancelling
        .apply_reply(
            LOCAL_ID,
            Pc2pcOfferReply0104 {
                envelope: cancel.envelope,
                kind: Pc2pcOfferReplyKind0104::Cancelled,
            },
        )
        .unwrap();
    assert!(cancelling.pending().is_none());
}
