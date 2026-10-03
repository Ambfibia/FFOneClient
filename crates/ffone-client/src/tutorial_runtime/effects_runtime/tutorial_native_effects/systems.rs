use super::*;

pub(super) fn apply_native_requests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut runtime: ResMut<TutorialEffectRuntime>,
    registry: Option<Res<TutorialActorRegistry>>,
    names: Query<&Name>,
    children: Query<&Children>,
    globals: Query<&GlobalTransform>,
    unloading_stream_owners: Query<(), With<crate::world::PendingNativeWorldSceneUnload>>,
    roots: Query<(Entity, &NativeRoot)>,
    detached: Query<(Entity, &NativeDetachedOwner)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TutorialParticleMaterial>>,
    mut visual_assets: ResMut<NativeVisualAssets>,
    mut preload_cache: ResMut<NativeEffectPreloadCache>,
) {
    if visual_assets.quad.is_none() {
        visual_assets.quad = Some(meshes.add(particle_quad()));
    }
    for request in runtime.native_preloads.drain(..).collect::<Vec<_>>() {
        for emitter in &request.plan.emitters {
            texture_handle(&emitter.texture, &mut images, &mut visual_assets);
        }
        if let Some(path) = request.plan.mesh_scene {
            preload_cache.paths.entry(request.effect_id).or_insert(path);
            preload_cache
                .gltfs
                .entry(request.effect_id)
                .or_insert_with(|| asset_server.load(path));
            preload_cache
                .scenes
                .entry(request.effect_id)
                .or_insert_with(|| asset_server.load(GltfAssetLabel::Scene(0).from_asset(path)));
        } else {
            // Particle textures are decoded into `Assets<Image>` above, so a
            // closure without an asynchronous GLTF is terminal immediately.
            runtime.mark_native_preload_complete(request.effect_id);
        }
    }
    for id in runtime.native_despawns.drain(..).collect::<Vec<_>>() {
        despawn_native_instance(id, &mut commands, &roots, &detached);
    }
    let stale_instances = {
        let TutorialEffectRuntime {
            active,
            native_spawns,
            ..
        } = &mut *runtime;
        purge_stale_native_spawn_requests(
            native_spawns,
            |instance_id| active.contains_key(&instance_id),
            |owner| globals.get(owner).is_ok() && unloading_stream_owners.get(owner).is_err(),
        )
    };
    for instance_id in stale_instances {
        runtime.forget_instance(instance_id);
    }
    let spawn_batch = take_native_spawn_batch(
        &mut runtime.native_spawns,
        NATIVE_SPAWN_WORK_PER_FRAME,
        STREAMED_WORLD_NATIVE_SPAWN_WORK_PER_FRAME,
    );
    for request in spawn_batch {
        let instance_id = request.instance_id();
        if !runtime.active.contains_key(&instance_id) {
            continue;
        }
        // An ambient request can wait behind gameplay for several frames. Its
        // tile may unload during that interval; never materialize a child under
        // a dead stream owner merely to remove it again later in this chain.
        if request.stream_owner().is_some_and(|owner| {
            globals.get(owner).is_err() || unloading_stream_owners.get(owner).is_ok()
        }) {
            runtime.forget_instance(instance_id);
            continue;
        }
        match request {
            NativeSpawnRequest::Effect {
                instance_id,
                effect_id,
                streamed_world,
                placement,
                scale,
                name,
                destroy_after_seconds,
                source_line,
                plan,
            } => spawn_effect(
                &mut commands,
                &asset_server,
                &mut runtime,
                registry.as_deref(),
                &names,
                &children,
                &globals,
                instance_id,
                effect_id,
                streamed_world,
                placement,
                scale,
                name,
                destroy_after_seconds,
                source_line,
                plan,
                &mut preload_cache,
            ),
            NativeSpawnRequest::Projectile {
                instance_id,
                effect_id,
                source,
                target,
                scale,
                reverse,
                sampled_initial_velocity,
                plan,
            } => spawn_projectile(
                &mut commands,
                &asset_server,
                &mut runtime,
                &mut meshes,
                &mut images,
                &mut materials,
                &mut visual_assets,
                instance_id,
                effect_id,
                source,
                target,
                scale,
                reverse,
                sampled_initial_velocity,
                plan,
            ),
            NativeSpawnRequest::LinearProjectile {
                instance_id,
                bullet_type,
                effect_id,
                source,
                target,
                scale,
                motion,
                impact,
                plan,
                carried_effect,
            } => spawn_linear_projectile(
                &mut commands,
                &asset_server,
                &mut runtime,
                &mut meshes,
                &mut images,
                &mut materials,
                &mut visual_assets,
                instance_id,
                bullet_type,
                effect_id,
                source,
                target,
                scale,
                motion,
                impact,
                plan,
                carried_effect,
            ),
        }
    }
}

pub(super) fn sync_native_preload_completion(
    asset_server: Res<AssetServer>,
    preload_cache: Res<NativeEffectPreloadCache>,
    mut runtime: ResMut<TutorialEffectRuntime>,
) {
    for (&effect_id, gltf) in &preload_cache.gltfs {
        if runtime.is_native_preload_complete(effect_id) {
            continue;
        }
        let Some(scene) = preload_cache.scenes.get(&effect_id) else {
            continue;
        };
        let path = preload_cache
            .paths
            .get(&effect_id)
            .copied()
            .unwrap_or("<unknown tutorial effect GLB>");
        let gltf_state = native_preload_handle_state(&asset_server, gltf, "GLTF");
        let scene_state = native_preload_handle_state(&asset_server, scene, "scene");
        match (gltf_state, scene_state) {
            (NativePreloadHandleState::Failed(detail), _)
            | (_, NativePreloadHandleState::Failed(detail)) => {
                runtime.mark_native_preload_failed(effect_id, path, detail);
            }
            (NativePreloadHandleState::Complete, NativePreloadHandleState::Complete) => {
                runtime.mark_native_preload_complete(effect_id);
            }
            _ => {}
        }
    }
}

pub(super) fn update_native_sword_trails(
    mut commands: Commands,
    time: Res<Time>,
    attachments: Query<&TutorialPlayerWeaponAttachment>,
    actions: Query<&LegacyAvatarActionState>,
    globals: Query<&GlobalTransform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut trails: Query<(Entity, &Mesh3d, &mut NativeSwordTrail, &mut Visibility)>,
) {
    for (entity, mesh_handle, mut trail, mut visibility) in &mut trails {
        if attachments.get(trail.attachment).is_err() {
            commands.entity(entity).despawn();
            continue;
        }
        let Ok(action) = actions.get(trail.controller_root) else {
            continue;
        };
        let attacking = native_sword_attack_active(action);
        if !attacking {
            trail.attack_generation = None;
            if trail.emitting {
                trail.emitting = false;
                *visibility = Visibility::Hidden;
            }
            continue;
        }
        let (Ok(rig_global), Ok(top_global), Ok(bottom_global)) = (
            globals.get(trail.rig_root),
            globals.get(trail.top_point),
            globals.get(trail.bottom_point),
        ) else {
            continue;
        };
        let rig_from_world = rig_global.affine().inverse();
        let top = rig_from_world.transform_point3(top_global.translation());
        let bottom = rig_from_world.transform_point3(bottom_global.translation());

        if trail.attack_generation != Some(action.attack_generation()) {
            trail.edges.fill(NativeSwordTrailEdge { top, bottom });
            trail.attack_generation = Some(action.attack_generation());
            trail.emitting = true;
            trail.elapsed_seconds = 0.0;
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        }
        if !trail.emitting {
            continue;
        }
        trail.elapsed_seconds += time.delta_secs().max(0.0);
        if trail.elapsed_seconds >= RETROBUTION_SWORD_TRAIL_MAX_SECONDS {
            trail.emitting = false;
            *visibility = Visibility::Hidden;
            continue;
        }

        advance_native_sword_trail(&mut trail.edges, top, bottom);
        if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = native_sword_trail_mesh(&trail.edges);
        }
    }
}

pub(super) fn advance_native_sword_trail(
    edges: &mut [NativeSwordTrailEdge],
    current_top: Vec3,
    current_bottom: Vec3,
) {
    if edges.len() < 2 {
        return;
    }
    let last = edges[edges.len() - 1];
    let previous = edges[edges.len() - 2];
    let dot = (current_top - last.top).dot(last.top - previous.top);
    for fraction in [0.2_f32, 0.4, 0.6, 0.8] {
        let last = edges[edges.len() - 1];
        let edge_direction = last.top - last.bottom;
        let top_center = (last.top + current_top) * 0.5 - (1.5 - dot) * edge_direction;
        let top = unity_vector_slerp(last.top - top_center, current_top - top_center, fraction)
            + top_center;
        let bottom_center = (last.bottom + current_bottom) * 0.5 - (2.0 - dot) * edge_direction;
        let bottom = unity_vector_slerp(
            last.bottom - bottom_center,
            current_bottom - bottom_center,
            fraction,
        ) + bottom_center;
        push_native_sword_trail_edge(edges, NativeSwordTrailEdge { top, bottom });
    }
    push_native_sword_trail_edge(
        edges,
        NativeSwordTrailEdge {
            top: current_top,
            bottom: current_bottom,
        },
    );
}

pub(super) fn update_emitters(
    mut commands: Commands,
    time: Res<Time>,
    mut emitters: Query<(Entity, &GlobalTransform, &mut Transform, &mut NativeEmitter)>,
    particles: Query<&NativeParticle>,
    mut random: ResMut<NativeParticleRandomStream>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TutorialParticleMaterial>>,
    mut visual_assets: ResMut<NativeVisualAssets>,
    cameras: Query<(&GlobalTransform, &Projection), (With<Camera3d>, With<LegacyOrbitCamera>)>,
    live_stream_owners: Query<Entity, Without<crate::world::PendingNativeWorldSceneUnload>>,
    parents: Query<&ChildOf>,
    roots: Query<&NativeEffectRoot>,
) {
    let delta = time.delta_secs();
    let camera = cameras.iter().next();
    let Some(quad) = visual_assets.quad.clone() else {
        return;
    };
    let mut remaining_world_particles = MAX_STREAMED_WORLD_PARTICLES.saturating_sub(
        particles
            .iter()
            .filter(|particle| particle.stream_owned)
            .count(),
    );
    let mut ordered_emitters = emitters
        .iter_mut()
        .map(|(entity, global, _, emitter)| {
            let distance_squared = camera.map_or(0.0, |(camera, _)| {
                global.translation().distance_squared(camera.translation())
            });
            // Player/projectile emitters are not subject to the ambient-world
            // budget and always run before the distance-sorted streamed set.
            let priority = if emitter.stream_owned && emitter.preserve_when_streamed_budget_is_full
            {
                -0.5
            } else if emitter.stream_owned {
                distance_squared
            } else {
                -1.0
            };
            (entity, priority)
        })
        .collect::<Vec<_>>();
    ordered_emitters.sort_unstable_by(|left, right| left.1.total_cmp(&right.1));
    for (entity, _) in ordered_emitters {
        let Ok((_, global, mut transform, mut emitter)) = emitters.get_mut(entity) else {
            continue;
        };
        // The source creates particles after the mesh object is ready. Keep
        // smoke, geometry and UV curves on the same clock during async loading.
        if parents
            .get(entity)
            .ok()
            .and_then(|parent| roots.get(parent.parent()).ok())
            .is_some_and(|root| root.waiting_for_mesh_surface)
        {
            continue;
        }
        if emitter
            .stream_owner
            .is_some_and(|owner| live_stream_owners.get(owner).is_err())
        {
            continue;
        }
        let within_emission_distance = !emitter.stream_owned
            || camera.is_some_and(|(camera, projection)| {
                world_effect_is_within_camera_distance(global.translation(), camera, projection)
            });
        emitter.age += delta;
        let mut emitting = emitter.plan.initial_emit;
        if !emitter.disable_update {
            if emitter.maximum_timer > 0.0 && emitter.age > emitter.maximum_timer {
                emitting = false;
            } else {
                let script_duration = emitter.maximum_timer.abs();
                let script_time = if script_duration > f32::EPSILON {
                    if emitter.maximum_timer < 0.0 {
                        (emitter.age % script_duration) / script_duration
                    } else {
                        emitter.age / script_duration
                    }
                } else {
                    emitter.age
                };
                let (translation, script_emit) =
                    script_state(&emitter.plan.script_keys, script_time);
                if transform.translation != translation {
                    transform.translation = translation;
                }
                emitting = script_emit;
            }
        }
        if !emitting {
            continue;
        }
        let generations_per_second = emitter.plan.generations_per_second;
        if !legacy_emission_due(&mut emitter.timer, delta, generations_per_second) {
            continue;
        }
        if !within_emission_distance {
            // Unity continues advancing the controller and its script while
            // the renderer is distance-culled. Skip only allocation of a
            // detached card; freezing age/timer here made a distant looping
            // effect restart when the camera approached it.
            continue;
        }
        let emitter_rotation = global.compute_transform().rotation;
        let requested_particles = emitter.plan.number_per_generation.ceil() as usize;
        let particles_to_spawn = if emitter.stream_owned {
            streamed_particle_spawn_count(
                requested_particles,
                &mut remaining_world_particles,
                emitter.preserve_when_streamed_budget_is_full,
            )
        } else {
            requested_particles
        };
        if particles_to_spawn == 0 {
            // Keep the accumulated emission ready. Resetting it here loses a
            // one-shot/very-low-rate Unity particle for a complete generation
            // interval (600 seconds for the Sector V hologram bodies).
            continue;
        }
        emitter.timer = 0.0;
        for _ in 0..particles_to_spawn {
            let (position, local_velocity) =
                legacy_particle_initial_state(&emitter.plan, &mut random);
            let material_is_animated = particle_material_is_animated(&emitter.plan);
            let material = if !material_is_animated {
                if let Some(material) = &emitter.static_material {
                    material.clone()
                } else {
                    let material = shared_particle_material(
                        &emitter.plan,
                        0.0,
                        &mut images,
                        &mut materials,
                        &mut visual_assets,
                    );
                    emitter.static_material = Some(material.clone());
                    material
                }
            } else {
                shared_particle_material(
                    &emitter.plan,
                    0.0,
                    &mut images,
                    &mut materials,
                    &mut visual_assets,
                )
            };
            let size = particle_size(&emitter.plan, 0.0, emitter.scale);
            let world_position = native_emitted_particle_position(global.translation(), position);
            let rotation = camera.map_or(Quat::IDENTITY, |(camera, _)| {
                particle_billboard_rotation(emitter.plan.render_mode, camera, world_position, 0.0)
            });
            let particle_transform = Transform::from_translation(world_position)
                .with_rotation(rotation)
                .with_scale(Vec3::new(size.x, size.y, 1.0));
            let mut particle_entity = commands.spawn((
                Mesh3d(quad.clone()),
                MeshMaterial3d(material.clone()),
                NativeParticle {
                    stream_owner: emitter.stream_owner,
                    plan: emitter.plan.clone(),
                    scale: emitter.scale,
                    age: 0.0,
                    velocity: emitter_rotation * local_velocity,
                    stream_owned: emitter.stream_owned,
                    alive: true,
                },
                NativeDetachedOwner(emitter.owner),
                // ParticleEmitterController calls
                // Emit(transform.position + position,
                //      transform.TransformDirection(velocity), ...).
                // The generated offset is deliberately neither rotated nor
                // scaled; only velocity receives the emitter rotation.
                particle_transform,
                // Emission now runs after TransformSystems::Propagate so the
                // emitter sees its real parented world position. The detached
                // particle itself is born after that propagation pass; seed
                // its identical world transform explicitly so its very first
                // extracted GPU frame does not flash at the origin.
                GlobalTransform::from(particle_transform),
                Visibility::Inherited,
            ));
            if material_is_animated {
                particle_entity.insert(NativeAnimatedParticleMaterial {
                    sample: particle_material_animation_sample(&emitter.plan, 0.0),
                });
            }
        }
    }
}

pub(super) fn update_animated_particle_materials(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TutorialParticleMaterial>>,
    mut visual_assets: ResMut<NativeVisualAssets>,
    mut particles: Query<(
        &NativeParticle,
        &mut MeshMaterial3d<TutorialParticleMaterial>,
        &mut NativeAnimatedParticleMaterial,
    )>,
) {
    for (particle, mut material, mut animation) in &mut particles {
        if !particle.alive {
            continue;
        }
        let normalized = (particle.age / particle.plan.lifetime).clamp(0.0, 1.0);
        let sample = particle_material_animation_sample(&particle.plan, normalized);
        if animation.sample == sample {
            continue;
        }
        let shared = shared_particle_material(
            &particle.plan,
            normalized,
            &mut images,
            &mut materials,
            &mut visual_assets,
        );
        if material.0 != shared {
            material.0 = shared;
        }
        animation.sample = sample;
    }
}

pub(super) fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Option<Res<AssetServer>>,
    audio_catalog: Option<Res<NativeAudioCatalog>>,
    audio_mix: Option<Res<RetrobutionAudioMix>>,
    projectile_visual_readiness: Option<Res<TutorialProjectileVisualReadiness>>,
    mut runtime: ResMut<TutorialEffectRuntime>,
    mut projectiles: Query<(Entity, &mut Transform, &mut NativeProjectile, &NativeRoot)>,
    detached: Query<(Entity, &NativeDetachedOwner)>,
    terrain: Query<(&crate::native_terrain::NativeHeightmapCollider, &GlobalTransform)>,
    colliders: Query<(&crate::world::AuthoredTriMeshCollider, &GlobalTransform, &crate::world::AuthoredColliderWorldBounds)>,
    npcs: Query<(&crate::entity_lifecycle::NetworkNpcAppearance0104, &GlobalTransform), With<crate::network_world_runtime::NetworkNpcVisual0104>>,
    mut gameplay: Option<ResMut<crate::world_behaviour::WorldGameplayIntentQueue>>,
    content: Option<Res<crate::tutorial_mission_content::TutorialMissionContent>>,
) {
    let target_shape = |npc: &crate::entity_lifecycle::NetworkNpcAppearance0104, global: &GlobalTransform| {
        let definition = content.as_deref()?.gameplay_npc(npc.0.npc_type)?;
        if npc.0.hp <= 0 || definition.team == 1 || definition.npc_class > 110 || definition.npc_class == 25 { return None; }
        let half_height = definition.height_server_units as f32 * 0.005;
        let radius = (definition.radius_server_units as f32 * 0.01).max(half_height);
        Some((global.translation() + Vec3::Y * half_height, radius))
    };
    for (entity, mut transform, mut projectile, root) in &mut projectiles {
        if projectile.waiting_for_mesh_surface {
            continue;
        }
        if projectile.waiting_for_trail_prewarm {
            if projectile_visual_readiness
                .as_deref()
                .is_some_and(|readiness| !readiness.ready)
            {
                continue;
            }
            projectile.waiting_for_trail_prewarm = false;
        }
        let target = projectile.target;
        let previous_position = projectile.position();
        let destroyed = match &mut projectile.motion {
            NativeProjectileMotion::Oni(state) => {
                state.step(time.delta_secs(), target);
                state.phase == TutorialOniMotionPhase::Destroyed
            }
            NativeProjectileMotion::Linear {
                position,
                hide_remaining,
                elapsed,
                duration,
            } => {
                let mut delta = time.delta_secs();
                if *hide_remaining > 0.0 {
                    let consumed = delta.min(*hide_remaining);
                    *hide_remaining -= consumed;
                    delta -= consumed;
                }
                if *hide_remaining <= 0.0 {
                    if *duration <= 0.0 {
                        *position = target;
                    } else if delta > 0.0 {
                        let remaining = (*duration - *elapsed).max(f32::EPSILON);
                        let factor = (delta / remaining).min(1.0);
                        *position += (target - *position) * factor;
                        *elapsed += delta;
                        if *elapsed >= *duration {
                            *position = target;
                        }
                    }
                }
                *hide_remaining <= 0.0 && (*duration <= 0.0 || *elapsed >= *duration)
            }
            NativeProjectileMotion::Warhead {
                position, velocity, gravity, elapsed, duration, authority,
            } => {
                let delta = time.delta_secs().min((*duration - *elapsed).max(0.0));
                if *gravity { velocity.y -= 9.8 * delta; }
                let start = *position;
                let end = start + *velocity * delta;
                let probe_end = end + velocity.normalize_or_zero() * 0.2;
                let mut hit: Option<(f32, Vec3, Vec3)> = None;
                for (collider, global) in &terrain {
                    if let Some(h) = collider.segment_hit(global, start, probe_end) {
                        if hit.is_none_or(|old| h.fraction < old.0) { hit = Some((h.fraction, h.point, h.normal)); }
                    }
                }
                for (collider, global, bounds) in &colliders {
                    if collider.is_trigger() { continue; }
                    if let Some(h) = crate::world::collider_segment_hit_with_bounds(collider, global.to_matrix(), bounds, start, probe_end) {
                        if hit.is_none_or(|old| h.fraction < old.0) { hit = Some((h.fraction, h.point, h.normal)); }
                    }
                }
                let mut crashed = false;
                *position = end;
                if let Some((_, point, normal)) = hit {
                    *position = point + normal * 0.201;
                    if *gravity {
                        let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
                        let horizontal = (Vec3::new(velocity.x, 0.0, velocity.z).normalize_or_zero()
                            + Vec3::new(normal.x, 0.0, normal.z)).normalize_or_zero();
                        velocity.x = horizontal.x * horizontal_speed;
                        velocity.z = horizontal.z * horizontal_speed;
                        velocity.y = normal.y * velocity.y.abs() * 0.8;
                    } else { crashed = true; }
                }
                // Sweep the whole travelled segment so fast rockets cannot skip a target.
                let npc_hit = npcs.iter().filter_map(|(npc, global)| {
                    let (point, radius) = target_shape(npc, global)?;
                    warhead_sphere_hit(start, *position, point, radius + 0.2)
                }).min_by(f32::total_cmp);
                if let Some(fraction) = npc_hit {
                    *position = start.lerp(*position, fraction);
                    crashed = true;
                }
                *elapsed += time.delta_secs();
                crashed |= *duration <= 0.0 || *elapsed >= *duration;
                if crashed {
                    if let Some(owner) = authority.take() {
                        let mut targets = npcs.iter().filter_map(|(npc, global)| {
                            let (point, radius) = target_shape(npc, global)?;
                            let distance = point.distance_squared(*position);
                            (distance <= (radius + owner.blast_radius).powi(2)).then_some((distance, npc.0.npc_id))
                        }).collect::<Vec<_>>();
                        targets.sort_by(|a,b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                        targets.truncate(owner.target_capacity);
                        let [x,y,z] = crate::coordinates::ProtocolPosition::from_native(*position).raw();
                        use ffone_protocol::WirePayload;
                        let mut payload = ffone_protocol::wire_0104::PcRocketStyleHitRequest0104 {
                            bullet_id: owner.bullet_id, x,y,z, target_cnt: targets.len() as i32,
                        }.encode();
                        for (_, id) in targets { payload.extend(id.to_le_bytes()); payload.extend(4_i32.to_le_bytes()); }
                        if let Some(queue) = gameplay.as_deref_mut() { queue.push_payload(0x1300_001d, payload); }
                    }
                }
                crashed
            }
        };
        let position = projectile.position();
        let movement = position - previous_position;
        transform.translation = position;
        if movement.length_squared() > f32::EPSILON {
            // Legacy projectile models are parented below the moving root and
            // inherit its per-frame Quaternion.LookRotation.
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, movement.normalize());
        }
        if destroyed {
            if let Some(impact) = projectile.impact.take()
                && runtime.active.contains_key(&impact.instance_id)
            {
                let sound_path = impact.sound_path.clone();
                // `BulletMoveScript.Crash` forwards the carrier transform's
                // current rotation to EffectLoad. Directional success
                // emitters therefore inherit the final flight direction.
                spawn_world_impact(
                    &mut commands,
                    asset_server.as_deref(),
                    position,
                    transform.rotation,
                    impact,
                );
                if let (Some(sound_path), Some(asset_server), Some(audio_catalog)) =
                    (sound_path, asset_server.as_ref(), audio_catalog.as_ref())
                {
                    match audio_catalog.by_legacy_path(&sound_path) {
                        Some(audio) if audio.category == NativeAudioCategory::Sfx => {
                            let gain = audio_mix
                                .as_deref()
                                .map_or(0.5, |mix| mix.effects)
                                .clamp(0.0, 1.0);
                            commands.spawn((
                                Name::new(format!("Projectile impact SFX {}", audio.true_name)),
                                NativeImpactSfx,
                                GameplaySfxAudio,
                                Transform::from_translation(position),
                                AudioPlayer::new(asset_server.load(audio.path.clone())),
                                native_impact_playback_settings(gain),
                            ));
                        }
                        Some(audio) => warn!(
                            "Projectile impact audio {:?} resolved to non-SFX category {:?}",
                            audio.true_name, audio.category
                        ),
                        None => warn!(
                            "Projectile impact SFX path {sound_path:?} is absent from the native audio catalog"
                        ),
                    }
                }
            }
            commands.entity(entity).despawn();
            for (detached_entity, owner) in &detached {
                if owner.0 == root.instance_id {
                    commands.entity(detached_entity).despawn();
                }
            }
            runtime.forget_instance(root.instance_id);
        }
    }
}

pub(super) fn update_trails(
    time: Res<Time>,
    cameras: Query<&GlobalTransform, (With<Camera3d>, With<LegacyOrbitCamera>)>,
    projectiles: Query<&NativeProjectile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut trails: Query<(&Mesh3d, &mut NativeTrail)>,
) {
    let camera_right = cameras
        .iter()
        .next()
        .map(|camera| camera.compute_transform().rotation * Vec3::X)
        .unwrap_or(Vec3::X);
    for (mesh_handle, mut trail) in &mut trails {
        let Ok(projectile) = projectiles.get(trail.projectile) else {
            continue;
        };
        if projectile.waiting_for_mesh_surface {
            continue;
        }
        if projectile.waiting_for_trail_prewarm {
            continue;
        }
        let current = projectile.position();
        let interpolation_step = trail.plan.interpolation_step;
        push_interpolated_trail_points(&mut trail.points, current, interpolation_step);
        trail.elapsed += time.delta_secs();
        if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = trail_mesh(
                &trail.points,
                camera_right,
                trail.elapsed,
                trail.plan.height * trail.scale,
            );
        }
    }
}
