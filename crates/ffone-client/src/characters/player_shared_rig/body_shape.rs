//! Actor-owned proportions layered over an independent preview idle.
use super::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativePlayerBodyShape {
    pub height: i8,
    pub body: i8,
}

impl NativePlayerBodyShape {
    pub fn normalized_times(self) -> [f32; 4] {
        let height = 1.0 - f32::from(self.height.clamp(0, 4)) / 4.0;
        let body = f32::from(self.body.clamp(0, 2)) / 2.0;
        [height, body, height, body]
    }
}

#[derive(Component)]
pub struct NativePlayerBodyShapePlayback {
    nodes: [AnimationNodeIndex; 4],
    durations: [f32; 4],
}

impl NativePlayerBodyShapePlayback {
    /// Paused clips still evaluate every frame, preserving their pose over idle.
    /// Also restores them after the preview's foreign-graph recovery.
    pub fn apply(&self, player: &mut AnimationPlayer, shape: NativePlayerBodyShape) {
        for ((node, duration), time) in self
            .nodes
            .into_iter()
            .zip(self.durations)
            .zip(shape.normalized_times())
        {
            let seek = duration * time;
            if player.animation(node).is_some_and(|active| {
                active.is_paused() && active.seek_time() == seek && active.weight() == 1.0
            }) {
                continue;
            }
            player
                .play(node)
                .set_seek_time(seek)
                .set_weight(1.0)
                .pause();
        }
    }
}

fn body_graph(
    idle: Handle<AnimationClip>,
    clips: [Handle<AnimationClip>; 4],
) -> (AnimationGraph, AnimationNodeIndex, [AnimationNodeIndex; 4]) {
    let mut graph = AnimationGraph::new();
    // Compose the absolute idle and bind-relative TR deltas within one Add.
    // The two scale clips share an ordinary layer, as in the world player rig.
    let body = graph.add_additive_blend(1.0, graph.root);
    let idle_layer = graph.add_blend(1.0, body);
    let idle = graph.add_clip(idle, 1.0, idle_layer);
    let height_add = graph.add_clip(clips[0].clone(), 1.0, body);
    let shape_add = graph.add_clip(clips[1].clone(), 1.0, body);
    let scales = graph.add_blend(1.0, body);
    let height = graph.add_clip(clips[2].clone(), 1.0, scales);
    let shape = graph.add_clip(clips[3].clone(), 1.0, scales);
    (graph, idle, [height_add, shape_add, height, shape])
}

pub(super) fn apply_body_shape(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut rigs: Query<(
        Entity,
        &NativePlayerRigRuntime,
        &NativePlayerBodyShape,
        &mut NativePlayerRigStand1Playback,
        Option<&NativePlayerBodyShapePlayback>,
        &mut NativePlayerRigStatus,
    )>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (root, runtime, shape, mut idle, body, mut status) in &mut rigs {
        if status.is_blocked() {
            continue;
        }
        let Ok(mut player) = players.get_mut(idle.animation_player) else {
            continue;
        };
        if let Some(body) = body {
            body.apply(&mut player, *shape);
            continue;
        }
        let Some(gltf) = gltfs.get(&runtime.skeleton_gltf) else {
            continue;
        };
        let names = ["height_Add", "shape_Add", "height", "shape"];
        let mut handles = Vec::new();
        let mut durations = Vec::new();
        for name in names {
            let Some(handle) = gltf.named_animations.get(name) else {
                *status = NativePlayerRigStatus::Blocked(format!(
                    "player skeleton is missing body clip {name}"
                ));
                break;
            };
            let Some(clip) = clips.get(handle) else { break };
            handles.push(handle.clone());
            durations.push(clip.duration());
        }
        let (Ok(handles), Ok(durations)) = (handles.try_into(), durations.try_into()) else {
            continue;
        };
        let Some(idle_clip) = gltf.named_animations.get(runtime.animation_name.as_str()) else {
            continue;
        };
        let (graph, node, nodes) = body_graph(idle_clip.clone(), handles);
        let body = NativePlayerBodyShapePlayback { nodes, durations };
        let seek = player
            .animation(idle.animation_node)
            .map_or(0.0, |active| active.seek_time());
        player.stop_all();
        player.start(node).set_seek_time(seek).repeat();
        body.apply(&mut player, *shape);
        idle.animation_graph = graphs.add(graph);
        idle.animation_node = node;
        commands
            .entity(idle.animation_player)
            .insert(AnimationGraphHandle(idle.animation_graph.clone()));
        commands.entity(root).insert(body);
    }
}

#[cfg(test)]
mod tests;
