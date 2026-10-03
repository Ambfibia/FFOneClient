use super::*;

#[test]
fn custom_waypoints_filters_and_drag_are_independent_of_frame_time() {
    let mut model = open_other_local();
    let gates = WorldMapInputGates::default();
    let center = model.map_draw_rect().center();
    for _ in 0..6 {
        assert!(model.place_custom_waypoint(center, gates));
    }
    assert!(!model.place_custom_waypoint(center, gates));
    assert!(model.remove_custom_waypoint(2, gates));
    assert!(model.place_custom_waypoint(center, gates));
    assert_eq!(model.preferences.waypoints.last().unwrap().color, 2);
    assert!(!model.remove_custom_waypoint(
        2,
        WorldMapInputGates {
            system_popup_open: true,
            ..Default::default()
        }
    ));
    model.select_world_view(gates).unwrap();
    assert_eq!(model.toggle_filter(0, gates), WorldMapInputResult::Changed);
    assert!(!model.preferences.enabled_icons.contains(&15));
    assert!(!model.preferences.enabled_icons.contains(&16));
    assert!(model.preferences.enabled_icons.contains(&22));
    model.select_local_view(gates).unwrap();
    model.begin_drag(center, gates).unwrap();
    let mut other = model.clone();
    model.drag_by(12.0, 7.0, 1.0 / 30.0, gates).unwrap();
    other.drag_by(12.0, 7.0, 1.0 / 144.0, gates).unwrap();
    assert_eq!(model.target_view(), other.target_view());
    assert_eq!(model.preferences.waypoints.len(), 6);
}
