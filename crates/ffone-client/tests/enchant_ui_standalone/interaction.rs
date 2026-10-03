use super::*;

#[test]
fn clean_shared_inventory_opening_ease_and_scroll_clamp_are_preserved() {
    assert!(!ENCHANT_KEYBOARD_CLOSE_REACHABLE_0104);
    assert!(!ENCHANT_CONFIGURABLE_SCROLL_REACHABLE_0104);
    assert!(!ENCHANT_EQUIPMENT_STRIP_INTERACTIVE_0104);
    assert_eq!(
        ENCHANT_EQUIPMENT_LABELS_0104,
        [
            "HEAD", "FACE", "BACK", "CHEST", "LEGS", "FEET", "WEAPON 1", "WEAPON 2", "VEHICLE",
        ]
    );
    assert_eq!(
        ENCHANT_EQUIPMENT_WIRE_ORDER_0104,
        [4, 5, 6, 1, 2, 3, 0, 7, 8]
    );
    assert_eq!(enchant_counter_digit_0104(123_456_789, 0), Some(1));
    assert_eq!(enchant_counter_digit_0104(123_456_789, 8), Some(9));
    assert_eq!(enchant_counter_digit_0104(42, 7), Some(4));
    assert_eq!(enchant_counter_digit_0104(42, 8), Some(2));
    assert_eq!(enchant_counter_digit_0104(42, 9), None);
    let closed = enchant_mode_animated_layout_0104(1_280, 720, 0.0, 0.0);
    assert_eq!(closed.inventory_opening_eased_fraction, 0.0);
    assert_eq!(closed.pc_stuff_panel.left, 1_020.0);
    assert_eq!(closed.equipment_panel.left, 1_020.0);

    let halfway = enchant_mode_animated_layout_0104(1_280, 720, 0.5, 0.0);
    assert!(
        (halfway.inventory_opening_eased_fraction - std::f32::consts::FRAC_1_SQRT_2).abs()
            < 0.000_001
    );
    assert_eq!(halfway.pc_stuff_panel.left, 804.0);
    assert_eq!(halfway.equipment_panel.left, 747.0);

    let mut state = EnchantInventoryUiState0104::default();
    state.tick(0.0, true);
    assert!(!state.panel_controls_enabled());
    state.tick(0.5, true);
    assert_eq!(state.opening_elapsed_seconds(), 0.5);
    state.tick(0.5, true);
    assert!(state.panel_controls_enabled());

    state.apply_legacy_scroll_axis(-1.0);
    assert_eq!(state.scroll_y(), ENCHANT_INVENTORY_SCROLL_MAX_0104);
    let scrolled = enchant_mode_animated_layout_0104(
        1_280,
        720,
        state.opening_elapsed_seconds(),
        state.scroll_y(),
    );
    assert_eq!(scrolled.inventory_scroll_y, 190.0);
    state.apply_legacy_scroll_axis(1.0);
    assert_eq!(state.scroll_y(), 0.0);
    state.tick(0.0, false);
    assert_eq!(state.opening_elapsed_seconds(), 0.0);
    assert_eq!(state.scroll_y(), 0.0);
}
