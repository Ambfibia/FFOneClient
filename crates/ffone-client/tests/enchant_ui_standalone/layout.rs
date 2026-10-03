use super::*;

#[test]
fn clean_anchors_layout_and_camera_boundaries_are_exact() {
    assert_eq!(ENCHANT_GAME_MODE_0104, 29);
    assert_eq!(ENCHANT_NPC_TYPE_0104, 27);
    assert_eq!(ENCHANT_GAME_OBJECT_PATH_ID, 1_291);
    assert_eq!(ENCHANT_TRANSFORM_PATH_ID, 1_186);
    assert_eq!(ENCHANT_MODE_COMPONENT_PATH_ID, 1_478);
    assert_eq!(ENCHANT_GUI_COMPONENT_PATH_ID, 1_479);
    assert_eq!(ENCHANT_PRIMARY_CAMERA_PATH_ID, 1_480);
    assert_eq!(ENCHANT_WAITING_CAMERA_PATH_ID, 1_482);
    assert_eq!(ENCHANT_SKIN_PATH_ID, 1_367);

    let layout = enchant_mode_layout_0104(1_280, 720);
    assert_eq!(
        layout.main_group,
        EnchantUiRect0104::new(122.0, 33.0, 585.0, 653.0)
    );
    assert_eq!(
        layout.right_backplate,
        EnchantUiRect0104::new(707.0, 33.0, 451.0, 653.0)
    );
    assert_eq!(
        layout.pc_stuff_panel,
        EnchantUiRect0104::new(715.0, 41.0, 380.0, 632.0)
    );
    assert_eq!(
        layout.equipment_panel,
        EnchantUiRect0104::new(634.0, 41.0, 66.0, 639.0)
    );
    assert_eq!(
        layout.waiting_group,
        EnchantUiRect0104::new(423.0, 142.5, 434.0, 435.0)
    );
    assert_eq!(
        layout.success_group,
        EnchantUiRect0104::new(464.0, 85.0, 352.0, 550.0)
    );
    assert_eq!(
        ENCHANT_PRIMARY_NPC_RECT,
        EnchantUiRect0104::new(355.0, 2.0, 148.0, 147.0)
    );
    assert_eq!(
        ENCHANT_WAITING_NPC_RECT,
        EnchantUiRect0104::new(18.0, 69.0, 403.0, 288.0)
    );
    assert_eq!(
        ENCHANT_DEXLABS_RECT_0104,
        EnchantUiRect0104::new(170.0, 561.0, 203.0, 67.0)
    );
    assert_eq!(
        ENCHANT_TAROS_COUNTER_RECT_0104,
        EnchantUiRect0104::new(20.0, 560.0, 149.0, 32.0)
    );
    assert_eq!(
        ENCHANT_REDEEM_CODE_RECT_0104,
        EnchantUiRect0104::new(15.0, 598.0, 149.0, 25.0)
    );
    assert_eq!(
        ENCHANT_BATTERY_SLOT_RECTS_0104,
        [
            EnchantUiRect0104::new(0.0, 572.0, 64.0, 30.0),
            EnchantUiRect0104::new(0.0, 602.0, 64.0, 30.0),
        ]
    );
}
