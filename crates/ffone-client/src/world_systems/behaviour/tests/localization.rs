use super::*;

#[test]
fn world_matrix_conversion_recovers_translation_and_scale() {
    let matrix = [
        [2.0, 0.0, 0.0, -1536.0],
        [0.0, 2.0, 0.0, 12.5],
        [0.0, 0.0, 2.0, 2048.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let transform = transform_from_world_matrix(&matrix);
    assert!((transform.translation - Vec3::new(-1536.0, 12.5, 2048.0)).length() < 1e-3);
    assert!((transform.scale - Vec3::splat(2.0)).length() < 1e-3);
}
