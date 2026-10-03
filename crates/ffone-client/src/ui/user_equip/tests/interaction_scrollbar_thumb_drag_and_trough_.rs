use super::*;

#[test]
fn scrollbar_thumb_drag_and_trough_page_emit_clamped_scroll_requests() {
    let metrics = UserEquipScrollbarMetrics::for_mode(UserEquipMode::Item);
    let mut state = UserEquipUiState::default();
    state.open_item_mode();
    state.tick(USER_EQUIP_OPEN_SECONDS);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(state)
        .init_resource::<UserEquipModalState>()
        .init_resource::<UserEquipUiOutbox>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_systems(Update, collect_user_equip_scrollbar_pointer);
    let track_top = 100.0_f32;
    let track_height = USER_EQUIP_SCROLL_TRACK_RECT.height;
    let mut window = Window::default();
    window.focused = true;
    let window = app.world_mut().spawn((window, PrimaryWindow)).id();
    let set_cursor = |app: &mut App, y: f32| {
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_physical_cursor_position(Some(bevy::math::DVec2::new(363.0, f64::from(y))));
    };
    let node = |height: f32| {
        let mut node = ComputedNode::default();
        node.size = Vec2::new(17.0, height);
        node.inverse_scale_factor = 1.0;
        node
    };
    let track = app
        .world_mut()
        .spawn((
            UserEquipScrollTrackControl,
            Interaction::None,
            node(track_height),
            UiGlobalTransform::from(bevy::math::Affine2::from_translation(Vec2::new(
                363.0,
                track_top + track_height * 0.5,
            ))),
        ))
        .id();
    let thumb = app
        .world_mut()
        .spawn((
            UserEquipScrollThumbControl,
            Interaction::Pressed,
            node(metrics.thumb_height),
            UiGlobalTransform::default(),
        ))
        .id();
    // Production drains the outbox through the reducer every frame.
    let drain = |app: &mut App| {
        let requests = app
            .world_mut()
            .resource_mut::<UserEquipUiOutbox>()
            .drain()
            .collect::<Vec<_>>();
        let mut state = *app.world().resource::<UserEquipUiState>();
        let mut modal = *app.world().resource::<UserEquipModalState>();
        for request in &requests {
            apply_user_equip_ui_action(&mut state, &mut modal, *request);
        }
        app.world_mut().insert_resource(state);
        requests
    };

    // Grab the thumb 5 px below its top edge: the press alone is inert.
    set_cursor(&mut app, track_top + 5.0);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(drain(&mut app).is_empty());
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    set_cursor(&mut app, track_top + 5.0 + metrics.travel * 0.5);
    app.update();
    let requests = drain(&mut app);
    let [UserEquipUiAction::SetScrollY { scroll_y }] = requests[..] else {
        panic!("expected one scroll request, got {requests:?}");
    };
    assert!((scroll_y - metrics.scroll_max * 0.5).abs() < 1e-2);
    // Leaving the window keeps the capture; the drag then resumes from its
    // original grab and clamps below the end of the track.
    set_cursor(&mut app, 10_000.0);
    app.update();
    assert!(drain(&mut app).is_empty());
    set_cursor(&mut app, track_top + track_height + 50.0);
    app.update();
    assert_eq!(
        drain(&mut app),
        vec![UserEquipUiAction::SetScrollY {
            scroll_y: metrics.scroll_max
        }]
    );

    // Releasing ends the drag; a trough press above the thumb pages up.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    assert!(drain(&mut app).is_empty());
    *app.world_mut().get_mut::<Interaction>(thumb).unwrap() = Interaction::None;
    *app.world_mut().get_mut::<Interaction>(track).unwrap() = Interaction::Pressed;
    set_cursor(&mut app, track_top + 20.0);
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.clear();
    buttons.press(MouseButton::Left);
    app.update();
    assert_eq!(
        drain(&mut app),
        vec![UserEquipUiAction::SetScrollY {
            scroll_y: (metrics.scroll_max - USER_EQUIP_INVENTORY_VIEWPORT_RECT.height * 0.9)
                .max(0.0)
        }]
    );

    let mut state = *app.world().resource::<UserEquipUiState>();
    let mut modal = UserEquipModalState::default();
    assert_eq!(
        apply_user_equip_ui_action(
            &mut state,
            &mut modal,
            UserEquipUiAction::SetScrollY { scroll_y: 1.0e6 }
        ),
        UserEquipUiActionOutcome::Scrolled {
            previous: 0.0,
            current: metrics.scroll_max
        }
    );
    modal.help_active = true;
    assert_eq!(
        apply_user_equip_ui_action(
            &mut state,
            &mut modal,
            UserEquipUiAction::SetScrollY { scroll_y: 0.0 }
        ),
        UserEquipUiActionOutcome::ScrollBlocked
    );
    assert_eq!(state.scroll_y(), metrics.scroll_max);
}

#[test]
fn nano_tab_switch_clamps_scroll_and_emits_no_inventory_request() {
    assert_close(user_equip_bevy_wheel_to_legacy_axis(1.0), 0.1);
    assert_close(user_equip_bevy_wheel_to_legacy_axis(-1.0), -0.1);
    assert_close(user_equip_bevy_wheel_to_legacy_axis(f32::NAN), 0.0);
    let mut state = UserEquipUiState::default();
    let mut modal = UserEquipModalState::default();
    state.open_item_mode();
    state.tick(USER_EQUIP_OPEN_SECONDS);
    state.set_scroll_y(190.0);
    assert_eq!(
        apply_user_equip_ui_action(
            &mut state,
            &mut modal,
            UserEquipUiAction::SelectTab {
                mode: UserEquipMode::Nano,
            },
        ),
        UserEquipUiActionOutcome::TabSelected(UserEquipMode::Nano)
    );
    assert_eq!(state.mode(), UserEquipMode::Nano);
    assert_close(state.scroll_y(), 0.0);
    state.apply_legacy_scroll_axis(user_equip_bevy_wheel_to_legacy_axis(-1.0));
    assert_close(state.scroll_y(), 20.0);
    state.set_scroll_y(0.0);
    state.apply_legacy_scroll_axis(-1.0);
    assert_close(state.scroll_y(), 200.0);
    state.apply_legacy_scroll_axis(-1.0);
    assert_close(state.scroll_y(), 400.0);
    state.apply_legacy_scroll_axis(-1.0);
    assert_close(state.scroll_y(), user_equip_nano_scroll_max());
    assert_eq!(
        apply_user_equip_ui_action(
            &mut state,
            &mut modal,
            UserEquipUiAction::SelectTab {
                mode: UserEquipMode::Item,
            },
        ),
        UserEquipUiActionOutcome::TabSelected(UserEquipMode::Item)
    );
}

#[test]
fn clean_drag_controller_covers_swap_equip_and_first_empty_unequip() {
    let runtime = runtime_with(
        &[(4, item(4, 91, 0, 0))],
        &[(0, item(4, 77, 0, 0)), (1, item(7, 40, 2, 0))],
    );
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    let bag0 = UserEquipSlotEndpoint::Inventory { slot_index: 0 };
    let bag1 = UserEquipSlotEndpoint::Inventory { slot_index: 1 };
    let head = UserEquipSlotEndpoint::Equipment {
        visual_index: 0,
        wire_slot_index: 4,
    };

    assert_eq!(
        user_equip_drag_move(bag0, bag1, &projection),
        Some(UserEquipUiAction::MoveItem {
            from: bag0,
            to: bag1,
        }),
        "bag-to-bag preserves clean swap/reorder endpoints"
    );
    assert_eq!(
        user_equip_drag_move(bag0, head, &projection),
        Some(UserEquipUiAction::MoveItem {
            from: bag0,
            to: head,
        })
    );
    assert_eq!(
        user_equip_drag_move(
            head,
            UserEquipSlotEndpoint::Inventory { slot_index: 49 },
            &projection
        ),
        Some(UserEquipUiAction::MoveItem {
            from: head,
            to: UserEquipSlotEndpoint::Inventory { slot_index: 2 },
        }),
        "equipped items ignore the hovered bag cell and use the first empty slot"
    );
    assert_eq!(user_equip_drag_move(bag0, bag0, &projection), None);
    assert_eq!(
        user_equip_trash_drop(bag1, &projection),
        Some(UserEquipUiAction::DeleteInventoryItem { slot_index: 1 }),
        "clean PCStuff trash is an inventory-only drop target"
    );
    assert_eq!(user_equip_trash_drop(head, &projection), None);
    assert_eq!(
        user_equip_trash_drop(
            UserEquipSlotEndpoint::Inventory { slot_index: 2 },
            &projection,
        ),
        None
    );
    assert_eq!(
        user_equip_drag_move(
            UserEquipSlotEndpoint::Inventory { slot_index: 1 },
            head,
            &projection,
        ),
        None,
        "general items cannot be dropped into the head slot"
    );

    let equipment_swap_runtime =
        runtime_with(&[(1, item(1, 40, 0, 0)), (4, item(4, 91, 0, 0))], &[]);
    let equipment_swap =
        UserEquipItemModeProjection::from_authoritative(&equipment_swap_runtime, &AllCatalog);
    let shirt = UserEquipSlotEndpoint::Equipment {
        visual_index: 3,
        wire_slot_index: 1,
    };
    assert_eq!(
        user_equip_drag_move(head, shirt, &equipment_swap),
        None,
        "equipment-to-equipment swap must keep both resulting wire slots compatible"
    );

    let full_inventory = (0..INVENTORY_SLOT_COUNT_0104)
        .map(|slot| (slot, item(7, 100 + slot as i16, 1, 0)))
        .collect::<Vec<_>>();
    let full_runtime = runtime_with(&[(4, item(4, 91, 0, 0))], &full_inventory);
    let full_projection =
        UserEquipItemModeProjection::from_authoritative(&full_runtime, &AllCatalog);
    assert_eq!(
        user_equip_drag_move(
            head,
            UserEquipSlotEndpoint::Inventory { slot_index: 12 },
            &full_projection,
        ),
        None,
        "clean unequip fails closed when all 50 bag cells are occupied"
    );
}

#[test]
fn item_popup_and_one_click_emit_requests_without_projection_mutation() {
    let runtime = runtime_with(
        &[(4, item(4, 91, 0, 0))],
        &[(0, item(0, 77, 0, 0)), (1, item(9, 40, 1, 0))],
    );
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    let before = projection.clone();
    let weapon = UserEquipSlotEndpoint::Inventory { slot_index: 0 };
    let chest = UserEquipSlotEndpoint::Inventory { slot_index: 1 };
    let equipped_head = UserEquipSlotEndpoint::Equipment {
        visual_index: 0,
        wire_slot_index: 4,
    };
    let mut popup = UserEquipItemPopupState::default();
    popup.open(weapon);
    assert_eq!(
        popup.commands(&projection, None),
        vec![
            UserEquipPopupCommand::EquipPrimary,
            UserEquipPopupCommand::EquipSecondary,
        ]
    );
    popup.open(equipped_head);
    assert_eq!(
        popup.commands(&projection, None),
        vec![UserEquipPopupCommand::Unequip,]
    );

    let mut outbox = UserEquipUiOutbox::default();
    execute_user_equip_one_click(weapon, &projection, &mut outbox);
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![UserEquipUiAction::MoveItem {
            from: weapon,
            to: UserEquipSlotEndpoint::Equipment {
                visual_index: 6,
                wire_slot_index: 0,
            },
        }]
    );
    execute_user_equip_one_click(
        UserEquipSlotEndpoint::Inventory { slot_index: 1 },
        &projection,
        &mut outbox,
    );
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![UserEquipUiAction::OpenInventoryChest { slot_index: 1 }]
    );

    // `OneClickItem` on an equipment cell unequips into the first empty
    // authoritative bag slot instead of doing nothing.
    execute_user_equip_one_click(equipped_head, &projection, &mut outbox);
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![UserEquipUiAction::MoveItem {
            from: equipped_head,
            to: UserEquipSlotEndpoint::Inventory { slot_index: 2 },
        }]
    );

    let mut modal = UserEquipModalState {
        item_popup_active: true,
        ..default()
    };
    popup.open(chest);
    execute_user_equip_popup_command(
        UserEquipPopupCommand::Open,
        &projection,
        &mut popup,
        &mut modal,
        &mut outbox,
    );
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![UserEquipUiAction::OpenInventoryChest { slot_index: 1 }]
    );
    assert_eq!(popup.selected(), None);
    assert!(!modal.item_popup_active);
    assert_eq!(projection, before);
}

#[test]
fn typed_close_scroll_and_popup_reducer_never_mutate_inventory_authority() {
    let runtime = runtime_with(&[], &[(0, item(7, 40, 9, 0))]);
    let inventory_before = *runtime.inventory();
    let equipment_before = *runtime.equipment();
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    let projection_before = projection.clone();
    let mut state = UserEquipUiState::default();
    let mut modal = UserEquipModalState::default();

    state.open_from(UserEquipOpenSource::NanocomMyStuff);
    state.tick(USER_EQUIP_OPEN_SECONDS);
    state.set_scroll_y(user_equip_scroll_max());
    assert_eq!(
        apply_user_equip_ui_action(
            &mut state,
            &mut modal,
            UserEquipUiAction::ApplyLegacyScrollAxis { axis: 1.0 },
        ),
        UserEquipUiActionOutcome::Scrolled {
            previous: user_equip_scroll_max(),
            current: 0.0,
        }
    );

    modal.item_popup_active = true;
    assert_eq!(
        apply_user_equip_ui_action(
            &mut state,
            &mut modal,
            UserEquipUiAction::RequestClose {
                source: UserEquipCloseSource::Escape,
            },
        ),
        UserEquipUiActionOutcome::DismissedPopups
    );
    assert!(!modal.item_popup_active);
    assert!(state.is_active());
    assert_eq!(
        apply_user_equip_ui_action(
            &mut state,
            &mut modal,
            UserEquipUiAction::RequestClose {
                source: UserEquipCloseSource::CloseButton,
            },
        ),
        UserEquipUiActionOutcome::Closed
    );
    assert!(!state.is_active());
    assert_eq!(*runtime.inventory(), inventory_before);
    assert_eq!(*runtime.equipment(), equipment_before);
    assert_eq!(projection, projection_before);
}

#[test]
fn slide_scroll_lifecycle_and_modal_gates_are_deterministic() {
    let mut state = UserEquipUiState::default();
    assert_eq!(state.phase(), UserEquipLifecyclePhase::Hidden);
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, UserEquipModalState::default()),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::NotVisible)
    );

    state.open_item_mode();
    assert_eq!(state.phase(), UserEquipLifecyclePhase::Opening);
    let start = state.layout(1_280, 720);
    assert_eq!(start.user_clothes_panel.left, -475.0);
    assert_eq!(start.pc_stuff_panel.left, 1_020.0);
    assert_eq!(start.equipment_panel.left, 1_020.0);
    assert!(
        !state
            .input_capabilities(UserEquipModalState::default())
            .panel_controls
    );
    assert_eq!(
        state.close_disposition(
            UserEquipCloseSource::CloseButton,
            UserEquipModalState::default()
        ),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::OpeningAnimation)
    );
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, UserEquipModalState::default()),
        UserEquipCloseDisposition::ExitMode
    );

    state.tick(0.5);
    let halfway = state.layout(1_020, 638);
    assert_close(
        halfway.opening_eased_fraction,
        std::f32::consts::FRAC_1_SQRT_2,
    );
    assert_eq!(halfway.user_clothes_panel.left, -139.0);
    assert_eq!(halfway.pc_stuff_panel.left, 712.0);
    assert_eq!(halfway.equipment_panel.left, 655.0);
    state.tick(0.5);
    assert_eq!(state.phase(), UserEquipLifecyclePhase::Visible);
    assert_close(state.opening_elapsed_seconds(), 1.0);

    let enabled = state.input_capabilities(UserEquipModalState::default());
    assert!(enabled.panel_controls);
    assert!(enabled.slot_pointer);
    assert!(enabled.scroll);
    assert!(enabled.close_button);

    state.set_scroll_y(10_000.0);
    assert_close(state.scroll_y(), 190.0);
    state.apply_legacy_scroll_axis(1.0);
    assert_close(state.scroll_y(), 0.0);
    state.apply_legacy_scroll_axis(-99.0);
    assert_close(state.scroll_y(), 190.0);
    state.set_scroll_y(f32::NAN);
    assert_close(state.scroll_y(), 0.0);

    let pending = UserEquipModalState {
        send_pending: true,
        ..UserEquipModalState::default()
    };
    assert!(!state.input_capabilities(pending).panel_controls);
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, pending),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::SendPending)
    );

    let popup = UserEquipModalState {
        item_popup_active: true,
        ..UserEquipModalState::default()
    };
    assert!(!state.input_capabilities(popup).panel_controls);
    assert!(state.input_capabilities(popup).keyboard_close_request);
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, popup),
        UserEquipCloseDisposition::DismissPopups
    );

    let external = UserEquipModalState {
        external_exit_blocked: true,
        ..UserEquipModalState::default()
    };
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, external),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::ExternalExitGate)
    );

    let system = UserEquipModalState {
        system_popup_active: true,
        ..UserEquipModalState::default()
    };
    assert!(!state.input_capabilities(system).keyboard_close_request);
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, system),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::SystemPopup)
    );

    let help = UserEquipModalState {
        help_active: true,
        ..UserEquipModalState::default()
    };
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, help),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::HelpActive)
    );

    let redeem = UserEquipModalState {
        redeem_code_view: true,
        ..UserEquipModalState::default()
    };
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, redeem),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::RedeemCodeView)
    );

    state.close();
    assert_eq!(state.phase(), UserEquipLifecyclePhase::Hidden);
    assert_close(state.scroll_y(), 0.0);
}
