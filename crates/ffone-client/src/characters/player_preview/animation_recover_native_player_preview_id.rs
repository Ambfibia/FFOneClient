use super::*;

/// Keeps the isolated preview on the idle graph that assembled it.
///
/// The full client owns several independent animation presenters. A late
/// presenter may replace an `AnimationPlayer` graph after the shared rig has
/// reported `ReadyAnimated`; the resulting clip then drives the same bone
/// palette and can fold the modular avatar into a death-like pose. Selection,
/// creation, and inventory previews never consume world animation requests,
/// so their one valid playback owner is the graph recorded by the rig.
pub(super) fn recover_native_player_preview_idle_animation(
    mut commands: Commands,
    previews: Query<
        (
            &NativePlayerRigStand1Playback,
            Option<&NativePlayerBodyShape>,
            Option<&NativePlayerBodyShapePlayback>,
        ),
        With<NativePlayerPreviewRoot>,
    >,
    mut players: Query<(&mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
) {
    for (playback, shape, body) in &previews {
        let Ok((mut player, graph)) = players.get_mut(playback.animation_player) else {
            continue;
        };
        if native_player_preview_idle_animation_is_healthy(&player, graph, playback) {
            continue;
        }

        player.stop_all();
        player
            .start(playback.animation_node)
            .set_repeat(RepeatAnimation::Forever)
            .set_speed(1.0)
            .resume();
        commands
            .entity(playback.animation_player)
            .insert(AnimationGraphHandle(playback.animation_graph.clone()));
        if let (Some(shape), Some(body)) = (shape, body) {
            body.apply(&mut player, *shape);
        }
    }
}

pub(super) fn native_player_preview_idle_animation_is_healthy(
    player: &AnimationPlayer,
    graph: Option<&AnimationGraphHandle>,
    playback: &NativePlayerRigStand1Playback,
) -> bool {
    if graph.is_none_or(|graph| graph.0 != playback.animation_graph) {
        return false;
    }
    player
        .animation(playback.animation_node)
        .is_some_and(|active| {
            active.repeat_mode() == RepeatAnimation::Forever
                && !active.is_paused()
                && active.speed().is_finite()
                && active.speed() > 0.0
        })
}
