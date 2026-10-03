//! Guide production runtime, UI context/outboxes, guide warps and mentor replies.

use super::npc_warp::{NORMAL_NPC_WARP_EFFECT_ID, WarpDepartureClock};
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use bevy::prelude::*;
use ffone_client::{
    entity_lifecycle::NetworkNpcAppearance0104,
    game_guide_ui::GameGuideUiModel,
    gameplay_audio::GameplayAudioRuntime,
    gameplay_ui::GameplayUiModel,
    guide_runtime::{GuidePostChangeIntent, GuideRuntime},
    guide_ui::{
        GUIDE_ALREADY_CURRENT_MESSAGE, GUIDE_CHANGE_FAILURE_MESSAGE, GuideChangeSuccess,
        GuideMentor, GuideUiAction, GuideUiAudioOutbox, GuideUiCommand, GuideUiDismissalSource,
        GuideUiModel, GuideUiOutbox, apply_guide_ui_command,
        resolve_correlated_guide_change_success, resolve_guide_change_failure,
    },
    network::{NetworkBridge, NetworkCommand},
    resurrect_ui::ResurrectUiModel,
    system_message_ui::{
        SystemMessageButtonType, SystemMessageRequest, SystemMessageUiAction, SystemMessageUiModel,
        SystemMessageUiOutbox,
    },
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
    },
    tutorial_mission_content::TutorialMissionContent,
};
use ffone_protocol::{
    DecodedFrame, PcChangeMentorFailure0104, PcChangeMentorSuccess0104,
    PcSpecialStateSwitchRequest0104, PcWarpUseNpcRequest0104, WirePayload, packet,
};
use std::collections::BTreeMap;

pub(super) const GUIDE_FIRST_WARP_DELAY_SECONDS: f32 = 1.5;
pub(super) const GUIDE_FIRST_WARP_ID: i32 = 76;
pub(super) const GUIDE_WARP_FAILURE_MESSAGE_ID: i32 = 173;
pub(super) const GUIDE_SYSTEM_MESSAGE_ID_BASE: u64 = 0x4755_4944_0000_0000;
pub(super) const PAST_WARP_UPSELL_LEVEL: i32 = 4;
pub(super) const GUIDE_FIRST_WARP_PRESENTATION: &str = "local-player.time-machine.departure";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ActiveGuideSourceNpc {
    pub(super) runtime_npc_id: i32,
    pub(super) table_npc_id: i32,
    pub(super) ai_type: i32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingGuideWarp {
    pub(super) npc_id: i32,
    pub(super) map_number: i32,
    pub(super) target: [i32; 3],
    pub(super) elapsed_seconds: f32,
    pub(super) sent: bool,
}

#[derive(Debug, Resource)]
pub(super) struct GuideProductionRuntime {
    pub(super) payment_flag: Option<i8>,
    pub(super) active_source_npc: Option<ActiveGuideSourceNpc>,
    pub(super) special_state_active: bool,
    pub(super) pending_warp: Option<PendingGuideWarp>,
    pub(super) departure_clock: WarpDepartureClock,
    pub(super) pending_system_messages: BTreeMap<u64, i32>,
    pub(super) next_system_message_id: u64,
}

impl Default for GuideProductionRuntime {
    fn default() -> Self {
        Self {
            payment_flag: None,
            active_source_npc: None,
            special_state_active: false,
            pending_warp: None,
            departure_clock: WarpDepartureClock::default(),
            pending_system_messages: BTreeMap::new(),
            next_system_message_id: GUIDE_SYSTEM_MESSAGE_ID_BASE,
        }
    }
}

impl GuideProductionRuntime {
    pub(super) fn reset_world(&mut self) {
        self.active_source_npc = None;
        self.special_state_active = false;
        self.pending_warp = None;
        self.departure_clock = WarpDepartureClock::default();
        self.pending_system_messages.clear();
        self.next_system_message_id = GUIDE_SYSTEM_MESSAGE_ID_BASE;
    }

    pub(super) fn reset_session(&mut self) {
        self.payment_flag = None;
        self.reset_world();
    }

    pub(super) fn push_system_message(
        &mut self,
        messages: &mut SystemMessageUiModel,
        legacy_message_id: i32,
        text: impl Into<String>,
    ) {
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < GUIDE_SYSTEM_MESSAGE_ID_BASE {
            self.next_system_message_id = GUIDE_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending_system_messages
            .insert(request_id, legacy_message_id);
        messages.push(SystemMessageRequest::new(
            request_id,
            text,
            SystemMessageButtonType::Ok,
        ));
    }

    #[must_use]
    pub(super) fn modal_active(&self, model: &GuideUiModel) -> bool {
        model.visible || self.pending_warp.is_some()
    }
}

pub(super) fn reset_guide_shell(
    model: &mut GuideUiModel,
    outbox: &mut GuideUiOutbox,
    audio: &mut GuideUiAudioOutbox,
    runtime: &mut GuideRuntime,
    production: &mut GuideProductionRuntime,
    clear_login_metadata: bool,
) {
    model.close();
    outbox.clear();
    audio.clear();
    runtime.clear_pc_state();
    if clear_login_metadata {
        production.reset_session();
    } else {
        production.reset_world();
    }
}

pub(super) fn switch_guide_special_state(
    bridge: &NetworkBridge,
    player_id: Option<i32>,
    production: &mut GuideProductionRuntime,
    active: bool,
) -> Result<(), String> {
    if production.special_state_active == active {
        return Ok(());
    }
    let player_id =
        player_id.ok_or_else(|| "Guide mode has no authoritative local PC ID".to_owned())?;
    bridge.send(NetworkCommand::SwitchSpecialState(
        PcSpecialStateSwitchRequest0104 {
            pc_id: player_id,
            special_state_flag: 16,
        },
    ))?;
    // Clean sends the same switch flag on both entry and exit. This boolean
    // tracks request ownership only; the authoritative bitfield still comes
    // from the shard's special-state reply.
    production.special_state_active = active;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_guide_ui_context(
    state: Res<State<ClientState>>,
    gameplay_ui: Res<GameplayUiModel>,
    system_messages: Res<SystemMessageUiModel>,
    help: Res<GameGuideUiModel>,
    resurrect_ui: Res<ResurrectUiModel>,
    bridge: Res<NetworkBridge>,
    mut status: ResMut<RuntimeStatus>,
    mut runtime: ResMut<GuideRuntime>,
    mut model: ResMut<GuideUiModel>,
    mut outbox: ResMut<GuideUiOutbox>,
    mut audio: ResMut<GuideUiAudioOutbox>,
    mut production: ResMut<GuideProductionRuntime>,
) {
    model.set_ui_scale(gameplay_ui.ui_scale);
    model.set_external_modes(system_messages.is_popup(), help.modal_active());

    if *state.get() != ClientState::World {
        model.close();
        outbox.clear();
        audio.clear();
        runtime.cancel_pending_change();
        production.reset_world();
        return;
    }

    // ResurrectMode owns the higher-priority death surface. End Guide's
    // special-state lifecycle and discard only local in-flight modal intent.
    if resurrect_ui.visible || status.hp.is_some_and(|hp| hp <= 0) {
        if let Err(error) =
            switch_guide_special_state(&bridge, status.player_id, &mut production, false)
        {
            status.message = format!("Guide mode exit during death failed: {error}");
        }
        model.close();
        outbox.clear();
        audio.clear();
        runtime.cancel_pending_change();
        production.active_source_npc = None;
        production.pending_warp = None;
    }
}

pub(super) fn capture_guide_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut model: ResMut<GuideUiModel>,
    mut outbox: ResMut<GuideUiOutbox>,
    mut audio: ResMut<GuideUiAudioOutbox>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        apply_guide_ui_command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::Dismiss(GuideUiDismissalSource::EscapeKey),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_guide_ui_outbox(
    bridge: Res<NetworkBridge>,
    content: Res<TutorialMissionContent>,
    npc_appearances: Query<&NetworkNpcAppearance0104>,
    local_player: Query<(Entity, &Transform), With<super::LocalPlayer>>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut model: ResMut<GuideUiModel>,
    mut outbox: ResMut<GuideUiOutbox>,
    mut guide_runtime: ResMut<GuideRuntime>,
    mut production: ResMut<GuideProductionRuntime>,
    mut system_messages: ResMut<SystemMessageUiModel>,
    mut status: ResMut<RuntimeStatus>,
    mut help: ResMut<GameGuideUiModel>,
    mut audio: Option<ResMut<GameplayAudioRuntime>>,
) {
    while let Some(action) = outbox.pop_front() {
        match action {
            GuideUiAction::WarpWarningAccepted => {
                status.message = "Guide selection opened from the Past Warp warning".to_owned();
            }
            GuideUiAction::Dismissed { .. } => {
                let exit =
                    switch_guide_special_state(&bridge, status.player_id, &mut production, false);
                let source = production.active_source_npc.take();
                match (exit, source) {
                    (Err(error), _) => {
                        status.message = format!("Guide mode exit failed: {error}");
                    }
                    (Ok(()), Some(source)) => {
                        status.message = format!(
                            "Guide mode closed for runtime NPC {} (table {}, AI type {})",
                            source.runtime_npc_id, source.table_npc_id, source.ai_type
                        );
                    }
                    (Ok(()), None) => {
                        status.message = "Guide mode closed".to_owned();
                    }
                }
            }
            GuideUiAction::HelpRequested {
                receiver,
                event_group,
                event_function,
            } => {
                if event_group == 2
                    && event_function == 21
                    && usize::try_from(receiver).is_ok_and(|id| help.open_first_use(id))
                {
                    model.set_external_modes(system_messages.is_popup(), true);
                    if let Some(audio) = audio.as_mut() {
                        audio.queue_gameplay_ui_sound("Open_Screen");
                    }
                }
            }
            GuideUiAction::CurrentMentorRejected {
                system_message_id, ..
            } => {
                production.push_system_message(
                    &mut system_messages,
                    system_message_id,
                    GUIDE_ALREADY_CURRENT_MESSAGE,
                );
            }
            GuideUiAction::ChangeMentorRequested { mentor } => {
                let request = match guide_runtime.request_change(mentor) {
                    Ok(request) => request,
                    Err(error) => {
                        model.cancel_pending_request();
                        status.message =
                            format!("Guide change request failed correlation: {error:?}");
                        continue;
                    }
                };
                if let Err(error) = bridge.send(NetworkCommand::ChangeMentor(request)) {
                    guide_runtime.cancel_pending_change();
                    model.cancel_pending_request();
                    status.message = format!("Guide change transport failed: {error}");
                } else {
                    status.message =
                        format!("Requesting {} as Guide from OpenFusion...", mentor.name());
                }
            }
            GuideUiAction::FirstMentorChangeSucceeded {
                warp_npc_table_id, ..
            } => {
                if let Err(error) =
                    switch_guide_special_state(&bridge, status.player_id, &mut production, false)
                {
                    status.message = format!("Guide first-change exit failed: {error}");
                }
                production.active_source_npc = None;
                let Some(warp) = content.gameplay_warp(GUIDE_FIRST_WARP_ID) else {
                    status.message = format!(
                        "Guide first-change warp {} is absent from validated TableData",
                        GUIDE_FIRST_WARP_ID
                    );
                    production.pending_warp = None;
                    continue;
                };
                if warp.npc_type != warp_npc_table_id {
                    status.message = format!(
                        "Guide first-change NPC mismatch: UI requested table {warp_npc_table_id}, warp row owns {}",
                        warp.npc_type
                    );
                    production.pending_warp = None;
                    continue;
                }
                let mut matches = npc_appearances
                    .iter()
                    .filter(|appearance| appearance.0.npc_type == warp_npc_table_id);
                let target = matches.next().map(|appearance| appearance.0.npc_id);
                let duplicate = matches.next().is_some();
                production.pending_warp = if duplicate {
                    status.message = format!(
                        "Guide first-change warp rejected duplicate live NPC table {warp_npc_table_id}"
                    );
                    None
                } else {
                    target.map(|npc_id| PendingGuideWarp {
                        npc_id,
                        map_number: warp.target.map_id,
                        target: [warp.target.x, warp.target.y, warp.target.z],
                        elapsed_seconds: 0.0,
                        sent: false,
                    })
                };
                if target.is_none() && !duplicate {
                    // Clean closes silently when FindNPC cannot resolve 1425.
                    status.message = format!(
                        "Guide first-change completed; live warp NPC table {warp_npc_table_id} was not present"
                    );
                }
                if production.pending_warp.is_some() {
                    let Ok((player_entity, transform)) = local_player.single() else {
                        production.pending_warp = None;
                        status.message =
                            "Guide first-change warp requires one local player".to_owned();
                        continue;
                    };
                    if let Some(audio) = audio.as_deref_mut() {
                        audio.queue_legacy_world_sound(player_entity, super::npc_warp::NORMAL_NPC_WARP_SOUND_TRUE_NAME);
                    }
                    effects.enqueue(TutorialEffectRuntimeCommand::Preload {
                        effect_id: NORMAL_NPC_WARP_EFFECT_ID,
                        source_line: line!(),
                    });
                    effects.enqueue(TutorialEffectRuntimeCommand::Add {
                        effect_id: NORMAL_NPC_WARP_EFFECT_ID,
                        placement: TutorialEffectPlacement::World {
                            position: transform.translation,
                            rotation: transform.rotation,
                        },
                        scale: 1.0,
                        tracked: false,
                        name: Some(GUIDE_FIRST_WARP_PRESENTATION.to_owned()),
                        destroy_after_seconds: None,
                        source_line: line!(),
                    });
                    production.departure_clock = WarpDepartureClock::default();
                }
            }
            GuideUiAction::MentorChangeSucceeded {
                mentor,
                refresh_guide_missions,
                ..
            } => {
                if let Err(error) =
                    switch_guide_special_state(&bridge, status.player_id, &mut production, false)
                {
                    status.message = format!("Guide change exit failed: {error}");
                }
                production.active_source_npc = None;
                status.message = if refresh_guide_missions {
                    format!(
                        "{} is now the authoritative Guide; world Guide-mission refresh remains unavailable",
                        mentor.name()
                    )
                } else {
                    format!("{} is now the authoritative Guide", mentor.name())
                };
            }
            GuideUiAction::MentorChangeFailed {
                system_message_id, ..
            } => {
                if let Err(error) =
                    switch_guide_special_state(&bridge, status.player_id, &mut production, false)
                {
                    status.message = format!("Guide failure exit failed: {error}");
                }
                production.active_source_npc = None;
                production.push_system_message(
                    &mut system_messages,
                    system_message_id,
                    GUIDE_CHANGE_FAILURE_MESSAGE,
                );
            }
        }
    }
}

pub(super) fn advance_pending_guide_warp(
    time: Res<Time>,
    effects: Res<TutorialEffectRuntime>,
    bridge: Res<NetworkBridge>,
    mut production: ResMut<GuideProductionRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    let Some(mut pending) = production.pending_warp else {
        return;
    };
    if pending.sent {
        return;
    }
    pending.elapsed_seconds += production.departure_clock.delta(
        effects.named_native_presentation_ready(GUIDE_FIRST_WARP_PRESENTATION),
        effects.has_named_native_instance(GUIDE_FIRST_WARP_PRESENTATION),
        time.delta_secs(),
    );
    if pending.elapsed_seconds < GUIDE_FIRST_WARP_DELAY_SECONDS {
        production.pending_warp = Some(pending);
        return;
    }
    let request = PcWarpUseNpcRequest0104 {
        npc_id: pending.npc_id,
        warp_id: GUIDE_FIRST_WARP_ID,
        e_il_1: 4,
        item_slot_1: 0,
        e_il_2: 4,
        item_slot_2: 0,
    };
    match bridge.send(NetworkCommand::UseNpcWarp(request)) {
        Ok(()) => {
            pending.sent = true;
            production.pending_warp = Some(pending);
            status.message = format!(
                "Guide first-change warp {} sent to OpenFusion",
                GUIDE_FIRST_WARP_ID
            );
        }
        Err(error) => {
            production.pending_warp = None;
            status.message = format!("Guide first-change warp transport failed: {error}");
        }
    }
}

pub(super) fn consume_guide_ui_audio_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut outbox: ResMut<GuideUiAudioOutbox>,
    audio_runtime: Option<ResMut<GameplayAudioRuntime>>,
) {
    let mut audio_runtime = audio_runtime;
    while let Some(cue) = outbox.pop_front() {
        let Some(path) = cue.exact_path() else {
            if let Some(audio_runtime) = audio_runtime.as_deref_mut() {
                audio_runtime.queue_legacy_button_sound();
            }
            continue;
        };
        commands.spawn((
            Name::new(format!("Guide UI audio {path}")),
            ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.7)),
        ));
    }
}

pub(super) fn consume_guide_system_message_outbox(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut production: ResMut<GuideProductionRuntime>,
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
        status.message = format!("Guide SystemMessage {message_id} dismissed");
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GuideMentorReplyOutcome {
    Success {
        mentor: GuideMentor,
        raw_mentor_count: i16,
        fusion_matter: i32,
        intent: GuidePostChangeIntent,
    },
    Failure {
        mentor: GuideMentor,
        error_code: i32,
    },
}

pub(super) fn apply_guide_mentor_reply_transactional(
    frame: &DecodedFrame,
    runtime: &mut GuideRuntime,
    model: &mut GuideUiModel,
    outbox: &mut GuideUiOutbox,
) -> Result<Option<GuideMentorReplyOutcome>, String> {
    match frame.packet_type {
        packet::P_FE2CL_REP_PC_CHANGE_MENTOR_SUCC => {
            let reply = PcChangeMentorSuccess0104::decode(&frame.payload)
                .map_err(|error| format!("malformed protocol-0104 mentor success: {error}"))?;
            let mut next_runtime = runtime.clone();
            let applied = next_runtime
                .accept_success(reply)
                .map_err(|error| format!("uncorrelated mentor success: {error:?}"))?;
            let mut next_model = model.clone();
            let mut staged_outbox = GuideUiOutbox::default();
            let first_change = matches!(applied.intent, GuidePostChangeIntent::WarpToNpc { .. });
            if !resolve_correlated_guide_change_success(
                &mut next_model,
                &mut staged_outbox,
                GuideChangeSuccess {
                    mentor: applied.mentor,
                    mentor_count: applied.raw_mentor_count,
                    fusion_matter: applied.fusion_matter,
                },
                first_change,
            ) {
                return Err(
                    "mentor success matched runtime authority but not the active Guide UI request"
                        .to_owned(),
                );
            }
            *runtime = next_runtime;
            *model = next_model;
            while let Some(action) = staged_outbox.pop_front() {
                outbox.push(action);
            }
            Ok(Some(GuideMentorReplyOutcome::Success {
                mentor: applied.mentor,
                raw_mentor_count: applied.raw_mentor_count,
                fusion_matter: applied.fusion_matter,
                intent: applied.intent,
            }))
        }
        packet::P_FE2CL_REP_PC_CHANGE_MENTOR_FAIL => {
            let reply = PcChangeMentorFailure0104::decode(&frame.payload)
                .map_err(|error| format!("malformed protocol-0104 mentor failure: {error}"))?;
            let mut next_runtime = runtime.clone();
            let failed = next_runtime
                .accept_failure(reply)
                .map_err(|error| format!("uncorrelated mentor failure: {error:?}"))?;
            let mut next_model = model.clone();
            let mut staged_outbox = GuideUiOutbox::default();
            if !resolve_guide_change_failure(
                &mut next_model,
                &mut staged_outbox,
                failed.mentor,
                failed.error_code,
            ) {
                return Err(
                    "mentor failure matched runtime authority but not the active Guide UI request"
                        .to_owned(),
                );
            }
            *runtime = next_runtime;
            *model = next_model;
            while let Some(action) = staged_outbox.pop_front() {
                outbox.push(action);
            }
            Ok(Some(GuideMentorReplyOutcome::Failure {
                mentor: failed.mentor,
                error_code: failed.error_code,
            }))
        }
        _ => Ok(None),
    }
}
