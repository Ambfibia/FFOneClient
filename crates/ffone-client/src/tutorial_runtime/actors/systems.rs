use super::*;

/// Persistent `NpcMoveController.bForceUpdate` state.
///
/// Retrobution's controller returns before target movement, collision and
/// gravity while this flag is set. Tutorial coroutines use that contract for
/// actors whose transforms they animate directly, including Buttercup's
/// camera-relative flyby in `BasicMoveEvent`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Component)]
pub struct TutorialActorForceUpdate;

/// Exclusive system so a Spawn followed by Damage/Delete in the same queue
/// observes strict FIFO semantics instead of Bevy's deferred command boundary.
pub fn apply_tutorial_actor_commands(world: &mut World) {
    let mut pending = world.resource_mut::<TutorialActorCommandQueue>().take_all();
    while let Some(command) = pending.pop_front() {
        match command {
            TutorialActorCommand::Spawn(spawn) => spawn_actor(world, spawn),
            TutorialActorCommand::Delete { id } => delete_actor(world, id),
            TutorialActorCommand::WarpNative { id, target } => warp_actor_native(world, id, target),
            TutorialActorCommand::TranslateNative { id, delta } => {
                translate_actor_native(world, id, delta)
            }
            TutorialActorCommand::MoveNative {
                id,
                target,
                speed_units_per_second,
            } => move_actor_native(world, id, target, speed_units_per_second),
            TutorialActorCommand::StopMotion { id } => stop_actor_motion(world, id),
            TutorialActorCommand::SetNativeRotation { id, rotation } => {
                rotate_actor_native(world, id, rotation)
            }
            TutorialActorCommand::FaceNativePosition { id, target } => {
                face_actor_native_position(world, id, target)
            }
            TutorialActorCommand::FaceActor { id, target } => face_actor(world, id, target),
            TutorialActorCommand::PlayPose { id, clip, once } => {
                play_actor_pose(world, id, clip, once);
            }
            TutorialActorCommand::SetPoseState { id, state } => {
                set_actor_pose_state(world, id, state)
            }
            TutorialActorCommand::ForceUpdate { id } => force_update_actor(world, id),
            TutorialActorCommand::Damage { id, amount } => damage_actor(world, id, amount),
            TutorialActorCommand::SetInteracting { id, interacting } => {
                set_actor_interacting(world, id, interacting);
            }
            TutorialActorCommand::ClearInteractions => clear_actor_interactions(world),
            TutorialActorCommand::AttemptPlayerAttack {
                id,
                player_position,
            } => attempt_player_attack(world, id, player_position),
            TutorialActorCommand::ConfigureDemoMonster { dont_kill } => {
                configure_demo_monster(world, dont_kill);
            }
        }
    }
    start_unforced_actor_stand_poses(world);
}

pub(super) fn force_update_actor(world: &mut World, id: i32) {
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        entity_mut.insert(TutorialActorForceUpdate);
        let mut pose = entity_mut
            .get_mut::<TutorialActorPose>()
            .map(|pose| *pose)
            .unwrap_or_default();
        pose.force_update = true;
        pose.request_serial = pose.request_serial.wrapping_add(1);
        pose.restart_serial = pose.request_serial;
        entity_mut.insert(pose);
    }
}

pub fn advance_tutorial_actor_motion(
    time: Res<Time>,
    mut commands: Commands,
    mut stand_random: ResMut<TutorialActorStandRandomStream>,
    mut actors: Query<
        (
            Entity,
            &mut Transform,
            &TutorialActorMotion,
            &TutorialActorPose,
            Option<&TutorialActorCombatTransition>,
        ),
        (With<TutorialActor>, Without<TutorialActorForceUpdate>),
    >,
) {
    let maximum_delta_seconds = time.delta_secs().max(0.0);
    for (entity, mut transform, motion, pose, combat) in &mut actors {
        // Retrobution's `NpcMoveController.DoMovementCalculations` explicitly
        // clears `val.y` before both its arrival check and target translation.
        // The server target height is therefore never locomotion input: the
        // controller's downward ray and gravity own Y independently. Mixing Y
        // into this distance makes a grounded actor chase a height it can
        // never reach, so it remains in walk and is pulled into the floor on
        // every frame before grounding snaps it back up.
        let offset = Vec3::new(
            motion.target.x - transform.translation.x,
            0.0,
            motion.target.z - transform.translation.z,
        );
        let distance = offset.length();
        let maximum_step = motion.speed_units_per_second * maximum_delta_seconds;
        if distance <= maximum_step.max(0.000_1) {
            transform.translation.x = motion.target.x;
            transform.translation.z = motion.target.z;
            commands.entity(entity).remove::<TutorialActorMotion>();
            if combat.is_none() && pose.role == LegacyNpcAnimationRole::Locomotion {
                let request_serial = pose.request_serial.wrapping_add(1);
                let (clip, resolved_clip, role, blend) = if pose.clip == Some("run") {
                    // `NpcAnimation.StandMotion` remembers the previous run in
                    // `bOldWalk=false` and enters AttackReady instead of a
                    // random stand when the destination is reached.
                    (
                        "ready",
                        "ready",
                        LegacyNpcAnimationRole::Ready,
                        LegacyAnimationBlend::CrossFade200Ms,
                    )
                } else {
                    let stand = tutorial_actor_idle_stand_for_roll(
                        stand_random.draw_percent(),
                        pose.resolved_clip,
                    );
                    (
                        "idle",
                        stand,
                        LegacyNpcAnimationRole::Stand,
                        LegacyAnimationBlend::CrossFade300Ms,
                    )
                };
                commands.entity(entity).insert(TutorialActorPose {
                    clip: Some(clip),
                    resolved_clip: Some(resolved_clip),
                    once: false,
                    role,
                    blend,
                    state: TutorialActorPoseState::Playing,
                    force_update: false,
                    request_serial,
                    restart_serial: request_serial,
                });
            }
            continue;
        }
        if distance > 0.0 && maximum_step > 0.0 {
            transform.translation += offset / distance * maximum_step;
        }
        if combat.is_none()
            && pose.state == TutorialActorPoseState::Playing
            && matches!(
                pose.role,
                LegacyNpcAnimationRole::Stand
                    | LegacyNpcAnimationRole::Ready
                    | LegacyNpcAnimationRole::Locomotion
            )
        {
            let desired = tutorial_actor_move_clip(motion.speed_units_per_second);
            if pose.role != LegacyNpcAnimationRole::Locomotion || pose.clip != Some(desired) {
                let request_serial = pose.request_serial.wrapping_add(1);
                commands.entity(entity).insert(TutorialActorPose {
                    clip: Some(desired),
                    resolved_clip: Some(desired),
                    once: false,
                    role: LegacyNpcAnimationRole::Locomotion,
                    blend: LegacyAnimationBlend::CrossFade300Ms,
                    state: TutorialActorPoseState::Playing,
                    force_update: false,
                    request_serial,
                    restart_serial: request_serial,
                });
            }
        }
    }
}

pub fn refresh_tutorial_npc_observation(
    avatars: Query<&Transform, (With<LegacyAvatarTargetFeed>, Without<TutorialActor>)>,
    actors: Query<(&TutorialActor, &Transform)>,
    mut snapshot: ResMut<TutorialNpcObservationSnapshot>,
) {
    let Ok(avatar) = avatars.single() else {
        snapshot.0 = TutorialNpcObservation::default();
        return;
    };
    snapshot.0 = build_tutorial_npc_observation(
        avatar.translation,
        actors
            .iter()
            .map(|(actor, transform)| TutorialActorObservationSample {
                id: actor.id,
                position: transform.translation,
                damaged: actor.damaged,
                interacting: actor.interacting,
            }),
    );
}
