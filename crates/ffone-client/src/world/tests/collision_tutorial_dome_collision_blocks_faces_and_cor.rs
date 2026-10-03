use super::*;

#[test]
fn tutorial_dome_collision_blocks_faces_and_corners_around_the_full_perimeter() {
    const DOME_MODEL: &str = "objects/structures/etc_domeglass_04_01_default_etc_domglass/models/etc_dome_glass_04/ETC_domeglass_04.glb";
    let source_dome = authored_collider_from_glb_mesh(DOME_MODEL, 1);
    let visible_dome = authored_collider_from_glb_mesh(DOME_MODEL, 0);
    assert_eq!(source_dome.vertex_count(), 140);
    assert_eq!(source_dome.index_count(), 198);
    assert_eq!(visible_dome.vertex_count(), 56);
    assert_eq!(visible_dome.index_count(), 84);
    let visible_scale = Vec2::splat(0.119_403_06);
    let footprint_points = visible_dome
        .vertices
        .iter()
        .map(|vertex| Vec2::new(vertex.x, vertex.y) * visible_scale)
        .collect::<Vec<_>>();
    let (vertices, indices) =
        cook_convex_xy_prism(&source_dome.vertices, &footprint_points).unwrap();
    assert_eq!(vertices.len(), 22);
    assert_eq!(indices.len(), 66);
    let (local_min, local_max) = vertices.iter().copied().fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(minimum, maximum), vertex| (minimum.min(vertex), maximum.max(vertex)),
    );
    let dome = AuthoredTriMeshCollider {
        vertices: vertices.into(),
        indices: indices.into(),
        local_min,
        local_max,
        ..source_dome
    };

    let dome_root = Transform::from_translation(Vec3::new(-550.0, -110.0, 650.0))
        .with_scale(Vec3::new(1.0, 2.0, 1.0));
    let collision_child = Transform::from_translation(Vec3::new(-6.232_064_2, 0.0, 10.907_462))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    let world_from_collider = dome_root.to_matrix() * collision_child.to_matrix();
    let colliders = [(world_from_collider, &dome)];

    // The spawned dome surrounds the tutorial start area. Probe more
    // directions than the authored side count so both flat faces and the
    // shared vertices between them are exercised.
    let center = Vec3::new(-556.232_06, -105.4, 656.53);
    let perimeter = (0..dome.vertices.len())
        .step_by(2)
        .map(|index| {
            let vertex = world_from_collider.transform_point3(dome.vertices[index]);
            Vec2::new(vertex.x, vertex.z)
        })
        .collect::<Vec<_>>();
    let catalog = NativeWorldCatalog::open(asset_root()).unwrap();
    let terrain_scene = catalog
        .scenes()
        .iter()
        .find(|scene| scene.scope == Some(NativeWorldScope::WorldMap) && scene.tile == [1, 1])
        .unwrap();
    let terrain_instance = terrain_scene.native_terrain.as_ref().unwrap();
    let terrain = catalog.load_terrain(terrain_instance).unwrap();
    let world_from_terrain = terrain_scene
        .terrain_world_matrix(terrain_instance)
        .unwrap();
    let terrain_from_world = world_from_terrain.inverse();
    let terrain_positions = terrain.geometry().positions();
    let terrain_width = terrain.descriptor().dimensions.width as usize;
    let terrain_height = terrain.descriptor().dimensions.height as usize;
    let terrain_ground = |world_x: f32, world_z: f32| {
        let local = terrain_from_world.transform_point3(Vec3::new(world_x, 0.0, world_z));
        let spacing_x = -terrain_positions[1][0];
        let spacing_z = terrain_positions[terrain_width][2];
        let column_coordinate = -local.x / spacing_x;
        let row_coordinate = local.z / spacing_z;
        assert!(
            column_coordinate >= 0.0
                && column_coordinate <= (terrain_width - 1) as f32
                && row_coordinate >= 0.0
                && row_coordinate <= (terrain_height - 1) as f32,
            "tutorial dome must remain on tile_01_01 terrain: local={local:?}"
        );
        let column = (column_coordinate.floor() as usize).min(terrain_width - 2);
        let row = (row_coordinate.floor() as usize).min(terrain_height - 2);
        let u = column_coordinate - column as f32;
        let v = row_coordinate - row as f32;
        let i00 = row * terrain_width + column;
        let h00 = terrain_positions[i00][1];
        let h10 = terrain_positions[i00 + 1][1];
        let h01 = terrain_positions[i00 + terrain_width][1];
        let h11 = terrain_positions[i00 + terrain_width + 1][1];
        let local_height = if u + v <= 1.0 {
            h00 + u * (h10 - h00) + v * (h01 - h00)
        } else {
            h11 + (1.0 - v) * (h10 - h11) + (1.0 - u) * (h01 - h11)
        };
        world_from_terrain
            .transform_point3(Vec3::new(local.x, local_height, local.z))
            .y
    };
    let assert_inside_perimeter = |position: Vec3, context: &str| {
        let center = Vec2::new(center.x, center.z);
        let world_position = position;
        let position = Vec2::new(position.x, position.z);
        for edge in 0..perimeter.len() {
            let start = perimeter[edge];
            let end = perimeter[(edge + 1) % perimeter.len()];
            let edge_vector = end - start;
            let interior_sign = edge_vector.perp_dot(center - start).signum();
            let signed_distance =
                interior_sign * edge_vector.perp_dot(position - start) / edge_vector.length();
            assert!(
                signed_distance >= -AUTHORED_COLLISION_CONTACT_TOLERANCE,
                "{context} crossed tutorial dome perimeter edge {edge} by {}: \
                 position={world_position:?}",
                -signed_distance
            );
        }
    };
    for sample in 0..64 {
        let angle = std::f32::consts::TAU * sample as f32 / 64.0;
        let direction = Vec3::new(angle.cos(), 0.0, angle.sin());
        let mut resolved = center;
        for frame in 0..800 {
            resolved = resolve_authored_wall_motion(resolved, direction * 0.08, &colliders);
            let nearby_triangles =
                collect_authored_motion_triangles(resolved, Vec3::ZERO, &colliders);
            let residual = deepest_authored_capsule_penetration(
                resolved,
                resolved.y,
                &nearby_triangles,
                true,
            );
            assert!(
                residual.is_none_or(|push| {
                    push.length() <= AUTHORED_COLLISION_CONTACT_TOLERANCE
                }),
                "tutorial dome left a residual capsule penetration at perimeter sample \
                 {sample} on frame {frame}: center={center:?}, direction={direction:?}, \
                 resolved={resolved:?}, residual={residual:?}"
            );
            assert!(
                resolved.distance(center) < 35.0,
                "tutorial dome let the capsule escape through perimeter sample {sample} on \
                 frame {frame}: center={center:?}, direction={direction:?}, \
                 resolved={resolved:?}"
            );
            assert_inside_perimeter(
                resolved,
                &format!("fixed direction sample {sample}, frame {frame}"),
            );
        }
        assert!(
            resolved.distance(center + direction * 64.0) > 20.0,
            "tutorial dome let the capsule escape through perimeter sample {sample}: \
            center={center:?}, direction={direction:?}, resolved={resolved:?}"
        );
    }

    // Reproduce holding W while sweeping the mouse across every authored
    // perimeter corner. Each run first reaches that corner, then keeps an
    // outward component while camera yaw oscillates between its two
    // adjacent faces. A frame-local manifold must not forget the face
    // touched immediately before the yaw change.
    for corner in (0..dome.vertices.len()).step_by(2) {
        let corner_world = world_from_collider.transform_point3(dome.vertices[corner]);
        let radial =
            Vec3::new(corner_world.x - center.x, 0.0, corner_world.z - center.z).normalize();
        for frame_distance in [0.08, 0.4, 0.8, 1.4] {
            let mut resolved = center.with_y(terrain_ground(center.x, center.z));
            for _ in 0..500 {
                resolved = resolve_authored_wall_motion(resolved, radial * 0.08, &colliders);
                resolved.y = terrain_ground(resolved.x, resolved.z);
            }
            for frame in 0..500 {
                let yaw = (frame as f32 * 0.19).sin() * 75.0_f32.to_radians();
                let direction = Quat::from_rotation_y(yaw) * radial;
                resolved = resolve_authored_wall_motion(
                    resolved,
                    direction * frame_distance,
                    &colliders,
                );
                resolved.y = terrain_ground(resolved.x, resolved.z);
                assert!(
                    resolved.distance(center) < 35.0,
                    "tutorial dome let W + camera yaw escape through cooked corner {} on frame \
                     {frame} at frame distance {frame_distance}: center={center:?}, \
                     radial={radial:?}, direction={direction:?}, resolved={resolved:?}",
                    corner / 2
                );
                assert_inside_perimeter(
                    resolved,
                    &format!(
                        "camera yaw corner {}, frame {frame}, distance {frame_distance}",
                        corner / 2
                    ),
                );
            }
        }
    }
}

#[test]
fn authored_collider_materialization_is_cardinality_bound_and_indexed() {
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
    )
    .with_inserted_indices(Indices::U16(vec![0, 1, 2]));
    let pending = PendingAuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/scene_model.glb".to_owned(),
        is_trigger: false,
        expected_vertex_count: 3,
        expected_index_count: 3,
        perimeter_footprint: None,
        cooking: AuthoredColliderCooking::ExactTriangleMesh,
    };
    let collider = authored_collider_from_mesh(&pending, &mesh, None).unwrap();
    assert_eq!(collider.vertex_count(), 3);
    assert_eq!(collider.index_count(), 3);
    assert_eq!(collider.indices.as_ref(), &[0, 2, 1]);
    assert_eq!(collider.source_model_path(), "models/scene_model.glb");

    let published_sidecar = authored_collider_from_mesh(
        &PendingAuthoredTriMeshCollider {
            cooking: AuthoredColliderCooking::PublishedGltfTriangleMesh,
            ..pending.clone()
        },
        &mesh,
        None,
    )
    .unwrap();
    assert_eq!(
        published_sidecar.indices.as_ref(),
        &[0, 1, 2],
        "runtime sidecars are already published in their final glTF handedness"
    );

    let mismatched = PendingAuthoredTriMeshCollider {
        expected_vertex_count: 4,
        ..pending
    };
    assert!(authored_collider_from_mesh(&mismatched, &mesh, None).is_err());
}

#[test]
fn authored_collider_accepts_bevy_flat_normal_vertex_expansion() {
    // The source glTF is a four-vertex indexed quad. Bevy's glTF loader
    // expands it to six corners before computing its missing flat normals.
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    );
    let pending = PendingAuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/collider.glb".to_owned(),
        is_trigger: false,
        expected_vertex_count: 4,
        expected_index_count: 6,
        perimeter_footprint: None,
        cooking: AuthoredColliderCooking::ExactTriangleMesh,
    };

    let collider = authored_collider_from_mesh(&pending, &mesh, None).unwrap();
    assert_eq!(collider.vertex_count(), 6);
    assert_eq!(collider.indices.as_ref(), &[0, 2, 1, 3, 5, 4]);

    let published_sidecar = authored_collider_from_mesh(
        &PendingAuthoredTriMeshCollider {
            cooking: AuthoredColliderCooking::PublishedGltfTriangleMesh,
            ..pending
        },
        &mesh,
        None,
    )
    .unwrap();
    assert_eq!(published_sidecar.vertex_count(), 6);
    assert_eq!(published_sidecar.indices.as_ref(), &[0, 1, 2, 3, 4, 5]);
}
