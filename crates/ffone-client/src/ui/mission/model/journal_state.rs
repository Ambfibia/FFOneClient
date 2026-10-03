//! MissionUiModel mission journal, task stop and journal tabs.

use super::super::layout::journal_right_layout;
use super::super::requests::PendingMissionUiRequest;
use super::{JournalListTab, JournalOtherUi, MissionJournalUi, MissionUiEntry, MissionUiModel};
use crate::gameplay_ui::{GameplayUiAction, GameplayUiOutbox};
use bevy::prelude::*;

impl MissionUiModel {
    pub(in super::super) fn open_journal(
        &mut self,
        journal: JournalOtherUi,
        outbox: &mut GameplayUiOutbox,
    ) -> bool {
        self.nanocom_main_menu_visible = false;
        self.journal_tab = JournalListTab::Active;
        self.selected_journal_task_id = self
            .selected_journal_task_id
            .filter(|id| {
                journal
                    .active_missions
                    .iter()
                    .any(|mission| mission.task_id == *id)
            })
            .or_else(|| {
                journal
                    .active_missions
                    .first()
                    .map(|mission| mission.task_id)
            });
        self.viewed_journal_task_id = self
            .remembered_active_journal_task_id
            .filter(|id| {
                journal
                    .active_missions
                    .iter()
                    .any(|mission| mission.task_id == *id)
            })
            .or(self.selected_journal_task_id);
        self.remembered_active_journal_task_id = self.viewed_journal_task_id;
        self.journal = MissionJournalUi::Other(journal);
        outbox.push(GameplayUiAction::CloseNanocomMenu);
        outbox.push(GameplayUiAction::OpenMissionJournal);
        true
    }

    pub fn close_journal(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if matches!(self.journal, MissionJournalUi::Hidden) || self.pending.is_some() {
            return false;
        }
        let returns_to_npc = matches!(
            self.journal,
            MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. }
        );
        self.journal = MissionJournalUi::Hidden;
        self.journal_tab = JournalListTab::Active;
        self.viewed_journal_task_id = None;
        self.npc_icon_mode_visible = returns_to_npc && self.npc_interaction.is_some();
        outbox.push(GameplayUiAction::CloseMissionJournal);
        true
    }

    /// The Active-list DELETE MISSION click owns only the confirmation
    /// intent. The journal remains visible and no network-pending lock is
    /// installed until the central SystemMessage confirms it.
    pub fn can_request_task_stop_confirmation(&self, task_id: i32) -> bool {
        if !self.enabled
            || self.pending.is_some()
            || self.journal_tab != JournalListTab::Active
            || self.viewed_journal_task_id != Some(task_id)
        {
            return false;
        }
        let MissionJournalUi::Other(journal) = &self.journal else {
            return false;
        };
        journal
            .active_missions
            .iter()
            .any(|mission| mission.task_id == task_id)
    }

    pub fn request_task_stop_confirmation(&self, outbox: &mut GameplayUiOutbox) -> bool {
        let Some(task_id) = self.viewed_journal_task_id else {
            return false;
        };
        if !self.can_request_task_stop_confirmation(task_id) {
            return false;
        }
        outbox.push(GameplayUiAction::RequestTaskStopConfirmation { task_id });
        true
    }

    /// Installs the TaskStop network lock only after the correlated central
    /// confirmation revalidates the same viewed Active task.
    pub fn begin_task_stop(&mut self, task_id: i32) -> bool {
        if !self.enabled
            || self.pending.is_some()
            || self.journal_tab != JournalListTab::Active
            || self.viewed_journal_task_id != Some(task_id)
        {
            return false;
        }
        let MissionJournalUi::Other(journal) = &self.journal else {
            return false;
        };
        if !journal
            .active_missions
            .iter()
            .any(|mission| mission.task_id == task_id)
        {
            return false;
        }
        self.pending = Some(PendingMissionUiRequest::TaskStop { task_id });
        true
    }

    /// Resolves an authoritative TaskStop success without leaving the
    /// NanoCom journal. The tracked task wins when it remains active; clean's
    /// last Active row is the deterministic fallback.
    pub fn confirm_task_stop(&mut self, task_id: i32, remaining_active_task_ids: &[i32]) -> bool {
        if !matches!(
            self.pending,
            Some(PendingMissionUiRequest::TaskStop { task_id: pending }) if pending == task_id
        ) {
            return false;
        }
        self.pending = None;
        self.journal_tab = JournalListTab::Active;
        self.npc_icon_mode_visible = false;
        self.nanocom_journal
            .active_missions
            .retain(|mission| mission.task_id != task_id);
        if let MissionJournalUi::Other(journal) = &mut self.journal {
            journal
                .active_missions
                .retain(|mission| mission.task_id != task_id);
        }
        let selected = self
            .selected_journal_task_id
            .filter(|selected| remaining_active_task_ids.contains(selected))
            .or_else(|| remaining_active_task_ids.last().copied());
        self.selected_journal_task_id = selected;
        self.viewed_journal_task_id = selected;
        self.remembered_active_journal_task_id = selected;
        true
    }

    pub fn select_journal_mission(&mut self, visible_row: usize) -> bool {
        if self.pending.is_some() || !matches!(self.journal, MissionJournalUi::Other(_)) {
            return false;
        }
        let Some(task_id) = journal_right_layout(self)
            .rows
            .get(visible_row)
            .map(|row| row.mission.task_id)
        else {
            return false;
        };
        self.viewed_journal_task_id = Some(task_id);
        if self.journal_tab == JournalListTab::Active {
            self.remembered_active_journal_task_id = Some(task_id);
        }
        true
    }

    /// Refresh quest data without replacing a surviving choice by list position.
    pub fn set_active_journal_missions(&mut self, missions: Vec<MissionUiEntry>) {
        let remembered = self.remembered_active_journal_task_id.or_else(|| {
            (self.journal_tab == JournalListTab::Active
                && matches!(self.journal, MissionJournalUi::Other(_)))
            .then_some(self.viewed_journal_task_id)
            .flatten()
        });
        if remembered.is_some()
            || (self.journal_tab == JournalListTab::Active
                && matches!(self.journal, MissionJournalUi::Other(_)))
        {
            self.remembered_active_journal_task_id = remembered
                .filter(|id| missions.iter().any(|mission| mission.task_id == *id))
                .or_else(|| {
                    self.selected_journal_task_id
                        .filter(|id| missions.iter().any(|mission| mission.task_id == *id))
                })
                .or_else(|| missions.first().map(|mission| mission.task_id));
        }
        self.nanocom_journal.active_missions = missions.clone();
        if let MissionJournalUi::Other(journal) = &mut self.journal {
            journal.active_missions = missions;
            if self.journal_tab == JournalListTab::Active {
                self.viewed_journal_task_id = self.remembered_active_journal_task_id;
            }
        }
    }

    pub fn set_completed_journal_missions(&mut self, missions: Vec<MissionUiEntry>) {
        self.nanocom_journal.completed_missions = missions.clone();
        if let MissionJournalUi::Other(journal) = &mut self.journal {
            journal.completed_missions = missions;
        }
        let completed = match &self.journal {
            MissionJournalUi::Other(journal) => &journal.completed_missions,
            _ => &self.nanocom_journal.completed_missions,
        };
        if self.journal_tab == JournalListTab::Completed
            && !completed
                .iter()
                .any(|mission| Some(mission.task_id) == self.viewed_journal_task_id)
        {
            self.viewed_journal_task_id = completed.first().map(|mission| mission.task_id);
        }
    }

    pub fn select_journal_tab(&mut self, tab: JournalListTab) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let MissionJournalUi::Other(journal) = &self.journal else {
            return false;
        };
        if self.journal_tab == tab {
            return false;
        }
        if self.journal_tab == JournalListTab::Active {
            self.remembered_active_journal_task_id = self.viewed_journal_task_id;
        }
        self.journal_tab = tab;
        self.completed_category_expanded = [true; 3];
        self.viewed_journal_task_id = match tab {
            JournalListTab::Active => self
                .remembered_active_journal_task_id
                .or(self.selected_journal_task_id)
                .and_then(|task_id| {
                    journal
                        .active_missions
                        .iter()
                        .any(|mission| mission.task_id == task_id)
                        .then_some(task_id)
                })
                .or_else(|| {
                    journal
                        .active_missions
                        .last()
                        .map(|mission| mission.task_id)
                }),
            JournalListTab::Completed => journal
                .completed_missions
                .first()
                .map(|mission| mission.task_id),
        };
        true
    }

    pub fn toggle_completed_category(&mut self, section: usize) -> bool {
        if self.pending.is_some()
            || !matches!(self.journal, MissionJournalUi::Other(_))
            || self.journal_tab != JournalListTab::Completed
        {
            return false;
        }
        let Some(expanded) = self.completed_category_expanded.get_mut(section) else {
            return false;
        };
        *expanded = !*expanded;
        true
    }
}
