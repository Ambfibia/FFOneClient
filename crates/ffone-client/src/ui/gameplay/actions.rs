//! Typed HUD actions, NPC service kinds, audio cues and their outboxes.

use super::chat_model::ChatChannel;
use super::quick_chat::QuickChatItem;
use crate::{tutorial_mission_content::TutorialWarpTarget, user_equip_ui::UserEquipOpenSource};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NpcServiceKind {
    Barber,
    Vendor,
    NanoStation,
    Bank,
    LocalBank,
    GuideChanger,
    PastWarp,
    TransportationWarp,
    TransportationWyvern,
    Race,
    RaceRank,
    Combine,
    Enchant,
    Rule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameplayUiAction {
    SelectChatChannel(ChatChannel),
    ToggleMenuChat,
    ToggleEmotes,
    SelectQuickChatItem(QuickChatItem),
    SendChat(String),
    /// Clean `CnGuiChat.WarpAway`: start the visible 20-second countdown and
    /// take the shared SystemMessage focus-out latch.
    WarpAwayStarted,
    /// Clean `CnGuiChat.SendWarpAway`: request regeneration type 5 after the
    /// local countdown expires.
    RequestWarpAway,
    /// The Enter-key Nanocom menu transition from `cnGUINanocom.SetViewMenu`.
    OpenNanocomMenu,
    CloseNanocomMenu,
    /// Clean `cnGUINanocom.OnStatus`, reachable through the exact first
    /// `MY STUFF` menu button. No guessed configurable-key binding is exposed.
    OpenUserEquipItemMode {
        source: UserEquipOpenSource,
    },
    /// Clean `cnGUINanocom.OnOption`: the SETTINGS row requests mode 12,
    /// then closes the Nanocom menu through event `(2, 3, 0)`.
    OpenOptionFromNanocomSettings,
    /// Clean `cnGUINanocom.OnEMail`: request mode 18, then close the Nanocom
    /// menu through event `(2, 3, 0)`.
    OpenEmailFromNanocom,
    /// Clean `cnGUINanocom.OnWorldMap`: request mode 15, then close the
    /// Nanocom menu through event `(2, 3, 0)`.
    OpenWorldMapFromNanocom,
    /// Clean `cnGUINanocom.OnGuide`: send event `(2, 18)`. This event does not
    /// close the Nanocom menu.
    OpenGameGuideFromNanocom,
    /// Clean `cnGUINanocom.OnQuitMenu`: request mode 23. This event does not
    /// close the Nanocom menu.
    OpenQuitFromNanocom,
    /// The journal list opened by the Nanocom JOURNAL button.
    OpenMissionJournal,
    CloseMissionJournal,
    /// The normal-world Active journal's DELETE MISSION button requests the
    /// source SystemMessage confirmation before any TaskStop packet exists.
    RequestTaskStopConfirmation {
        task_id: i32,
    },
    /// SystemMessage 47 (`OKAY/CANCEL`) confirms the original tutorial-exit path.
    ConfirmTutorialExit {
        message_id: i32,
        button_type: i32,
    },
    /// A mission row selected from `NpcIconMode`.
    OpenMissionAllow {
        task_id: i32,
        npc_id: i32,
    },
    OpenMissionReward {
        task_id: i32,
        npc_id: i32,
    },
    /// `cnMissionJournal` Accept. The UI remains locked until its server reply.
    TaskStart {
        task_id: i32,
        npc_id: i32,
    },
    /// `cnMissionJournal.CheckComplete`. The UI remains locked until its server reply.
    QuestEnd {
        task_id: i32,
        npc_id: i32,
        box1_choice: i32,
        box2_choice: i32,
    },
    NpcWarp {
        npc_id: i32,
        npc_type: i32,
        warp_id: i32,
        required_task_id: Option<i32>,
        target: TutorialWarpTarget,
    },
    NpcService {
        npc_id: i32,
        service: NpcServiceKind,
    },
    NpcIconClose {
        npc_id: i32,
    },
}

#[derive(Default, Resource)]
pub struct GameplayUiOutbox {
    pub(super) actions: VecDeque<GameplayUiAction>,
}

impl GameplayUiOutbox {
    pub fn push(&mut self, action: GameplayUiAction) {
        self.actions.push_back(action);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = GameplayUiAction> + '_ {
        self.actions.drain(..)
    }

    pub fn drain_matching(
        &mut self,
        mut matches: impl FnMut(&GameplayUiAction) -> bool,
    ) -> Vec<GameplayUiAction> {
        let mut selected = Vec::new();
        let mut retained = VecDeque::new();
        for action in self.actions.drain(..) {
            if matches(&action) {
                selected.push(action);
            } else {
                retained.push_back(action);
            }
        }
        self.actions = retained;
        selected
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

/// Clean `SoundUtil` calls owned by gameplay HUD surfaces. Keeping these cues
/// separate from semantic UI actions preserves exact call order without
/// teaching the presentation crate how the native audio catalog is played.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameplayUiAudioCue {
    SelectColor,
    ScrollDown,
    ActionFailure,
    ButtonSound,
    TabClick01,
    ClickWindowSlideOut,
    OpenScreen,
    CloseScreen,
    MissionDecline,
    MissionAccepted,
    AbandonMission,
    YesButton,
    NoButton,
    HeightUp,
    HeightDown,
}

impl GameplayUiAudioCue {
    #[must_use]
    pub const fn true_name(self) -> Option<&'static str> {
        match self {
            Self::SelectColor => Some("select_color"),
            Self::ScrollDown => Some("scroll_down"),
            Self::ActionFailure => Some("action_failure01"),
            Self::ButtonSound => None,
            Self::TabClick01 => Some("Tab_Click01"),
            Self::ClickWindowSlideOut => Some("Click_WindowSlideOut"),
            Self::OpenScreen => Some("Open_Screen"),
            Self::CloseScreen => Some("Close_Screen"),
            Self::MissionDecline => Some("Mission_Decline"),
            Self::MissionAccepted => Some("Mission_Accepted"),
            Self::AbandonMission => Some("Abandon_Mission"),
            Self::YesButton => Some("Yes_Button"),
            Self::NoButton => Some("No_Button"),
            Self::HeightUp => Some("Height_Up"),
            Self::HeightDown => Some("Height_Down"),
        }
    }
}

#[derive(Default, Resource)]
pub struct GameplayUiAudioOutbox {
    pub(super) cues: VecDeque<GameplayUiAudioCue>,
}

impl GameplayUiAudioOutbox {
    pub fn push(&mut self, cue: GameplayUiAudioCue) {
        self.cues.push_back(cue);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = GameplayUiAudioCue> + '_ {
        self.cues.drain(..)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }
}
