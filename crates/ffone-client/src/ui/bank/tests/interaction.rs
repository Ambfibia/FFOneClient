use super::*;

#[test]
fn locked_and_empty_sources_fail_closed_and_half_one_click_stays_below_60() {
    let mut bank = Vec::new();
    for slot in 0..59 {
        bank.push((slot, item(7, 100 + slot as i16, 1, 0)));
    }
    bank.push((60, item(7, 999, 1, 0)));
    let projection = projection(0, &bank, &[(0, item(7, 88, 3, 0))]);
    let inventory_source = BankSlotRef0104::new(BankSlotLocation0104::Inventory, 0).unwrap();
    let intent = projection.one_click_intent(inventory_source).unwrap();
    assert_eq!(intent.to.location(), BankSlotLocation0104::Bank);
    assert_eq!(intent.to.index(), 59);

    let locked = BankSlotRef0104::new(BankSlotLocation0104::Bank, 60).unwrap();
    assert_eq!(
        projection.one_click_intent(locked),
        Err(BankTransferError0104::LockedBankSlot { slot: locked })
    );
    let empty_source = BankSlotRef0104::new(BankSlotLocation0104::Inventory, 1).unwrap();
    assert_eq!(
        projection.one_click_intent(empty_source),
        Err(BankTransferError0104::EmptySource { slot: empty_source })
    );
}

#[test]
fn lifecycle_modal_scroll_and_outbox_match_clean_fail_closed_boundary() {
    let projection = projection(1, &[(0, item(7, 77, 2, 0))], &[]);
    let mut state = BankUiState::default();
    let mut outbox = BankUiOutbox0104::default();
    state.begin_open(OWNER_PC_ID, NPC_ID, &mut outbox);
    assert_eq!(
        outbox.pop_front(),
        Some(BankUiCommand0104::Open(PcBankOpenRequest0104 {
            pc_id: OWNER_PC_ID,
            npc_id: NPC_ID,
        }))
    );
    assert_eq!(state.phase, BankLifecyclePhase::Opening);
    assert!(state.send_pending);
    state.tick(0.5);
    assert!(
        (bank_opening_eased_fraction(state.opening_elapsed_seconds)
            - std::f32::consts::FRAC_1_SQRT_2)
            .abs()
            < 0.000_01
    );
    state.accept_open_success();
    state.tick(0.5);
    assert_eq!(state.phase, BankLifecyclePhase::Visible);
    assert!(
        state
            .input_capabilities(BankModalState::default())
            .item_move
    );

    state.apply_scroll_axis(BankScrollTarget::Bank, -1.0);
    assert_eq!(state.bank_scroll_y, 200.0);
    state.apply_scroll_axis(BankScrollTarget::Bank, f32::NEG_INFINITY);
    assert_eq!(state.bank_scroll_y, 200.0);

    let from = BankSlotRef0104::new(BankSlotLocation0104::Bank, 0).unwrap();
    let to = BankSlotRef0104::new(BankSlotLocation0104::Inventory, 0).unwrap();
    let before = projection.clone();
    state
        .request_transfer(
            BankModalState::default(),
            &projection,
            from,
            to,
            &mut outbox,
        )
        .unwrap();
    assert!(state.send_pending);
    assert_eq!(
        outbox.pop_front(),
        Some(BankUiCommand0104::ItemMove(ItemMoveRequest0104 {
            from_location: 3,
            from_slot_num: 0,
            to_location: 1,
            to_slot_num: 0,
        }))
    );
    assert_eq!(projection, before);

    state.accept_item_move_success();
    let modal = BankModalState {
        system_popup: true,
        ..default()
    };
    assert_eq!(
        state.request_close(modal, &mut outbox),
        Err(BankActionBlocked::ControlsDisabled)
    );
    state
        .request_close(BankModalState::default(), &mut outbox)
        .unwrap();
    assert_eq!(outbox.pop_front(), Some(BankUiCommand0104::ExitMode));
    assert_eq!(state, BankUiState::default());
    assert!(
        outbox.pop_front().is_none(),
        "clean BankOut emits no bank-close packet"
    );
}

pub(super) fn pointer_test_app() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.insert_resource(projection(1, &[(0, item(7, 77, 2, 0))], &[]))
        .init_resource::<BankLocalRequests>()
        .init_resource::<drag::DragVisual>()
        .insert_resource(BankUiState {
            phase: BankLifecyclePhase::Visible,
            ..default()
        })
        .init_resource::<BankModalState>()
        .init_resource::<BankUiOutbox0104>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<BankItemPopupState>()
        .add_systems(Update, collect_bank_slot_input);
    let source = app
        .world_mut()
        .spawn((
            BankSlotControl(BankSlotRef0104::new(BankSlotLocation0104::Bank, 0).unwrap()),
            Interaction::Hovered,
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            BankSlotControl(BankSlotRef0104::new(BankSlotLocation0104::Inventory, 4).unwrap()),
            Interaction::None,
        ))
        .id();
    (app, source, target)
}

pub(super) fn pointer_edge(app: &mut App, button: MouseButton, pressed: bool) {
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.clear();
    if pressed {
        mouse.press(button);
    } else {
        mouse.release(button);
    }
    app.update();
}

#[test]
fn bank_pointer_secondary_moves_once_without_mutating_authority() {
    let (mut app, _, _) = pointer_test_app();
    let before = app.world().resource::<BankModeProjection0104>().clone();
    pointer_edge(&mut app, MouseButton::Right, true);
    assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
    pointer_edge(&mut app, MouseButton::Right, false);
    assert_eq!(
        app.world_mut()
            .resource_mut::<BankUiOutbox0104>()
            .pop_front(),
        Some(BankUiCommand0104::ItemMove(ItemMoveRequest0104 {
            from_location: 3,
            from_slot_num: 0,
            to_location: 1,
            to_slot_num: 0,
        }))
    );
    pointer_edge(&mut app, MouseButton::Right, true);
    pointer_edge(&mut app, MouseButton::Right, false);
    assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
    assert_eq!(app.world().resource::<BankModeProjection0104>(), &before);
}

#[test]
fn bank_pointer_drag_uses_released_destination() {
    let (mut app, source, target) = pointer_test_app();
    pointer_edge(&mut app, MouseButton::Left, true);
    *app.world_mut().get_mut::<Interaction>(source).unwrap() = Interaction::None;
    *app.world_mut().get_mut::<Interaction>(target).unwrap() = Interaction::Hovered;
    pointer_edge(&mut app, MouseButton::Left, false);
    assert_eq!(
        app.world_mut()
            .resource_mut::<BankUiOutbox0104>()
            .pop_front(),
        Some(BankUiCommand0104::ItemMove(ItemMoveRequest0104 {
            from_location: 3,
            from_slot_num: 0,
            to_location: 1,
            to_slot_num: 4,
        }))
    );
}

#[test]
fn bank_pointer_cancels_on_modal_owner_item_or_focus_change() {
    for reason in 0..4 {
        let (mut app, _, _) = pointer_test_app();
        pointer_edge(&mut app, MouseButton::Right, true);
        match reason {
            0 => {
                app.world_mut()
                    .resource_mut::<BankModalState>()
                    .system_popup = true
            }
            1 => {
                app.world_mut()
                    .resource_mut::<BankModeProjection0104>()
                    .owner_pc_id += 1
            }
            2 => {
                app.world_mut()
                    .resource_mut::<BankModeProjection0104>()
                    .bank[0]
                    .item
                    .item_id += 1
            }
            _ => {
                app.world_mut().spawn((
                    Window {
                        focused: false,
                        ..default()
                    },
                    PrimaryWindow,
                ));
            }
        }
        pointer_edge(&mut app, MouseButton::Right, false);
        assert!(
            app.world().resource::<BankUiOutbox0104>().is_empty(),
            "reason {reason}"
        );
    }
}

#[test]
fn bank_help_opens_context_and_blocks_bank_input() {
    let (mut app, _, _) = pointer_test_app();
    app.init_resource::<crate::game_guide_ui::GameGuideUiModel>()
        .add_systems(
            Update,
            collect_bank_local_controls.after(collect_bank_slot_input),
        );
    let button = app
        .world_mut()
        .spawn((BankHelpControl, Interaction::Pressed))
        .id();
    app.update();
    let mut expected = crate::game_guide_ui::GameGuideUiModel::default();
    assert!(expected.open_first_use(3));
    assert_eq!(expected.selected_main_topic, 6);
    assert_eq!(expected.selected_sub_topic, 0);
    assert_eq!(
        app.world()
            .resource::<crate::game_guide_ui::GameGuideUiModel>(),
        &expected
    );
    assert!(app.world().resource::<BankModalState>().help);
    pointer_edge(&mut app, MouseButton::Right, true);
    pointer_edge(&mut app, MouseButton::Right, false);
    assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
    app.world_mut()
        .resource_mut::<crate::game_guide_ui::GameGuideUiModel>()
        .visible = false;
    app.update();
    assert!(!app.world().resource::<BankModalState>().help);
}
