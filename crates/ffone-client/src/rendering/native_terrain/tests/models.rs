use super::*;

#[test]
fn map_05_05_applies_the_authored_unity_cliff_vertex_shift() {
    let descriptor_path = "map/tiles/map_05_05/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    let geometry = terrain.geometry();

    assert_eq!(terrain.descriptor().heightmap.vertex_shifts.len(), 634);
    assert!(
        terrain
            .descriptor()
            .heightmap
            .vertex_shifts
            .contains(&NativeTerrainVertexShift {
                flags: 4,
                column: 60,
                row: 18,
            })
    );

    // Primary TerrainData_05_05 stores a 22.8 m transition and moves its
    // complete upper edge one full row toward the lower ground. Unity's
    // m_Shifts therefore turns the slope into the vertical wall that meets
    // the neighboring road geometry instead of leaving a four-metre gap.
    let column = 60;
    let lower = geometry.positions()[17 * 129 + column];
    let upper = geometry.positions()[18 * 129 + column];
    assert!((upper[1] - lower[1] - 22.797_326).abs() < 0.001);
    assert_eq!(lower[2], 68.0);
    assert_eq!(upper[2], 68.0);
    assert_eq!(geometry.positions()[19 * 129 + column][2], 76.0);
    assert_eq!(
        geometry.uvs()[18 * 129 + column][1],
        1.0 - 18.0 / 128.0 + 1.0 / 256.0
    );

    let cliff = [17 * 129 + column, 17 * 129 + column + 1, 18 * 129 + column];
    let a = Vec3::from_array(geometry.positions()[cliff[0]]);
    let b = Vec3::from_array(geometry.positions()[cliff[1]]);
    let c = Vec3::from_array(geometry.positions()[cliff[2]]);
    let face = (b - a).cross(c - a);
    assert!(face.y.abs() <= f32::EPSILON);
    assert!(face.z.abs() > 1.0);

    let collider = NativeHeightmapCollider {
        source_mesh: Handle::default(),
        source_descriptor_path: descriptor_path.to_owned(),
        geometry: terrain.geometry().clone(),
        width: 129,
        height: 129,
        sample_spacing_x: 4.0,
        sample_spacing_z: 4.0,
        terrain_size_x: 512.0,
        terrain_size_z: 512.0,
        gameplay_attributes: None,
    };
    let owner = GlobalTransform::from(Transform::from_xyz(0.0, -300.0, 0.0));
    let closed_edge_height = collider
        .ground_height(&owner, -240.0, 70.0, -100.0, 100.0)
        .unwrap();
    assert!(
        closed_edge_height > -30.0,
        "the shifted upper plateau must cover z=70 instead of leaving the old sloped gap"
    );
}
