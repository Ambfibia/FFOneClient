use super::*;

#[test]
fn background_strip_wraps_at_the_same_final_frame_boundary() {
    let layout = CharacterSelectionLayout::for_viewport(Vec2::new(1264.0, 681.0), 1.0);
    let wrap_time = (5420.0 - 1264.0) / CHARACTER_SELECTION_BACKGROUND_SPEED;
    assert_eq!(
        layout.background_frame(0, wrap_time),
        LegacySelectionRect::new(1264.0, 38.5, 542.0, 477.0)
    );
    let before = layout.background_frame(9, wrap_time - 0.01);
    assert!(before.x + before.width > 1264.0);
}
