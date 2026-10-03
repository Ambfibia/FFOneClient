use super::*;

#[test]
fn clean_layout_keeps_exact_shell_and_external_camera_rects() {
    let layout = combi_mode_layout_0104(1_264, 681);
    assert_eq!(layout.panel, CombiUiRect::new(114.0, 14.0, 585.0, 653.0));
    assert_eq!(layout.main_group, layout.panel);
    assert_eq!(
        layout.right_backplate,
        CombiUiRect::new(699.0, 14.0, 451.0, 653.0)
    );
    assert_eq!(
        layout.success_group,
        CombiUiRect::new(456.0, 65.5, 352.0, 550.0)
    );
    assert_eq!(
        layout.waiting_group,
        CombiUiRect::new(453.5, 148.5, 357.0, 384.0)
    );
    assert_eq!(
        COMBI_PRIMARY_NPC_PREVIEW_RECT,
        CombiUiRect::new(363.0, 2.0, 143.0, 128.0)
    );
    assert_eq!(
        COMBI_WAITING_NPC_PREVIEW_RECT,
        CombiUiRect::new(0.0, 0.0, 357.0, 384.0)
    );
}
