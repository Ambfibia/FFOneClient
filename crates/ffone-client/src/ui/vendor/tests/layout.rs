use super::*;

#[test]
fn final_1264x681_layout_matches_clean_vendor_and_shared_item_panels() {
    let layout = vendor_mode_layout(1_264, 681, VENDOR_OPEN_SECONDS, 0.0, 0.0, 8);
    assert_eq!(
        layout.vendor_backplate,
        VendorUiRect::new(114.0, 14.0, 585.0, 653.0)
    );
    assert_eq!(
        layout.right_backplate,
        VendorUiRect::new(699.0, 14.0, 451.0, 653.0)
    );
    assert_eq!(
        layout.vendor_panel,
        VendorUiRect::new(122.0, 21.0, 498.0, 638.0)
    );
    assert_eq!(
        layout.npc_preview_boundary,
        VendorUiRect::new(432.0, 21.0, 200.0, 150.0)
    );
    assert_eq!(
        layout.vendor_info,
        VendorUiRect::new(125.0, 30.0, 481.0, 87.0)
    );
    assert_eq!(
        layout.vendor_dialog,
        VendorUiRect::new(133.0, 153.0, 486.0, 528.0)
    );
    assert_eq!(layout.table, VendorUiRect::new(135.0, 185.0, 479.0, 410.0));
    assert_eq!(
        layout.list_viewport,
        VendorUiRect::new(145.0, 195.0, 464.0, 400.0)
    );
    assert_eq!(
        layout.go_to_stuff,
        VendorUiRect::new(435.0, 616.0, 161.0, 25.0)
    );
    assert_eq!(layout.item_mode.pc_stuff_panel.left, 707.0);
    assert_eq!(layout.item_mode.equipment_panel.left, 626.0);
    assert_eq!(vendor_scroll_max(8), 240.0);
    assert_eq!(
        layout.row_rect(0),
        Some(VendorUiRect::new(145.0, 195.0, 433.0, 75.0))
    );
    assert_eq!(
        layout.row_rect(7),
        Some(VendorUiRect::new(145.0, 755.0, 433.0, 75.0))
    );

    let start = vendor_mode_layout(1_264, 681, 0.0, 0.0, 0.0, 8);
    assert_eq!(start.vendor_panel.left, -498.0);
    assert_eq!(start.item_mode.pc_stuff_panel.left, 1_020.0);
    assert_eq!(start.item_mode.equipment_panel.left, 1_020.0);
}
