use super::*;

#[test]
fn exact_1264x681_shell_and_shared_right_panel_match_clean_geometry() {
    let final_layout = pc2pc_mode_layout(1_264, 681, PC2PC_OPEN_SECONDS, 0.0);
    assert_eq!(
        final_layout.trade_backplate,
        Pc2pcUiRect::new(114.0, 14.0, 585.0, 653.0)
    );
    assert_eq!(
        final_layout.right_backplate,
        Pc2pcUiRect::new(699.0, 14.0, 451.0, 653.0)
    );
    assert_eq!(
        final_layout.trade_panel,
        Pc2pcUiRect::new(122.0, 21.0, 550.0, 700.0)
    );
    assert_eq!(
        final_layout.trade_area,
        Pc2pcUiRect::new(134.0, 50.0, 480.0, 448.0)
    );
    assert_eq!(
        final_layout.local_offer,
        Pc2pcUiRect::new(147.0, 56.0, 480.0, 157.0)
    );
    assert_eq!(
        final_layout.remote_offer,
        Pc2pcUiRect::new(122.0, 221.0, 480.0, 172.0)
    );
    assert_eq!(
        final_layout.local_offer_slot(0),
        Some(Pc2pcUiRect::new(198.0, 92.0, 62.0, 62.0))
    );
    assert_eq!(
        final_layout.local_offer_slot(4),
        Some(Pc2pcUiRect::new(466.0, 92.0, 62.0, 62.0))
    );
    assert_eq!(
        final_layout.remote_offer_slot(0),
        Some(Pc2pcUiRect::new(240.0, 258.0, 62.0, 62.0))
    );
    assert_eq!(
        final_layout.main_button,
        Pc2pcUiRect::new(462.0, 453.0, 133.0, 30.0)
    );
    assert_eq!(
        final_layout.chat_box,
        Pc2pcUiRect::new(132.0, 511.0, 483.0, 141.0)
    );
    assert_eq!(final_layout.item_mode.pc_stuff_panel.left, 707.0);
    assert_eq!(final_layout.item_mode.equipment_panel.left, 626.0);

    let start = pc2pc_mode_layout(1_264, 681, 0.0, 0.0);
    assert_eq!(start.trade_panel.left, -475.0);
    assert_eq!(start.item_mode.pc_stuff_panel.left, 1_020.0);
    assert_eq!(start.item_mode.equipment_panel.left, 1_020.0);
}
