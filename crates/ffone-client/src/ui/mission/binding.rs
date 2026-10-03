//! Per-frame binding of the mission UI model onto spawned views.

use super::layout::{JournalRightLayout, NpcUtilityRow};

use super::model::{MissionUiEntry, NpcInteractionUi};

use super::system_dialog::TutorialSystemDialogUi;

use crate::tutorial_mission_content::TutorialNanoJournalUi;

use super::assets::MissionUiAssets;

use super::components::MissionUiView;

use super::geometry::{
    CHAT_QUICK_MENU_RECT, JOURNAL_WINDOW_RECT, NANOCOM_MENU_RECT, NPC_CLOSE_RECT,
    NPC_SINGLE_MISSION_RECT, SYSTEM_DIALOG_PANEL_RECT,
};

use super::journal::current_mission;

use super::layout::{
    MissionUiRect, NpcMissionRowHeights, apply_scaled_group, journal_right_layout, npc_rows,
    npc_utility_panel_height, npc_utility_rows, quest_bottom_height, quest_list_end,
};

use super::model::{MissionJournalUi, MissionUiModel};

use super::nanocom::chat_quick_menu_logical_rect;

use crate::{
    gameplay_ui::GameplayMenuTransition,
    localization::{Language, Localization, LocalizedText},
    tutorial_mission_content::TutorialMissionContent,
};

use bevy::{prelude::*, window::PrimaryWindow};

mod journal_detail_view;
mod journal_list_view;
mod menu_view;
mod npc_view;
use journal_detail_view::{bind_journal_detail_view, bind_journal_nano_view};
use journal_list_view::bind_journal_list_view;
use menu_view::bind_menu_view;
use npc_view::bind_npc_view;

pub(super) fn update_mission_ui_layout(
    model: Res<MissionUiModel>,
    heights: Res<NpcMissionRowHeights>,
    transition: Res<GameplayMenuTransition>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut groups: Query<(&MissionUiView, &mut Node, &mut UiTransform)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
        model.ui_scale
    } else {
        1.0
    };
    let screen_center = viewport * 0.5;
    let npc_pivot = Vec2::new(viewport.x * 0.75, viewport.y * 0.5);
    let quest_height = crate::ui_support::window_height(
        NPC_SINGLE_MISSION_RECT.height,
        quest_list_end(&model, &heights),
        quest_bottom_height(&model),
    );
    let quest_scale = scale.min((viewport.y / quest_height).clamp(0.0, 1.0));

    for (view, mut node, mut transform) in &mut groups {
        match *view {
            MissionUiView::NpcQuestRoot => {
                let logical = MissionUiRect::new(
                    npc_pivot.x - NPC_SINGLE_MISSION_RECT.width * 0.5,
                    npc_pivot.y - quest_height * 0.5,
                    NPC_SINGLE_MISSION_RECT.width,
                    quest_height,
                );
                apply_scaled_group(
                    &mut node,
                    &mut transform,
                    logical.scale_about(npc_pivot, quest_scale),
                    Vec2::new(NPC_SINGLE_MISSION_RECT.width, quest_height),
                    quest_scale,
                );
            }
            MissionUiView::NpcWarpRoot => {
                let height = npc_utility_panel_height(&model);
                let rect = MissionUiRect::new(
                    NPC_CLOSE_RECT.x,
                    npc_pivot.y - height * 0.5,
                    NPC_CLOSE_RECT.width,
                    height,
                );
                let logical = MissionUiRect::new(
                    npc_pivot.x - rect.width * 0.5,
                    npc_pivot.y - rect.height * 0.5,
                    rect.width,
                    rect.height,
                );
                apply_scaled_group(
                    &mut node,
                    &mut transform,
                    logical.scale_about(npc_pivot, scale),
                    Vec2::new(rect.width, rect.height),
                    scale,
                );
            }
            MissionUiView::JournalRoot => {
                let logical = MissionUiRect::new(
                    screen_center.x - JOURNAL_WINDOW_RECT.width * 0.5,
                    screen_center.y - JOURNAL_WINDOW_RECT.height * 0.5,
                    JOURNAL_WINDOW_RECT.width,
                    JOURNAL_WINDOW_RECT.height,
                );
                apply_scaled_group(
                    &mut node,
                    &mut transform,
                    logical.scale_about(screen_center, scale),
                    Vec2::new(JOURNAL_WINDOW_RECT.width, JOURNAL_WINDOW_RECT.height),
                    scale,
                );
            }
            MissionUiView::JournalBackdrop => {
                node.left = px((viewport.x - 1920.0) * 0.5);
                node.top = px((viewport.y - 1440.0) * 0.5);
                node.width = px(1920.0);
                node.height = px(1440.0);
            }
            MissionUiView::NanocomRoot => {
                let pivot = Vec2::new(viewport.x, 0.0);
                let logical = MissionUiRect::new(
                    viewport.x - NANOCOM_MENU_RECT.width,
                    NANOCOM_MENU_RECT.y,
                    NANOCOM_MENU_RECT.width,
                    NANOCOM_MENU_RECT.height,
                );
                let visual = logical.scale_about(pivot, scale);
                apply_scaled_group(
                    &mut node,
                    &mut transform,
                    visual,
                    Vec2::new(NANOCOM_MENU_RECT.width, NANOCOM_MENU_RECT.height),
                    scale,
                );
                node.left = px(visual.x
                    + (visual.width - NANOCOM_MENU_RECT.width) * 0.5
                    + visual.width * transition.slide_offset());
            }
            MissionUiView::ChatQuickRoot => {
                let pivot = Vec2::new(0.0, viewport.y);
                let logical = chat_quick_menu_logical_rect(viewport.y);
                let visual = logical.scale_about(pivot, scale);
                apply_scaled_group(
                    &mut node,
                    &mut transform,
                    visual,
                    Vec2::new(CHAT_QUICK_MENU_RECT.width, CHAT_QUICK_MENU_RECT.height),
                    scale,
                );
                // `CnGuiChat.OnOwnGUI` moves the 170 px `EmoteBoxRect` from
                // exactly `-width` to zero with the shared squared-sine curve.
                node.left = px(visual.x - visual.width * transition.slide_offset());
            }
            MissionUiView::SystemDialogPanel => {
                let logical = MissionUiRect::new(
                    screen_center.x - SYSTEM_DIALOG_PANEL_RECT.width * 0.5 + 10.0,
                    screen_center.y - SYSTEM_DIALOG_PANEL_RECT.height * 0.5 + 10.0,
                    SYSTEM_DIALOG_PANEL_RECT.width,
                    SYSTEM_DIALOG_PANEL_RECT.height,
                );
                apply_scaled_group(
                    &mut node,
                    &mut transform,
                    logical.scale_about(screen_center, scale),
                    Vec2::new(
                        SYSTEM_DIALOG_PANEL_RECT.width,
                        SYSTEM_DIALOG_PANEL_RECT.height,
                    ),
                    scale,
                );
            }
            _ => {}
        }
    }
}

/// Values `bind_mission_ui` computes once per run and shares with every view helper.
#[derive(Clone, Copy)]
struct MissionUiBindContext<'a> {
    model: &'a Res<'a, MissionUiModel>,
    heights: &'a Res<'a, NpcMissionRowHeights>,
    transition: &'a Res<'a, GameplayMenuTransition>,
    assets: &'a Res<'a, MissionUiAssets>,
    content: &'a Res<'a, TutorialMissionContent>,
    asset_server: &'a Res<'a, AssetServer>,
    localization: &'a Option<Res<'a, Localization>>,
    language: &'a Option<Res<'a, Language>>,
    rows: &'a Vec<(&'a MissionUiEntry, f32)>,
    utility_rows: &'a Vec<NpcUtilityRow<'a>>,
    list_end: f32,
    interaction: Option<&'a NpcInteractionUi>,
    mission: Option<&'a MissionUiEntry>,
    nano: Option<&'a TutorialNanoJournalUi>,
    journal_layout: &'a JournalRightLayout<'a>,
    completed_journal: bool,
    active_journal: bool,
    other_journal: bool,
    quest_visible: bool,
    npc_utility_visible: bool,
    journal_visible: bool,
    system_dialog: Option<TutorialSystemDialogUi>,
}

pub(super) fn bind_mission_ui(
    model: Res<MissionUiModel>,
    heights: Res<NpcMissionRowHeights>,
    transition: Res<GameplayMenuTransition>,
    assets: Res<MissionUiAssets>,
    content: Res<TutorialMissionContent>,
    asset_server: Res<AssetServer>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    mut views: Query<(
        &MissionUiView,
        Option<&mut Node>,
        Option<&mut LocalizedText>,
        Option<&mut ImageNode>,
    )>,
) {
    if !model.is_changed()
        && !heights.is_changed()
        && !transition.is_changed()
        && !language
            .as_ref()
            .is_some_and(|language| language.is_changed())
    {
        return;
    }
    let rows = npc_rows(&model, &heights);
    let utility_rows = npc_utility_rows(&model);
    let list_end = quest_list_end(&model, &heights);
    let interaction = model.npc_interaction.as_ref();
    let mission = current_mission(&model);
    let nano = mission.and_then(|mission| mission.nano.as_ref());
    let journal_layout = journal_right_layout(&model);
    let completed_journal = journal_layout.completed;
    let active_journal = matches!(model.journal, MissionJournalUi::Other(_)) && !completed_journal;
    let other_journal = matches!(model.journal, MissionJournalUi::Other(_));
    let quest_visible = model.enabled
        && model.npc_icon_mode_visible
        && interaction.is_some_and(|npc| {
            !npc.available_missions.is_empty() || !npc.completed_missions.is_empty()
        });
    let npc_utility_visible =
        model.enabled && model.npc_icon_mode_visible && !quest_visible && interaction.is_some();
    let journal_visible = model.enabled && !matches!(model.journal, MissionJournalUi::Hidden);
    let system_dialog = model.system_dialog();

    let context = MissionUiBindContext {
        model: &model,
        heights: &heights,
        transition: &transition,
        assets: &assets,
        content: &content,
        asset_server: &asset_server,
        localization: &localization,
        language: &language,
        rows: &rows,
        utility_rows: &utility_rows,
        list_end,
        interaction,
        mission,
        nano,
        journal_layout: &journal_layout,
        completed_journal,
        active_journal,
        other_journal,
        quest_visible,
        npc_utility_visible,
        journal_visible,
        system_dialog,
    };

    for (view, node, localized_text, image) in &mut views {
        match *view {
            MissionUiView::NpcLetterboxRoot
            | MissionUiView::NpcTopNotice
            | MissionUiView::NpcTopNoticeText
            | MissionUiView::NpcQuestRoot
            | MissionUiView::NpcWarpRoot
            | MissionUiView::NpcName
            | MissionUiView::NpcCompletedBody
            | MissionUiView::NpcAvailableBody
            | MissionUiView::NpcQuestUtilityButton
            | MissionUiView::NpcQuestUtilityText
            | MissionUiView::NpcQuestUtilityIcon
            | MissionUiView::NpcQuestCloseButton
            | MissionUiView::NpcUtilityButton (_)
            | MissionUiView::NpcUtilityText (_)
            | MissionUiView::NpcUtilityIcon (_)
            | MissionUiView::NpcCloseButton
            | MissionUiView::NpcCompletedHeader
            | MissionUiView::NpcAvailableHeader
            | MissionUiView::NpcMissionRow (_)
            | MissionUiView::NpcMissionRowIcon (_)
            | MissionUiView::NpcMissionRowText (_)
            | MissionUiView::NpcQuestBottom => {
                bind_npc_view(&context, view, node, localized_text, image)
            }
            MissionUiView::JournalRoot
            | MissionUiView::JournalBackdrop
            | MissionUiView::JournalBackground
            | MissionUiView::JournalEmptySelectionPanel
            | MissionUiView::JournalEmptySelectionText
            | MissionUiView::JournalMissionBanner
            | MissionUiView::JournalWindowTitle
            | MissionUiView::JournalModeTitle
            | MissionUiView::JournalMissionName
            | MissionUiView::JournalMissionDifficulty
            | MissionUiView::JournalNpcName
            | MissionUiView::JournalNpcPosition
            | MissionUiView::JournalNpcPortrait
            | MissionUiView::JournalObjectiveHeader
            | MissionUiView::JournalDescription
            | MissionUiView::JournalRewardHeader
            | MissionUiView::JournalRewardCard
            | MissionUiView::JournalCashCard
            | MissionUiView::JournalCashAmount
            | MissionUiView::JournalRewardAmount
            | MissionUiView::JournalPrimary
            | MissionUiView::JournalPrimaryText => {
                bind_journal_detail_view(&context, view, node, localized_text, image)
            }
            MissionUiView::JournalNanoLabel
            | MissionUiView::JournalNanoGroup
            | MissionUiView::JournalNanoPortrait
            | MissionUiView::JournalNanoName
            | MissionUiView::JournalNanoAttribute
            | MissionUiView::JournalNanoDescription
            | MissionUiView::JournalNanoBox (_)
            | MissionUiView::JournalNanoSkillIcon (_)
            | MissionUiView::JournalNanoSkillName (_)
            | MissionUiView::JournalNanoSkillType (_)
            | MissionUiView::JournalNanoSkillDescription (_) => {
                bind_journal_nano_view(&context, view, node, localized_text, image)
            }
            MissionUiView::JournalAllowListPanel
            | MissionUiView::JournalAllowListTitle
            | MissionUiView::JournalActiveListPanel
            | MissionUiView::JournalActiveListTitle
            | MissionUiView::JournalCompletedListPanel
            | MissionUiView::JournalCompletedListTitle
            | MissionUiView::JournalActiveTab
            | MissionUiView::JournalCompletedTab
            | MissionUiView::JournalRightFrame
            | MissionUiView::JournalCategoryHeader (_)
            | MissionUiView::JournalCategoryCount (_)
            | MissionUiView::JournalCategoryIcon (_)
            | MissionUiView::JournalCategoryToggle (_)
            | MissionUiView::JournalEmptyRow (_)
            | MissionUiView::JournalMissionRow (_)
            | MissionUiView::JournalMissionRowSelectedTitle (_)
            | MissionUiView::JournalMissionRowSelectedObjective (_)
            | MissionUiView::JournalMissionRowUnselectedTitle (_)
            | MissionUiView::JournalMissionRowUnselectedObjective (_)
            | MissionUiView::JournalAllowSecondary
            | MissionUiView::JournalRewardSecondary
            | MissionUiView::JournalActiveDelete
            | MissionUiView::JournalActiveMakeCurrent
            | MissionUiView::JournalMissionRowSelection (_)
            | MissionUiView::JournalMissionRowPointer (_)
            | MissionUiView::JournalMissionRowPortraitFrame (_)
            | MissionUiView::JournalMissionRowCurrent (_)
            | MissionUiView::JournalMissionRowTrackedField (_)
            | MissionUiView::JournalMissionRowPortrait (_)
            | MissionUiView::JournalMissionRowCheck (_) => {
                bind_journal_list_view(&context, view, node, localized_text, image)
            }
            MissionUiView::NanocomRoot
            | MissionUiView::ChatQuickRoot
            | MissionUiView::WarpAwayCountdownRoot
            | MissionUiView::WarpAwayCountdownText
            | MissionUiView::SystemDialogRoot
            | MissionUiView::SystemDialogPanel
            | MissionUiView::SystemDialogText => {
                bind_menu_view(&context, view, node, localized_text, image)
            }
        }
    }
}
