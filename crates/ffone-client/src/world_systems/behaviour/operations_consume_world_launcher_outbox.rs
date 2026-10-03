use super::*;

/// `controller.collisionFlags == 0` ends the clean EP-slope state.  Run this
/// after native collision so the decision observes this frame's actual floor
/// contact instead of the trigger sphere alone.
pub fn finish_unsupported_world_slopes(
    mut commands: Commands,
    mut players: Query<(
        Entity,
        &mut crate::movement::LegacyPlayerController,
        &WorldSlopeTraversal,
    )>,
) {
    for (entity, mut controller, traversal) in &mut players {
        // A jump/endpoint exit can remove the component through deferred
        // commands earlier in this frame. `finish_scripted_traversal` is the
        // immediate ownership hand-off, so never overwrite that exit merely
        // because the stale component remains visible until `apply_deferred`.
        if controller.grounded || controller.movement_enabled {
            continue;
        }
        let mut exit_velocity = controller.velocity;
        if exit_velocity.length_squared() <= f32::EPSILON {
            exit_velocity = traversal.drift * traversal.speed;
        }
        controller.finish_scripted_traversal(exit_velocity);
        commands.entity(entity).remove::<WorldSlopeTraversal>();
    }
}

/// Apply the launcher UI's source-ordered side effects to production world
/// entities instead of leaving them as a disconnected test outbox.
pub fn consume_world_launcher_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut outbox: ResMut<LauncherUiOutbox>,
    mut active: ResMut<ActiveWorldLauncher>,
    triggers: Query<&WorldTrigger>,
    mut players: Query<(
        Entity,
        &mut Transform,
        &mut crate::movement::LegacyPlayerController,
    )>,
    mut cameras: Query<&mut LegacyOrbitCamera>,
    mut gameplay: ResMut<WorldGameplayIntentQueue>,
) {
    for effect in outbox.drain() {
        match effect {
            LauncherUiEffect::Audio(cue) => {
                commands.spawn((
                    AudioPlayer::new(asset_server.load(cue.path())),
                    PlaybackSettings::DESPAWN,
                ));
            }
            LauncherUiEffect::SetTriggerRenderersVisible(visible) => {
                let Some(trigger_entity) = active.trigger else {
                    continue;
                };
                let Ok(trigger) = triggers.get(trigger_entity) else {
                    continue;
                };
                let visibility = if visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                for entity in &trigger.model_entities {
                    commands.entity(*entity).insert(visibility);
                }
            }
            LauncherUiEffect::SetAvatarRenderersVisible(visible) => {
                if let Ok((entity, _, _)) = players.single_mut() {
                    commands.entity(entity).insert(if visible {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    });
                }
            }
            LauncherUiEffect::SetCameraPosition(position) => {
                active.camera_position = Some(position);
            }
            LauncherUiEffect::SetCameraCustomControl(enabled) => {
                active.custom_camera = enabled;
            }
            LauncherUiEffect::SetCameraRotationY(yaw) => {
                if let Ok(mut camera) = cameras.single_mut() {
                    camera.yaw_degrees = yaw;
                }
            }
            LauncherUiEffect::StartLauncher(shot) => {
                let Ok((_entity, mut transform, mut controller)) = players.single_mut() else {
                    continue;
                };
                transform.translation = shot.position;
                transform.rotation =
                    LegacyUnityHeadingDegrees::new(shot.facing_yaw_degrees).native_root_rotation();
                controller.yaw_degrees = shot.facing_yaw_degrees;
                if controller.launch_scripted_ballistic(shot.velocity) {
                    let request = PcLauncherRequest0104 {
                        client_time: 0,
                        position: ProtocolPosition::from_native(shot.position).raw(),
                        velocity: ProtocolScaledVelocity::from_native(shot.velocity).raw(),
                        angle: LegacyUnityHeadingDegrees::new(shot.facing_yaw_degrees)
                            .to_protocol()
                            .degrees(),
                        // Clean StartLauncher writes only the vertical launch
                        // component to iSpeed, not the charged magnitude.
                        speed: (shot.velocity.y * 100.0) as i32,
                    };
                    let _ = gameplay.push(packet::P_CL2FE_REQ_PC_LAUNCHER, &request);
                }
            }
            LauncherUiEffect::RestoreAvatarPosition(position) => {
                if let Ok((_entity, mut transform, _)) = players.single_mut() {
                    transform.translation = position;
                }
            }
            LauncherUiEffect::ExitMode { .. } => {
                if let Ok((_entity, _, mut controller)) = players.single_mut() {
                    controller.movement_enabled = true;
                }
                active.custom_camera = false;
                active.trigger = None;
            }
            LauncherUiEffect::SetCombatIcon(_)
            | LauncherUiEffect::SetNameVisible(_)
            | LauncherUiEffect::RequestEscapeCloseGate { .. } => {}
        }
    }
}

pub(super) fn retire_world_behaviour_document(document: Arc<NativeWorldBehaviourDocument>) {
    // Dropping the final Arc for a dense decoded tile recursively frees tens
    // of thousands of heap allocations. Do that destructor work off the main
    // schedule so the final admission/reveal frame remains bounded too.
    IoTaskPool::get()
        .spawn(async move { drop(document) })
        .detach();
}

/// Transfer decoded documents out of roots as soon as world streaming marks
/// them for teardown. The incremental entity walk can last many frames, but a
/// component destructor must never make one of those frames recursively free
/// a multi-megabyte decoded graph on the main thread.
pub fn retire_unloading_world_behaviour_documents(
    mut pending_loads: Query<&mut PendingWorldBehaviourLoad, With<PendingNativeWorldSceneUnload>>,
    mut pending_behaviours: Query<
        &mut PendingWorldBehaviourSpawn,
        With<PendingNativeWorldSceneUnload>,
    >,
    mut pending_effects: Query<
        &mut PendingWorldScriptedEffects,
        With<PendingNativeWorldSceneUnload>,
    >,
) {
    for mut pending in &mut pending_loads {
        if let Some(task) = pending.0.take() {
            retire_world_behaviour_load_task(task);
        }
    }
    for mut pending in &mut pending_behaviours {
        if let Some(document) = pending.document.take() {
            retire_world_behaviour_document(document);
        }
    }
    for mut pending in &mut pending_effects {
        if let Some(document) = pending.document.take() {
            retire_world_behaviour_document(document);
        }
    }
}

/// Admit decoded controller graphs in bounded batches. The static tile and its
/// terrain remain hidden behind `NativeWorldBehaviourStatus::Loading` until
/// this graph and its separately-budgeted scripted effects are complete.
pub fn materialize_pending_world_behaviours(
    mut commands: Commands,
    mut pending_roots: Query<
        (
            Entity,
            &crate::world::NativeWorldSceneRoot,
            &mut PendingWorldBehaviourSpawn,
        ),
        Without<PendingNativeWorldSceneUnload>,
    >,
) {
    let mut remaining_global_work = WORLD_BEHAVIOUR_WORK_PER_FRAME;
    for (root, scene_root, mut pending) in &mut pending_roots {
        if remaining_global_work == 0 {
            break;
        }
        let root_budget = remaining_global_work.min(WORLD_BEHAVIOUR_WORK_PER_ROOT_PER_FRAME);
        let spent = streaming::materialize_world_behaviour_batch(
            &mut commands,
            root,
            &mut pending,
            root_budget,
        );
        remaining_global_work = remaining_global_work.saturating_sub(spent);
        if pending.phase != WorldBehaviourSpawnPhase::Complete {
            continue;
        }

        let document = Arc::clone(pending.document());
        let applied = WorldBehavioursApplied {
            document: document.id.clone(),
            billboards: document.billboards.len(),
            visibility_switches: document.visibility_switches.len(),
            effect_emitters: document.effect_emitters.len(),
            animations: document.animations.len(),
            triggers: document.triggers.len(),
            waypoints: document.waypoints.len(),
            trigger_volumes: document.trigger_volumes.len(),
            rigid_bodies: document.rigid_bodies.len(),
            blockers: document.blockers.len(),
        };
        info!(
            tile = %scene_root.name,
            billboards = applied.billboards,
            effect_emitters = applied.effect_emitters,
            triggers = applied.triggers,
            trigger_volumes = applied.trigger_volumes,
            animations = applied.animations,
            blockers = applied.blockers,
            native_particle_effects = pending.native_particle_effects,
            native_scripted_effects = pending.native_scripted_effects,
            "applied native world behaviours"
        );
        let native_scripted_effects = pending.native_scripted_effects;
        let owning_document = pending.take_document();
        drop(document);
        let mut root_commands = commands.entity(root);
        root_commands
            .remove::<PendingWorldBehaviourSpawn>()
            .insert(applied);
        if native_scripted_effects > 0 {
            root_commands.insert(PendingWorldScriptedEffects {
                document: Some(owning_document),
                next_record: 0,
            });
        } else {
            retire_world_behaviour_document(owning_document);
            root_commands.insert(NativeWorldBehaviourStatus::Ready);
        }
    }
}

/// Install the rendering-only behaviour slice used by deterministic native
/// world captures. Keeping this wiring beside the private animation systems
/// prevents the screenshot harness from silently omitting target samples or
/// visibility/material binding preparation as those systems evolve.
pub fn add_world_preview_behaviour_systems(app: &mut App) {
    app.add_systems(
        Update,
        retire_unloading_world_behaviour_documents
            .after(apply_pending_world_behaviours)
            .before(crate::world::NativeWorldSet::Unload),
    );
    app.add_systems(
        Update,
        (
            apply_pending_world_behaviours,
            materialize_pending_world_behaviours.after(apply_pending_world_behaviours),
            enqueue_pending_world_scripted_effects.after(materialize_pending_world_behaviours),
            update_world_animations,
            apply_world_animation_samples.after(update_world_animations),
            prepare_world_animation_material_bindings
                .after(update_world_animations)
                .before(crate::world::NativeWorldSet::RevealPresentation),
            update_world_animation_materials
                .after(update_world_animations)
                .after(prepare_world_animation_material_bindings)
                .before(crate::world::NativeWorldSet::RevealPresentation),
            update_world_billboards,
            update_world_visibility_switches,
            update_world_platforms.before(crate::world::NativeWorldSet::ResolveCollision),
        ),
    );
}
