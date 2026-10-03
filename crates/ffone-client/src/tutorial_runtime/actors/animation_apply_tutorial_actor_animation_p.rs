use super::*;

/// Strong handle for the exact GLB which supplied an actor scene. Animation
/// lookup always uses this parent GLTF asset so names remain case-sensitive.
#[derive(Debug, Clone, Component)]
pub struct TutorialActorAnimationSource {
    pub path: String,
    pub gltf: Handle<Gltf>,
}

/// Last pose revision applied to one AnimationPlayer below an actor root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct TutorialActorAnimationPlayback {
    pub actor_root: Entity,
    pub request_serial: u64,
    pub restart_serial: u64,
    pub clip: Option<&'static str>,
    /// Exact native GLB clip selected for the semantic request. For legacy
    /// `idle`, this is one of `stand1` through `stand4` (or `stand1` fallback).
    pub resolved_clip: Option<&'static str>,
    pub node: Option<AnimationNodeIndex>,
    /// The selected node is a delta clip attached to the graph's additive
    /// composition, so completion removes only this high layer and leaves the
    /// current base animation running.
    pub additive: bool,
    pub once: bool,
    pub state: TutorialActorPoseState,
    pub force_update: bool,
    pub terminally_unavailable: bool,
}

#[derive(Debug, Default, Component)]
pub struct TutorialActorAnimationIssueHistory {
    pub(super) reported_clips: BTreeSet<&'static str>,
}

/// App-lifetime strong handles and animation graphs shared by actors which use
/// the same published GLB.
///
/// Retrobution keeps every clip on one Unity `Animation` component and
/// cross-fades between them. Keeping a separate Bevy graph per clip makes a
/// graph-handle swap race the new pose request and can leave a skipped actor
/// visibly running. One graph per GLB preserves the original state machine and
/// lets `AnimationTransitions` blend exact named clips.
pub(super) struct TutorialActorPreparedAnimationGraph {
    pub(super) graph: Handle<AnimationGraph>,
    pub(super) nodes: HashMap<String, AnimationNodeIndex>,
    pub(super) additive_nodes: HashMap<String, AnimationNodeIndex>,
}

#[derive(Default, Resource)]
pub struct TutorialActorAnimationAssets {
    pub(super) gltfs: BTreeMap<String, Handle<Gltf>>,
    pub(super) graphs: HashMap<AssetId<Gltf>, TutorialActorPreparedAnimationGraph>,
}

impl TutorialActorAnimationAssets {
    #[must_use]
    pub fn retained_gltfs(&self) -> usize {
        self.gltfs.len()
    }

    #[must_use]
    pub fn retained_animation_graphs(&self) -> usize {
        self.graphs.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialActorPoseState {
    Playing,
    Paused,
    ForcedStop,
    Dead,
}

/// Semantic pose request retained on the actor even when its exact legacy
/// animation clip has not yet been published on the native rig.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct TutorialActorPose {
    pub clip: Option<&'static str>,
    /// Exact native clip chosen when a semantic request has multiple legacy
    /// outcomes. Non-`idle` requests retain their exact requested name here.
    pub resolved_clip: Option<&'static str>,
    pub once: bool,
    /// Low/high mode and blending decision owned by the recovered
    /// `NpcAnimation` state machine, not by the Bevy scene bridge.
    pub role: LegacyNpcAnimationRole,
    pub blend: LegacyAnimationBlend,
    pub state: TutorialActorPoseState,
    pub force_update: bool,
    /// Advances for every pose command, including state-only commands.
    pub request_serial: u64,
    /// Advances only when the current clip must restart from time zero.
    pub restart_serial: u64,
}

impl Default for TutorialActorPose {
    fn default() -> Self {
        Self {
            clip: None,
            resolved_clip: None,
            once: false,
            role: LegacyNpcAnimationRole::Forced,
            blend: LegacyAnimationBlend::Immediate,
            state: TutorialActorPoseState::Playing,
            force_update: false,
            request_serial: 0,
            restart_serial: 0,
        }
    }
}

pub(super) fn tutorial_actor_move_clip(speed_units_per_second: f32) -> &'static str {
    // `cntutorialscript.MoveNpc` writes iMoveStyle=Walk for iSpeed <= 400 and
    // Run only above 400. The choreography bridge converts those protocol
    // hundredths to native units before reaching this function.
    if speed_units_per_second > 4.0 {
        "run"
    } else {
        "walk"
    }
}

pub(super) fn play_actor_pose(world: &mut World, id: i32, clip: &'static str, once: bool) -> Option<u64> {
    let role = if clip == "idle" {
        LegacyNpcAnimationRole::Stand
    } else {
        npc_role_for_forced_clip(clip)
    };
    play_actor_pose_with_contract(
        world,
        id,
        clip,
        once,
        role,
        LegacyAnimationBlend::CrossFade300Ms,
    )
}

pub(super) fn play_actor_pose_with_contract(
    world: &mut World,
    id: i32,
    clip: &'static str,
    once: bool,
    role: LegacyNpcAnimationRole,
    blend: LegacyAnimationBlend,
) -> Option<u64> {
    let Some(entity) = actor_entity(world, id) else {
        return None;
    };
    let previous = world
        .get::<TutorialActorPose>(entity)
        .copied()
        .unwrap_or_default();
    let resolved_clip = if clip == "idle" {
        let roll = world
            .resource_mut::<TutorialActorStandRandomStream>()
            .draw_percent();
        tutorial_actor_idle_stand_for_roll(roll, previous.resolved_clip)
    } else {
        clip
    };
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        let request_serial = previous.request_serial.wrapping_add(1);
        entity_mut.insert(TutorialActorPose {
            clip: Some(clip),
            resolved_clip: Some(resolved_clip),
            once,
            role,
            blend,
            state: TutorialActorPoseState::Playing,
            force_update: false,
            request_serial,
            restart_serial: request_serial,
        });
        return Some(request_serial);
    }
    None
}

pub(super) fn play_actor_combat_pose(
    world: &mut World,
    id: i32,
    clip: &'static str,
    completion: TutorialActorCombatCompletion,
) {
    let role = match completion {
        TutorialActorCombatCompletion::Ready if clip.starts_with("melee") => {
            LegacyNpcAnimationRole::Melee
        }
        TutorialActorCombatCompletion::Ready => LegacyNpcAnimationRole::Wound,
        TutorialActorCombatCompletion::Despawn => LegacyNpcAnimationRole::Death,
    };
    let blend = tutorial_actor_native_combat_blend(role);
    let Some(request_serial) = play_actor_pose_with_contract(world, id, clip, true, role, blend)
    else {
        return;
    };
    if let Some(entity) = world.resource::<TutorialActorRegistry>().entity(id) {
        world
            .entity_mut(entity)
            .insert(TutorialActorCombatTransition {
                request_serial,
                completion,
            });
    }
}

pub(super) fn tutorial_actor_pose_effective_once(pose: &TutorialActorPose) -> bool {
    // Both AnimationNpc("idle") and AnimationNpcOnce("idle") call
    // ForceStandMotion, whose selected stand clip is always looped.
    matches!(pose.role.repeat(pose.once), LegacyAnimationRepeat::Once)
}

pub(super) fn set_actor_pose_state(world: &mut World, id: i32, state: TutorialActorPoseState) {
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        let mut pose = entity_mut
            .get_mut::<TutorialActorPose>()
            .map(|pose| *pose)
            .unwrap_or_default();
        pose.state = state;
        pose.request_serial = pose.request_serial.wrapping_add(1);
        entity_mut.insert(pose);
        if matches!(
            state,
            TutorialActorPoseState::ForcedStop | TutorialActorPoseState::Dead
        ) {
            entity_mut.remove::<TutorialActorMotion>();
        }
    }
}

/// Exact `NpcAnimation.AttackMotion` selection for tutorial combatants.
///
/// Retrobution first draws melee1/melee2, then checks the loaded Unity
/// `Animation` component and falls back to melee1 only when the selected state
/// is absent. The shared native Sneaky Spawn package publishes both clips, so
/// it follows the same random selection as the ordinary-game mob.
#[must_use]
pub(super) const fn tutorial_actor_attack_clip(_npc_type: i32, roll: u32) -> &'static str {
    if roll < 50 { "melee1" } else { "melee2" }
}

/// Central completion callback for the recovered `NpcAnimation` owner.
/// Melee/wound return to `ready`, death removes the tutorial mob, and a
/// clamped `ForceAnimationOnce` restarts instead of freezing on its last pose.
pub fn advance_tutorial_actor_combat_animation(
    mut commands: Commands,
    mut actors: Query<(
        Entity,
        &TutorialActor,
        &mut TutorialActorPose,
        Option<&TutorialActorCombatTransition>,
    )>,
    mut players: Query<(&mut AnimationPlayer, &TutorialActorAnimationPlayback)>,
    mut actor_commands: ResMut<TutorialActorCommandQueue>,
) {
    for (actor_root, actor, mut pose, transition) in &mut actors {
        if transition.is_some_and(|transition| pose.request_serial != transition.request_serial) {
            commands
                .entity(actor_root)
                .remove::<TutorialActorCombatTransition>();
            continue;
        }
        let restart_forced = pose
            .clip
            .is_some_and(|clip| pose.role.restarts_after_completion(clip, pose.once));
        if transition.is_none() && !restart_forced {
            continue;
        }
        let mut matched = false;
        let mut finished = true;
        let mut all_terminally_unavailable = true;
        for (player, playback) in &mut players {
            if playback.actor_root != actor_root || playback.request_serial != pose.request_serial {
                continue;
            }
            matched = true;
            all_terminally_unavailable &= playback.terminally_unavailable;
            finished &= playback.terminally_unavailable
                || playback.node.map_or_else(
                    || player.all_finished(),
                    |node| {
                        player
                            .animation(node)
                            .is_some_and(|animation| animation.is_finished())
                    },
                );
        }
        if !matched || !finished {
            continue;
        }
        // Retrobution's EndAnimation removes the completed high state before
        // AttackReady. Stop only the native delta node; the absolute fallback
        // still owns the sampled skeleton until the replacement base starts.
        for (mut player, playback) in &mut players {
            if playback.actor_root == actor_root
                && playback.request_serial == pose.request_serial
                && playback.additive
            {
                if let Some(node) = playback.node {
                    player.stop(node);
                }
            }
        }
        if restart_forced && !all_terminally_unavailable {
            pose.request_serial = pose.request_serial.wrapping_add(1);
            pose.restart_serial = pose.request_serial;
            pose.blend = LegacyAnimationBlend::Immediate;
            continue;
        }

        let Some(transition) = transition else {
            continue;
        };
        commands
            .entity(actor_root)
            .remove::<TutorialActorCombatTransition>();
        match transition.completion {
            TutorialActorCombatCompletion::Ready => {
                let request_serial = pose.request_serial.wrapping_add(1);
                *pose = TutorialActorPose {
                    clip: Some("ready"),
                    resolved_clip: Some("ready"),
                    once: false,
                    role: LegacyNpcAnimationRole::Ready,
                    blend: LegacyAnimationBlend::CrossFade200Ms,
                    state: TutorialActorPoseState::Playing,
                    force_update: false,
                    request_serial,
                    restart_serial: request_serial,
                };
            }
            TutorialActorCombatCompletion::Despawn => actor_commands.delete(actor.id),
        }
    }
}

/// Resolves exact GLTF animation names and applies pose revisions to every
/// AnimationPlayer instantiated below the corresponding tutorial actor.
pub fn apply_tutorial_actor_animation_playback(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut animation_assets: ResMut<TutorialActorAnimationAssets>,
    parents: Query<&ChildOf>,
    mut actors: Query<(
        Entity,
        &TutorialActor,
        &TutorialActorPose,
        &TutorialActorAnimationSource,
        &mut TutorialActorAnimationIssueHistory,
    )>,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&mut AnimationTransitions>,
        Option<&TutorialActorAnimationPlayback>,
    )>,
    mut issues: ResMut<TutorialActorIssueQueue>,
) {
    for (actor_root, actor, pose, source, mut issue_history) in &mut actors {
        let mut terminally_unavailable = false;
        let prepared = if matches!(
            pose.state,
            TutorialActorPoseState::Playing | TutorialActorPoseState::Paused
        ) {
            if let Some(clip_name) = pose.clip {
                let selected_clip = pose.resolved_clip.unwrap_or(clip_name);
                let fallback_clip =
                    (clip_name == "idle" && selected_clip != "stand1").then_some("stand1");
                if tutorial_actor_animation_asset_failed(&asset_server, &source.gltf) {
                    report_tutorial_actor_animation_unavailable(
                        &mut issue_history,
                        &mut issues,
                        actor.id,
                        clip_name,
                    );
                    terminally_unavailable = true;
                    None
                } else if let Some(gltf) = gltfs.get(&source.gltf) {
                    let gltf_id = source.gltf.id();
                    if !animation_assets.graphs.contains_key(&gltf_id) {
                        let prepared = prepare_tutorial_actor_animation_graph(gltf, &mut graphs);
                        animation_assets.graphs.insert(gltf_id, prepared);
                    }
                    let prepared = animation_assets
                        .graphs
                        .get(&gltf_id)
                        .expect("prepared graph was inserted");
                    if let Some((node, resolved_clip, use_additive_node)) =
                        [Some(selected_clip), fallback_clip]
                            .into_iter()
                            .flatten()
                            .find_map(|resolved_clip| {
                                let use_additive_node = pose.role.is_additive()
                                    && network_npc_animation_uses_delta_additive_layer_0104(
                                        &source.path,
                                        resolved_clip,
                                    );
                                let nodes = if use_additive_node {
                                    &prepared.additive_nodes
                                } else {
                                    &prepared.nodes
                                };
                                nodes
                                    .get(resolved_clip)
                                    .copied()
                                    .map(|node| (node, resolved_clip, use_additive_node))
                            })
                    {
                        Some((
                            prepared.graph.clone(),
                            node,
                            resolved_clip,
                            use_additive_node,
                            prepared.nodes.values().copied().collect::<Vec<_>>(),
                            prepared
                                .nodes
                                .get("ready")
                                .or_else(|| prepared.nodes.get("stand1"))
                                .copied(),
                        ))
                    } else {
                        report_tutorial_actor_animation_unavailable(
                            &mut issue_history,
                            &mut issues,
                            actor.id,
                            clip_name,
                        );
                        terminally_unavailable = true;
                        None
                    }
                } else {
                    // The parent GLTF has not reached a terminal state yet.
                    continue;
                }
            } else {
                None
            }
        } else {
            None
        };

        for (player_entity, mut player, transitions, applied) in &mut players {
            if !tutorial_actor_animation_descendant(player_entity, actor_root, &parents) {
                continue;
            }
            if applied.is_some_and(|applied| {
                applied.actor_root == actor_root
                    && applied.request_serial == pose.request_serial
                    && applied.terminally_unavailable == terminally_unavailable
            }) {
                continue;
            }

            let applied = applied.copied();
            match tutorial_actor_animation_control(
                pose,
                applied.as_ref(),
                prepared.is_some(),
                terminally_unavailable,
            ) {
                TutorialActorAnimationControl::Restart { paused } => {
                    let Some((graph, node, _, use_additive_node, base_nodes, fallback_base_node)) =
                        prepared.as_ref()
                    else {
                        continue;
                    };
                    let transition_duration = if applied.is_some() {
                        pose.blend.duration()
                    } else {
                        // `NpcAnimation.SetModel` immediately plays a forced
                        // pose already queued before the asynchronous model
                        // became available.
                        Duration::ZERO
                    };
                    if *use_additive_node {
                        ensure_tutorial_actor_additive_base_animation(
                            &mut player,
                            base_nodes,
                            *fallback_base_node,
                            paused,
                        );
                        restart_tutorial_actor_additive_animation(&mut player, *node, paused);
                    } else if let Some(mut transitions) = transitions {
                        restart_tutorial_actor_animation(
                            &mut player,
                            &mut transitions,
                            *node,
                            tutorial_actor_pose_effective_once(pose),
                            paused,
                            transition_duration,
                        );
                    } else {
                        let mut transitions = AnimationTransitions::new();
                        restart_tutorial_actor_animation(
                            &mut player,
                            &mut transitions,
                            *node,
                            tutorial_actor_pose_effective_once(pose),
                            paused,
                            Duration::ZERO,
                        );
                        commands.entity(player_entity).insert(transitions);
                    }
                    commands
                        .entity(player_entity)
                        .insert(AnimationGraphHandle(graph.clone()));
                }
                TutorialActorAnimationControl::Resume => {
                    player.resume_all();
                }
                TutorialActorAnimationControl::Pause => {
                    player.pause_all();
                }
                TutorialActorAnimationControl::Stop => {
                    player.stop_all();
                }
                TutorialActorAnimationControl::Hold => {}
            }

            commands
                .entity(player_entity)
                .insert(TutorialActorAnimationPlayback {
                    actor_root,
                    request_serial: pose.request_serial,
                    restart_serial: pose.restart_serial,
                    clip: pose.clip,
                    resolved_clip: prepared
                        .as_ref()
                        .map(|(_, _, clip, _, _, _)| *clip)
                        .or(pose.resolved_clip),
                    node: prepared.as_ref().map(|(_, node, _, _, _, _)| *node),
                    additive: prepared
                        .as_ref()
                        .is_some_and(|(_, _, _, use_additive_node, _, _)| *use_additive_node),
                    once: pose.once,
                    state: pose.state,
                    force_update: pose.force_update,
                    terminally_unavailable,
                });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TutorialActorAnimationControl {
    Restart {
        paused: bool,
    },
    Resume,
    Pause,
    Stop,
    /// Keep the last sampled skeleton pose and any valid base animation.
    /// Unity's legacy Animation component does not clear the current state
    /// when a requested clip cannot be resolved, and the tutorial `Dead`
    /// message belongs to `DeadMotion`, not to `NpcAnimation`.
    Hold,
}

pub(super) fn tutorial_actor_animation_control(
    pose: &TutorialActorPose,
    applied: Option<&TutorialActorAnimationPlayback>,
    clip_prepared: bool,
    terminally_unavailable: bool,
) -> TutorialActorAnimationControl {
    if terminally_unavailable {
        return TutorialActorAnimationControl::Hold;
    }
    let needs_restart = applied.is_none_or(|applied| {
        applied.clip != pose.clip || applied.restart_serial != pose.restart_serial
    });
    match pose.state {
        TutorialActorPoseState::Playing if clip_prepared && needs_restart => {
            TutorialActorAnimationControl::Restart { paused: false }
        }
        TutorialActorPoseState::Playing => TutorialActorAnimationControl::Resume,
        TutorialActorPoseState::Paused if clip_prepared && needs_restart => {
            TutorialActorAnimationControl::Restart { paused: true }
        }
        TutorialActorPoseState::Paused => TutorialActorAnimationControl::Pause,
        TutorialActorPoseState::ForcedStop => TutorialActorAnimationControl::Stop,
        TutorialActorPoseState::Dead => TutorialActorAnimationControl::Hold,
    }
}

#[cfg(test)]
pub(super) fn exact_tutorial_actor_named_animation(
    gltf: &Gltf,
    clip_name: &str,
) -> Option<Handle<AnimationClip>> {
    gltf.named_animations.get(clip_name).cloned()
}

pub(super) fn prepare_tutorial_actor_animation_graph(
    gltf: &Gltf,
    graphs: &mut Assets<AnimationGraph>,
) -> TutorialActorPreparedAnimationGraph {
    let mut graph = AnimationGraph::new();
    // Unity's NpcAnimation keeps one ordinary low layer alive while melee and
    // wound play on top as additive states. The normal blend must therefore
    // live *inside* the Add node; making the additive result a sibling of an
    // absolute base clip would blend the latter against identity/T-pose.
    let composition = graph.add_additive_blend(1.0, graph.root);
    let base = graph.add_blend(1.0, composition);
    let mut named_animations = gltf.named_animations.iter().collect::<Vec<_>>();
    named_animations.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    let mut nodes = HashMap::new();
    let mut additive_nodes = HashMap::new();
    for (name, clip) in named_animations {
        let node = graph.add_clip(clip.clone(), 1.0, base);
        nodes.insert(name.to_string(), node);
        if legacy_npc_clip_is_additive(name) {
            additive_nodes.insert(
                name.to_string(),
                graph.add_clip(clip.clone(), 1.0, composition),
            );
        }
    }
    TutorialActorPreparedAnimationGraph {
        graph: graphs.add(graph),
        nodes,
        additive_nodes,
    }
}

pub(super) fn ensure_tutorial_actor_additive_base_animation(
    player: &mut AnimationPlayer,
    base_nodes: &[AnimationNodeIndex],
    fallback_base_node: Option<AnimationNodeIndex>,
    paused: bool,
) {
    if base_nodes
        .iter()
        .any(|node| player.animation(*node).is_some())
    {
        return;
    }
    let Some(fallback_base_node) = fallback_base_node else {
        return;
    };
    let base = player
        .start(fallback_base_node)
        .set_repeat(RepeatAnimation::Forever);
    if paused {
        base.pause();
    } else {
        base.resume();
    }
}

#[must_use]
pub(super) fn legacy_npc_clip_is_additive(name: &str) -> bool {
    !name.contains("upper")
        && !name.contains("event")
        && (name.contains("melee") || name.contains("wound"))
}

#[cfg(test)]
pub(super) fn exact_tutorial_actor_pose_animation(
    gltf: &Gltf,
    pose: &TutorialActorPose,
) -> Option<(Handle<AnimationClip>, &'static str)> {
    let requested_clip = pose.clip?;
    let resolved_clip = pose.resolved_clip.unwrap_or(requested_clip);
    if let Some(clip) = exact_tutorial_actor_named_animation(gltf, resolved_clip) {
        return Some((clip, resolved_clip));
    }
    (requested_clip == "idle" && resolved_clip != "stand1")
        .then(|| exact_tutorial_actor_named_animation(gltf, "stand1").map(|clip| (clip, "stand1")))
        .flatten()
}

pub(super) fn tutorial_actor_animation_asset_failed(asset_server: &AssetServer, gltf: &Handle<Gltf>) -> bool {
    matches!(asset_server.load_state(gltf.id()), LoadState::Failed(_))
        || matches!(
            asset_server.get_recursive_dependency_load_state(gltf.id()),
            Some(RecursiveDependencyLoadState::Failed(_))
        )
}

pub(super) fn report_tutorial_actor_animation_unavailable(
    history: &mut TutorialActorAnimationIssueHistory,
    issues: &mut TutorialActorIssueQueue,
    id: i32,
    clip: &'static str,
) {
    if history.reported_clips.insert(clip) {
        issues.push(TutorialActorIssue::RigAnimationUnavailable { id, clip });
    }
}

pub(super) fn restart_tutorial_actor_animation(
    player: &mut AnimationPlayer,
    transitions: &mut AnimationTransitions,
    node: AnimationNodeIndex,
    once: bool,
    paused: bool,
    transition_duration: Duration,
) {
    transitions
        .play(player, node, transition_duration)
        .set_repeat(if once {
            RepeatAnimation::Never
        } else {
            RepeatAnimation::Forever
        })
        .resume();
    if paused {
        player.pause_all();
    }
}

pub(super) fn restart_tutorial_actor_additive_animation(
    player: &mut AnimationPlayer,
    node: AnimationNodeIndex,
    paused: bool,
) {
    player
        .start(node)
        .set_repeat(RepeatAnimation::Never)
        .set_weight(1.0)
        .resume();
    if paused {
        player.pause_all();
    }
}

pub(super) fn tutorial_actor_animation_descendant(
    mut entity: Entity,
    ancestor: Entity,
    parents: &Query<&ChildOf>,
) -> bool {
    while let Ok(parent) = parents.get(entity) {
        entity = parent.parent();
        if entity == ancestor {
            return true;
        }
    }
    false
}
