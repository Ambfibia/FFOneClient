use super::*;
use bevy::animation::graph::{AnimationGraphNode, AnimationNodeType};

/// `MakeUperLayer` gives wound its own additive layer 104, above attacks.
/// Subtract the first sample before adding the moving sample, preserving the
/// authored clip and its interpolation: pose + wound(t) - wound(0).
#[derive(Clone, Copy, Debug)]
pub(super) struct PlayerDamagePlayback {
    reference: AnimationNodeIndex,
    pub(super) motion: AnimationNodeIndex,
}

impl PlayerDamagePlayback {
    pub(super) fn attach(graph: &mut AnimationGraph, clip: Handle<AnimationClip>) -> Self {
        let pose = graph.root;
        let root = graph.graph.add_node(AnimationGraphNode {
            node_type: AnimationNodeType::Add,
            weight: 1.0,
            mask: 0,
        });
        graph.graph.add_edge(root, pose, ());
        graph.root = root;
        let reference = graph.add_clip(clip.clone(), -1.0, root);
        let motion = graph.add_clip(clip, 1.0, root);
        Self { reference, motion }
    }

    pub(super) fn play(self, player: &mut AnimationPlayer) {
        // Legacy Animation.Play leaves an already playing wound running.
        if player
            .animation(self.motion)
            .is_some_and(|active| !active.is_finished())
        {
            return;
        }
        player
            .start(self.reference)
            .set_seek_time(0.0)
            .set_weight(1.0)
            .pause();
        player
            .start(self.motion)
            .set_repeat(RepeatAnimation::Never)
            .set_weight(1.0)
            .resume();
    }

    pub(super) fn advance(self, player: &mut AnimationPlayer, dead: bool) {
        if dead
            || player
                .animation(self.motion)
                .is_none_or(|active| active.is_finished())
        {
            player.stop(self.motion);
            player.stop(self.reference);
        }
    }
}
