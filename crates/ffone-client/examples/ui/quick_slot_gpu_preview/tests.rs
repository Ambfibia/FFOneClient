use ffone_client::quick_slot_ui::{
    QuickSlotUiConfig, QuickSlotUiRect, QuickSlotVisual, quick_slot_ui_view,
};

use super::*;

#[test]
fn clean_default_remains_hidden_until_preview_opt_in() {
    let view = quick_slot_ui_view(
        CLIENT_AREA_WIDTH,
        CLIENT_AREA_HEIGHT,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::default(),
        &preview_model(),
        true,
    );
    assert!(!view.visible);
    assert_eq!(view.background, None);
}

#[test]
fn opt_in_frame_uses_exact_geometry_and_cooldown_state() {
    let view = quick_slot_ui_view(
        CLIENT_AREA_WIDTH,
        CLIENT_AREA_HEIGHT,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::enabled(),
        &preview_model(),
        true,
    );
    assert!(view.visible);
    assert_eq!(
        view.background,
        Some(QuickSlotUiRect::new(450.0, 641.0, 290.0, 39.0))
    );
    assert_eq!(
        view.slots[0].frame,
        QuickSlotUiRect::new(456.0, 645.0, 34.0, 34.0)
    );
    assert_eq!(
        view.slots[7].frame,
        QuickSlotUiRect::new(701.0, 645.0, 34.0, 34.0)
    );
    assert!(
        view.slots
            .iter()
            .all(|slot| slot.visual == QuickSlotVisual::Occupied)
    );
    assert_eq!(
        view.slots[0].cooldown,
        Some(QuickSlotUiRect::new(457.0, 646.0, 32.0, 32.0))
    );
    assert_eq!(view.slots[7].cooldown, None);
    assert!(!view.slots[7].pointer_enabled);
}
