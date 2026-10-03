use super::*;

#[test]
fn warhead_sweep_detects_crossed_targets_without_hitting_behind_walls() {
    assert_eq!(warhead_sphere_hit(Vec3::ZERO, Vec3::X * 10.0, Vec3::X * 5.0, 1.0), Some(0.4));
    assert_eq!(warhead_sphere_hit(Vec3::ZERO, Vec3::X * 3.0, Vec3::X * 5.0, 1.0), None);
    assert_eq!(warhead_sphere_hit(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, 1.0), Some(0.0));
}

pub(super) fn empty_effect_spawn(
    instance_id: u64,
    owner: Option<Entity>,
    streamed_world: bool,
    rendered_nodes: usize,
) -> NativeSpawnRequest {
    let placement = owner.map_or(
        TutorialEffectPlacement::World {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        },
        |root_entity| TutorialEffectPlacement::ExactEntityWorld {
            root_entity,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        },
    );
    NativeSpawnRequest::Effect {
        instance_id,
        effect_id: if streamed_world { i32::MIN } else { 464 },
        streamed_world,
        placement,
        scale: 1.0,
        name: None,
        destroy_after_seconds: None,
        source_line: 0,
        plan: NativeEffectPlan {
            rendered_nodes,
            emitters: Vec::new(),
            mesh_scene: None,
            material_animation: None,
            maximum_timer: -1.0,
            longest_lifetime: 0.0,
            disable_update: true,
        },
    }
}

#[test]
fn native_linear_projectile_exists_between_source_and_target_before_impact() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<TutorialEffectRuntime>()
        .init_resource::<TutorialProjectileVisualReadiness>()
        .add_systems(Update, update_projectiles);
    let instance_id = app
        .world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .allocate_instance(false, None);
    let projectile = app
        .world_mut()
        .spawn((
            NativeRoot { instance_id },
            NativeProjectile {
                target: Vec3::new(0.0, 0.0, -10.0),
                motion: NativeProjectileMotion::Linear {
                    position: Vec3::ZERO,
                    hide_remaining: 0.0,
                    elapsed: 0.0,
                    duration: 1.0,
                },
                impact: None,
                waiting_for_mesh_surface: true,
                waiting_for_trail_prewarm: true,
            },
            Transform::IDENTITY,
        ))
        .id();

    // The clean client starts BulletMoveScript only after instantiating
    // its already-loaded NIF. A first-shot async GLB must not consume the
    // projectile lifetime while there is nothing renderable on screen.
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.25));
    app.update();
    assert!(
        app.world()
            .entity(projectile)
            .get::<Transform>()
            .unwrap()
            .translation
            .abs_diff_eq(Vec3::ZERO, 1.0e-5)
    );
    let mut projectile_entity = app.world_mut().entity_mut(projectile);
    let mut native_projectile = projectile_entity.get_mut::<NativeProjectile>().unwrap();
    assert!(matches!(
        native_projectile.motion,
        NativeProjectileMotion::Linear { elapsed: 0.0, .. }
    ));
    native_projectile.waiting_for_mesh_surface = false;

    // An immediately requested first trail also waits for the exact
    // texture/material render prewarm rather than expiring invisibly.
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.25));
    app.update();
    assert!(
        app.world()
            .entity(projectile)
            .get::<Transform>()
            .unwrap()
            .translation
            .abs_diff_eq(Vec3::ZERO, 1.0e-5)
    );
    app.world_mut()
        .resource_mut::<TutorialProjectileVisualReadiness>()
        .ready = true;

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.25));
    app.update();
    assert!(
        app.world()
            .entity(projectile)
            .get::<Transform>()
            .unwrap()
            .translation
            .abs_diff_eq(Vec3::new(0.0, 0.0, -2.5), 1.0e-5)
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.25));
    app.update();
    assert!(
        app.world()
            .entity(projectile)
            .get::<Transform>()
            .unwrap()
            .translation
            .abs_diff_eq(Vec3::new(0.0, 0.0, -5.0), 1.0e-5)
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.5));
    app.update();
    assert!(app.world().get_entity(projectile).is_err());
    assert!(
        !app.world()
            .resource::<TutorialEffectRuntime>()
            .active
            .contains_key(&instance_id)
    );
}

#[test]
fn expanded_weapon_projectile_meshes_are_present_and_native_ready() {
    let expected = [
        (378, "cnanoProjectile_a3", false),
        (379, "Rocket2", true),
        (729, "Ice_L", true),
        (754, "weapon_fluid_0", false),
        (755, "Ballisitic", true),
        (787, "Rocket2_up", true),
        (790, "Ballisitic_up", true),
        (791, "Ice_L_up", true),
        (792, "weapon_fluid_up", true),
        (793, "needset_up", true),
        (794, "cnanoProjectile_a3_up", false),
    ];
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&asset_root).unwrap();

    for (effect_id, legacy_name, repeats) in expected {
        let compiled = compile_projectile_plan(
            effect_id,
            library.projectile_effects.get(&effect_id).unwrap(),
        );
        let plan = compiled.plan.as_ref().unwrap_or_else(|| {
            panic!(
                "projectile ES{effect_id} blockers: {:#?}",
                compiled.blockers
            )
        });
        assert!(
            compiled.blockers.is_empty(),
            "projectile ES{effect_id} retained blockers: {:#?}",
            compiled.blockers
        );
        let scene = plan
            .mesh_scene
            .unwrap_or_else(|| panic!("projectile ES{effect_id} has no exact mesh scene"));
        assert_eq!(scene, exact_projectile_mesh_scene(effect_id).unwrap());
        assert_eq!(
            projectile_mesh_repeats_standard_animation(effect_id),
            repeats
        );

        let glb_path = asset_root.join(scene);
        let glb = std::fs::read(&glb_path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", glb_path.display()));
        assert!(!glb.is_empty());
        let report_path = glb_path.with_extension("publish.json");
        let report: JsonValue =
            serde_json::from_slice(&std::fs::read(&report_path).unwrap_or_else(|error| {
                panic!("could not read {}: {error}", report_path.display())
            }))
            .unwrap();
        assert_eq!(
            report
                .pointer("/contract/legacy_name")
                .and_then(JsonValue::as_str),
            Some(legacy_name)
        );
        assert_eq!(
            report
                .pointer("/contract/glb_blake3")
                .and_then(JsonValue::as_str),
            Some(blake3::hash(&glb).to_hex().as_str())
        );
        assert_eq!(
            report
                .pointer("/semanticProof/matched")
                .and_then(JsonValue::as_bool),
            Some(true)
        );
    }
}

#[test]
fn native_spawn_batch_prioritizes_gameplay_and_bounds_ambient_work() {
    let owner = Entity::from_bits(42);
    let mut queue = VecDeque::from([
        empty_effect_spawn(1, Some(owner), true, 2),
        // Gameplay can also use ExactEntityWorld (for example the
        // transportation effect parented to the local player). Priority
        // therefore follows the explicit queue class, not placement.
        empty_effect_spawn(3, Some(owner), false, 0),
        empty_effect_spawn(2, Some(owner), true, 0),
        empty_effect_spawn(5, None, false, 0),
        empty_effect_spawn(4, Some(owner), true, 0),
    ]);

    let batch = take_native_spawn_batch(&mut queue, 5, 3);

    assert_eq!(
        batch
            .iter()
            .map(NativeSpawnRequest::instance_id)
            .collect::<Vec<_>>(),
        [3, 5, 1]
    );
    assert_eq!(
        queue
            .iter()
            .map(NativeSpawnRequest::instance_id)
            .collect::<Vec<_>>(),
        [2, 4]
    );
}

#[test]
fn stale_stream_requests_are_purged_before_ambient_budgeting() {
    let unloading_owner = Entity::from_bits(41);
    let live_owner = Entity::from_bits(42);
    let mut queue = VecDeque::from([
        empty_effect_spawn(1, Some(unloading_owner), true, 99),
        empty_effect_spawn(2, Some(live_owner), true, 0),
        empty_effect_spawn(3, None, false, 0),
        empty_effect_spawn(4, Some(live_owner), true, 0),
    ]);

    let stale = purge_stale_native_spawn_requests(
        &mut queue,
        |instance_id| instance_id != 4,
        |owner| owner == live_owner,
    );
    assert_eq!(stale, [1, 4]);
    assert_eq!(
        queue
            .iter()
            .map(NativeSpawnRequest::instance_id)
            .collect::<Vec<_>>(),
        [2, 3]
    );

    let batch = take_native_spawn_batch(&mut queue, 2, 1);
    assert_eq!(
        batch
            .iter()
            .map(NativeSpawnRequest::instance_id)
            .collect::<Vec<_>>(),
        [3, 2]
    );
}

#[test]
fn exact_entity_attachment_does_not_reclassify_gameplay_as_ambient() {
    let owner = Entity::from_bits(42);
    let placement = TutorialEffectPlacement::ExactEntityWorld {
        root_entity: owner,
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };
    assert_eq!(streamed_effect_ownership(false, &placement), (false, None));
    assert_eq!(
        streamed_effect_ownership(true, &placement),
        (true, Some(owner))
    );
}

#[test]
fn unloading_stream_cleanup_leaves_rooted_entities_to_the_world_budget() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<TutorialEffectRuntime>()
        .add_systems(Update, cleanup_stream_owned_native_effects);
    let owner = app
        .world_mut()
        .spawn(crate::world::PendingNativeWorldSceneUnload::default())
        .id();
    let instance_id = {
        let mut runtime = app.world_mut().resource_mut::<TutorialEffectRuntime>();
        let instance_id = runtime.allocate_instance(false, None);
        runtime.active.get_mut(&instance_id).unwrap().stream_owner = Some(owner);
        instance_id
    };
    let rooted_effect = app
        .world_mut()
        .spawn((
            NativeRoot { instance_id },
            NativeEffectRoot {
                stream_owner: Some(owner),
                age: 10.0,
                destroy_after: Some(0.0),
                natural_destroy_after: Some(0.0),
                material_animation: None,
                waiting_for_mesh_surface: false,
            },
            ChildOf(owner),
        ))
        .id();

    app.add_systems(Update, cleanup_effect_roots);

    app.update();

    assert!(
        !app.world()
            .resource::<TutorialEffectRuntime>()
            .active
            .contains_key(&instance_id)
    );
    assert!(app.world().get_entity(rooted_effect).is_ok());
}

#[test]
fn detached_stream_particle_cleanup_is_cardinality_bounded() {
    let mut remaining = STREAMED_WORLD_PARTICLE_DESPAWNS_PER_FRAME;
    for _ in 0..STREAMED_WORLD_PARTICLE_DESPAWNS_PER_FRAME {
        assert!(admit_particle_despawn(true, &mut remaining));
    }
    assert!(!admit_particle_despawn(true, &mut remaining));
    assert!(
        admit_particle_despawn(false, &mut remaining),
        "gameplay particle cleanup is independent from ambient teardown"
    );
}

#[test]
fn eruption_removes_geometry_at_emission_end_and_retains_particle_tail() {
    use crate::assets::AssetLocator;
    use std::time::Duration;
    let root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root_path).unwrap();
    let plans = native_catalog::open(&locator).unwrap();
    let plan = &plans[&766];
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<TutorialEffectRuntime>()
        .add_systems(
            Update,
            (cleanup_effect_roots, cleanup_effect_meshes).chain(),
        );
    let root = app
        .world_mut()
        .spawn(NativeEffectRoot {
            stream_owner: None,
            age: 0.0,
            destroy_after: None,
            natural_destroy_after: Some(plan.maximum_timer + plan.longest_lifetime),
            material_animation: None,
            waiting_for_mesh_surface: true,
        })
        .id();
    let mesh = app
        .world_mut()
        .spawn((ChildOf(root), NativeMeshEffectLifetime::from_plan(plan)))
        .id();
    let particle = app.world_mut().spawn(ChildOf(root)).id();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(3));
    app.update();
    assert!(
        app.world().get_entity(mesh).is_ok(),
        "loading must not consume effect lifetime"
    );
    app.world_mut()
        .get_mut::<NativeEffectRoot>(root)
        .unwrap()
        .waiting_for_mesh_surface = false;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(1));
    app.update();
    assert!(
        app.world().get_entity(mesh).is_ok(),
        "source uses strictly greater than maxTimer"
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(50));
    app.update();
    assert!(app.world().get_entity(mesh).is_err());
    assert!(app.world().get_entity(particle).is_ok());
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(1));
    app.update();
    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().get_entity(particle).is_err());
}

#[test]
fn streamed_world_effect_distance_is_independent_of_view_angle_and_rejects_far_effects() {
    let camera = GlobalTransform::IDENTITY;
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: 60.0_f32.to_radians(),
        far: 300.0,
        aspect_ratio: 16.0 / 9.0,
        ..default()
    });

    assert!(world_effect_is_within_camera_distance(
        Vec3::new(0.0, 0.0, -100.0),
        &camera,
        &projection,
    ));
    assert!(world_effect_is_within_camera_distance(
        Vec3::new(0.0, 0.0, 100.0),
        &camera,
        &projection,
    ));
    assert!(world_effect_is_within_camera_distance(
        Vec3::new(100.0, 0.0, 0.0),
        &camera,
        &projection,
    ));
    assert!(!world_effect_is_within_camera_distance(
        Vec3::new(0.0, 0.0, -400.0),
        &camera,
        &projection,
    ));
}

#[test]
fn extended_world_camera_does_not_expand_ambient_effect_simulation() {
    let camera = GlobalTransform::IDENTITY;
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: 60.0_f32.to_radians(),
        far: 340.0,
        aspect_ratio: 16.0 / 9.0,
        ..default()
    });

    assert!(world_effect_is_within_camera_distance(
        Vec3::new(0.0, 0.0, -331.0),
        &camera,
        &projection,
    ));
    assert!(!world_effect_is_within_camera_distance(
        Vec3::new(0.0, 0.0, -333.0),
        &camera,
        &projection,
    ));
}

#[test]
fn streamed_particle_keeps_simulating_behind_the_camera() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let compiled = compile_effect_plan(771, library.effects.get(&771).unwrap());
    let mut emitter = compiled
        .plan
        .unwrap_or_else(|| panic!("ES771 native blockers: {:#?}", compiled.blockers))
        .emitters
        .into_iter()
        .next()
        .expect("ES771 particle emitter");
    emitter.force = Vec3::ZERO;
    emitter.lifetime = 5.0;

    let mut app = App::new();
    app.init_resource::<Time>()
        .add_systems(Update, simulate_particles);
    let camera_target = app.world_mut().spawn_empty().id();
    app.world_mut().spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            far: 300.0,
            aspect_ratio: 16.0 / 9.0,
            ..default()
        }),
        LegacyOrbitCamera::new(camera_target),
        GlobalTransform::IDENTITY,
    ));
    let stream_owner = app.world_mut().spawn_empty().id();
    let particle_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 100.0),
            Visibility::Inherited,
            NativeParticle {
                stream_owner: Some(stream_owner),
                plan: Arc::new(emitter),
                scale: 1.0,
                age: 0.0,
                velocity: Vec3::X * 4.0,
                stream_owned: true,
                alive: true,
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.25));

    app.update();

    let entity = app.world().entity(particle_entity);
    assert_eq!(*entity.get::<Visibility>().unwrap(), Visibility::Inherited);
    assert!((entity.get::<NativeParticle>().unwrap().age - 0.25).abs() < 1.0e-6);
    assert!(entity.get::<Transform>().unwrap().translation.x > 0.9);
}

#[test]
fn sector_v_publishes_the_clean_buttercup_long_lived_hologram() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/map/tiles/map_07_08/behaviour.json");
    let document: JsonValue = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let emitters = document["effectEmitters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|emitter| emitter["effectName"].as_str() == Some("buttercup_hologram"))
        .collect::<Vec<_>>();
    let [emitter] = emitters.as_slice() else {
        panic!("Sector V must publish exactly one Buttercup hologram emitter");
    };
    assert_eq!(emitter["node"], "BuildPlayer-Map_07_08#12703");
    assert_eq!(emitter["maxTimer"].as_f64(), Some(-2.0));
    assert_eq!(emitter["longestLifeTime"].as_f64(), Some(600.0));
    assert_eq!(emitter["conformToScale"].as_bool(), Some(true));
    assert_eq!(
        emitter["resolvedParticlePrefabs"].as_array().unwrap()[0],
        "CustomAssetBundle-fa9dbcf4604f64024b06ff1d5e375918#8991"
    );

    let closure_id = emitter["resolvedParticlePrefabs"].as_array().unwrap()[0]
        .as_str()
        .unwrap();
    let closure = document["effectPrefabClosures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|closure| closure["id"].as_str() == Some(closure_id))
        .expect("Buttercup prefab closure");
    let particle = closure["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|object| {
            object["objectType"].as_str() == Some("MonoBehaviour")
                && object["value"]["particleName"].as_str() == Some("buttercup_hologram")
        })
        .expect("Buttercup body particle");
    assert!(
        (particle["value"]["generationsPerSecond"].as_f64().unwrap() - 1.0 / 600.0).abs()
            < 1.0e-9
    );
    assert_eq!(particle["value"]["lifeTime"].as_f64(), Some(600.0));
}

#[test]
fn persistent_landmark_particle_is_not_lost_when_ambient_budget_is_full() {
    let mut remaining = 0;
    assert_eq!(streamed_particle_spawn_count(1, &mut remaining, true), 1);
    assert_eq!(remaining, 0);

    let mut ordinary_remaining = 0;
    assert_eq!(
        streamed_particle_spawn_count(1, &mut ordinary_remaining, false),
        0
    );
}

#[test]
fn argb4444_decoder_preserves_legacy_channel_order() {
    let source = [0x23, 0xf1, 0xbc, 0x4a];
    let decoded = decode_argb4444_level(&source, 2, 1).unwrap();
    assert_eq!(decoded, [17, 34, 51, 255, 170, 187, 204, 68]);
    assert!(decode_argb4444_level(&source[..2], 2, 1).is_err());
}

#[test]
fn compressed_decoders_fail_closed_on_malformed_levels() {
    assert!(decode_bc1_level(&[0_u8; 7], 4, 4).is_err());
    assert!(decode_bc1_level(&[0_u8; 8], 5, 4).is_err());
    assert!(decode_bc1_level(&[0_u8; 8], 0, 4).is_err());
    assert!(decode_bc3_level(&[0_u8; 15], 4, 4).is_err());
    assert!(decode_bc3_level(&[0_u8; 16], 4, 0).is_err());
}

#[test]
fn legacy_billboard_modes_keep_their_authored_world_axis_constraints() {
    let camera = GlobalTransform::from(
        Transform::from_xyz(4.0, 3.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
    );

    let vertical = particle_billboard_rotation(
        LegacyParticleRenderMode::VerticalBillboard,
        &camera,
        Vec3::ZERO,
        0.0,
    );
    let horizontal_to_camera = Vec3::new(4.0, 0.0, -6.0).normalize();
    assert!((vertical * Vec3::Y).abs_diff_eq(Vec3::Y, 1.0e-6));
    assert!((vertical * Vec3::Z).abs_diff_eq(horizontal_to_camera, 1.0e-6));

    let horizontal = particle_billboard_rotation(
        LegacyParticleRenderMode::HorizontalBillboard,
        &camera,
        Vec3::ZERO,
        0.0,
    );
    assert!((horizontal * Vec3::Z).abs_diff_eq(Vec3::Y, 1.0e-6));

    let full = particle_billboard_rotation(
        LegacyParticleRenderMode::Billboard,
        &camera,
        Vec3::ZERO,
        0.0,
    );
    assert!(
        !(full * Vec3::Y).abs_diff_eq(Vec3::Y, 1.0e-3),
        "a full billboard must retain camera pitch"
    );
}

#[test]
fn every_past_particle_program_requires_the_primary_black_fog_stage() {
    let asset_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/map/tiles");
    let mut audited_programs = BTreeSet::new();

    // The user's affected set is the clean-primary Past Suburbs cluster.
    // Read the production publications rather than a hand-written shader
    // list so a newly imported Past effect cannot silently bypass fog.
    for tile in [
        "map_04_06",
        "map_04_07",
        "map_05_06",
        "map_05_07",
        "map_06_06",
        "map_07_06",
        "map_07_07",
        "map_07_08",
        "map_08_05",
        "map_08_06",
        "map_08_07",
        "map_09_06",
        "map_09_07",
        "map_09_08",
    ] {
        let path = asset_root.join(tile).join("behaviour.json");
        let document: JsonValue =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        for closure in document["effectPrefabClosures"].as_array().unwrap() {
            for object in closure["objects"].as_array().unwrap() {
                if object["objectType"].as_str() != Some("Shader") {
                    continue;
                }
                let Some(script) = object["value"]["m_Script"].as_str() else {
                    continue;
                };
                let shader_name = exact_shader_name(&TutorialUnityObjectProof {
                    asset: object["asset"].as_str().unwrap().to_owned(),
                    path_id: object["pathId"].as_i64().unwrap(),
                    type_id: object["typeId"].as_i64().unwrap(),
                    class_id: object["classId"].as_i64().unwrap(),
                    object_type: "Shader".to_owned(),
                    name: object["name"].as_str().unwrap_or_default().to_owned(),
                    canonical_blake3: object["canonicalBlake3"].as_str().unwrap().to_owned(),
                    value: object["value"].clone(),
                })
                .unwrap();
                if particle_blend_mode(&shader_name).is_none() {
                    continue;
                }
                assert!(
                    script.contains("Fog { Color (0,0,0,0) }"),
                    "{tile} publishes particle program {shader_name} without the primary black fog stage"
                );
                audited_programs.insert(shader_name);
            }
        }
    }

    assert_eq!(audited_programs.len(), 7);
}

#[test]
fn nuclear_plant_galadriel_is_an_authored_huge_fogged_particle_card() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/map/tiles/map_04_07/behaviour.json");
    let document: JsonValue = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let emitter = document["effectEmitters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|emitter| emitter["node"].as_str() == Some("BuildPlayer-Map_04_07#22636"))
        .expect("clean Nuclear Plant galadriel emitter");
    assert_eq!(emitter["conformToScale"].as_bool(), Some(true));
    let authored_scale = emitter["worldMatrix"][1][1].as_f64().unwrap();
    assert!((authored_scale - 17.300_003_051_757_812).abs() < 1.0e-9);

    let closure_id = emitter["resolvedParticlePrefabs"][0].as_str().unwrap();
    let closure = document["effectPrefabClosures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|closure| closure["id"].as_str() == Some(closure_id))
        .expect("galadriel prefab closure");
    let objects = closure["objects"].as_array().unwrap();
    let controller = objects
        .iter()
        .find(|object| {
            object["objectType"].as_str() == Some("MonoBehaviour")
                && object["value"]["particleName"].as_str() == Some("green")
        })
        .expect("galadriel particle controller");
    assert_eq!(controller["value"]["initialSize"].as_f64(), Some(1.0));
    let renderer = objects
        .iter()
        .find(|object| object["objectType"].as_str() == Some("ParticleRenderer"))
        .expect("galadriel particle renderer");
    let width_curve = renderer["value"]["m_WidthCurve"]["m_Curve"]
        .as_array()
        .unwrap();
    assert_eq!(width_curve.last().unwrap()["value"].as_f64(), Some(10.0));
    assert!(objects.iter().any(|object| {
        object["objectType"].as_str() == Some("Texture2D")
            && object["name"].as_str() == Some("galadriel")
    }));
    let source_shader = objects
        .iter()
        .find(|object| object["objectType"].as_str() == Some("Shader"))
        .and_then(|object| object["value"]["m_Script"].as_str())
        .expect("galadriel source shader");
    assert!(source_shader.contains("Blend One One"));
    assert!(source_shader.contains("Fog { Color (0,0,0,0) }"));
}
