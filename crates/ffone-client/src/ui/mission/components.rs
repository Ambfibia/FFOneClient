//! View, control and marker components shared by the mission UI entities.

use super::layout::MissionUiRect;
use super::model::JournalListTab;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum MissionUiView {
    NpcLetterboxRoot,
    NpcTopNotice,
    NpcTopNoticeText,
    NpcQuestRoot,
    NpcWarpRoot,
    NpcCompletedBody,
    NpcAvailableBody,
    NpcCompletedHeader,
    NpcAvailableHeader,
    NpcQuestUtilityButton,
    NpcQuestUtilityText,
    NpcQuestUtilityIcon,
    NpcQuestCloseButton,
    NpcUtilityButton(usize),
    NpcUtilityText(usize),
    NpcUtilityIcon(usize),
    NpcCloseButton,
    NpcMissionRow(usize),
    NpcMissionRowIcon(usize),
    NpcMissionRowText(usize),
    NpcQuestBottom,
    NpcName,
    JournalRoot,
    JournalBackdrop,
    JournalBackground,
    JournalMissionBanner,
    JournalWindowTitle,
    JournalModeTitle,
    JournalMissionName,
    JournalMissionDifficulty,
    JournalNpcName,
    JournalNpcPosition,
    JournalObjectiveHeader,
    JournalDescription,
    JournalRewardHeader,
    JournalRewardCard,
    JournalRewardAmount,
    JournalCashCard,
    JournalCashAmount,
    JournalNanoLabel,
    JournalNanoGroup,
    JournalNanoPortrait,
    JournalNanoName,
    JournalNanoAttribute,
    JournalNanoDescription,
    JournalNanoBox(usize),
    JournalNanoSkillIcon(usize),
    JournalNanoSkillName(usize),
    JournalNanoSkillType(usize),
    JournalNanoSkillDescription(usize),
    JournalPrimary,
    JournalPrimaryText,
    JournalAllowListPanel,
    JournalAllowListTitle,
    JournalActiveListPanel,
    JournalActiveListTitle,
    JournalCompletedListPanel,
    JournalCompletedListTitle,
    JournalMissionRow(usize),
    JournalMissionRowSelectedTitle(usize),
    JournalMissionRowSelectedObjective(usize),
    JournalMissionRowUnselectedTitle(usize),
    JournalMissionRowUnselectedObjective(usize),
    JournalMissionRowSelection(usize),
    JournalMissionRowTrackedField(usize),
    JournalMissionRowCurrent(usize),
    JournalMissionRowPointer(usize),
    JournalMissionRowPortraitFrame(usize),
    JournalMissionRowPortrait(usize),
    JournalMissionRowCheck(usize),
    JournalNpcPortrait,
    JournalCategoryHeader(usize),
    JournalCategoryCount(usize),
    JournalCategoryIcon(usize),
    JournalCategoryToggle(usize),
    JournalEmptyRow(usize),
    JournalActiveTab,
    JournalCompletedTab,
    JournalEmptySelectionPanel,
    JournalEmptySelectionText,
    JournalRightFrame,
    JournalAllowSecondary,
    JournalRewardSecondary,
    JournalActiveDelete,
    JournalActiveMakeCurrent,
    NanocomRoot,
    ChatQuickRoot,
    WarpAwayCountdownRoot,
    WarpAwayCountdownText,
    SystemDialogPanel,
    SystemDialogRoot,
    SystemDialogText,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum MissionUiControl {
    NpcMissionRow(usize),
    NpcUtility(usize),
    NpcClose,
    JournalPrimary,
    JournalClose,
    JournalRewardIcon,
    JournalMissionRow(usize),
    JournalTrackMission(usize),
    JournalActiveTab,
    JournalCompletedTab,
    JournalCategoryToggle(usize),
    JournalHelp,
    JournalSecondary,
    JournalMakeCurrent,
    NanocomMyStuff,
    NanocomJournal,
    NanocomEmail,
    NanocomMap,
    NanocomSettings,
    NanocomGameGuide,
    NanocomExitGame,
    NanocomClose,
    WarpAway,
    SystemDialogOkay,
    SystemDialogCancel,
}

#[derive(Component)]
#[require(crate::ui::shared::controller::ControllerUiDefault)]
pub(super) struct NpcMissionButton;

#[derive(Component)]
#[require(crate::ui::shared::controller::ControllerUiDefault)]
pub(super) struct NpcActionButton;

#[derive(Component)]
pub(super) struct JournalMissionButton;

#[derive(Component)]
pub(super) struct JournalCloseButton;

#[derive(Component)]
pub(super) struct JournalTabButton(pub(super) JournalListTab);

#[derive(Component)]
pub(super) struct JournalTabBackground(pub(super) JournalListTab);

#[derive(Component)]
pub(super) struct JournalCategoryToggleButton(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum JournalAuxButtonKind {
    Blue,
    Red,
    Cancel,
    Help,
}

#[derive(Component)]
pub(super) struct JournalPrimaryButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum MenuButtonStyle {
    HudBlue,
    HudRed,
    ChatRed,
    ChatBlue,
}

impl MenuButtonStyle {
    pub(super) const fn skin_and_style(self) -> (&'static str, &'static str) {
        match self {
            Self::HudBlue => ("FusionFallHUDSkin", "button"),
            Self::HudRed => ("FusionFallHUDSkin", "RedButton"),
            Self::ChatRed => ("FusionFallChatSkin", "RedButton"),
            Self::ChatBlue => ("FusionFallChatSkin", "BlueButton"),
        }
    }

    pub(super) const fn is_red(self) -> bool {
        matches!(self, Self::HudRed | Self::ChatRed)
    }
}

#[derive(Component)]
pub(super) struct NanocomMenuButton(pub(super) MenuButtonStyle);

#[derive(Component)]
pub(super) struct ChatQuickMenuButton {
    pub(super) style: MenuButtonStyle,
    pub(super) enabled: bool,
}

#[derive(Component)]
pub(super) struct NanocomCloseButton;

#[derive(Component)]
pub(super) struct SystemDialogButton;

#[derive(Clone, Copy, Debug, Component)]
pub(super) struct JournalContentRect {
    pub(super) offer_rect: MissionUiRect,
    pub(super) active_rect: MissionUiRect,
    pub(super) completed_rect: MissionUiRect,
    pub(super) cash_element: bool,
}

/// Vertical half of Unity's `TextAnchor` for a direct IMGUI label.
///
/// Bevy's `TextLayout::justify` only carries the horizontal half, so middle
/// and lower anchored Journal labels need a separate post-layout glyph
/// offset. This is especially visible in the selected mission row, where the
/// source centers a one-line objective inside a 50 px `GUI.Label` rectangle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum MissionTextVerticalAnchor {
    Middle,
    Lower,
}

impl JournalContentRect {
    pub(super) const fn standard(offer_rect: MissionUiRect) -> Self {
        Self {
            offer_rect,
            active_rect: offer_rect.translated(1.0, 7.0),
            completed_rect: offer_rect.translated(1.0, 7.0),
            cash_element: false,
        }
    }

    pub(super) const fn objective_header(offer_rect: MissionUiRect) -> Self {
        Self {
            offer_rect,
            active_rect: offer_rect.translated(1.0, 7.0),
            completed_rect: MissionUiRect::new(
                offer_rect.x - 63.0,
                offer_rect.y + 32.0,
                offer_rect.width + 200.0,
                offer_rect.height,
            ),
            cash_element: false,
        }
    }

    pub(super) const fn description(offer_rect: MissionUiRect) -> Self {
        Self {
            offer_rect,
            active_rect: offer_rect.translated(1.0, 7.0),
            completed_rect: MissionUiRect::new(
                offer_rect.x - 23.0,
                offer_rect.y + 33.0,
                offer_rect.width + 25.0,
                offer_rect.height,
            ),
            cash_element: false,
        }
    }

    pub(super) const fn reward(offer_rect: MissionUiRect) -> Self {
        Self {
            offer_rect,
            active_rect: offer_rect.translated(1.0, 7.0),
            completed_rect: offer_rect.translated(1.0, 32.0),
            cash_element: false,
        }
    }

    pub(super) const fn cash(offer_rect: MissionUiRect) -> Self {
        Self {
            offer_rect,
            active_rect: offer_rect.translated(1.0, 7.0),
            completed_rect: offer_rect.translated(1.0, 32.0),
            cash_element: true,
        }
    }
}
