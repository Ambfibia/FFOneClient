use super::*;
use bevy::ecs::{system::RunSystemOnce, world::CommandQueue};

fn queue_unload(world: &mut World, root: Entity) {
    let mut queue = CommandQueue::default();
    queue_native_world_scene_unload(&mut Commands::new(&mut queue, world), root);
    queue.apply(world);
}

#[test]
fn repeated_unload_requests_preserve_progress_and_cancel_terrain() {
    let mut world = World::new();
    let root = world.spawn(PendingNativeWorldSceneSpawn::new(None)).id();
    for _ in 0..20 {
        world.spawn(ChildOf(root));
    }
    queue_unload(&mut world, root);
    unload_native_world_scene_batch(&mut world, 3, None);
    let progress = world
        .get::<PendingNativeWorldSceneUnload>(root)
        .unwrap()
        .traversal[0]
        .next_child;
    queue_unload(&mut world, root);
    assert_eq!(
        world
            .get::<PendingNativeWorldSceneUnload>(root)
            .unwrap()
            .traversal[0]
            .next_child,
        progress
    );
    assert!(world.get::<PendingNativeWorldSceneSpawn>(root).is_none());
    while world.get_entity(root).is_ok() {
        queue_unload(&mut world, root);
        unload_native_world_scene_batch(&mut world, 3, None);
    }
    assert_eq!(game_entity_count(&world), 0);
}

#[test]
fn unload_respects_late_children_and_reparented_subtrees() {
    let mut world = World::new();
    let survivor = world.spawn_empty().id();
    let root = world.spawn(PendingNativeWorldSceneUnload::default()).id();
    let branch = world.spawn(ChildOf(root)).id();
    for _ in 0..10 {
        world.spawn(ChildOf(branch));
    }
    unload_native_world_scene_batch(&mut world, 1, None);
    world.entity_mut(branch).insert(ChildOf(survivor));
    let late = world.spawn(ChildOf(root)).id();
    for _ in 0..10 {
        world.spawn(ChildOf(late));
    }
    while world.get_entity(root).is_ok() {
        let before = game_entity_count(&world);
        unload_native_world_scene_batch(&mut world, 2, None);
        assert!(before - game_entity_count(&world) <= 2);
    }
    assert_eq!(world.get::<Children>(branch).unwrap().len(), 9);
    assert_eq!(game_entity_count(&world), 11);
}

#[test]
fn unload_time_budget_still_makes_progress() {
    let mut world = World::new();
    let root = world.spawn(PendingNativeWorldSceneUnload::default()).id();
    for _ in 0..100 {
        world.spawn(ChildOf(root));
    }
    unload_native_world_scene_batch(&mut world, 256, Some(std::time::Duration::ZERO));
    assert_eq!(game_entity_count(&world), 100);
}

#[test]
fn unload_bounds_deep_descent_before_first_leaf() {
    let mut world = World::new();
    let root = world.spawn(PendingNativeWorldSceneUnload::default()).id();
    let mut parent = root;
    for _ in 0..100 {
        parent = world.spawn(ChildOf(parent)).id();
    }
    unload_native_world_scene_batch(&mut world, 2, None);
    assert_eq!(game_entity_count(&world), 101);
    assert!(
        world
            .get::<PendingNativeWorldSceneUnload>(root)
            .unwrap()
            .traversal
            .len()
            <= 9
    );
    for _ in 0..100 {
        unload_native_world_scene_batch(&mut world, 2, None);
    }
    assert_eq!(game_entity_count(&world), 0);
}

fn streaming_app(with_terrain: bool) -> App {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut catalog = load_native_world_scenes(&asset_root).unwrap();
    // Only native scene selection is needed; pending I/O is supplied explicitly
    // so this test is deterministic even on a very fast or very slow disk.
    if !with_terrain {
        for scene in &mut catalog.scenes {
            scene.native_terrain = None;
        }
    }
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(catalog)
        .init_resource::<NativeWorldStreamingStatus>()
        .add_systems(Update, stream_native_world_dongs);
    let target = app
        .world_mut()
        .spawn(Transform::from_xyz(-6374.3188, -56.4746, 668.2039))
        .id();
    app.world_mut().spawn(LegacyOrbitCamera::new(target));
    app
}

#[test]
fn neighbor_admission_does_not_wait_for_previous_terrain_or_duplicate_roots() {
    let mut app = streaming_app(false);
    app.update();
    let first = app
        .world_mut()
        .query_filtered::<Entity, With<NativeWorldSceneRoot>>()
        .single(app.world())
        .unwrap();
    let task = IoTaskPool::get().spawn(async { std::future::pending().await });
    app.world_mut()
        .entity_mut(first)
        .insert(PendingNativeWorldSceneTerrainLoad { task });
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&NativeWorldSceneRoot>()
            .iter(app.world())
            .count(),
        2
    );
    for _ in 0..12 {
        app.update();
    }
    let roots = app
        .world_mut()
        .query::<&NativeWorldSceneRoot>()
        .iter(app.world())
        .map(|root| root.tile)
        .collect::<Vec<_>>();
    assert!(roots.len() <= NATIVE_WORLD_MAX_RESIDENT_TILES);
    assert_eq!(
        roots.len(),
        roots.iter().copied().collect::<HashSet<_>>().len()
    );
    queue_unload(app.world_mut(), first);
    assert!(
        app.world()
            .get::<PendingNativeWorldSceneTerrainLoad>(first)
            .is_none()
    );
}

#[test]
#[ignore = "opt-in CPU streaming benchmark; writes target/performance"]
fn streaming_unload_benchmark() {
    let mut reports = Vec::new();
    for budget in [32, NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME] {
        let mut world = World::new();
        let root = world.spawn(PendingNativeWorldSceneUnload::default()).id();
        // Wide placement list plus nested pass/behaviour entities.
        for _ in 0..6400 {
            let visual = world
                .spawn((ChildOf(root), Transform::default(), Visibility::Hidden))
                .id();
            world.spawn((ChildOf(visual), Transform::default(), Visibility::Hidden));
        }
        let mut frames = Vec::new();
        while world.get_entity(root).is_ok() {
            let started = std::time::Instant::now();
            unload_native_world_scene_batch(
                &mut world,
                budget,
                Some(std::time::Duration::from_millis(1)),
            );
            frames.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        let count = frames.len();
        let total: f64 = frames.iter().sum();
        frames.sort_by(f64::total_cmp);
        reports.push(serde_json::json!({"entityBudget": budget, "entities": 12801, "frames": count, "cpuTotalMs": total, "p95Ms": frames[(count - 1) * 95 / 100], "maxMs": frames[count - 1]}));
    }
    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/performance/tile-streaming");
    fs::create_dir_all(&output).unwrap();
    let json = serde_json::to_string_pretty(&reports).unwrap();
    fs::write(output.join("unload-cpu.json"), &json).unwrap();
    println!("{json}");
}

#[test]
fn terrain_io_reserves_capacity_and_cancels_on_unload() {
    let mut app = streaming_app(true);
    let mut pending = Vec::new();
    for _ in 0..NATIVE_WORLD_TERRAIN_LOADS_IN_FLIGHT {
        let task = IoTaskPool::get().spawn(async { std::future::pending().await });
        pending.push(
            app.world_mut()
                .spawn(PendingNativeWorldSceneTerrainLoad { task })
                .id(),
        );
    }
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&NativeWorldSceneRoot>()
            .iter(app.world())
            .count(),
        0
    );
    queue_unload(app.world_mut(), pending[0]);
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&NativeWorldSceneRoot>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query::<&PendingNativeWorldSceneTerrainLoad>()
            .iter(app.world())
            .count(),
        NATIVE_WORLD_TERRAIN_LOADS_IN_FLIGHT
    );
}

#[test]
fn in_flight_visual_limit_includes_roots_with_completed_admission() {
    let mut app = streaming_app(false);
    app.update();
    let root = app
        .world_mut()
        .query_filtered::<Entity, With<NativeWorldSceneRoot>>()
        .single(app.world())
        .unwrap();
    let scene_root = app
        .world()
        .get::<NativeWorldSceneRoot>(root)
        .unwrap()
        .clone();
    {
        let mut catalog = app.world_mut().resource_mut::<NativeWorldCatalog>();
        let scene = catalog
            .scenes
            .iter_mut()
            .find(|scene| {
                scene.name == scene_root.name
                    && scene.scope == Some(scene_root.scope)
                    && scene.tile == scene_root.tile
            })
            .unwrap();
        scene.colliders.clear();
        assert!(!scene.visuals.is_empty());
    }
    app.add_plugins(AssetPlugin::default());
    for _ in 0..NATIVE_WORLD_MAX_IN_FLIGHT_VISUAL_ASSETS {
        // Outstanding requests belonging to already fully admitted roots were
        // omitted by the old pending-roots/ready-children subtraction.
        app.world_mut().spawn(PendingNativeWorldVisualAsset {
            gltf: Handle::default(),
            mesh: Handle::default(),
        });
    }
    app.world_mut()
        .run_system_once(materialize_pending_native_world_scene_spawns)
        .unwrap();
    assert_eq!(
        app.world()
            .get::<PendingNativeWorldSceneSpawn>(root)
            .unwrap()
            .next_visual,
        0
    );
}

#[test]
fn unsorted_stream_selection_matches_sorted_reference_at_edges_and_ties() {
    let app = streaming_app(false);
    let catalog = app.world().resource::<NativeWorldCatalog>();
    for scope in [NativeWorldScope::WorldMap, NativeWorldScope::Tutorial] {
        for z in 0..16 {
            for x in 0..16 {
                for offset in [0.0, 256.0, 356.0, 511.9] {
                    let position =
                        Vec3::new(-(x as f32 * 512.0 + offset), 0.0, z as f32 * 512.0 + offset);
                    let mut candidates = catalog
                        .scenes
                        .iter()
                        .filter(|scene| catalog.scene_is_selected_for_scope(scope, scene))
                        .collect::<Vec<_>>();
                    candidates.sort_by_key(|scene| (scene.tile[1], scene.tile[0]));
                    let mut loaded = HashSet::new();
                    for _ in 0..3 {
                        let mut distance = EXTENDED_DONG_LOAD_DISTANCE_NATIVE.powi(2);
                        let mut expected = None;
                        for scene in &candidates {
                            if loaded.contains(&scene.tile) {
                                continue;
                            }
                            if let Some(value) =
                                catalog.presentation_squared_distance(scope, position, scene)
                            {
                                if value < distance {
                                    distance = value;
                                    expected = Some(scene.tile);
                                }
                            }
                        }
                        let actual = catalog
                            .next_legacy_stream_load(scope, position, &loaded)
                            .map(|scene| scene.tile);
                        assert_eq!(actual, expected);
                        if let Some(tile) = actual {
                            loaded.insert(tile);
                        }
                    }
                }
            }
        }
    }
}

// Resources became entities in Bevy 0.19; unload budgets count scene entities.
fn game_entity_count(world: &World) -> u32 {
    world.entity_count() - world.resource_entities().iter().count() as u32
}
