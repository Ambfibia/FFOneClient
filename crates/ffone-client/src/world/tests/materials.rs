use super::*;

#[test]
fn near_direct_mesh_stays_hidden_until_exact_material_presentation_is_ready() {
    let mut app = App::new();
    init_world_presentation_assets(&mut app);
    app.init_resource::<NativeWorldObjectRangeCameraCache>()
        .add_systems(
            Update,
            (
                update_native_world_object_residency,
                reveal_native_world_scenes,
            )
                .chain(),
        );
    let range_center = Vec3::new(-3500.0, -50.0, 4500.0);
    app.world_mut().spawn((
        Camera3d::default(),
        LegacyOrbitCamera::new(Entity::PLACEHOLDER),
        Transform::from_translation(range_center),
    ));
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "direct-range-gate".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldBehaviourStatus::Ready,
            NativeWorldVisualPresentationStatus::Loading,
            NativeWorldPresentationStatus::Loading,
            Visibility::Inherited,
        ))
        .id();
    let tag = pack_native_world_object_range(range_center, 100.0).unwrap();
    let visual = app
        .world_mut()
        .spawn((
            ChildOf(root),
            SpawnedNativeWorldVisual {
                model_path: "models/direct.glb".to_owned(),
                source_model_path: "models/direct.glb".to_owned(),
                scene: 0,
            },
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldVisualSceneReady,
            NativeWorldObjectRangeReady,
            NativeWorldObjectRangeContract {
                tag,
                meshes: Vec::new(),
                visible: false,
            },
            Mesh3d::default(),
            MeshTag(tag),
            GltfMaterialExtras::default(),
            Visibility::Hidden,
        ))
        .id();
    app.world_mut()
        .get_mut::<NativeWorldObjectRangeContract>(visual)
        .unwrap()
        .meshes
        .push(visual);

    app.update();
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeContract>(visual)
            .unwrap()
            .visible,
        "logical residency must remember that the direct mesh is near"
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden),
        "the glTF fallback must stay hidden while exact material application is pending"
    );
    assert_eq!(
        app.world().get::<NativeWorldVisualPresentationStatus>(root),
        Some(&NativeWorldVisualPresentationStatus::Loading)
    );

    app.world_mut()
        .entity_mut(visual)
        .insert(LegacyMaterialApplied);
    app.update();
    assert_eq!(
        app.world().get::<NativeWorldVisualPresentationStatus>(root),
        Some(&NativeWorldVisualPresentationStatus::Ready)
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Inherited),
        "the remembered near contract must reveal only after exact material readiness"
    );
}

#[test]
fn metadata_error_never_reveals_the_standard_material_fallback() {
    let mut app = App::new();
    init_world_presentation_assets(&mut app);
    app.add_systems(Update, reveal_native_world_scenes);
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "metadata-error".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldBehaviourStatus::Ready,
            NativeWorldVisualPresentationStatus::Loading,
            NativeWorldPresentationStatus::Loading,
            Visibility::Hidden,
        ))
        .id();
    let visual = app
        .world_mut()
        .spawn((
            ChildOf(root),
            SpawnedNativeWorldVisual {
                model_path: "models/error.glb".to_owned(),
                source_model_path: "models/error.glb".to_owned(),
                scene: 0,
            },
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldObjectRangeReady,
            NativeWorldVisualSceneReady,
            Visibility::Hidden,
        ))
        .id();
    app.world_mut().spawn((
        ChildOf(visual),
        GltfMaterialExtras::default(),
        LegacyMaterialMetadataError("fixture".to_owned()),
    ));

    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Loading)
    );
    app.world_mut()
        .entity_mut(visual)
        .insert(NativeWorldObjectRangeUnbounded);
    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Ready)
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden),
        "a settled metadata error must not expose its fallback mesh"
    );
}

#[test]
fn late_material_error_hides_an_already_ready_direct_mesh_permanently() {
    let mut app = App::new();
    app.add_systems(Update, make_unsafe_native_world_range_groups_unbounded);
    let key = NativeWorldObjectRangeGroupKey::Visual(0);
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "late-material-error".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldVisualPresentationStatus::Ready,
            NativeWorldObjectRangeGroups {
                pending: HashMap::new(),
                ready_to_finalize: VecDeque::new(),
                unbounded: HashSet::new(),
            },
        ))
        .id();
    let tag = pack_native_world_object_range(Vec3::new(-3500.0, -50.0, 4500.0), 100.0).unwrap();
    let visual = app
        .world_mut()
        .spawn((
            ChildOf(root),
            SpawnedNativeWorldVisual {
                model_path: "models/late-error.glb".to_owned(),
                source_model_path: "models/late-error.glb".to_owned(),
                scene: 0,
            },
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: key.clone(),
            },
            NativeWorldObjectRangeContract {
                tag,
                meshes: Vec::new(),
                visible: true,
            },
            MeshTag(tag),
            Visibility::Inherited,
            LegacyMaterialMetadataError("late fixture".to_owned()),
        ))
        .id();
    app.world_mut()
        .get_mut::<NativeWorldObjectRangeContract>(visual)
        .unwrap()
        .meshes
        .push(visual);

    app.update();
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden)
    );
    assert!(
        app.world()
            .get::<NativeWorldMaterialPresentationFailed>(visual)
            .is_some()
    );
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeContract>(visual)
            .is_none()
    );
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeUnbounded>(visual)
            .is_some()
    );
    assert!(app.world().get::<MeshTag>(visual).is_none());
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeGroups>(root)
            .unwrap()
            .unbounded
            .contains(&key)
    );
}

#[test]
fn world_presentation_waits_for_budgeted_animation_material_bindings() {
    let mut app = App::new();
    init_world_presentation_assets(&mut app);
    app.add_systems(Update, reveal_native_world_scenes);
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "animated".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldBehaviourStatus::Ready,
            NativeWorldVisualPresentationStatus::Loading,
            NativeWorldPresentationStatus::Loading,
            Visibility::Hidden,
        ))
        .id();
    let animation = app
        .world_mut()
        .spawn((
            ChildOf(root),
            crate::world_behaviour::WorldAnimationMaterialBindings::default(),
        ))
        .id();

    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Loading)
    );
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden)
    );

    app.world_mut()
        .entity_mut(animation)
        .remove::<crate::world_behaviour::WorldAnimationMaterialBindings>();
    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Ready)
    );
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Inherited)
    );
}

#[test]
fn tall_steep_concavity_stops_the_down_pass_and_reenables_jump_at_rest() {
    let dimple = step_sized_dimple_collider(2.0, 0.8);
    let global = GlobalTransform::IDENTITY;
    let bounds = AuthoredColliderWorldBounds::from_collider(&dimple, &global).unwrap();
    let left = dimple.indices.chunks_exact(3).next().unwrap();
    let a = dimple.vertices[left[0] as usize];
    let b = dimple.vertices[left[1] as usize];
    let c = dimple.vertices[left[2] as usize];
    let normal = (b - a).cross(c - a).normalize();
    assert!(normal.x > 0.8 && normal.y < AUTHORED_WALKABLE_MIN_UP_DOT);
    let start_x = -0.5;
    let plane_y = a.y - normal.x * (start_x - a.x) / normal.y;
    let radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let start_y = plane_y + (radius / normal.y - AUTHORED_CHARACTER_CONTROLLER_RADIUS).max(0.0);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState::default())
        .init_resource::<crate::movement::MovementIntentQueue>()
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                crate::movement::simulate_legacy_players,
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    app.world_mut().spawn((global, dimple, bounds));
    app.update();

    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    controller
        .set_external_collision_result(crate::movement::LEGACY_COLLISION_SIDES, Some(normal));
    let player = app
        .world_mut()
        .spawn((Transform::from_xyz(start_x, start_y, 0.0), controller))
        .id();

    let mut settled = None;
    for frame in 0..180 {
        app.update();
        let position = app.world().get::<Transform>(player).unwrap().translation;
        let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
        if controller.grounded && !controller.jumping && !controller.surface_sliding() {
            settled = Some((frame, position));
            break;
        }
    }
    let (settled_frame, settled_position) = settled.expect(
        "the full DOWN contact must hold the capsule until terminal rest clears sliding",
    );
    assert!(
        settled_frame >= 50,
        "terminal rest fired before capped gravity"
    );
    assert!(
        settled_position.x.abs() < 0.35 && settled_position.y > 0.35,
        "the capsule fell through or escaped the steep concavity: {settled_position:?}"
    );

    app.world_mut()
        .resource_mut::<crate::movement::LegacyInputState>()
        .jump_just_pressed = true;
    app.update();
    let jumped_position = app.world().get::<Transform>(player).unwrap().translation;
    let jumped = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!(
        jumped.jumping,
        "terminal rest did not re-enable normal jump"
    );
    assert!(
        jumped_position.y > settled_position.y + 0.01,
        "the player settled but could not push off: before={settled_position:?}, after={jumped_position:?}"
    );
}
