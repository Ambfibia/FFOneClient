//! Exact temporary Nano presentation used by the Retrobution tutorial.
//!
//! `cntutorialscript` creates a `newNano` container up front, asks
//! `EventNanoController` for Nano id 1, and later drives that object's live
//! transform and `EventNanoAnimation`.  This module keeps the same split: the
//! presentation root owns the gameplay transform, while the exact exported
//! `nano/nano_buttercup.kfm` Scene0 is its child.

use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use bevy::{
    animation::RepeatAnimation,
    asset::{LoadState, RecursiveDependencyLoadState},
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
};

use crate::{
    assets::AssetLocator,
    character_scene::{NativeSceneRole, native_scene_container_transform},
    coordinates::unity_to_native_vector,
    gameplay_audio::{GameplayAudioRuntime, GameplayAudioSet},
    legacy_model_material::{
        LegacyModelMaterial, PendingLegacyModelMaterial, load_legacy_main_texture_replacement,
    },
    legacy_npc_nano_animation::{
        LegacyAnimationBlend, LegacyNanoAnimationMachine, LegacyNanoAnimationMode,
        LegacyNanoCompletion, LegacyNanoStandRandomStream, event_nano_blend_for_action,
        event_nano_mode_for_action,
    },
    network_world_runtime::{
        NetworkNpcAnimationSoundEvent0104, parse_network_npc_animation_sound_events,
    },
    tutorial_effects_runtime::animation_event_crossings,
};

pub const TUTORIAL_NANO_LEGACY_ROUTE: &str = "nano/nano_buttercup.kfm";
pub const TUTORIAL_NANO_MODEL_PATH: &str = "characters/nanos/nano_buttercup/nano_buttercup.glb";
pub const TUTORIAL_NANO_FACE_TEXTURE_PATH: &str =
    "characters/nanos/nano_buttercup/runtime-textures/nano_buttercup_face.png";
pub const TUTORIAL_NANO_FACE_MATERIAL_NAME: &str = "nano_buttercup-sub-link_b.dds";

/// Exact clips needed by `Infection_Event_A`.  The semantic `stand` action is
/// the legacy `SetStandMotion` call and therefore resolves to `stand1..3`.
pub const TUTORIAL_NANO_REQUIRED_ANIMATION_CLIPS: &[&str] = &[
    "call2", "happy", "flex", "call", "hello", "stand1", "stand2", "stand3", "dance2", "dance5",
    "shocked",
];

pub const TUTORIAL_NANO_CHOREOGRAPHY_ANIMATIONS: &[&str] = &[
    "call2", "happy", "flex", "call", "hello", "stand", "dance2", "dance5", "shocked",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialNanoPresentationSpawn {
    pub transform: Transform,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TutorialNanoPresentationAnimation {
    LegacyNamed(String),
    Emote(String),
    Call,
    Happy,
    Stand,
}

impl TutorialNanoPresentationAnimation {
    fn clip_name(&self) -> &str {
        match self {
            Self::LegacyNamed(clip) | Self::Emote(clip) => clip,
            Self::Call => "call",
            Self::Happy => "happy",
            Self::Stand => "stand",
        }
    }

    fn apply(
        self,
        animation: &mut LegacyNanoAnimationMachine,
        stand_random: &mut LegacyNanoStandRandomStream,
    ) {
        match self {
            Self::Stand => animation.request_stand(stand_random),
            Self::Call => animation.request(
                LegacyNanoAnimationMode::Call,
                "call",
                LegacyAnimationBlend::CrossFade100Ms,
            ),
            Self::Happy => animation.request(
                LegacyNanoAnimationMode::Happy,
                "happy",
                LegacyAnimationBlend::CrossFade100Ms,
            ),
            Self::Emote(clip) => animation.request(
                LegacyNanoAnimationMode::Emote,
                clip,
                LegacyAnimationBlend::CrossFade300Ms,
            ),
            Self::LegacyNamed(clip) => animation.request(
                event_nano_mode_for_action(&clip),
                clip.clone(),
                event_nano_blend_for_action(&clip),
            ),
        }
    }
}

impl TutorialNanoPresentationSpawn {
    #[must_use]
    pub fn legacy_hidden() -> Self {
        Self {
            transform: Transform::from_translation(unity_to_native_vector(Vec3::new(
                -1_000.0, 0.0, -3_000.0,
            ))),
        }
    }

    #[must_use]
    pub const fn at(transform: Transform) -> Self {
        Self { transform }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TutorialNanoPresentationCommand {
    Spawn(TutorialNanoPresentationSpawn),
    SetTransform(Transform),
    SetTranslation(Vec3),
    SetRotation(Quat),
    PlayAnimation(TutorialNanoPresentationAnimation),
    SetVoiceDisabled(bool),
    Destroy,
}

#[derive(Debug, Default, Resource)]
pub struct TutorialNanoPresentationCommandQueue {
    pending: VecDeque<TutorialNanoPresentationCommand>,
}

impl TutorialNanoPresentationCommandQueue {
    pub fn push(&mut self, command: TutorialNanoPresentationCommand) {
        self.pending.push_back(command);
    }

    pub fn spawn(&mut self, spawn: TutorialNanoPresentationSpawn) {
        self.push(TutorialNanoPresentationCommand::Spawn(spawn));
    }

    pub fn spawn_legacy_hidden(&mut self) {
        self.spawn(TutorialNanoPresentationSpawn::legacy_hidden());
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.push(TutorialNanoPresentationCommand::SetTransform(transform));
    }

    pub fn set_translation(&mut self, translation: Vec3) {
        self.push(TutorialNanoPresentationCommand::SetTranslation(translation));
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.push(TutorialNanoPresentationCommand::SetRotation(rotation));
    }

    pub fn play_animation(&mut self, animation: impl Into<String>) {
        self.push(TutorialNanoPresentationCommand::PlayAnimation(
            TutorialNanoPresentationAnimation::LegacyNamed(animation.into()),
        ));
    }

    pub fn play_emote(&mut self, animation: impl Into<String>) {
        self.push(TutorialNanoPresentationCommand::PlayAnimation(
            TutorialNanoPresentationAnimation::Emote(animation.into()),
        ));
    }

    pub fn call(&mut self) {
        self.push(TutorialNanoPresentationCommand::PlayAnimation(
            TutorialNanoPresentationAnimation::Call,
        ));
    }

    pub fn happy(&mut self) {
        self.push(TutorialNanoPresentationCommand::PlayAnimation(
            TutorialNanoPresentationAnimation::Happy,
        ));
    }

    pub fn set_stand_motion(&mut self) {
        self.push(TutorialNanoPresentationCommand::PlayAnimation(
            TutorialNanoPresentationAnimation::Stand,
        ));
    }

    pub fn set_voice_disabled(&mut self, disabled: bool) {
        self.push(TutorialNanoPresentationCommand::SetVoiceDisabled(disabled));
    }

    pub fn destroy(&mut self) {
        self.push(TutorialNanoPresentationCommand::Destroy);
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    fn pop_front(&mut self) -> Option<TutorialNanoPresentationCommand> {
        self.pending.pop_front()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TutorialNanoPresentationStatus {
    #[default]
    Absent,
    Loading,
    Ready,
    Blocked(String),
}

#[derive(Debug, Clone, Resource)]
pub struct TutorialNanoPresentationState {
    entity: Option<Entity>,
    live_transform: Option<Transform>,
    requested_animation: Option<String>,
    animation: LegacyNanoAnimationMachine,
    available_animation_names: Vec<String>,
    voice_disabled: bool,
    status: TutorialNanoPresentationStatus,
    asset_contract_ready: bool,
    face_texture_bound: bool,
    generation: u64,
    applied_animation_request_serial: Option<u64>,
}

impl Default for TutorialNanoPresentationState {
    fn default() -> Self {
        Self {
            entity: None,
            live_transform: None,
            requested_animation: None,
            animation: LegacyNanoAnimationMachine::default(),
            available_animation_names: Vec::new(),
            voice_disabled: true,
            status: TutorialNanoPresentationStatus::Absent,
            asset_contract_ready: false,
            face_texture_bound: false,
            generation: 0,
            applied_animation_request_serial: None,
        }
    }
}

impl TutorialNanoPresentationState {
    #[must_use]
    pub const fn entity(&self) -> Option<Entity> {
        self.entity
    }

    #[must_use]
    pub const fn live_transform(&self) -> Option<Transform> {
        self.live_transform
    }

    #[must_use]
    pub fn requested_animation(&self) -> Option<&str> {
        self.requested_animation.as_deref()
    }

    #[must_use]
    pub fn resolved_animation(&self) -> Option<&str> {
        self.animation.clip()
    }

    #[must_use]
    pub fn available_animation_names(&self) -> &[String] {
        &self.available_animation_names
    }

    #[must_use]
    pub const fn voice_disabled(&self) -> bool {
        self.voice_disabled
    }

    #[must_use]
    pub const fn status(&self) -> &TutorialNanoPresentationStatus {
        &self.status
    }

    #[must_use]
    pub const fn animation_applied(&self) -> bool {
        match self.applied_animation_request_serial {
            Some(serial) => serial == self.animation.request_serial(),
            None => false,
        }
    }

    fn refresh_readiness(&mut self) {
        if matches!(
            self.status,
            TutorialNanoPresentationStatus::Absent | TutorialNanoPresentationStatus::Blocked(_)
        ) {
            return;
        }
        self.status = if self.asset_contract_ready && self.face_texture_bound {
            TutorialNanoPresentationStatus::Ready
        } else {
            TutorialNanoPresentationStatus::Loading
        };
    }

    pub fn mark_absent(&mut self) {
        self.entity = None;
        self.live_transform = None;
        self.requested_animation = None;
        self.animation.clear();
        self.available_animation_names.clear();
        self.voice_disabled = true;
        self.status = TutorialNanoPresentationStatus::Absent;
        self.asset_contract_ready = false;
        self.face_texture_bound = false;
        self.applied_animation_request_serial = None;
    }
}

#[derive(Debug, Component)]
pub struct TutorialNanoPresentationRoot {
    pub generation: u64,
}

#[derive(Debug, Component)]
pub struct TutorialNanoPresentationScene;

#[derive(Debug, Clone, PartialEq, Eq, Component)]
pub struct TutorialNanoAnimationPlayback {
    pub generation: u64,
    pub request_serial: u64,
    pub clip: String,
    pub node: AnimationNodeIndex,
}

#[derive(Debug, Component)]
struct TutorialNanoFaceTextureBound;

#[derive(Default, Resource)]
struct TutorialNanoPresentationAssets {
    gltf: Option<Handle<Gltf>>,
    scene: Option<Handle<WorldAsset>>,
    animation_graph: Option<Handle<AnimationGraph>>,
    animation_nodes: BTreeMap<String, AnimationNodeIndex>,
    sound_events: Option<Arc<[NetworkNpcAnimationSoundEvent0104]>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum TutorialNanoPresentationSet {
    ApplyCommands,
    PrepareAsset,
    PlayAnimation,
    BindMaterials,
    Observe,
}

pub struct TutorialNanoPresentationPlugin;

impl Plugin for TutorialNanoPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialNanoPresentationCommandQueue>()
            .init_resource::<TutorialNanoPresentationState>()
            .init_resource::<TutorialNanoPresentationAssets>()
            .init_resource::<LegacyNanoStandRandomStream>()
            .configure_sets(
                Update,
                (
                    TutorialNanoPresentationSet::ApplyCommands,
                    TutorialNanoPresentationSet::PrepareAsset,
                    TutorialNanoPresentationSet::PlayAnimation,
                    TutorialNanoPresentationSet::BindMaterials,
                    TutorialNanoPresentationSet::Observe,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                apply_tutorial_nano_commands.in_set(TutorialNanoPresentationSet::ApplyCommands),
            )
            .add_systems(
                Update,
                prepare_tutorial_nano_asset.in_set(TutorialNanoPresentationSet::PrepareAsset),
            )
            .add_systems(
                Update,
                play_tutorial_nano_animation.in_set(TutorialNanoPresentationSet::PlayAnimation),
            )
            .add_systems(
                Update,
                emit_tutorial_nano_animation_sounds
                    .after(TutorialNanoPresentationSet::PlayAnimation)
                    .in_set(GameplayAudioSet::Collect),
            )
            .add_systems(
                Update,
                bind_tutorial_nano_face_texture.in_set(TutorialNanoPresentationSet::BindMaterials),
            )
            .add_systems(
                Update,
                observe_tutorial_nano_transform.in_set(TutorialNanoPresentationSet::Observe),
            );
    }
}

fn apply_tutorial_nano_commands(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut queue: ResMut<TutorialNanoPresentationCommandQueue>,
    mut state: ResMut<TutorialNanoPresentationState>,
    mut assets: ResMut<TutorialNanoPresentationAssets>,
    mut stand_random: ResMut<LegacyNanoStandRandomStream>,
) {
    while let Some(command) = queue.pop_front() {
        match command {
            TutorialNanoPresentationCommand::Spawn(spawn) => {
                if let Some(entity) = state.entity {
                    commands.entity(entity).despawn();
                }
                state.generation = state.generation.wrapping_add(1).max(1);
                let gltf = asset_server.load(TUTORIAL_NANO_MODEL_PATH);
                let scene = asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset(TUTORIAL_NANO_MODEL_PATH));
                let entity = commands
                    .spawn((
                        Name::new("newNano"),
                        TutorialNanoPresentationRoot {
                            generation: state.generation,
                        },
                        spawn.transform,
                        Visibility::Inherited,
                    ))
                    .with_child((
                        Name::new("nano_buttercup Scene0"),
                        TutorialNanoPresentationScene,
                        WorldAssetRoot(scene.clone()),
                        native_scene_container_transform(NativeSceneRole::CharacterGameplay),
                        Visibility::Inherited,
                    ))
                    .id();
                assets.gltf = Some(gltf);
                assets.scene = Some(scene);
                state.entity = Some(entity);
                state.live_transform = Some(spawn.transform);
                state.requested_animation = None;
                state.animation.clear();
                state.available_animation_names.clear();
                state.voice_disabled = true;
                state.status = TutorialNanoPresentationStatus::Loading;
                state.asset_contract_ready = false;
                state.face_texture_bound = false;
                state.applied_animation_request_serial = None;
            }
            TutorialNanoPresentationCommand::SetTransform(transform) => {
                if let Some(entity) = state.entity {
                    commands.entity(entity).insert(transform);
                    state.live_transform = Some(transform);
                }
            }
            TutorialNanoPresentationCommand::SetTranslation(translation) => {
                if let (Some(entity), Some(mut transform)) = (state.entity, state.live_transform) {
                    transform.translation = translation;
                    commands.entity(entity).insert(transform);
                    state.live_transform = Some(transform);
                }
            }
            TutorialNanoPresentationCommand::SetRotation(rotation) => {
                if let (Some(entity), Some(mut transform)) = (state.entity, state.live_transform) {
                    transform.rotation = rotation;
                    commands.entity(entity).insert(transform);
                    state.live_transform = Some(transform);
                }
            }
            TutorialNanoPresentationCommand::PlayAnimation(action) => {
                state.requested_animation = Some(action.clip_name().to_owned());
                action.apply(&mut state.animation, &mut stand_random);
                state.applied_animation_request_serial = None;
            }
            TutorialNanoPresentationCommand::SetVoiceDisabled(disabled) => {
                state.voice_disabled = disabled;
            }
            TutorialNanoPresentationCommand::Destroy => {
                if let Some(entity) = state.entity {
                    commands.entity(entity).despawn();
                }
                state.mark_absent();
            }
        }
    }
}

fn prepare_tutorial_nano_asset(
    asset_server: Res<AssetServer>,
    locator: Option<Res<AssetLocator>>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut assets: ResMut<TutorialNanoPresentationAssets>,
    mut state: ResMut<TutorialNanoPresentationState>,
) {
    let Some(gltf_handle) = assets.gltf.clone() else {
        return;
    };
    if matches!(state.status, TutorialNanoPresentationStatus::Loading)
        && !state.asset_contract_ready
    {
        if let Some(error) = asset_load_failure(&asset_server, &gltf_handle) {
            state.status = TutorialNanoPresentationStatus::Blocked(error);
            return;
        }
        let Some(gltf) = gltfs.get(&gltf_handle) else {
            return;
        };
        if assets.sound_events.is_none()
            && let Some(locator) = locator.as_deref()
        {
            let sound_events = match locator
                .read(TUTORIAL_NANO_MODEL_PATH)
                .and_then(|bytes| parse_network_npc_animation_sound_events(&bytes))
            {
                Ok(events) => events,
                Err(error) => {
                    state.status = TutorialNanoPresentationStatus::Blocked(format!(
                        "tutorial Nano AnimationEvent audio is invalid: {error}"
                    ));
                    return;
                }
            };
            assets.sound_events = Some(sound_events.into());
        }
        let missing = TUTORIAL_NANO_REQUIRED_ANIMATION_CLIPS
            .iter()
            .filter(|name| !gltf.named_animations.contains_key(**name))
            .copied()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            state.status = TutorialNanoPresentationStatus::Blocked(format!(
                "exact tutorial Nano GLB is missing required clips: {missing:?}"
            ));
            return;
        }
        state.available_animation_names = gltf
            .named_animations
            .keys()
            .map(|name| name.to_string())
            .collect();
        state.available_animation_names.sort();
        if assets.animation_graph.is_none() {
            let mut graph = AnimationGraph::new();
            let root = graph.root;
            let mut named = gltf.named_animations.iter().collect::<Vec<_>>();
            named.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
            assets.animation_nodes = named
                .into_iter()
                .map(|(name, clip)| {
                    let node = graph.add_clip(clip.clone(), 1.0, root);
                    (name.to_string(), node)
                })
                .collect();
            assets.animation_graph = Some(graphs.add(graph));
        }
        state.asset_contract_ready = true;
        state.refresh_readiness();
    }
}

#[derive(Debug, Clone, PartialEq, Component)]
struct TutorialNanoAnimationSoundCursor {
    generation: u64,
    request_serial: u64,
    clip: String,
    node: AnimationNodeIndex,
    seek_time: f32,
    completions: u32,
}

fn emit_tutorial_nano_animation_sounds(
    mut commands: Commands,
    state: Res<TutorialNanoPresentationState>,
    assets: Res<TutorialNanoPresentationAssets>,
    players: Query<(
        Entity,
        &AnimationPlayer,
        &TutorialNanoAnimationPlayback,
        Option<&TutorialNanoAnimationSoundCursor>,
    )>,
    runtime: Option<ResMut<GameplayAudioRuntime>>,
) {
    let (Some(root), Some(events), Some(mut runtime)) =
        (state.entity, assets.sound_events.as_deref(), runtime)
    else {
        return;
    };
    for (entity, player, applied, cursor) in &players {
        if applied.generation != state.generation {
            continue;
        }
        let Some(active) = player.animation(applied.node) else {
            continue;
        };
        let same_playback = cursor.is_some_and(|cursor| {
            cursor.generation == applied.generation
                && cursor.request_serial == applied.request_serial
                && cursor.clip == applied.clip
                && cursor.node == applied.node
                && active.completions() >= cursor.completions
        });
        let (previous_seek, previous_completions) = if same_playback {
            let cursor = cursor.expect("matching Nano sound cursor must exist");
            (cursor.seek_time, cursor.completions)
        } else {
            (0.0, 0)
        };
        for event in events.iter().filter(|event| event.clip == applied.clip) {
            // EventNanoAnimation.IsStand is the clean bVODisable gate. Dance
            // SFX are its one explicit exception.
            if state.voice_disabled() && !event.payload.contains("_SFX_Dance") {
                continue;
            }
            let crossings = animation_event_crossings(
                previous_seek,
                previous_completions,
                active.seek_time(),
                active.completions(),
                active.repeat_mode(),
                event.time,
            );
            for _ in 0..crossings {
                runtime.queue_legacy_nano_animation_sound(root, &event.payload);
            }
        }
        commands
            .entity(entity)
            .insert(TutorialNanoAnimationSoundCursor {
                generation: applied.generation,
                request_serial: applied.request_serial,
                clip: applied.clip.clone(),
                node: applied.node,
                seek_time: active.seek_time(),
                completions: active.completions(),
            });
    }
}

fn play_tutorial_nano_animation(
    mut commands: Commands,
    assets: Res<TutorialNanoPresentationAssets>,
    mut state: ResMut<TutorialNanoPresentationState>,
    mut stand_random: ResMut<LegacyNanoStandRandomStream>,
    parents: Query<&ChildOf>,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&mut AnimationTransitions>,
        Option<&TutorialNanoAnimationPlayback>,
    )>,
) {
    if !matches!(state.status, TutorialNanoPresentationStatus::Ready) {
        return;
    }
    let Some(root) = state.entity else {
        return;
    };
    let generation = state.generation;
    let request_serial = state.animation.request_serial();
    let mut matched_request = false;
    let mut request_finished = true;
    for (entity, player, _, applied) in &mut players {
        if !is_descendant_of(entity, root, &parents) {
            continue;
        }
        let Some(applied) = applied.filter(|applied| {
            applied.generation == generation && applied.request_serial == request_serial
        }) else {
            continue;
        };
        matched_request = true;
        request_finished &= player
            .animation(applied.node)
            .is_some_and(|animation| animation.is_finished());
    }
    if matched_request && request_finished {
        match state.animation.complete(&mut stand_random) {
            LegacyNanoCompletion::Despawn => {
                commands.entity(root).despawn();
                state.mark_absent();
                return;
            }
            LegacyNanoCompletion::Continue | LegacyNanoCompletion::SkillSpecialFinished => {
                state.applied_animation_request_serial = None;
            }
        }
    }

    let Some(clip_name) = state.animation.clip().map(str::to_owned) else {
        return;
    };
    let Some(graph) = assets.animation_graph.clone() else {
        return;
    };
    let Some(node) = assets.animation_nodes.get(&clip_name).copied() else {
        return;
    };
    let request_serial = state.animation.request_serial();
    let blend = state.animation.blend().duration();
    let mut found_player = false;
    for (entity, mut player, transitions, applied) in &mut players {
        if !is_descendant_of(entity, root, &parents) {
            continue;
        }
        found_player = true;
        if applied.is_some_and(|applied| {
            applied.generation == generation && applied.request_serial == request_serial
        }) {
            continue;
        }
        if let Some(mut transitions) = transitions {
            transitions
                .play(&mut player, node, blend)
                .set_repeat(RepeatAnimation::Never)
                .resume();
            commands.entity(entity).insert((
                AnimationGraphHandle(graph.clone()),
                TutorialNanoAnimationPlayback {
                    generation,
                    request_serial,
                    clip: clip_name.clone(),
                    node,
                },
            ));
        } else {
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, node, Duration::ZERO)
                .set_repeat(RepeatAnimation::Never)
                .resume();
            commands.entity(entity).insert((
                AnimationGraphHandle(graph.clone()),
                transitions,
                TutorialNanoAnimationPlayback {
                    generation,
                    request_serial,
                    clip: clip_name.clone(),
                    node,
                },
            ));
        }
    }
    if found_player {
        state.applied_animation_request_serial = Some(request_serial);
    }
}

fn bind_tutorial_nano_face_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<TutorialNanoPresentationState>,
    parents: Query<&ChildOf>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut surfaces: Query<
        (
            Entity,
            &mut MeshMaterial3d<LegacyModelMaterial>,
            &PendingLegacyModelMaterial,
        ),
        Without<TutorialNanoFaceTextureBound>,
    >,
) {
    if !state.asset_contract_ready {
        return;
    }
    let Some(root) = state.entity else {
        return;
    };
    for (entity, mut handle, metadata) in &mut surfaces {
        if metadata.true_name != TUTORIAL_NANO_FACE_MATERIAL_NAME
            || !is_descendant_of(entity, root, &parents)
        {
            continue;
        }
        let face_texture = match load_legacy_main_texture_replacement(
            &asset_server,
            metadata,
            TUTORIAL_NANO_FACE_TEXTURE_PATH,
        ) {
            Ok(texture) => texture,
            Err(error) => {
                state.status = TutorialNanoPresentationStatus::Blocked(format!(
                    "tutorial Nano face texture override is not exact: {error}"
                ));
                return;
            }
        };
        crate::legacy_model_material::make_legacy_material_unique(&mut handle.0, &mut materials);
        let Some(mut material) = materials.get_mut(&handle.0) else {
            continue;
        };
        material.base_texture = Some(face_texture);
        commands.entity(entity).insert(TutorialNanoFaceTextureBound);
        state.face_texture_bound = true;
        state.refresh_readiness();
    }
}

fn observe_tutorial_nano_transform(
    mut state: ResMut<TutorialNanoPresentationState>,
    roots: Query<&Transform, With<TutorialNanoPresentationRoot>>,
) {
    let Some(entity) = state.entity else {
        return;
    };
    if let Ok(transform) = roots.get(entity) {
        state.live_transform = Some(*transform);
    } else {
        state.mark_absent();
    }
}

fn asset_load_failure(asset_server: &AssetServer, handle: &Handle<Gltf>) -> Option<String> {
    match asset_server.load_state(handle.id()) {
        LoadState::Failed(error) => Some(format!("tutorial Nano GLB load failed: {error}")),
        _ => asset_server
            .get_recursive_dependency_load_state(handle.id())
            .and_then(|state| match state {
                RecursiveDependencyLoadState::Failed(error) => {
                    Some(format!("tutorial Nano GLB dependency load failed: {error}"))
                }
                _ => None,
            }),
    }
}

use crate::scene_hierarchy::is_descendant_of;

/// Exact state-transition cleanup hook for tutorial exit/skip boundaries.
pub fn cleanup_tutorial_nano_presentation(world: &mut World) {
    let entity = world.resource::<TutorialNanoPresentationState>().entity();
    if let Some(entity) = entity {
        let _ = world.despawn(entity);
    }
    let generation = world.resource::<TutorialNanoPresentationState>().generation;
    *world.resource_mut::<TutorialNanoPresentationState>() = TutorialNanoPresentationState {
        generation,
        ..default()
    };
    *world.resource_mut::<TutorialNanoPresentationCommandQueue>() =
        TutorialNanoPresentationCommandQueue::default();
}

#[cfg(test)]
mod tests;
