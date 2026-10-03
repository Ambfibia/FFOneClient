//! MissionUiModel NPC interaction: letterbox, mission selection, rewards, utility warps and close.

use super::super::requests::PendingMissionUiRequest;
use super::{MissionJournalUi, MissionUiEntry, MissionUiModel, NpcInteractionUi};
use crate::{
    gameplay_ui::{GameplayUiAction, GameplayUiOutbox},
    localization::LocalizedText,
    tutorial_mission_content::TutorialWarpTarget,
};

impl MissionUiModel {
    /// Keep the cinematic frame while the interaction panel is dismissed for
    /// departure. Never reopen its buttons merely to retain the black bars.
    pub fn npc_letterbox_visible(&self) -> bool {
        self.npc_icon_mode_visible || self.pending_warp.is_some() || self.warp_transition_active
    }

    /// `NpcIconMode` owns one persistent camera sub-target while switching to
    /// and from its Allow/Reward journal. Departure releases it before the
    /// effect, as does closing the interaction or opening a non-NPC journal.
    pub fn npc_subtarget_actor_id(&self) -> Option<i32> {
        // WarpOK ends the camera subtarget before playing the departure.
        // Retaining it also hides the local character's visual subtree.
        if self.pending_warp.is_some() || self.warp_transition_active {
            return None;
        }
        let npc_mode_active = self.npc_icon_mode_visible
            || matches!(
                self.journal,
                MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. }
            );
        npc_mode_active
            .then(|| {
                self.npc_interaction
                    .as_ref()
                    .map(|interaction| interaction.npc_id)
            })
            .flatten()
    }

    pub fn is_interacting_with(&self, npc_id: i32) -> bool {
        self.enabled
            && self.npc_icon_mode_visible
            && self
                .npc_interaction
                .as_ref()
                .is_some_and(|interaction| interaction.npc_id == npc_id)
    }

    pub fn show_npc_interaction(&mut self, interaction: NpcInteractionUi) {
        self.enabled = true;
        self.npc_interaction = Some(interaction);
        self.npc_icon_mode_visible = true;
        self.nanocom_main_menu_visible = false;
        self.journal = MissionJournalUi::Hidden;
        self.pending = None;
        self.pending_warp = None;
        self.npc_top_notice = None;
    }

    /// `NpcIconMode.ReceivePacket` for a transportation-registration success:
    /// type 1 (S.C.A.M.P.E.R.) names a warp hub, every other type a generic one.
    pub fn show_transportation_registration_notice(&mut self, transportation_type: i32) {
        self.npc_top_notice = Some(if transportation_type == 1 {
            LocalizedText::new(
                "ui.transportation.registered.warp",
                "New warp hub registered in your NanoCom!",
            )
        } else {
            LocalizedText::new(
                "ui.transportation.registered.hub",
                "New transportation hub registered in your NanoCom!",
            )
        });
    }

    #[must_use]
    pub fn npc_top_notice(&self) -> Option<&LocalizedText> {
        self.npc_top_notice.as_ref()
    }

    pub fn select_npc_mission(
        &mut self,
        visible_row: usize,
        outbox: &mut GameplayUiOutbox,
    ) -> bool {
        if !self.enabled
            || !self.npc_icon_mode_visible
            || self.pending.is_some()
            || self.pending_warp.is_some()
        {
            return false;
        }
        let Some(interaction) = self.npc_interaction.as_ref() else {
            return false;
        };
        let completed_len = interaction.completed_missions.len();
        let selection = if visible_row < completed_len {
            interaction
                .completed_missions
                .get(visible_row)
                .cloned()
                .map(|mission| (mission, true))
        } else {
            interaction
                .available_missions
                .get(visible_row - completed_len)
                .cloned()
                .map(|mission| (mission, false))
        };
        let Some((mission, reward)) = selection else {
            return false;
        };
        let Some(npc_id) = mission.npc_id else {
            return false;
        };

        self.nanocom_main_menu_visible = false;
        if reward {
            let task_id = mission.task_id;
            // Every ready row, including the first Nano talk, follows the
            // clean NpcIconMode reward gate; none opens the Allow journal.
            if !mission.has_task_reward {
                // NpcIconMode checks the current task's reward, not its type
                // or the final reward displayed in the mission journal. This
                // also covers item handoffs, object interactions and terminal
                // rewardless tasks; none opens the Reward page.
                // Keep NpcIconMode (including its letterbox) visible while
                // awaiting the reply. Hiding it here flashes the world/HUD
                // before confirm_quest_end restores the same NPC menu.
                // `pending` blocks duplicate clicks without closing the UI.
                self.journal = MissionJournalUi::Hidden;
                let request = PendingMissionUiRequest::QuestEnd {
                    task_id,
                    npc_id,
                    box1_choice: 0,
                    box2_choice: 0,
                };
                self.pending = Some(request);
                outbox.push(GameplayUiAction::QuestEnd {
                    task_id,
                    npc_id,
                    box1_choice: 0,
                    box2_choice: 0,
                });
                return true;
            }
            self.npc_icon_mode_visible = false;
            outbox.push(GameplayUiAction::OpenMissionReward { task_id, npc_id });
            self.journal = MissionJournalUi::Reward {
                mission,
                box1_choice: 0,
                box2_choice: 0,
            };
        } else {
            self.npc_icon_mode_visible = false;
            outbox.push(GameplayUiAction::OpenMissionAllow {
                task_id: mission.task_id,
                npc_id,
            });
            self.journal = MissionJournalUi::Allow(mission);
        }
        true
    }

    /// Opens the clean source-owned Reward journal emitted by mission
    /// progress (kill/item/proximity), without inventing an NPC-icon click.
    pub fn open_automatic_reward(&mut self, mission: MissionUiEntry) -> bool {
        if matches!(
            &self.journal,
            MissionJournalUi::Reward { mission: current, .. }
                if current.task_id == mission.task_id
        ) {
            return false;
        }
        if self.pending.is_some() || self.system_dialog.is_some() {
            return false;
        }
        self.enabled = true;
        self.npc_icon_mode_visible = false;
        self.nanocom_main_menu_visible = false;
        self.viewed_journal_task_id = Some(mission.task_id);
        self.journal = MissionJournalUi::Reward {
            mission,
            box1_choice: 0,
            box2_choice: 0,
        };
        true
    }

    pub fn accept_mission(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let MissionJournalUi::Allow(mission) = &self.journal else {
            return false;
        };
        let Some(npc_id) = mission.npc_id else {
            return false;
        };
        let task_id = mission.task_id;
        self.pending = Some(PendingMissionUiRequest::TaskStart { task_id, npc_id });
        outbox.push(GameplayUiAction::TaskStart { task_id, npc_id });
        true
    }

    pub fn complete_mission(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let MissionJournalUi::Reward {
            mission,
            box1_choice,
            box2_choice,
        } = &self.journal
        else {
            return false;
        };
        let Some(npc_id) = mission.npc_id else {
            return false;
        };
        let request = PendingMissionUiRequest::QuestEnd {
            task_id: mission.task_id,
            npc_id,
            box1_choice: *box1_choice,
            box2_choice: *box2_choice,
        };
        self.pending = Some(request);
        outbox.push(GameplayUiAction::QuestEnd {
            task_id: mission.task_id,
            npc_id,
            box1_choice: *box1_choice,
            box2_choice: *box2_choice,
        });
        true
    }

    /// Models the successful TaskStart reply (`822083612`) and
    /// `cnMissionJournal.EndModeToTarget`.
    pub fn confirm_task_start(&mut self, task_id: i32) -> bool {
        if !matches!(
            self.pending,
            Some(PendingMissionUiRequest::TaskStart {
                task_id: pending,
                ..
            }) if pending == task_id
        ) {
            return false;
        }
        self.pending = None;
        self.journal = MissionJournalUi::Hidden;
        if let Some(interaction) = &mut self.npc_interaction {
            interaction
                .available_missions
                .retain(|mission| mission.task_id != task_id);
            self.npc_icon_mode_visible = true;
        }
        true
    }

    /// Models the successful TaskEnd reply (`822083614`) and
    /// `cnMissionJournal.EndModeToTarget`.
    pub fn confirm_quest_end(&mut self, task_id: i32) -> bool {
        if !matches!(
            self.pending,
            Some(PendingMissionUiRequest::QuestEnd {
                task_id: pending,
                ..
            }) if pending == task_id
        ) {
            return false;
        }
        self.pending = None;
        self.journal = MissionJournalUi::Hidden;
        if let Some(interaction) = &mut self.npc_interaction {
            interaction
                .completed_missions
                .retain(|mission| mission.task_id != task_id);
            self.npc_icon_mode_visible = true;
        }
        true
    }

    /// A failed reply unlocks the current dialog. Auto-submitted rewardless steps
    /// keep the NPC list because they deliberately own no journal page.
    pub fn reject_pending(&mut self, task_id: i32) -> bool {
        if self
            .pending
            .is_some_and(|pending| pending.task_id() == task_id)
        {
            self.pending = None;
            if matches!(self.journal, MissionJournalUi::Hidden) && self.npc_interaction.is_some() {
                self.npc_icon_mode_visible = true;
            }
            true
        } else {
            false
        }
    }

    pub fn activate_npc_utility(
        &mut self,
        visible_row: usize,
        outbox: &mut GameplayUiOutbox,
    ) -> bool {
        if !self.enabled
            || !self.npc_icon_mode_visible
            || self.pending.is_some()
            || self.pending_warp.is_some()
        {
            return false;
        }
        let Some(interaction) = self.npc_interaction.as_ref() else {
            return false;
        };
        if let Some(entry) = interaction.services.get(visible_row) {
            let npc_id = interaction.npc_id;
            let service = entry.service;
            self.npc_icon_mode_visible = false;
            outbox.push(GameplayUiAction::NpcService { npc_id, service });
            return true;
        }
        if visible_row == interaction.services.len() && interaction.warp.is_some() {
            return self.warp(outbox);
        }
        false
    }

    pub fn warp(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.enabled
            || !self.npc_icon_mode_visible
            || self.pending.is_some()
            || self.pending_warp.is_some()
        {
            return false;
        }
        let Some(warp) = self
            .npc_interaction
            .as_ref()
            .and_then(|interaction| interaction.warp.as_ref())
            .cloned()
        else {
            return false;
        };
        self.npc_icon_mode_visible = false;
        self.pending_warp = Some(warp.clone());
        outbox.push(GameplayUiAction::NpcWarp {
            npc_id: warp.npc_id,
            npc_type: warp.npc_type,
            warp_id: warp.warp_id,
            required_task_id: warp.required_task_id,
            target: warp.target,
        });
        true
    }

    pub fn pending_warp_matches(
        &self,
        npc_id: i32,
        npc_type: i32,
        warp_id: i32,
        required_task_id: Option<i32>,
        target: TutorialWarpTarget,
    ) -> bool {
        self.pending_warp.as_ref().is_some_and(|warp| {
            warp.npc_id == npc_id
                && warp.npc_type == npc_type
                && warp.warp_id == warp_id
                && warp.required_task_id == required_task_id
                && warp.target == target
        })
    }

    pub fn confirm_warp(
        &mut self,
        npc_id: i32,
        npc_type: i32,
        warp_id: i32,
        required_task_id: Option<i32>,
        target: TutorialWarpTarget,
    ) -> bool {
        if !self.pending_warp_matches(npc_id, npc_type, warp_id, required_task_id, target) {
            return false;
        }
        self.pending_warp = None;
        self.npc_icon_mode_visible = false;
        true
    }

    pub fn reject_warp(
        &mut self,
        npc_id: i32,
        npc_type: i32,
        warp_id: i32,
        required_task_id: Option<i32>,
        target: TutorialWarpTarget,
    ) -> bool {
        if !self.pending_warp_matches(npc_id, npc_type, warp_id, required_task_id, target) {
            return false;
        }
        self.pending_warp = None;
        self.npc_icon_mode_visible = self.npc_interaction.is_some();
        true
    }

    pub fn close_npc_interaction(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.enabled
            || !self.npc_icon_mode_visible
            || self.pending.is_some()
            || self.pending_warp.is_some()
        {
            return false;
        }
        let Some(npc_id) = self
            .npc_interaction
            .as_ref()
            .map(|interaction| interaction.npc_id)
        else {
            return false;
        };
        self.npc_icon_mode_visible = false;
        outbox.push(GameplayUiAction::NpcIconClose { npc_id });
        true
    }

    /// End an inherited `NpcIconMode` owner without emitting its normal
    /// interaction-close packet. Clean `NpcIconMode.StartRule` deliberately
    /// skips `EndMode`; Rule's local `2/1` return clears the owner only after
    /// the linked mode exits.
    pub fn clear_npc_interaction_locally(&mut self) {
        self.npc_icon_mode_visible = false;
        self.npc_interaction = None;
        self.pending = None;
        self.pending_warp = None;
        self.journal = MissionJournalUi::Hidden;
    }
}
