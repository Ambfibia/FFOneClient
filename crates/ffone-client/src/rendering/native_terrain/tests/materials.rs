use super::*;

#[test]
fn realtime_vertex_pass_uses_the_original_main_data_ambient_and_direction() {
    let flat = legacy_terrain_vertex_light(Vec3::Y);
    let expected = LEGACY_TERRAIN_VERTEX_AMBIENT + LEGACY_TERRAIN_SURFACE_TO_LIGHT.y;
    assert!((flat.x - expected).abs() <= f32::EPSILON);
    assert_eq!(flat, Vec4::new(expected, expected, expected, 1.0));

    let away = legacy_terrain_vertex_light(-LEGACY_TERRAIN_SURFACE_TO_LIGHT);
    assert_eq!(
        away,
        Vec4::new(
            LEGACY_TERRAIN_VERTEX_AMBIENT,
            LEGACY_TERRAIN_VERTEX_AMBIENT,
            LEGACY_TERRAIN_VERTEX_AMBIENT,
            1.0,
        )
    );
}
