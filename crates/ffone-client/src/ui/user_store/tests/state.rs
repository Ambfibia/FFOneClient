use super::*;

pub(super) fn ready_state() -> (
    UserStoreUiState0104,
    UserStoreAuthority0104,
    UserStoreUiOutbox0104,
) {
    let mut state = UserStoreUiState0104::default();
    let mut authority = UserStoreAuthority0104 {
        owner_pc_id: OWNER,
        maximum_list_slots: 5,
        open_item_inventory_slot: 0,
        ..default()
    };
    authority.inventory[0] = item(0, 10, 1);
    let mut outbox = UserStoreUiOutbox0104::default();
    let _ = state.begin_my_store(&mut authority, OWNER, 0, true, &mut outbox);
    let mut ready = fixed_reply_payload(STREETSTALL_REP_READY_SUCCESS);
    ready[12] = 7;
    state
        .receive_reply(
            &mut authority,
            STREETSTALL_REP_READY_SUCCESS,
            &ready,
            &mut outbox,
        )
        .expect("valid ready");
    outbox.clear();
    (state, authority, outbox)
}

#[test]
fn source_text_spawns_are_key_first_and_runtime_binding_never_writes_raw_text() {
    let source = concat!(
        include_str!("../constants.rs"),
        "\n",
        include_str!("../containers.rs"),
        "\n",
        include_str!("../state.rs"),
        "\n",
        include_str!("../assets.rs"),
        "\n",
        include_str!("../layout.rs"),
        "\n",
        include_str!("../interaction.rs"),
        "\n",
        include_str!("../commands.rs"),
        "\n",
        include_str!("../codec.rs"),
        "\n",
        include_str!("../types_user_store_authority0104.rs"),
        "\n",
        include_str!("../types_user_store_ui_state0104.rs"),
        "\n",
        include_str!("../types_user_store_popup_text_style0104.rs"),
        "\n",
        include_str!("../validation.rs"),
        "\n",
        include_str!("../input_read_i32.rs"),
        "\n",
        include_str!("../output.rs"),
        "\n",
        include_str!("../binding_user_store_ui.rs"),
        "\n",
        include_str!("../operations_user_store_missing_checker_image.rs"),
        "\n",
        include_str!("../systems.rs"),
        "\n",
        include_str!("../projection.rs"),
        "\n",
        include_str!("../view_user_store_popup.rs"),
        "\n",
        include_str!("../view_user_store_image.rs"),
        "\n",
        include_str!("../textures.rs"),
        "\n",
        include_str!("../mod.rs")
    );
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    let lines = production_source.lines().collect::<Vec<_>>();
    let text_spawns = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("Text::new("))
        .collect::<Vec<_>>();
    assert_eq!(text_spawns.len(), 5, "audit every authored Text spawn");
    for (index, line) in text_spawns {
        let attachment = lines[index..(index + 4).min(lines.len())].join("\n");
        assert!(
            attachment.contains("localized")
                || attachment.contains("LocalizedText::new")
                || attachment.contains("user_store_passthrough_text"),
            "Text spawn lacks a key-first LocalizedText near line {}: {line}",
            index + 1
        );
    }
    assert!(
        !production_source.contains("Query<&mut Text")
            && !production_source.contains("Query<(&mut Text")
            && !production_source.contains("Query<(&UserStoreUiTextRole0104, &mut Text"),
        "runtime bindings must update LocalizedText rather than raw Text"
    );
    assert!(!production_source.contains("UserStoreUiElement0104::GenericCalculator"));
}

#[test]
fn gum_popup_digits_and_quantity_to_price_bridge_match_clean_state_machine() {
    let mut state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::MyStore,
        opening_elapsed_seconds: USER_STORE_OPEN_SECONDS,
        ..default()
    };
    let mut authority = UserStoreAuthority0104 {
        maximum_list_slots: USER_STORE_LIST_CAPACITY,
        ..default()
    };
    authority.inventory[1] = item(USER_STORE_GENERAL_ITEM_TYPE, 75, 12);
    let mut outbox = UserStoreUiOutbox0104::default();
    let outcome = state.activate_inventory_row(
        &authority,
        1,
        UserStoreRowButton0104::Primary,
        &mut outbox,
    );
    let mut popup = UserStorePopupPresentation0104::default();
    adopt_user_store_popup_outcome_0104(&mut popup, &outcome);
    assert!(state.modal.item_popup);
    assert_eq!(popup.value, 0);
    popup.append_digit(1);
    popup.append_digit(5);
    assert_eq!(popup.value, 12, "clean CacuNums clamps to iMaxNums");
    popup.clear_value();
    popup.append_digit(3);
    assert_eq!(popup.value, 3);

    outbox.clear();
    let outcome =
        apply_user_store_popup_action_0104(&mut popup, &mut state, &authority, &mut outbox);
    assert!(matches!(outcome, UserStoreActionOutcome0104::Popup(_)));
    let price_popup = popup.popup.expect("quantity advances to price");
    assert_eq!(price_popup.kind, UserStorePopupKind0104::RegisterPrice);
    assert_eq!(price_popup.item.option, 3);
    assert_eq!(popup.value, 0);
    assert_eq!(popup.maximum_input(), USER_STORE_POPUP_MAX_PRICE);

    let zero =
        apply_user_store_popup_action_0104(&mut popup, &mut state, &authority, &mut outbox);
    assert_eq!(
        zero,
        UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidPrice)
    );
    assert!(
        popup.popup.is_some(),
        "invalid zero price leaves GumPopup open"
    );

    popup.append_digit(5);
    popup.append_digit(0);
    popup.append_digit(0);
    let sent =
        apply_user_store_popup_action_0104(&mut popup, &mut state, &authority, &mut outbox);
    assert!(matches!(sent, UserStoreActionOutcome0104::Sent(_)));
    assert!(popup.popup.is_none());
    assert!(!state.modal.item_popup);
    assert!(matches!(
        state.pending,
        Some(UserStorePendingRequest0104::Register { price: 500, .. })
    ));
}

#[test]
fn register_and_unregister_commit_authoritative_inventory_atomically() {
    let (mut state, mut authority, mut outbox) = ready_state();
    authority.inventory[3] = item(7, 42, 10);
    let registered = item(7, 42, 4);
    assert!(matches!(
        state.request_register(&authority, 3, registered, 1_500, &mut outbox),
        UserStoreActionOutcome0104::Sent(_)
    ));
    assert_eq!(authority.inventory[3].option, 10);
    let pending_before = state.pending;
    let mut reply = vec![0_u8; STREETSTALL_REGISTER_SUCCESS_SIZE];
    write_i32(&mut reply, 0, 0);
    write_i32(&mut reply, 4, 3);
    encode_item(registered, &mut reply[8..20]);
    write_i32(&mut reply, 20, 1_500);
    state
        .receive_reply(
            &mut authority,
            STREETSTALL_REP_REGISTER_ITEM_SUCCESS,
            &reply,
            &mut outbox,
        )
        .expect("register commit");
    assert_ne!(state.pending, pending_before);
    assert_eq!(state.pending, None);
    assert_eq!(authority.inventory[3].option, 6);
    assert_eq!(authority.listings[0].expect("listing").item, registered);

    assert!(matches!(
        state.request_unregister(&authority, 0, &mut outbox),
        UserStoreActionOutcome0104::Sent(_)
    ));
    assert_eq!(authority.inventory[3].option, 6);
    state
        .receive_reply(
            &mut authority,
            STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS,
            &0_i32.to_le_bytes(),
            &mut outbox,
        )
        .expect("unregister commit");
    assert_eq!(authority.inventory[3].option, 10);
    assert!(authority.listings[0].is_none());
}

#[test]
fn buy_requires_taros_and_authoritative_empty_inventory_slot() {
    let mut state = UserStoreUiState0104 {
        active: true,
        mode: UserStoreMode0104::UserStore,
        ..default()
    };
    let listing_item = item(0, 99, 1);
    let mut authority = UserStoreAuthority0104 {
        target_pc_id: Some(TARGET),
        taros: 99,
        store_open: true,
        ..default()
    };
    authority.listings[0] = Some(UserStoreListing0104 {
        list_slot: 0,
        inventory_slot: None,
        item: listing_item,
        price: 100,
    });
    let mut outbox = UserStoreUiOutbox0104::default();
    assert_eq!(
        state.request_buy(&authority, 0, &mut outbox),
        UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InsufficientTaros)
    );
    authority.taros = 100;
    authority.inventory.fill(item(7, 1, 1));
    assert_eq!(
        state.request_buy(&authority, 0, &mut outbox),
        UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InventoryFull)
    );
    authority.inventory[49] = EMPTY_ITEM_0104;
    assert!(matches!(
        state.request_buy(&authority, 0, &mut outbox),
        UserStoreActionOutcome0104::Sent(UserStorePacket0104 {
            packet_id: STREETSTALL_REQ_ITEM_BUY,
            ..
        })
    ));
    assert_eq!(
        read_i32(
            &outbox
                .0
                .back()
                .and_then(|command| match command {
                    UserStoreUiCommand0104::SendPacket(packet) => Some(packet),
                    _ => None,
                })
                .expect("send")
                .payload,
            8
        ),
        49
    );
}
