use super::*;

#[test]
fn production_model_does_not_show_debug_or_reference_content() {
    let model = GameplayUiModel::default();
    assert!(!model.visible);
    assert!(model.player.name.is_empty());
    assert!(model.chat.lines.is_empty());
    assert!(model.minimap.waypoint.is_none());
    assert_eq!(model.minimap.player_marker_alpha, 1.0);
}
