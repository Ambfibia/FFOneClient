use super::*;

pub(super) fn game_entity_count(world: &World) -> u32 {
    world.entity_count() - world.resource_entities().iter().count() as u32
}

#[test]
fn water_occlusion_respects_close_floors_edges_triggers_and_submersion() {
    for indexed in [false, true] {
        for (floor_y, x, avatar_y, trigger, expected) in [
            (0.1, 0.0, 0.1, false, true),  // Dry floor inside water proximity band.
            (0.1, 0.0, 0.3, false, true),  // Jumping above the same floor.
            (0.1, 2.0, 0.1, false, false), // Beyond the actual floor edge.
            (0.1, 0.0, 0.1, true, false),
            (0.0, 0.0, 0.1, false, false), // Water's own collision sheet.
            (-0.1, 0.0, 0.1, false, false), // Submerged floor.
            (0.1, 0.0, -1.0, false, false), // Initial submersion.
            (0.3, 0.0, 0.1, false, false), // Floor above the feet.
        ] {
            let mut world = World::new();
            let mut collider = closed_vehicle_test_collider();
            collider.is_trigger = trigger;
            let global = GlobalTransform::from_translation(Vec3::new(0.0, floor_y - 2.0, 0.0));
            let bounds =
                AuthoredColliderWorldBounds::from_collider(&collider, &global).unwrap();
            let entity = world.spawn((global, collider, bounds.clone())).id();
            if indexed {
                let mut index = AuthoredColliderSpatialIndex::default();
                index.insert(entity, &bounds);
                world.insert_resource(index);
            }
            let mut state =
                bevy::ecs::system::SystemState::<NativeWaterOcclusion>::new(&mut world);
            assert_eq!(
                state
                    .get_mut(&mut world)
                    .unwrap()
                    .blocks(0.0, Vec3::new(x, avatar_y, 0.0)),
                expected,
                "indexed={indexed}, floor={floor_y}, x={x}, avatar={avatar_y}, trigger={trigger}"
            );
        }
    }
}

#[test]
fn city_hall_water_occlusion_uses_production_sloped_surfaces_above_water() {
    let scene: NativeWorldScene = serde_json::from_slice(
        &fs::read(asset_root().join("map/tiles/map_03_05/scene.json")).unwrap(),
    )
    .unwrap();
    let root = scene.root.try_to_bevy("root").unwrap().to_matrix();
    let water = scene
        .visuals
        .iter()
        .find(|visual| native_world_water_surface(&scene, visual).is_some())
        .unwrap();
    let (vertices, indices) =
        authored_gltf_mesh_data(&scene.model(&water.model).unwrap().path, Some(0));
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices
            .iter()
            .map(|vertex| vertex.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_indices(Indices::U32(indices));
    let water_global =
        GlobalTransform::from(root * water.transform.try_to_bevy("water").unwrap().to_matrix());
    let mut world = World::new();
    let mut index = AuthoredColliderSpatialIndex::default();
    for authored in &scene.colliders {
        let model = scene.model(&authored.model).unwrap();
        if model.root_name == "ffWater" || model.root_name == "ffPoison" {
            continue;
        }
        let mut collider = authored_collider_from_glb_mesh(&model.path, authored.mesh);
        collider.is_trigger = authored.is_trigger;
        let global = GlobalTransform::from(
            root * authored
                .transform
                .try_to_bevy("collider")
                .unwrap()
                .to_matrix(),
        );
        let bounds = AuthoredColliderWorldBounds::from_collider(&collider, &global).unwrap();
        let entity = world.spawn((global, collider, bounds)).id();
        index.insert(entity, &bounds);
    }
    world.insert_resource(index);
    let mut state = bevy::ecs::system::SystemState::<NativeWaterOcclusion>::new(&mut world);
    // Actual sloped building triangle centers at City Hall. Both surfaces
    // are less than 0.2 units above water, inside the old 0.5-unit band.
    // Their normals have y < 0.2: walkable-floor queries miss the barrier.
    for position in [
        Vec3::new(-1845.7359, -57.24495, 2720.1978),
        Vec3::new(-1722.0511, -57.24612, 2766.8975),
    ] {
        let height = crate::legacy_environment::legacy_water_surface_height(
            &mesh,
            &water_global,
            position.x,
            position.z,
        )
        .unwrap();
        assert!(crate::legacy_environment::legacy_water_surface_contact(
            height, position.y
        ));
        assert!(
            state.get_mut(&mut world).unwrap().blocks(height, position),
            "{position:?}"
        );
    }
}

#[test]
fn darktree_bridge_primary_guard_production_spawn_stays_a_standard_rendered_visual() {
    let scene_path = asset_root().join("map/tiles/map_12_02/scene.json");
    let scene: NativeWorldScene =
        serde_json::from_slice(&fs::read(scene_path).unwrap()).unwrap();
    let visuals = scene
        .visuals
        .iter()
        .enumerate()
        .filter(|(_, visual)| visual.name.contains("#9439") || visual.name.contains("#9442"))
        .collect::<Vec<_>>();
    assert_eq!(
        visuals.len(),
        2,
        "both audited Pokey Oaks dark-tree bridge renderers"
    );
    for (_, visual) in &visuals {
        assert_eq!(visual.legacy_layer, Some(12));
        assert!(!is_legacy_collision_helper_visual(&scene, visual));
        assert!(!is_legacy_zero_contribution_visual(&scene, visual));
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Gltf>().init_asset::<GltfMesh>();
    let root = app.world_mut().spawn_empty().id();
    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut queue = CommandQueue::default();
    let bridges = {
        let mut commands = Commands::new(&mut queue, app.world());
        visuals
            .iter()
            .map(|(visual_index, visual)| {
                spawn_native_world_visual(
                    &mut commands,
                    &asset_server,
                    &scene,
                    root,
                    *visual_index,
                    visual,
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    };
    queue.apply(app.world_mut());

    for bridge in bridges {
        let entity = app.world().entity(bridge);
        assert!(entity.contains::<PendingNativeWorldVisualAsset>());
        assert!(entity.contains::<NativeWorldObjectRangeMember>());
        assert!(entity.contains::<NativeWorldPrimaryFarPresentationGuard>());
        assert!(!entity.contains::<NativeWorldObjectRangeUnbounded>());
        assert_eq!(entity.get::<Visibility>(), Some(&Visibility::Hidden));
    }
}

#[test]
fn exact_additive_black_primary_plane_is_metadata_only() {
    let scene_path = asset_root().join("map/tiles/map_08_06/scene.json");
    let scene: NativeWorldScene =
        serde_json::from_slice(&fs::read(scene_path).unwrap()).unwrap();
    let (visual_index, visual) = scene
        .visuals
        .iter()
        .enumerate()
        .find(|(_, visual)| is_legacy_zero_contribution_visual(&scene, visual))
        .expect("Candy Cove must retain the audited additive-black source metadata");
    let model = scene.model(&visual.model).unwrap();
    assert_eq!(model.id, LEGACY_ZERO_CONTRIBUTION_MODEL_ID);
    assert_eq!(model.path, LEGACY_ZERO_CONTRIBUTION_MODEL_PATH);
    assert_eq!(model.blake3, LEGACY_ZERO_CONTRIBUTION_MODEL_BLAKE3);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    let root = app.world_mut().spawn_empty().id();
    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut queue = CommandQueue::default();
    let entity = {
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

    let entity = app.world().entity(entity);
    assert!(entity.contains::<NativeWorldVisualSceneReady>());
    assert_eq!(entity.get::<Visibility>(), Some(&Visibility::Hidden));
    assert!(!entity.contains::<PendingNativeWorldVisualAsset>());
    assert!(!entity.contains::<NativeWorldObjectRangeMember>());
    assert!(!entity.contains::<Mesh3d>());
}

#[test]
fn wall_spatial_query_keeps_candidates_reachable_only_after_a_slide() {
    let candidate = Entity::from_bits(7);
    let mut index = AuthoredColliderSpatialIndex::default();
    index.insert(
        candidate,
        &AuthoredColliderWorldBounds {
            minimum: Vec3::new(0.0, -2.0, 39.0),
            maximum: Vec3::new(2.0, 5.0, 41.0),
        },
    );

    let previous = Vec3::new(1.0, 2.0, 16.0);
    let current = Vec3::new(41.0, 2.0, 16.0);
    let radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;

    // The superseded segment AABB only touched the z=0 spatial row and
    // dropped the z=1 candidate before the exact solver could redirect the
    // movement into it.
    let old_minimum = previous.min(current) - Vec3::splat(radius);
    let old_maximum = previous.max(current)
        + Vec3::new(
            radius,
            AUTHORED_CHARACTER_CONTROLLER_HEIGHT + radius,
            radius,
        );
    let mut candidates = Vec::new();
    index.candidates(old_minimum, old_maximum, &mut candidates);
    assert!(!candidates.contains(&candidate));

    let (minimum, maximum) = authored_wall_spatial_query_bounds(previous, current);
    index.candidates(minimum, maximum, &mut candidates);
    assert_eq!(candidates, vec![candidate]);
    let reach = previous.distance(current) + radius;
    assert_eq!(
        minimum,
        Vec3::new(previous.x - reach, 2.0 - radius, previous.z - reach)
    );
    assert_eq!(
        maximum,
        Vec3::new(
            previous.x + reach,
            2.0 + AUTHORED_CHARACTER_CONTROLLER_HEIGHT + radius,
            previous.z + reach,
        )
    );
}

pub(super) fn published_character_node_world_transform(
    relative: &str,
    node_name: &str,
    table_scale: f32,
) -> Mat4 {
    let gltf = gltf::Gltf::open(join_relative(&asset_root(), relative)).unwrap();
    let nodes = gltf.nodes().collect::<Vec<_>>();
    let mut parents = vec![None; nodes.len()];
    for node in &nodes {
        for child in node.children() {
            parents[child.index()] = Some(node.index());
        }
    }
    let mut node = nodes
        .iter()
        .find(|node| node.name() == Some(node_name))
        .expect("published character must retain the accepted collider node")
        .index();
    let mut chain = vec![node];
    while let Some(parent) = parents[node] {
        chain.push(parent);
        node = parent;
    }
    chain.reverse();
    chain.into_iter().fold(
        Mat4::from_scale(Vec3::splat(table_scale)),
        |world_from_parent, node| {
            world_from_parent * Mat4::from_cols_array_2d(&nodes[node].transform().matrix())
        },
    )
}

pub(super) fn first_scene() -> NativeWorldScene {
    let root = asset_root();
    let catalog = NativeWorldCatalog::open(&root).unwrap();
    let mut scene = catalog
        .scenes()
        .iter()
        .find(|scene| scene.scope == Some(NativeWorldScope::WorldMap) && scene.tile == [8, 6])
        .cloned()
        .unwrap();
    let terrain = catalog
        .load_terrain(scene.native_terrain.as_ref().unwrap())
        .unwrap();
    scene.native_terrain.as_mut().unwrap().loaded = Some(terrain);
    scene
}

pub(super) fn remove_key_recursively(value: &mut serde_json::Value, key: &str) {
    match value {
        serde_json::Value::Object(object) => {
            object.remove(key);
            for child in object.values_mut() {
                remove_key_recursively(child, key);
            }
        }
        serde_json::Value::Array(array) => {
            for child in array {
                remove_key_recursively(child, key);
            }
        }
        _ => {}
    }
}

#[test]
fn reveal_world_scene_system_has_disjoint_visibility_queries() {
    let mut app = App::new();
    init_world_presentation_assets(&mut app);
    app.add_systems(Update, reveal_native_world_scenes);
    app.update();
}

#[test]
fn world_presentation_cannot_finish_before_stream_admission() {
    let mut app = App::new();
    init_world_presentation_assets(&mut app);
    app.add_systems(Update, reveal_native_world_scenes);
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "streamed".to_owned(),
                tile: [8, 6],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldBehaviourStatus::Ready,
            NativeWorldVisualPresentationStatus::Loading,
            NativeWorldPresentationStatus::Loading,
            Visibility::Hidden,
            PendingNativeWorldSceneSpawn::new(None),
        ))
        .id();
    let visual = app
        .world_mut()
        .spawn((
            ChildOf(root),
            SpawnedNativeWorldVisual {
                model_path: "models/streamed.glb".to_owned(),
                source_model_path: "models/streamed.glb".to_owned(),
                scene: 0,
            },
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldVisualSceneReady,
            NativeWorldObjectRangeReady,
            NativeWorldObjectRangeUnbounded,
            Visibility::Hidden,
        ))
        .id();

    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Loading)
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden)
    );

    app.world_mut()
        .entity_mut(root)
        .remove::<PendingNativeWorldSceneSpawn>();
    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Ready)
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Inherited)
    );
}

#[test]
fn multipart_meshes_keep_distinct_aabbs_but_share_range_and_residency() {
    let mut app = App::new();
    app.init_resource::<NativeWorldObjectRangeCameraCache>()
        .add_systems(
            Update,
            (
                propagate_native_world_object_ranges_to_late_meshes,
                update_native_world_object_residency
                    .after(propagate_native_world_object_ranges_to_late_meshes),
            ),
        );
    app.world_mut().spawn((
        Camera3d::default(),
        LegacyOrbitCamera::new(Entity::PLACEHOLDER),
        Transform::from_xyz(-3000.0, 0.0, 4500.0),
    ));
    let scene_root = app.world_mut().spawn_empty().id();
    let tag = pack_native_world_object_range(Vec3::new(-3500.0, -50.0, 4500.0), 100.0).unwrap();
    let visual = app
        .world_mut()
        .spawn((
            NativeWorldObjectRangeMember {
                scene_root,
                group: NativeWorldObjectRangeGroupKey::Prefab(Arc::from("owner")),
            },
            NativeWorldObjectRangeContract {
                tag,
                meshes: Vec::new(),
                visible: true,
            },
        ))
        .id();
    let first_aabb = Aabb::from_min_max(Vec3::new(-4.0, -1.0, -2.0), Vec3::ZERO);
    let second_aabb = Aabb::from_min_max(Vec3::X * 20.0, Vec3::new(24.0, 8.0, 3.0));
    let first = app
        .world_mut()
        .spawn((ChildOf(visual), Mesh3d::default(), first_aabb))
        .id();
    let second = app
        .world_mut()
        .spawn((ChildOf(visual), Mesh3d::default(), second_aabb))
        .id();

    app.update();
    assert_eq!(app.world().get::<Aabb>(first), Some(&first_aabb));
    assert_eq!(app.world().get::<Aabb>(second), Some(&second_aabb));
    assert_eq!(app.world().get::<MeshTag>(first), Some(&MeshTag(tag)));
    assert_eq!(app.world().get::<MeshTag>(second), Some(&MeshTag(tag)));
    assert_eq!(
        app.world().get::<Visibility>(first),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(second),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn complete_range_group_retries_finalization_without_rescanning_pending_groups() {
    let mut app = App::new();
    app.add_systems(Update, prepare_native_world_object_ranges);
    let key = NativeWorldObjectRangeGroupKey::Visual(0);
    let scene_root = app.world_mut().spawn_empty().id();
    let visual = app
        .world_mut()
        .spawn(NativeWorldObjectRangeMember {
            scene_root,
            group: key.clone(),
        })
        .id();
    app.world_mut()
        .entity_mut(scene_root)
        .insert(NativeWorldObjectRangeGroups {
            pending: HashMap::from([(
                key.clone(),
                NativeWorldPendingRangeGroup {
                    expected_members: 1,
                    members: vec![NativeWorldPreparedRangeMember {
                        visual_root: visual,
                        local_minimum: Vec3::splat(-1.0),
                        local_maximum: Vec3::splat(1.0),
                        meshes: Vec::new(),
                    }],
                },
            )]),
            ready_to_finalize: VecDeque::from([key.clone()]),
            unbounded: HashSet::new(),
        });

    // Missing Transform is an ordinary one-frame readiness race. The
    // complete key remains queued instead of being lost or rediscovered by
    // a full pending-map scan.
    app.update();
    let groups = app
        .world()
        .get::<NativeWorldObjectRangeGroups>(scene_root)
        .unwrap();
    assert!(groups.pending.contains_key(&key));
    assert_eq!(groups.ready_to_finalize, VecDeque::from([key.clone()]));
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeReady>(visual)
            .is_none()
    );

    app.world_mut()
        .entity_mut(visual)
        .insert(Transform::from_xyz(-1000.0, 0.0, 1000.0));
    app.update();
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeReady>(visual)
            .is_some()
    );
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeContract>(visual)
            .is_some()
    );
    let groups = app
        .world()
        .get::<NativeWorldObjectRangeGroups>(scene_root)
        .unwrap();
    assert!(!groups.pending.contains_key(&key));
    assert!(groups.ready_to_finalize.is_empty());
}

#[test]
fn first_scene_preserves_native_tile_origin_and_exact_static_scene() {
    let scene = first_scene();
    assert_eq!(scene.tile, [8, 6]);
    let root = scene.root.try_to_bevy("root").unwrap();
    assert_eq!(root.translation, Vec3::ZERO);
    assert_eq!(root.rotation, Quat::IDENTITY);
    assert_eq!(root.scale, Vec3::ONE);
    assert!(!scene.coordinate_contract.gameplay_facing_rotation_applied);

    assert_eq!(scene.coverage, "native-heightmap+exact-static-scene");
    assert!(!scene.models.is_empty());
    assert!(!scene.visuals.is_empty());
    assert!(!scene.colliders.is_empty());
    let terrain = scene.native_terrain.as_ref().unwrap();
    assert_eq!(
        terrain.transform.try_to_bevy("terrain").unwrap(),
        Transform::from_xyz(0.0, -300.0, 0.0)
    );
    assert!(
        scene
            .terrain_origin()
            .unwrap()
            .abs_diff_eq(Vec3::new(-4096.0, -300.0, 3072.0), f32::EPSILON),
        "serialized Map_08_06 rootChain and terrain owner transform must be composed exactly once"
    );
}

#[test]
fn tile_membership_uses_reflected_native_x_and_unswapped_z() {
    let scene = first_scene();
    assert!(scene.contains_native_xz(Vec3::new(-4096.0, 10.0, 3072.0)));
    assert!(scene.contains_native_xz(Vec3::new(-4607.99, -30.0, 3583.99)));
    assert!(!scene.contains_native_xz(Vec3::new(-4608.0, -30.0, 3584.0)));
    assert!(!scene.contains_native_xz(Vec3::new(4096.0, 0.0, 3072.0)));
    assert!(!scene.contains_native_xz(Vec3::new(-4096.0, 0.0, -3072.0)));
}

#[test]
fn legacy_dong_stream_distance_reflects_x_and_preserves_source_aabb_edges() {
    let tile = [12, 3];
    assert_eq!(
        legacy_dong_squared_distance_native(Vec3::new(-6400.0, 0.0, 1792.0), tile),
        Some(0.0)
    );
    assert_eq!(
        legacy_dong_squared_distance_native(Vec3::new(-6000.0, 0.0, 1792.0), tile),
        Some(144.0 * 144.0)
    );
    assert_eq!(
        legacy_dong_squared_distance_native(Vec3::new(-(6144.0 - 280.0), 0.0, 1792.0), tile),
        Some(280.0 * 280.0)
    );
    assert_eq!(
        legacy_dong_squared_distance_native(Vec3::new(f32::NAN, 0.0, 0.0), tile),
        None
    );
}

#[test]
fn legacy_dong_stream_loads_one_nearest_tile_in_source_scan_order() {
    assert_eq!(LEGACY_DONG_LOAD_DISTANCE_NATIVE, 280.0);
    assert_eq!(EXTENDED_DONG_LOAD_DISTANCE_NATIVE, 356.0);
    assert_eq!(LEGACY_DONG_UNLOAD_DISTANCE_NATIVE, 340.0);
    assert_eq!(EXTENDED_DONG_UNLOAD_DISTANCE_NATIVE, 420.0);
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let position = Vec3::new(-6400.0, 0.0, 1792.0);
    let mut loaded = HashSet::new();
    let expected = [[12, 3], [12, 2], [11, 3], [13, 3], [12, 4]];
    for tile in expected {
        let next = catalog
            .next_legacy_stream_load(NativeWorldScope::WorldMap, position, &loaded)
            .unwrap();
        assert_eq!(next.tile, tile);
        loaded.insert(tile);
    }
    assert!(
        catalog
            .next_legacy_stream_load(NativeWorldScope::WorldMap, position, &loaded)
            .is_none(),
        "diagonal tiles remain outside the conservative 356-unit extended load radius"
    );
}

#[test]
fn published_presentation_overflow_envelope_is_bounded_and_audited() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let mut raw_overflow = 0_usize;
    let mut raw_over_320 = 0_usize;
    let mut raw_over_356 = 0_usize;
    let mut render_overflow = 0_usize;
    let mut render_over_320 = 0_usize;
    let mut render_over_356 = 0_usize;
    let mut maximum = 0.0_f32;
    for scene in catalog.scenes() {
        if scene.scope != Some(NativeWorldScope::WorldMap) {
            continue;
        }
        for visual in &scene.visuals {
            let center = Vec3::from_array(visual.transform.translation);
            let distance = legacy_dong_squared_distance_native(center, scene.tile)
                .unwrap()
                .sqrt();
            if distance <= 0.0 {
                continue;
            }
            raw_overflow += 1;
            raw_over_320 += usize::from(distance > 320.0);
            raw_over_356 += usize::from(distance > 356.0);
            if is_legacy_non_presenting_visual(scene, visual) {
                continue;
            }
            render_overflow += 1;
            render_over_320 += usize::from(distance > 320.0);
            render_over_356 += usize::from(distance > 356.0);
            maximum = maximum.max(distance);
        }
    }
    // Raw census includes 87 non-rendering collision_alpha helpers. The
    // runtime footprint deliberately excludes those sheets.
    assert_eq!((raw_overflow, raw_over_320, raw_over_356), (562, 25, 24));
    assert_eq!(
        (render_overflow, render_over_320, render_over_356),
        (475, 24, 24)
    );
    assert!((maximum - 555.145_4).abs() < 0.001);
    assert_eq!(
        catalog
            .presentation_footprints
            .values()
            .map(|footprint| footprint.overflow_centers.len())
            .sum::<usize>(),
        render_overflow
    );
}

#[test]
fn published_water_surfaces_do_not_use_center_distance_culling() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let mut water_visuals = 0_usize;
    let mut initially_unbounded = 0_usize;
    for scene in catalog.scenes() {
        let groups = NativeWorldObjectRangeGroups::from_scene(scene);
        initially_unbounded += groups.unbounded.len();
        for (visual_index, visual) in scene.visuals.iter().enumerate() {
            if !is_native_world_water_visual(scene, visual) {
                continue;
            }
            water_visuals += 1;
            let key = native_world_object_range_group_key(visual_index, &visual.name);
            assert!(groups.unbounded.contains(&key));
            assert!(!groups.pending.contains_key(&key));
        }
    }
    assert_eq!(
        water_visuals, 141,
        "published ffWater plus ffPoison visual census"
    );
    assert_eq!(initially_unbounded, water_visuals);

    // Primary map_08_06 publishes an almost tile-wide surface with exact
    // world AABB X=-4592..-4096, Z=3072..3584. The former adaptive policy
    // used the AABB center and a 340-unit cap: a point at the visible far
    // corner is still on the water bounds but lies ~356 units from that
    // center, so a resident neighbor could disappear as one straight
    // sheet. The water-only unbounded group leaves that decision to the
    // renderer AABB/frustum instead.
    let scene = catalog
        .scenes()
        .iter()
        .find(|scene| scene.tile == [8, 6])
        .expect("published map_08_06 scene");
    let water = scene
        .visuals
        .iter()
        .find(|visual| is_native_world_water_visual(scene, visual))
        .expect("published map_08_06 ffWater visual");
    assert_eq!(
        scene.model(&water.model).map(|model| model.path.as_str()),
        Some("objects/unclassified/ffpoison/models/ffwater_variant_0011/visual.glb")
    );
    let old_center = Vec3::new(-4_344.0, -53.123_53, 3_328.0);
    let visible_water_corner = Vec3::new(-4_592.0, -58.062_428, 3_584.0);
    assert!(
        visible_water_corner.distance(old_center) > EXTENDED_WORLD_CAMERA_FAR_NATIVE,
        "the regression fixture must exercise the former center-distance seam"
    );
}

#[test]
fn production_water_visual_spawn_marks_environment_contact_authority() {
    for (scene_path, root_name, infected) in [
        ("map/tiles/map_08_06/scene.json", "ffWater", false),
        ("map/tiles/map_00_01/scene.json", "ffPoison", true),
    ] {
        let scene: NativeWorldScene =
            serde_json::from_slice(&fs::read(asset_root().join(scene_path)).unwrap()).unwrap();
        let (visual_index, visual) = scene
            .visuals
            .iter()
            .enumerate()
            .find(|(_, visual)| {
                scene
                    .model(&visual.model)
                    .is_some_and(|model| model.root_name == root_name)
                    && native_world_water_surface(&scene, visual).is_some()
            })
            .expect("published water visual");

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Gltf>().init_asset::<GltfMesh>();
        let root = app.world_mut().spawn_empty().id();
        let asset_server = app.world().resource::<AssetServer>().clone();
        let mut queue = CommandQueue::default();
        let water_entity = {
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

        assert_eq!(
            app.world()
                .entity(water_entity)
                .get::<crate::legacy_environment::LegacyWaterSurface>(),
            Some(&crate::legacy_environment::LegacyWaterSurface { infected }),
            "{root_name} must reach update_legacy_avatar_environment"
        );
    }
}
