use super::*;

pub(super) const LEGACY_HEIGHT_ADDITIVE_CLIP: &str = "height_Add";

pub(super) const LEGACY_SHAPE_ADDITIVE_CLIP: &str = "shape_Add";

pub(super) const LEGACY_HEIGHT_SCALE_CLIP: &str = "height";

pub(super) const LEGACY_SHAPE_SCALE_CLIP: &str = "shape";

pub(super) const LEGACY_TURN_LEFT_CLIP: &str = "turnleft";

pub(super) const LEGACY_TURN_RIGHT_CLIP: &str = "turnright";

pub(super) const LEGACY_RIFLE_TURN_LEFT_CLIP: &str = "rifleturnleft";

pub(super) const LEGACY_RIFLE_TURN_RIGHT_CLIP: &str = "rifleturnright";

pub(super) const SKYWAY_MODEL_ANIMATION: &str = "nif-default";

pub(super) fn motion_without_body_scales(
    source: &AnimationClip,
    body_scale_targets: &BTreeSet<AnimationTargetId>,
) -> Option<AnimationClip> {
    use bevy::animation::{prelude::AnimatableProperty, animated_field};
    let scale = animated_field!(Transform::scale);
    let owns_scale = |curve: &bevy::animation::VariableCurve| {
        curve.0.evaluator_id() == scale.evaluator_id()
    };
    if !source.curves().iter().any(|(target, curves)| {
        body_scale_targets.contains(target) && curves.iter().any(owns_scale)
    }) {
        return None;
    }
    let mut projected = source.clone();
    projected.curves_mut().retain(|target, curves| {
        if body_scale_targets.contains(target) {
            curves.retain(|curve| !owns_scale(curve));
        }
        !curves.is_empty()
    });
    Some(projected)
}

#[derive(Component)]
pub struct TutorialSelectedPlayerRig {
    pub controller_root: Entity,
    pub gender: PlayerRigGender,
    pub(super) look: NativePlayerLook,
    pub(super) consumer: TutorialPlayerPresentationConsumer,
    pub(super) clip_indices: BTreeMap<TutorialPlayerClip, u32>,
    pub(super) skeleton_gltf: Handle<Gltf>,
    pub(super) weapon_models: BTreeMap<i16, NativePlayerPartLook>,
    pub(super) attached_weapon: Option<Entity>,
    pub(super) attached_weapon_item_id: Option<i16>,
    pub(super) render_layers: RenderLayers,
    pub(super) tutorial_stand_semantics: bool,
    pub(super) initial_weapon_profile: Option<PlayerWeaponAnimationProfile>,
}

/// The one selected-player rig allowed to consume gameplay presentation.
///
/// Apparel refreshes build a second hidden rig. Keeping this role explicit
/// prevents animation, weapon, audio, and startup consumers from selecting a
/// half-loaded candidate by query iteration order.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TutorialSelectedPlayerRigActive;

/// A hidden apparel replacement which keeps `fallback` visible until every
/// shared-rig, material, texture, and animation readiness gate succeeds.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TutorialSelectedPlayerRigCandidate {
    pub fallback: Entity,
}

#[derive(Component)]
pub(super) struct PendingTutorialSkywayAnimation;

impl TutorialSelectedPlayerRig {
    #[must_use]
    pub const fn attached_weapon(&self) -> Option<Entity> {
        self.attached_weapon
    }

    #[must_use]
    pub const fn attached_weapon_item_id(&self) -> Option<i16> {
        self.attached_weapon_item_id
    }

    #[must_use]
    pub fn render_layers(&self) -> RenderLayers {
        self.render_layers.clone()
    }

    #[must_use]
    pub const fn tutorial_stand_semantics(&self) -> bool {
        self.tutorial_stand_semantics
    }
}

impl TutorialSelectedPlayerRig {
    #[must_use]
    pub const fn look(&self) -> &NativePlayerLook {
        &self.look
    }
}

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum TutorialSelectedPlayerRigStatus {
    Loading,
    Ready,
    Blocked(String),
}

#[derive(Component)]
pub(super) struct TutorialPlayerAnimationAdapter {
    pub(super) nodes: BTreeMap<TutorialPlayerClip, AnimationNodeIndex>,
    pub(super) upper_masked_nodes: BTreeMap<TutorialPlayerClip, AnimationNodeIndex>,
    pub(super) body_shape: TutorialBodyShapePlayback,
    pub(super) directional_turn: TutorialDirectionalTurnPlayback,
    pub(super) animation_player: Entity,
    pub(super) active_node: AnimationNodeIndex,
    pub(super) active_clip: TutorialPlayerClip,
    pub(super) upper_layer: Option<TutorialUpperLayerPlayback>,
    pub(super) damage: damage::PlayerDamagePlayback,
    pub(super) delay: Option<TutorialAnimationDelayPlayback>,
    pub(super) emote_state: bool,
    pub(super) emote_cursor: PlayerEmoteCursor,
    pub(super) hand_attachment_hidden: bool,
    pub(super) pending_base_completion: Option<PendingLegacyVisualCompletion>,
    pub(super) pending_upper_completion: Option<PendingLegacyVisualCompletion>,
}

/// `DelayCurrent` hit-stop. Expiry resumes the adapter's *current* base and
/// upper nodes: a composed-mask switch during the stop hands the paused state
/// to another graph node, and resuming only the originally captured node
/// left that replacement paused, so the attack clamp could never complete.
#[derive(Clone, Copy, Debug)]
pub(super) struct TutorialAnimationDelayPlayback {
    pub(super) remaining_seconds: f32,
    pub(super) base: bool,
    pub(super) upper: bool,
}

#[derive(Component, Clone, Debug)]
pub struct TutorialPlayerAnimationApplied {
    pub rig_root: Entity,
    pub animation_player: Entity,
    pub clip: TutorialPlayerClip,
    pub source_path_id: i64,
    pub animation_node: AnimationNodeIndex,
    pub playback: TutorialPlayerClipPlayback,
    pub dispatch: TutorialPlayerAnimationDispatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TutorialPlayerRigIssue {
    AdapterBlocked {
        rig_root: Entity,
        reason: String,
    },
    AnimationBlocked {
        rig_root: Entity,
        clip: TutorialPlayerClip,
        reason: String,
    },
    EquipmentBlocked {
        rig_root: Entity,
        item_id: i16,
        reason: String,
    },
    AppearanceBlocked {
        rig_root: Entity,
        reason: String,
    },
}

#[derive(Resource, Default)]
pub struct TutorialPlayerRigIssueQueue {
    pub(super) pending: VecDeque<TutorialPlayerRigIssue>,
}

impl TutorialPlayerRigIssueQueue {
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn take_all(&mut self) -> VecDeque<TutorialPlayerRigIssue> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnedTutorialSelectedPlayerRig {
    pub rig_root: Entity,
    pub skeleton_scene: Entity,
}

pub struct TutorialPlayerRigRuntimePlugin;

impl Plugin for TutorialPlayerRigRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialPlayerRigIssueQueue>()
            .init_resource::<TutorialPlayerPresentationCommandQueue>()
            .init_resource::<PlayerWeaponAnimationCatalog>()
            .init_resource::<TutorialSkywayPresentation>()
            .init_resource::<PersonalVehiclePresentation>()
            .init_resource::<PlayerEmoteContinuation>()
            .add_systems(
                Update,
                (
                    prepare_tutorial_player_animation_adapter,
                    bind_tutorial_player_appearance_attachments,
                    bind_tutorial_player_appearance,
                    finalize_tutorial_player_rig_readiness,
                    swap_tutorial_player_fallback_on_ready,
                    bridge_legacy_avatar_visual_requests,
                    advance_tutorial_player_animation_state.in_set(PlayerEmoteAdvance),
                    consume_tutorial_player_presentation,
                    bind_tutorial_player_weapon_materials,
                    personal_vehicle::bind_vehicle_materials,
                    personal_vehicle::play_vehicle_model_animation,
                    personal_vehicle::vehicle_animation_audio,
                    personal_vehicle::vehicle_engine_audio,
                    sync_tutorial_zipline_attachment,
                    play_tutorial_skyway_animation,
                    sync_tutorial_player_weapon_visibility,
                )
                    .chain()
                    .after(LegacyMovementSet::Simulate)
                    .after(LegacyAvatarActionSet::Locomotion)
                    .after(LegacyEnvironmentSet::Update),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_tutorial_skyway_attachment,
                    personal_vehicle::sync_vehicle_attachment,
                    personal_vehicle::sync_remote_vehicle_attachments,
                )
                    .after(bevy::transform::TransformSystems::Propagate),
            );
    }
}

pub(super) fn play_tutorial_skyway_animation(
    mut commands: Commands,
    gltfs: Option<Res<Assets<Gltf>>>,
    graphs: Option<ResMut<Assets<AnimationGraph>>>,
    parents: Query<&ChildOf>,
    attachments: Query<&TutorialSkywayAttachment>,
    mut players: Query<
        (
            Entity,
            &mut AnimationPlayer,
            Option<&PendingTutorialSkywayAnimation>,
        ),
        Or<(Added<AnimationPlayer>, With<PendingTutorialSkywayAnimation>)>,
    >,
) {
    let mut graphs = graphs;
    for (entity, mut player, pending) in &mut players {
        let Some(attachment) = owning_skyway_attachment(entity, &parents, &attachments) else {
            continue;
        };
        if pending.is_none() {
            commands
                .entity(entity)
                .insert(PendingTutorialSkywayAnimation);
        }
        let (Some(gltfs), Some(graphs)) = (gltfs.as_deref(), graphs.as_deref_mut()) else {
            continue;
        };
        let Some(gltf) = gltfs.get(&attachment.gltf) else {
            continue;
        };
        let Some(clip) = gltf.named_animations.get(SKYWAY_MODEL_ANIMATION).cloned() else {
            commands
                .entity(entity)
                .remove::<PendingTutorialSkywayAnimation>();
            continue;
        };
        let (graph, node) = AnimationGraph::from_clip(clip);
        player.stop_all();
        player.start(node).set_repeat(RepeatAnimation::Forever);
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(graphs.add(graph)))
            .remove::<PendingTutorialSkywayAnimation>();
    }
}

pub(super) fn prepare_tutorial_player_animation_adapter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    gltfs: Res<Assets<Gltf>>,
    mut issues: ResMut<TutorialPlayerRigIssueQueue>,
    mut rigs: Query<
        (
            Entity,
            &TutorialSelectedPlayerRig,
            &NativePlayerRigStatus,
            &NativePlayerRigBones,
            &NativePlayerRigStand1Playback,
            &mut TutorialSelectedPlayerRigStatus,
        ),
        Without<TutorialPlayerAnimationAdapter>,
    >,
    animation_targets: Query<(&AnimationTargetId, &AnimatedBy)>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (rig_root, selected, native_status, bones, stand1, mut status) in &mut rigs {
        if matches!(*status, TutorialSelectedPlayerRigStatus::Blocked(_)) {
            continue;
        }
        if let NativePlayerRigStatus::Blocked(reason) = native_status {
            let reason = format!("native selected-player rig blocked: {reason}");
            *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
            issues
                .pending
                .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            continue;
        }
        if !native_status.is_ready() {
            continue;
        }
        let Some(gltf) = gltfs.get(&selected.skeleton_gltf) else {
            if let LoadState::Failed(error) = asset_server.load_state(selected.skeleton_gltf.id()) {
                let reason = format!("selected-player skeleton GLB failed to load: {error}");
                *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
                issues
                    .pending
                    .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            }
            continue;
        };
        let mut clips = Vec::with_capacity(TutorialPlayerClip::ALL.len());
        let mut ordered = Vec::with_capacity(TutorialPlayerClip::ALL.len());
        let mut block = None;
        for clip in TutorialPlayerClip::ALL {
            let Some(index) = selected.clip_indices.get(&clip).copied() else {
                block = Some(format!("missing contract index for {:?}", clip.name()));
                break;
            };
            let indexed = gltf.animations.get(index as usize);
            let named = gltf.named_animations.get(clip.name());
            let (Some(indexed), Some(named)) = (indexed, named) else {
                block = Some(format!(
                    "shared GLB has no exact {:?} animation at index {index}",
                    clip.name()
                ));
                break;
            };
            if indexed != named {
                block = Some(format!(
                    "shared GLB named {:?} does not match contract index {index}",
                    clip.name()
                ));
                break;
            }
            clips.push(named.clone());
            ordered.push(clip);
        }
        if let Some(reason) = block {
            *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
            issues
                .pending
                .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            continue;
        }
        if clips.iter().any(|handle| animation_clips.get(handle).is_none()) {
            // GLTF handles can be published before their clip sub-assets.
            continue;
        }
        let Some(height_additive) = gltf
            .named_animations
            .get(LEGACY_HEIGHT_ADDITIVE_CLIP)
            .cloned()
        else {
            let reason =
                format!("shared GLB has no exact {LEGACY_HEIGHT_ADDITIVE_CLIP:?} animation");
            *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
            issues
                .pending
                .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            continue;
        };
        let Some(shape_additive) = gltf
            .named_animations
            .get(LEGACY_SHAPE_ADDITIVE_CLIP)
            .cloned()
        else {
            let reason =
                format!("shared GLB has no exact {LEGACY_SHAPE_ADDITIVE_CLIP:?} animation");
            *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
            issues
                .pending
                .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            continue;
        };
        let Some(height_scale) = gltf.named_animations.get(LEGACY_HEIGHT_SCALE_CLIP).cloned()
        else {
            let reason = format!("shared GLB has no exact {LEGACY_HEIGHT_SCALE_CLIP:?} animation");
            *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
            issues
                .pending
                .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            continue;
        };
        let Some(shape_scale) = gltf.named_animations.get(LEGACY_SHAPE_SCALE_CLIP).cloned() else {
            let reason = format!("shared GLB has no exact {LEGACY_SHAPE_SCALE_CLIP:?} animation");
            *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
            issues
                .pending
                .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            continue;
        };
        let directional_handles = [
            LEGACY_TURN_LEFT_CLIP,
            LEGACY_TURN_RIGHT_CLIP,
            LEGACY_RIFLE_TURN_LEFT_CLIP,
            LEGACY_RIFLE_TURN_RIGHT_CLIP,
            "scooter_turnleft",
            "scooter_turnright",
        ]
        .map(|name| gltf.named_animations.get(name).cloned());
        let [
            Some(turn_left),
            Some(turn_right),
            Some(rifle_turn_left),
            Some(rifle_turn_right),
            Some(scooter_turn_left),
            Some(scooter_turn_right),
        ] = directional_handles
        else {
            let reason = "shared GLB lacks the exact turnleft/turnright directional additive layer"
                .to_owned();
            *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
            issues
                .pending
                .push_back(TutorialPlayerRigIssue::AdapterBlocked { rig_root, reason });
            continue;
        };
        let (
            Some(height_additive_clip),
            Some(shape_additive_clip),
            Some(height_scale_clip),
            Some(shape_scale_clip),
        ) = (
            animation_clips.get(&height_additive),
            animation_clips.get(&shape_additive),
            animation_clips.get(&height_scale),
            animation_clips.get(&shape_scale),
        )
        else {
            // The GLTF can become available one frame before all of its
            // animation sub-assets. Keep the rig loading instead of
            // permanently rejecting an otherwise valid contract.
            continue;
        };
        let directional_clips = [
            animation_clips.get(&turn_left),
            animation_clips.get(&turn_right),
            animation_clips.get(&rifle_turn_left),
            animation_clips.get(&rifle_turn_right),
            animation_clips.get(&scooter_turn_left),
            animation_clips.get(&scooter_turn_right),
        ];
        let [
            Some(turn_left_clip),
            Some(turn_right_clip),
            Some(rifle_turn_left_clip),
            Some(rifle_turn_right_clip),
            Some(scooter_turn_left_clip),
            Some(scooter_turn_right_clip),
        ] = directional_clips
        else {
            continue;
        };
        let height_duration = height_additive_clip.duration();
        let shape_duration = shape_additive_clip.duration();
        let height_scale_duration = height_scale_clip.duration();
        let shape_scale_duration = shape_scale_clip.duration();
        let body_scale_targets = height_scale_clip
            .curves()
            .keys()
            .chain(shape_scale_clip.curves().keys())
            .copied()
            .collect::<BTreeSet<_>>();
        let directional_durations = [
            turn_left_clip.duration(),
            turn_right_clip.duration(),
            rifle_turn_left_clip.duration(),
            rifle_turn_right_clip.duration(),
            scooter_turn_left_clip.duration(),
            scooter_turn_right_clip.duration(),
        ];
        let Some(unarmed_upper_attack) = gltf
            .named_animations
            .get(TutorialPlayerClip::Attack1Upper.name())
            .cloned()
        else {
            continue;
        };
        let Some(unarmed_full_attack) = gltf
            .named_animations
            .get(TutorialPlayerClip::Attack1.name())
            .cloned()
        else {
            continue;
        };
        let (Some(upper_clip), Some(full_attack_clip)) = (
            animation_clips.get(&unarmed_upper_attack),
            animation_clips.get(&unarmed_full_attack),
        ) else {
            continue;
        };
        let attack_body_chain = ["Bip01", "Bip01 NonAccum", "Bip01 Pelvis"]
            .into_iter()
            .map(|true_name| {
                bones
                    .unique_by_true_name(true_name)
                    .and_then(|entity| animation_targets.get(entity).ok())
                    .filter(|(_, owner)| owner.0 == stand1.animation_player)
                    .map(|(id, _)| *id)
            })
            .collect::<Option<BTreeSet<_>>>();
        let Some(attack_body_chain) = attack_body_chain else {
            // Native readiness normally guarantees these targets. If the GLTF
            // scene became visible one frame before AnimationTarget insertion,
            // keep loading instead of publishing a permanently incomplete mask.
            continue;
        };

        // ActorSkinCombiner first samples ordinary height/shape scale clips at
        // layer 100, then leaves height_Add/shape_Add continuously enabled in
        // additive mode. The published `_Add` clips have already been rebased
        // from Unity's absolute local TR curves to bind-pose deltas.
        let upper_targets = upper_clip.curves().keys().copied().collect::<BTreeSet<_>>();
        let masked_attack_targets = full_attack_clip
            .curves()
            .keys()
            .filter(|target| {
                tutorial_unarmed_attack_masks_target(target, &upper_targets, &attack_body_chain)
            })
            .copied()
            .collect::<Vec<_>>();
        // Growth/body scales own these properties at the legacy layer 100.
        // Bevy's Add sums Vec3 scales: an absolute unit scale in a dance
        // would add another 1 at every joint and exponentially enlarge the
        // skinned body. Project motion onto the properties it actually owns.
        // Keep the GLTF sub-assets intact for other consumers.
        let clips = clips.into_iter().map(|handle| {
            let source = animation_clips.get(&handle).expect("loaded motion clip");
            if let Some(projected) = motion_without_body_scales(source, &body_scale_targets) {
                animation_clips.add(projected)
            } else {
                handle
            }
        }).collect::<Vec<_>>();
        let mut graph = AnimationGraph::new();
        for target in upper_targets {
            graph.add_target_to_mask_group(target, 0);
        }
        for target in masked_attack_targets {
            graph.add_target_to_mask_group(target, 1);
        }
        // The root remains a normal blend so `rifleattack1upper` can cross-fade
        // against the already composed body pose. Locomotion, static scales,
        // and bind-relative body deltas must first be combined inside one Add
        // node; making the body Add a sibling of locomotion would blend the
        // absolute pose with identity and fold 180-degree biped joints toward
        // 90 degrees.
        let body_composition = graph.add_additive_blend(1.0, graph.root);
        let upper_masked_body_composition =
            graph.add_additive_blend_with_mask(UPPER_BODY_MASK, 1.0, graph.root);
        let locomotion_layer = graph.add_blend(1.0, body_composition);
        let masked_locomotion_layer = graph.add_blend(1.0, upper_masked_body_composition);
        // The actor-owned `attack1upper` is a different swing from `attack1`.
        // The requested unarmed running behavior must retain the stationary
        // punch, so project the exact full-body attack onto its body chain and
        // upper targets. Keeping Bip01/NonAccum/Pelvis is essential: masking
        // the pelvis left run/jump rotations beneath the arms and erased the
        // wind-up. Locomotion still owns the leg branches and world movement.
        let mut nodes = BTreeMap::new();
        let mut upper_masked_nodes = BTreeMap::new();
        let mut damage_clip = None;
        for (clip, handle) in ordered.into_iter().zip(clips) {
            if clip == TutorialPlayerClip::WoundUpper {
                damage_clip = Some(handle);
                continue;
            }
            if clip.is_upper_layer() {
                let (handle, mask) =
                    if tutorial_upper_pose_source(clip) == TutorialPlayerClip::Attack1 {
                        (unarmed_full_attack.clone(), UNARMED_ATTACK_LOWER_BODY_MASK)
                    } else {
                        (handle, 0)
                    };
                let node = add_tutorial_upper_clip(&mut graph, handle, mask);
                nodes.insert(clip, node);
                continue;
            }
            nodes.insert(clip, graph.add_clip(handle.clone(), 1.0, locomotion_layer));
            upper_masked_nodes.insert(clip, graph.add_clip(handle, 1.0, masked_locomotion_layer));
        }
        let height_additive_node = graph.add_clip(height_additive.clone(), 1.0, body_composition);
        let shape_additive_node = graph.add_clip(shape_additive.clone(), 1.0, body_composition);
        let masked_height_additive_node =
            graph.add_clip(height_additive, 1.0, upper_masked_body_composition);
        let masked_shape_additive_node =
            graph.add_clip(shape_additive, 1.0, upper_masked_body_composition);
        let body_scale_layer = graph.add_blend(1.0, body_composition);
        let masked_body_scale_layer = graph.add_blend(1.0, upper_masked_body_composition);
        let height_scale_node = graph.add_clip(height_scale.clone(), 1.0, body_scale_layer);
        let shape_scale_node = graph.add_clip(shape_scale.clone(), 1.0, body_scale_layer);
        let masked_height_scale_node = graph.add_clip(height_scale, 1.0, masked_body_scale_layer);
        let masked_shape_scale_node = graph.add_clip(shape_scale, 1.0, masked_body_scale_layer);
        let directional_handles = [
            turn_left,
            turn_right,
            rifle_turn_left,
            rifle_turn_right,
            scooter_turn_left,
            scooter_turn_right,
        ];
        let directional_nodes = directional_handles
            .clone()
            .map(|handle| graph.add_clip(handle, 1.0, body_composition));
        let masked_directional_nodes = directional_handles
            .map(|handle| graph.add_clip(handle, 1.0, upper_masked_body_composition));
        let damage = damage::PlayerDamagePlayback::attach(
            &mut graph,
            damage_clip.expect("validated wound clip"),
        );
        nodes.insert(TutorialPlayerClip::WoundUpper, damage.motion);
        let initial_clip = if selected.tutorial_stand_semantics {
            TutorialPlayerClip::Stand1
        } else {
            selected
                .initial_weapon_profile
                .map_or(TutorialPlayerClip::Stand1, |profile| profile.stand())
        };
        let Some(stand_node) = nodes.get(&initial_clip).copied() else {
            continue;
        };
        let Ok(mut player) = players.get_mut(stand1.animation_player) else {
            continue;
        };
        let graph = graphs.add(graph);
        player.stop_all();
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, stand_node, Duration::ZERO)
            .set_repeat(RepeatAnimation::Forever)
            .resume();
        let (height_time, shape_time) = legacy_body_shape_normalized_times(
            selected.look.height_selector,
            selected.look.body_selector,
        );
        player
            .start(height_additive_node)
            .set_seek_time(height_duration * height_time)
            .set_weight(1.0)
            .pause();
        player
            .start(shape_additive_node)
            .set_seek_time(shape_duration * shape_time)
            .set_weight(1.0)
            .pause();
        player
            .start(height_scale_node)
            .set_seek_time(height_scale_duration * height_time)
            .set_weight(1.0)
            .pause();
        player
            .start(shape_scale_node)
            .set_seek_time(shape_scale_duration * shape_time)
            .set_weight(1.0)
            .pause();
        for node in directional_nodes {
            player
                .start(node)
                .set_seek_time(0.0)
                .set_weight(1.0)
                .pause();
        }
        commands
            .entity(stand1.animation_player)
            .insert((AnimationGraphHandle(graph.clone()), transitions));
        commands
            .entity(rig_root)
            .insert(TutorialPlayerAnimationAdapter {
                nodes,
                upper_masked_nodes,
                body_shape: TutorialBodyShapePlayback {
                    normal_nodes: [
                        height_additive_node,
                        shape_additive_node,
                        height_scale_node,
                        shape_scale_node,
                        directional_nodes[0],
                        directional_nodes[1],
                        directional_nodes[2],
                        directional_nodes[3],
                        directional_nodes[4],
                        directional_nodes[5],
                    ],
                    upper_masked_nodes: [
                        masked_height_additive_node,
                        masked_shape_additive_node,
                        masked_height_scale_node,
                        masked_shape_scale_node,
                        masked_directional_nodes[0],
                        masked_directional_nodes[1],
                        masked_directional_nodes[2],
                        masked_directional_nodes[3],
                        masked_directional_nodes[4],
                        masked_directional_nodes[5],
                    ],
                    upper_masked: false,
                },
                directional_turn: TutorialDirectionalTurnPlayback {
                    normal_nodes: directional_nodes,
                    upper_masked_nodes: masked_directional_nodes,
                    durations: directional_durations,
                    current_left: 0.0,
                    current_right: 0.0,
                },
                animation_player: stand1.animation_player,
                active_node: stand_node,
                active_clip: initial_clip,
                upper_layer: None,
                damage,
                delay: None,
                emote_state: false,
                emote_cursor: PlayerEmoteCursor::default(),
                hand_attachment_hidden: false,
                pending_base_completion: None,
                pending_upper_completion: None,
            });
    }
}

pub(super) fn switch_tutorial_composed_pose_mask(
    adapter: &mut TutorialPlayerAnimationAdapter,
    player: &mut AnimationPlayer,
    transitions: &mut AnimationTransitions,
    masked: bool,
) -> bool {
    // Repair both halves independently. In particular, a removed clamp base
    // node must not prevent the continuously sampled body-shape/turn layers
    // from being unmasked after an upper animation, which otherwise strands
    // part of the skeleton in a malformed pose.
    let base_switched = switch_tutorial_base_layer_mask(adapter, player, transitions, masked);
    let body_switched = switch_tutorial_body_shape_mask(adapter, player, masked);
    base_switched && body_switched
}

/// Primary CharacterSelection.resourceFile AnimationClip `end` events. Clean
/// `cnAvatarAnimation.EndAnimation` consumes these events instead of waiting
/// for the final authored recovery key or Unity/Bevy playback completion.
pub(super) const fn tutorial_player_clip_end_event_seconds(
    gender: PlayerRigGender,
    clip: TutorialPlayerClip,
) -> Option<f32> {
    if clip.is_vehicle() {
        return personal_vehicle::end_event(gender, clip);
    }
    if let Some(events) = emote_events(gender, clip) {
        return Some(events.end);
    }
    match (gender, clip) {
        (_, TutorialPlayerClip::Attack1 | TutorialPlayerClip::Attack1Upper) => Some(0.75),
        // Both primary 2026-08-21 CharacterSelection standup clips (male
        // PathID 34404 and female PathID 34533) carry `end` at this time.
        (_, TutorialPlayerClip::Standup) => Some(2.083_333_5),
        // Primary actor/m.kfm landing clips all fire end on frame 5.5.
        (
            PlayerRigGender::Male,
            TutorialPlayerClip::JumpEnd
            | TutorialPlayerClip::StickJumpEnd
            | TutorialPlayerClip::PistolJumpEnd
            | TutorialPlayerClip::RifleJumpEnd
            | TutorialPlayerClip::BombJumpEnd
            | TutorialPlayerClip::RocketJumpEnd,
        ) => Some(0.183_333_34),
        // Primary actor/w.kfm uses frame 6.5 for these landing families.
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::JumpEnd
            | TutorialPlayerClip::StickJumpEnd
            | TutorialPlayerClip::PistolJumpEnd
            | TutorialPlayerClip::RifleJumpEnd
            | TutorialPlayerClip::BombJumpEnd,
        ) => Some(0.216_666_67),
        // Female rocket landing and every running landing fire on frame 5.5.
        (_, TutorialPlayerClip::RocketJumpEnd)
        | (
            _,
            TutorialPlayerClip::JumpLandRun
            | TutorialPlayerClip::StickJumpLandRun
            | TutorialPlayerClip::PistolJumpLandRun
            | TutorialPlayerClip::RifleJumpLandRun
            | TutorialPlayerClip::BombJumpLandRun
            | TutorialPlayerClip::RocketJumpLandRun,
        ) => Some(0.183_333_34),
        _ => None,
    }
}
