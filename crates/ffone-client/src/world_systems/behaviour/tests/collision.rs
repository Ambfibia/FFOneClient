use super::*;

#[test]
fn organized_collider_routes_restore_the_moving_platform_collision_id() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let routes = load_native_world_object_source_routes(&asset_root, "map_05_12").unwrap();
    let source_path = routes
        .by_node_and_geometry
        .get(&(
            "BuildPlayer-Map_05_12#15874".to_owned(),
            "map-geometry-e52de2b4632baec0b4819c11102e7a1c3d81d8b649284565014f10e5c19a978e"
                .to_owned(),
        ))
        .unwrap();
    assert_eq!(
        legacy_static_model_id(source_path).as_deref(),
        Some("static-map_05_12-01086")
    );

    let behaviours =
        load_native_world_behaviours(&asset_root, NativeWorldScope::WorldMap, "map_05_12")
            .unwrap();
    let platform = behaviours
        .triggers
        .iter()
        .find(|trigger| trigger.node == "BuildPlayer-Map_05_12#15877")
        .unwrap();
    assert_eq!(platform.kind, "platform");
    assert!(
        platform
            .models
            .iter()
            .any(|model| model == "static-map_05_12-01086")
    );
}
