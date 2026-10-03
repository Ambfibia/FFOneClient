//! Typed UserEquip UI actions, audio cues and their outboxes.

use super::catalog::UserEquipSlotEndpoint;
use super::nano_station::UserEquipNanoStationAction;
use super::state::{
    UserEquipCloseBlockedReason, UserEquipCloseDisposition, UserEquipCloseSource,
    UserEquipModalState, UserEquipMode, UserEquipUiState,
};
use bevy::prelude::*;
use std::collections::VecDeque;

/// Proven local requests emitted by the native Item-mode shell. Move/use/delete
/// requests carry only exact legacy endpoints; production owns protocol
/// encoding and waits for authoritative replies before rebuilding projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserEquipUiAction {
    NanoStation(UserEquipNanoStationAction),
    RequestClose {
        source: UserEquipCloseSource,
    },
    ApplyLegacyScrollAxis {
        axis: f32,
    },
    /// Unity scroll-view thumb drag or trough page, in content pixels.
    SetScrollY {
        scroll_y: f32,
    },
    SelectTab {
        mode: UserEquipMode,
    },
    MoveItem {
        from: UserEquipSlotEndpoint,
        to: UserEquipSlotEndpoint,
    },
    UseInventoryItem {
        slot_index: usize,
    },
    UseInventoryItemOnNano {
        slot_index: usize,
        nano_slot: usize,
    },
    OpenInventoryChest {
        slot_index: usize,
    },
    DeleteInventoryItem {
        slot_index: usize,
    },
}

/// Source-proven `SoundUtil` calls emitted by the clean UserEquip shell.
/// Physical files stay owned by `NativeAudioCatalog` in the application
/// runtime; the UI only reports the semantic legacy cue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipUiAudioCue {
    ButtonSound,
    OpenScreen,
    TabClick01,
}

#[derive(Default, Resource)]
pub struct UserEquipUiAudioOutbox {
    pub(super) cues: VecDeque<UserEquipUiAudioCue>,
}

impl UserEquipUiAudioOutbox {
    pub fn push(&mut self, cue: UserEquipUiAudioCue) {
        self.cues.push_back(cue);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = UserEquipUiAudioCue> + '_ {
        self.cues.drain(..)
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }
}

#[derive(Default, Resource)]
pub struct UserEquipUiOutbox {
    pub(super) actions: VecDeque<UserEquipUiAction>,
}

impl UserEquipUiOutbox {
    pub fn push(&mut self, action: UserEquipUiAction) {
        self.actions.push_back(action);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = UserEquipUiAction> + '_ {
        self.actions.drain(..)
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserEquipUiActionOutcome {
    Closed,
    DismissedPopups,
    Scrolled { previous: f32, current: f32 },
    CloseBlocked(UserEquipCloseBlockedReason),
    ScrollBlocked,
    TabSelected(UserEquipMode),
    RequestQueued,
}

/// Applies one already-typed UI request without touching inventory authority.
///
/// Production drains [`UserEquipUiOutbox`] through this reducer. Keeping it
/// pure makes the no-speculation boundary testable independently from Bevy
/// pointer and keyboard collection.
pub fn apply_user_equip_ui_action(
    state: &mut UserEquipUiState,
    modal: &mut UserEquipModalState,
    action: UserEquipUiAction,
) -> UserEquipUiActionOutcome {
    match action {
        UserEquipUiAction::RequestClose { source } => {
            match state.close_disposition(source, *modal) {
                UserEquipCloseDisposition::ExitMode => {
                    state.close();
                    UserEquipUiActionOutcome::Closed
                }
                UserEquipCloseDisposition::DismissPopups => {
                    modal.inventory_popup_modal = false;
                    modal.item_popup_active = false;
                    UserEquipUiActionOutcome::DismissedPopups
                }
                UserEquipCloseDisposition::Blocked(reason) => {
                    UserEquipUiActionOutcome::CloseBlocked(reason)
                }
            }
        }
        UserEquipUiAction::ApplyLegacyScrollAxis { axis } => {
            if !state.input_capabilities(*modal).scroll {
                return UserEquipUiActionOutcome::ScrollBlocked;
            }
            let previous = state.scroll_y();
            state.apply_legacy_scroll_axis(axis);
            UserEquipUiActionOutcome::Scrolled {
                previous,
                current: state.scroll_y(),
            }
        }
        UserEquipUiAction::SetScrollY { scroll_y } => {
            if !state.input_capabilities(*modal).scroll || !scroll_y.is_finite() {
                return UserEquipUiActionOutcome::ScrollBlocked;
            }
            let previous = state.scroll_y();
            state.set_scroll_y(scroll_y);
            UserEquipUiActionOutcome::Scrolled {
                previous,
                current: state.scroll_y(),
            }
        }
        UserEquipUiAction::SelectTab { mode } => {
            // CnEquip's configurable I/N reader remains live during the
            // opening animation; pointer controls keep their own slide gate.
            if state.input_capabilities(*modal).keyboard_close_request
                && !modal.has_popup_to_dismiss()
            {
                match mode {
                    UserEquipMode::Item => state.select_item_tab(),
                    UserEquipMode::Nano => state.select_nano_tab(),
                }
                UserEquipUiActionOutcome::TabSelected(mode)
            } else {
                UserEquipUiActionOutcome::ScrollBlocked
            }
        }
        UserEquipUiAction::NanoStation(_)
        | UserEquipUiAction::MoveItem { .. }
        | UserEquipUiAction::UseInventoryItem { .. }
        | UserEquipUiAction::UseInventoryItemOnNano { .. }
        | UserEquipUiAction::OpenInventoryChest { .. }
        | UserEquipUiAction::DeleteInventoryItem { .. } => UserEquipUiActionOutcome::RequestQueued,
    }
}
