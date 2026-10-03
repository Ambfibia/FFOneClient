use super::*;

#[test]
fn final_1264x681_layout_matches_clean_bank_and_shared_item_panels() {
    let layout = bank_mode_layout(1_264, 681, BANK_OPEN_SECONDS, 0.0, 0.0);
    assert_eq!(
        layout.bank_backplate,
        BankUiRect::new(114.0, 14.0, 585.0, 653.0)
    );
    assert_eq!(
        layout.right_backplate,
        BankUiRect::new(699.0, 14.0, 451.0, 653.0)
    );
    assert_eq!(
        layout.bank_panel,
        BankUiRect::new(122.0, 21.0, 495.0, 638.0)
    );
    assert_eq!(layout.bank_info, BankUiRect::new(125.0, 30.0, 481.0, 87.0));
    assert_eq!(
        layout.bank_dialog,
        BankUiRect::new(133.0, 126.0, 482.0, 531.0)
    );
    assert_eq!(
        layout.bank_viewport,
        BankUiRect::new(162.0, 160.0, 450.0, 445.0)
    );
    assert_eq!(layout.bank_content.width, 402.0);
    assert_eq!(layout.bank_content.height, 2_278.0);
    assert_eq!(layout.item_mode.pc_stuff_panel.left, 707.0);
    assert_eq!(layout.item_mode.equipment_panel.left, 626.0);
    assert_eq!(bank_scroll_max(), 1_833.0);

    let first = layout.bank_slot_rect(0).unwrap();
    let last = layout.bank_slot_rect(199).unwrap();
    assert_eq!(first, BankUiRect::new(162.0, 160.0, 66.0, 66.0));
    assert_eq!(last, BankUiRect::new(229.0, 2_371.0, 66.0, 66.0));

    let start = bank_mode_layout(1_264, 681, 0.0, 0.0, 0.0);
    assert_eq!(start.bank_panel.left, -475.0);
    assert_eq!(start.item_mode.pc_stuff_panel.left, 1_020.0);
    assert_eq!(start.item_mode.equipment_panel.left, 1_020.0);
}
