use super::*;

#[test]
fn world_map_player_projection_preserves_unity_position_and_heading() {
    let transform = Transform::from_translation(Vec3::new(-12.0, 3.0, 45.0));
    let controller = LegacyPlayerController::from_baseline_table().with_yaw(271.5);

    assert_eq!(
        world_map_player_from_native(&transform, &controller),
        WorldMapPlayer::new(WorldMapPoint::new(12.0, 3.0, 45.0), 271.5)
    );
}
