use super::*;

#[test]
fn clean_gum_popup_geometry_styles_and_generic_calculator_exclusion_are_exact() {
    let register = UserStoreItemPopup0104 {
        kind: UserStorePopupKind0104::RegisterQuantity,
        slot_type: 1,
        slot: 0,
        item: item(7, 75, 12),
        maximum_quantity: 12,
        price: 0,
    };
    let listing = UserStoreItemPopup0104 {
        slot_type: USER_STORE_MY_SLOT_TYPE,
        kind: UserStorePopupKind0104::Unregister,
        ..register
    };
    assert_eq!(
        user_store_popup_rect_0104(1_264, 681, register),
        UserStoreUiRect::new(732.0, 111.0, 310.0, 355.0)
    );
    assert_eq!(
        user_store_popup_rect_0104(1_264, 681, listing),
        UserStoreUiRect::new(322.0, 111.0, 310.0, 355.0)
    );
    assert_eq!(USER_STORE_POPUP_DEPTH, 2);
    assert_eq!(USER_STORE_PANEL_DEPTH, 10);
    assert_eq!(USER_STORE_POPUP_LOCAL_Z_INDEX, 8);
    assert_eq!(USER_STORE_POPUP_TYPE, 7);
    assert_eq!(USER_STORE_POPUP_BACKGROUND_PATH_ID, 390);
    assert_eq!(USER_STORE_POPUP_CALCULATOR_PATH_ID, 385);
    assert_eq!(
        USER_STORE_POPUP_CALCULATOR_RECT,
        UserStoreUiRect::new(87.0, 162.0, 136.0, 109.0)
    );
    assert_eq!(USER_STORE_POPUP_BUTTON_RECTS[0].1.width, 44.0);
    assert_eq!(USER_STORE_POPUP_BUTTON_RECTS[0].1.height, 19.0);
    assert_eq!(
        USER_STORE_POPUP_REGISTER_ACTION_RECT,
        UserStoreUiRect::new(79.0, 301.0, 150.0, 25.0)
    );
    assert_eq!(
        USER_STORE_POPUP_LISTING_ACTION_RECT,
        UserStoreUiRect::new(59.0, 301.0, 190.0, 25.0)
    );
    assert_eq!(
        USER_STORE_GENERIC_CALCULATOR_RECT,
        UserStoreUiRect::new(-100.0, -200.0, 115.0, 200.0),
        "generic eWindowType_Cacu evidence is retained as center-relative geometry only"
    );
}

#[test]
fn entry_geometry_scroll_and_dead_return_mode_are_retained() {
    assert_eq!(user_store_opening_eased_fraction(0.0), 0.0);
    assert!(
        (user_store_opening_eased_fraction(0.5) - std::f32::consts::FRAC_1_SQRT_2).abs()
            < 0.000_01
    );
    assert_eq!(user_store_opening_eased_fraction(1.0), 1.0);
    let start = user_store_layout_0104(1_020, 638, 0.0, 0.0);
    let end = user_store_layout_0104(1_020, 638, 1.0, 0.0);
    assert_eq!(start.store_panel.left, -498.0);
    assert_eq!(end.store_panel.left, 0.0);
    assert_eq!(
        end.store_panel,
        UserStoreUiRect::new(0.0, 0.0, 498.0, 638.0)
    );
    assert_eq!(end.list_viewport.left, 23.0);
    assert_eq!(end.list_viewport.top, 174.0);
    let mut state = UserStoreUiState0104::default();
    state.scroll_store_dead(100.0);
    assert_eq!(state.inert_store_scroll_y, -30.0);
    let before = user_store_layout_0104(1_020, 638, 1.0, state.inventory_scroll_y);
    state.scroll_store_dead(-100.0);
    let after = user_store_layout_0104(1_020, 638, 1.0, state.inventory_scroll_y);
    assert_eq!(before.list_viewport, after.list_viewport);
    for _ in 0..10 {
        state.scroll_inventory(-10_000.0);
    }
    assert_eq!(state.inventory_scroll_y, user_store_inventory_scroll_max());
    assert!(!UserStoreMode0104::ReturnStore.observed_setup());
    assert_eq!(UserStoreMode0104::ReturnStore.listing_slot_type(), None);
}

#[test]
fn exact_serialized_geometry_and_shared_asset_contract_are_stable() {
    assert_eq!(
        USER_STORE_DIALOG_RECT,
        UserStoreUiRect::new(11.0, 132.0, 486.0, 528.0)
    );
    assert_eq!(
        USER_STORE_LIST_BACK_RECT,
        UserStoreUiRect::new(0.0, 20.0, 486.0, 485.0)
    );
    assert_eq!(
        USER_STORE_TABLE_RECT,
        UserStoreUiRect::new(2.0, 32.0, 479.0, 410.0)
    );
    assert_eq!(
        USER_STORE_LIST_VIEWPORT_RECT,
        UserStoreUiRect::new(10.0, 10.0, 464.0, 400.0)
    );
    assert_eq!(
        USER_STORE_PRIMARY_BUTTON_RECT,
        UserStoreUiRect::new(313.0, 595.0, 161.0, 25.0)
    );
    assert_eq!(
        USER_STORE_GO_TO_GAME_RECT,
        UserStoreUiRect::new(30.0, 595.0, 161.0, 25.0)
    );
    assert_eq!(
        USER_STORE_ROW_ITEM_RECT,
        UserStoreUiRect::new(4.0, 5.0, 66.0, 66.0)
    );
    assert_eq!(USER_STORE_ROW_NAME_RECT.top, 8.0);
    assert_eq!(USER_STORE_ROW_LEVEL_RECT.top, 22.0);
    assert_eq!(USER_STORE_ROW_PRICE_RECT.top, 20.0);
    assert_eq!(USER_STORE_ROW_SELLER_NET_RECT.top, 30.0);
    assert_eq!(USER_STORE_ALL_ASSET_PATHS.len(), 24);
    assert_eq!(USER_STORE_IMAGE_ASSET_PATHS.len(), 22);
    assert_eq!(USER_STORE_SOURCE_ARCHIVE_SHA256.len(), 64);
    assert_eq!(USER_STORE_GAME_MODE_SLOT, 28);
    assert_eq!(USER_STORE_GAME_OBJECT_PATH_ID, 1_312);
}
