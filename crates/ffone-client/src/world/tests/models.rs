use super::*;

#[test]
fn audited_smallstuff_guard_matches_only_exact_primary_model_identities() {
    const PRODUCTION_15_VARIANT: &str = "objects/nature/15_default_wd_dldt_darktree_01_14/models/\
         wd_dldt_darktree_bridge_01_15_default_variant_0002/visual.glb";
    assert!(matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/15_default_wd_dldt_darktree_01_14/models/\
         wd_dldt_darktree_bridge_01_15_default/visual.glb"
    ));
    assert!(matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        PRODUCTION_15_VARIANT
    ));
    assert!(matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/16_default_wd_dldt_darktree_01_15/models/\
         wd_dldt_darktree_bridge_01_16_default/visual.glb"
    ));
    assert!(matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/16_default_wd_dldt_darktree_01_15/models/\
         wd_dldt_darktree_bridge_01_16_default_variant_0004/visual.glb"
    ));

    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        None,
        PRODUCTION_15_VARIANT
    ));
    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        Some(16),
        PRODUCTION_15_VARIANT
    ));

    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/15_default_wd_dldt_darktree_01_14/models/\
         wd_dldt_darktree_01_15_default_variant_0002/visual.glb"
    ));
    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/15_default_wd_dldt_darktree_01_14/models/\
         wd_dldt_darktree_bridge_010_15_default_variant_0002/visual.glb"
    ));
    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/15_default_wd_dldt_darktree_01_14/models/\
         wd_dldt_darktree_bridge_01_15_default_variant_002/visual.glb"
    ));
    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/14_default_wd_dldt_darktree_01_13/models/\
         wd_dldt_darktree_bridge_01_15_default_variant_0002/visual.glb"
    ));

    assert!(matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        PRIMARY_ETC_TREE_07_FROND_MODEL_PATH
    ));
    assert!(matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        PRIMARY_ETC_TREE_07_TRUNK_MODEL_PATH
    ));
    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        None,
        PRIMARY_ETC_TREE_07_FROND_MODEL_PATH
    ));
    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        Some(16),
        PRIMARY_ETC_TREE_07_FROND_MODEL_PATH
    ));
    assert!(!matches_primary_audited_smallstuff_presentation_identity(
        Some(12),
        "objects/nature/etc_tree_07_variant_0002/models/etc_tree_07_variant_0004/visual.glb"
    ));
}

pub(super) fn authored_gltf_mesh_data(relative: &str, mesh_index: Option<usize>) -> (Vec<Vec3>, Vec<u32>) {
    let gltf = gltf::Gltf::open(join_relative(&asset_root(), relative)).unwrap();
    let binary = gltf.blob.as_deref();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for primitive in gltf
        .meshes()
        .enumerate()
        .filter(|(index, _)| mesh_index.is_none_or(|expected| expected == *index))
        .flat_map(|(_, mesh)| mesh.primitives())
    {
        let reader = primitive.reader(|buffer| match buffer.source() {
            gltf::buffer::Source::Bin => binary,
            gltf::buffer::Source::Uri(_) => None,
        });
        let base = vertices.len() as u32;
        vertices.extend(
            reader
                .read_positions()
                .expect("collider primitive must have positions")
                .map(Vec3::from),
        );
        indices.extend(
            reader
                .read_indices()
                .expect("collider primitive must have indices")
                .into_u32()
                .map(|index| base + index),
        );
    }
    (vertices, indices)
}

#[test]
fn world_presentation_waits_for_late_mesh_tag_after_range_ready() {
    let mut app = App::new();
    init_world_presentation_assets(&mut app);
    app.add_systems(Update, reveal_native_world_scenes);
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "late-pass".to_owned(),
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
                model_path: "models/late-pass.glb".to_owned(),
                source_model_path: "models/late-pass.glb".to_owned(),
                scene: 0,
            },
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldVisualSceneReady,
            NativeWorldObjectRangeReady,
            Visibility::Hidden,
        ))
        .id();
    let late_mesh = app
        .world_mut()
        .spawn((ChildOf(visual), Mesh3d::default()))
        .id();

    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Loading)
    );
    app.world_mut().entity_mut(late_mesh).insert(MeshTag(7));
    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Ready)
    );
}

#[test]
fn late_mesh_retries_until_contract_then_receives_exact_shared_tag() {
    let mut app = App::new();
    app.add_systems(Update, propagate_native_world_object_ranges_to_late_meshes);
    let scene_root = app.world_mut().spawn_empty().id();
    let visual = app
        .world_mut()
        .spawn(NativeWorldObjectRangeMember {
            scene_root,
            group: NativeWorldObjectRangeGroupKey::Visual(0),
        })
        .id();
    let mesh = app
        .world_mut()
        .spawn((ChildOf(visual), Mesh3d::default()))
        .id();

    app.update();
    assert!(
        app.world()
            .get::<PendingNativeWorldObjectRangeTag>(mesh)
            .is_some()
    );
    assert!(app.world().get::<MeshTag>(mesh).is_none());

    let tag = pack_native_world_object_range(Vec3::new(-3500.0, -50.0, 4500.0), 250.0).unwrap();
    app.world_mut()
        .entity_mut(visual)
        .insert(NativeWorldObjectRangeContract {
            tag,
            meshes: Vec::new(),
            visible: false,
        });
    app.update();
    assert_eq!(app.world().get::<MeshTag>(mesh), Some(&MeshTag(tag)));
    assert_eq!(
        app.world().get::<Visibility>(mesh),
        Some(&Visibility::Hidden)
    );
    let expected_range = native_world_pipeline_sentinel_range();
    let actual_range = app
        .world()
        .get::<VisibilityRange>(mesh)
        .expect("late mesh should inherit the native-world pipeline range");
    assert_eq!(actual_range.start_margin, expected_range.start_margin);
    assert_eq!(actual_range.end_margin, expected_range.end_margin);
    assert!(
        app.world()
            .get::<PendingNativeWorldObjectRangeTag>(mesh)
            .is_none()
    );
    assert_eq!(
        app.world()
            .get::<NativeWorldObjectRangeContract>(visual)
            .unwrap()
            .meshes,
        vec![mesh]
    );
}

#[test]
fn unbounded_late_mesh_drops_retry_without_receiving_a_false_range() {
    let mut app = App::new();
    app.add_systems(Update, propagate_native_world_object_ranges_to_late_meshes);
    let scene_root = app.world_mut().spawn_empty().id();
    let visual = app
        .world_mut()
        .spawn((
            NativeWorldObjectRangeMember {
                scene_root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldObjectRangeUnbounded,
        ))
        .id();
    let mesh = app
        .world_mut()
        .spawn((
            ChildOf(visual),
            Mesh3d::default(),
            PendingNativeWorldObjectRangeTag,
        ))
        .id();

    app.update();
    assert!(
        app.world()
            .get::<PendingNativeWorldObjectRangeTag>(mesh)
            .is_none()
    );
    assert!(app.world().get::<MeshTag>(mesh).is_none());
    assert!(app.world().get::<VisibilityRange>(mesh).is_none());
}

#[test]
fn streamed_scene_unload_budgets_nested_gltf_entities_not_only_roots() {
    let mut app = App::new();
    app.add_systems(Update, unload_native_world_scenes_incrementally);
    let root = app
        .world_mut()
        .spawn(PendingNativeWorldSceneUnload::default())
        .id();
    for _ in 0..4 {
        let scene_root = app.world_mut().spawn(ChildOf(root)).id();
        for _ in 0..20 {
            app.world_mut().spawn(ChildOf(scene_root));
        }
    }

    let mut previous_count = game_entity_count(app.world());
    while app.world().get_entity(root).is_ok() {
        app.update();
        let count = game_entity_count(app.world());
        assert!(
            previous_count - count <= NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME as u32,
            "one unload frame removed {} actual entities",
            previous_count - count
        );
        previous_count = count;
    }
    assert_eq!(previous_count, 0);
}

#[test]
fn authored_mesh_front_face_blocks_entry_while_back_face_allows_escape() {
    let wall = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/one-sided-wall.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(0.0, 3.0, -2.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 3.0, 2.0),
        ]
        .into(),
        // Both faces point toward +X. This is the playable/front side.
        indices: vec![0, 1, 2, 2, 1, 3].into(),
        local_min: Vec3::new(0.0, 0.0, -2.0),
        local_max: Vec3::new(0.0, 3.0, 2.0),
        is_trigger: false,
    };
    let colliders = [(Mat4::IDENTITY, &wall)];
    let radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;

    let entering = resolve_authored_wall_motion(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-2.0, 0.0, 0.0),
        &colliders,
    );
    assert!(
        entering.x >= radius - AUTHORED_COLLISION_CONTACT_TOLERANCE,
        "the authored front face must stop entry before the capsule crosses it: {entering:?}"
    );

    // Reproduce a streamed/teleported controller already overlapping the
    // back of the same MeshCollider. The old two-sided normal flip pinned
    // it at X<0; PhysX CCT's ordinary mesh sweep lets it leave.
    let mut escaped = Vec3::new(-0.1, 0.0, 0.0);
    for frame in 0..16 {
        let next = resolve_authored_wall_motion(escaped, Vec3::new(0.05, 0.0, 0.0), &colliders);
        assert!(
            next.x >= escaped.x - AUTHORED_COLLISION_EPSILON,
            "back-face escape moved the capsule deeper inside on frame {frame}: \
             before={escaped:?}, after={next:?}"
        );
        escaped = next;
    }
    assert!(
        escaped.x > 0.6,
        "a capsule already behind the mesh must be able to exit through its back face: \
         {escaped:?}"
    );
}

#[test]
fn authored_capsule_motion_cannot_cross_a_shared_mesh_corner() {
    let corner = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/corner.glb".to_owned(),
        vertices: vec![
            // X=0 wall extending into -Z.
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(0.0, 3.0, -2.0),
            // Z=0 wall extending into -X.
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(-2.0, 3.0, 0.0),
        ]
        .into(),
        indices: vec![0, 1, 2, 2, 1, 3, 4, 6, 5, 6, 7, 5].into(),
        local_min: Vec3::new(-2.0, 0.0, -2.0),
        local_max: Vec3::new(0.0, 3.0, 0.0),
        is_trigger: false,
    };
    let previous = Vec3::new(-1.0, 0.0, -1.0);
    let displacement = Vec3::new(1.5, 0.0, 1.5);
    let colliders = [(Mat4::IDENTITY, &corner)];
    let resolved = resolve_authored_wall_motion(previous, displacement, &colliders);
    let contact_limit = -(AUTHORED_CHARACTER_CONTROLLER_RADIUS
        - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH)
        + AUTHORED_COLLISION_EPSILON * 4.0;
    assert!(
        resolved.x <= contact_limit && resolved.z <= contact_limit,
        "the capsule must remain outside both faces of the shared corner: {resolved:?}"
    );

    let against_first_wall =
        resolve_authored_wall_motion(previous, Vec3::new(1.5, 0.0, 0.0), &colliders);
    let around_corner =
        resolve_authored_wall_motion(against_first_wall, Vec3::new(0.75, 0.0, 1.5), &colliders);
    assert!(
        around_corner.x <= contact_limit && around_corner.z <= contact_limit,
        "holding into a wall while turning toward its corner must not cross either face: \
         first={against_first_wall:?}, second={around_corner:?}"
    );
}

#[test]
fn tutorial_00413_mesh_corner_keeps_a_grounded_capsule_outside() {
    // Exact side faces around the convex corner visible during
    // MovementStage::ReachLedge, extracted from
    // c-00413-6608-f652008795a6.glb (triangles 51, 52, 56, and 57).
    let ledge = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/world/tutorial/tile_01_01/c-00413-6608-f652008795a6.glb"
            .to_owned(),
        vertices: vec![
            Vec3::new(-557.237_6, -104.693_214, 653.493_35),
            Vec3::new(-557.073_3, -107.313_63, 653.483_3),
            Vec3::new(-557.866_8, -104.725_876, 651.719_7),
            Vec3::new(-557.702_45, -107.346_29, 651.709_66),
            Vec3::new(-557.313_3, -107.341_17, 656.694_4),
            Vec3::new(-557.477_66, -104.720_59, 656.704_47),
        ]
        .into(),
        indices: vec![
            0, 1, 2, // triangle 51
            3, 2, 1, // triangle 52
            1, 0, 4, // triangle 56
            5, 4, 0, // triangle 57
        ]
        .into(),
        local_min: Vec3::new(-557.866_8, -107.346_29, 651.709_66),
        local_max: Vec3::new(-557.073_3, -104.693_214, 656.704_47),
        is_trigger: false,
    };
    let colliders = [(Mat4::IDENTITY, &ledge)];
    let mut feet = Vec3::new(-556.7, -107.35, 652.2);

    // First hold directly into the wall, then keep holding into it while
    // strafing across the shared 51/56 corner.
    for _ in 0..20 {
        feet = resolve_authored_wall_motion(feet, Vec3::new(-0.08, 0.0, 0.0), &colliders);
    }
    for _ in 0..40 {
        feet = resolve_authored_wall_motion(feet, Vec3::new(-0.08, 0.0, 0.08), &colliders);
    }

    assert!(
        feet.x > -557.237_6,
        "the capsule crossed through tutorial MeshCollider 00413: {feet:?}"
    );
}

#[test]
fn tall_mesh_does_not_treat_its_lower_triangles_as_independent_steps() {
    let wall = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/tessellated-tall-wall.glb".to_owned(),
        vertices: vec![
            // Only this low wall band is near the capsule. Its individual
            // maximum is below stepOffset.
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.2, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            // A high band belongs to the same MeshCollider but is far
            // outside the narrow-phase XZ query.
            Vec3::new(0.0, 1.8, 9.0),
            Vec3::new(0.0, 2.0, 9.0),
            Vec3::new(0.0, 1.8, 11.0),
        ]
        .into(),
        indices: vec![0, 2, 1, 3, 5, 4].into(),
        local_min: Vec3::new(0.0, 0.0, -1.0),
        local_max: Vec3::new(0.0, 2.0, 11.0),
        is_trigger: false,
    };
    let colliders = [(Mat4::IDENTITY, &wall)];
    let previous = Vec3::new(-0.2, 0.0, 0.0);
    let resolved = resolve_authored_wall_motion(previous, Vec3::new(0.4, 0.0, 0.0), &colliders);
    assert!(
        resolved.x < 0.0,
        "stepOffset must use the complete MeshCollider height, not open a hole through its \
         low triangle band: {resolved:?}"
    );
}
