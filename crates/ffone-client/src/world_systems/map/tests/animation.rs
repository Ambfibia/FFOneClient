use super::*;

#[test]
fn route_segments_retain_arcs_dashes_and_clip_both_endpoints() {
    let clip = WorldMapUiRect::new(0.0, 0.0, 100.0, 100.0);
    let solid =
        world_map_route_segments(Vec2::new(-20.0, 20.0), Vec2::new(120.0, 20.0), false, clip);
    let dashed =
        world_map_route_segments(Vec2::new(-20.0, 20.0), Vec2::new(120.0, 20.0), true, clip);
    assert!(solid.len() > dashed.len() && !dashed.is_empty());
    assert!(
        solid
            .iter()
            .flat_map(|(a, b)| [a, b])
            .all(|p| p.x >= -0.001 && p.x <= 100.001 && p.y >= 0.0 && p.y <= 100.0)
    );
    assert!(solid.iter().any(|(_, b)| b.y > 25.0));
    assert!(world_map_route_segments(Vec2::ZERO, Vec2::ZERO, false, clip).is_empty());
}
