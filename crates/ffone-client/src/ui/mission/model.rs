//! Neutral mission UI model: NPC interaction, journal, warp-away and dialog state.

use super::requests::PendingMissionUiRequest;
use super::system_dialog::TutorialSystemDialogUi;
use crate::{
    gameplay_ui::NpcServiceKind,
    localization::LocalizedText,
    tutorial_logic::TutorialJournalMode,
    tutorial_mission_content::{TutorialNanoJournalUi, TutorialWarpTarget},
};
use bevy::prelude::*;

mod dialog_state;
mod journal_state;
mod nanocom_state;
mod npc_state;
mod warp_away_state;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MissionUiEntry {
    pub task_id: i32,
    /// Present only for an actual NPC interaction. Nanocom Journal entries are
    /// source projections and deliberately carry no fabricated live ID.
    pub npc_id: Option<i32>,
    /// XDT `m_iHMissionType`: 1 guide, 2 nano, 3 normal/world.
    pub mission_type: i32,
    /// XDT `m_iHTaskType`, used by the first Nano offer handoff.
    pub task_type: i32,
    /// XDT `m_iSUOutgoingTask`. A nonzero value distinguishes an in-mission
    /// conversation/update from the terminal hand-in screen.
    pub outgoing_task_id: i32,
    /// Whether this exact task grants a reward. Journal reward amounts below
    /// describe the final mission reward and must not control NPC hand-in UI.
    pub has_task_reward: bool,
    /// True only for the first serialized task in this mission chain.
    pub is_first_mission_task: bool,
    /// XDT `m_iHJournalNPCID`, retained separately from the live NPC instance.
    pub journal_npc_type: i32,
    pub required_level: i32,
    /// XDT `m_iHDifficultyType`: 0 easy, 1 normal, 2 hard.
    pub difficulty_type: i32,
    /// Clean TableData projection consumed by the Nano mission page.
    pub nano: Option<TutorialNanoJournalUi>,
    pub title: String,
    pub npc_name: String,
    pub npc_position: String,
    /// Exact `m_pMissionStringData[m_iHCurrentObjective]` text. Mission-list
    /// rows show this alone; the expanded Active page separately appends the
    /// detailed task description.
    pub objective: String,
    /// `m_iDetaileMissionDesc`, used only by the Allow/offer screen.
    pub offer_description: String,
    /// Exact `m_iDetailedTaskDesc`, shown without the current objective when
    /// the player inspects an untracked Active mission.
    pub task_description: String,
    /// Current objective plus `m_iDetailedTaskDesc`, used by the active journal.
    pub active_description: String,
    /// `m_iMissionSummary`, used by each Completed-list row.
    pub mission_summary: String,
    /// `m_iMissionCompleteSummary`, shown as MY NOTES in Completed detail.
    pub mission_complete_summary: String,
    /// `m_iDetaileMissionCompleteSummary`, used only by the Reward screen.
    pub completion_description: String,
    pub rewards: MissionUiRewards,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MissionUiRewards {
    pub cash: i32,
    pub fusion_matter: i32,
}

impl MissionUiRewards {
    pub const fn is_empty(self) -> bool {
        self.cash == 0 && self.fusion_matter == 0
    }
}

impl MissionUiEntry {
    #[must_use]
    pub const fn is_intermediate_talk(&self) -> bool {
        self.task_type == 1 && self.outgoing_task_id != 0
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WarpUiEntry {
    pub npc_id: i32,
    pub npc_type: i32,
    pub warp_id: i32,
    pub required_task_id: Option<i32>,
    pub target: TutorialWarpTarget,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NpcServiceUiEntry {
    pub service: NpcServiceKind,
    pub label: String,
}

impl NpcServiceUiEntry {
    pub fn original(service: NpcServiceKind) -> Self {
        let label = match service {
            NpcServiceKind::Barber => " BARBER",
            NpcServiceKind::Vendor => "ENTER STORE",
            NpcServiceKind::NanoStation => "NANO STATION",
            NpcServiceKind::Bank => "BANK",
            NpcServiceKind::LocalBank => "LOCAL BANK",
            NpcServiceKind::GuideChanger => " GUIDE CHANGER",
            NpcServiceKind::PastWarp => " WARP TO PAST",
            NpcServiceKind::TransportationWarp | NpcServiceKind::TransportationWyvern => " WARP",
            NpcServiceKind::Race => " START RACE",
            NpcServiceKind::RaceRank => " RACE RANK",
            NpcServiceKind::Combine => " COMBINE ITEMS",
            NpcServiceKind::Enchant => " ENCHANT ITEMS",
            NpcServiceKind::Rule => "RULES",
        };
        Self {
            service,
            label: label.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NpcInteractionUi {
    pub npc_id: i32,
    /// Stable TableData identity used by every NPC-icon-mode title.
    pub npc_type: i32,
    pub name: String,
    pub available_missions: Vec<MissionUiEntry>,
    pub completed_missions: Vec<MissionUiEntry>,
    pub services: Vec<NpcServiceUiEntry>,
    pub warp: Option<WarpUiEntry>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JournalOtherUi {
    pub title: String,
    pub active_missions: Vec<MissionUiEntry>,
    pub completed_missions: Vec<MissionUiEntry>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum JournalListTab {
    #[default]
    Active,
    Completed,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MissionJournalUi {
    #[default]
    Hidden,
    Allow(MissionUiEntry),
    Reward {
        mission: MissionUiEntry,
        box1_choice: i32,
        box2_choice: i32,
    },
    Other(JournalOtherUi),
}

impl MissionJournalUi {
    pub const fn tutorial_mode(&self) -> TutorialJournalMode {
        match self {
            Self::Hidden => TutorialJournalMode::Hidden,
            Self::Allow(_) => TutorialJournalMode::Allow,
            Self::Reward { .. } => TutorialJournalMode::Reward,
            Self::Other(_) => TutorialJournalMode::Other,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct MissionUiModel {
    /// The mission UI is inert until the tutorial/runtime explicitly enables it.
    pub enabled: bool,
    pub npc_interaction: Option<NpcInteractionUi>,
    pub npc_icon_mode_visible: bool,
    pub nanocom_main_menu_visible: bool,
    /// Presentation/input suppression while another clean game mode owns the
    /// screen. GuideMode and QuitMenu deliberately do not clear the source
    /// NanoCom latch, so this must remain separate from
    /// `nanocom_main_menu_visible` and release when that foreign owner closes.
    pub(super) nanocom_foreign_modal_suppressed: bool,
    /// Inverse of clean `cnGUINanocom.bNewMailSelect`: the notice is visible
    /// after `SetNewMail` and remains so until the E-MAIL row is accepted.
    pub nanocom_new_mail_notice_visible: bool,
    pub nanocom_journal: JournalOtherUi,
    pub journal: MissionJournalUi,
    pub journal_tab: JournalListTab,
    pub completed_category_expanded: [bool; 3],
    /// Clean `cnMissionJournal.pointScroll.y` for the right-hand mission list.
    pub(super) journal_scroll: f32,
    /// Retrobution's FFGUIUtility scale, applied about each screen's source pivot.
    pub ui_scale: f32,
    /// The expanded row (`cnMissionJournal.currentMission`). Browsing a row
    /// does not change the tracked mission.
    pub viewed_journal_task_id: Option<i32>,
    /// Last browsed Active quest, retained while history or NPC dialogs own the page.
    pub(super) remembered_active_journal_task_id: Option<i32>,
    /// The tracked mission (`cnMissionJournal.SelectMission`), selected by
    /// the explicit `MAKE CURRENT MISSION` action.
    pub selected_journal_task_id: Option<i32>,
    pub pending: Option<PendingMissionUiRequest>,
    pub pending_warp: Option<WarpUiEntry>,
    /// Shared departure/destination presentation, independent of NPC buttons.
    pub warp_transition_active: bool,
    /// Clean `CnGuiChat.fWarpTimer`/`bWarpEnable`. The timer is local and
    /// produces the type-5 regeneration request only when it reaches zero.
    pub(super) warp_away_countdown_seconds: Option<f32>,
    /// Channel 12 from `GameCondition.InitCoolTime` is 60 seconds.
    pub(super) warp_away_cooldown_seconds: f32,
    /// True after `SendWarpAway` has handed the request to the network owner
    /// and until regeneration succeeds or the request is reset.
    pub(super) warp_away_request_pending: bool,
    pub(super) system_dialog: Option<TutorialSystemDialogUi>,
    pub(super) tutorial_exit_confirmation_pending: bool,
    /// Clean `NpcIconMode.TopString`: set by a transportation-registration
    /// success reply, drawn in the upper letterbox, cleared by `InitMode`.
    pub(super) npc_top_notice: Option<LocalizedText>,
}

impl Default for MissionUiModel {
    fn default() -> Self {
        Self {
            enabled: false,
            npc_interaction: None,
            npc_icon_mode_visible: false,
            nanocom_main_menu_visible: false,
            nanocom_foreign_modal_suppressed: false,
            nanocom_new_mail_notice_visible: false,
            nanocom_journal: JournalOtherUi::default(),
            journal: MissionJournalUi::Hidden,
            journal_tab: JournalListTab::Active,
            completed_category_expanded: [true; 3],
            journal_scroll: 0.0,
            ui_scale: 1.0,
            viewed_journal_task_id: None,
            remembered_active_journal_task_id: None,
            selected_journal_task_id: None,
            pending: None,
            pending_warp: None,
            warp_transition_active: false,
            warp_away_countdown_seconds: None,
            warp_away_cooldown_seconds: 0.0,
            warp_away_request_pending: false,
            system_dialog: None,
            tutorial_exit_confirmation_pending: false,
            npc_top_notice: None,
        }
    }
}

/// Keeps persistent gameplay presentation state intact while `NpcIconMode`
/// owns the screen. The NPC window itself and its barker layer are separate
/// owners; only HUD chrome should use this gate.
pub(crate) fn gameplay_chrome_visible(
    base_visible: bool,
    mission_ui: Option<&MissionUiModel>,
) -> bool {
    base_visible && !mission_ui.is_some_and(MissionUiModel::npc_letterbox_visible)
}
