use super::*;

fn assert_gate(app: &App, entity: Entity, visible: bool) {
    assert_eq!(
        app.world()
            .get::<InheritedVisibility>(entity)
            .unwrap()
            .get(),
        visible,
        "unexpected inherited visibility for {entity:?}"
    );
}

#[test]
fn managed_pod_passes_follow_root_across_distance_and_reload() {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin::default(),
        bevy::mesh::MeshPlugin,
        TransformPlugin,
        bevy::camera::visibility::VisibilityPlugin,
    ))
    .init_resource::<NativeWorldObjectRangeCameraCache>()
    .add_systems(
        Update,
        (
            propagate_native_world_object_ranges_to_late_meshes,
            make_unsafe_native_world_range_groups_unbounded,
            update_native_world_object_residency,
        )
            .chain(),
    );
    let scene = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "race pod test".into(),
                tile: [5, 10],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldVisualPresentationStatus::Loading,
        ))
        .id();
    let center = Vec3::new(-2936.0, -50.0, 5470.0);
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            LegacyOrbitCamera::new(scene),
            Transform::from_translation(center),
        ))
        .id();
    let tag = pack_native_world_object_range(center, 100.0).unwrap();
    for cycle in 0..2 {
        let root = app
            .world_mut()
            .spawn((
                Mesh3d::default(),
                Transform::from_translation(center),
                Visibility::Hidden,
                RuntimeManagedNativeWorldVisual,
                NativeWorldObjectRangeMember {
                    scene_root: scene,
                    group: NativeWorldObjectRangeGroupKey::Visual(cycle),
                },
            ))
            .id();
        let early = app
            .world_mut()
            .spawn((
                Mesh3d::default(),
                Transform::IDENTITY,
                Visibility::Hidden,
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .entity_mut(root)
            .insert(NativeWorldObjectRangeContract {
                tag,
                meshes: vec![root, early],
                visible: true,
            });
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(early),
            Some(&Visibility::Inherited)
        );
        assert_gate(&app, early, false);

        // Scene reveal changes the root without another camera movement.
        app.world_mut()
            .entity_mut(scene)
            .insert(NativeWorldVisualPresentationStatus::Ready);

        // Material color/outline passes can arrive after the source was hidden.
        let late = app
            .world_mut()
            .spawn((
                Mesh3d::default(),
                Transform::IDENTITY,
                Visibility::Hidden,
                ChildOf(root),
            ))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(late),
            Some(&Visibility::Inherited)
        );
        app.world_mut()
            .entity_mut(root)
            .insert(Visibility::Inherited);
        app.update();
        assert_gate(&app, early, true);
        assert_gate(&app, late, true);

        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation = center + Vec3::X * 200.0;
        app.update();
        assert_gate(&app, early, false);
        assert_gate(&app, late, false);
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation = center;
        app.update();
        // Residency must not activate an inactive/collected pod on its own.
        assert_gate(&app, late, false);
        app.world_mut()
            .entity_mut(root)
            .insert(Visibility::Inherited);
        app.update();
        assert_gate(&app, early, true);
        assert_gate(&app, late, true);
        // Animation attaches while the production scene is still loading.
        // Existing passes lose the range contract and will not be revisited by
        // Added<Mesh3d> or residency after reveal.
        app.world_mut()
            .entity_mut(scene)
            .insert(NativeWorldVisualPresentationStatus::Loading);
        app.world_mut()
            .entity_mut(root)
            .insert(NativeWorldDynamicObjectRange);
        app.update();
        assert!(
            app.world()
                .get::<NativeWorldObjectRangeContract>(root)
                .is_none()
        );
        assert_gate(&app, late, false);
        assert_eq!(
            app.world().get::<Visibility>(late),
            Some(&Visibility::Inherited)
        );
        app.world_mut()
            .entity_mut(scene)
            .insert(NativeWorldVisualPresentationStatus::Ready);
        app.world_mut()
            .entity_mut(root)
            .insert(Visibility::Inherited);
        app.update();
        assert_gate(&app, early, true);
        assert_gate(&app, late, true);
        app.world_mut().entity_mut(root).insert(Visibility::Hidden);
        app.update();
        assert_gate(&app, late, false);
        app.world_mut().entity_mut(root).despawn();
    }
}

#[test]
fn unbounded_managed_pass_inherits_root_but_material_failure_stays_hidden() {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin::default(),
        bevy::mesh::MeshPlugin,
        TransformPlugin,
        bevy::camera::visibility::VisibilityPlugin,
    ))
    .add_systems(Update, propagate_native_world_object_ranges_to_late_meshes);
    let scene = app.world_mut().spawn_empty().id();
    let root = app
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Visibility::Hidden,
            RuntimeManagedNativeWorldVisual,
            NativeWorldObjectRangeUnbounded,
            NativeWorldObjectRangeMember {
                scene_root: scene,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
        ))
        .id();
    let pass = app
        .world_mut()
        .spawn((
            Mesh3d::default(),
            Transform::IDENTITY,
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(pass),
        Some(&Visibility::Inherited)
    );
    assert_gate(&app, pass, false);
    app.world_mut()
        .entity_mut(root)
        .insert(Visibility::Inherited);
    app.update();
    assert_gate(&app, pass, true);
    app.world_mut()
        .entity_mut(root)
        .insert(NativeWorldMaterialPresentationFailed);
    let failed_pass = app
        .world_mut()
        .spawn((
            Mesh3d::default(),
            Transform::IDENTITY,
            Visibility::Inherited,
            ChildOf(root),
        ))
        .id();
    app.update();
    assert_gate(&app, failed_pass, false);
}
