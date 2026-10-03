use super::*;

pub(in super::super) fn resolve_pending_tutorial_actor_effects(
    mission_runtime: Res<TutorialMissionRuntime>,
    actor_registry: Res<TutorialActorRegistry>,
    actor_transforms: Query<&Transform, With<TutorialActor>>,
    mut pending: ResMut<PendingTutorialActorEffects>,
    mut effect_runtime: ResMut<TutorialEffectRuntime>,
) {
    let active_sequence = mission_runtime.auxiliary.active();
    let mut unresolved = VecDeque::new();
    while let Some(request) = pending.spawns.pop_front() {
        // A deferred request from a stopped/skipped auxiliary coroutine must
        // never materialize later in an unrelated tutorial stage.
        if active_sequence != Some(request.sequence) {
            continue;
        }
        let Some(actor_position) = actor_registry
            .entity(request.runtime_id)
            .and_then(|entity| actor_transforms.get(entity).ok())
            .map(|transform| transform.translation)
        else {
            // AddNpc commands are applied at the start of the following frame.
            // Retain the request so a timeline/system-order race cannot lose
            // the one-shot ES668 spawn.
            unresolved.push_back(request);
            continue;
        };
        effect_runtime.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: request.effect_id,
            placement: TutorialEffectPlacement::World {
                position: tutorial_actor_effect_world_position(actor_position, request.offset),
                rotation: Quat::IDENTITY,
            },
            scale: 1.0,
            // `cntutorialscript.AddEffect` stores this object in `arrEffect`;
            // it survives until TalkNumTwo1 calls the later ClearEffect.
            tracked: true,
            name: None,
            destroy_after_seconds: None,
            source_line: request.source_line,
        });
    }
    pending.spawns = unresolved;
}
