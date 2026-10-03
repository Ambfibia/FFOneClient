use super::*;

pub(super) fn sanitize_runtime_environment(value: &mut serde_json::Value) {
    let object = value.as_object_mut().unwrap();
    object.remove("mapScene");
    object.remove("placementAudit");
    object.remove("sourceCodeEvidence");
    object
        .get_mut("ambience")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .remove("sourceObject");
    let detail = object
        .get_mut("terrainDetail")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    detail.remove("sourceObject");
    detail.remove("terrainRendererSourceObject");
    remove_key_recursively(value, "parsedData");
}

#[test]
fn dynamic_group_state_makes_late_siblings_ready_and_unbounded() {
    let mut app = App::new();
    app.add_systems(Update, make_unsafe_native_world_range_groups_unbounded);
    let key = NativeWorldObjectRangeGroupKey::Prefab(Arc::from("moving-owner"));
    let scene_root = app
        .world_mut()
        .spawn(NativeWorldObjectRangeGroups {
            pending: HashMap::from([(
                key.clone(),
                NativeWorldPendingRangeGroup {
                    expected_members: 2,
                    members: Vec::new(),
                },
            )]),
            ready_to_finalize: VecDeque::new(),
            unbounded: HashSet::new(),
        })
        .id();
    let first = app
        .world_mut()
        .spawn((
            NativeWorldObjectRangeMember {
                scene_root,
                group: key.clone(),
            },
            NativeWorldDynamicObjectRange,
        ))
        .id();

    app.update();
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeUnbounded>(first)
            .is_some()
    );
    let state = app
        .world()
        .get::<NativeWorldObjectRangeGroups>(scene_root)
        .unwrap();
    assert!(state.unbounded.contains(&key));
    assert!(!state.pending.contains_key(&key));

    // This sibling represents a WorldAssetRoot admitted after the first member
    // already made the logical group permanently dynamic.
    let late = app
        .world_mut()
        .spawn((
            NativeWorldObjectRangeMember {
                scene_root,
                group: key,
            },
            PendingNativeWorldObjectBounds::default(),
        ))
        .id();
    app.add_systems(Update, prepare_native_world_object_ranges);
    app.update();
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeReady>(late)
            .is_some()
    );
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeUnbounded>(late)
            .is_some()
    );
    assert!(
        app.world()
            .get::<PendingNativeWorldObjectBounds>(late)
            .is_none()
    );
}

#[test]
fn runtime_managed_visual_stays_hidden_across_ready_near_residency_updates() {
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
    let center = Vec3::new(-3500.0, -50.0, 4500.0);
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            LegacyOrbitCamera::new(Entity::PLACEHOLDER),
            Transform::from_translation(center),
        ))
        .id();
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "runtime-managed-residency".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldVisualPresentationStatus::Ready,
        ))
        .id();
    let tag = pack_native_world_object_range(center, 100.0).unwrap();
    let visual = app
        .world_mut()
        .spawn((
            ChildOf(root),
            RuntimeManagedNativeWorldVisual,
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldObjectRangeContract {
                tag,
                meshes: Vec::new(),
                visible: false,
            },
            Mesh3d::default(),
            Visibility::Hidden,
        ))
        .id();
    app.world_mut()
        .get_mut::<NativeWorldObjectRangeContract>(visual)
        .unwrap()
        .meshes
        .push(visual);
    let late_mesh = app
        .world_mut()
        .spawn((ChildOf(visual), Mesh3d::default(), Visibility::Inherited))
        .id();

    app.update();
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeContract>(visual)
            .unwrap()
            .visible,
        "the near object should remain logically resident"
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(late_mesh),
        Some(&Visibility::Inherited),
        "the companion inherits the hidden root and can follow its later activation"
    );

    // Reproduce either a late reveal or a previous residency writer, then
    // cross the camera sampling threshold while remaining inside range.
    app.world_mut()
        .entity_mut(visual)
        .insert(Visibility::Inherited);
    app.world_mut()
        .entity_mut(late_mesh)
        .insert(Visibility::Inherited);
    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation
        .x += NATIVE_WORLD_RANGE_CAMERA_STEP;
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(late_mesh),
        Some(&Visibility::Inherited)
    );

    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation = center + Vec3::X * 200.0;
    app.update();
    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation = center;
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden),
        "leaving and re-entering residency must not publish the controller-owned duplicate"
    );
    assert_eq!(
        app.world().get::<Visibility>(late_mesh),
        Some(&Visibility::Inherited)
    );
}
