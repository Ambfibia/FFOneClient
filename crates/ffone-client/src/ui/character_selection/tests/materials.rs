use super::*;

#[test]
fn blocking_modal_camera_is_above_every_character_preview_pass() {
    assert!(
        CHARACTER_SELECTION_PORTRAIT_OVERLAY_CAMERA_ORDER
            > crate::character_selection_portraits::CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE
                + 3
    );
    assert!(
        CHARACTER_SELECTION_MODAL_CAMERA_ORDER
            > crate::player_preview::NATIVE_PLAYER_PREVIEW_CAMERA_ORDER
    );
    assert!(
        CHARACTER_SELECTION_MODAL_CAMERA_ORDER
            > crate::character_selection_portraits::CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE
                + 3
    );
    assert!(
        CHARACTER_SELECTION_MODAL_CAMERA_ORDER
            > CHARACTER_SELECTION_PORTRAIT_OVERLAY_CAMERA_ORDER
    );
    assert_eq!(
        CHARACTER_SELECTION_PORTRAIT_OVERLAY_CAMERA_ORDER,
        GAMEPLAY_UI_CAMERA_ORDER + 8
    );
    assert_eq!(
        CHARACTER_SELECTION_MODAL_CAMERA_ORDER,
        GAMEPLAY_UI_CAMERA_ORDER + 16
    );
    assert!(
        CHARACTER_SELECTION_BASE_INTERACTION_Z_INDEX
            < CHARACTER_SELECTION_PORTRAIT_OVERLAY_INTERACTION_Z_INDEX
    );
    assert!(
        CHARACTER_SELECTION_PORTRAIT_OVERLAY_INTERACTION_Z_INDEX
            < CHARACTER_SELECTION_MODAL_INTERACTION_Z_INDEX
    );
}
