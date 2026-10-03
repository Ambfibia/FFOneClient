use super::super::speech_bubbles::npc_bubble_in_range;
use super::*;

#[test]
fn npc_bubble_visibility_distance_retains_nearby_speaker_without_boundary_flicker() {
    let player = Vec3::new(100.0, 3.0, -40.0);
    let mut visible = false;
    for (distance, expected) in [
        (40.0, false),
        (30.5, false),
        (30.0, true),
        (30.01, true),
        (29.99, true),
        (30.8, true),
        (31.01, false),
        (30.8, false),
        (29.9, true),
    ] {
        visible = npc_bubble_in_range(player, player + Vec3::X * distance, visible);
        assert_eq!(visible, expected, "distance={distance}");
    }
    // Vertical separation counts too; the camera's zoom is not the distance origin.
    assert!(!npc_bubble_in_range(player, player + Vec3::Y * 32.0, true));
    assert!(!npc_bubble_in_range(Vec3::NAN, player, true));
    assert!(!npc_bubble_in_range(player, Vec3::INFINITY, true));
}
