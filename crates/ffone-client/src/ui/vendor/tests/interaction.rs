use super::*;

#[test]
fn inventory_clicks_and_drop_targets_follow_clean_vendor_owners() {
    let projected = projection(
        &[],
        &[],
        &[
            (0, item(0, 1, 0, 0)),
            (1, item(7, 4, 10, 0)),
            (2, item(9, 1, 0, 0)),
            (3, item(4, 1, 0, 0)),
            (4, item(1, 1, 0, 0)),
        ],
    );

    let VendorActivationOutcome0104::Popup(equip_popup) =
        projected.primary_inventory_activation(0)
    else {
        panic!("ordinary inventory click must open EquipPopup contract");
    };
    assert_eq!(equip_popup.sell, Some(VendorQuantityContract0104::Fixed(1)));
    assert!(equip_popup.delete);
    assert!(matches!(
        projected.commit_item_popup(equip_popup, VendorPopupCommit0104::Sell { count: 1 }),
        VendorActionOutcome0104::Intent(VendorIntent0104::Sell(VendorSellIntent0104 {
            inventory_slot: 0,
            count: 1,
            ..
        }))
    ));
    assert!(matches!(
        projected.commit_item_popup(equip_popup, VendorPopupCommit0104::Delete),
        VendorActionOutcome0104::Confirmation(VendorConfirmation0104 {
            message_id: VendorSystemMessageId0104::ConfirmDelete,
            ..
        })
    ));

    let VendorActivationOutcome0104::Popup(gum_popup) =
        projected.primary_inventory_activation(1)
    else {
        panic!("general inventory click must open GumPopup contract");
    };
    assert_eq!(
        gum_popup.sell,
        Some(VendorQuantityContract0104::Calculator { maximum: 10 })
    );
    assert!(!gum_popup.delete);
    assert!(matches!(
        projected.commit_item_popup(gum_popup, VendorPopupCommit0104::Sell { count: 7 }),
        VendorActionOutcome0104::Intent(VendorIntent0104::Sell(VendorSellIntent0104 {
            inventory_slot: 1,
            count: 7,
            ..
        }))
    ));
    assert!(matches!(
        projected.secondary_inventory_activation(1),
        VendorActivationOutcome0104::Action(VendorActionOutcome0104::Intent(
            VendorIntent0104::Sell(VendorSellIntent0104 {
                inventory_slot: 1,
                count: 1,
                ..
            })
        ))
    ));

    let VendorActivationOutcome0104::Popup(chest_popup) =
        projected.primary_inventory_activation(2)
    else {
        panic!("chest inventory click must open TuringPopup contract");
    };
    assert!(chest_popup.open_chest);
    assert_eq!(chest_popup.sell, None);
    assert!(chest_popup.delete);
    assert_eq!(
        projected.secondary_inventory_activation(2),
        VendorActivationOutcome0104::Action(VendorActionOutcome0104::SystemMessage(
            VendorSystemMessage0104::plain(VendorSystemMessageId0104::CannotSellChest)
        ))
    );

    assert!(matches!(
        projected.inventory_drop_activation(1, VendorInventoryDropTarget0104::Trash),
        VendorActionOutcome0104::Confirmation(VendorConfirmation0104 {
            message_id: VendorSystemMessageId0104::ConfirmDelete,
            delete_count: Some(10),
            ..
        })
    ));
    assert!(matches!(
        projected.inventory_drop_activation(4, VendorInventoryDropTarget0104::Hammer),
        VendorActionOutcome0104::Confirmation(VendorConfirmation0104 {
            message_id: VendorSystemMessageId0104::ConfirmDisassemble,
            ..
        })
    ));
    assert_eq!(
        projected.inventory_drop_activation(3, VendorInventoryDropTarget0104::Hammer),
        VendorActionOutcome0104::SilentBlocked(VendorSilentBlock0104::DisassembleIneligible {
            item_type: 4
        })
    );

    let mut state = VendorUiState {
        phase: VendorLifecyclePhase::Visible,
        ..default()
    };
    let mut outbox = VendorUiOutbox0104::default();
    dispatch_vendor_activation(
        &mut state,
        VendorModalState::default(),
        projected.primary_inventory_activation(0),
        true,
        &mut outbox,
    );
    assert_eq!(
        outbox.pop_front(),
        Some(VendorUiCommand0104::OpenItemActionPopup(equip_popup))
    );
    state
        .dispatch_inventory_drop(
            VendorModalState::default(),
            &projected,
            4,
            VendorInventoryDropTarget0104::Hammer,
            &mut outbox,
        )
        .unwrap();
    assert!(matches!(
        outbox.pop_front(),
        Some(VendorUiCommand0104::OpenConfirmation(
            VendorConfirmation0104 {
                message_id: VendorSystemMessageId0104::ConfirmDisassemble,
                ..
            }
        ))
    ));
    assert_eq!(
        state.dispatch_inventory_drop(
            VendorModalState {
                inventory_popup: true,
                ..default()
            },
            &projected,
            4,
            VendorInventoryDropTarget0104::Hammer,
            &mut outbox,
        ),
        Err(VendorActionBlocked0104::ControlsDisabled)
    );
}

#[test]
fn lifecycle_tabs_scroll_and_close_help_gates_match_clean_behavior() {
    let mut state = VendorUiState::default();
    let mut outbox = VendorUiOutbox0104::default();
    state.begin_open(NPC_ID, TABLE_VENDOR_ID, &mut outbox);
    assert_eq!(
        outbox.pop_front(),
        Some(VendorUiCommand0104::StartSession {
            requested_npc_id: NPC_ID,
            table_vendor_id: TABLE_VENDOR_ID,
        })
    );
    state.accept_authoritative_table();
    state.tick(VENDOR_OPEN_SECONDS);
    assert_eq!(state.phase, VendorLifecyclePhase::Visible);
    state.vendor_scroll_y = 125.0;
    state
        .switch_tab(VendorModalState::default(), VendorTab0104::Buyback)
        .unwrap();
    assert_eq!(state.tab.legacy_mode(), 3);
    assert_eq!(state.vendor_scroll_y, 0.0);
    state
        .apply_scroll_axis(
            VendorModalState::default(),
            VendorScrollTarget::Vendor,
            -1.0,
            20,
        )
        .unwrap();
    assert_eq!(state.vendor_scroll_y, 30.0);

    state
        .request_help(VendorModalState::default(), &mut outbox)
        .unwrap();
    assert_eq!(
        outbox.pop_front(),
        Some(VendorUiCommand0104::OpenHelp {
            event_id: VENDOR_HELP_EVENT_ID
        })
    );
    state
        .request_redeem_code(VendorModalState::default(), &mut outbox)
        .unwrap();
    assert_eq!(
        outbox.pop_front(),
        Some(VendorUiCommand0104::OpenRedeemCode)
    );
    assert_eq!(
        state.request_redeem_code(
            VendorModalState {
                inventory_popup: true,
                ..default()
            },
            &mut outbox,
        ),
        Err(VendorActionBlocked0104::ControlsDisabled)
    );
    assert_eq!(
        state.request_close_button(
            VendorModalState::default(),
            VendorCloseGate0104 {
                mode_accepts_escape: true,
                target_action_idle: false,
            },
            &mut outbox,
        ),
        Err(VendorActionBlocked0104::CloseGateRejected)
    );
    assert_eq!(
        state.request_escape(
            VendorModalState {
                system_popup: true,
                ..default()
            },
            VendorCloseGate0104 {
                mode_accepts_escape: true,
                target_action_idle: true,
            },
            &mut outbox,
        ),
        Err(VendorActionBlocked0104::CloseGateRejected)
    );
    state
        .request_escape(
            VendorModalState::default(),
            VendorCloseGate0104 {
                mode_accepts_escape: true,
                target_action_idle: true,
            },
            &mut outbox,
        )
        .unwrap();
    assert_eq!(outbox.pop_front(), Some(VendorUiCommand0104::ExitMode));
    assert_eq!(state, VendorUiState::default());
}
