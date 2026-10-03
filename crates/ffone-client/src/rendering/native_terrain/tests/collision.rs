use super::*;

pub(super) fn legacy_height_test_collider() -> NativeHeightmapCollider {
    NativeHeightmapCollider {
        source_mesh: Handle::default(),
        source_descriptor_path: "test/terrain.json".to_owned(),
        geometry: Arc::new(NativeTerrainGeometry {
            positions: vec![
                [0.0, 0.0, 0.0],
                [-2.0, 10.0, 0.0],
                [0.0, 20.0, 4.0],
                [-2.0, 40.0, 4.0],
            ]
            .into(),
            normals: vec![[0.0, 1.0, 0.0]; 4].into(),
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]].into(),
            indices: vec![0, 1, 2, 1, 3, 2].into(),
        }),
        width: 2,
        height: 2,
        sample_spacing_x: 2.0,
        sample_spacing_z: 4.0,
        terrain_size_x: 2.0,
        terrain_size_z: 4.0,
        gameplay_attributes: None,
    }
}

#[test]
fn legacy_sample_height_is_four_neighbor_bilinear_not_rendered_triangle() {
    let collider = legacy_height_test_collider();
    let owner_and_root = GlobalTransform::from(Transform::from_xyz(100.0, -30.0, 200.0));

    let legacy = require_legacy_height(collider.legacy_interpolated_height(
        &owner_and_root,
        99.0,
        202.0,
    ));
    let rendered_triangle = collider
        .ground_height(&owner_and_root, 99.0, 202.0, -100.0, 100.0)
        .unwrap();

    // Four-corner midpoint: (0 + 10 + 20 + 40) / 4 - 30.
    assert!((legacy - -12.5).abs() <= f32::EPSILON);
    // The rendered diagonal evaluates to 15 - 30 at the same X/Z.
    assert!((rendered_triangle - -15.0).abs() <= f32::EPSILON);
    assert_ne!(legacy, rendered_triangle);
}
