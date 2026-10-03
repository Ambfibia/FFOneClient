//! Bounded main-thread admission for decoded world behaviour documents.

use crate::world_behaviour::*;

pub(super) fn materialize_world_behaviour_batch(
    commands: &mut Commands,
    root: Entity,
    pending: &mut PendingWorldBehaviourSpawn,
    work_budget: usize,
) -> usize {
    let document = Arc::clone(pending.document());
    let mut remaining = work_budget;

    while remaining > 0 && pending.phase != WorldBehaviourSpawnPhase::Complete {
        match pending.phase {
            WorldBehaviourSpawnPhase::Billboards => {
                let Some(record) = document.billboards.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::VisibilitySwitches);
                    continue;
                };
                let pivot = record
                    .world_matrix
                    .as_ref()
                    .map(mat4_from_world_matrix)
                    .unwrap_or(Mat4::IDENTITY);
                let models = pending.resolve(&record.models);
                for model in &models {
                    commands
                        .entity(*model)
                        .insert(crate::world::NativeWorldDynamicObjectRange);
                }
                let entity = commands
                    .spawn((
                        Name::new(format!("billboard {}", record.node)),
                        WorldBillboard {
                            mode: BillboardMode::from_legacy(record.mode),
                            runtime_enabled: record.enabled,
                        },
                        ChildOf(root),
                        Transform::from_matrix(pivot),
                        Visibility::Inherited,
                    ))
                    .id();
                for model in &models {
                    reparent_authored_world_model(
                        commands,
                        *model,
                        entity,
                        pivot,
                        &pending.authored_model_worlds,
                    );
                }
                pending
                    .nodes
                    .entry(record.node.clone())
                    .or_default()
                    .push(entity);
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1 + models.len());
            }
            WorldBehaviourSpawnPhase::VisibilitySwitches => {
                let Some(record) = document.visibility_switches.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::EffectEmitters);
                    continue;
                };
                let controlled_billboards = pending
                    .nodes
                    .get(&record.switches)
                    .cloned()
                    .unwrap_or_default();
                commands.spawn((
                    Name::new(format!("visibility switch {}", record.node)),
                    WorldVisibilitySwitch {
                        controlled_billboards,
                        renderer_entities: pending.resolve(&record.models),
                    },
                    ChildOf(root),
                    placement(record.world_matrix.as_ref()),
                    visibility_for(record.enabled),
                ));
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1);
            }
            WorldBehaviourSpawnPhase::EffectEmitters => {
                let Some(record) = document.effect_emitters.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::Animations);
                    continue;
                };
                let model_entities = pending.resolve(&record.models);
                if record.effect_name.as_deref() == Some("neededjump_nano") {
                    for model in &model_entities {
                        commands.entity(*model).insert((
                            WorldFloatingIconSpin::default(),
                            crate::world::NativeWorldDynamicObjectRange,
                        ));
                    }
                }
                let blackhole_animation =
                    blackhole_nif_animation_player(record, &pending.entities_by_model).expect(
                        "world NIF animation bindings were validated before materialization",
                    );
                commands.spawn((
                    Name::new(format!("effect emitter {}", record.node)),
                    WorldEffectEmitter {
                        controller: record.controller.clone(),
                        effect_name: record.effect_name.clone(),
                        priority: record.priority,
                        max_timer: record.max_timer as f32,
                        longest_life_time: record.longest_life_time as f32,
                        trail_times: record.trail_times as f32,
                        conform_to_scale: record.conform_to_scale,
                        disable_update: record.disable_update,
                        particle_element_count: record.particle_element_count,
                        model_entities,
                        nif_object: record.nif_object.clone(),
                        particles: record.particles.clone(),
                        particle_elements: record.particle_elements.clone(),
                    },
                    ChildOf(root),
                    placement(record.world_matrix.as_ref()),
                    visibility_for(record.enabled),
                ));
                if let Some((player, compiled_tracks)) = blackhole_animation {
                    commands.spawn((
                        Name::new(format!("effect NIF animation {}", record.node)),
                        player,
                        WorldAnimationCompiledTracks(compiled_tracks),
                        WorldAnimationSamples::default(),
                        WorldAnimationVisibilityBindings::default(),
                        WorldAnimationMaterialBindings::default(),
                        ChildOf(root),
                        placement(record.world_matrix.as_ref()),
                        visibility_for(record.enabled),
                    ));
                }
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1);
            }
            WorldBehaviourSpawnPhase::Animations => {
                let Some(record) = document.animations.get(pending.next_record) else {
                    pending.animation_record = None;
                    pending.finish_phase(WorldBehaviourSpawnPhase::Triggers);
                    continue;
                };
                if pending.animation_record.is_none() {
                    let active_clip_id = record
                        .default_clip_id
                        .as_deref()
                        .or_else(|| record.clip_ids.first().map(String::as_str));
                    let active_clip_index = active_clip_id
                        .and_then(|id| pending.animation_clip_indices.get(id))
                        .copied();
                    let anchor = record.targets.first().and_then(|target| {
                        target.root_parent_world_matrix.as_ref().map(|matrix| {
                            commands
                                .spawn((
                                    Name::new(format!("animation anchor {}", record.node)),
                                    ChildOf(root),
                                    Transform::from_matrix(mat4_from_world_matrix(matrix)),
                                    Visibility::Inherited,
                                ))
                                .id()
                        })
                    });
                    pending.animation_record = Some(PendingWorldAnimationRecord {
                        record_index: pending.next_record,
                        active_clip_index,
                        anchor,
                        next_target: 0,
                        target_entities: HashMap::new(),
                        target_bindings: HashMap::new(),
                        deferred_parent_bindings: Vec::new(),
                    });
                    charge_world_behaviour_work(&mut remaining, 1);
                    if remaining == 0 {
                        continue;
                    }
                }

                let mut state = pending
                    .animation_record
                    .take()
                    .expect("the current animation record was initialized");
                debug_assert_eq!(state.record_index, pending.next_record);
                let batch_end = state
                    .next_target
                    .saturating_add(WORLD_ANIMATION_BINDINGS_PER_FRAME.min(remaining.max(1)))
                    .min(record.targets.len());
                let mut reparented_models = 0_usize;
                for target in &record.targets[state.next_target..batch_end] {
                    let base_transform = target.base_local_trs.transform();
                    let resolved_parent = target
                        .parent_path
                        .as_deref()
                        .and_then(|path| state.target_entities.get(path).copied());
                    let parent = resolved_parent.or(state.anchor).unwrap_or(root);
                    let target_models = pending.resolve(&target.models);
                    let integrated_billboard = pending
                        .nodes
                        .get(&target.node)
                        .and_then(|entities| (entities.len() == 1).then_some(entities[0]));
                    let entity = if let Some(entity) = integrated_billboard {
                        commands.entity(entity).insert((
                            WorldAnimationTarget {
                                path: target.path.clone(),
                                base_transform,
                            },
                            ChildOf(parent),
                            base_transform,
                        ));
                        entity
                    } else {
                        let entity = commands
                            .spawn((
                                Name::new(format!(
                                    "animation target {} {}",
                                    record.node, target.path
                                )),
                                WorldAnimationTarget {
                                    path: target.path.clone(),
                                    base_transform,
                                },
                                ChildOf(parent),
                                base_transform,
                                Visibility::Inherited,
                            ))
                            .id();
                        let target_world = mat4_from_world_matrix(&target.base_world_matrix);
                        for model in &target_models {
                            reparent_authored_world_model(
                                commands,
                                *model,
                                entity,
                                target_world,
                                &pending.authored_model_worlds,
                            );
                        }
                        reparented_models += target_models.len();
                        entity
                    };
                    if let Some(parent_path) = target.parent_path.as_ref()
                        && resolved_parent.is_none()
                    {
                        state
                            .deferred_parent_bindings
                            .push((entity, parent_path.clone()));
                    }
                    state.target_entities.insert(target.path.clone(), entity);
                    state.target_bindings.insert(
                        target.path.clone(),
                        WorldAnimationTargetBinding {
                            entity,
                            model_entities: target_models,
                        },
                    );
                }
                charge_world_behaviour_work(
                    &mut remaining,
                    batch_end.saturating_sub(state.next_target) + reparented_models,
                );
                state.next_target = batch_end;
                if state.next_target < record.targets.len() {
                    pending.animation_record = Some(state);
                    continue;
                }

                let active_clip = state
                    .active_clip_index
                    .and_then(|index| pending.animation_clips.get(index));
                let transform_repeats = world_animation_transform_repeats(
                    active_clip.map(Arc::as_ref),
                    record.wrap_mode,
                );
                // Compiling the lookup table is intentionally charged after
                // target admission. Tracks retain channel indices into the
                // tile's shared Arc document, so even a 2,418-keyframe clip
                // no longer clones every curve in its completion frame.
                let compilation_work = active_clip
                    .map(|clip| {
                        clip.channels
                            .len()
                            .div_ceil(WORLD_ANIMATION_BINDINGS_PER_FRAME)
                    })
                    .unwrap_or(1)
                    .max(1);
                if compilation_work > remaining && remaining < work_budget {
                    pending.animation_record = Some(state);
                    remaining = 0;
                    continue;
                }
                charge_world_behaviour_work(&mut remaining, compilation_work);
                let compiled_tracks = compile_world_animation_tracks(
                    active_clip.map(Arc::as_ref),
                    &state.target_bindings,
                );
                for model in world_animation_dynamic_model_entities(
                    &record.targets,
                    &state.target_bindings,
                    &compiled_tracks,
                ) {
                    commands
                        .entity(model)
                        .insert(crate::world::NativeWorldDynamicObjectRange);
                }
                let samples = Vec::with_capacity(compiled_tracks.len());
                for (entity, parent_path) in &state.deferred_parent_bindings {
                    if let Some(parent) = state.target_entities.get(parent_path).copied() {
                        commands.entity(*entity).insert(ChildOf(parent));
                    }
                }
                commands.spawn((
                    Name::new(format!("animation {}", record.node)),
                    WorldAnimationPlayer {
                        play_automatically: record.play_automatically,
                        animate_physics: record.animate_physics,
                        animate_only_if_visible: record.animate_only_if_visible,
                        wrap_mode: record.wrap_mode,
                        clip_count: record.clip_count,
                        model_entities: pending.resolve(&record.models),
                        default_clip: record.default_clip.clone(),
                        clip_refs: record.clip_refs.clone(),
                        default_clip_id: record.default_clip_id.clone(),
                        clip_ids: record.clip_ids.clone(),
                        targets: state.target_bindings,
                        elapsed_seconds: 0.0,
                        transform_repeats,
                        active_clip: active_clip.cloned(),
                    },
                    WorldAnimationCompiledTracks(compiled_tracks),
                    WorldAnimationSamples(samples),
                    WorldAnimationVisibilityBindings::default(),
                    WorldAnimationMaterialBindings::default(),
                    ChildOf(root),
                    placement(record.world_matrix.as_ref()),
                    visibility_for(record.enabled),
                ));
                pending.next_record += 1;
            }
            WorldBehaviourSpawnPhase::Triggers => {
                let Some(record) = document.triggers.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::WaypointEntities);
                    continue;
                };
                let mut path_points = Vec::new();
                let mut current = record.head.as_deref();
                let maximum = record.waypoint_count.max(0) as usize + 1;
                for _ in 0..maximum.min(4_096) {
                    let Some(waypoint) = current
                        .and_then(|node| pending.waypoint_record_indices.get(node))
                        .and_then(|index| document.waypoints.get(*index))
                    else {
                        break;
                    };
                    path_points.push(unity_vec3(waypoint.point));
                    current = waypoint.next.as_deref();
                }
                let kind = WorldTriggerKind::from_document(&record.kind);
                let server_id = clean_initial_trigger_server_id(
                    kind,
                    record.enabled,
                    record.server_id,
                    &mut pending.next_ring_server_id,
                );
                let pivot = record
                    .world_matrix
                    .as_ref()
                    .map(mat4_from_world_matrix)
                    .unwrap_or(Mat4::IDENTITY);
                let model_entities = pending.resolve(&record.models);
                let entity = commands
                    .spawn((
                        Name::new(format!("trigger {} {}", record.kind, record.node)),
                        WorldTrigger {
                            kind,
                            server_id,
                            object_id: record.object_id,
                            cne_id: record.cne_id,
                            trigger_type: record.trigger_type,
                            radius: record.radius as f32,
                            velocity: record.velocity as f32,
                            speed: record.speed as f32,
                            add_power: record.add_power as f32,
                            min_power: record.min_power as f32,
                            max_power: record.max_power as f32,
                            move_type: record.move_type,
                            waypoint_count: record.waypoint_count,
                            start_position: unity_vec3(record.start_position),
                            from: unity_vec3(record.from),
                            to: unity_vec3(record.to),
                            spin_velocity: record.spin_velocity as f32,
                            initial_rotate: vec3(record.initial_rotation),
                            max_rotate: vec3(record.max_rotation),
                            head: record.head.clone(),
                            tail: record.tail.clone(),
                            target_element_id: record.target_element_id,
                            target_element_trigger: record.target_element_trigger,
                            model_entities: model_entities.clone(),
                            path_points: path_points.clone(),
                        },
                        ChildOf(root),
                        Transform::from_matrix(pivot),
                        visibility_for(record.enabled),
                    ))
                    .id();
                if kind == WorldTriggerKind::Ring {
                    for model in &model_entities {
                        commands.entity(*model).insert((Visibility::Hidden, crate::world::RuntimeManagedNativeWorldVisual));
                    }
                }
                if kind == WorldTriggerKind::Platform {
                    let transform = Transform::from_matrix(pivot);
                    commands.entity(entity).insert(WorldPlatformMotion {
                        initial_translation: transform.translation,
                        initial_rotation: transform.rotation,
                        initial_scale: transform.scale,
                        from: unity_vec3(record.from),
                        to: unity_vec3(record.to),
                        velocity: record.velocity as f32,
                        spin_velocity: record.spin_velocity as f32,
                        move_type: record.move_type,
                        path_points,
                    });
                    for model in &model_entities {
                        commands
                            .entity(*model)
                            .insert(crate::world::NativeWorldDynamicObjectRange);
                        reparent_authored_world_model(
                            commands,
                            *model,
                            entity,
                            pivot,
                            &pending.authored_model_worlds,
                        );
                    }
                } else if kind == WorldTriggerKind::Switch {
                    commands.entity(entity).insert(WorldSwitchState::default());
                }
                pending.trigger_entities.insert(record.node.clone(), entity);
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1 + model_entities.len());
            }
            WorldBehaviourSpawnPhase::WaypointEntities => {
                let Some(record) = document.waypoints.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::WaypointLinks);
                    continue;
                };
                let entity = commands
                    .spawn((
                        Name::new(format!("waypoint {}", record.node)),
                        WorldWaypoint {
                            point: vec3(record.point),
                            next: None,
                            previous: None,
                        },
                        ChildOf(root),
                        placement(record.world_matrix.as_ref()),
                        visibility_for(record.enabled),
                    ))
                    .id();
                pending
                    .waypoint_entities
                    .insert(record.node.clone(), entity);
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1);
            }
            WorldBehaviourSpawnPhase::WaypointLinks => {
                let Some(record) = document.waypoints.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::TriggerVolumes);
                    continue;
                };
                if let Some(entity) = pending.waypoint_entities.get(&record.node).copied() {
                    let next = record
                        .next
                        .as_ref()
                        .and_then(|node| pending.waypoint_entities.get(node))
                        .copied();
                    let previous = record
                        .previous
                        .as_ref()
                        .and_then(|node| pending.waypoint_entities.get(node))
                        .copied();
                    commands.entity(entity).insert(WorldWaypoint {
                        point: vec3(record.point),
                        next,
                        previous,
                    });
                }
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1);
            }
            WorldBehaviourSpawnPhase::TriggerVolumes => {
                let Some(record) = document.trigger_volumes.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::RigidBodies);
                    continue;
                };
                let shape = match record.kind.as_str() {
                    "box" => WorldTriggerVolumeShape::Box,
                    "capsule" => WorldTriggerVolumeShape::Capsule,
                    _ => WorldTriggerVolumeShape::Sphere,
                };
                let trigger = pending.trigger_entities.get(&record.node).copied();
                commands.spawn((
                    Name::new(format!("trigger volume {}", record.node)),
                    WorldTriggerVolume {
                        shape,
                        center: unity_vec3(record.center),
                        radius: record.radius as f32,
                        size: record.size.map(vec3).unwrap_or(Vec3::ONE),
                        height: record.height as f32,
                        direction: record.direction,
                        is_trigger: record.is_trigger,
                        trigger,
                    },
                    WorldTriggerVolumeOccupied::default(),
                    ChildOf(trigger.unwrap_or(root)),
                    if trigger.is_some() {
                        Transform::IDENTITY
                    } else {
                        placement(record.world_matrix.as_ref())
                    },
                    visibility_for(record.enabled),
                ));
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1);
            }
            WorldBehaviourSpawnPhase::RigidBodies => {
                let Some(record) = document.rigid_bodies.get(pending.next_record) else {
                    pending.finish_phase(WorldBehaviourSpawnPhase::Complete);
                    continue;
                };
                commands.spawn((
                    Name::new(format!("rigid body {}", record.node)),
                    WorldRigidBody {
                        mass: record.mass as f32,
                        drag: record.drag as f32,
                        angular_drag: record.angular_drag as f32,
                        use_gravity: record.use_gravity,
                        is_kinematic: record.is_kinematic,
                    },
                    ChildOf(root),
                    placement(record.world_matrix.as_ref()),
                    visibility_for(record.enabled),
                ));
                pending.next_record += 1;
                charge_world_behaviour_work(&mut remaining, 1);
            }
            WorldBehaviourSpawnPhase::Complete => break,
        }
    }

    work_budget.saturating_sub(remaining)
}
