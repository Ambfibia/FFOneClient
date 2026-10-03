use super::*;

pub(super) fn closed_vehicle_test_collider() -> AuthoredTriMeshCollider {
    AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/test-kinematic-car.glb".to_owned(),
        vertices: vec![
            Vec3::new(-1.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, -1.0),
            Vec3::new(1.0, 2.0, -1.0),
            Vec3::new(-1.0, 2.0, -1.0),
            Vec3::new(-1.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 2.0, 1.0),
            Vec3::new(-1.0, 2.0, 1.0),
        ]
        .into(),
        // Closed, consistently outward-wound car-like body.
        indices: vec![
            0, 2, 1, 0, 3, 2, // -Z
            4, 5, 6, 4, 6, 7, // +Z
            0, 4, 7, 0, 7, 3, // -X
            1, 2, 6, 1, 6, 5, // +X
            0, 1, 5, 0, 5, 4, // -Y
            3, 7, 6, 3, 6, 2, // +Y
        ]
        .into(),
        local_min: Vec3::new(-1.0, 0.0, -1.0),
        local_max: Vec3::new(1.0, 2.0, 1.0),
        is_trigger: false,
    }
}

#[test]
fn peach_creek_collision_alpha_sheets_are_not_rendered() {
    let visual = NativeWorldVisual {
        name: "collision_512-02 - default-collision_alpha [BuildPlayer-Map_07_08#9581 visual]"
            .to_owned(),
        model: "map-geometry-helper".to_owned(),
        scene: 0,
        source_model_path: String::new(),
        legacy_layer: None,
        transform: AuthoredWorldTransform {
            translation: [-3656.7, -50.6, 4563.2],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
    };
    assert!(has_legacy_collision_helper_identity(&visual, None));

    let mut ordinary = visual;
    ordinary.name = "wd_etc_stone07_02-standard_1-wd_etc_stone05".to_owned();
    assert!(!has_legacy_collision_helper_identity(&ordinary, None));
    assert!(has_legacy_collision_helper_identity(
        &ordinary,
        Some(
            "objects/collision/default_collision_alpha/models/\
             collision_512_02_default_collision_alpha/visual.glb"
        )
    ));
}

#[test]
fn collision_alpha_helper_spawn_has_metadata_but_no_render_asset_request() {
    let scene_path = asset_root().join("map/tiles/map_08_05/scene.json");
    let scene: NativeWorldScene =
        serde_json::from_slice(&fs::read(scene_path).unwrap()).unwrap();
    let (visual_index, visual) = scene
        .visuals
        .iter()
        .enumerate()
        .find(|(_, visual)| is_legacy_collision_helper_visual(&scene, visual))
        .expect("Candy Cove must retain the audited helper metadata");

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    let root = app.world_mut().spawn_empty().id();
    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut queue = CommandQueue::default();
    let helper = {
        let mut commands = Commands::new(&mut queue, app.world());
        spawn_native_world_visual(
            &mut commands,
            &asset_server,
            &scene,
            root,
            visual_index,
            visual,
        )
        .unwrap()
    };
    queue.apply(app.world_mut());

    let entity = app.world().entity(helper);
    assert!(entity.contains::<SpawnedNativeWorldVisual>());
    assert!(entity.contains::<NativeWorldVisualSceneReady>());
    assert_eq!(entity.get::<Visibility>(), Some(&Visibility::Hidden));
    assert!(!entity.contains::<PendingNativeWorldVisualAsset>());
    assert!(!entity.contains::<NativeWorldObjectRangeMember>());
    assert!(!entity.contains::<Mesh3d>());
    assert!(!entity.contains::<WorldAssetRoot>());
}

#[test]
fn authored_collider_spatial_index_returns_only_overlapping_xz_cells() {
    let near = Entity::from_bits(1);
    let far = Entity::from_bits(2);
    let oversized = Entity::from_bits(3);
    let mut index = AuthoredColliderSpatialIndex::default();
    index.insert(
        near,
        &AuthoredColliderWorldBounds {
            minimum: Vec3::new(-1.0, -10.0, -1.0),
            maximum: Vec3::new(1.0, 10.0, 1.0),
        },
    );
    index.insert(
        far,
        &AuthoredColliderWorldBounds {
            minimum: Vec3::new(95.0, -10.0, 95.0),
            maximum: Vec3::new(97.0, 10.0, 97.0),
        },
    );
    index.insert(
        oversized,
        &AuthoredColliderWorldBounds {
            minimum: Vec3::new(-10_000.0, -10.0, -10_000.0),
            maximum: Vec3::new(10_000.0, 10.0, 10_000.0),
        },
    );

    let mut candidates = Vec::new();
    index.point_candidates(0.0, 0.0, &mut candidates);
    assert_eq!(candidates, vec![near, oversized]);
    index.point_candidates(96.0, 96.0, &mut candidates);
    assert_eq!(candidates, vec![far, oversized]);

    index.insert(
        near,
        &AuthoredColliderWorldBounds {
            minimum: Vec3::new(159.0, -10.0, 159.0),
            maximum: Vec3::new(161.0, 10.0, 161.0),
        },
    );
    index.point_candidates(0.0, 0.0, &mut candidates);
    assert_eq!(candidates, vec![oversized]);
    index.point_candidates(160.0, 160.0, &mut candidates);
    assert_eq!(candidates, vec![near, oversized]);

    index.remove(oversized);
    index.point_candidates(160.0, 160.0, &mut candidates);
    assert_eq!(candidates, vec![near]);
}

#[test]
fn cached_collider_placements_share_exact_cooked_buffers() {
    let vertices: Arc<[Vec3]> = vec![Vec3::ZERO, Vec3::X, Vec3::Z].into();
    let indices: Arc<[u32]> = vec![0, 1, 2].into();
    let geometry = AuthoredColliderGeometry {
        vertices: Arc::clone(&vertices),
        indices: Arc::clone(&indices),
        local_min: Vec3::ZERO,
        local_max: Vec3::ONE,
    };
    let pending = PendingAuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "objects/test/collision.glb".to_owned(),
        is_trigger: false,
        expected_vertex_count: 3,
        expected_index_count: 3,
        perimeter_footprint: None,
        cooking: AuthoredColliderCooking::ExactTriangleMesh,
    };

    let first = authored_collider_from_cached_geometry(&pending, &geometry);
    let second = authored_collider_from_cached_geometry(&pending, &geometry);
    assert!(Arc::ptr_eq(&first.vertices, &second.vertices));
    assert!(Arc::ptr_eq(&first.indices, &second.indices));
    assert!(Arc::ptr_eq(&first.vertices, &vertices));
    assert!(Arc::ptr_eq(&first.indices, &indices));
}

#[test]
fn every_published_collision_alpha_visual_is_classified_as_a_helper() {
    let tiles = asset_root().join("map/tiles");
    let mut helper_count = 0_usize;
    let mut affected_tiles = 0_usize;
    for entry in std::fs::read_dir(tiles).unwrap() {
        let scene_path = entry.unwrap().path().join("scene.json");
        if !scene_path.is_file() {
            continue;
        }
        let scene: NativeWorldScene =
            serde_json::from_slice(&std::fs::read(scene_path).unwrap()).unwrap();
        let tile_helpers = scene
            .visuals
            .iter()
            .filter(|visual| visual.name.to_ascii_lowercase().contains("collision_alpha"))
            .inspect(|visual| assert!(is_legacy_collision_helper_visual(&scene, visual)))
            .count();
        helper_count += tile_helpers;
        affected_tiles += usize::from(tile_helpers > 0);
    }
    assert_eq!(helper_count, 1_639);
    assert_eq!(affected_tiles, 101);
}

pub(super) fn authored_collider_from_glb(relative: &str) -> AuthoredTriMeshCollider {
    authored_collider_from_glb_meshes(relative, None)
}

pub(super) fn authored_collider_from_glb_mesh(
    relative: &str,
    mesh_index: usize,
) -> AuthoredTriMeshCollider {
    authored_collider_from_glb_meshes(relative, Some(mesh_index))
}

pub(super) fn authored_collider_from_glb_meshes(
    relative: &str,
    mesh_index: Option<usize>,
) -> AuthoredTriMeshCollider {
    let (vertices, mut indices) = authored_gltf_mesh_data(relative, mesh_index);
    for triangle in indices.chunks_exact_mut(3) {
        triangle.swap(1, 2);
    }
    let local_min = vertices
        .iter()
        .copied()
        .reduce(Vec3::min)
        .expect("collider must have vertices");
    let local_max = vertices.iter().copied().reduce(Vec3::max).unwrap();
    AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: relative.to_owned(),
        vertices: vertices.into(),
        indices: indices.into(),
        local_min,
        local_max,
        is_trigger: false,
    }
}

pub(super) fn published_gltf_collider_from_glb(relative: &str) -> AuthoredTriMeshCollider {
    cooked_collider_from_glb(relative, AuthoredColliderCooking::PublishedGltfTriangleMesh)
}

pub(super) fn native_water_collider_from_glb(relative: &str) -> AuthoredTriMeshCollider {
    cooked_collider_from_glb(relative, AuthoredColliderCooking::NativeWaterSurface)
}

pub(super) fn cooked_collider_from_glb(
    relative: &str,
    cooking: AuthoredColliderCooking,
) -> AuthoredTriMeshCollider {
    let (vertices, indices) = authored_gltf_mesh_data(relative, None);
    let expected_vertex_count = vertices.len();
    let expected_index_count = indices.len();
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices
            .into_iter()
            .map(|vertex| vertex.to_array())
            .collect::<Vec<_>>(),
    )
    .with_inserted_indices(Indices::U32(indices));
    authored_collider_from_mesh(
        &PendingAuthoredTriMeshCollider {
            source_mesh: Handle::default(),
            source_model_path: relative.to_owned(),
            is_trigger: false,
            expected_vertex_count,
            expected_index_count,
            perimeter_footprint: None,
            cooking,
        },
        &mesh,
        None,
    )
    .expect("published glTF collider must satisfy its runtime cooking contract")
}

pub(super) fn published_map_collider_for_source(
    source_model_path: &str,
) -> (Mat4, AuthoredTriMeshCollider) {
    let legacy_tile = source_model_path
        .split('/')
        .find(|component| component.starts_with("tile_"))
        .expect("source model path must retain its legacy tile identity");
    let tile_id = format!(
        "map_{}",
        legacy_tile
            .strip_prefix("tile_")
            .expect("legacy tile identity was checked above")
    );
    let objects: serde_json::Value = serde_json::from_slice(
        &fs::read(asset_root().join(format!("map/tiles/{tile_id}/objects.json"))).unwrap(),
    )
    .unwrap();
    let (instance, geometry_id) = objects["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|instance| {
            instance["sourceGeometry"]
                .as_object()
                .unwrap()
                .iter()
                .find(|(_, source)| source.as_str() == Some(source_model_path))
                .map(|(geometry_id, _)| (instance, geometry_id.clone()))
        })
        .expect("published map tile must retain the source-geometry identity");
    let catalog: serde_json::Value =
        serde_json::from_slice(&fs::read(asset_root().join("map/catalog.json")).unwrap())
            .unwrap();
    let geometry = catalog["geometry"]
        .as_array()
        .unwrap()
        .iter()
        .find(|geometry| geometry["id"].as_str() == Some(&geometry_id))
        .expect("map catalog must own the source geometry");
    let model_path = geometry["model"]["path"].as_str().unwrap();
    let mut collider = authored_collider_from_glb(model_path);
    collider.source_model_path = source_model_path.to_owned();

    let rows = instance["worldMatrix"].as_array().unwrap();
    let mut columns = [0.0_f32; 16];
    for row in 0..4 {
        for column in 0..4 {
            columns[column * 4 + row] =
                rows[row][column].as_str().unwrap().parse::<f32>().unwrap();
        }
    }
    (Mat4::from_cols_array(&columns), collider)
}

pub(super) fn authored_collider_contains_point(collider: &AuthoredTriMeshCollider, point: Vec3) -> bool {
    let mut solid_angle = 0.0_f32;
    for triangle in collider.indices.chunks_exact(3) {
        let a = collider.vertices[triangle[0] as usize] - point;
        let b = collider.vertices[triangle[1] as usize] - point;
        let c = collider.vertices[triangle[2] as usize] - point;
        let denominator = a.length() * b.length() * c.length()
            + a.dot(b) * c.length()
            + b.dot(c) * a.length()
            + c.dot(a) * b.length();
        solid_angle += 2.0 * a.dot(b.cross(c)).atan2(denominator);
    }
    solid_angle.abs() > std::f32::consts::TAU
}

#[test]
fn world_presentation_waits_for_every_scene_instance_and_collider() {
    let ready = NativeWorldColliderStatus::Ready {
        vertex_count: 3,
        index_count: 3,
    };
    let loading = NativeWorldColliderStatus::Loading;
    let blocked = NativeWorldColliderStatus::Blocked("fixture".to_owned());

    assert!(!native_world_presentation_ready(
        [true, false],
        [true],
        [&ready]
    ));
    assert!(!native_world_presentation_ready([true], [false], [&ready]));
    assert!(!native_world_presentation_ready([true], [true], [&loading]));
    assert!(!native_world_presentation_ready([true], [true], [&blocked]));
    assert!(native_world_presentation_ready(
        [true, true],
        [true, true],
        [&ready]
    ));
}

#[test]
fn tutorial_infected_water_keeps_upward_native_collision_winding() {
    const COLLIDER_PATH: &str =
        "objects/unclassified/ffpoison/models/ffpoison_variant_0059/collision.glb";
    const REVERSED_COLLIDER_PATH: &str =
        "objects/unclassified/ffpoison/models/ffpoison_variant_0002/collision.glb";

    assert_eq!(
        native_world_collider_cooking("ffPoison"),
        AuthoredColliderCooking::NativeWaterSurface
    );
    assert_eq!(
        native_world_collider_cooking("ffWater"),
        AuthoredColliderCooking::NativeWaterSurface
    );
    assert_eq!(
        native_world_collider_cooking("ordinary-world-object"),
        AuthoredColliderCooking::ExactTriangleMesh
    );

    let (vertices, indices) = authored_gltf_mesh_data(COLLIDER_PATH, Some(0));
    let triangle = indices
        .chunks_exact(3)
        .find_map(|triangle| {
            let a = vertices[triangle[0] as usize];
            let b = vertices[triangle[1] as usize];
            let c = vertices[triangle[2] as usize];
            ((b - a).cross(c - a).normalize_or_zero().y > 0.9).then_some((a, b, c))
        })
        .expect("published tutorial ffPoison must have an upward-facing surface");
    let probe = (triangle.0 + triangle.1 + triangle.2) / 3.0;

    let native = native_water_collider_from_glb(COLLIDER_PATH);
    let support = collider_ground_height(
        &native,
        Mat4::IDENTITY,
        probe.x,
        probe.z,
        probe.y - 1.0,
        probe.y + 1.0,
    );
    assert!(
        support.is_some_and(|height| (height - probe.y).abs() < 0.01),
        "the final native winding must support the player on tutorial infected water"
    );

    let incorrectly_reversed = authored_collider_from_glb(COLLIDER_PATH);
    assert_eq!(
        collider_ground_height(
            &incorrectly_reversed,
            Mat4::IDENTITY,
            probe.x,
            probe.z,
            probe.y - 1.0,
            probe.y + 1.0,
        ),
        None,
        "the former generic static-world reversal makes the water one-sided downward"
    );

    let reversed = native_water_collider_from_glb(REVERSED_COLLIDER_PATH);
    let first = &reversed.indices[..3];
    let a = reversed.vertices[first[0] as usize];
    let b = reversed.vertices[first[1] as usize];
    let c = reversed.vertices[first[2] as usize];
    assert!(
        (b - a).cross(c - a).y > 0.0,
        "source-order water variants must still be normalized upward"
    );
}

#[test]
fn collider_cook_budget_is_cardinality_bounded_without_starvation() {
    let mut budget = AuthoredColliderCookBudget::default();
    assert!(budget.admit(20_000, 40_000));
    assert!(!budget.admit(20_000, 40_000));
    assert!(budget.admit(1_000, 1_000));

    let mut oversized = AuthoredColliderCookBudget::default();
    assert!(oversized.admit(
        AUTHORED_COLLIDER_VERTICES_PER_FRAME + 1,
        AUTHORED_COLLIDER_INDICES_PER_FRAME + 1
    ));
    assert!(!oversized.admit(1, 1));
}

#[test]
fn runtime_vehicle_collider_uses_current_parent_transform_before_post_update() {
    let mut app = App::new();
    app.add_systems(Update, sync_runtime_attached_authored_collider_transforms);
    let parent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            // Deliberately stale, matching a root moved during Update
            // before Bevy's ordinary PostUpdate propagation.
            GlobalTransform::IDENTITY,
        ))
        .id();
    let collider = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "vehicle.collision.glb".to_owned(),
        vertices: vec![Vec3::ZERO, Vec3::X, Vec3::Y].into(),
        indices: vec![0, 1, 2].into(),
        local_min: Vec3::ZERO,
        local_max: Vec3::ONE,
        is_trigger: false,
    };
    let entity = app
        .world_mut()
        .spawn((
            ChildOf(parent),
            Transform::from_xyz(2.0, 0.0, 0.0),
            GlobalTransform::IDENTITY,
            collider,
            RuntimeAttachedAuthoredModelCollider,
        ))
        .id();

    app.update();

    let global = app.world().get::<GlobalTransform>(entity).unwrap();
    assert_eq!(global.translation(), Vec3::new(7.0, 0.0, 0.0));
    let bounds = app
        .world()
        .get::<AuthoredColliderWorldBounds>(entity)
        .unwrap();
    assert_eq!(bounds.minimum, Vec3::new(7.0, 0.0, 0.0));
    assert_eq!(bounds.maximum, Vec3::new(8.0, 1.0, 1.0));
    let motion = app
        .world()
        .get::<RuntimeAuthoredColliderMotion>(entity)
        .unwrap();
    assert_eq!(
        motion.previous_world_from_local,
        motion.current_world_from_local
    );
    assert_eq!(
        motion.current_world_from_local.transform_point3(Vec3::ZERO),
        Vec3::new(7.0, 0.0, 0.0),
        "initial materialization must not invent a sweep from identity"
    );

    app.world_mut()
        .get_mut::<Transform>(parent)
        .unwrap()
        .translation
        .x = 8.0;
    app.update();
    let motion = app
        .world()
        .get::<RuntimeAuthoredColliderMotion>(entity)
        .unwrap();
    assert_eq!(
        motion
            .previous_world_from_local
            .transform_point3(Vec3::ZERO),
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(
        motion.current_world_from_local.transform_point3(Vec3::ZERO),
        Vec3::new(10.0, 0.0, 0.0)
    );
}

#[test]
fn authored_triangle_ground_respects_origin_rotation_and_scale() {
    let collider = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/test.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
        ]
        .into(),
        indices: vec![0, 2, 1].into(),
        local_min: Vec3::ZERO,
        local_max: Vec3::new(2.0, 0.0, 2.0),
        is_trigger: false,
    };
    let transform = Transform::from_translation(Vec3::new(10.0, 7.0, -5.0))
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(2.0, 3.0, 4.0));
    let matrix = transform.to_matrix();
    let a = matrix.transform_point3(collider.vertices[0]);
    let b = matrix.transform_point3(collider.vertices[1]);
    let c = matrix.transform_point3(collider.vertices[2]);
    let centroid = (a + b + c) / 3.0;
    let height =
        collider_ground_height(&collider, matrix, centroid.x, centroid.z, 6.0, 8.0).unwrap();
    assert!((height - 7.0).abs() <= AUTHORED_COLLISION_EPSILON);
}

pub(super) fn step_sized_dimple_collider(rise: f32, half_width: f32) -> AuthoredTriMeshCollider {
    AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/step-sized-branch-dimple.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-half_width, rise, 0.0),
            Vec3::new(half_width, rise, 0.0),
        ]
        .into(),
        indices: vec![2, 1, 0, 0, 1, 3].into(),
        local_min: Vec3::new(-half_width, 0.0, -1.0),
        local_max: Vec3::new(half_width, rise, 1.0),
        is_trigger: false,
    }
}

#[test]
fn gameplay_camera_stops_at_the_first_authored_collider_crossing() {
    let wall = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/camera-wall.glb".to_owned(),
        vertices: vec![
            Vec3::new(-2.0, 0.0, -2.0),
            Vec3::new(-2.0, 3.0, -2.0),
            Vec3::new(2.0, 0.0, -2.0),
            Vec3::new(2.0, 3.0, -2.0),
        ]
        .into(),
        indices: vec![0, 1, 2, 2, 1, 3].into(),
        local_min: Vec3::new(-2.0, 0.0, -2.0),
        local_max: Vec3::new(2.0, 3.0, -2.0),
        is_trigger: false,
    };
    let wall_global = GlobalTransform::IDENTITY;
    let wall_bounds = AuthoredColliderWorldBounds::from_collider(&wall, &wall_global).unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                sync_authored_collider_spatial_index,
                crate::movement::update_legacy_camera_pose,
                resolve_authored_camera_occlusion,
            )
                .chain(),
        );
    let target = app
        .world_mut()
        .spawn((
            Transform::default(),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();
    let camera = app
        .world_mut()
        .spawn((LegacyOrbitCamera::new(target), Transform::default()))
        .id();
    app.world_mut().spawn((wall_global, wall, wall_bounds));

    app.update();

    let camera_transform = app.world().get::<Transform>(camera).unwrap();
    assert!(
        camera_transform
            .translation
            .abs_diff_eq(Vec3::new(0.0, 1.4, -2.0), 0.000_01),
        "camera crossed the wall: {}",
        camera_transform.translation
    );
    assert!((camera_transform.rotation * Vec3::NEG_Z).abs_diff_eq(Vec3::Z, 0.000_01));
}
