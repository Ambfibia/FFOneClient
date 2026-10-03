//! MissionUiModel Nanocom menu and the windows it opens.

use super::{JournalOtherUi, MissionJournalUi, MissionUiModel};
use crate::{
    gameplay_ui::{GameplayUiAction, GameplayUiOutbox},
    user_equip_ui::UserEquipOpenSource,
};

impl MissionUiModel {
    pub fn open_nanocom_menu(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.enabled
            || self.nanocom_main_menu_visible
            || self.nanocom_foreign_modal_suppressed
            || self.npc_icon_mode_visible
            || !matches!(self.journal, MissionJournalUi::Hidden)
            || self.pending_warp.is_some()
        {
            return false;
        }
        self.nanocom_main_menu_visible = true;
        outbox.push(GameplayUiAction::OpenNanocomMenu);
        true
    }

    pub fn close_nanocom_menu(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.nanocom_main_menu_visible {
            return false;
        }
        self.nanocom_main_menu_visible = false;
        outbox.push(GameplayUiAction::CloseNanocomMenu);
        true
    }

    /// Mirrors `cnGUINanocom.SetNewMail`: selecting E-MAIL clears this latch.
    pub fn notify_new_email(&mut self) {
        self.nanocom_new_mail_notice_visible = true;
    }

    pub fn open_user_equip_from_nanocom(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.nanocom_menu_presented() {
            return false;
        }
        self.nanocom_main_menu_visible = false;
        // `OnStatus` requests mode 6 before emitting the menu-close event.
        outbox.push(GameplayUiAction::OpenUserEquipItemMode {
            source: UserEquipOpenSource::NanocomMyStuff,
        });
        outbox.push(GameplayUiAction::CloseNanocomMenu);
        true
    }

    pub fn open_email_from_nanocom(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.nanocom_menu_presented() {
            return false;
        }
        self.nanocom_main_menu_visible = false;
        self.nanocom_new_mail_notice_visible = false;
        outbox.push(GameplayUiAction::OpenEmailFromNanocom);
        outbox.push(GameplayUiAction::CloseNanocomMenu);
        true
    }

    pub fn open_world_map_from_nanocom(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.nanocom_menu_presented() {
            return false;
        }
        self.nanocom_main_menu_visible = false;
        outbox.push(GameplayUiAction::OpenWorldMapFromNanocom);
        outbox.push(GameplayUiAction::CloseNanocomMenu);
        true
    }

    pub fn open_game_guide_from_nanocom(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.nanocom_menu_presented() {
            return false;
        }
        // `OnGuide` emits only `(2, 18)`; it does not send the shared
        // Nanocom-menu close event.
        self.nanocom_foreign_modal_suppressed = true;
        outbox.push(GameplayUiAction::OpenGameGuideFromNanocom);
        true
    }

    pub fn open_quit_from_nanocom(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.nanocom_menu_presented() {
            return false;
        }
        // `OnQuitMenu` emits only the mode-23 request.
        self.nanocom_foreign_modal_suppressed = true;
        outbox.push(GameplayUiAction::OpenQuitFromNanocom);
        true
    }

    pub fn open_option_from_nanocom(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.nanocom_menu_presented() {
            return false;
        }
        self.nanocom_main_menu_visible = false;
        // Preserve `cnGUINanocom.OnOption` event order: request mode 12,
        // then emit the shared `(2, 3, 0)` Nanocom-menu close event.
        outbox.push(GameplayUiAction::OpenOptionFromNanocomSettings);
        outbox.push(GameplayUiAction::CloseNanocomMenu);
        true
    }

    pub fn open_journal_from_nanocom(
        &mut self,
        journal: JournalOtherUi,
        outbox: &mut GameplayUiOutbox,
    ) -> bool {
        if !self.nanocom_menu_presented() {
            return false;
        }
        self.open_journal(journal, outbox)
    }

    pub fn open_journal_from_shortcut(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.enabled || self.gameplay_input_blocked() || self.pending.is_some() {
            return false;
        }
        self.open_journal(self.nanocom_journal.clone(), outbox)
    }
}
