use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(super) struct RetrobutionActorAnimationEventCursor {
    pub(super) request_serial: u64,
    pub(super) restart_serial: u64,
    pub(super) node: AnimationNodeIndex,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}

pub(super) fn emit_retrobution_actor_animation_events(
    mut commands: Commands,
    actors: Query<(
        Entity,
        &TutorialActor,
        &NetworkNpcVisual0104,
        &GlobalTransform,
        Option<&RetrobutionActorAnimationEventCursor>,
        Option<&TutorialActorPlayerKillDeathPresentation>,
    )>,
    players: Query<(&AnimationPlayer, &TutorialActorAnimationPlayback)>,
    content: Res<TutorialMissionContent>,
    mut runtime: ResMut<TutorialEffectRuntime>,
    mut actor_events: ResMut<TutorialActorEventQueue>,
) {
    for (actor_root, actor, visual, actor_transform, cursor, death_marker) in &actors {
        let Some((active, playback, node)) = players.iter().find_map(|(player, playback)| {
            if playback.actor_root != actor_root || playback.terminally_unavailable {
                return None;
            }
            let node = playback.node?;
            player
                .animation(node)
                .map(|active| (active, playback, node))
        }) else {
            continue;
        };
        let Some(clip) = playback.resolved_clip.or(playback.clip) else {
            continue;
        };
        let matching_effects = RETROBUTION_ACTOR_EFFECT_EVENTS
            .iter()
            .filter(|event| event.npc_type == actor.npc_type && event.clip == clip)
            .collect::<Vec<_>>();
        let death_presentation = death_marker
            .filter(|marker| marker.request_serial == playback.request_serial && clip == "death")
            .and_then(|_| {
                RETROBUTION_ACTOR_DEATH_PRESENTATION_EVENTS
                    .iter()
                    .find(|event| event.npc_type == actor.npc_type)
            });
        if matching_effects.is_empty() && death_presentation.is_none() {
            continue;
        }

        let same_playback = cursor.is_some_and(|cursor| {
            cursor.request_serial == playback.request_serial
                && cursor.restart_serial == playback.restart_serial
                && cursor.node == node
                && active.completions() >= cursor.completions
        });
        let (previous_seek, previous_completions) = if same_playback {
            let cursor = cursor.expect("same playback requires an event cursor");
            (cursor.seek_time, cursor.completions)
        } else {
            (0.0, 0)
        };

        for event in matching_effects {
            let crossings = animation_event_crossings(
                previous_seek,
                previous_completions,
                active.seek_time(),
                active.completions(),
                active.repeat_mode(),
                event.event_seconds,
            );
            for _ in 0..crossings {
                let local_rotation_after_parenting = if event.node_name.is_some() {
                    Quat::from_rotation_x(90_f32.to_radians())
                } else {
                    Quat::IDENTITY
                };
                runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id: event.effect_id,
                    placement: TutorialEffectPlacement::ExactBone {
                        actor_id: actor.id,
                        node_name: event.node_name.unwrap_or(&visual.logical_name).to_owned(),
                        spawn_world_rotation: Quat::IDENTITY,
                        local_translation_after_parenting: Vec3::ZERO,
                        local_rotation_after_parenting,
                    },
                    scale: 1.0,
                    tracked: false,
                    // AnimationEventHandler owns one currentEffect slot per
                    // actor and replaces it only when the new prefab has the
                    // same Unity name. Key by actor + ES id: path/time-based
                    // names incorrectly allowed the two ES38 hand callbacks
                    // (and any repeated ES734 callback) to coexist.
                    name: Some(retrobution_actor_current_effect_name(
                        actor.id,
                        event.effect_id,
                    )),
                    destroy_after_seconds: None,
                    source_line: u32::try_from(event.source_clip_path_id).unwrap_or(line!()),
                });
            }
        }

        if let Some(event) = death_presentation {
            let crossings = animation_event_crossings(
                previous_seek,
                previous_completions,
                active.seek_time(),
                active.completions(),
                active.repeat_mode(),
                event.event_seconds,
            );
            if crossings > 0 {
                let (_, rotation, position) = actor_transform.to_scale_rotation_translation();
                let Some(scale) = content
                    .gameplay_npc(actor.npc_type)
                    .map(|definition| definition.radius() * 2.0)
                else {
                    continue;
                };
                runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id: event.effect_id,
                    placement: TutorialEffectPlacement::World { position, rotation },
                    scale,
                    tracked: false,
                    name: Some(format!(
                        "DeadMotion actor {} pathId {} ES{}",
                        actor.id, event.source_clip_path_id, event.effect_id
                    )),
                    destroy_after_seconds: None,
                    source_line: u32::try_from(event.source_clip_path_id).unwrap_or(line!()),
                });
                actor_events.push(TutorialActorEvent::PlayerKillDeathPresentation {
                    id: actor.id,
                    entity: actor_root,
                });
                commands
                    .entity(actor_root)
                    .remove::<TutorialActorPlayerKillDeathPresentation>();
            }
        }

        commands
            .entity(actor_root)
            .insert(RetrobutionActorAnimationEventCursor {
                request_serial: playback.request_serial,
                restart_serial: playback.restart_serial,
                node,
                seek_time: active.seek_time(),
                completions: active.completions(),
            });
    }
}

pub fn animation_event_crossings(
    previous_seek: f32,
    previous_completions: u32,
    current_seek: f32,
    current_completions: u32,
    repeat: RepeatAnimation,
    event_seconds: f32,
) -> u32 {
    if !previous_seek.is_finite()
        || !current_seek.is_finite()
        || !event_seconds.is_finite()
        || event_seconds < 0.0
        || current_completions < previous_completions
    {
        return 0;
    }
    let completion_delta = current_completions - previous_completions;
    if completion_delta == 0 {
        return u32::from(previous_seek < event_seconds && event_seconds <= current_seek);
    }
    if repeat != RepeatAnimation::Forever {
        return u32::from(previous_seek < event_seconds);
    }

    u32::from(previous_seek < event_seconds)
        .saturating_add(completion_delta.saturating_sub(1))
        .saturating_add(u32::from(event_seconds <= current_seek))
}
