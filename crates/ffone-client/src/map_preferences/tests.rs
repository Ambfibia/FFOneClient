use super::*;
#[test]
fn six_distinct_colors_survive_round_trip_and_deleted_colors_are_reused() {
    let mut prefs = MapPreferences::default();
    for color in 0..6 {
        assert_eq!(prefs.add_waypoint(4096.0, 4096.0), Some(color));
    }
    assert_eq!(prefs.add_waypoint(4096.0, 4096.0), None);
    prefs.waypoints.remove(2);
    assert_eq!(prefs.add_waypoint(100.0, 200.0), Some(2));
    assert!(prefs.validate());
    assert_eq!(
        serde_json::from_str::<MapPreferences>(&serde_json::to_string(&prefs).unwrap())
            .unwrap(),
        prefs
    );
    assert_eq!(prefs.add_waypoint(f32::NAN, 1.0), None);
}
