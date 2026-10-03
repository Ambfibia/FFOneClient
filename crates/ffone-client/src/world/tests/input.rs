use super::*;

#[test]
fn tutorial_startup_targets_match_retrobution_load_position_radius() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let native_position = Vec3::new(-547.0, -105.4, 655.0);
    assert_eq!(
        catalog.legacy_stream_target_tiles(NativeWorldScope::Tutorial, native_position),
        vec![[0, 0], [1, 0], [0, 1], [1, 1]]
    );
}
