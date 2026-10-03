use super::*;

#[test]
fn mismatched_success_is_fail_closed_and_preserves_pending_request() {
    let mut model = model_with(500, &[(0, item(7, 90, 12))]);
    let request = model
        .request_register_item(Pc2pcModalState::default(), 0, 0, Some(4))
        .unwrap();
    let before_snapshot = model.snapshot.clone();
    let before_pending = model.state.pending.clone();
    let result = model.apply_server_outcome(
        Pc2pcServerOutcome0104::RegisterItemSuccess {
            envelope: local_envelope(),
            trade_item: Pc2pcTradeItem0104 {
                offer_slot: 1,
                ..request.item
            },
            inventory_item: request.item,
        },
        Pc2pcBackendCapabilities::default(),
        &Catalog,
        &AllowEquip,
    );
    assert!(matches!(
        result,
        Err(Pc2pcCorrelationError0104::RegisterItemMismatch { .. })
    ));
    assert_eq!(model.snapshot, before_snapshot);
    assert_eq!(model.state.pending, before_pending);

    let wrong_pair = Pc2pcPair0104::new(LOCAL_ID, 3_003).unwrap();
    let result = model.apply_server_outcome(
        Pc2pcServerOutcome0104::RegisterItemSuccess {
            envelope: Pc2pcEnvelope0104 {
                pair: wrong_pair,
                requester_pc_id: LOCAL_ID,
            },
            trade_item: request.item,
            inventory_item: request.item,
        },
        Pc2pcBackendCapabilities::default(),
        &Catalog,
        &AllowEquip,
    );
    assert!(matches!(
        result,
        Err(Pc2pcCorrelationError0104::PairMismatch { .. })
    ));
    assert_eq!(model.snapshot, before_snapshot);
    assert_eq!(model.state.pending, before_pending);
}

#[test]
fn talk_player_offer_request_is_exact_registered_0104_body() {
    let (identity, request) =
        Pc2pcOfferRequest0104::outgoing_offer(LOCAL_ID, REMOTE_ID).unwrap();
    assert_eq!(identity.local_pc_id, LOCAL_ID);
    assert_eq!(identity.remote_pc_id, REMOTE_ID);
    assert_eq!(identity.direction, Pc2pcOfferDirection0104::Outgoing);

    let registered = request.encode_registered().unwrap();
    assert_eq!(
        registered.packet_type(),
        PC2PC_TRADE_OFFER_REQUEST_PACKET_ID_0104
    );
    assert_eq!(registered.payload().len(), 12);
    assert_eq!(&registered.payload()[0..4], &LOCAL_ID.to_le_bytes());
    assert_eq!(&registered.payload()[4..8], &LOCAL_ID.to_le_bytes());
    assert_eq!(&registered.payload()[8..12], &REMOTE_ID.to_le_bytes());
}

#[test]
fn offer_reply_decoder_is_strict_and_preserves_unrelated_frames() {
    let incoming = offer_reply_frame(
        PC2PC_TRADE_OFFER_RESPONSE_PACKET_ID_0104,
        REMOTE_ID,
        REMOTE_ID,
        LOCAL_ID,
        None,
    );
    assert_eq!(
        decode_pc2pc_offer_frame_0104(&incoming).unwrap(),
        Some(Pc2pcOfferReply0104 {
            envelope: Pc2pcEnvelope0104 {
                pair: Pc2pcPair0104::new(REMOTE_ID, LOCAL_ID).unwrap(),
                requester_pc_id: REMOTE_ID,
            },
            kind: Pc2pcOfferReplyKind0104::Offered,
        })
    );

    let unrelated = DecodedFrame {
        packet_type: 0x3100_0001,
        flags: 0,
        checksum: 0,
        payload: vec![1, 2, 3],
    };
    assert_eq!(decode_pc2pc_offer_frame_0104(&unrelated).unwrap(), None);

    let mut malformed = incoming;
    malformed.payload.pop();
    assert!(matches!(
        decode_pc2pc_offer_frame_0104(&malformed),
        Err(Pc2pcOfferFrameError0104::Payload(PayloadError::WrongSize {
            expected: 12,
            actual: 11
        }))
    ));

    let mut runtime = Pc2pcOfferRuntime0104::default();
    let before = runtime.clone();
    assert!(matches!(
        runtime.apply_frame(LOCAL_ID, &malformed),
        Err(Pc2pcOfferIngressError0104::Frame(
            Pc2pcOfferFrameError0104::Payload(PayloadError::WrongSize {
                expected: 12,
                actual: 11
            })
        ))
    ));
    assert_eq!(runtime, before);
}
