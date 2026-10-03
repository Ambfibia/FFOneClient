//! Tutorial session, logic/choreography/mission runtimes and tutorial drives.

use super::LocalPlayer;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_ambience::{
    TutorialAmbientAudio, TutorialAmbientRuntime, TutorialDome, TutorialLoopAudio,
    TutorialMusicAudio, TutorialVoiceAudio,
};
use super::tutorial_startup::TUTORIAL_PROJECTILE_RNG_FALLBACK_SEED;
use super::tutorial_warp::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use ffone_client::{
    coordinates::LegacyUnityHeadingDegrees,
    gameplay_audio::{GameplayAudioRuntime, RetrobutionAudioMix},
    gameplay_ui::ChatLineUi,
    localization::{Language, Localization, VoiceLanguage},
    mission_ui::MissionUiModel,
    movement::{LegacyOrbitCamera, LegacyPlayerController, advance_native_xorshift32},
    nanocom_message_ui::NanocomMessageUiModel,
    network::CharacterSummary,
    tutorial::{TutorialProgress, TutorialScene, TutorialStage},
    tutorial_actors::{
        TutorialActor, TutorialActorCommandQueue, TutorialActorRegistry,
        TutorialNpcObservationSnapshot,
    },
    tutorial_auxiliary_choreography::{
        TutorialAuxiliaryPlayer, TutorialAuxiliarySequence,
        TutorialScreenPivot as AuxiliaryScreenPivot, TutorialScreenPoint, TutorialText,
    },
    tutorial_choreography_formula::TutorialCameraCaptureStore,
    tutorial_choreography_runtime::{
        ChoreographyPlaybackEvent, TutorialChoreographyIssueQueue, TutorialChoreographyPlayer,
        TutorialChoreographyPresentation,
    },
    tutorial_effects_runtime::{
        TutorialEffectRuntime, tutorial_oni_velocity_samples_from_unity_unit_draws,
    },
    tutorial_logic::{
        TutorialDialogue, TutorialIntent, TutorialNpcObservation, TutorialUiObservation,
    },
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_gameplay::{TutorialNanoGameplayCommandQueue, TutorialNanoGameplayState},
    tutorial_nano_presentation::{
        TutorialNanoPresentationCommandQueue, TutorialNanoPresentationState,
    },
    tutorial_native_mechanics::TutorialNativeMechanics,
    tutorial_overlay_ui::TutorialOverlayUiModel,
    tutorial_player_presentation::TutorialPlayerPresentationCommandQueue,
    tutorial_voice_subtitles::TutorialVoiceSubtitleState,
    world_mission_runtime::{WorldMissionCompletionKind, WorldMissionDialogueEdge},
};
use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Default, Resource)]
pub(super) struct TutorialSession {
    pub(super) character: Option<CharacterSummary>,
    pub(super) progress: TutorialProgress,
    pub(super) scene: TutorialScene,
    pub(super) observed_stage: Option<TutorialStage>,
    pub(super) step_elapsed: f32,
    pub(super) step_origin_position: Vec3,
    pub(super) step_origin_yaw: f32,
    pub(super) jump_seen: bool,
    pub(super) completion_requested: bool,
    pub(super) exit_teardown_applied: bool,
    pub(super) presentation_cursor: usize,
}

/// Bevy-side observations and blocking state consumed by the engine-neutral
/// Chapter_03..Chapter_07 transition table.
///
/// NPC and UI observations intentionally stay false until their native
/// implementations write real state here. Keeping unresolved intents visible
/// prevents the tutorial driver from pretending that an absent mechanic
/// completed.
#[derive(Debug, Default, Resource)]
pub(super) struct TutorialLogicRuntime {
    pub(super) npcs: TutorialNpcObservation,
    pub(super) ui: TutorialUiObservation,
    pub(super) reminder_elapsed_seconds: f32,
    pub(super) delay_remaining_seconds: Option<f32>,
    pub(super) deferred_intents: Vec<TutorialIntent>,
}

#[derive(Debug, Default, Resource)]
pub(super) struct TutorialChoreographyExecution {
    pub(super) pending: Vec<ChoreographyPlaybackEvent>,
    pub(super) player_start: Vec3,
    pub(super) player_pose: Option<&'static str>,
    pub(super) player_target_heading: Option<LegacyUnityHeadingDegrees>,
    pub(super) tutorial_weapon_id: Option<i32>,
    pub(super) camera_captures: TutorialCameraCaptureStore,
}

#[derive(Debug, Default, Resource)]
pub(super) struct TutorialMissionRuntime {
    pub(super) dialogues: Vec<TutorialDialogue>,
    pub(super) dialogue_edges: Vec<(i32, WorldMissionDialogueEdge)>,
    /// Retrobution `CnGuiChat.AllChatStrings` entries emitted by tutorial
    /// `VoiceOut`, `SubText`, and `SubText2` calls, in emission order.
    pub(super) chat_lines: Vec<ChatLineUi>,
    pub(super) auxiliary_dialogue: Option<TutorialDialogue>,
    pub(super) auxiliary: TutorialAuxiliaryPlayer,
    pub(super) basic_arrow_started: bool,
    pub(super) location_scan_elapsed_seconds: f32,
    pub(super) active_tasks: Vec<i32>,
    pub(super) completed_tasks: Vec<i32>,
    /// Retrobution `cnMissionManager.m_iMenualSelectedMissionID`.
    pub(super) selected_mission_id: Option<i32>,
    pub(super) completed_warp_ids: Vec<i32>,
    pub(super) pending_warp: Option<PendingTutorialNpcWarp>,
    pub(super) waypoint_actor_id: Option<i32>,
    pub(super) my_point_event: bool,
    pub(super) waypoint_event: bool,
    pub(super) fusion_matter: i32,
}

/// `CnGuiChat.iMaxChatList` in the clean Retrobution client.
pub(super) const TUTORIAL_CHAT_HISTORY_LIMIT: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PendingTutorialActorEffect {
    pub(super) sequence: TutorialAuxiliarySequence,
    pub(super) runtime_id: i32,
    pub(super) offset: [f32; 3],
    pub(super) effect_id: i32,
    pub(super) source_line: u32,
}

/// `MinimapEvent` issues AddEffect one second after AddNpc. Resolve the actor
/// transform after this frame's grounding pass, rather than capturing a stale
/// pre-grounding transform in the auxiliary timeline system.
#[derive(Debug, Default, Resource)]
pub(super) struct PendingTutorialActorEffects {
    pub(super) spawns: VecDeque<PendingTutorialActorEffect>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct TutorialTaskMutation {
    pub(super) state_changed: bool,
    pub(super) completion_kind: Option<WorldMissionCompletionKind>,
    pub(super) outgoing_task_id: Option<i32>,
    pub(super) outgoing_started: bool,
}

impl TutorialTaskMutation {
    pub(super) fn completion_audio_true_name(self) -> Option<&'static str> {
        self.completion_kind.map(WorldMissionCompletionKind::legacy_audio_true_name)
    }

    pub(super) fn queue_completion_audio(self, audio: &mut GameplayAudioRuntime) {
        if let Some(true_name) = self.completion_audio_true_name() {
            audio.queue_gameplay_ui_sound(true_name);
        }
    }
}

impl TutorialMissionRuntime {
    pub(super) fn push_chat_line(&mut self, line: ChatLineUi) {
        if line.text.is_empty() {
            return;
        }
        self.chat_lines.push(line);
        if self.chat_lines.len() > TUTORIAL_CHAT_HISTORY_LIMIT {
            let overflow = self.chat_lines.len() - TUTORIAL_CHAT_HISTORY_LIMIT;
            self.chat_lines.drain(..overflow);
        }
    }

    pub(super) fn push_voice_chat_line(&mut self, text: impl Into<String>) {
        self.push_chat_line(ChatLineUi::normal(text));
    }

    pub(super) fn push_instruction_chat_line(&mut self, text: impl Into<String>) {
        self.push_chat_line(ChatLineUi::tutorial(text));
    }

    pub(super) fn task_is_active(&self, task_id: i32) -> bool {
        self.active_tasks.contains(&task_id)
    }

    pub(super) fn start_task(
        &mut self,
        content: &TutorialMissionContent,
        task_id: i32,
    ) -> Result<bool, String> {
        content
            .mission(task_id)
            .map_err(|error| error.to_string())?;
        if self.task_is_active(task_id) || self.completed_tasks.contains(&task_id) {
            return Ok(false);
        }
        self.active_tasks.push(task_id);
        self.dialogue_edges
            .push((task_id, WorldMissionDialogueEdge::Start));
        sort_tutorial_task_ids(&mut self.active_tasks, content);
        let selected_is_active = self.selected_mission_id.is_some_and(|selected| {
            self.active_tasks.iter().any(|active_task_id| {
                content
                    .mission(*active_task_id)
                    .is_ok_and(|definition| definition.provenance.mission_id == selected)
            })
        });
        if !selected_is_active {
            self.selected_mission_id = Some(
                content
                    .mission(task_id)
                    .map_err(|error| error.to_string())?
                    .provenance
                    .mission_id,
            );
        }
        Ok(true)
    }

    pub(super) fn complete_task(
        &mut self,
        content: &TutorialMissionContent,
        task_id: i32,
    ) -> Result<TutorialTaskMutation, String> {
        let outgoing_task_id = content
            .outgoing_task_id(task_id)
            .map_err(|error| error.to_string())?;
        if !self.task_is_active(task_id) {
            if self.completed_tasks.contains(&task_id) {
                return Ok(TutorialTaskMutation {
                    outgoing_task_id,
                    ..default()
                });
            }
            return Err(format!("tutorial task {task_id} is not active"));
        }
        if let Some(outgoing) = outgoing_task_id {
            content
                .mission(outgoing)
                .map_err(|error| error.to_string())?;
        }
        let completes_mission = outgoing_task_id.is_none()
            || content
                .is_final_serialized_task(task_id)
                .map_err(|error| error.to_string())?;

        self.active_tasks.retain(|active| *active != task_id);
        if !self.completed_tasks.contains(&task_id) {
            self.completed_tasks.push(task_id);
            sort_tutorial_task_ids(&mut self.completed_tasks, content);
        }
        self.dialogue_edges
            .push((task_id, WorldMissionDialogueEdge::Complete));
        let outgoing_started = if let Some(outgoing) = outgoing_task_id {
            self.start_task(content, outgoing)?
        } else {
            false
        };
        if outgoing_task_id.is_none()
            && content.mission(task_id).is_ok_and(|definition| {
                self.selected_mission_id == Some(definition.provenance.mission_id)
            })
        {
            self.selected_mission_id = self.active_tasks.last().and_then(|active_task_id| {
                content
                    .mission(*active_task_id)
                    .ok()
                    .map(|definition| definition.provenance.mission_id)
            });
        }
        Ok(TutorialTaskMutation {
            state_changed: true,
            completion_kind: Some(if completes_mission {
                WorldMissionCompletionKind::Mission
            } else {
                WorldMissionCompletionKind::Task
            }),
            outgoing_task_id,
            outgoing_started,
        })
    }
}

pub(super) fn sort_tutorial_task_ids(task_ids: &mut [i32], content: &TutorialMissionContent) {
    task_ids.sort_by_key(|task_id| content.mission_row_index(*task_id).unwrap_or(usize::MAX));
}

#[derive(Debug, Default, Resource)]
pub(super) struct TutorialAuxiliaryPresentation {
    pub(super) sequence: Option<TutorialAuxiliarySequence>,
    pub(super) choreography_owned: bool,
    pub(super) picture: Option<(&'static str, TutorialScreenPoint, AuxiliaryScreenPivot)>,
    pub(super) cursor: Option<(&'static str, TutorialScreenPoint, AuxiliaryScreenPivot)>,
    pub(super) primary_subtitle: Option<TutorialText>,
    pub(super) secondary_subtitle: Option<TutorialText>,
    pub(super) message_box: Option<(i32, TutorialText, i32)>,
    pub(super) picture_touched: bool,
    pub(super) cursor_touched: bool,
    pub(super) primary_subtitle_touched: bool,
    pub(super) secondary_subtitle_touched: bool,
}

/// App-lifetime compatibility stream for the two tutorial Oni projectiles.
///
/// Retrobution uses Unity's app-global `Random` stream, whose seed/state is not
/// serialized. This stream is therefore initialized once and never reset on a
/// tutorial transition. It reproduces only the proven inclusive unit range and
/// exact draw order/count, not Unity's unknown random sequence.
#[derive(Debug, Resource)]
pub(super) struct TutorialProjectileRandomStream {
    pub(super) state: u32,
}

impl Default for TutorialProjectileRandomStream {
    fn default() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let folded = nanos ^ (nanos >> 32) ^ (nanos >> 64) ^ (nanos >> 96);
        Self::with_seed(folded as u32)
    }
}

impl TutorialProjectileRandomStream {
    pub(super) fn with_seed(seed: u32) -> Self {
        Self {
            state: if seed == 0 {
                TUTORIAL_PROJECTILE_RNG_FALLBACK_SEED
            } else {
                seed
            },
        }
    }

    pub(super) fn next_unity_unit_draw(&mut self) -> f32 {
        let value = advance_native_xorshift32(&mut self.state);
        (f64::from(value) / f64::from(u32::MAX)) as f32
    }

    pub(super) fn sample_oni_pair(&mut self, reverse: bool) -> [Vec3; 2] {
        let draw_count = if reverse { 4 } else { 6 };
        let mut draws = [0.0; 6];
        for draw in &mut draws[..draw_count] {
            *draw = self.next_unity_unit_draw();
        }
        tutorial_oni_velocity_samples_from_unity_unit_draws(reverse, &draws[..draw_count])
            .expect("xorshift32 unit draws satisfy the sampled Oni contract")
    }
}

#[derive(SystemParam)]
pub(super) struct TutorialActorDrive<'w, 's> {
    pub(super) tutorial_logic: ResMut<'w, TutorialLogicRuntime>,
    pub(super) native_mechanics: ResMut<'w, TutorialNativeMechanics>,
    pub(super) next_state: ResMut<'w, NextState<ClientState>>,
    pub(super) actor_commands: ResMut<'w, TutorialActorCommandQueue>,
    pub(super) gameplay_nano_commands: ResMut<'w, TutorialNanoGameplayCommandQueue>,
    pub(super) gameplay_nano: Res<'w, TutorialNanoGameplayState>,
    pub(super) effect_runtime: ResMut<'w, TutorialEffectRuntime>,
    pub(super) npc_snapshot: Res<'w, TutorialNpcObservationSnapshot>,
    pub(super) choreography_player: ResMut<'w, TutorialChoreographyPlayer>,
    pub(super) choreography_presentation: ResMut<'w, TutorialChoreographyPresentation>,
    pub(super) choreography_issues: ResMut<'w, TutorialChoreographyIssueQueue>,
    pub(super) choreography_execution: ResMut<'w, TutorialChoreographyExecution>,
    pub(super) mission_runtime: ResMut<'w, TutorialMissionRuntime>,
    pub(super) tutorial_content: Res<'w, TutorialMissionContent>,
    pub(super) audio_mix: Res<'w, RetrobutionAudioMix>,
    pub(super) gameplay_audio: Option<ResMut<'w, GameplayAudioRuntime>>,
    pub(super) localization: Res<'w, Localization>,
    pub(super) voice_language: Res<'w, VoiceLanguage>,
    pub(super) voice_subtitles: ResMut<'w, TutorialVoiceSubtitleState>,
    pub(super) auxiliary_presentation: ResMut<'w, TutorialAuxiliaryPresentation>,
    pub(super) ambient: ResMut<'w, TutorialAmbientRuntime>,
    pub(super) tutorial_domes:
        Query<'w, 's, (Entity, &'static mut Transform), (With<TutorialDome>, Without<LocalPlayer>)>,
}

#[derive(SystemParam)]
pub(super) struct TutorialChoreographyDrive<'w, 's> {
    pub(super) choreography_player: ResMut<'w, TutorialChoreographyPlayer>,
    pub(super) execution: ResMut<'w, TutorialChoreographyExecution>,
    pub(super) presentation: ResMut<'w, TutorialChoreographyPresentation>,
    pub(super) issues: ResMut<'w, TutorialChoreographyIssueQueue>,
    pub(super) actor_commands: ResMut<'w, TutorialActorCommandQueue>,
    pub(super) actor_registry: Res<'w, TutorialActorRegistry>,
    pub(super) actor_transforms: Query<'w, 's, &'static Transform, With<TutorialActor>>,
    // The portrait renderer is also Camera3d; only the gameplay orbit camera
    // is a valid source for choreography camera expressions and captures.
    pub(super) cameras: Query<'w, 's, &'static Transform, (With<LegacyOrbitCamera>, With<Camera3d>)>,
    pub(super) players: Query<
        'w,
        's,
        (&'static mut Transform, &'static mut LegacyPlayerController),
        (With<LocalPlayer>, Without<Camera3d>, Without<TutorialActor>),
    >,
    pub(super) tutorial: ResMut<'w, TutorialSession>,
    pub(super) runtime: ResMut<'w, RuntimeStatus>,
    pub(super) language: Res<'w, Language>,
    pub(super) voice_subtitles: ResMut<'w, TutorialVoiceSubtitleState>,
    pub(super) effect_runtime: ResMut<'w, TutorialEffectRuntime>,
    pub(super) projectile_random: ResMut<'w, TutorialProjectileRandomStream>,
    pub(super) tutorial_voices: Query<'w, 's, Entity, With<TutorialVoiceAudio>>,
    pub(super) tutorial_music_audio: Query<'w, 's, Entity, With<TutorialMusicAudio>>,
    pub(super) ambient: ResMut<'w, TutorialAmbientRuntime>,
    pub(super) auxiliary_presentation: ResMut<'w, TutorialAuxiliaryPresentation>,
    pub(super) nano_commands: ResMut<'w, TutorialNanoPresentationCommandQueue>,
    pub(super) gameplay_nano_commands: ResMut<'w, TutorialNanoGameplayCommandQueue>,
    pub(super) nano_state: Res<'w, TutorialNanoPresentationState>,
    pub(super) player_commands: ResMut<'w, TutorialPlayerPresentationCommandQueue>,
}

#[derive(SystemParam)]
pub(super) struct TutorialAuxiliaryDriveState<'w> {
    pub(super) mission_runtime: ResMut<'w, TutorialMissionRuntime>,
    pub(super) presentation: ResMut<'w, TutorialAuxiliaryPresentation>,
    pub(super) nanocom_messages: ResMut<'w, NanocomMessageUiModel>,
    pub(super) pending_actor_effects: ResMut<'w, PendingTutorialActorEffects>,
}

#[derive(SystemParam)]
pub(super) struct TutorialExitDrive<'w, 's> {
    pub(super) tutorial: ResMut<'w, TutorialSession>,
    pub(super) tutorial_overlay: ResMut<'w, TutorialOverlayUiModel>,
    pub(super) logic: ResMut<'w, TutorialLogicRuntime>,
    pub(super) native: ResMut<'w, TutorialNativeMechanics>,
    pub(super) choreography_player: ResMut<'w, TutorialChoreographyPlayer>,
    pub(super) choreography_execution: ResMut<'w, TutorialChoreographyExecution>,
    pub(super) mission_runtime: ResMut<'w, TutorialMissionRuntime>,
    pub(super) mission_model: ResMut<'w, MissionUiModel>,
    pub(super) auxiliary_presentation: ResMut<'w, TutorialAuxiliaryPresentation>,
    pub(super) actor_commands: ResMut<'w, TutorialActorCommandQueue>,
    pub(super) nano_commands: ResMut<'w, TutorialNanoPresentationCommandQueue>,
    pub(super) gameplay_nano_commands: ResMut<'w, TutorialNanoGameplayCommandQueue>,
    pub(super) nano_state: ResMut<'w, TutorialNanoPresentationState>,
    pub(super) player_commands: ResMut<'w, TutorialPlayerPresentationCommandQueue>,
    pub(super) pending_actor_effects: ResMut<'w, PendingTutorialActorEffects>,
    pub(super) ambient: ResMut<'w, TutorialAmbientRuntime>,
    pub(super) actors: Query<'w, 's, &'static mut TutorialActor>,
    pub(super) domes: Query<'w, 's, Entity, With<TutorialDome>>,
    pub(super) ambient_audio: Query<'w, 's, Entity, With<TutorialAmbientAudio>>,
    pub(super) music_audio: Query<'w, 's, Entity, With<TutorialMusicAudio>>,
    pub(super) loop_audio: Query<'w, 's, Entity, With<TutorialLoopAudio>>,
    pub(super) players: Query<'w, 's, &'static mut LegacyPlayerController, With<LocalPlayer>>,
}

#[derive(Clone)]
pub(super) struct TutorialChoreographyFrame {
    pub(super) player: Transform,
    pub(super) camera: Transform,
    pub(super) start: Transform,
    pub(super) nano: Option<Transform>,
}

impl TutorialLogicRuntime {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn tick(&mut self, delta_seconds: f32) {
        self.reminder_elapsed_seconds += delta_seconds.max(0.0);
        if let Some(remaining) = &mut self.delay_remaining_seconds {
            *remaining = (*remaining - delta_seconds.max(0.0)).max(0.0);
        }
    }

    pub(super) fn restart_reminder(&mut self) {
        self.reminder_elapsed_seconds = 0.0;
    }

    pub(super) fn delay_completed(&self) -> bool {
        self.delay_remaining_seconds
            .is_some_and(|remaining| remaining <= 0.0)
    }

    pub(super) fn defer(&mut self, intent: TutorialIntent) {
        if !self.deferred_intents.contains(&intent) {
            self.deferred_intents.push(intent);
        }
    }
}

impl TutorialSession {
    pub(super) fn character(&self) -> Option<&CharacterSummary> {
        self.character.as_ref()
    }

    pub(super) fn owns_local_hp(&self) -> bool {
        self.character.is_some() && !self.completion_requested
    }

    pub(super) fn clear(&mut self) {
        self.character = None;
        self.progress = TutorialProgress::default();
        self.scene = TutorialScene::None;
        self.reset_runtime_observation();
    }

    pub(super) fn reset_runtime_observation(&mut self) {
        self.observed_stage = None;
        self.step_elapsed = 0.0;
        self.step_origin_position = Vec3::ZERO;
        self.step_origin_yaw = 0.0;
        self.jump_seen = false;
        self.completion_requested = false;
        self.exit_teardown_applied = false;
        self.presentation_cursor = 0;
    }

    pub(super) fn init_step(&mut self, step: i16) {
        self.progress.init_step(step);
        self.observed_stage = None;
        self.step_elapsed = 0.0;
        self.jump_seen = false;
        self.presentation_cursor = 0;
    }

    pub(super) fn write_step_preserving_state(&mut self, step: i16) {
        self.progress.write_step_preserving_state(step);
        self.observed_stage = None;
        self.step_elapsed = 0.0;
        self.jump_seen = false;
        self.presentation_cursor = 0;
    }

    pub(super) fn init_chapter(&mut self, chapter: u8) -> Result<(), u8> {
        self.progress.init_chapter(chapter)?;
        self.observed_stage = None;
        self.step_elapsed = 0.0;
        self.jump_seen = false;
        self.presentation_cursor = 0;
        Ok(())
    }
}
