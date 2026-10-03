//! MissionUiModel system dialog, modal/input gates and tutorial-exit confirmation.

use super::super::system_dialog::{
    TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE, TUTORIAL_EXIT_SYSTEM_MESSAGE_ID,
    TutorialSystemDialogUi,
};
use super::{MissionJournalUi, MissionUiModel};
use crate::{
    gameplay_ui::{GameplayUiAction, GameplayUiOutbox},
    tutorial_logic::{TutorialJournalMode, TutorialUiObservation},
};

impl MissionUiModel {
    pub const fn system_dialog(&self) -> Option<TutorialSystemDialogUi> {
        self.system_dialog
    }

    pub const fn system_popup_active(&self) -> bool {
        self.system_dialog.is_some()
    }

    pub const fn nanocom_foreign_modal_suppressed(&self) -> bool {
        self.nanocom_foreign_modal_suppressed
    }

    pub const fn nanocom_menu_presented(&self) -> bool {
        self.enabled && self.nanocom_main_menu_visible && !self.nanocom_foreign_modal_suppressed
    }

    pub fn set_nanocom_foreign_modal_suppressed(&mut self, suppressed: bool) {
        self.nanocom_foreign_modal_suppressed = suppressed;
    }

    /// Mirrors the linked legacy game-mode surfaces which call
    /// `cnMainGame.SetAvatarInputEnable(false)`. A pending warp remains in the
    /// NPC mode until its fade/teleport reply completes.
    pub const fn gameplay_input_blocked(&self) -> bool {
        self.stops_auto_run() || self.nanocom_main_menu_visible
    }

    /// Mission and NPC modes stop existing movement. The Enter/NanoCom menu
    /// only captures manual input and leaves an existing auto-run active.
    pub const fn stops_auto_run(&self) -> bool {
        self.warp_transition_active
            || self.system_dialog.is_some()
            || self.npc_icon_mode_visible
            || !matches!(self.journal, MissionJournalUi::Hidden)
            || self.pending_warp.is_some()
    }

    /// Chat is part of the open NanoCom surface, so the NanoCom main menu
    /// blocks avatar controls but must not revoke the chat field's focus.
    /// The other linked modal owners still clear and block chat input.
    pub const fn chat_input_blocked(&self) -> bool {
        self.warp_transition_active
            || self.system_dialog.is_some()
            || self.npc_icon_mode_visible
            || !matches!(self.journal, MissionJournalUi::Hidden)
            || self.pending_warp.is_some()
    }

    pub fn open_tutorial_exit_dialog(&mut self) -> bool {
        if self.system_dialog.is_some() || self.tutorial_exit_confirmation_pending {
            return false;
        }
        self.system_dialog = Some(TutorialSystemDialogUi::tutorial_exit());
        true
    }

    pub fn cancel_tutorial_exit_dialog(&mut self) -> bool {
        if !self
            .system_dialog
            .is_some_and(TutorialSystemDialogUi::is_tutorial_exit)
        {
            return false;
        }
        self.system_dialog = None;
        true
    }

    pub fn confirm_tutorial_exit_dialog(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        let Some(dialog) = self.system_dialog else {
            return false;
        };
        if !dialog.is_tutorial_exit() || self.tutorial_exit_confirmation_pending {
            return false;
        }
        self.system_dialog = None;
        self.tutorial_exit_confirmation_pending = true;
        outbox.push(GameplayUiAction::ConfirmTutorialExit {
            message_id: dialog.message_id(),
            button_type: dialog.button_type(),
        });
        true
    }

    pub fn take_tutorial_exit_confirmation(&mut self, message_id: i32, button_type: i32) -> bool {
        if !self.tutorial_exit_confirmation_pending
            || message_id != TUTORIAL_EXIT_SYSTEM_MESSAGE_ID
            || button_type != TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE
        {
            return false;
        }
        self.tutorial_exit_confirmation_pending = false;
        true
    }

    pub fn tutorial_observation(&self) -> TutorialUiObservation {
        TutorialUiObservation {
            hostile_target_selected: false,
            nanocom_main_menu_visible: self.enabled && self.nanocom_main_menu_visible,
            journal_mode: if self.enabled {
                self.journal.tutorial_mode()
            } else {
                TutorialJournalMode::Hidden
            },
            npc_icon_mode_visible: self.enabled && self.npc_icon_mode_visible,
        }
    }
}
