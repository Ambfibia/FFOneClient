use super::*;

pub(super) const fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

pub(super) const fn empty() -> ItemBase0104 {
    item(0, 0, 0, 0)
}

pub(super) fn projection(
    vendor: &[VendorCatalogEntry0104],
    recent: &[VendorRecentBuyEntry0104],
    inventory: &[(usize, ItemBase0104)],
) -> VendorModeProjection0104 {
    VendorModeProjection0104::from_authoritative(
        OWNER_PC_ID,
        session(),
        500,
        25,
        40,
        "Computress",
        "The best gear in town.",
        vendor,
        recent,
        &runtime_with(inventory, &[(0, item(0, 1, 0, 0))]),
        &TestCatalog,
        &TestEligibility,
    )
    .unwrap()
}

#[test]
fn buy_routes_normal_general_and_battery_without_currency_or_equip_rejection() {
    let projected = projection(
        &[
            VendorCatalogEntry0104 {
                source_slot_id: 0,
                item: item(0, 1, 0, 77),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 1,
                item: item(7, 2, 0, 88),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 2,
                item: item(7, 3, 0, 99),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 3,
                item: item(7, 4, 0, 111),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 4,
                item: item(0, 6, 0, 0),
            },
        ],
        &[],
        &[],
    );
    let identity = session().request_identity();
    assert_eq!(
        projected.request_buy(0, 0),
        VendorActionOutcome0104::Intent(VendorIntent0104::Buy(VendorBuyIntent0104 {
            identity,
            item: item(0, 1, 1, 77),
            inventory_slot: 0,
        }))
    );
    assert_eq!(
        projected.request_buy(1, 4),
        VendorActionOutcome0104::Intent(VendorIntent0104::BuyGeneral(
            VendorBuyGeneralIntent0104 {
                identity,
                item: item(7, 2, 4, 88),
                inventory_slot: 0,
            }
        ))
    );
    assert_eq!(
        projected.request_buy(2, 6),
        VendorActionOutcome0104::Intent(VendorIntent0104::Battery(VendorBatteryIntent0104 {
            identity,
            item: item(7, 3, 6, 99),
            battery_recharge: 100,
        }))
    );
    assert!(matches!(
        projected.request_buy(3, 5),
        VendorActionOutcome0104::Intent(VendorIntent0104::Buy(VendorBuyIntent0104 {
            item: ItemBase0104 { option: 5, .. },
            ..
        }))
    ));
    assert_eq!(
        projected.catalog_rows[4].equip_validation,
        VendorEquipValidation0104::Rejected
    );
    assert!(matches!(
        projected.request_buy(4, 0),
        VendorActionOutcome0104::Intent(VendorIntent0104::Buy(_))
    ));
}

#[test]
fn restore_uses_recent_slot_plus_one_and_zeroes_time_limit() {
    let projected = projection(
        &[],
        &[VendorRecentBuyEntry0104 {
            source_slot_id: 7,
            item: item(7, 4, 3, 999),
        }],
        &[],
    );
    assert_eq!(
        projected.request_restore(0),
        VendorActionOutcome0104::Intent(VendorIntent0104::Restore(VendorRestoreIntent0104 {
            identity: session().request_identity(),
            restore_list_id: 8,
            item: item(7, 4, 3, 0),
            inventory_slot: 0,
        }))
    );
}

#[test]
fn primary_and_secondary_vendor_gestures_keep_popup_and_quick_paths_distinct() {
    let projected = projection(
        &[
            VendorCatalogEntry0104 {
                source_slot_id: 0,
                item: item(0, 1, 0, 77),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 1,
                item: item(7, 2, 0, 88),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 2,
                item: item(7, 3, 0, 99),
            },
        ],
        &[VendorRecentBuyEntry0104 {
            source_slot_id: 7,
            item: item(0, 1, 0, 999),
        }],
        &[],
    );

    let VendorActivationOutcome0104::Popup(normal_popup) =
        projected.primary_row_activation(VendorTab0104::Buy, 0)
    else {
        panic!("ordinary catalog click must open EquipPopup contract");
    };
    assert_eq!(
        normal_popup,
        VendorItemActionPopup0104 {
            source: VendorItemPopupSource0104::CatalogRow { row_index: 0 },
            buy: Some(VendorQuantityContract0104::Fixed(0)),
            buyback: false,
            sell: None,
            delete: false,
            open_chest: false,
        }
    );
    assert!(matches!(
        projected.commit_item_popup(
            normal_popup,
            VendorPopupCommit0104::Buy { selected_option: 0 }
        ),
        VendorActionOutcome0104::Intent(VendorIntent0104::Buy(VendorBuyIntent0104 {
            item: ItemBase0104 { option: 1, .. },
            ..
        }))
    ));
    assert_eq!(
        projected.commit_item_popup(
            normal_popup,
            VendorPopupCommit0104::Buy { selected_option: 1 }
        ),
        VendorActionOutcome0104::SilentBlocked(VendorSilentBlock0104::PopupActionUnavailable)
    );
    assert!(matches!(
        projected.secondary_row_activation(VendorTab0104::Buy, 0),
        VendorActivationOutcome0104::Action(VendorActionOutcome0104::Intent(
            VendorIntent0104::Buy(_)
        ))
    ));

    let VendorActivationOutcome0104::Popup(general_popup) =
        projected.primary_row_activation(VendorTab0104::Buy, 1)
    else {
        panic!("general catalog click must open GumPopup contract");
    };
    assert_eq!(
        general_popup.buy,
        Some(VendorQuantityContract0104::Calculator { maximum: 4 })
    );
    assert_eq!(
        projected.secondary_row_activation(VendorTab0104::Buy, 1),
        VendorActivationOutcome0104::Popup(general_popup)
    );
    assert!(matches!(
        projected.commit_item_popup(
            general_popup,
            VendorPopupCommit0104::Buy { selected_option: 4 }
        ),
        VendorActionOutcome0104::Intent(VendorIntent0104::BuyGeneral(
            VendorBuyGeneralIntent0104 {
                item: ItemBase0104 { option: 4, .. },
                ..
            }
        ))
    ));

    let VendorActivationOutcome0104::Popup(battery_popup) =
        projected.primary_row_activation(VendorTab0104::Buy, 2)
    else {
        panic!("battery catalog click must open GumPopup contract");
    };
    assert_eq!(
        battery_popup.buy,
        Some(VendorQuantityContract0104::Calculator { maximum: -5 })
    );
    assert!(
        !battery_popup.buy.unwrap().accepts(1),
        "a negative clean maximum must reject every positive buy count"
    );

    let VendorActivationOutcome0104::Popup(buyback_popup) =
        projected.primary_row_activation(VendorTab0104::Buyback, 0)
    else {
        panic!("ordinary buyback click must open EquipPopup contract");
    };
    assert!(buyback_popup.buyback);
    assert!(matches!(
        projected.commit_item_popup(buyback_popup, VendorPopupCommit0104::Buyback),
        VendorActionOutcome0104::Intent(VendorIntent0104::Restore(VendorRestoreIntent0104 {
            restore_list_id: 8,
            ..
        }))
    ));
    assert!(matches!(
        projected.secondary_row_activation(VendorTab0104::Buyback, 0),
        VendorActivationOutcome0104::Action(VendorActionOutcome0104::Intent(
            VendorIntent0104::Restore(VendorRestoreIntent0104 {
                restore_list_id: 8,
                ..
            })
        ))
    ));
}

#[test]
fn delete_and_disassemble_confirm_before_emitting_typed_intents() {
    let projected = projection(
        &[],
        &[],
        &[
            (0, item(7, 4, 12, 0)),
            (1, item(1, 1, 0, 0)),
            (2, item(4, 1, 0, 0)),
        ],
    );
    let VendorActionOutcome0104::Confirmation(delete) = projected.request_delete(0) else {
        panic!("general delete must ask for confirmation");
    };
    assert_eq!(delete.message_id, VendorSystemMessageId0104::ConfirmDelete);
    assert_eq!(
        delete.callback,
        VendorConfirmationCallback0104::DeleteItemOk
    );
    assert_eq!(delete.delete_count, Some(12));
    assert_eq!(
        delete.confirm(),
        VendorIntent0104::Delete(VendorDeleteIntent0104 {
            location: VendorInventoryLocation0104::Inventory,
            inventory_slot: 0,
            item: item(7, 4, 12, 0),
        })
    );

    let VendorActionOutcome0104::Confirmation(hammer) = projected.request_disassemble(1) else {
        panic!("upper-body item must ask for hammer confirmation");
    };
    assert_eq!(
        hammer.message_id,
        VendorSystemMessageId0104::ConfirmDisassemble
    );
    assert_eq!(
        hammer.callback,
        VendorConfirmationCallback0104::HammerItemOk
    );
    assert!(matches!(
        hammer.confirm(),
        VendorIntent0104::Disassemble(VendorDisassembleIntent0104 {
            inventory_slot: 1,
            ..
        })
    ));
    assert_eq!(
        projected.request_disassemble(2),
        VendorActionOutcome0104::SilentBlocked(VendorSilentBlock0104::DisassembleIneligible {
            item_type: 4
        })
    );
}

#[test]
fn server_failures_map_to_clean_system_message_ids_and_exit_callbacks() {
    let cases = [
        (
            VendorServerFailure0104::Start,
            VendorSystemMessageId0104::StartFailed,
            Some(VendorSystemMessageCallback0104::Exit),
        ),
        (
            VendorServerFailure0104::TableUpdate,
            VendorSystemMessageId0104::TableUpdateFailed,
            Some(VendorSystemMessageCallback0104::Exit),
        ),
        (
            VendorServerFailure0104::Buy { error_code: 1 },
            VendorSystemMessageId0104::PurchaseRestricted,
            None,
        ),
        (
            VendorServerFailure0104::Buy { error_code: 99 },
            VendorSystemMessageId0104::BuyFailed,
            None,
        ),
        (
            VendorServerFailure0104::Battery,
            VendorSystemMessageId0104::BatteryFailed,
            None,
        ),
        (
            VendorServerFailure0104::Sell,
            VendorSystemMessageId0104::SellFailed,
            None,
        ),
        (
            VendorServerFailure0104::Restore { error_code: 7 },
            VendorSystemMessageId0104::BuyFailed,
            None,
        ),
    ];
    for (failure, expected_id, expected_callback) in cases {
        let VendorActionOutcome0104::SystemMessage(message) =
            vendor_server_failure_outcome(failure)
        else {
            panic!("failure must map to system message");
        };
        assert_eq!(message.message_id, expected_id);
        assert_eq!(message.callback, expected_callback);
    }
}

#[test]
fn hidden_missing_assets_and_unsupported_npc_preview_fail_closed() {
    let projected = VendorModeProjection0104::default();
    assert!(
        vendor_mode_view(
            1_264,
            681,
            VendorUiState::default(),
            VendorModalState::default(),
            &projected,
            true,
        )
        .is_none()
    );
    let state = VendorUiState {
        phase: VendorLifecyclePhase::Visible,
        opening_elapsed_seconds: VENDOR_OPEN_SECONDS,
        ..default()
    };
    assert!(
        vendor_mode_view(
            1_264,
            681,
            state,
            VendorModalState::default(),
            &projected,
            false,
        )
        .is_none()
    );
    let view = vendor_mode_view(
        1_264,
        681,
        state,
        VendorModalState::default(),
        &projected,
        true,
    )
    .unwrap();
    // Live portrait ownership is independent from the pure item projection.
    assert_eq!(
        view.layout.npc_preview_boundary,
        VendorUiRect::new(432.0, 21.0, 200.0, 150.0)
    );
}

#[test]
fn vendor_tree_localizes_every_text_and_preserves_source_style_metrics() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(VendorUiPlugin);
    app.update();

    let world = app.world_mut();
    let assets = world.resource::<VendorUiAssets>().clone();
    let mut every_text = world.query::<(&Text, &LocalizedText, &VendorUiTextStyle)>();
    let every_text = every_text.iter(world).collect::<Vec<_>>();
    assert_eq!(every_text.len(), 330);
    assert!(
        every_text
            .iter()
            .all(|(_, localized, _)| !localized.key.is_empty())
    );

    let mut styled = world.query::<(
        &VendorUiElement,
        &ChildOf,
        (&TextFont, &LineHeight),
        &LocalizedText,
    )>();
    let styled = styled
        .iter(world)
        .map(|(element, parent, font, localized)| {
            (
                element,
                world.get::<Node>(parent.parent()).unwrap(),
                font,
                localized,
            )
        })
        .collect::<Vec<_>>();
    let (_, service_node, service_font, service_text) = styled
        .iter()
        .find(|(element, ..)| **element == VendorUiElement::VendorService)
        .copied()
        .unwrap();
    assert_eq!(
        service_font.0.font,
        bevy::text::FontSource::Handle(assets.service_font.clone())
    );
    assert_eq!(
        service_font.0.font_size.eval(Vec2::ZERO, 16.0),
        VENDOR_SERVICE_FONT_SIZE
    );
    assert_eq!(
        (*service_font.1),
        LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT)
    );
    assert_eq!(service_node.align_items, AlignItems::FlexStart);
    assert_eq!(service_node.overflow, Overflow::clip());
    assert_eq!(service_text.key, "ui.vendor.service");
    assert_eq!(
        service_text
            .args
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["service"]
    );

    let (_, price_node, price_font, price_text) = styled
        .iter()
        .find(|(element, ..)| **element == VendorUiElement::RowPrice(0))
        .copied()
        .unwrap();
    assert_eq!(price_node.justify_content, JustifyContent::FlexEnd);
    assert_eq!(price_node.align_items, AlignItems::Center);
    assert_eq!(
        (*price_font.1),
        LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT)
    );
    assert_eq!(price_text.key, "ui.vendor.item.price");

    let (_, slot_node, slot_font, slot_text) = styled
        .iter()
        .find(|(element, ..)| **element == VendorUiElement::EquipmentSlotLabel(0))
        .copied()
        .unwrap();
    assert_eq!(slot_node.justify_content, JustifyContent::FlexEnd);
    assert_eq!(slot_node.align_items, AlignItems::Center);
    assert_eq!(
        (*slot_font.1),
        LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT)
    );
    assert_eq!(slot_text.key, "ui.inventory.slot.head");
}

#[test]
fn published_vendor_assets_are_present_as_native_assets() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let asset_root = workspace_root.join("assets/game");
    for path in VENDOR_UI_DEFAULT_IMAGE_PATHS {
        assert!(asset_root.join(path).is_file(), "missing {path}");
    }
    assert!(asset_root.join(USER_EQUIP_FONT_PATH).is_file());
    assert!(asset_root.join(VENDOR_SERVICE_FONT_PATH).is_file());
}
