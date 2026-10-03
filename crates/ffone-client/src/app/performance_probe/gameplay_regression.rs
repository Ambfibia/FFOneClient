//! Offline acceptance of the real player damage animation and its release.
use super::*;
use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeType};

#[derive(Resource, Default)]
struct DamageProbe {
    frame: u32, played: bool, released: bool,
    playback: Option<(Entity, bevy::animation::graph::AnimationNodeIndex)>,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<DamageProbe>()
        .add_systems(Update, drive.after(NetworkSessionLifecycleSet::Apply).before(animate_local_player_damage))
        .add_systems(Last, record.before(super::measure));
}

fn drive(capture: Res<Capture>, mut probe: ResMut<DamageProbe>, mut status: ResMut<RuntimeStatus>) {
    if capture.ready.is_none() || capture.samples.is_empty() { return; }
    probe.frame += 1;
    if probe.frame == 30 { status.hp = status.hp.map(|hp| hp - 50); }
}

fn record(
    mut commands: Commands, capture: Res<Capture>, mut probe: ResMut<DamageProbe>,
    players: Query<(Entity, &AnimationPlayer, &AnimationGraphHandle)>,
    graphs: Res<Assets<AnimationGraph>>, assets: Res<AssetServer>, catalog: Res<NativePlayerRigCatalog>,
) {
    for (entity, player, graph_handle) in &players {
        let Some(graph) = graphs.get(&graph_handle.0) else { continue; };
        for (node, active) in player.playing_animations() {
            let Some(graph_node) = graph.get(*node) else { continue; };
            let AnimationNodeType::Clip(handle) = &graph_node.node_type else { continue; };
            let Some(path) = assets.get_path(handle.id()) else { continue; };
            let wound = catalog.contract().genders.iter().any(|gender| {
                path.path() == std::path::Path::new(&gender.skeleton_glb)
                    && gender.clips.iter().any(|clip| clip.name == "woundupper"
                        && path.label() == Some(format!("Animation{}", clip.gltf_animation_index).as_str()))
            });
            if wound && graph_node.weight > 0.0 && !active.is_paused() && !active.is_finished() {
                probe.played = true;
                probe.playback = Some((entity, *node));
            }
        }
    }
    // The next base request replaces TutorialPlayerAnimationApplied. Track
    // the actual damage node through release instead of requiring that marker
    // to remain the last request for the rest of the replay.
    if let Some((entity, node)) = probe.playback {
        let active = players.get(entity).ok().and_then(|(_, p, _)| p.animation(node));
        if active.is_none_or(|a| a.is_finished()) { probe.released = true; }
    }
    if matches!(probe.frame, 32 | 38 | 100) {
        commands.spawn(Screenshot::primary_window()).observe(bevy::render::view::screenshot::save_to_disk(
            capture.output.join(format!("damage-{}.png", probe.frame))));
    }
    if probe.frame == 580 {
        assert!(probe.played && probe.released, "damage clip must play and release the upper body: played={}, released={}, playback={:?}", probe.played, probe.released, probe.playback);
        fs::write(capture.output.join("damage-animation.json"), serde_json::to_vec_pretty(
            &serde_json::json!({"played":probe.played,"released":probe.released})).unwrap()).unwrap();
    }
}
