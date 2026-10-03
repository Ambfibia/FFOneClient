use super::*;

pub(super) fn parameters(bullet_type: i32) -> TutorialBulletParameters {
    TutorialBulletParameters {
        cancel_script: 0,
        fire_script: 0,
        particle_script: if bullet_type == 76 { 15 } else { 391 },
        success_script: 0,
        cancel_model_scale: 0.0,
        curve_height: 0.0,
        fire_model_scale: 0.0,
        bullet_model_scale: 1.0,
        success_model_scale: 0.0,
        hide_time_seconds: 0.0,
        maximum_time_seconds: 2.0,
        fire_link: "\"".to_owned(),
        success_link: "\"".to_owned(),
        success_sound: "\"".to_owned(),
    }
}

pub(super) fn library() -> TutorialEffectLibrary {
    let source = TutorialSourceFileProof {
        logical_name: "source".to_owned(),
        bytes: 1,
        blake3: "source".to_owned(),
    };
    let mut effects = BTreeMap::new();
    let effect_entries = RETROBUTION_TUTORIAL_EFFECT_IDS
        .into_iter()
        .map(|effect_id| {
            let entry = TutorialEffectCatalogEntry {
                effect_id,
                container_route: format!(
                    "prefabs/particle/effectscripts/es[{effect_id}].prefab"
                ),
                root_asset: "Effects".to_owned(),
                root_path_id: i64::from(effect_id),
                closure_path: format!("map/shared/effects/es{effect_id}.closure.json"),
                closure_bytes: 1,
                closure_blake3: format!("effect-{effect_id}"),
                object_count: 1,
                object_types: vec!["GameObject".to_owned()],
                component_types: vec!["ParticleEmitter".to_owned()],
            };
            effects.insert(
                effect_id,
                ValidatedEffect {
                    entry: entry.clone(),
                    closure: TutorialEffectClosureFile {
                        schema: TUTORIAL_EFFECT_CLOSURE_SCHEMA.to_owned(),
                        effect_id: Some(effect_id),
                        container_route: entry.container_route.clone(),
                        root_asset: entry.root_asset.clone(),
                        root_path_id: i64::from(effect_id),
                        source_bundle_blake3: "source".to_owned(),
                        source_dump_blake3: "source".to_owned(),
                        source_assets: Vec::new(),
                        objects: Vec::new(),
                    },
                },
            );
            entry
        })
        .collect::<Vec<_>>();
    let rows = RETROBUTION_TUTORIAL_BULLET_TYPES
        .into_iter()
        .map(|bullet_type| TutorialBulletCatalogEntry {
            bullet_type,
            row_path: format!("map/shared/projectiles/bullet-{bullet_type}.json"),
            row_bytes: 1,
            row_blake3: format!("bullet-{bullet_type}"),
            serialized_row_blake3: format!("serialized-{bullet_type}"),
            parameters: parameters(bullet_type),
        })
        .collect::<Vec<_>>();
    let mut projectile_effects = BTreeMap::new();
    let projectile_effect_entries = RETROBUTION_TUTORIAL_PROJECTILE_EFFECT_IDS
        .into_iter()
        .map(|effect_id| {
            let entry = TutorialEffectCatalogEntry {
                effect_id,
                container_route: format!(
                    "prefabs/particle/effectscripts/es[{effect_id}].prefab"
                ),
                root_asset: "Effects".to_owned(),
                root_path_id: i64::from(effect_id),
                closure_path: format!(
                    "map/shared/projectiles/effects/es{effect_id}.closure.json"
                ),
                closure_bytes: 1,
                closure_blake3: format!("projectile-effect-{effect_id}"),
                object_count: 1,
                object_types: vec!["GameObject".to_owned()],
                component_types: vec!["ParticleEmitter".to_owned()],
            };
            projectile_effects.insert(
                effect_id,
                ValidatedEffect {
                    entry: entry.clone(),
                    closure: TutorialEffectClosureFile {
                        schema: TUTORIAL_EFFECT_CLOSURE_SCHEMA.to_owned(),
                        effect_id: Some(effect_id),
                        container_route: entry.container_route.clone(),
                        root_asset: entry.root_asset.clone(),
                        root_path_id: i64::from(effect_id),
                        source_bundle_blake3: "source".to_owned(),
                        source_dump_blake3: "source".to_owned(),
                        source_assets: Vec::new(),
                        objects: Vec::new(),
                    },
                },
            );
            entry
        })
        .collect::<Vec<_>>();
    TutorialEffectLibrary {
        root: PathBuf::from("test"),
        effect_catalog: TutorialEffectCatalog {
            schema: TUTORIAL_EFFECT_CATALOG_SCHEMA.to_owned(),
            source_build: RETROBUTION_TUTORIAL_BUILD_ID.to_owned(),
            source_bundle: source.clone(),
            source_dump: source.clone(),
            source_assets: Vec::new(),
            renderer_status: EFFECT_RENDERER_STATUS.to_owned(),
            effects: effect_entries,
        },
        projectile_catalog: TutorialProjectileCatalog {
            schema: TUTORIAL_PROJECTILE_CATALOG_SCHEMA.to_owned(),
            source_build: RETROBUTION_TUTORIAL_BUILD_ID.to_owned(),
            source_bundle: source.clone(),
            source_dump: source,
            source_assets: Vec::new(),
            renderer_status: PROJECTILE_RENDERER_STATUS.to_owned(),
            bullet_table_route: "bullettable.asset".to_owned(),
            bullet_table_root_path_id: 5,
            bullet_table_closure_path: "map/shared/projectiles/bullettable.closure.json"
                .to_owned(),
            bullet_table_closure_bytes: 1,
            bullet_table_closure_blake3: "table".to_owned(),
            particle_effects: projectile_effect_entries,
            rows,
        },
        effects,
        projectile_effects,
        bullets: BTreeMap::new(),
    }
}

#[test]
fn repeated_actor_effect_callback_replaces_the_same_current_effect_slot() {
    let mut runtime = TutorialEffectRuntime::default();
    let name = retrobution_actor_current_effect_name(204, 734);
    let first = runtime.allocate_instance(false, Some(name.clone()));
    let second = runtime.allocate_instance(false, Some(name.clone()));

    assert!(!runtime.active.contains_key(&first));
    assert!(runtime.active.contains_key(&second));
    assert_eq!(runtime.named.get(&name), Some(&second));
    assert_eq!(runtime.native_despawns, VecDeque::from([first]));
}

#[test]
fn streamed_world_effect_admission_is_bounded_behind_gameplay() {
    let mut runtime = TutorialEffectRuntime::default();
    for index in 0..(STREAMED_WORLD_EFFECT_COMMANDS_PER_FRAME + 3) {
        runtime.enqueue_streamed_world_effect(TutorialEffectRuntimeCommand::Add {
            effect_id: 464,
            placement: TutorialEffectPlacement::ExactEntityWorld {
                root_entity: Entity::PLACEHOLDER,
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            },
            scale: 1.0,
            tracked: false,
            name: None,
            destroy_after_seconds: None,
            source_line: 100 + index as u32,
        });
    }
    runtime.enqueue(TutorialEffectRuntimeCommand::ClearTracked { source_line: 7 });

    runtime.process_pending();

    let records = runtime.drain_records().collect::<Vec<_>>();
    assert_eq!(records.len(), 1 + STREAMED_WORLD_EFFECT_COMMANDS_PER_FRAME);
    assert!(matches!(
        records[0].command,
        TutorialEffectRuntimeCommand::ClearTracked { source_line: 7 }
    ));
    for (index, record) in records[1..].iter().enumerate() {
        assert_eq!(record.command.source_line(), 100 + index as u32);
    }
    assert_eq!(runtime.streamed_world_pending.len(), 3);

    runtime.clear_scene_instances();
    assert!(runtime.streamed_world_pending.is_empty());
}

#[test]
fn unloaded_tile_commands_do_not_consume_live_ambient_admission() {
    let dead_owner = Entity::from_bits(41);
    let live_owner = Entity::from_bits(42);
    let mut runtime = TutorialEffectRuntime::default();
    for index in 0..(STREAMED_WORLD_EFFECT_COMMANDS_PER_FRAME * 2) {
        runtime.enqueue_streamed_world_effect(TutorialEffectRuntimeCommand::Add {
            effect_id: 464,
            placement: TutorialEffectPlacement::ExactEntityWorld {
                root_entity: dead_owner,
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            },
            scale: 1.0,
            tracked: false,
            name: None,
            destroy_after_seconds: None,
            source_line: index as u32,
        });
    }
    runtime.enqueue_streamed_world_effect(TutorialEffectRuntimeCommand::Add {
        effect_id: 464,
        placement: TutorialEffectPlacement::ExactEntityWorld {
            root_entity: live_owner,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        },
        scale: 1.0,
        tracked: false,
        name: None,
        destroy_after_seconds: None,
        source_line: 999,
    });

    runtime.process_pending_with_stream_liveness(|owner| owner == live_owner);

    let records = runtime.drain_records().collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].command.source_line(), 999);
    assert!(runtime.streamed_world_pending.is_empty());
}

#[test]
fn unload_marker_is_terminal_stream_owner_liveness_before_despawn() {
    let mut app = App::new();
    app.init_resource::<TutorialEffectRuntime>()
        .add_systems(Update, process_tutorial_effect_runtime);
    let unloading_owner = app
        .world_mut()
        .spawn(crate::world::PendingNativeWorldSceneUnload::default())
        .id();
    let live_owner = app.world_mut().spawn_empty().id();
    {
        let mut runtime = app.world_mut().resource_mut::<TutorialEffectRuntime>();
        for (owner, source_line) in [(unloading_owner, 1), (live_owner, 2)] {
            runtime.enqueue_streamed_world_effect(TutorialEffectRuntimeCommand::Add {
                effect_id: 464,
                placement: TutorialEffectPlacement::ExactEntityWorld {
                    root_entity: owner,
                    position: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                },
                scale: 1.0,
                tracked: false,
                name: None,
                destroy_after_seconds: None,
                source_line,
            });
        }
    }

    app.update();

    let mut runtime = app.world_mut().resource_mut::<TutorialEffectRuntime>();
    let records = runtime.drain_records().collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].command.source_line(), 2);
    assert!(runtime.streamed_world_pending.is_empty());
}

#[test]
fn synthetic_closure_fails_with_a_node_specific_blocker() {
    let mut runtime = TutorialEffectRuntime::with_library(library());
    runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
        effect_id: 372,
        source_line: 3192,
    });
    runtime.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: 372,
        placement: TutorialEffectPlacement::World {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        },
        scale: 2.0,
        tracked: true,
        name: None,
        destroy_after_seconds: None,
        source_line: 3192,
    });
    runtime.process_pending();
    assert!(runtime.is_serialized_closure_preloaded(372));
    assert!(runtime.is_native_preload_complete(372));
    assert_eq!(runtime.active_native_instance_count(), 0);
    assert!(runtime.drain_issues().any(|issue| matches!(
        issue,
        TutorialEffectRuntimeIssue::UnsupportedSerializedClosureNode {
            effect_id: 372,
            reason: TutorialNativeClosureBlockerReason::InvalidSerializedNode { .. },
            ..
        }
    )));
}

#[test]
fn exact_preload_queues_native_assets_once_before_effect_instantiation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::with_library(
        TutorialEffectLibrary::load(&root).expect("exact tutorial effect library"),
    );
    for source_line in [3544, 3545] {
        runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
            effect_id: 668,
            source_line,
        });
        runtime.process_pending();
    }
    assert!(runtime.is_serialized_closure_preloaded(668));
    assert_eq!(runtime.native_preloads.len(), 1);
    assert_eq!(runtime.native_preloads[0].effect_id, 668);
    assert!(!runtime.is_native_preload_complete(668));
    runtime.mark_native_preload_complete(668);
    assert!(runtime.is_native_preload_complete(668));
}

#[test]
fn named_warp_presentation_waits_for_renderer_and_replacement() {
    let mut runtime = TutorialEffectRuntime::default();
    let name = "local-player.warp.departure";
    let old_id = runtime.allocate_instance(false, Some(name.to_owned()));
    assert!(!runtime.named_native_presentation_ready(name));
    runtime.mark_native_presentation_ready(old_id);
    assert!(runtime.named_native_presentation_ready(name));

    runtime.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: 394,
        placement: TutorialEffectPlacement::World {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        },
        scale: 1.0,
        tracked: false,
        name: Some(name.to_owned()),
        destroy_after_seconds: None,
        source_line: line!(),
    });
    assert!(!runtime.named_native_presentation_ready(name));
    runtime.pending.clear();
    let new_id = runtime.allocate_instance(false, Some(name.to_owned()));
    runtime.mark_native_presentation_ready(old_id);
    assert!(!runtime.named_native_presentation_ready(name));
    runtime.mark_native_presentation_ready(new_id);
    assert!(runtime.named_native_presentation_ready(name));
    runtime.clear_scene_instances();
    assert!(!runtime.named_native_presentation_ready(name));
    assert!(runtime.active.is_empty());
}

#[test]
fn named_native_instance_liveness_tracks_destroy_named() {
    let mut runtime = TutorialEffectRuntime::with_library(library());
    let effect_name = "Numbuh Two quest symbol";
    runtime.allocate_instance(false, Some(effect_name.to_owned()));
    assert!(runtime.has_named_native_instance(effect_name));
    runtime.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
        name: effect_name.to_owned(),
        source_line: 3072,
    });
    runtime.process_pending();
    assert!(!runtime.has_named_native_instance(effect_name));
}

#[test]
fn named_native_instance_liveness_tracks_a_removed_renderer_root() {
    let mut runtime = TutorialEffectRuntime::with_library(library());
    let effect_name = "world NPC 42 game icon";
    let instance_id = runtime.allocate_instance(false, Some(effect_name.to_owned()));
    let root_entity = Entity::from_bits(84);
    runtime.bind_native_root(instance_id, root_entity);

    assert!(
        runtime
            .reconcile_named_native_roots(&BTreeMap::from([(instance_id, root_entity)]))
            .is_empty()
    );
    assert!(runtime.has_named_native_instance(effect_name));

    assert_eq!(
        runtime.reconcile_named_native_roots(&BTreeMap::new()),
        [instance_id]
    );
    assert!(!runtime.has_named_native_instance(effect_name));
    assert_eq!(runtime.native_despawns.pop_front(), Some(instance_id));
}

#[test]
fn ordinary_mob_particle_effect_is_carried_by_the_linear_projectile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::with_library(
        TutorialEffectLibrary::load(&root).expect("exact tutorial effect library"),
    );
    runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
        bullet_type: 66,
        source: Vec3::ZERO,
        target: Vec3::X * 8.0,
        target_exists: true,
        source_style: 0,
        target_style: 0,
        motion: TutorialProjectileMotion::BulletMove,
        source_line: 66,
    });
    runtime.process_pending();

    assert!(!runtime.native_spawns.iter().any(|request| matches!(
        request,
        tutorial_native_effects::NativeSpawnRequest::Effect { effect_id: 17, .. }
    )));
    let carried = runtime
        .native_spawns
        .iter()
        .find_map(|request| match request {
            tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
                bullet_type: 66,
                effect_id: 17,
                plan: None,
                carried_effect,
                ..
            } => carried_effect.as_ref(),
            _ => None,
        });
    assert!(
        carried.is_some(),
        "BulletMoveScript must parent ES17 below the moving carrier; spawns={:#?}; issues={:#?}",
        runtime.native_spawns,
        runtime.issues,
    );
}

#[test]
fn fusion_spawn_attack_restores_its_invisible_carrier_and_es653_impact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::with_library(
        TutorialEffectLibrary::load(&root).expect("exact tutorial effect library"),
    );
    runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
        bullet_type: 106,
        source: Vec3::new(0.0, 0.9, 0.0),
        target: Vec3::new(2.4, 0.8, 0.0),
        target_exists: true,
        source_style: 2,
        target_style: -1,
        motion: TutorialProjectileMotion::BulletMove,
        source_line: 106,
    });
    runtime.process_pending();

    assert!(runtime.native_spawns.iter().any(|request| matches!(
        request,
        tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
            bullet_type: 106,
            effect_id: 0,
            scale,
            motion: tutorial_native_effects::NativeLinearProjectileMotion::BulletMove {
                hide_seconds,
                duration_seconds,
                ..
            },
            impact: Some(tutorial_native_effects::NativeLinearImpactPlan {
                effect_id: 653,
                ..
            }),
            plan: None,
            carried_effect: None,
            ..
        } if *scale == 0.0
            && (*hide_seconds - 0.300_000_01).abs() < 1.0e-7
            && (*duration_seconds - 0.300_000_01).abs() < 1.0e-7
    )));
    assert!(runtime.drain_issues().next().is_none());
}

#[test]
fn grenade_warhead_uses_weapon_motion_and_raw_es405_impact_scale() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::with_library(
        TutorialEffectLibrary::load(&root).expect("exact tutorial effect library"),
    );
    runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
        bullet_type: 72,
        source: Vec3::new(1.0, 0.8, 2.0),
        target: Vec3::new(8.0, 0.0, 9.0),
        // `cnWarHead.Crash` still loads its success effect without the
        // `BulletMoveScript` target-object gate.
        target_exists: false,
        source_style: 2,
        target_style: 1,
        motion: TutorialProjectileMotion::Warhead {
            speed: 8.0,
            initial_vertical_speed: Some(6.0),
            duration_seconds: 2.5,
            authority: None,
        },
        source_line: 72,
    });
    runtime.process_pending();

    assert!(runtime.native_spawns.iter().any(|request| matches!(
        request,
        tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
            bullet_type: 72,
            effect_id: 31,
            motion: tutorial_native_effects::NativeLinearProjectileMotion::Warhead {
                speed,
                initial_vertical_speed: Some(vertical),
                duration_seconds,
                ..
            },
            impact: Some(tutorial_native_effects::NativeLinearImpactPlan {
                effect_id: 405,
                scale,
                ..
            }),
            ..
        } if *speed == 8.0
            && *vertical == 6.0
            && *duration_seconds == 2.5
            && (*scale - 2.200_000_05).abs() < 1.0e-6
    )));
    let issues = runtime.drain_issues().collect::<Vec<_>>();
    assert!(issues.is_empty(), "grenade issues={issues:#?}");
}

#[test]
fn oni_motion_arrival_and_delayed_destroy_follow_original_thresholds() {
    let mut state =
        TutorialOniProjectileState::new(Vec3::ZERO, Vec3::new(1.0, 5.0, 1.0), false).unwrap();
    state.step(0.016, Vec3::new(0.1, 0.0, 0.0));
    assert_eq!(state.position, Vec3::new(0.1, 0.0, 0.0));
    assert_eq!(state.phase, TutorialOniMotionPhase::EmitterStopped);
    state.step(0.5, Vec3::ZERO);
    assert_eq!(state.phase, TutorialOniMotionPhase::EmitterStopped);
    state.step(0.001, Vec3::ZERO);
    assert_eq!(state.phase, TutorialOniMotionPhase::Destroyed);
}

#[test]
fn native_preload_failure_is_terminal_visible_and_deduplicated() {
    let mut runtime = TutorialEffectRuntime::default();
    runtime.mark_native_preload_failed(742, "map/shared/effects/es742.glb", "GLTF load failed");
    runtime.mark_native_preload_failed(
        742,
        "map/shared/effects/es742.glb",
        "scene dependency load failed",
    );

    assert!(runtime.is_native_preload_complete(742));
    assert_eq!(
        runtime.drain_issues().collect::<Vec<_>>(),
        vec![TutorialEffectRuntimeIssue::NativePreloadFailed {
            effect_id: 742,
            asset_path: "map/shared/effects/es742.glb".to_owned(),
            detail: "GLTF load failed".to_owned(),
        }]
    );
}

#[test]
fn every_published_bullet_row_materializes_a_moving_native_carrier() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::with_library(
        TutorialEffectLibrary::load(&root).expect("exact tutorial effect library"),
    );
    for bullet_type in RETROBUTION_TUTORIAL_BULLET_TYPES {
        runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
            bullet_type,
            source: Vec3::new(1.0, 0.8, 2.0),
            target: Vec3::new(9.0, 1.2, 7.0),
            target_exists: true,
            source_style: 0,
            target_style: 1,
            motion: TutorialProjectileMotion::BulletMove,
            source_line: line!(),
        });
    }
    runtime.process_pending();

    let queued = runtime
        .native_spawns
        .iter()
        .filter_map(|request| match request {
            tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
                bullet_type,
                ..
            } => Some(*bullet_type),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let missing = RETROBUTION_TUTORIAL_BULLET_TYPES
        .into_iter()
        .filter(|bullet_type| !queued.contains(bullet_type))
        .collect::<Vec<_>>();
    let issues = runtime.issues.iter().cloned().collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "published BulletTable rows never queued a moving native carrier: {missing:?}; issues={issues:#?}"
    );
}

#[test]
fn projectile_success_scale_matches_retrobution_style_matrix() {
    for ((source, target), expected) in [
        ((0, 0), 2.6),
        ((0, 1), 3.12),
        ((0, 2), 1.82),
        ((1, 2), 3.12),
        ((2, 1), 1.82),
        ((-1, -1), 2.6),
    ] {
        assert!(
            (exact_projectile_success_scale(2.0, source, target) - expected).abs() < 1.0e-6
        );
    }
}

#[test]
fn null_target_projectile_suppresses_retrobution_success_script() {
    assert_eq!(exact_projectile_success_script(false, 742), None);
    assert_eq!(exact_projectile_success_script(true, 742), Some(742));
    assert_eq!(exact_projectile_success_script(true, 0), None);
}
