//! Tutorial sequence start, startup probes and player enable after collider readiness.

use super::asset_residency::{
    AssetResidency, AssetResidencyGroupId, TutorialEffectLibraryLoadStatus, resident_group_probe,
};
use super::dexter_ship_drive::dexter_ship_descendant;
use super::loading_screen::{GameplayLoadingPhase, GameplayLoadingState};
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_ambience::{
    TutorialAmbientAudio, TutorialAmbientCue, TutorialAmbientRuntime, TutorialDome,
};
use super::tutorial_session::{TutorialLogicRuntime, TutorialSession};
use super::world_scene::NativeWorldStartupQueries;
use super::{
    LocalNetworkIdentity, LocalPlayer, WorldSliceEntity, client_state_sends_movement_intents,
};
use bevy::{
    asset::{Asset, LoadState, RecursiveDependencyLoadState},
    audio::AudioSink,
    gltf::GltfAssetLabel,
    prelude::*,
};
use ffone_client::{
    coordinates::unity_to_native_vector,
    mission_ui::MissionUiModel,
    movement::{LegacyPlayerController, LegacyWorldColliderPending},
    player_shared_rig::NativePlayerRigStatus,
    tutorial::TutorialInputLock,
    tutorial_actors::{
        TutorialActor, TutorialActorAnimationSource, TutorialActorCommandQueue,
        TutorialActorRegistry,
    },
    tutorial_auxiliary_choreography::TUTORIAL_INITIALIZATION,
    tutorial_effects_runtime::{TutorialEffectRuntime, TutorialEffectRuntimeCommand},
    tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn},
    tutorial_nano_presentation::{
        TutorialNanoPresentationCommandQueue, TutorialNanoPresentationState,
        TutorialNanoPresentationStatus,
    },
    tutorial_native_mechanics::TutorialNativeMechanics,
    tutorial_player_presentation::{
        TutorialPlayerClip, TutorialPlayerPresentationCommandQueue,
        tutorial_player_gender_from_protocol,
    },
    tutorial_player_rig_runtime::TutorialSelectedPlayerRigStatus,
    world::{
        NativeWorldBehaviourStatus, NativeWorldColliderStatus, NativeWorldPresentationStatus,
        NativeWorldScope, spawn_pending_authored_model_perimeter_collider,
    },
};
use ffone_runtime_contracts::RETROBUTION_TUTORIAL_EFFECT_IDS;

// Primary cntutorialscript synthesizes PC_ENTER_SUCC from this fixed StartPos
// and angle instead of consuming the character's persisted shard position.
pub(super) const TUTORIAL_START_SERVER_POSITION: [i32; 3] = [54_700, 65_500, -10_540];
pub(super) const TUTORIAL_START_ANGLE: i32 = 270;
pub(super) const TUTORIAL_START_UNITY_POSITION: Vec3 = Vec3::new(547.0, -105.4, 655.0);
pub(super) const TUTORIAL_PROJECTILE_RNG_FALLBACK_SEED: u32 = 0xa511_e9b3;
pub(super) const TUTORIAL_MOVEMENT_MARKER_EFFECT_ID: i32 = 750;
pub(super) const TUTORIAL_DOME_MODEL_PATH: &str = "objects/structures/etc_domeglass_04_01_default_etc_domglass/models/etc_dome_glass_04/ETC_domeglass_04.glb";
pub(super) const TUTORIAL_DOME_COLLIDER_MESH: usize = 1;
pub(super) const TUTORIAL_DOME_COLLIDER_VERTEX_COUNT: usize = 140;
pub(super) const TUTORIAL_DOME_COLLIDER_INDEX_COUNT: usize = 198;
pub(super) const TUTORIAL_DOME_FOOTPRINT_MESH: usize = 0;
pub(super) const TUTORIAL_DOME_FOOTPRINT_SCALE: f32 = 0.119_403_06;
pub(super) const TUTORIAL_DOME_FOOTPRINT_VERTEX_COUNT: usize = 56;
pub(super) const TUTORIAL_DOME_FOOTPRINT_INDEX_COUNT: usize = 84;

#[allow(clippy::too_many_arguments)]
pub(super) fn start_tutorial_sequence(
    mut commands: Commands,
    tutorial: Res<TutorialSession>,
    mut tutorial_logic: ResMut<TutorialLogicRuntime>,
    mut native_mechanics: ResMut<TutorialNativeMechanics>,
    mut actor_commands: ResMut<TutorialActorCommandQueue>,
    mut runtime: ResMut<RuntimeStatus>,
    mut next_state: ResMut<NextState<ClientState>>,
    asset_server: Res<AssetServer>,
    local_players: Query<&LocalNetworkIdentity, With<LocalPlayer>>,
    mut nano_commands: ResMut<TutorialNanoPresentationCommandQueue>,
    mut ambient: ResMut<TutorialAmbientRuntime>,
) {
    tutorial_logic.reset();
    native_mechanics.reset();
    if let Some(stage) = tutorial.progress.stage() {
        native_mechanics.sync_stable_stage(stage);
    }
    let Some(character) = tutorial.character() else {
        runtime.message = "Tutorial start was requested without a selected character".to_owned();
        next_state.set(ClientState::CharacterSelect);
        return;
    };
    let Ok(identity) = local_players.single() else {
        runtime.message = "Tutorial sequence requires the shard-owned local player".to_owned();
        next_state.set(ClientState::CharacterSelect);
        return;
    };
    if identity.pc_uid != character.pc_uid {
        runtime.message = format!(
            "Tutorial sequence character {} does not match shard player {}",
            character.pc_uid, identity.pc_uid
        );
        next_state.set(ClientState::CharacterSelect);
        return;
    }

    nano_commands.spawn_legacy_hidden();
    let dome_position = TUTORIAL_INITIALIZATION.dome_offscreen_position;
    let dome_gltf = asset_server.load(TUTORIAL_DOME_MODEL_PATH);
    let dome_scene =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(TUTORIAL_DOME_MODEL_PATH));
    let dome = commands
        .spawn((
            Name::new("Retrobution tutorial dome"),
            WorldSliceEntity,
            TutorialDome {
                gltf: dome_gltf,
                scene: dome_scene.clone(),
            },
            WorldAssetRoot(dome_scene),
            Transform::from_translation(unity_to_native_vector(Vec3::new(
                dome_position[0],
                dome_position[1],
                dome_position[2],
            ))),
            Visibility::Inherited,
        ))
        .id();
    // The source KFM carries a separate child named `collision`
    // (Mesh pathId 670, MeshCollider pathId 6913). The visible Line02
    // shell has only 84 indices while the exact non-trigger source
    // collider has 198. The collider's disconnected panels leave
    // gameplay-sized corner gaps, so runtime retains their proven
    // height but cooks one continuous perimeter aligned to Line02's
    // exact visible footprint.
    spawn_pending_authored_model_perimeter_collider(
        &mut commands,
        &asset_server,
        dome,
        "Retrobution tutorial dome collision",
        TUTORIAL_DOME_MODEL_PATH,
        TUTORIAL_DOME_COLLIDER_MESH,
        0,
        Transform::from_translation(Vec3::new(-6.232_064_2, 0.0, 10.907_462))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        TUTORIAL_DOME_COLLIDER_VERTEX_COUNT,
        TUTORIAL_DOME_COLLIDER_INDEX_COUNT,
        TUTORIAL_DOME_FOOTPRINT_MESH,
        0,
        Vec2::splat(TUTORIAL_DOME_FOOTPRINT_SCALE),
        TUTORIAL_DOME_FOOTPRINT_VERTEX_COUNT,
        TUTORIAL_DOME_FOOTPRINT_INDEX_COUNT,
    );
    let initial_npc = TUTORIAL_INITIALIZATION.initial_npc;
    actor_commands.spawn(TutorialNpcSpawn::new(
        initial_npc.runtime_id,
        initial_npc.npc_type,
        LegacySpawnPosition::centiunits(
            initial_npc.legacy_server_position[0] * 100,
            initial_npc.legacy_server_position[1] * 100,
            initial_npc.legacy_server_position[2] * 100,
        ),
        Some(
            initial_npc
                .legacy_angle_degrees
                .try_into()
                .expect("tutorial initialization angle fits i16"),
        ),
    ));
    ambient
        .request_legacy_cue(TUTORIAL_INITIALIZATION.ambient_loop)
        .expect("the source-owned tutorial initialization ambient must have an exact route");
    runtime.message = "Tutorial sequence loading over the shared shard gameplay world".to_owned();
}

pub(super) fn prime_tutorial_startup_pose(
    tutorial: Res<TutorialSession>,
    mut player_commands: ResMut<TutorialPlayerPresentationCommandQueue>,
) {
    let Some(character) = tutorial.character() else {
        return;
    };
    let Ok(gender) = tutorial_player_gender_from_protocol(character.style.gender) else {
        return;
    };
    // BasicMoveEvent's first visible player command is the immediate
    // `staying` pose. Prime it while the loading overlay is still opaque; the
    // normal choreography call repeats the source command at t=0.
    player_commands
        .avatar_emote(gender, TutorialPlayerClip::Staying.name())
        .expect("the source-proven tutorial startup pose must resolve");
}

pub(super) fn preload_tutorial_effects(mut effect_runtime: ResMut<TutorialEffectRuntime>) {
    // The source AssetLoader fetched the complete Tutorial package before
    // handing control to the chapter scripts. Queue every effect referenced by
    // those scripts now; their later PreloadEffect calls become cache checks.
    for effect_id in RETROBUTION_TUTORIAL_EFFECT_IDS {
        effect_runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
            effect_id,
            source_line: 0,
        });
    }
    effect_runtime.process_pending();
}

pub(super) fn sync_tutorial_native_stage(
    state: Res<State<ClientState>>,
    tutorial: Res<TutorialSession>,
    mut native: ResMut<TutorialNativeMechanics>,
) {
    if *state.get() != ClientState::Tutorial {
        return;
    }
    if tutorial.completion_requested {
        native.fail_closed_input();
        return;
    }
    if let Some(stage) = tutorial.progress.stage() {
        native.sync_stable_stage(stage);
    } else {
        native.fail_closed_input();
    }
}

pub(super) fn suppress_locked_tutorial_ui_shortcuts(
    state: Res<State<ClientState>>,
    native: Res<TutorialNativeMechanics>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    if *state.get() != ClientState::Tutorial {
        return;
    }
    // `cnAvatarThirdPersonMove` never evaluates its Home-key AutoRun toggle
    // while `cntutorialscript.bTutorial` is true.
    keyboard.clear_just_pressed(KeyCode::Home);
    // Clean `CnGuiChat.Update` gates the Enter shortcut only with eLock.Menu.
    // ModeChange belongs to mode-specific close/exit callbacks (for example,
    // `cnMissionJournal.EndMode`) and must not swallow the key that opens the
    // Nanocom menu in Chapter_05 step 6.
    if native.is_locked(TutorialInputLock::Menu) {
        keyboard.clear_just_pressed(KeyCode::Enter);
        keyboard.clear_just_pressed(KeyCode::NumpadEnter);
    }
}

pub(super) fn tutorial_actor_needs_early_modal_lock(model: &MissionUiModel, actor: &TutorialActor) -> bool {
    actor.interacting
        && actor.is_alive()
        && model
            .npc_interaction
            .as_ref()
            .is_none_or(|interaction| interaction.npc_id != actor.id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TutorialStartupProbe {
    Ready,
    Loading(&'static str),
    Blocked(String),
}

/// Separates the resources required to admit the first playable tutorial
/// frame from speculative work retained for later chapters.
///
/// A future mob or effect may fail its background preload without trapping
/// the player behind the post-cutscene loader. Its own typed runtime remains
/// responsible for reporting that failure when the chapter actually uses it.
#[derive(Debug, Default)]
pub(super) struct TutorialStartupProbePlan {
    pub(super) admission: Vec<TutorialStartupProbe>,
    pub(super) background: Vec<TutorialStartupProbe>,
}

impl TutorialStartupProbePlan {
    pub(super) fn readiness(&self) -> TutorialStartupProbe {
        combine_tutorial_startup_probes(self.admission.iter().cloned())
    }
}

pub(super) fn combine_tutorial_startup_probes(
    probes: impl IntoIterator<Item = TutorialStartupProbe>,
) -> TutorialStartupProbe {
    let mut first_loading = None;
    for probe in probes {
        match probe {
            TutorialStartupProbe::Ready => {}
            TutorialStartupProbe::Loading(label) => {
                first_loading.get_or_insert(label);
            }
            TutorialStartupProbe::Blocked(error) => {
                return TutorialStartupProbe::Blocked(error);
            }
        }
    }
    first_loading.map_or(TutorialStartupProbe::Ready, TutorialStartupProbe::Loading)
}

pub(super) fn native_asset_probe<A: Asset>(
    asset_server: &AssetServer,
    handle: &Handle<A>,
    label: &'static str,
) -> TutorialStartupProbe {
    if asset_server.is_loaded_with_dependencies(handle.id()) {
        return TutorialStartupProbe::Ready;
    }
    if let LoadState::Failed(error) = asset_server.load_state(handle.id()) {
        return TutorialStartupProbe::Blocked(format!("{label} load failed: {error}"));
    }
    if let Some(RecursiveDependencyLoadState::Failed(error)) =
        asset_server.get_recursive_dependency_load_state(handle.id())
    {
        return TutorialStartupProbe::Blocked(format!("{label} dependency load failed: {error}"));
    }
    TutorialStartupProbe::Loading(label)
}

pub(super) fn tutorial_player_startup_pose_ready(
    status: &TutorialSelectedPlayerRigStatus,
    applied_clip: Option<TutorialPlayerClip>,
) -> bool {
    matches!(status, TutorialSelectedPlayerRigStatus::Ready)
        && applied_clip == Some(TutorialPlayerClip::Staying)
}

pub(super) fn selected_player_presentation_probe(
    state: ClientState,
    status: Option<&TutorialSelectedPlayerRigStatus>,
    applied_clip: Option<TutorialPlayerClip>,
    has_duplicate: bool,
) -> TutorialStartupProbe {
    if has_duplicate {
        return TutorialStartupProbe::Blocked(
            "gameplay startup created more than one selected player presentation rig".to_owned(),
        );
    }
    match status {
        None => TutorialStartupProbe::Loading("selected player presentation rig"),
        Some(TutorialSelectedPlayerRigStatus::Blocked(error)) => {
            TutorialStartupProbe::Blocked(format!("selected player presentation blocked: {error}"))
        }
        Some(TutorialSelectedPlayerRigStatus::Loading) => {
            TutorialStartupProbe::Loading("assembled selected player presentation rig")
        }
        Some(status)
            if state == ClientState::Tutorial
                && !tutorial_player_startup_pose_ready(status, applied_clip) =>
        {
            TutorialStartupProbe::Loading("initial tutorial player staying pose")
        }
        Some(TutorialSelectedPlayerRigStatus::Ready) => TutorialStartupProbe::Ready,
    }
}

pub(super) fn gameplay_loading_phase_for_startup_label(label: &str) -> GameplayLoadingPhase {
    if label.contains("appearance")
        || label.contains("pose")
        || label.contains("rig")
        || label.contains("Nano")
    {
        GameplayLoadingPhase::PresentationBinding
    } else if label.contains("world")
        || label.contains("collision")
        || label.contains("assembled")
        || label.contains("actor")
        || label.contains("dome")
    {
        GameplayLoadingPhase::SceneAssembly
    } else {
        GameplayLoadingPhase::Assets
    }
}

pub(super) fn enable_player_after_native_collider_ready(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    residency: Res<AssetResidency>,
    effect_library: Res<TutorialEffectLibraryLoadStatus>,
    effect_runtime: Res<TutorialEffectRuntime>,
    world: NativeWorldStartupQueries,
    registry: Res<TutorialActorRegistry>,
    actors: Query<(&TutorialActor, Option<&TutorialActorAnimationSource>)>,
    domes: Query<&TutorialDome>,
    nano: Res<TutorialNanoPresentationState>,
    ambient: Res<TutorialAmbientRuntime>,
    ambient_audio: Query<(&TutorialAmbientAudio, Option<&AudioSink>)>,
    mut players: Query<
        (
            Entity,
            &Transform,
            &mut LegacyPlayerController,
            &mut Visibility,
        ),
        With<LegacyWorldColliderPending>,
    >,
    state: Res<State<ClientState>>,
    mut runtime: ResMut<RuntimeStatus>,
    mut loading: ResMut<GameplayLoadingState>,
) {
    // Deferred teardown may leave a pending player for a transition frame.
    // Only gameplay owns this barrier; it must not reopen or reset a menu's
    // loading screen while that menu is settling its own presentation.
    if !client_state_sends_movement_intents(*state.get()) || players.is_empty() {
        return;
    }

    let scope = if *state.get() == ClientState::Tutorial {
        NativeWorldScope::Tutorial
    } else {
        NativeWorldScope::WorldMap
    };
    let Some(player_position) = players
        .iter()
        .next()
        .map(|(_, transform, _, _)| transform.translation)
    else {
        return;
    };
    let target_tiles = world
        .catalog
        .legacy_stream_target_tiles(scope, player_position);
    let mut world_probes = Vec::with_capacity(target_tiles.len());
    if target_tiles.is_empty() {
        world_probes.push(TutorialStartupProbe::Blocked(format!(
            "no authored {scope:?} tiles are inside the Retrobution startup radius"
        )));
    }
    if let Some(error) = world.streaming.blocker.as_ref() {
        world_probes.push(TutorialStartupProbe::Blocked(error.clone()));
    }
    if world.location.scope != Some(scope)
        || world.location.position != Some(player_position)
        || !world.location.ready
    {
        world_probes.push(TutorialStartupProbe::Loading("nearby world and neighboring water presentation"));
    }
    for tile in &target_tiles {
        let Some((root_entity, _, presentation, behaviour)) = world
            .roots
            .iter()
            .find(|(_, root, _, _)| root.selection_scope == scope && root.tile == *tile)
        else {
            world_probes.push(TutorialStartupProbe::Loading("nearby world tile"));
            continue;
        };

        let behaviour_probe = match behaviour {
            NativeWorldBehaviourStatus::Loading => {
                TutorialStartupProbe::Loading("nearby world behaviour")
            }
            NativeWorldBehaviourStatus::Ready => TutorialStartupProbe::Ready,
            NativeWorldBehaviourStatus::Blocked(error) => {
                TutorialStartupProbe::Blocked(format!("native world behaviour blocked: {error}"))
            }
        };

        let tile_colliders = world
            .colliders
            .iter()
            .filter(|(entity, _)| {
                *entity == root_entity
                    || dexter_ship_descendant(*entity, root_entity, &world.parents)
            })
            .map(|(_, status)| status)
            .collect::<Vec<_>>();
        let collider_probe = if tile_colliders.is_empty()
            || tile_colliders
                .iter()
                .any(|status| matches!(status, NativeWorldColliderStatus::Loading))
        {
            TutorialStartupProbe::Loading("nearby world collision")
        } else if let Some(error) = tile_colliders.iter().find_map(|status| match status {
            NativeWorldColliderStatus::Blocked(error) => Some(error.clone()),
            NativeWorldColliderStatus::Loading | NativeWorldColliderStatus::Ready { .. } => None,
        }) {
            TutorialStartupProbe::Blocked(format!("native world collider blocked: {error}"))
        } else {
            TutorialStartupProbe::Ready
        };

        let scene_probe = combine_tutorial_startup_probes(
            world
                .scenes
                .iter()
                .filter(|(entity, _)| {
                    *entity == root_entity
                        || dexter_ship_descendant(*entity, root_entity, &world.parents)
                })
                .map(|(_, scene)| {
                    native_asset_probe(&asset_server, &scene.0, "nearby world visuals")
                }),
        );
        let presentation_probe = match presentation {
            NativeWorldPresentationStatus::Loading => {
                TutorialStartupProbe::Loading("assembled nearby world presentation")
            }
            NativeWorldPresentationStatus::Ready => TutorialStartupProbe::Ready,
        };
        world_probes.push(combine_tutorial_startup_probes([
            collider_probe,
            scene_probe,
            presentation_probe,
            behaviour_probe,
        ]));
    }
    let world_probe = combine_tutorial_startup_probes(world_probes.iter().cloned());

    let rig_probe = match world.player_rigs.single() {
        Ok(NativePlayerRigStatus::ReadyAnimated { .. }) => TutorialStartupProbe::Ready,
        Ok(NativePlayerRigStatus::Blocked(error)) => {
            TutorialStartupProbe::Blocked(format!("selected player rig blocked: {error}"))
        }
        Ok(NativePlayerRigStatus::Loading(_)) | Err(_) => {
            TutorialStartupProbe::Loading("selected player appearance")
        }
    };

    let mut selected_rigs = world.tutorial_player_rigs.iter();
    let selected_rig = selected_rigs.next();
    let selected_presentation_probe = selected_player_presentation_probe(
        *state.get(),
        selected_rig.map(|(status, _)| status),
        selected_rig.and_then(|(_, applied)| applied.map(|applied| applied.clip)),
        selected_rigs.next().is_some(),
    );
    let effect_library_probe = match &*effect_library {
        TutorialEffectLibraryLoadStatus::Dormant => {
            TutorialStartupProbe::Loading("effect catalog load request")
        }
        TutorialEffectLibraryLoadStatus::Loading => {
            TutorialStartupProbe::Loading("validated effect catalog")
        }
        TutorialEffectLibraryLoadStatus::Ready => TutorialStartupProbe::Ready,
        TutorialEffectLibraryLoadStatus::Failed(error) => {
            TutorialStartupProbe::Blocked(format!("validated effect catalog blocked: {error}"))
        }
    };
    let mut probe_plan = TutorialStartupProbePlan {
        admission: vec![
            world_probe,
            rig_probe,
            selected_presentation_probe,
            effect_library_probe,
        ],
        ..default()
    };
    if *state.get() == ClientState::Tutorial {
        let residency_probe =
            resident_group_probe(&asset_server, &residency, AssetResidencyGroupId::Tutorial);
        let resident_tutorial_probe = if let Some(error) = residency_probe.blocker.clone() {
            TutorialStartupProbe::Blocked(format!("tutorial package blocked: {error}"))
        } else if residency_probe.is_ready() {
            TutorialStartupProbe::Ready
        } else {
            TutorialStartupProbe::Loading("complete tutorial cutscene package")
        };
        let effects_probe =
            combine_tutorial_startup_probes(RETROBUTION_TUTORIAL_EFFECT_IDS.into_iter().map(
                |effect_id| {
                    if effect_runtime.is_native_preload_complete(effect_id) {
                        TutorialStartupProbe::Ready
                    } else {
                        TutorialStartupProbe::Loading("tutorial effect package")
                    }
                },
            ));
        let dome_probe = {
            let mut domes = domes.iter();
            match (domes.next(), domes.next()) {
                (None, _) => TutorialStartupProbe::Loading("exact tutorial dome"),
                (Some(_), Some(_)) => TutorialStartupProbe::Blocked(
                    "tutorial startup created more than one exact dome".to_owned(),
                ),
                (Some(dome), None) => combine_tutorial_startup_probes([
                    native_asset_probe(&asset_server, &dome.gltf, "exact tutorial dome GLB"),
                    native_asset_probe(&asset_server, &dome.scene, "exact tutorial dome Scene0"),
                ]),
            }
        };

        let initial_npc = TUTORIAL_INITIALIZATION.initial_npc;
        let actor_probe = registry.entity(initial_npc.runtime_id).map_or(
            TutorialStartupProbe::Loading("initial npc_building actor"),
            |entity| match actors.get(entity) {
                Ok((actor, _source))
                    if actor.id != initial_npc.runtime_id
                        || actor.npc_type != initial_npc.npc_type =>
                {
                    TutorialStartupProbe::Blocked(
                        "initial tutorial actor identity contradicts InitStartChapter".to_owned(),
                    )
                }
                Ok((_, None)) => {
                    TutorialStartupProbe::Loading("initial npc_building/NpcTexture closure")
                }
                Ok((_, Some(source))) => native_asset_probe(
                    &asset_server,
                    &source.gltf,
                    "initial npc_building/NpcTexture closure",
                ),
                Err(_) => TutorialStartupProbe::Loading("initial npc_building actor"),
            },
        );

        let nano_probe = match nano.status() {
            TutorialNanoPresentationStatus::Ready if nano.entity().is_some() => {
                TutorialStartupProbe::Ready
            }
            TutorialNanoPresentationStatus::Blocked(error) => {
                TutorialStartupProbe::Blocked(format!("hidden tutorial Nano blocked: {error}"))
            }
            TutorialNanoPresentationStatus::Absent | TutorialNanoPresentationStatus::Ready => {
                TutorialStartupProbe::Loading("hidden exact tutorial Nano")
            }
            TutorialNanoPresentationStatus::Loading => {
                TutorialStartupProbe::Loading("hidden exact tutorial Nano and face texture")
            }
        };

        let ambient_probe = if let Some(error) = ambient.blocked_error.as_ref() {
            TutorialStartupProbe::Blocked(format!("initial tutorial ambient blocked: {error}"))
        } else if ambient.is_stable(TutorialAmbientCue::TutorialMain) {
            let Some(entity) = ambient.current_entity else {
                unreachable!("stable ambient always owns an entity")
            };
            match ambient_audio.get(entity) {
                Ok((marker, Some(_))) if marker.cue == TutorialAmbientCue::TutorialMain => {
                    TutorialStartupProbe::Ready
                }
                Ok((marker, _)) if marker.cue != TutorialAmbientCue::TutorialMain => {
                    TutorialStartupProbe::Blocked(
                        "startup ambient entity owns the wrong exact cue".to_owned(),
                    )
                }
                _ => TutorialStartupProbe::Loading("playing TutorialMain ambient sink"),
            }
        } else {
            TutorialStartupProbe::Loading("TutorialMain ambient crossfade")
        };
        // These two probes cover assets used by later chapters. Keep their
        // strong handles and preload work, but never make the first playable
        // frame depend on every future mob/effect remaining valid.
        probe_plan
            .background
            .extend([resident_tutorial_probe, effects_probe]);
        probe_plan
            .admission
            .extend([dome_probe, actor_probe, nano_probe, ambient_probe]);
    }

    let ready_world_tiles = world_probes
        .iter()
        .filter(|probe| matches!(probe, TutorialStartupProbe::Ready))
        .count();
    let ready_other = probe_plan
        .admission
        .iter()
        .skip(1)
        .filter(|probe| matches!(probe, TutorialStartupProbe::Ready))
        .count();
    let ready_background = probe_plan
        .background
        .iter()
        .filter(|probe| matches!(probe, TutorialStartupProbe::Ready))
        .count();
    let completed = ready_world_tiles + ready_other + ready_background;
    let total = target_tiles.len()
        + probe_plan.admission.len().saturating_sub(1)
        + probe_plan.background.len();
    let progress = if total == 0 {
        0.0
    } else {
        completed as f32 / total as f32
    };
    let readiness = probe_plan.readiness();
    if loading.last_startup_probe.as_ref() != Some(&readiness) {
        info!("Gameplay startup readiness: {readiness:?}");
        loading.last_startup_probe = Some(readiness.clone());
    }

    match readiness {
        TutorialStartupProbe::Blocked(error) => {
            runtime.message = format!("Gameplay startup blocked: {error}");
            loading.block(error);
            return;
        }
        TutorialStartupProbe::Loading(label) => {
            let phase = gameplay_loading_phase_for_startup_label(label);
            if phase == GameplayLoadingPhase::Assets {
                loading.loading(progress, progress);
            } else {
                loading.prepare(phase, progress, progress);
            }
            return;
        }
        TutorialStartupProbe::Ready => {
            // Reveal below the still-opaque loader. This lets the render world
            // extract the already-applied `staying` pose instead of first
            // seeing the rig's authored stand1 default after the loader closes.
            for (_, _, _, mut visibility) in &mut players {
                *visibility = Visibility::Inherited;
            }
            if !loading.settle_render_presentation() {
                return;
            }
        }
    }

    for (entity, _, mut controller, mut visibility) in &mut players {
        controller.movement_enabled = true;
        *visibility = Visibility::Inherited;
        commands
            .entity(entity)
            .remove::<LegacyWorldColliderPending>();
    }
    loading.finish();
    info!(
        "Gameplay entry ready: state={:?}, collider and player presentation admitted",
        state.get()
    );
    runtime.message = if *state.get() == ClientState::Tutorial {
        "Shard tutorial world ready; scenario sequence active".to_owned()
    } else {
        "OpenFusion world ready; authored collision active".to_owned()
    };
}
