use crate::app::*;

/// NpcIconMode owns the departure presentation; the tutorial virtual server
/// applies the destination only after its 1.5-second wait has finished.
#[derive(Debug)]
pub(super) struct PendingTutorialNpcWarp {
    identity: NormalNpcWarpIdentity,
    elapsed_seconds: Option<f32>,
}

impl PendingTutorialNpcWarp {
    pub(super) fn new(identity: NormalNpcWarpIdentity) -> Self {
        Self {
            identity,
            elapsed_seconds: None,
        }
    }
}

pub(super) fn advance_tutorial_npc_warp(
    time: Res<Time>,
    mut mission: ResMut<TutorialMissionRuntime>,
    mut model: ResMut<MissionUiModel>,
    mut tutorial: ResMut<TutorialSession>,
    content: Res<TutorialMissionContent>,
    mut status: ResMut<RuntimeStatus>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut audio: ResMut<GameplayAudioRuntime>,
    mut actor_commands: ResMut<TutorialActorCommandQueue>,
    mut ambient: ResMut<TutorialAmbientRuntime>,
    mut fade: ResMut<NormalNpcWarpRuntime>,
    actors: Query<(Entity, &TutorialActor)>,
    mut players: Query<(&mut Transform, &mut LegacyPlayerController), With<LocalPlayer>>,
) {
    let Some(mut pending) = mission.pending_warp.take() else {
        return;
    };
    let identity = pending.identity;
    if tutorial.completion_requested
        || !model.pending_warp_matches(
            identity.npc_id,
            identity.npc_type,
            identity.warp_id,
            identity.required_task_id,
            identity.target,
        )
    {
        identity.reject_ui(&mut model);
        return;
    }
    let Some((npc_entity, _)) = actors.iter().find(|(_, actor)| {
        actor.id == identity.npc_id
            && actor.npc_type == identity.npc_type
            && actor.interacting
            && actor.is_alive()
    }) else {
        identity.reject_ui(&mut model);
        status.message = "Tutorial warp cancelled: its live NPC owner disappeared".to_owned();
        return;
    };
    let Ok((mut transform, mut controller)) = players.single_mut() else {
        identity.reject_ui(&mut model);
        status.message = "Tutorial warp cancelled: no unique local player".to_owned();
        return;
    };

    if pending.elapsed_seconds.is_none() {
        audio.queue_legacy_world_sound(npc_entity, NORMAL_NPC_WARP_SOUND_TRUE_NAME);
        if let Some(npc) = content.gameplay_npc(identity.npc_type) {
            audio.queue_legacy_npc_voice(
                npc_entity,
                &npc.move_voice_owner,
                LegacyNpcVoiceCue::WarpOk,
            );
        }
        effects.enqueue(TutorialEffectRuntimeCommand::Preload {
            effect_id: NORMAL_NPC_WARP_EFFECT_ID,
            source_line: line!(),
        });
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: NORMAL_NPC_WARP_EFFECT_ID,
            placement: TutorialEffectPlacement::World {
                position: transform.translation,
                rotation: transform.rotation,
            },
            scale: 1.0,
            tracked: false,
            name: None,
            destroy_after_seconds: None,
            source_line: line!(),
        });
        pending.elapsed_seconds = Some(0.0);
        mission.pending_warp = Some(pending);
        return;
    }
    let elapsed = pending.elapsed_seconds.unwrap() + time.delta_secs().max(0.0);
    if elapsed < NORMAL_NPC_WARP_DELAY_SECONDS {
        pending.elapsed_seconds = Some(elapsed);
        mission.pending_warp = Some(pending);
        return;
    }

    match apply_tutorial_warp(
        &content,
        &mut mission,
        status.map_number,
        identity.npc_type,
        identity.warp_id,
        identity.required_task_id,
        identity.target,
        &mut transform,
        &mut controller,
    ) {
        Ok(()) => {
            identity.confirm_ui(&mut model);
            fade.start_window_in_fade();
            actor_commands.clear_interactions();
            tutorial.progress.receive_event(TutorialEvent::NpcWarp, 1);
            if let Some(cue) = tutorial_ambient_after_confirmed_warp(identity.warp_id) {
                ambient.request(Some(cue));
            }
        }
        Err(error) => {
            identity.reject_ui(&mut model);
            status.message = format!("Tutorial warp {} rejected: {error}", identity.warp_id);
        }
    }
}
