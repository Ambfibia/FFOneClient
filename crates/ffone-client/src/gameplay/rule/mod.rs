//! Production hand-off for clean-Retrobution `eGameMode.Rule`.
//!
//! The clean route is entirely local. `NpcIconMode.StartRule` selects Rules
//! Table page `1` for NPC service number `9` and page `2` for service number
//! `10`, then dispatches GameFrame events `2/0(mode=30)` and
//! `2/3/element-0(page)`. The privileged chat command `/rule <page>` uses the
//! same two events. `cnRule.ReceivePacket` does not consume a packet and none
//! of the open, navigation, close, or Escape paths sends one.
//!
//! This adapter keeps those local event-bus effects typed, correlates the
//! otherwise synchronous Computress close gate, and exposes the exact input,
//! cursor, and camera lease for the Bevy shell. It deliberately does not open
//! a legacy Unity container or depend on an evidence JSON at runtime.

use std::collections::VecDeque;

use bevy::prelude::Resource;

use crate::rule_ui::{
    RULE_UI_BUTTON_SOUND_GAIN, RULE_UI_BUTTON_SOUND_PATHS, RULE_UI_GAME_MODE_VALUE, RulePageId,
    RulePageIndexError, RuleUiAction, RuleUiAudioCue, RuleUiAudioOutbox, RuleUiDismissalSource,
    RuleUiModel, RuleUiOutbox, resolve_rule_ui_escape_close_gate,
};

pub const RULE_RUNTIME_MAIN_GAME_MODE_VALUE: u8 = 5;
pub const RULE_RUNTIME_NPC_VEHICLE_SERVICE_NUMBER: i32 = 9;
pub const RULE_RUNTIME_NPC_COMBINING_SERVICE_NUMBER: i32 = 10;
pub const RULE_RUNTIME_GM_CHAT_MAX_USER_LEVEL: i32 = 50;
pub const RULE_RUNTIME_NPC_LABEL_PREFIX: &str = " ";

pub const RULE_RUNTIME_EVENT_GROUP: u8 = 2;
pub const RULE_RUNTIME_CREATE_MODE_FUNCTION: u8 = 0;
pub const RULE_RUNTIME_EXIT_MODE_FUNCTION: u8 = 1;
pub const RULE_RUNTIME_SEND_TO_ELEMENT_FUNCTION: u8 = 3;
pub const RULE_RUNTIME_MODE_INIT_ELEMENT_FUNCTION: u8 = 0;
pub const RULE_RUNTIME_ESCAPE_GATE_FUNCTION: u8 = 24;

pub const RULE_RUNTIME_OPEN_SOUND_NAME: &str = "Open_Screen";
pub const RULE_RUNTIME_CLOSE_SOUND_NAME: &str = "Close_Screen";

/// The Rule surface has no protocol request/reply owner in the clean client.
pub const RULE_RUNTIME_IS_LOCAL_ONLY: bool = true;
pub const RULE_RUNTIME_SENDS_PACKETS: bool = false;
pub const RULE_RUNTIME_CHANGES_SPECIAL_STATE: bool = false;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleNpcServiceRoute {
    pub service_number: i32,
    pub page: RulePageId,
    pub table_index: i32,
    /// Exact first RulesTable string used by `NpcIconMode` for the button.
    pub label_key: &'static str,
    /// `NpcIconMode` prepends one literal space to every utility-row label.
    pub label_prefix: &'static str,
}

/// Exact `NpcTableElement.m_iServiceNumber -> RulesTable` routing.
#[must_use]
pub const fn rule_npc_service_route(service_number: i32) -> Option<RuleNpcServiceRoute> {
    let page = match service_number {
        RULE_RUNTIME_NPC_VEHICLE_SERVICE_NUMBER => RulePageId::Vehicle,
        RULE_RUNTIME_NPC_COMBINING_SERVICE_NUMBER => RulePageId::Combining,
        _ => return None,
    };
    Some(RuleNpcServiceRoute {
        service_number,
        page,
        table_index: page.table_index() as i32,
        label_key: page.spec().strings[0],
        label_prefix: RULE_RUNTIME_NPC_LABEL_PREFIX,
    })
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleOpenSource {
    /// `NpcIconMode` leaves its camera sub-target alive when it calls
    /// `StartRule`; the runtime NPC identity keeps that ownership explicit.
    NpcService {
        runtime_npc_id: i32,
        table_npc_id: i32,
        service_number: i32,
    },
    /// Local `CnGuiChat.CheckCheatKey` route. Authorization is checked with
    /// the clean `iUserLevel <= 50` condition before the mode is committed.
    GmChatCommand { user_level: i32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleOpenRequest {
    NpcService {
        runtime_npc_id: i32,
        table_npc_id: i32,
        service_number: i32,
    },
    GmChatCommand {
        user_level: i32,
        table_index: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleLegacyOpenDispatch {
    pub event_group: u8,
    pub create_mode_function: u8,
    pub game_mode: u8,
    pub send_to_element_function: u8,
    pub mode_element_function: u8,
    pub table_index: i32,
}

impl RuleLegacyOpenDispatch {
    #[must_use]
    pub const fn for_page(page: RulePageId) -> Self {
        Self {
            event_group: RULE_RUNTIME_EVENT_GROUP,
            create_mode_function: RULE_RUNTIME_CREATE_MODE_FUNCTION,
            game_mode: RULE_UI_GAME_MODE_VALUE,
            send_to_element_function: RULE_RUNTIME_SEND_TO_ELEMENT_FUNCTION,
            mode_element_function: RULE_RUNTIME_MODE_INIT_ELEMENT_FUNCTION,
            table_index: page.table_index() as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleSession {
    pub source: RuleOpenSource,
    pub page: RulePageId,
    pub dispatch: RuleLegacyOpenDispatch,
}

impl RuleSession {
    #[must_use]
    pub const fn source_npc_id(self) -> Option<i32> {
        match self.source {
            RuleOpenSource::NpcService { runtime_npc_id, .. } => Some(runtime_npc_id),
            RuleOpenSource::GmChatCommand { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleEscapeGateToken(u64);

impl RuleEscapeGateToken {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    #[must_use]
    pub const fn from_test_value(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum RuleRuntimePhase {
    #[default]
    Hidden,
    Active,
    AwaitingEscapeGate {
        token: RuleEscapeGateToken,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleCursorPolicy {
    /// `cnRule.Update` forces `Screen.lockCursor=false`; the serialized/default
    /// `bOldLockCursor` was never assigned, so `Exit` also writes `false`.
    ForceUnlockedDuringAndAfterExit,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleCameraPolicy {
    /// `NpcIconMode.StartRule` does not call `EndMode`; GameFrame ends the
    /// sub-target only when event `2/1` returns to linked MainGame.
    PreserveInheritedUntilExitThenEnd,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleTransportPolicy {
    LocalEventBusOnlyNoPackets,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleModeLease {
    pub game_mode: u8,
    pub linked_game_mode: u8,
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub mouse_controls_enabled: bool,
    pub escape_close_gate_enabled: bool,
    pub gameplay_hud_visible: bool,
    pub gameplay_input_enabled: bool,
    pub cursor_policy: RuleCursorPolicy,
    pub camera_policy: RuleCameraPolicy,
    pub transport_policy: RuleTransportPolicy,
    pub changes_special_state: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuleRuntimeSound {
    OpenScreen,
    CloseScreen,
    RandomButtonSound,
}

impl RuleRuntimeSound {
    #[must_use]
    pub const fn legacy_name(self) -> Option<&'static str> {
        match self {
            Self::OpenScreen => Some(RULE_RUNTIME_OPEN_SOUND_NAME),
            Self::CloseScreen => Some(RULE_RUNTIME_CLOSE_SOUND_NAME),
            Self::RandomButtonSound => None,
        }
    }

    #[must_use]
    pub const fn random_candidates(self) -> Option<&'static [&'static str; 5]> {
        match self {
            Self::RandomButtonSound => Some(&RULE_UI_BUTTON_SOUND_PATHS),
            Self::OpenScreen | Self::CloseScreen => None,
        }
    }

    #[must_use]
    pub const fn gain(self) -> f32 {
        match self {
            Self::RandomButtonSound => RULE_UI_BUTTON_SOUND_GAIN,
            Self::OpenScreen | Self::CloseScreen => 0.7,
        }
    }
}

/// One atomic entry hand-off. The shell must retain an inherited NPC camera
/// target and must not send an NPC-interaction close packet for this route.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleEnterModeCommand {
    pub session: RuleSession,
    pub linked_game_mode: u8,
    pub hide_gameplay_hud: bool,
    pub gameplay_input_enabled: bool,
    pub cursor_locked: bool,
    pub preserve_camera_sub_target: bool,
    pub send_npc_interaction_close_packet: bool,
    pub send_special_state_packet: bool,
}

/// Exact local return-to-MainGame obligations produced by `cnRule.Exit` plus
/// `GameFrame.ChangeGameMode`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleExitModeCommand {
    pub source: RuleUiDismissalSource,
    pub event_group: u8,
    pub event_function: u8,
    pub game_mode: u8,
    pub linked_game_mode: u8,
    pub cursor_locked: bool,
    pub show_gameplay_hud: bool,
    pub gameplay_input_enabled: bool,
    pub force_nanocom_open: bool,
    pub end_camera_sub_target: bool,
    pub clear_npc_interaction_locally: bool,
    pub send_npc_interaction_close_packet: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuleRuntimeCommand {
    EnterMode(RuleEnterModeCommand),
    PlaySound(RuleRuntimeSound),
    RequestComputressExitGate {
        token: RuleEscapeGateToken,
        event_group: u8,
        event_function: u8,
    },
    ExitMode(RuleExitModeCommand),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleRuntimeError {
    AlreadyActive,
    UiVisibleWithoutRuntime,
    UnsupportedNpcServiceNumber(i32),
    GmChatNotAuthorized {
        user_level: i32,
    },
    InvalidPageIndex(RulePageIndexError),
    UiActionWhileHidden,
    UiModelOutOfSync,
    UnexpectedUiEventRoute {
        event_group: u8,
        event_function: u8,
    },
    DuplicateEscapeGate,
    EscapeResolutionWithoutRequest,
    StaleEscapeGate {
        expected: RuleEscapeGateToken,
        actual: RuleEscapeGateToken,
    },
    EscapeUiResolutionRejected,
    UnexpectedEscapeExitAction,
}

#[derive(Debug, Resource)]
pub struct RuleRuntime {
    phase: RuleRuntimePhase,
    session: Option<RuleSession>,
    next_escape_gate_token: u64,
    commands: VecDeque<RuleRuntimeCommand>,
}

impl Default for RuleRuntime {
    fn default() -> Self {
        Self {
            phase: RuleRuntimePhase::Hidden,
            session: None,
            next_escape_gate_token: 1,
            commands: VecDeque::new(),
        }
    }
}

impl RuleRuntime {
    #[must_use]
    pub const fn phase(&self) -> RuleRuntimePhase {
        self.phase
    }

    #[must_use]
    pub const fn session(&self) -> Option<RuleSession> {
        self.session
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self.phase, RuleRuntimePhase::Hidden)
    }

    /// Treat either owner as modal if a shell update briefly observes them
    /// between ordered systems. This fails closed instead of leaking input.
    #[must_use]
    pub const fn modal_active(&self, model: &RuleUiModel) -> bool {
        self.is_active() || model.visible
    }

    /// The clean NpcIcon camera stays targeted throughout Rule. Chat-opened
    /// Rule has no source NPC and therefore contributes no camera target.
    #[must_use]
    pub const fn camera_sub_target_npc_id(&self) -> Option<i32> {
        match self.session {
            Some(session) => session.source_npc_id(),
            None => None,
        }
    }

    #[must_use]
    pub fn mode_lease(&self, model: &RuleUiModel) -> Option<RuleModeLease> {
        if !self.is_active() {
            return None;
        }
        let boundary = model.input_boundary();
        Some(RuleModeLease {
            game_mode: RULE_UI_GAME_MODE_VALUE,
            linked_game_mode: RULE_RUNTIME_MAIN_GAME_MODE_VALUE,
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: true,
            mouse_controls_enabled: boundary.mouse_controls_enabled,
            escape_close_gate_enabled: boundary.escape_close_gate_enabled,
            gameplay_hud_visible: false,
            gameplay_input_enabled: false,
            cursor_policy: RuleCursorPolicy::ForceUnlockedDuringAndAfterExit,
            camera_policy: RuleCameraPolicy::PreserveInheritedUntilExitThenEnd,
            transport_policy: RuleTransportPolicy::LocalEventBusOnlyNoPackets,
            changes_special_state: false,
        })
    }

    /// Validate the complete source route before changing the UI or runtime.
    /// Rejections leave the model, phase, session, and command queue intact.
    pub fn open(
        &mut self,
        model: &mut RuleUiModel,
        request: RuleOpenRequest,
    ) -> Result<RuleSession, RuleRuntimeError> {
        if self.is_active() {
            return Err(RuleRuntimeError::AlreadyActive);
        }
        if model.visible {
            return Err(RuleRuntimeError::UiVisibleWithoutRuntime);
        }

        let (source, page) = match request {
            RuleOpenRequest::NpcService {
                runtime_npc_id,
                table_npc_id,
                service_number,
            } => {
                let route = rule_npc_service_route(service_number).ok_or(
                    RuleRuntimeError::UnsupportedNpcServiceNumber(service_number),
                )?;
                (
                    RuleOpenSource::NpcService {
                        runtime_npc_id,
                        table_npc_id,
                        service_number,
                    },
                    route.page,
                )
            }
            RuleOpenRequest::GmChatCommand {
                user_level,
                table_index,
            } => {
                if user_level > RULE_RUNTIME_GM_CHAT_MAX_USER_LEVEL {
                    return Err(RuleRuntimeError::GmChatNotAuthorized { user_level });
                }
                let page = RulePageId::try_from(table_index)
                    .map_err(RuleRuntimeError::InvalidPageIndex)?;
                (RuleOpenSource::GmChatCommand { user_level }, page)
            }
        };

        let session = RuleSession {
            source,
            page,
            dispatch: RuleLegacyOpenDispatch::for_page(page),
        };

        // Commit only after every fallible route check has completed.
        model.open(page);
        self.session = Some(session);
        self.phase = RuleRuntimePhase::Active;
        // `NpcIconMode.OnGUI` calls ButtonSound once for every utility row and
        // then calls it a second time in icon case 23 before `StartRule`.
        // The local GM chat route has neither call.
        if matches!(source, RuleOpenSource::NpcService { .. }) {
            self.commands.push_back(RuleRuntimeCommand::PlaySound(
                RuleRuntimeSound::RandomButtonSound,
            ));
            self.commands.push_back(RuleRuntimeCommand::PlaySound(
                RuleRuntimeSound::RandomButtonSound,
            ));
        }
        self.commands
            .push_back(RuleRuntimeCommand::EnterMode(RuleEnterModeCommand {
                session,
                linked_game_mode: RULE_RUNTIME_MAIN_GAME_MODE_VALUE,
                hide_gameplay_hud: true,
                gameplay_input_enabled: false,
                cursor_locked: false,
                preserve_camera_sub_target: true,
                send_npc_interaction_close_packet: false,
                send_special_state_packet: false,
            }));
        self.commands
            .push_back(RuleRuntimeCommand::PlaySound(RuleRuntimeSound::OpenScreen));
        Ok(session)
    }

    /// Mirror the two global values read by `cnRule.OnGUI`/`Update`.
    pub fn sync_external_gates(
        &self,
        model: &mut RuleUiModel,
        system_popup_active: bool,
        help_active: bool,
    ) {
        model.set_system_popup_active(system_popup_active);
        model.set_help_active(help_active);
    }

    /// Consume passive UI actions and audio. Errors are returned in source
    /// order while valid later entries continue to be translated.
    pub fn flush_ui_outboxes(
        &mut self,
        model: &RuleUiModel,
        ui: &mut RuleUiOutbox,
        audio: &mut RuleUiAudioOutbox,
    ) -> Vec<RuleRuntimeError> {
        let mut errors = Vec::new();
        while let Some(action) = ui.pop_front() {
            if let Err(error) = self.accept_ui_action(model, action) {
                errors.push(error);
            }
        }
        while let Some(cue) = audio.pop_front() {
            match cue {
                RuleUiAudioCue::ButtonSound => self.commands.push_back(
                    RuleRuntimeCommand::PlaySound(RuleRuntimeSound::RandomButtonSound),
                ),
            }
        }
        errors
    }

    #[cfg(test)]
    pub(crate) fn accept_ui_action_for_test(
        &mut self,
        model: &RuleUiModel,
        action: RuleUiAction,
    ) -> Result<(), RuleRuntimeError> {
        self.accept_ui_action(model, action)
    }

    fn accept_ui_action(
        &mut self,
        model: &RuleUiModel,
        action: RuleUiAction,
    ) -> Result<(), RuleRuntimeError> {
        let Some(session) = self.session else {
            return Err(RuleRuntimeError::UiActionWhileHidden);
        };
        match action {
            RuleUiAction::RequestEscapeCloseGate {
                event_group,
                event_function,
            } => {
                if (event_group, event_function)
                    != (RULE_RUNTIME_EVENT_GROUP, RULE_RUNTIME_ESCAPE_GATE_FUNCTION)
                {
                    return Err(RuleRuntimeError::UnexpectedUiEventRoute {
                        event_group,
                        event_function,
                    });
                }
                if !model.visible || !model.escape_close_pending {
                    return Err(RuleRuntimeError::UiModelOutOfSync);
                }
                if matches!(self.phase, RuleRuntimePhase::AwaitingEscapeGate { .. }) {
                    return Err(RuleRuntimeError::DuplicateEscapeGate);
                }
                if self.phase != RuleRuntimePhase::Active {
                    return Err(RuleRuntimeError::UiActionWhileHidden);
                }
                let token = self.allocate_escape_gate_token();
                self.phase = RuleRuntimePhase::AwaitingEscapeGate { token };
                self.commands
                    .push_back(RuleRuntimeCommand::RequestComputressExitGate {
                        token,
                        event_group,
                        event_function,
                    });
                Ok(())
            }
            RuleUiAction::ExitMode {
                source,
                event_group,
                event_function,
                cursor_locked,
            } => {
                if (event_group, event_function)
                    != (RULE_RUNTIME_EVENT_GROUP, RULE_RUNTIME_EXIT_MODE_FUNCTION)
                {
                    return Err(RuleRuntimeError::UnexpectedUiEventRoute {
                        event_group,
                        event_function,
                    });
                }
                if model.visible || cursor_locked {
                    return Err(RuleRuntimeError::UiModelOutOfSync);
                }
                if source == RuleUiDismissalSource::EscapeCloseGate
                    && !matches!(self.phase, RuleRuntimePhase::AwaitingEscapeGate { .. })
                {
                    return Err(RuleRuntimeError::EscapeResolutionWithoutRequest);
                }
                self.commit_exit(session, source);
                Ok(())
            }
        }
    }

    /// Resolve GameFrame `2/24`'s local `iReturn`. `accepted=true` means
    /// `OnComputressExitPressed()` returned false and GameFrame set return 1.
    /// A stale response cannot close a newer Rule session.
    pub fn resolve_escape_gate(
        &mut self,
        model: &mut RuleUiModel,
        token: RuleEscapeGateToken,
        accepted: bool,
    ) -> Result<(), RuleRuntimeError> {
        let Some(session) = self.session else {
            return Err(RuleRuntimeError::EscapeResolutionWithoutRequest);
        };
        let RuleRuntimePhase::AwaitingEscapeGate { token: expected } = self.phase else {
            return Err(RuleRuntimeError::EscapeResolutionWithoutRequest);
        };
        if token != expected {
            return Err(RuleRuntimeError::StaleEscapeGate {
                expected,
                actual: token,
            });
        }
        if !model.visible || !model.escape_close_pending {
            return Err(RuleRuntimeError::UiModelOutOfSync);
        }

        let mut generated = RuleUiOutbox::default();
        if !resolve_rule_ui_escape_close_gate(model, &mut generated, accepted) {
            return Err(RuleRuntimeError::EscapeUiResolutionRejected);
        }
        if !accepted {
            debug_assert!(generated.is_empty());
            self.phase = RuleRuntimePhase::Active;
            return Ok(());
        }

        let expected_action = RuleUiAction::ExitMode {
            source: RuleUiDismissalSource::EscapeCloseGate,
            event_group: RULE_RUNTIME_EVENT_GROUP,
            event_function: RULE_RUNTIME_EXIT_MODE_FUNCTION,
            cursor_locked: false,
        };
        if generated.pop_front() != Some(expected_action) || !generated.is_empty() {
            return Err(RuleRuntimeError::UnexpectedEscapeExitAction);
        }
        self.commit_exit(session, RuleUiDismissalSource::EscapeCloseGate);
        Ok(())
    }

    fn commit_exit(&mut self, session: RuleSession, source: RuleUiDismissalSource) {
        self.phase = RuleRuntimePhase::Hidden;
        self.session = None;
        self.commands
            .push_back(RuleRuntimeCommand::ExitMode(RuleExitModeCommand {
                source,
                event_group: RULE_RUNTIME_EVENT_GROUP,
                event_function: RULE_RUNTIME_EXIT_MODE_FUNCTION,
                game_mode: RULE_UI_GAME_MODE_VALUE,
                linked_game_mode: RULE_RUNTIME_MAIN_GAME_MODE_VALUE,
                cursor_locked: false,
                show_gameplay_hud: true,
                gameplay_input_enabled: true,
                force_nanocom_open: false,
                end_camera_sub_target: true,
                clear_npc_interaction_locally: session.source_npc_id().is_some(),
                send_npc_interaction_close_packet: false,
            }));
        self.commands
            .push_back(RuleRuntimeCommand::PlaySound(RuleRuntimeSound::CloseScreen));
    }

    fn allocate_escape_gate_token(&mut self) -> RuleEscapeGateToken {
        let token = RuleEscapeGateToken(self.next_escape_gate_token.max(1));
        self.next_escape_gate_token = self.next_escape_gate_token.wrapping_add(1).max(1);
        token
    }

    #[must_use]
    pub fn command_len(&self) -> usize {
        self.commands.len()
    }

    #[must_use]
    pub fn commands_are_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn pop_command(&mut self) -> Option<RuleRuntimeCommand> {
        self.commands.pop_front()
    }

    pub fn drain_commands(&mut self) -> impl Iterator<Item = RuleRuntimeCommand> + '_ {
        self.commands.drain(..)
    }

    /// Session teardown for disconnect/state replacement. It intentionally
    /// emits no close-mode effects into a shell which is already disappearing.
    pub fn reset_session(
        &mut self,
        model: &mut RuleUiModel,
        ui: &mut RuleUiOutbox,
        audio: &mut RuleUiAudioOutbox,
    ) {
        model.close();
        model.set_system_popup_active(false);
        model.set_help_active(false);
        ui.clear();
        audio.clear();
        self.phase = RuleRuntimePhase::Hidden;
        self.session = None;
        self.commands.clear();
    }
}
