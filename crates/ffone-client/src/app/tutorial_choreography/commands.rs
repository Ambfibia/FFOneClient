use super::*;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_choreography_effect_action(
    action: EffectAction,
    source_line: u32,
    frame: &TutorialChoreographyFrame,
    captures: &TutorialCameraCaptureStore,
    language: &Language,
    actor_registry: &TutorialActorRegistry,
    actor_transforms: &Query<&Transform, With<TutorialActor>>,
    issues: &mut TutorialChoreographyIssueQueue,
    effect_runtime: &mut TutorialEffectRuntime,
) {
    let mut add_world = |spawn: ffone_client::tutorial_choreography::EffectSpawn| {
        if let Some(position) = resolve_choreography_position(
            spawn.position,
            source_line,
            frame,
            captures,
            actor_registry,
            actor_transforms,
            issues,
        ) {
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id: spawn.effect_id,
                placement: TutorialEffectPlacement::World {
                    position,
                    rotation: Quat::IDENTITY,
                },
                scale: spawn.scale,
                tracked: spawn.tracked,
                name: tutorial_named_world_effect(source_line, spawn.effect_id).map(str::to_owned),
                destroy_after_seconds: None,
                source_line,
            });
        }
    };
    match action {
        EffectAction::Preload(effect_id) => {
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
                effect_id,
                source_line,
            });
        }
        EffectAction::Add(spawn) => add_world(spawn),
        EffectAction::AddBatch(spawns) => {
            for spawn in spawns.iter().copied() {
                add_world(spawn);
            }
        }
        EffectAction::AddAtNpcInclusive {
            effect_id,
            first,
            last,
            scale,
            tracked,
        } => {
            for id in first..=last {
                add_world(ffone_client::tutorial_choreography::EffectSpawn {
                    effect_id,
                    position: PositionExpr::Entity(EntityRef::Npc(id)),
                    scale,
                    tracked,
                });
            }
        }
        EffectAction::AttachBone(attachment) => {
            if let EffectLocaleGate::LegacyLocalEquals(expected_locale) = attachment.locale_gate {
                let actual_locale = i32::from(!is_english_locale(&language.effective));
                if actual_locale != expected_locale {
                    return;
                }
            }
            let Some(origin) = actor_registry
                .entity(attachment.actor_id)
                .and_then(|entity| actor_transforms.get(entity).ok())
                .map(|transform| transform.translation)
            else {
                issues.push(TutorialChoreographyIssue::MissingEntityReference {
                    source_line,
                    detail: "tutorial effect bone actor is unavailable",
                });
                return;
            };
            let Some(spawn_world_rotation) = resolve_choreography_rotation(
                attachment.spawn_world_rotation,
                source_line,
                origin,
                frame,
                captures,
                actor_registry,
                actor_transforms,
                issues,
            ) else {
                return;
            };
            let Some(local_rotation_after_parenting) = resolve_choreography_rotation(
                attachment.local_rotation_after_parenting,
                source_line,
                origin,
                frame,
                captures,
                actor_registry,
                actor_transforms,
                issues,
            ) else {
                return;
            };
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id: attachment.effect_id,
                placement: TutorialEffectPlacement::ExactBone {
                    actor_id: attachment.actor_id,
                    node_name: attachment.exact_node_name.to_owned(),
                    spawn_world_rotation,
                    local_translation_after_parenting: Vec3::ZERO,
                    local_rotation_after_parenting,
                },
                scale: attachment.scale,
                tracked: attachment.tracked,
                name: Some(attachment.name.to_owned()),
                destroy_after_seconds: Some(attachment.destroy_after_seconds),
                source_line,
            });
        }
        EffectAction::ClearTracked => {
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::ClearTracked { source_line });
        }
        EffectAction::DestroyNamed(name) => {
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
                name: name.to_owned(),
                source_line,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_choreography_projectile_action(
    action: ProjectileAction,
    source_line: u32,
    frame: &TutorialChoreographyFrame,
    actor_registry: &TutorialActorRegistry,
    actor_transforms: &Query<&Transform, With<TutorialActor>>,
    issues: &mut TutorialChoreographyIssueQueue,
    effect_runtime: &mut TutorialEffectRuntime,
    projectile_random: &mut TutorialProjectileRandomStream,
) {
    let reverse = projectile_action_reverse_mode(action);
    let (types, source, target) = match action {
        ProjectileAction::NpcToPlayerPair { npc_id, types } => {
            let Some(source) = resolve_choreography_entity_position(
                EntityRef::Npc(npc_id),
                source_line,
                frame,
                actor_registry,
                actor_transforms,
                issues,
            ) else {
                return;
            };
            (types, source, frame.player.translation)
        }
        ProjectileAction::PlayerToPositionPair { types } => {
            let Some(target) = resolve_choreography_entity_position(
                EntityRef::Npc(4000),
                source_line,
                frame,
                actor_registry,
                actor_transforms,
                issues,
            ) else {
                return;
            };
            (
                types,
                frame.player.translation,
                target + choreography_client_vector(ClientVec3::new(0.0, 3.0, 0.0)),
            )
        }
    };
    effect_runtime.enqueue(sampled_oni_projectile_pair_command(
        types,
        source,
        target,
        reverse,
        source_line,
        projectile_random,
    ));
}

pub(in super::super) fn sampled_oni_projectile_pair_command(
    types: [i32; 2],
    source: Vec3,
    target: Vec3,
    reverse: bool,
    source_line: u32,
    projectile_random: &mut TutorialProjectileRandomStream,
) -> TutorialEffectRuntimeCommand {
    TutorialEffectRuntimeCommand::ProjectilePairSampled {
        types,
        source,
        target,
        oni: true,
        priority: 0,
        reverse,
        sampled_initial_velocity: projectile_random.sample_oni_pair(reverse),
        source_line,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_choreography_npc_action(
    action: NpcAction,
    source_line: u32,
    frame: &mut TutorialChoreographyFrame,
    captures: &TutorialCameraCaptureStore,
    actor_registry: &TutorialActorRegistry,
    actor_transforms: &Query<&Transform, With<TutorialActor>>,
    actor_commands: &mut TutorialActorCommandQueue,
    nano_commands: &mut TutorialNanoPresentationCommandQueue,
    issues: &mut TutorialChoreographyIssueQueue,
) {
    let mut spawn = |spawn: ffone_client::tutorial_choreography::NpcSpawn| {
        actor_commands.spawn(TutorialNpcSpawn::new(
            spawn.id,
            spawn.npc_type,
            LegacySpawnPosition::centiunits(
                (spawn.position.x * 100.0) as i32,
                (spawn.position.y * 100.0) as i32,
                (spawn.position.z * 100.0) as i32,
            ),
            spawn.helper_angle_degrees,
        ));
        if let Some(clip) = spawn.initial_animation {
            actor_commands.play_pose(spawn.id, clip, false);
        }
        if spawn.force_update {
            actor_commands.force_update(spawn.id);
        }
    };
    match action {
        NpcAction::Spawn(value) => spawn(value),
        NpcAction::SpawnBatch(values) => {
            for value in values.iter().copied() {
                spawn(value);
            }
        }
        NpcAction::Delete(-1) => {
            nano_commands.destroy();
            frame.nano = None;
        }
        NpcAction::Delete(id) => actor_commands.delete(id),
        NpcAction::DeleteInclusive { first, last } => {
            for id in first..=last {
                actor_commands.delete(id);
            }
        }
        NpcAction::Angle {
            id: -1,
            helper_degrees,
        } => {
            let rotation =
                ProtocolYawDegrees::new(i32::from(helper_degrees)).native_root_rotation();
            nano_commands.set_rotation(rotation);
            let transform = frame
                .nano
                .get_or_insert_with(|| TutorialNanoPresentationSpawn::legacy_hidden().transform);
            transform.rotation = rotation;
        }
        NpcAction::Angle { id, helper_degrees } => {
            actor_commands.set_native_rotation(
                id,
                ProtocolYawDegrees::new(i32::from(helper_degrees)).native_root_rotation(),
            );
        }
        NpcAction::Move { id, target, speed } => {
            if let Some(target) = resolve_choreography_position(
                target,
                source_line,
                frame,
                captures,
                actor_registry,
                actor_transforms,
                issues,
            ) {
                if id == -1 {
                    nano_commands.set_translation(target);
                    let transform = frame.nano.get_or_insert_with(|| {
                        TutorialNanoPresentationSpawn::legacy_hidden().transform
                    });
                    transform.translation = target;
                } else {
                    actor_commands.move_native(id, target, speed as f32 / 100.0);
                }
            }
        }
        NpcAction::WarpServer { id, target } => {
            let target = choreography_client_vector(target.client());
            if id == -1 {
                nano_commands.set_translation(target);
                let transform = frame.nano.get_or_insert_with(|| {
                    TutorialNanoPresentationSpawn::legacy_hidden().transform
                });
                transform.translation = target;
            } else {
                actor_commands.warp_native(id, target);
            }
        }
        NpcAction::WarpClient { id, target } | NpcAction::SetPosition { id, target } => {
            if let Some(target) = resolve_choreography_position(
                target,
                source_line,
                frame,
                captures,
                actor_registry,
                actor_transforms,
                issues,
            ) {
                if id == -1 {
                    nano_commands.set_translation(target);
                    let transform = frame.nano.get_or_insert_with(|| {
                        TutorialNanoPresentationSpawn::legacy_hidden().transform
                    });
                    transform.translation = target;
                } else {
                    actor_commands.warp_native(id, target);
                }
            }
        }
        NpcAction::SetRotation { id: -1, rotation } => {
            let origin = frame
                .nano
                .map_or(Vec3::ZERO, |transform| transform.translation);
            if let Some(rotation) = resolve_choreography_rotation(
                rotation,
                source_line,
                origin,
                frame,
                captures,
                actor_registry,
                actor_transforms,
                issues,
            ) {
                let rotation = choreography_character_root_rotation(rotation);
                nano_commands.set_rotation(rotation);
                let transform = frame.nano.get_or_insert_with(|| {
                    TutorialNanoPresentationSpawn::legacy_hidden().transform
                });
                transform.rotation = rotation;
            }
        }
        NpcAction::SetRotation { id, rotation } => match rotation {
            RotationExpr::FaceEntity(entity) => {
                if let Some(target) = resolve_choreography_entity_position(
                    entity,
                    source_line,
                    frame,
                    actor_registry,
                    actor_transforms,
                    issues,
                ) {
                    actor_commands.face_native_position(id, target);
                }
            }
            RotationExpr::FacePosition(position) => {
                if let Some(target) = resolve_choreography_position(
                    position,
                    source_line,
                    frame,
                    captures,
                    actor_registry,
                    actor_transforms,
                    issues,
                ) {
                    actor_commands.face_native_position(id, target);
                }
            }
            rotation => {
                let origin = actor_registry
                    .entity(id)
                    .and_then(|entity| actor_transforms.get(entity).ok())
                    .map_or(Vec3::ZERO, |transform| transform.translation);
                if let Some(rotation) = resolve_choreography_rotation(
                    rotation,
                    source_line,
                    origin,
                    frame,
                    captures,
                    actor_registry,
                    actor_transforms,
                    issues,
                ) {
                    actor_commands
                        .set_native_rotation(id, choreography_character_root_rotation(rotation));
                }
            }
        },
        NpcAction::Animation { id: -1, clip, .. } => nano_commands.play_animation(clip),
        NpcAction::Animation { id, clip, once, .. } => {
            actor_commands.play_pose(id, clip, once);
        }
        NpcAction::AnimationInclusive {
            first,
            last,
            clip,
            once,
        } => {
            for id in first..=last {
                actor_commands.play_pose(id, clip, once);
            }
        }
        NpcAction::Command { id, command } => match command {
            NpcCommand::ForceStop => actor_commands.stop_motion(id),
            NpcCommand::Pause => {
                actor_commands.set_pose_state(id, TutorialActorPoseState::Paused);
            }
            // Exact clean-client ownership: SendMessage("Dead", npc) is
            // received by `DeadMotion`, which only arms the later sink/effect
            // timer. `NpcAnimation` has no Dead method, so it keeps the current
            // base clip until the actor is deleted by the choreography.
            NpcCommand::Dead => {}
            NpcCommand::ForceUpdate => actor_commands.force_update(id),
            NpcCommand::SetAngleTo(entity) => {
                if let Some(target) = resolve_choreography_entity_position(
                    entity,
                    source_line,
                    frame,
                    actor_registry,
                    actor_transforms,
                    issues,
                ) {
                    actor_commands.face_native_position(id, target);
                }
            }
            NpcCommand::ForceAnimation(clip) => actor_commands.play_pose(id, clip, false),
        },
    }
}

pub(in super::super) fn apply_choreography_picture_action(
    action: TutorialPictureAction,
    presentation: &mut TutorialAuxiliaryPresentation,
) {
    if !presentation.choreography_owned {
        *presentation = TutorialAuxiliaryPresentation {
            choreography_owned: true,
            ..default()
        };
    }
    match action {
        TutorialPictureAction::ShowCursor {
            position,
            resource,
            direction,
            legacy_pivot,
        } => {
            let expected_resource = match direction {
                ChoreographyCursorDirection::Right => "tut_right",
                ChoreographyCursorDirection::Down => "tut_down",
            };
            debug_assert_eq!(resource, expected_resource);
            presentation.cursor = Some((
                resource,
                choreography_screen_point_to_auxiliary(position),
                AuxiliaryScreenPivot::Legacy(legacy_pivot),
            ));
            presentation.cursor_touched = true;
        }
        TutorialPictureAction::HideAll => {
            presentation.picture = None;
            presentation.cursor = None;
            presentation.picture_touched = true;
            presentation.cursor_touched = true;
        }
    }
}
