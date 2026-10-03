use super::*;

#[test]
fn area51_ship_vortex_keeps_rotating_after_first_cycle() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let document =
        load_native_world_behaviours(&root, NativeWorldScope::WorldMap, "map_12_10").unwrap();
    let record = document
        .animations
        .iter()
        .find(|a| a.models.iter().any(|m| m == "static-map_12_10-01067"))
        .unwrap();
    let clip = document
        .animation_clips
        .iter()
        .find(|c| Some(c.id.as_str()) == record.default_clip_id.as_deref())
        .unwrap();
    assert_eq!(record.models.len(), 12);
    assert_eq!(clip.channels.len(), 12);
    assert!((clip.duration - 3.99).abs() < 1e-5);
    let repeats = world_animation_transform_repeats(Some(clip), record.wrap_mode);
    assert!(repeats);
    for cycle in [1., 2., 100.] {
        assert!(
            (world_animation_sample_time(cycle * clip.duration + 0.43, clip.duration, repeats)
                - 0.43)
                .abs()
                < 1e-5
        );
    }
    for channel in &clip.channels {
        let period = *channel.times.last().unwrap();
        assert_eq!(channel.post_infinity, 2);
        let expected = sample_world_animation_vector_fixed::<4>(channel, 0.2).unwrap();
        for cycles in [1., 3., 17.] {
            let elapsed = cycles * period + 0.2;
            let sample = world_channel_sample_time(
                channel,
                elapsed,
                world_animation_sample_time(elapsed, clip.duration, true),
            );
            let actual = sample_world_animation_vector_fixed::<4>(channel, sample).unwrap();
            assert!(
                expected
                    .iter()
                    .zip(actual)
                    .all(|(a, b)| (a - b).abs() < 1e-5)
            );
        }
        if matches!(
            channel.target_path.as_str(),
            "Cylinder06" | "Cylinder12" | "Cylinder13"
        ) {
            assert_eq!(period, 1.0);
            let first: [f64; 4] = channel.values[0].clone().try_into().unwrap();
            let last: [f64; 4] = channel.values.last().unwrap().clone().try_into().unwrap();
            let first = Quat::from_array(first.map(|v| v as f32));
            let last = Quat::from_array(last.map(|v| v as f32));
            assert!(first.normalize().dot(last.normalize()).abs() > 0.99999);
        }
    }
}

#[test]
fn organized_scene_names_keep_the_shared_source_node() {
    let visual = Name::new("platform [BuildPlayer-Map_08_06#11153 visual]");
    let collision = Name::new("platform [BuildPlayer-Map_08_06#11153 collision]");
    assert_eq!(
        organized_world_source_node(&visual),
        Some("BuildPlayer-Map_08_06#11153")
    );
    assert_eq!(
        organized_world_source_node(&collision),
        Some("BuildPlayer-Map_08_06#11153")
    );
}

#[test]
fn primary_zero_quaternion_foliage_targets_reconstruct_finite_world_matrices() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/map/tiles/map_08_06/behaviour.json");
    let document: NativeWorldBehaviourDocument =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let animation = document
        .animations
        .iter()
        .find(|animation| animation.node == "BuildPlayer-Map_08_06#13249")
        .expect("primary Candy Cove foliage animation");
    let mut worlds = HashMap::<String, Mat4>::new();
    let mut zero_quaternion_targets = Vec::new();

    for target in &animation.targets {
        let parent_world = if let Some(parent_path) = target.parent_path.as_ref() {
            *worlds
                .get(parent_path)
                .expect("parent target precedes child")
        } else {
            mat4_from_world_matrix(
                target
                    .root_parent_world_matrix
                    .as_ref()
                    .expect("root animation target retains its parent matrix"),
            )
        };
        let transform = target.base_local_trs.transform();
        let reconstructed = parent_world * transform.to_matrix();
        let expected = mat4_from_world_matrix(&target.base_world_matrix);
        assert!(
            reconstructed.is_finite(),
            "{} reconstructed a non-finite world matrix",
            target.node
        );
        assert!(
            (reconstructed - expected)
                .to_cols_array()
                .iter()
                .all(|component| component.abs() < 1.0e-3),
            "{} no longer matches the primary world matrix",
            target.node
        );
        worlds.insert(target.path.clone(), reconstructed);

        if target.base_local_trs.rotation == [0.0; 4] {
            assert_eq!(transform.rotation, Quat::IDENTITY);
            zero_quaternion_targets.push(target.node.as_str());
        }
    }

    assert_eq!(
        zero_quaternion_targets,
        [
            "BuildPlayer-Map_08_06#8712",
            "BuildPlayer-Map_08_06#8718",
            "BuildPlayer-Map_08_06#8715",
        ]
    );
}

#[test]
fn animated_parent_marks_descendant_renderer_bindings_dynamic() {
    let parent_entity = Entity::from_raw_u32(1).unwrap();
    let child_entity = Entity::from_raw_u32(2).unwrap();
    let parent_model = Entity::from_raw_u32(3).unwrap();
    let child_model = Entity::from_raw_u32(4).unwrap();
    let targets = vec![
        WorldAnimationTargetRecord {
            path: "root".to_owned(),
            node: "root".to_owned(),
            parent_path: None,
            animated: true,
            base_local_trs: WorldAnimationTrs {
                translation: [0.0; 3],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0; 3],
            },
            base_world_matrix: Mat4::IDENTITY
                .to_cols_array_2d()
                .map(|column| column.map(f64::from)),
            root_parent_world_matrix: None,
            models: Vec::new(),
        },
        WorldAnimationTargetRecord {
            path: "root/child".to_owned(),
            node: "child".to_owned(),
            parent_path: Some("root".to_owned()),
            animated: false,
            base_local_trs: WorldAnimationTrs {
                translation: [0.0; 3],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0; 3],
            },
            base_world_matrix: Mat4::IDENTITY
                .to_cols_array_2d()
                .map(|column| column.map(f64::from)),
            root_parent_world_matrix: None,
            models: Vec::new(),
        },
    ];
    let bindings = HashMap::from([
        (
            "root".to_owned(),
            WorldAnimationTargetBinding {
                entity: parent_entity,
                model_entities: vec![parent_model],
            },
        ),
        (
            "root/child".to_owned(),
            WorldAnimationTargetBinding {
                entity: child_entity,
                model_entities: vec![child_model],
            },
        ),
    ]);
    let tracks = vec![WorldAnimationCompiledTrack {
        entity: parent_entity,
        translation: Some(0),
        rotation: None,
        scale: None,
    }];

    let dynamic = world_animation_dynamic_model_entities(&targets, &bindings, &tracks);
    assert_eq!(dynamic, HashSet::from([parent_model, child_model]));
}

#[test]
fn trigger_kinds_cover_every_published_class() {
    for (kind, expected) in [
        ("ring", WorldTriggerKind::Ring),
        ("jumppad", WorldTriggerKind::Jumppad),
        ("platform", WorldTriggerKind::Platform),
        ("launcher", WorldTriggerKind::Launcher),
        ("zipline", WorldTriggerKind::Zipline),
        ("switch", WorldTriggerKind::Switch),
        ("rope", WorldTriggerKind::Rope),
        ("belt", WorldTriggerKind::Belt),
        ("slope", WorldTriggerKind::Slope),
        ("synchronizer", WorldTriggerKind::Synchronizer),
    ] {
        assert_eq!(WorldTriggerKind::from_document(kind), expected, "{kind}");
    }
    assert_eq!(
        WorldTriggerKind::from_document("unknown"),
        WorldTriggerKind::Other
    );
}

#[test]
fn clean_ring_reset_assigns_one_based_ids_only_to_active_ring_children() {
    let mut next_ring_server_id = 1;
    assert_eq!(
        clean_initial_trigger_server_id(
            WorldTriggerKind::Ring,
            true,
            -1,
            &mut next_ring_server_id,
        ),
        1
    );
    assert_eq!(
        clean_initial_trigger_server_id(
            WorldTriggerKind::Ring,
            false,
            -1,
            &mut next_ring_server_id,
        ),
        -1
    );
    assert_eq!(
        clean_initial_trigger_server_id(
            WorldTriggerKind::Ring,
            true,
            -1,
            &mut next_ring_server_id,
        ),
        2
    );
    assert_eq!(
        clean_initial_trigger_server_id(
            WorldTriggerKind::Platform,
            true,
            47,
            &mut next_ring_server_id,
        ),
        47
    );
    assert_eq!(next_ring_server_id, 3);
}

#[test]
fn behaviour_document_paths_share_the_unified_map_namespace() {
    assert_eq!(
        behaviour_document_path(NativeWorldScope::WorldMap, "map_03_04"),
        "map/tiles/map_03_04/behaviour.json"
    );
    assert_eq!(
        behaviour_document_path(NativeWorldScope::Tutorial, "tile_00_00"),
        "map/tiles/map_00_00/behaviour.json"
    );
}

#[test]
fn streamed_behaviour_hash_verification_fails_closed_on_stale_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("behaviour.json");
    std::fs::write(&path, b"original").unwrap();
    let expected = blake3::hash(b"original").to_hex().to_string();
    assert_eq!(
        read_hashed_world_behaviour_asset(&path, &expected, "world behaviour").unwrap(),
        b"original"
    );

    std::fs::write(&path, b"stale-or-tampered").unwrap();
    let error = read_hashed_world_behaviour_asset(&path, &expected, "world behaviour")
        .expect_err("mutated scripted data must not enter the runtime");
    assert!(error.contains("hash mismatch"), "{error}");
}

pub(super) fn trigger_volume(shape: WorldTriggerVolumeShape) -> WorldTriggerVolume {
    WorldTriggerVolume {
        shape,
        center: Vec3::ZERO,
        radius: 1.0,
        size: Vec3::splat(2.0),
        height: 4.0,
        direction: 1,
        is_trigger: true,
        trigger: None,
    }
}

#[test]
fn platform_line_uses_the_clean_eased_ping_pong_curve() {
    let motion = WorldPlatformMotion {
        initial_translation: Vec3::ZERO,
        initial_rotation: Quat::IDENTITY,
        initial_scale: Vec3::ONE,
        from: Vec3::ZERO,
        to: Vec3::new(0.0, 8.0, 0.0),
        velocity: 0.15,
        spin_velocity: 1.0,
        move_type: 0,
        path_points: Vec::new(),
    };
    assert_eq!(
        legacy_platform_source_position(&motion, 0.0, false),
        Vec3::ZERO
    );
    assert!(
        legacy_platform_source_position(&motion, 0.5, false).abs_diff_eq(Vec3::Y * 4.0, 1e-5)
    );
    assert!(
        legacy_platform_source_position(&motion, 1.0, false).abs_diff_eq(Vec3::Y * 8.0, 1e-5)
    );
}

#[test]
fn platform_endpoints_rotate_the_source_delta_but_do_not_scale_it() {
    let motion = WorldPlatformMotion {
        initial_translation: Vec3::new(-2_258.576, -39.637, 1_814.803),
        initial_rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        initial_scale: Vec3::splat(1.25),
        from: Vec3::new(2.0, 0.0, 0.0),
        to: Vec3::new(2.0, 8.0, 0.0),
        velocity: 0.1,
        spin_velocity: 1.0,
        move_type: 0,
        path_points: Vec::new(),
    };
    assert!(
        legacy_platform_world_translation(&motion, 0.0, false)
            .abs_diff_eq(motion.initial_translation, 1e-5)
    );
    let source_endpoint =
        motion.initial_translation + motion.initial_rotation * (motion.to - motion.from);
    assert!(
        legacy_platform_world_translation(&motion, 1.0, false)
            .abs_diff_eq(source_endpoint, 1e-5)
    );
}

#[test]
fn animate_if_visible_pauses_until_a_bound_renderer_is_visible() {
    assert!(world_animation_clock_is_paused(true, false, false, false));
    assert!(world_animation_clock_is_paused(true, true, true, false));
    assert!(!world_animation_clock_is_paused(true, true, true, true));
    assert!(!world_animation_clock_is_paused(false, true, true, false));
}

#[test]
fn rope_and_belt_paths_use_ordered_segment_distance() {
    let points = [Vec3::ZERO, Vec3::X * 3.0, Vec3::new(3.0, 4.0, 0.0)];
    assert_eq!(polyline_length(&points), 7.0);
    let (position, direction) = sample_polyline(&points, 5.0).unwrap();
    assert!(position.abs_diff_eq(Vec3::new(3.0, 2.0, 0.0), 1e-6));
    assert!(direction.abs_diff_eq(Vec3::Y, 1e-6));
    assert!((nearest_polyline_distance(&points, Vec3::new(2.0, 1.0, 0.0)) - 2.0).abs() < 1e-6);
}

#[test]
fn trigger_volume_occupancy_uses_the_exact_authored_shape() {
    let sphere = trigger_volume(WorldTriggerVolumeShape::Sphere);
    assert!(trigger_volume_contains_local_point(
        &sphere,
        Vec3::new(0.5, 0.5, 0.5)
    ));
    assert!(!trigger_volume_contains_local_point(
        &sphere,
        Vec3::new(1.0, 1.0, 0.0)
    ));

    let box_volume = trigger_volume(WorldTriggerVolumeShape::Box);
    assert!(trigger_volume_contains_local_point(
        &box_volume,
        Vec3::new(0.9, 0.9, 0.9)
    ));
    assert!(!trigger_volume_contains_local_point(
        &box_volume,
        Vec3::new(1.1, 0.0, 0.0)
    ));

    let capsule = trigger_volume(WorldTriggerVolumeShape::Capsule);
    assert!(trigger_volume_contains_local_point(
        &capsule,
        Vec3::new(0.0, 1.9, 0.0)
    ));
    assert!(!trigger_volume_contains_local_point(
        &capsule,
        Vec3::new(1.1, 0.0, 0.0)
    ));
}

#[test]
fn visibility_switch_document_preserves_the_switchable_billboard_node() {
    let record: VisibilitySwitchRecord = serde_json::from_value(serde_json::json!({
        "node": "detail-root",
        "enabled": true,
        "models": ["detail-model"],
        "switches": "billboard-root"
    }))
    .unwrap();
    assert_eq!(record.models, ["detail-model"]);
    assert_eq!(record.switches, "billboard-root");
}

#[test]
fn peach_creek_billboards_do_not_start_before_their_visibility_switches() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/map/tiles/map_08_07/behaviour.json");
    let document: NativeWorldBehaviourDocument =
        serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
    assert_eq!(document.billboards.len(), 47);
    assert_eq!(document.visibility_switches.len(), 47);
    assert!(document.billboards.iter().all(|record| !record.enabled));
    assert!(
        document
            .visibility_switches
            .iter()
            .all(|record| record.enabled)
    );
    assert!(document.billboards.iter().all(|billboard| {
        document
            .visibility_switches
            .iter()
            .any(|switch| switch.switches == billboard.node)
    }));
}

#[test]
fn gpu_uv_compiler_preserves_authored_one_shots_and_non_affine_curves() {
    let one_shot = material_clip(
        "authored-pulse",
        false,
        vec![linear_material_curve(
            "surface",
            "_MainTex.offset.x",
            2.0,
            0.5,
        )],
    );
    assert!(compile_world_gpu_uv_animation(&one_shot, "surface", 0, true, 0.0).is_none());

    let mut nonlinear = linear_material_curve("surface", "_MainTex.offset.x", 2.0, 0.5);
    nonlinear.values[2] = 0.75;
    let nonlinear = material_clip("nif-default", false, vec![nonlinear]);
    assert!(compile_world_gpu_uv_animation(&nonlinear, "surface", 0, true, 0.0).is_none());
}

#[test]
fn world_ep_element_uses_native_basis_and_exact_effect_id() {
    let element = serde_json::json!({
        "particleIndex": 464,
        "position": { "x": 1.0, "y": 2.0, "z": 3.0 },
        "rotation": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
        "scale": 4.0
    });
    let command = world_ep_effect_command(Entity::PLACEHOLDER, None, &element)
        .unwrap()
        .unwrap();
    let TutorialEffectRuntimeCommand::Add {
        effect_id,
        placement,
        scale,
        ..
    } = command
    else {
        panic!("world EP element did not produce an Add command");
    };
    assert_eq!(effect_id, 464);
    assert!((scale - 4.0).abs() < f32::EPSILON);
    let TutorialEffectPlacement::ExactEntityWorld {
        root_entity,
        position,
        rotation,
    } = placement
    else {
        panic!("world EP element is not owned by the streamed tile");
    };
    assert_eq!(root_entity, Entity::PLACEHOLDER);
    assert!((position - Vec3::new(-1.0, 2.0, 3.0)).length() < f32::EPSILON);
    assert!(rotation.angle_between(Quat::IDENTITY).abs() < f32::EPSILON);
}

#[test]
fn null_legacy_particle_prefabs_are_exact_no_particle_entries() {
    let mut record: EffectEmitterRecord = serde_json::from_value(serde_json::json!({
        "node": "recordbuild",
        "enabled": true,
        "controller": "EffectEmitterController",
        "particles": [{}, {}],
        "resolvedParticlePrefabs": [null, null]
    }))
    .unwrap();
    assert!(!has_reproducible_world_particle_prefab(&record));
    record.resolved_particle_prefabs[1] = Some("prefab#2".to_owned());
    assert!(has_reproducible_world_particle_prefab(&record));
}

#[test]
fn every_published_infected_zone_barrier_remains_a_reproducible_effect() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/map/tiles");
    let mut barrier_count = 0;
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path().join("behaviour.json");
        let document: NativeWorldBehaviourDocument =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for record in document
            .effect_emitters
            .iter()
            .filter(|record| record.effect_name.as_deref() == Some("EPbarrier"))
        {
            barrier_count += 1;
            assert!(reproducible_world_scripted_effect(record));
            assert!(record.resolved_particle_prefabs.iter().any(Option::is_some));
        }
    }
    assert_eq!(barrier_count, 799);
}

#[test]
#[ignore = "full published asset closure audit"]
fn every_published_world_effect_builds_a_complete_native_plan() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/map/tiles");
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", root.display()))
    {
        let path = entry
            .expect("invalid map tile directory entry")
            .path()
            .join("behaviour.json");
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();

    let mut document_count = 0;
    let mut scripted_effect_count = 0;
    let mut ep_effect_count = 0;
    for path in paths {
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
        let document: NativeWorldBehaviourDocument = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("could not parse {}: {error}", path.display()));
        assert_eq!(
            document.schema,
            NATIVE_WORLD_BEHAVIOUR_SCHEMA,
            "{}",
            path.display()
        );
        assert!(
            document.blockers.is_empty(),
            "{} still has extraction blockers: {:?}",
            path.display(),
            document.blockers
        );
        document_count += 1;

        let closures = document
            .effect_prefab_closures
            .iter()
            .map(|closure| (closure.id.as_str(), closure))
            .collect::<HashMap<_, _>>();
        for record in document.effect_emitters.iter().filter(|record| {
            record.enabled
                && record.controller == "EffectEmitterController"
                && !record.particles.is_empty()
                && has_reproducible_world_particle_prefab(record)
        }) {
            let closure = world_scripted_effect_closure(&document, record, &closures)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} scripted effect {:?} has an invalid closure: {error}",
                        path.display(),
                        record.node
                    )
                });
            let (placement, scale) =
                world_scripted_effect_placement(Entity::PLACEHOLDER, record).unwrap_or_else(
                    |error| {
                        panic!(
                            "{} scripted effect {:?} has invalid placement: {error}",
                            path.display(),
                            record.node
                        )
                    },
                );
            TutorialEffectRuntime::default()
                .enqueue_world_serialized_effect(closure, placement, scale)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} scripted effect {:?} did not compile: {error}",
                        path.display(),
                        record.node
                    )
                });
            scripted_effect_count += 1;
        }

        for record in document
            .effect_emitters
            .iter()
            .filter(|record| record.enabled && record.controller == "EPElementController")
        {
            for element in &record.particle_elements {
                if world_ep_effect_command(
                    Entity::PLACEHOLDER,
                    record.world_matrix.as_ref(),
                    element,
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "{} EP effect {:?} did not compile: {error}",
                        path.display(),
                        record.node
                    )
                })
                .is_some()
                {
                    ep_effect_count += 1;
                }
            }
        }
    }

    assert_eq!(document_count, 170);
    assert!(scripted_effect_count > 0);
    assert!(ep_effect_count > 0);
    eprintln!(
        "audited {document_count} documents, {scripted_effect_count} scripted effects and {ep_effect_count} EP effects"
    );
}
