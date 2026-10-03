use super::*;

pub(super) const fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

pub(super) fn identity() -> Pc2pcSessionIdentity0104 {
    Pc2pcSessionIdentity0104::new(
        Pc2pcPair0104::new(LOCAL_ID, REMOTE_ID).unwrap(),
        LOCAL_ID,
        Pc2pcOfferDirection0104::Outgoing,
    )
    .unwrap()
}

pub(super) fn local_envelope() -> Pc2pcEnvelope0104 {
    Pc2pcEnvelope0104::local(identity())
}

pub(super) fn remote_envelope() -> Pc2pcEnvelope0104 {
    Pc2pcEnvelope0104 {
        pair: identity().pair,
        requester_pc_id: REMOTE_ID,
    }
}

pub(super) fn capabilities_with_popup() -> Pc2pcBackendCapabilities {
    Pc2pcBackendCapabilities {
        numeric_popup_backend: true,
        ..default()
    }
}

#[test]
fn correlated_register_success_changes_only_trade_availability_and_cancel_needs_no_rollback() {
    let original = item(7, 90, 12);
    let mut model = model_with(500, &[(0, original)]);
    let request = model
        .request_register_item(Pc2pcModalState::default(), 0, 0, Some(4))
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: local_envelope(),
                trade_item: request.item,
                inventory_item: Pc2pcTradeItem0104 {
                    option: 8,
                    ..request.item
                },
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap();
    let snapshot = model.snapshot.as_ref().unwrap();
    assert_eq!(snapshot.base_inventory()[0], original);
    assert_eq!(snapshot.available_inventory()[0], item(7, 90, 8));
    assert_eq!(snapshot.local_offer()[0], Some(request.item));
    assert!(model.state.pending.is_none());

    let cancel = model.request_cancel(Pc2pcModalState::default()).unwrap();
    assert_eq!(
        model.snapshot.as_ref().unwrap().base_inventory()[0],
        original
    );
    model
        .acknowledge_local_cancel_sent(cancel.envelope)
        .unwrap();
    assert_eq!(model.state.phase, Pc2pcLifecyclePhase::Cancelled);
    assert_eq!(
        model.snapshot.as_ref().unwrap().base_inventory()[0],
        original
    );
}

#[test]
fn accept_confirm_final_success_commits_only_correlated_authority() {
    let mut model = model_with(1_000, &[(0, item(7, 90, 12))]);
    let register = model
        .request_register_item(Pc2pcModalState::default(), 0, 0, Some(4))
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: local_envelope(),
                trade_item: register.item,
                inventory_item: Pc2pcTradeItem0104 {
                    option: 8,
                    ..register.item
                },
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap();

    model.request_confirm(Pc2pcModalState::default()).unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::Confirmed {
                envelope: local_envelope(),
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::Confirmed {
                envelope: remote_envelope(),
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap();
    assert_eq!(model.state.button_label(), Pc2pcButtonLabel::Accept);
    model.request_confirm(Pc2pcModalState::default()).unwrap();
    assert_eq!(model.state.phase, Pc2pcLifecyclePhase::Completing);

    let mut received = [Pc2pcTradeItem0104::empty(); PC2PC_PROTOCOL_TRADE_ITEM_COUNT];
    received[0] = Pc2pcTradeItem0104 {
        item_type: 0,
        item_id: 120,
        option: 0,
        inventory_slot: 1,
        offer_slot: 0,
    };
    let mut stay = [Pc2pcTradeItem0104::empty(); PC2PC_PROTOCOL_TRADE_ITEM_COUNT];
    stay[0] = Pc2pcTradeItem0104 {
        inventory_slot: 18,
        ..register.item
    };
    let receipt = model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::ConfirmSuccess {
                envelope: local_envelope(),
                received,
                item_stay: stay,
                wallet_taros: 1_250,
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap()
        .unwrap();
    assert_eq!(receipt.received_item_count, 1);
    assert_eq!(receipt.wallet_taros, 1_250);
    assert_eq!(model.state.phase, Pc2pcLifecyclePhase::Completed);
    let snapshot = model.snapshot.as_ref().unwrap();
    assert_eq!(snapshot.base_inventory()[0], item(7, 90, 8));
    assert_eq!(snapshot.base_inventory()[1], item(0, 120, 0));
    assert_eq!(snapshot.available_inventory(), snapshot.base_inventory());
    assert_eq!(snapshot.local_wallet_taros(), 1_250);
    assert!(snapshot.local_offer().iter().all(Option::is_none));
}

#[test]
fn malformed_final_stay_cannot_commit_or_consume_pending_accept() {
    let mut model = model_with(1_000, &[(0, item(7, 90, 12))]);
    let register = model
        .request_register_item(Pc2pcModalState::default(), 0, 0, Some(4))
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: local_envelope(),
                trade_item: register.item,
                inventory_item: Pc2pcTradeItem0104 {
                    option: 8,
                    ..register.item
                },
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::Confirmed {
                envelope: remote_envelope(),
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap();
    model.request_confirm(Pc2pcModalState::default()).unwrap();
    let before = model.snapshot.clone();
    let pending = model.state.pending.clone();
    let result = model.apply_server_outcome(
        Pc2pcServerOutcome0104::ConfirmSuccess {
            envelope: local_envelope(),
            received: [Pc2pcTradeItem0104::empty(); PC2PC_PROTOCOL_TRADE_ITEM_COUNT],
            item_stay: [Pc2pcTradeItem0104::empty(); PC2PC_PROTOCOL_TRADE_ITEM_COUNT],
            wallet_taros: 2_000,
        },
        Pc2pcBackendCapabilities::default(),
        &Catalog,
        &AllowEquip,
    );
    assert!(matches!(
        result,
        Err(Pc2pcCorrelationError0104::Authority(
            Pc2pcAuthorityError0104::FinalStayMissingLocalOffer { .. }
        ))
    ));
    assert_eq!(model.snapshot, before);
    assert_eq!(model.state.pending, pending);
}

#[test]
fn remote_offer_updates_reset_both_ready_flags() {
    let mut model = model_with(500, &[]);
    model.state.local_ready = true;
    model.state.remote_ready = true;
    let remote_item = Pc2pcTradeItem0104 {
        item_type: 1,
        item_id: 42,
        option: 0,
        inventory_slot: 7,
        offer_slot: 3,
    };
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: remote_envelope(),
                trade_item: remote_item,
                inventory_item: remote_item,
            },
            Pc2pcBackendCapabilities::default(),
            &Catalog,
            &AllowEquip,
        )
        .unwrap();
    assert!(!model.state.local_ready);
    assert!(!model.state.remote_ready);
    assert_eq!(
        model.snapshot.as_ref().unwrap().remote_offer()[3],
        Some(remote_item)
    );
}

#[test]
fn chat_and_portraits_fail_closed_without_backends() {
    let mut model = model_with(500, &[]);
    assert_eq!(
        model.request_chat(
            Pc2pcModalState::default(),
            Pc2pcBackendCapabilities::default(),
            Some(true),
            "hello",
        ),
        Err(Pc2pcActionError0104::FreeChatBackendUnavailable)
    );
    assert!(model.outbox.is_empty());
    assert_eq!(Pc2pcPortraitBindings::default().local, None);

    let capabilities = Pc2pcBackendCapabilities {
        free_chat_backend: true,
        ..default()
    };
    assert_eq!(
        model.request_chat(
            Pc2pcModalState::default(),
            capabilities,
            Some(true),
            "/warp 100 100",
        ),
        Err(Pc2pcActionError0104::CheatCommandBlocked {
            command: "/warp".to_owned()
        })
    );
}

#[test]
fn modal_send_and_opening_gates_match_clean_panel() {
    let mut model = model_with(500, &[]);
    model.state.phase = Pc2pcLifecyclePhase::Opening;
    assert_eq!(
        model.request_confirm(Pc2pcModalState::default()),
        Err(Pc2pcActionError0104::PhaseBlocked {
            phase: Pc2pcLifecyclePhase::Opening
        })
    );
    model.state.tick(PC2PC_OPEN_SECONDS);
    let modal = Pc2pcModalState {
        menu_chat_level_one_visible: true,
        ..default()
    };
    assert_eq!(
        model.request_confirm(modal),
        Err(Pc2pcActionError0104::ModalBlocked)
    );
    assert_eq!(
        model.request_cancel(Pc2pcModalState {
            external_exit_blocked: true,
            ..default()
        }),
        Err(Pc2pcActionError0104::ModalBlocked)
    );
    model.request_confirm(Pc2pcModalState::default()).unwrap();
    assert_eq!(
        model.request_cancel(Pc2pcModalState::default()),
        Err(Pc2pcActionError0104::RequestPending)
    );
}

#[test]
fn production_pc2pc_tree_localizes_and_styles_every_text() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(Pc2pcUiPlugin);
    app.update();

    let (jeffe_font, chalet_font) = {
        let assets = app.world().resource::<Pc2pcUiAssets>();
        (assets.jeffe_font.clone(), assets.chalet_font.clone())
    };
    let world = app.world_mut();
    let total_text = world.query::<&Text>().iter(world).count();
    let mut text_query = world.query::<(
        &Text,
        Option<&LocalizedText>,
        Option<&Pc2pcTextStyle0104>,
        (&TextFont, &LineHeight),
    )>();
    let text_rows = text_query
        .iter(world)
        .map(|(text, localized, style, font)| {
            (
                text.0.clone(),
                localized.cloned(),
                style.copied(),
                font.clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(text_rows.len(), 85);
    assert_eq!(
        text_rows.len(),
        total_text,
        "a PC2PC Text entity bypassed the localized spawn helper"
    );
    assert!(text_rows.iter().all(|(_, localized, style, font)| {
        localized
            .as_ref()
            .is_some_and(|localized| !localized.key.is_empty())
            && style.is_some()
            && (font.0.font == bevy::text::FontSource::Handle(jeffe_font.clone())
                || font.0.font == bevy::text::FontSource::Handle(chalet_font.clone()))
    }));
    for expected_key in [
        "ui.pc2pc.button.submit",
        "ui.pc2pc.button.add_taros",
        "ui.pc2pc.chat.send",
        "ui.pc2pc.offer.local",
        "ui.pc2pc.offer.remote",
        "ui.pc2pc.ready.waiting_for",
        "ui.pc2pc.ready.local_subject",
        "ui.pc2pc.taros.amount",
        "ui.pc2pc.chat.log",
        "ui.inventory.tab.equipment",
        "ui.inventory.tab.nanos",
        "ui.inventory.equipped",
        "ui.inventory.item.count",
        "ui.inventory.slot.head",
        "ui.inventory.slot.weapon",
        "ui.inventory.slot.vehicle",
    ] {
        assert!(
            text_rows.iter().any(|(_, localized, _, _)| {
                localized
                    .as_ref()
                    .is_some_and(|localized| localized.key == expected_key)
            }),
            "missing semantic PC2PC text role {expected_key}"
        );
    }

    let mut element_query = world.query::<(
        &Pc2pcUiElement,
        &Pc2pcTextStyle0104,
        (&TextFont, &LineHeight),
        &Node,
    )>();
    let styled_elements = element_query
        .iter(world)
        .map(|(element, style, font, node)| (*element, *style, font.clone(), node.clone()))
        .collect::<Vec<_>>();
    let main_button = styled_elements
        .iter()
        .find(|(element, _, _, _)| *element == Pc2pcUiElement::MainButtonText)
        .unwrap();
    assert_eq!(main_button.1, Pc2pcTextStyle0104::Button);
    assert_eq!(
        main_button.2.0.font_size.eval(Vec2::ZERO, 16.0),
        PC2PC_JEFFE_14_FONT_SIZE
    );
    assert_eq!(
        (*main_button.2.1),
        LineHeight::Px(PC2PC_JEFFE_14_LINE_HEIGHT)
    );
    assert_eq!(main_button.3.padding.left, px(PC2PC_BUTTON_PADDING_LEFT));
    assert_eq!(main_button.3.padding.top, px(PC2PC_BUTTON_PADDING_TOP));

    let local_title = styled_elements
        .iter()
        .find(|(element, _, _, _)| *element == Pc2pcUiElement::LocalTitle)
        .unwrap();
    assert_eq!(local_title.1, Pc2pcTextStyle0104::LabelMiddleRight);
    assert_eq!(local_title.3.justify_content, JustifyContent::FlexEnd);
    assert_eq!(local_title.3.padding.top, px(PC2PC_LABEL_PADDING_TOP));

    let chat_input = styled_elements
        .iter()
        .find(|(element, _, _, _)| *element == Pc2pcUiElement::ChatInput)
        .unwrap();
    assert_eq!(chat_input.1, Pc2pcTextStyle0104::ChatInput);
    assert_eq!(
        chat_input.2.0.font,
        bevy::text::FontSource::Handle(chalet_font.clone())
    );
    assert_eq!(
        chat_input.2.0.font_size.eval(Vec2::ZERO, 16.0),
        PC2PC_CHALET_SMALL_FONT_SIZE
    );
    assert_eq!(
        (*chat_input.2.1),
        LineHeight::Px(PC2PC_CHALET_SMALL_LINE_HEIGHT)
    );

    let equipment_slot = styled_elements
        .iter()
        .find(|(element, _, _, _)| *element == Pc2pcUiElement::EquipmentSlotLabel(0))
        .unwrap();
    assert_eq!(equipment_slot.1, Pc2pcTextStyle0104::EquipmentSlot);
    assert_eq!(equipment_slot.3.top, px(-2));
    assert_eq!(equipment_slot.3.align_items, AlignItems::FlexEnd);

    let mut node_query = world.query::<(&Pc2pcUiElement, &Node)>();
    let (_, ready_name) = node_query
        .iter(world)
        .find(|(element, _)| **element == Pc2pcUiElement::ReadyName)
        .unwrap();
    assert_eq!(ready_name.column_gap, px(PC2PC_READY_NAME_GAP));
}

#[test]
fn final_success_after_local_confirm_ack_commits_once() {
    let mut model = model_with(500, &[]);
    model.apply_server_outcome(Pc2pcServerOutcome0104::Confirmed { envelope: remote_envelope() }, Default::default(), &Catalog, &AllowEquip).unwrap();
    model.request_confirm(Default::default()).unwrap();
    model.apply_server_outcome(Pc2pcServerOutcome0104::Confirmed { envelope: local_envelope() }, Default::default(), &Catalog, &AllowEquip).unwrap();
    assert!(model.state.pending.is_none());
    let blank = [Pc2pcTradeItem0104::default(); PC2PC_PROTOCOL_TRADE_ITEM_COUNT];
    let outcome = Pc2pcServerOutcome0104::ConfirmSuccess { envelope: local_envelope(), received: blank, item_stay: blank, wallet_taros: 500 };
    assert!(model.apply_server_outcome(outcome.clone(), Default::default(), &Catalog, &AllowEquip).unwrap().is_some());
    assert!(model.completed_inventory().is_some());
    assert!(model.apply_server_outcome(outcome, Default::default(), &Catalog, &AllowEquip).is_err());
}
