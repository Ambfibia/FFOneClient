//! Transportation (skyway/broomstick) production runtime and outboxes.

use super::npc_warp::{TRANSPORTATION_WARP_PRESENTATION, WarpDepartureClock};
use super::option_runtime::{OptionProductionRuntime, option_channel_gain};
use super::runtime_status::RuntimeStatus;
use super::world_intents::queue_service_farewell;
use super::{LocalPlayer, WorldSliceEntity};
use bevy::{
    audio::Volume,
    ecs::system::SystemParam,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use ffone_client::{
    gameplay_audio::GameplayAudioRuntime,
    localization::{LocalizedVoice, VoiceLanguage},
    mission_ui::MissionUiModel,
    network::{NetworkBridge, NetworkCommand},
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    system_message_ui::{
        SystemMessageRequest, SystemMessageUiAction, SystemMessageUiModel, SystemMessageUiOutbox,
    },
    transportation_ui::{
        TransportationFade, TransportationInputGates, TransportationInputResult,
        TransportationCloseReason, TransportationModel, TransportationOutboxEvent,
        TransportationPhase,
        TransportationPresentationInput, TransportationRegistrationIntent, TransportationService,
        TransportationSound, TransportationTravelIntent, TransportationUiCommand,
        TransportationUiCommandOutbox,
    },
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
    },
    tutorial_mission_content::TutorialMissionContent,
};
use ffone_protocol::NpcInteractionRequest0104;
use std::collections::BTreeMap;

pub(super) const TRANSPORTATION_SYSTEM_MESSAGE_ID_BASE: u64 = 0x5452_414e_0000_0000;
pub(super) const TRANSPORTATION_LEGACY_SCROLL_VELOCITY: f32 = 200.0;
pub(super) const TRANSPORTATION_BUTTON_SOUND_GAIN: f32 = 0.7;
pub(super) const TRANSPORTATION_ACTION_FAILURE_AUDIO_PATH: &str = "audio/sfx/ui/action_failure01.ogg";
pub(super) const TRANSPORTATION_WARP_AUDIO_PATH: &str = "audio/sfx/world_events/transportation_warp.ogg";
pub(super) const TRANSPORTATION_ACTION_SUCCESS_AUDIO_PATH: &str = "audio/sfx/ui/action_sucess.ogg";
pub(super) const TRANSPORTATION_RIDING_SUCCESS_PACKET_ID_0104: u32 = 0x3100_00d8;
pub(super) const TRANSPORTATION_BROOMSTICK_MOVE_PACKET_ID_0104: u32 = 0x3100_00da;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TransportationSkywayFrame0104 {
    Dismount {
        pc_id: i32,
        riding_type: i32,
    },
    Move {
        pc_id: i32,
        position: [i32; 3],
        speed: i32,
    },
}

pub(super) fn decode_transportation_skyway_frame_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<Option<TransportationSkywayFrame0104>, String> {
    let expected = match packet_id {
        TRANSPORTATION_RIDING_SUCCESS_PACKET_ID_0104 => 8,
        TRANSPORTATION_BROOMSTICK_MOVE_PACKET_ID_0104 => 20,
        _ => return Ok(None),
    };
    if payload.len() != expected {
        return Err(format!(
            "packet {packet_id:#010x} expected {expected} bytes, got {}",
            payload.len()
        ));
    }
    let read_i32 = |offset| {
        i32::from_le_bytes(
            payload[offset..offset + 4]
                .try_into()
                .expect("validated four-byte field"),
        )
    };
    Ok(Some(match packet_id {
        TRANSPORTATION_RIDING_SUCCESS_PACKET_ID_0104 => TransportationSkywayFrame0104::Dismount {
            pc_id: read_i32(0),
            riding_type: read_i32(4),
        },
        TRANSPORTATION_BROOMSTICK_MOVE_PACKET_ID_0104 => TransportationSkywayFrame0104::Move {
            pc_id: read_i32(0),
            position: [read_i32(4), read_i32(8), read_i32(12)],
            speed: read_i32(16),
        },
        _ => unreachable!("packet id checked above"),
    }))
}

#[derive(Debug)]
pub(super) struct TransportationNpcSession {
    pub(super) npc_id: i32,
    pub(super) interaction_close_required: bool,
    pub(super) service: TransportationService,
    pub(super) move_ok_voice_true_names: Option<[String; 3]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationWarpLease0104 {
    pub(super) intent: TransportationTravelIntent,
    pub(super) transportation_type: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SkywayTraversalMotion {
    pub(super) target: Vec3,
    pub(super) segment_speed: f32,
    pub(super) horizontal_direction: Vec3,
}

impl SkywayTraversalMotion {
    pub(super) fn new(position: Vec3) -> Self {
        Self {
            target: position,
            segment_speed: 0.0,
            horizontal_direction: Vec3::ZERO,
        }
    }

    /// Mirrors `cnAvatarThirdPersonMove.SetBroomStick`: finish a badly lagging
    /// prior segment, then derive the next segment speed from its distance.
    pub(super) fn retarget(&mut self, position: &mut Vec3, target: Vec3) {
        if position.distance(self.target) > 5.0 {
            *position = self.target;
        }
        let delta = target - *position;
        self.target = target;
        self.segment_speed = delta.length();
        let horizontal = Vec3::new(delta.x, 0.0, delta.z);
        if horizontal.length_squared() > f32::EPSILON {
            self.horizontal_direction = horizontal.normalize();
        }
    }

    pub(super) fn advance(&self, transform: &mut Transform, delta_seconds: f32) {
        let delta = self.target - transform.translation;
        let remaining = delta.length();
        if remaining > f32::EPSILON {
            let distance = (self.segment_speed * delta_seconds.max(0.0)).min(remaining);
            transform.translation += delta * (distance / remaining);
        }
        if self.horizontal_direction.length_squared() > f32::EPSILON {
            let target_rotation = Transform::IDENTITY
                .looking_to(self.horizontal_direction, Vec3::Y)
                .rotation;
            transform.rotation = transform
                .rotation
                .slerp(target_rotation, (delta_seconds * 4.0).clamp(0.0, 1.0));
        }
    }
}

/// Production-only ownership around the pure `cnTrans` model.
///
/// The model deliberately cannot mutate the avatar, network stream, cursor,
/// or SystemMessage queue. This adapter retains just the correlated NPC edge
/// and transient side-effect state needed to reproduce GameMode 19 without
/// making the presentation authoritative.
#[derive(Debug, Resource)]
pub(super) struct TransportationProductionRuntime {
    pub(super) departure_clock: WarpDepartureClock,
    pub(super) active_npc: Option<TransportationNpcSession>,
    pub(super) pending_registration: Option<TransportationRegistrationIntent>,
    pub(super) pending_warp: Option<TransportationWarpLease0104>,
    pub(super) pending_system_messages: BTreeMap<u64, i32>,
    pub(super) next_system_message_id: u64,
    pub(super) movement_packet_emission: bool,
    pub(super) skyway_active: bool,
    pub(super) skyway_motion: Option<SkywayTraversalMotion>,
    pub(super) camera_subtarget_active: bool,
    pub(super) game_condition_cooldown: Option<i32>,
    pub(super) window_fade_in_requested: bool,
    pub(super) last_handled_packet: Option<u32>,
    pub(super) button_sound_rng: u32,
}

impl Default for TransportationProductionRuntime {
    fn default() -> Self {
        Self {
            departure_clock: WarpDepartureClock::default(),
            active_npc: None,
            pending_registration: None,
            pending_warp: None,
            pending_system_messages: BTreeMap::new(),
            next_system_message_id: TRANSPORTATION_SYSTEM_MESSAGE_ID_BASE,
            movement_packet_emission: true,
            skyway_active: false,
            skyway_motion: None,
            camera_subtarget_active: false,
            game_condition_cooldown: None,
            window_fade_in_requested: false,
            last_handled_packet: None,
            button_sound_rng: 0x7472_616e,
        }
    }
}

impl TransportationProductionRuntime {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn begin_npc(
        &mut self,
        npc_id: i32,
        interaction_close_required: bool,
        service: TransportationService,
        move_ok_voice_true_names: Option<[String; 3]>,
    ) {
        self.active_npc = Some(TransportationNpcSession {
            npc_id,
            interaction_close_required,
            service,
            move_ok_voice_true_names,
        });
        self.pending_warp = None;
        self.movement_packet_emission = true;
        self.skyway_active = false;
        self.skyway_motion = None;
        self.camera_subtarget_active = true;
        self.game_condition_cooldown = None;
        self.window_fade_in_requested = false;
        self.last_handled_packet = None;
    }

    /// The latest NPC owns the registration reply. Already-unlocked stops
    /// are filtered by the caller using the authoritative session flags.
    pub(super) fn reserve_registration(&mut self, intent: TransportationRegistrationIntent) {
        self.pending_registration = Some(intent);
    }

    pub(super) fn release_registration(&mut self, intent: TransportationRegistrationIntent) {
        if self.pending_registration == Some(intent) {
            self.pending_registration = None;
        }
    }

    pub(super) fn take_registration_reply(
        &mut self,
        transportation_type: i32,
        location_id: i32,
    ) -> Option<TransportationRegistrationIntent> {
        let pending = self.pending_registration?;
        if pending.transportation_type != transportation_type || pending.location_id != location_id
        {
            return None;
        }
        self.pending_registration.take()
    }

    pub(super) fn reserve_warp(
        &mut self,
        intent: TransportationTravelIntent,
        service: TransportationService,
    ) -> Result<(), &'static str> {
        if self.pending_warp.is_some() {
            return Err("another transportation request already owns the reply lease");
        }
        let Some(session) = self.active_npc.as_ref() else {
            return Err("the clean NPC initialization target is no longer active");
        };
        if session.npc_id != intent.npc_id || session.service != service {
            return Err("the travel intent does not match the clean NPC initialization target");
        }
        if !matches!(
            service,
            TransportationService::Warp | TransportationService::Wyvern
        ) {
            return Err("the dormant item-use transportation route is not production-reachable");
        }
        self.pending_warp = Some(TransportationWarpLease0104 {
            intent,
            transportation_type: service as i32,
        });
        Ok(())
    }

    pub(super) fn release_warp(&mut self, intent: TransportationTravelIntent) {
        if self
            .pending_warp
            .is_some_and(|lease| lease.intent == intent)
        {
            self.pending_warp = None;
        }
    }

    pub(super) fn take_warp_success(
        &mut self,
        transportation_type: i32,
    ) -> Option<TransportationWarpLease0104> {
        if !self
            .pending_warp
            .is_some_and(|lease| lease.transportation_type == transportation_type)
        {
            return None;
        }
        self.pending_warp.take()
    }

    pub(super) fn take_warp_failure(&mut self, transportation_id: i32) -> Option<TransportationWarpLease0104> {
        if !self
            .pending_warp
            .is_some_and(|lease| lease.intent.transporation_id == transportation_id)
        {
            return None;
        }
        self.pending_warp.take()
    }

    pub(super) fn queue_system_message(
        &mut self,
        content: &TutorialMissionContent,
        messages: &mut SystemMessageUiModel,
        legacy_message_id: i32,
    ) -> Result<u64, String> {
        let definition = content
            .system_message_definition(legacy_message_id)
            .ok_or_else(|| {
                format!(
                    "TransportMode SystemMessage {legacy_message_id} has no source TableData row"
                )
            })?;
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < TRANSPORTATION_SYSTEM_MESSAGE_ID_BASE {
            self.next_system_message_id = TRANSPORTATION_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending_system_messages
            .insert(request_id, legacy_message_id);
        messages.push(SystemMessageRequest::new(
            request_id,
            definition.exact_text.clone(),
            definition.runtime_button_type,
        ));
        Ok(request_id)
    }

    pub(super) fn next_button_sound_path(&mut self) -> &'static str {
        // Unity chooses mouse_click01..05 from its shared global Random. The
        // native client keeps a dedicated deterministic stream so it uses the
        // exact family without perturbing gameplay random decisions.
        self.button_sound_rng ^= self.button_sound_rng << 13;
        self.button_sound_rng ^= self.button_sound_rng >> 17;
        self.button_sound_rng ^= self.button_sound_rng << 5;
        const PATHS: [&str; 5] = [
            "audio/sfx/ui/mouse_click01.ogg",
            "audio/sfx/ui/mouse_click02.ogg",
            "audio/sfx/ui/mouse_click03.ogg",
            "audio/sfx/ui/mouse_click04.ogg",
            "audio/sfx/ui/mouse_click05.ogg",
        ];
        PATHS[self.button_sound_rng as usize % PATHS.len()]
    }

    pub(super) fn next_move_voice_index(&mut self) -> usize {
        self.button_sound_rng ^= self.button_sound_rng << 13;
        self.button_sound_rng ^= self.button_sound_rng >> 17;
        self.button_sound_rng ^= self.button_sound_rng << 5;
        self.button_sound_rng as usize % 3
    }
}

pub(super) fn transportation_move_ok_voice_set(
    catalog: &NativeAudioCatalog,
    owner: &str,
) -> Option<[String; 3]> {
    // The clean cat-15/16 data can point at the generic `mvehicle` comment
    // owner, but the primary audio catalog contains no matching MoveOK takes.
    // Treat it as the source's silent branch instead of inventing aliases.
    if owner.is_empty() || owner.eq_ignore_ascii_case("mvehicle") {
        return None;
    }
    let true_names = [1, 2, 3].map(|take| format!("{owner}_ClickMove0{take}"));
    true_names
        .iter()
        .all(|true_name| {
            let matches = catalog
                .by_true_name(true_name)
                .into_iter()
                .filter(|asset| asset.category == NativeAudioCategory::Voice)
                .count();
            matches == 1
        })
        .then_some(true_names)
}

pub(super) const fn transportation_npc_service(
    service_category: i32,
    catalog_class: Option<i32>,
) -> Option<TransportationService> {
    match (service_category, catalog_class) {
        (15, Some(15)) => Some(TransportationService::Warp),
        (16, Some(16)) => Some(TransportationService::Wyvern),
        _ => None,
    }
}

pub(super) fn sync_transportation_presentation_input(
    system_messages: Res<SystemMessageUiModel>,
    mission_ui: Res<MissionUiModel>,
    mut input: ResMut<TransportationPresentationInput>,
) {
    input.gates = TransportationInputGates {
        system_popup_open: system_messages.is_popup() || mission_ui.system_popup_active(),
        // Clean `cnTrans.Update` asks GameFrame `(2, 24)` and closes when its
        // returned integer is one. FFOne has no competing Computress owner in
        // this mode, so the idle production route has that accepted result.
        escape_close_allowed: true,
    };
    // `InventoryManagerScript` initializes its shared `scrollVel` to 200f.
    input.legacy_scroll_velocity = TRANSPORTATION_LEGACY_SCROLL_VELOCITY;
}

pub(super) fn consume_transportation_ui_commands(
    mut model: ResMut<TransportationModel>,
    mut production: ResMut<TransportationProductionRuntime>,
    mut commands: ResMut<TransportationUiCommandOutbox>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(command) = commands.pop() {
        let result = match command {
            TransportationUiCommand::SelectRoute(index) => model.select_route(index),
            TransportationUiCommand::GoNow => model.press_go(),
            TransportationUiCommand::ToggleTurbo => model.toggle_turbo(),
            TransportationUiCommand::CloseButton(gates) => Ok(model.close_button(gates)),
            TransportationUiCommand::Escape(gates) => Ok(model.escape(gates)),
            TransportationUiCommand::ScrollAxis {
                axis,
                legacy_scroll_velocity,
            } => model.apply_scroll_axis(axis, legacy_scroll_velocity),
        };
        if matches!(result, Ok(TransportationInputResult::BeganWarp)) {
            production.departure_clock = WarpDepartureClock::default();
        }
        if let Err(error) = result {
            status.message = format!("TransportMode input rejected: {error:?}");
        }
    }
}

pub(super) fn advance_transportation_production(
    time: Res<Time>,
    effects: Res<TutorialEffectRuntime>,
    bridge: Res<NetworkBridge>,
    mut model: ResMut<TransportationModel>,
    mut production: ResMut<TransportationProductionRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    if model.phase() == TransportationPhase::Browsing
        && let Err(error) = model.advance_map_paint()
    {
        status.message = format!("TransportMode map projection rejected: {error:?}");
    }
    let delta = if model.phase() == TransportationPhase::PendingWarp {
        production.departure_clock.delta(
            effects.named_native_presentation_ready(TRANSPORTATION_WARP_PRESENTATION),
            effects.has_named_native_instance(TRANSPORTATION_WARP_PRESENTATION),
            time.delta_secs(),
        )
    } else {
        production.departure_clock = WarpDepartureClock::default();
        0.0
    };
    if let Err(error) = model.advance(delta) {
        status.message = format!("TransportMode delayed warp rejected: {error:?}");
    }
    while let Some(intent) = model.pop_travel_intent() {
        let service = model.service();
        if let Err(reason) = production.reserve_warp(intent, service) {
            status.message = format!("TransportMode request rejected: {reason}");
            continue;
        }
        let request = match intent.encode_registered() {
            Ok(request) => request,
            Err(error) => {
                production.release_warp(intent);
                status.message = format!("TransportMode request codec rejected: {error}");
                continue;
            }
        };
        match bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)) {
            Ok(()) => {
                status.message = format!(
                    "TransportMode request sent for transportation {}{}",
                    intent.transporation_id,
                    if intent.turbo { " with turbo" } else { "" }
                );
            }
            Err(error) => {
                production.release_warp(intent);
                status.message = format!("TransportMode request transport failed: {error}");
            }
        }
    }
}

#[derive(SystemParam)]
pub(super) struct TransportationProductionOwners<'w, 's> {
    pub(super) production: ResMut<'w, TransportationProductionRuntime>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) system_messages: ResMut<'w, SystemMessageUiModel>,
    pub(super) mission_ui: ResMut<'w, MissionUiModel>,
    pub(super) effects: ResMut<'w, TutorialEffectRuntime>,
    pub(super) audio_catalog: Res<'w, NativeAudioCatalog>,
    pub(super) voice_language: Res<'w, VoiceLanguage>,
    pub(super) option_runtime: Res<'w, OptionProductionRuntime>,
    pub(super) bridge: Res<'w, NetworkBridge>,
    pub(super) status: ResMut<'w, RuntimeStatus>,
    pub(super) players: Query<'w, 's, (Entity, &'static GlobalTransform), With<LocalPlayer>>,
    pub(super) cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}

pub(super) fn set_transportation_cursor_locked(
    cursors: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
    locked: bool,
) {
    if let Ok(mut cursor) = cursors.single_mut() {
        cursor.grab_mode = if locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !locked;
    }
}

pub(super) fn consume_transportation_model_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<TransportationModel>,
    mut ui_audio: ResMut<GameplayAudioRuntime>,
    mut owners: TransportationProductionOwners,
) {
    while let Some(event) = model.pop_outbox() {
        match event {
            TransportationOutboxEvent::SetCursorLocked(locked)
            | TransportationOutboxEvent::RestoreCursorLocked(locked) => {
                set_transportation_cursor_locked(&mut owners.cursors, locked);
            }
            TransportationOutboxEvent::SetTransportCameraTarget(npc_id) => {
                owners.production.camera_subtarget_active = owners
                    .production
                    .active_npc
                    .as_ref()
                    .is_some_and(|session| session.npc_id == npc_id);
            }
            TransportationOutboxEvent::SetGameConditionCooldown(condition) => {
                // Retain the exact `SetGameCondition(8)` edge. The active
                // modal already blocks ordinary avatar actions; this field is
                // the explicit hand-off point for a future shared owner.
                owners.production.game_condition_cooldown = Some(condition);
            }
            TransportationOutboxEvent::EndCameraSubTarget => {
                owners.production.camera_subtarget_active = false;
            }
            TransportationOutboxEvent::InstantiatePlayerEffect(effect_id) => {
                owners.production.departure_clock = WarpDepartureClock::default();
                let Ok((player, transform)) = owners.players.single() else {
                    owners.status.message = format!(
                        "TransportMode effect {effect_id} blocked without a unique local player"
                    );
                    continue;
                };
                owners.effects.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id,
                    placement: TutorialEffectPlacement::ExactEntityWorld {
                        root_entity: player,
                        position: transform.translation(),
                        rotation: transform.rotation(),
                    },
                    scale: 1.0,
                    tracked: false,
                    name: Some(TRANSPORTATION_WARP_PRESENTATION.to_owned()),
                    destroy_after_seconds: None,
                    source_line: line!(),
                });
            }
            TransportationOutboxEvent::PlayNpcMoveOkVoice(npc_id) => {
                let Some(true_names) = owners
                    .production
                    .active_npc
                    .as_ref()
                    .filter(|session| session.npc_id == npc_id)
                    .and_then(|session| session.move_ok_voice_true_names.clone())
                else {
                    // `mvehicle` and empty clean comment owners are an exact
                    // silent branch. The model normally suppresses this event;
                    // this defensive path also refuses a stale/fabricated cue.
                    continue;
                };
                let true_name = true_names[owners.production.next_move_voice_index()].clone();
                let candidates = owners
                    .audio_catalog
                    .by_true_name(&true_name)
                    .into_iter()
                    .filter(|asset| asset.category == NativeAudioCategory::Voice)
                    .collect::<Vec<_>>();
                let [asset] = candidates.as_slice() else {
                    owners.status.message = format!(
                        "TransportMode MoveOK voice {true_name:?} resolved to {} native voice assets",
                        candidates.len()
                    );
                    continue;
                };
                let Some(path) = owners
                    .audio_catalog
                    .path_for_locale(asset, &owners.voice_language.effective)
                    .map(str::to_owned)
                else {
                    continue;
                };
                let gain = option_channel_gain(owners.option_runtime.options.sound.voice);
                commands.spawn((
                    Name::new(format!(
                        "TransportMode NPC {npc_id} MoveOK voice {true_name}"
                    )),
                    WorldSliceEntity,
                    LocalizedVoice::by_true_name(true_name),
                    ffone_client::audio_channel::GameplayAudioChannel::new(NativeAudioCategory::Voice, 1.0),
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
                ));
            }
            TransportationOutboxEvent::PlaySound(sound) => {
                let (path, clean_gain) = match sound {
                    TransportationSound::Button => (
                        owners.production.next_button_sound_path(),
                        TRANSPORTATION_BUTTON_SOUND_GAIN,
                    ),
                    TransportationSound::ActionFailure => {
                        (TRANSPORTATION_ACTION_FAILURE_AUDIO_PATH, 0.7)
                    }
                    TransportationSound::TransportationWarp => {
                        (TRANSPORTATION_WARP_AUDIO_PATH, 0.7)
                    }
                    TransportationSound::ActionSuccess => {
                        (TRANSPORTATION_ACTION_SUCCESS_AUDIO_PATH, 0.7)
                    }
                };
                let gain =
                    clean_gain * option_channel_gain(owners.option_runtime.options.sound.effects);
                commands.spawn((
                    Name::new(format!("TransportMode audio {path}")),
                    WorldSliceEntity,
                    ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
                ));
            }
            TransportationOutboxEvent::SystemMessage(message_id) => {
                if let Err(error) = owners.production.queue_system_message(
                    &owners.content,
                    &mut owners.system_messages,
                    message_id,
                ) {
                    owners.status.message = error;
                }
            }
            TransportationOutboxEvent::SetMovementPacketEmission(enabled) => {
                owners.production.movement_packet_emission = enabled;
            }
            TransportationOutboxEvent::Fade(TransportationFade::WindowIn) => {
                // There is no generic production WindowIn overlay owner yet;
                // retain the exact request and let the authoritative teleport
                // enter the real world-loading cover rather than faking one.
                owners.production.window_fade_in_requested = true;
            }
            TransportationOutboxEvent::ExitMode { reason } => {
                ui_audio.queue_gameplay_ui_sound("Close_Screen");
                if reason == TransportationCloseReason::UserClose
                    && let Some(session) = owners.production.active_npc.as_ref()
                {
                    queue_service_farewell(&mut commands, session.npc_id);
                }
                owners.production.camera_subtarget_active = false;
                owners.production.movement_packet_emission = !owners.production.skyway_active;
                owners.production.game_condition_cooldown = None;
                owners.production.pending_warp = None;
                if let Some(session) = owners.production.active_npc.take()
                    && session.interaction_close_required
                    && let Err(error) = owners.bridge.send(NetworkCommand::InteractWithNpc(
                        NpcInteractionRequest0104 {
                            npc_id: session.npc_id,
                            flag: 0,
                        },
                    ))
                {
                    owners.status.message =
                        format!("TransportMode interaction-close send failed: {error}");
                }
                owners.mission_ui.clear_npc_interaction_locally();
            }
            TransportationOutboxEvent::MarkPacketHandled(packet_id) => {
                owners.production.last_handled_packet = Some(packet_id);
            }
        }
    }
}

pub(super) fn consume_transportation_system_message_outbox(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut production: ResMut<TransportationProductionRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    if production.pending_system_messages.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for action in system_outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen { request_id, .. } = action;
        let Some(message_id) = production.pending_system_messages.remove(&request_id) else {
            unrelated.push(action);
            continue;
        };
        status.message = format!("TransportMode SystemMessage {message_id} dismissed");
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}

pub(super) fn reset_transportation_shell(
    model: &mut TransportationModel,
    outbox: &mut TransportationUiCommandOutbox,
    production: &mut TransportationProductionRuntime,
) {
    *model = TransportationModel::default();
    while outbox.pop().is_some() {}
    production.reset();
}

pub(super) fn reset_transportation_session(
    mut model: ResMut<TransportationModel>,
    mut outbox: ResMut<TransportationUiCommandOutbox>,
    mut production: ResMut<TransportationProductionRuntime>,
) {
    reset_transportation_shell(&mut model, &mut outbox, &mut production);
}
