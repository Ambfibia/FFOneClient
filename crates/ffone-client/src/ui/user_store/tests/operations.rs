use super::*;

pub(super) fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

pub(super) fn user_store_template_args(template: &str) -> BTreeSet<&str> {
    let mut output = BTreeSet::new();
    let mut remaining = template;
    while let Some(open) = remaining.find('{') {
        remaining = &remaining[open + 1..];
        let Some(close) = remaining.find('}') else {
            break;
        };
        output.insert(&remaining[..close]);
        remaining = &remaining[close + 1..];
    }
    output
}

#[test]
fn malformed_stray_and_mismatched_replies_preserve_busy_and_authority() {
    let mut state = UserStoreUiState0104::default();
    let mut authority = UserStoreAuthority0104::default();
    authority.inventory[0] = item(0, 10, 1);
    let mut outbox = UserStoreUiOutbox0104::default();
    let _ = state.begin_my_store(&mut authority, OWNER, 0, true, &mut outbox);
    let before = authority.clone();
    let pending = state.pending;
    assert!(
        state
            .receive_reply(
                &mut authority,
                STREETSTALL_REP_READY_SUCCESS,
                &[0; 15],
                &mut outbox,
            )
            .is_err()
    );
    assert_eq!(state.pending, pending);
    assert_eq!(authority, before);
    assert!(
        state
            .receive_reply(
                &mut authority,
                STREETSTALL_REP_CANCEL_SUCCESS,
                &[0],
                &mut outbox,
            )
            .is_err()
    );
    assert_eq!(state.pending, pending);
    assert_eq!(authority, before);

    let mut ready = fixed_reply_payload(STREETSTALL_REP_READY_SUCCESS);
    write_i32(&mut ready, 0, 1);
    assert!(
        state
            .receive_reply(
                &mut authority,
                STREETSTALL_REP_READY_SUCCESS,
                &ready,
                &mut outbox,
            )
            .is_err()
    );
    assert_eq!(state.pending, pending);
    assert_eq!(authority, before);

    let mut idle = UserStoreUiState0104::default();
    assert!(
        idle.receive_reply(
            &mut authority,
            STREETSTALL_REP_READY_FAIL,
            &[0; 4],
            &mut outbox,
        )
        .is_err()
    );
}

#[test]
fn register_limits_and_open_store_restrictions_match_clean_messages() {
    let (mut state, mut authority, mut outbox) = ready_state();
    authority.inventory[2] = item(0, 77, 1);
    authority.store_open = true;
    assert_eq!(
        state.request_register(&authority, 2, authority.inventory[2], 5, &mut outbox),
        UserStoreActionOutcome0104::SystemMessage(USER_STORE_CANNOT_REGISTER)
    );
    authority.listings[0] = Some(UserStoreListing0104 {
        list_slot: 0,
        inventory_slot: Some(2),
        item: authority.inventory[2],
        price: 5,
    });
    assert_eq!(
        state.activate_store_row(&authority, 0, UserStoreRowButton0104::Primary, &mut outbox,),
        UserStoreActionOutcome0104::SystemMessage(USER_STORE_CANNOT_UNREGISTER)
    );
    authority.store_open = false;
    authority.maximum_list_slots = 1;
    assert_eq!(
        state.request_register(&authority, 2, authority.inventory[2], 5, &mut outbox),
        UserStoreActionOutcome0104::SystemMessage(USER_STORE_REGISTER_OVER_MAX)
    );
}

#[test]
fn opening_empty_store_is_a_noop_but_occupied_store_sends() {
    let (mut state, mut authority, mut outbox) = ready_state();
    assert_eq!(
        state.request_open_store(&authority, &mut outbox),
        UserStoreActionOutcome0104::NoOp
    );
    authority.listings[2] = Some(UserStoreListing0104 {
        list_slot: 2,
        inventory_slot: Some(3),
        item: item(0, 77, 1),
        price: 100,
    });
    let sent = state.request_open_store(&authority, &mut outbox);
    assert!(matches!(sent, UserStoreActionOutcome0104::Sent(_)));
    assert_eq!(
        state.pending,
        Some(UserStorePendingRequest0104::SaleStart {
            open_item_inventory_slot: 0
        })
    );
}

#[test]
fn buyer_and_async_seller_successes_are_atomic_and_mark_sold() {
    let listing_item = item(7, 88, 3);
    let mut buyer_state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::UserStore,
        ..default()
    };
    let mut buyer = UserStoreAuthority0104 {
        owner_pc_id: OWNER,
        target_pc_id: Some(TARGET),
        taros: 10_000,
        ..default()
    };
    buyer.listings[2] = Some(UserStoreListing0104 {
        list_slot: 2,
        inventory_slot: None,
        item: listing_item,
        price: 5_000,
    });
    let mut outbox = UserStoreUiOutbox0104::default();
    let _ = buyer_state.request_buy(&buyer, 2, &mut outbox);
    let destination = match buyer_state.pending.expect("buy pending") {
        UserStorePendingRequest0104::Buy {
            empty_inventory_slot,
            ..
        } => empty_inventory_slot,
        _ => panic!("wrong pending"),
    };
    let mut payload = vec![0_u8; STREETSTALL_ITEM_BUY_SUCCESS_SIZE];
    write_i32(&mut payload, 0, TARGET);
    write_i32(&mut payload, 4, 4_900);
    write_i32(&mut payload, 8, destination);
    encode_item(listing_item, &mut payload[12..24]);
    write_i32(&mut payload, 24, 2);
    buyer_state
        .receive_reply(
            &mut buyer,
            STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER,
            &payload,
            &mut outbox,
        )
        .expect("buyer commit");
    assert_eq!(buyer.taros, 4_900);
    assert_eq!(buyer.inventory[destination as usize], listing_item);
    assert_eq!(buyer.listings[2].expect("retained sold row").price, -1);

    let mut seller_state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::MyStore,
        pending: Some(UserStorePendingRequest0104::Cancel { owner_pc_id: OWNER }),
        ..default()
    };
    let mut seller = UserStoreAuthority0104 {
        owner_pc_id: OWNER,
        taros: 100,
        ..default()
    };
    seller.listings[4] = Some(UserStoreListing0104 {
        list_slot: 4,
        inventory_slot: Some(17),
        item: listing_item,
        price: 1_000,
    });
    let pending = seller_state.pending;
    let mut seller_payload = vec![0_u8; STREETSTALL_ITEM_BUY_SUCCESS_SIZE];
    write_i32(&mut seller_payload, 0, 404);
    write_i32(&mut seller_payload, 4, 1_050);
    write_i32(&mut seller_payload, 8, 17);
    encode_item(listing_item, &mut seller_payload[12..24]);
    write_i32(&mut seller_payload, 24, 4);
    seller_state
        .receive_reply(
            &mut seller,
            STREETSTALL_REP_ITEM_BUY_SUCCESS_SELLER,
            &seller_payload,
            &mut outbox,
        )
        .expect("async seller commit");
    assert_eq!(
        seller_state.pending, pending,
        "async sale keeps unrelated lock"
    );
    assert_eq!(seller.taros, 1_050);
    assert_eq!(seller.listings[4].expect("retained sold row").price, -1);
}

#[test]
fn seller_display_uses_clean_five_percent_truncation_quirk() {
    assert_eq!(user_store_seller_display_taros(100), 95);
    assert_eq!(user_store_seller_display_taros(99), 95);
    assert_eq!(user_store_seller_display_taros(1), 1);
    assert_eq!(user_store_format_number(1_234_567), "1,234,567");
    assert_eq!(user_store_format_number(-1), "-1");
}

#[test]
fn buyer_closes_locally_owner_waits_for_cancel_and_target_close_exits() {
    let mut state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::UserStore,
        ..default()
    };
    let mut authority = UserStoreAuthority0104 {
        owner_pc_id: OWNER,
        target_pc_id: Some(TARGET),
        ..default()
    };
    let mut outbox = UserStoreUiOutbox0104::default();
    assert_eq!(
        state.request_close(&authority, false, &mut outbox),
        UserStoreActionOutcome0104::LocalExit
    );
    assert!(!state.active);
    assert!(!outbox.0.iter().any(|command| matches!(
        command,
        UserStoreUiCommand0104::SendPacket(UserStorePacket0104 {
            packet_id: STREETSTALL_REQ_CANCEL,
            ..
        })
    )));

    state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::MyStore,
        ..default()
    };
    outbox.clear();
    assert!(matches!(
        state.request_close(&authority, false, &mut outbox),
        UserStoreActionOutcome0104::Sent(_)
    ));
    assert!(state.active);
    state
        .receive_reply(
            &mut authority,
            STREETSTALL_REP_CANCEL_SUCCESS,
            &[0],
            &mut outbox,
        )
        .expect("cancel success");
    assert!(!state.active);

    state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::UserStore,
        ..default()
    };
    outbox.clear();
    assert!(state.target_store_state_change(&authority, TARGET, 0, &mut outbox));
    assert!(!state.active);
    assert!(outbox.0.iter().any(|command| matches!(
        command,
        UserStoreUiCommand0104::SystemMessage(message)
            if message == USER_STORE_TARGET_CLOSED_MESSAGE_KEY
    )));
}

#[test]
fn escape_and_controls_honor_busy_modal_help_and_external_gates() {
    let mut state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::UserStore,
        ..default()
    };
    assert!(state.input_boundary().escape_enabled);
    state.modal.system_popup = true;
    assert!(!state.input_boundary().controls_enabled);
    state.modal = UserStoreModalState0104 {
        item_popup: true,
        ..default()
    };
    assert!(!state.input_boundary().close_enabled);
    state.modal = UserStoreModalState0104 {
        help: true,
        ..default()
    };
    assert!(!state.input_boundary().escape_enabled);
    state.modal = UserStoreModalState0104 {
        external_gameplay_gate: true,
        ..default()
    };
    assert!(state.input_boundary().close_enabled);
    assert!(!state.input_boundary().escape_enabled);
    state.modal = UserStoreModalState0104 {
        external_close_gate: true,
        ..default()
    };
    assert!(!state.input_boundary().close_enabled);
    state.modal = UserStoreModalState0104::default();
    state.pending = Some(UserStorePendingRequest0104::ItemList {
        target_pc_id: TARGET,
    });
    assert!(!state.input_boundary().controls_enabled);
}

#[test]
fn all_eleven_renderer_preview_modes_are_deterministic() {
    for mode in UserStorePreviewMode0104::ALL {
        let mut state = UserStoreUiState0104::default();
        let mut authority = UserStoreAuthority0104::default();
        let mut catalog = UserStoreItemCatalog0104::default();
        seed_user_store_preview_0104(mode, &mut state, &mut authority, &mut catalog);
        let projection = project_user_store_ui_0104(&state, &authority, &catalog);
        assert!(state.active);
        assert_eq!(state.opening_elapsed_seconds, 1.0);
        match mode {
            UserStorePreviewMode0104::MyReady => {
                assert_eq!(projection.listing_rows.len(), 5);
                assert!(projection.listing_rows.iter().all(|row| row.item.is_none()));
            }
            UserStorePreviewMode0104::MyItems => {
                assert_eq!(projection.primary_button, "OPEN STORE")
            }
            UserStorePreviewMode0104::MyOpen => {
                assert!(projection.show_go_to_game);
                assert_eq!(projection.primary_button, "CLOSE STORE");
            }
            UserStorePreviewMode0104::UserList => {
                assert_eq!(projection.listing_rows.len(), 3)
            }
            UserStorePreviewMode0104::UserSold => {
                assert!(projection.listing_rows.iter().any(|row| row.sold))
            }
            UserStorePreviewMode0104::PopupQuantity
            | UserStorePreviewMode0104::PopupPrice
            | UserStorePreviewMode0104::PopupUnregister
            | UserStorePreviewMode0104::PopupBuy => {
                let mut popup = UserStorePopupPresentation0104::default();
                seed_user_store_popup_preview_0104(mode, &mut state, &authority, &mut popup);
                assert!(state.modal.item_popup);
                assert!(popup.popup.is_some());
            }
            UserStorePreviewMode0104::Busy => assert!(projection.busy),
            UserStorePreviewMode0104::Error => assert_eq!(projection.error, Some(13)),
        }
    }
}
