use crate::legacy_world_location::*;

#[test]
fn tutorial_spawn_is_future_tech_square() {
    assert_eq!(
        legacy_world_location_name(547.0, 655.0),
        Some("Tech Square")
    );
}

#[test]
fn present_and_future_world_rectangles_are_kept_separate() {
    assert_eq!(
        legacy_world_location_name(3072.0, 2560.0),
        Some("Tech Square")
    );
    assert_eq!(legacy_world_location_name(6144.0, 1536.0), Some("Sector V"));
}

#[test]
fn unity_rect_maximum_edges_are_exclusive() {
    assert_eq!(
        legacy_world_location_name(4607.999, 2560.0),
        Some("Candy Cove")
    );
    assert_ne!(
        legacy_world_location_name(4608.0, 2560.0),
        Some("Candy Cove")
    );
}

#[test]
fn instance_points_keep_only_a_named_last_location() {
    // Instance/dungeon coordinates lie outside every WorldNameScript rect.
    assert_eq!(legacy_world_location_name(100.0, 100.0), None);
    assert_eq!(
        legacy_world_location_name_or_last("Candy Cove", 100.0, 100.0),
        Some("Candy Cove")
    );
    assert_eq!(
        legacy_world_location_name_or_last("Map 12", 100.0, 100.0),
        None
    );
    assert_eq!(legacy_world_location_name_or_last("", 100.0, 100.0), None);
    assert_eq!(
        legacy_world_location_name_or_last("Candy Cove", 3072.0, 2560.0),
        Some("Tech Square")
    );
}

#[test]
fn invalid_or_unmapped_positions_have_no_name() {
    assert_eq!(legacy_world_location_name(f32::NAN, 0.0), None);
    assert_eq!(legacy_world_location_name(8000.0, 8000.0), None);
}
