use crate::player_shared_rig::*;
use bevy::{
    asset::{AssetApp, AssetPlugin},
    ecs::system::SystemState,
};

#[test]
fn preview_asset_cache_retains_strong_handles_across_scene_transitions() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Gltf>()
        .init_asset::<WorldAsset>()
        .init_asset::<Image>();
    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut cache = NativePlayerRigAssetCache::default();
    cache.gltf(
        &asset_server,
        "characters/player/shared/test.glb".to_owned(),
    );
    cache.scene(
        &asset_server,
        "characters/player/shared/test.glb".to_owned(),
    );
    let image = asset_server.load("characters/player/textures/test.png");
    cache.retain_image("characters/player/textures/test.png".to_owned(), image);

    assert_eq!(cache.retained_gltfs(), 1);
    assert_eq!(cache.retained_scenes(), 1);
    assert!(cache.retained_handles_are_strong());

    cache.release_cached_handles();
    assert_eq!(cache.retained_gltfs(), 0);
    assert_eq!(cache.retained_scenes(), 0);
    assert!(cache.images.is_empty());
    assert!(cache.animation_graphs.is_empty());
}

#[test]
fn bug019_civilian_walk_is_available_on_both_shared_rigs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativePlayerRigCatalog::open(&root).unwrap();
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        let rig = catalog.gender(gender).unwrap();
        let clips: Vec<_> = rig.clips.iter().filter(|clip| clip.name == "walk").collect();
        assert_eq!(clips.len(), 1);
        let clip = clips[0];
        assert_eq!(clip.playback, "loop");
        assert!(clip.channel_count > 0);
        assert!(f64::from_bits(clip.duration_seconds_bits) > 0.0);
        let bytes = std::fs::read(root.join(&rig.skeleton_glb)).unwrap();
        let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
        let animation = gltf.animations().nth(clip.gltf_animation_index as usize).unwrap();
        assert_eq!(animation.name(), Some("walk"));
        assert_eq!(animation.channels().count(), clip.channel_count as usize);
        // The shared actor clip must coexist with normal player locomotion.
        assert!(rig.clips.iter().any(|clip| clip.name == "run"));
        assert!(rig.clips.iter().any(|clip| clip.name == "stand1"));
    }
}

#[test]
fn production_catalog_verifies_both_native_shared_rigs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativePlayerRigCatalog::open(root).unwrap();
    assert!(!catalog.contract.fake_animation_used);
    assert!(!catalog.contract.unity_runtime_required);
    let male = catalog.gender(PlayerRigGender::Male).unwrap();
    let female = catalog.gender(PlayerRigGender::Female).unwrap();
    assert_eq!(male.nodes.len(), 133);
    assert_eq!(female.nodes.len(), 159);
    assert!(
        male.creator_parts.len() >= 390,
        "male catalog must include the table-owned skinned equipment extension"
    );
    assert!(
        female.creator_parts.len() >= 350,
        "female catalog must include the table-owned skinned equipment extension"
    );
    assert_eq!(male.creator_choices.len(), 118);
    assert_eq!(female.creator_choices.len(), 113);
    assert_eq!(male.default_creator_part_routes.len(), 5);
    assert_eq!(female.default_creator_part_routes.len(), 5);
    for rig in [male, female] {
        assert_eq!(
            rig.creator_parts
                .iter()
                .filter(|part| {
                    part.actor_skin_combiner_clothes_index == SKINNED_BACK_CLOTHES_INDEX
                })
                .count(),
            14,
            "every unique equipType=1 Back route must be published in the shared rig"
        );
    }
    for route in [
        "wear/m_shirt_naked.nif",
        "wear/m_pants_naked.nif",
        "wear/m_shoes_naked.nif",
    ] {
        catalog
            .part_by_exact_route(PlayerRigGender::Male, route)
            .unwrap();
    }
    let table_owned = catalog
        .part_by_exact_route(PlayerRigGender::Male, "wear/m_pants_apacheshield.nif")
        .unwrap();
    assert_eq!(table_owned.actor_skin_combiner_clothes_index, 1);
    assert!(
        table_owned
            .skins
            .iter()
            .all(|skin| !skin.exact_transform_index_parity),
        "table-owned extension must not claim recovered ActorWearIndexTable parity"
    );
    assert!(catalog.table_extension_failures.values().any(|error| {
        error.contains("Bip01 Skirt back") && error.contains("absent from the actor rig")
    }));
    for route in [
        "wear/f_shirt_naked.nif",
        "wear/f_pants_naked.nif",
        "wear/f_shoes_naked.nif",
    ] {
        catalog
            .part_by_exact_route(PlayerRigGender::Female, route)
            .unwrap();
    }
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        let rig = catalog.gender(gender).unwrap();
        for type01 in rig.creator_parts.iter().filter(|part| {
            part.exact_route.ends_with("_type01.nif")
                && (part.true_name.contains("_face_") || part.true_name.contains("_head_"))
        }) {
            let type02_route = type01.exact_route.replace("_type01.nif", "_type02.nif");
            catalog
                .part_by_exact_route(gender, &type02_route)
                .unwrap_or_else(|error| {
                    panic!("missing required SetHat variant {type02_route}: {error}")
                });
        }
        let routes = catalog.default_creator_routes(gender).unwrap();
        let default_parts = routes
            .iter()
            .map(|route| catalog.part_by_exact_route(gender, route).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            default_parts
                .iter()
                .map(|part| part.actor_skin_combiner_clothes_index)
                .collect::<Vec<_>>(),
            [3, 4, 2, 1, 0],
            "default request order deliberately differs from legacy clothes order"
        );
        let mut ordered_clothes_slots = default_parts
            .iter()
            .map(|part| part.actor_skin_combiner_clothes_index)
            .collect::<Vec<_>>();
        ordered_clothes_slots.sort_unstable();
        assert_eq!(ordered_clothes_slots, [0, 1, 2, 3, 4]);
        let glbs = default_parts
            .iter()
            .map(|part| part.glb.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            catalog.exact_routes_for_glbs(gender, &glbs).unwrap(),
            routes
        );
    }
    assert!(male.stand1_runtime_ready && female.stand1_runtime_ready);
    let expected_emotes = [
        "cry", "angry", "shocked", "hello", "thank", "dance1", "kiss", "agree", "laugh", "no",
        "flex", "tease", "ok", "applaud", "cheer", "dance2", "dance3", "dance4", "dance5",
        "goodbye", "beach1", "beach2", "beach3",
    ];
    for gender in [male, female] {
        for name in [
            "stun",
            "stickdash",
            "rifledash",
            "rifletumbling",
            "rocketsomersault",
            "stickdodgeupper",
            "rifledodgeupper",
        ] {
            let clips: Vec<_> = gender
                .clips
                .iter()
                .filter(|clip| clip.name == name)
                .collect();
            assert_eq!(clips.len(), 1, "ability must resolve exactly once: {name}");
            let clip = clips[0];
            assert_eq!(
                clip.source_path_id, 0,
                "native additions use semantic identities"
            );
            assert_eq!(clip.playback, "clamp");
            assert_eq!(clip.runtime_status, "native-bevy-ready");
        }
        let rifle = gender
            .clips
            .iter()
            .find(|clip| clip.name == "riflestand1")
            .expect("native rig must publish the original rifle idle");
        assert_eq!(rifle.playback, "loop");
        assert_eq!(rifle.runtime_status, "native-bevy-ready");
        for (offset, name) in expected_emotes.into_iter().enumerate() {
            let clip = gender
                .clips
                .iter()
                .find(|clip| clip.name == name)
                .unwrap_or_else(|| panic!("native rig must publish emote clip {name}"));
            assert_eq!(clip.gltf_animation_index, 88 + offset as u32);
            assert_eq!(clip.playback, "clamp");
            assert_eq!(clip.runtime_status, "native-bevy-ready");
        }
        let wound = gender.clips.iter().find(|clip| clip.name == "woundupper")
            .expect("damage reaction must be published for both genders");
        assert_eq!(wound.playback, "clamp");
        assert!(wound.channel_count > 0);
        assert_eq!(wound.runtime_status, "native-bevy-ready");
        for name in crate::tutorial_player_presentation::FFR_CUSTOM_AVATAR_EMOTE_NAMES {
            let clip = gender.clips.iter().find(|clip| clip.name == name)
                .unwrap_or_else(|| panic!("missing Retrobution clip {name}"));
            assert!(clip.channel_count > 0, "empty Retrobution clip {name}");
            assert_eq!(clip.runtime_status, "native-bevy-ready");
        }
    }
}

#[test]
fn bone_lookup_is_instance_local_and_name_lookup_is_unique_only() {
    let a = Entity::from_raw_u32(1).unwrap();
    let b = Entity::from_raw_u32(2).unwrap();
    let bones = NativePlayerRigBones::new(vec![
        NativePlayerRigBoneEntity {
            actor_bone_index: 0,
            true_name: "m".to_owned(),
            full_path: "m".to_owned(),
            entity: a,
        },
        NativePlayerRigBoneEntity {
            actor_bone_index: 1,
            true_name: "Bip01 Head".to_owned(),
            full_path: "m/Bip01 Head".to_owned(),
            entity: b,
        },
    ]);
    assert_eq!(bones.by_actor_bone_index(1), Some(b));
    assert_eq!(bones.by_full_path("m/Bip01 Head"), Some(b));
    assert_eq!(bones.unique_by_true_name("Bip01 Head"), Some(b));
    assert_eq!(bones.unique_by_true_name("missing"), None);
}

#[test]
fn animation_player_ownership_stops_at_nested_attachment_scene_roots() {
    let mut world = World::new();
    let skeleton_scene = world.spawn(WorldAssetRoot(Handle::default())).id();
    let skeleton_bone = world.spawn(ChildOf(skeleton_scene)).id();
    let skeleton_player = world.spawn(ChildOf(skeleton_bone)).id();
    let attachment_scene = world
        .spawn((WorldAssetRoot(Handle::default()), ChildOf(skeleton_bone)))
        .id();
    let attachment_player = world.spawn(ChildOf(attachment_scene)).id();

    let mut state =
        SystemState::<(Query<&ChildOf>, Query<(), With<WorldAssetRoot>>)>::new(&mut world);
    let (parents, scene_roots) = state.get(&world).unwrap();
    assert_eq!(
        owning_scene_root(skeleton_player, &parents, &scene_roots),
        Some(skeleton_scene)
    );
    assert_eq!(
        owning_scene_root(attachment_player, &parents, &scene_roots),
        Some(attachment_scene)
    );
}

#[test]
fn late_material_pass_companion_inherits_the_rebound_shared_palette() {
    let mut app = App::new();
    app.add_systems(Update, rebind_native_player_rig_material_companions);

    let rig_root = app.world_mut().spawn_empty().id();
    let shared_joint = app.world_mut().spawn_empty().id();
    let replacement_shared_joint = app.world_mut().spawn_empty().id();
    let stale_part_joint = app.world_mut().spawn_empty().id();
    let mut source_skin = SkinnedMesh::default();
    source_skin.joints = vec![shared_joint];
    let source = app
        .world_mut()
        .spawn((
            source_skin,
            NativePlayerRigSkinBound {
                rig_root,
                generation: 7,
            },
        ))
        .id();
    let companion = app
        .world_mut()
        .spawn((
            {
                let mut skin = SkinnedMesh::default();
                skin.joints = vec![stale_part_joint];
                skin
            },
            LegacyMaterialPassCompanion {
                source_mesh_entity: source,
                pass: crate::legacy_model_material::LegacyPassKind::Outline,
            },
        ))
        .id();

    app.update();

    let rebound = app
        .world()
        .get::<SkinnedMesh>(companion)
        .expect("companion retains a skinned mesh");
    assert_eq!(rebound.joints, vec![shared_joint]);
    assert_eq!(
        app.world()
            .get::<NativePlayerRigSkinBound>(companion)
            .copied(),
        Some(NativePlayerRigSkinBound {
            rig_root,
            generation: 7,
        })
    );
    // A later source-palette replacement must also propagate. This is the
    // scheduling case which used to leave outlines in bind pose.
    app.world_mut()
        .get_mut::<SkinnedMesh>(source)
        .expect("source remains skinned")
        .joints = vec![replacement_shared_joint];
    app.update();
    assert_eq!(
        app.world()
            .get::<SkinnedMesh>(companion)
            .expect("companion remains skinned")
            .joints,
        vec![replacement_shared_joint]
    );
}

fn spawn_material_metadata_test_rig(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            NativePlayerRigInstance {
                identity: "metadata-test".to_owned(),
                generation: 11,
                gender: PlayerRigGender::Male,
            },
            NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::ModularParts),
            NativePlayerRigPartsBound {
                parts: 1,
                skin_palettes: 1,
                skinned_surfaces: 1,
            },
        ))
        .id()
}

#[test]
fn missing_typed_material_metadata_blocks_after_one_update_of_grace() {
    let mut app = App::new();
    app.add_systems(Update, fail_closed_native_player_rig_material_metadata);
    let rig = spawn_material_metadata_test_rig(&mut app);
    app.world_mut().spawn((
        Name::new("M_Face:0"),
        ChildOf(rig),
        LegacyMaterialRendererOrder { renderer_index: 0 },
    ));

    app.update();
    assert!(matches!(
        app.world().get::<NativePlayerRigStatus>(rig),
        Some(NativePlayerRigStatus::Loading(
            NativePlayerRigLoadingStage::ModularParts
        ))
    ));
    assert_eq!(
        app.world()
            .get::<NativePlayerRigMaterialMetadataGrace>(rig)
            .copied(),
        Some(NativePlayerRigMaterialMetadataGrace {
            generation: 11,
            missing_sources: 1,
        })
    );

    app.update();
    let Some(NativePlayerRigStatus::Blocked(error)) =
        app.world().get::<NativePlayerRigStatus>(rig)
    else {
        panic!("persistent missing typed metadata must block the rig");
    };
    assert!(error.contains("PendingLegacyModelMaterial"));
    assert!(error.contains("M_Face:0"));
}

#[test]
fn typed_material_metadata_error_blocks_immediately() {
    let mut app = App::new();
    app.add_systems(Update, fail_closed_native_player_rig_material_metadata);
    let rig = spawn_material_metadata_test_rig(&mut app);
    app.world_mut().spawn((
        Name::new("M_Face:0"),
        ChildOf(rig),
        LegacyMaterialRendererOrder { renderer_index: 0 },
        LegacyMaterialMetadataError("invalid exact material extras".to_owned()),
    ));

    app.update();
    let Some(NativePlayerRigStatus::Blocked(error)) =
        app.world().get::<NativePlayerRigStatus>(rig)
    else {
        panic!("typed metadata error must block the rig");
    };
    assert!(error.contains("M_Face:0"));
    assert!(error.contains("invalid exact material extras"));
}

#[test]
fn flattened_renderer_order_preserves_clothes_then_skin_array_order() {
    assert_eq!(flattened_renderer_index(0, 0), Some(0));
    assert_eq!(flattened_renderer_index(0, 1), Some(1));
    assert_eq!(
        flattened_renderer_index(2, 0),
        Some(2),
        "the first hair renderer follows both face renderers"
    );
    assert_eq!(flattened_renderer_index(u16::MAX, 1), None);
}

#[test]
fn rig_subtree_membership_preserves_source_order_and_observes_reparenting() {
    fn compare(world: &mut World, roots: &[Entity]) {
        let mut state: SystemState<(
            Query<&ChildOf>,
            Query<&Children>,
            Query<Entity, With<LegacyMaterialRendererOrder>>,
        )> = SystemState::new(world);
        let (parents, children, sources) = state.get(world).unwrap();
        let mut scratch = RigDescendantScratch::default();
        for root in roots {
            let expected: Vec<_> = sources
                .iter()
                .filter(|entity| is_descendant_of(*entity, *root, &parents))
                .collect();
            scratch.collect(*root, &children);
            let actual: Vec<_> = sources
                .iter()
                .filter(|entity| scratch.contains(*entity))
                .collect();
            assert_eq!(
                actual, expected,
                "membership and diagnostic order must match the hierarchy contract"
            );
        }
    }

    let mut world = World::new();
    let first = world.spawn_empty().id();
    let second = world.spawn_empty().id();
    let nested = world.spawn(ChildOf(first)).id();
    let movable = world
        .spawn((
            ChildOf(nested),
            LegacyMaterialRendererOrder { renderer_index: 0 },
        ))
        .id();
    for index in 0..32 {
        world.spawn((
            ChildOf(if index % 2 == 0 { first } else { second }),
            LegacyMaterialRendererOrder {
                renderer_index: index,
            },
        ));
    }

    let roots = [first, second, nested];
    compare(&mut world, &roots);
    world.entity_mut(movable).insert(ChildOf(second));
    compare(&mut world, &roots);
    world.entity_mut(movable).despawn();
    compare(&mut world, &roots);
}
