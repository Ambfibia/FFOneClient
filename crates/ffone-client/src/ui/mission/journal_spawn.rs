//! Spawning of the mission journal window and its scroll list.

use super::assets::{
    ACCEPT_BUTTON, JOURNAL_ACTIVE_PANEL, JOURNAL_ACTIVE_TAB, JOURNAL_ALLOW_RIGHT,
    JOURNAL_COMPLETED_PANEL, JOURNAL_COMPLETED_TAB, JOURNAL_EMPTY_BOX, JOURNAL_NANO_POWER,
    JOURNAL_NPC_ICON_BACK, JOURNAL_REWARD_BOX, JOURNAL_RIGHT_FRAME, JOURNAL_SELECTED_MISSION_BACK,
    JOURNAL_TRACKED_MISSION_BACK, JOURNAL_WINDOW, MISSION_BACK, MissionUiAssets, OFFER_DIALOG,
};
use super::components::{
    JournalAuxButtonKind, JournalCategoryToggleButton, JournalCloseButton, JournalContentRect,
    JournalMissionButton, JournalPrimaryButton, JournalTabBackground, JournalTabButton,
    MissionUiControl, MissionUiView,
};
use super::geometry::{
    JOURNAL_ACCEPT_RECT, JOURNAL_ACTIVE_DELETE_RECT, JOURNAL_ACTIVE_MAKE_CURRENT_RECT,
    JOURNAL_ACTIVE_TAB_RECT, JOURNAL_ALLOW_GROUP_RECT, JOURNAL_ALLOW_SECONDARY_RECT,
    JOURNAL_CLOSE_RECT, JOURNAL_COMPLETED_TAB_RECT, JOURNAL_DESCRIPTION_RECT,
    JOURNAL_FM_AMOUNT_RECT, JOURNAL_FM_CAPTION_RECT, JOURNAL_FM_CARD_RECT, JOURNAL_FM_ICON_RECT,
    JOURNAL_HELP_RECT, JOURNAL_LEFT_BOX_RECT, JOURNAL_MISSION_DIFFICULTY_RECT,
    JOURNAL_MISSION_TITLE_RECT, JOURNAL_NPC_FRAME_RECT, JOURNAL_NPC_NAME_RECT,
    JOURNAL_NPC_POSITION_RECT, JOURNAL_OBJECTIVE_HEADER_RECT, JOURNAL_REWARD_HEADER_RECT,
    JOURNAL_RIGHT_BOX_RECT, JOURNAL_SCROLL_VIEW_RECT, JOURNAL_TAROS_AMOUNT_RECT,
    JOURNAL_TAROS_CAPTION_RECT, JOURNAL_TAROS_CARD_RECT, JOURNAL_TAROS_ICON_RECT,
    JOURNAL_WINDOW_RECT,
};
use super::journal::{JournalScrollContent, JournalScrollViewport};
use super::labels::{
    JOURNAL_FUSION_MATTER_LABEL, JOURNAL_MISSION_SUMMARY_LABEL, JOURNAL_REWARD_LABEL,
    JOURNAL_TAROS_LABEL,
};
use super::layout::{JOURNAL_CATEGORY_LABELS, JOURNAL_MISSION_ROW_LIMIT, MissionUiRect};
use super::model::JournalListTab;
use super::widgets::{
    journal_aux_text_button, journal_managed_label, skin_label, skin_label_with_font_size,
    skin_style_background_rect, stretch, styled_image,
};
use crate::gui_skin::gui_style as mission_gui_style;
use bevy::{
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
};

pub(super) fn spawn_journal(root: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    root.spawn((
        MissionUiRect::new(0.0, 0.0, 1920.0, 1440.0).node(),
        stretch(&assets.journal_backdrop),
        MissionUiView::JournalBackdrop,
        Pickable::IGNORE,
    ));
    root.spawn((
        JOURNAL_WINDOW_RECT.node(),
        styled_image(&assets.journal_window, JOURNAL_WINDOW),
        UiTransform::default(),
        MissionUiView::JournalRoot,
        Pickable::IGNORE,
    ))
    .with_children(|journal| {
        skin_label(
            journal,
            MissionUiRect::new(0.0, 0.0, 560.0, 27.0),
            "Mission Journal",
            "FusionFallMissionSkin",
            "box",
            assets,
            Some(MissionUiView::JournalWindowTitle),
            None,
        );
        journal.spawn((
            JOURNAL_RIGHT_BOX_RECT.node(),
            styled_image(&assets.journal_allow_right, JOURNAL_ALLOW_RIGHT),
            MissionUiView::JournalAllowListPanel,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        skin_label(
            journal,
            JOURNAL_RIGHT_BOX_RECT,
            "Active Mission List",
            "FusionFallMissionSkinR",
            "AllowTab",
            assets,
            Some(MissionUiView::JournalAllowListTitle),
            None,
        );
        // `DoWindowRight` paints the unselected Active toggle first and the
        // selected Completed box over it. Keep the background separate from
        // the 100x30 hit rect so GUIStyle.overflow can extend the texture.
        let active_tab_style = mission_gui_style("FusionFallMissionSkinR", "Tab1")
            .expect("Retrobution journal Active tab style");
        journal.spawn((
            skin_style_background_rect(JOURNAL_ACTIVE_TAB_RECT, active_tab_style).node(),
            styled_image(&assets.journal_active_tab, JOURNAL_ACTIVE_TAB),
            MissionUiView::JournalActiveTab,
            JournalTabBackground(JournalListTab::Active),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        journal
            .spawn((
                Button,
                JOURNAL_ACTIVE_TAB_RECT.node(),
                BackgroundColor(Color::NONE),
                MissionUiControl::JournalActiveTab,
                MissionUiView::JournalActiveTab,
                JournalTabButton(JournalListTab::Active),
                crate::ui::shared::controller::ControllerUiTab(0),
            ))
            .with_children(|tab| {
                skin_label(
                    tab,
                    MissionUiRect::new(0.0, 0.0, 100.0, 30.0),
                    "Active",
                    "FusionFallMissionSkinR",
                    "Tab1",
                    assets,
                    None,
                    None,
                );
            });
        journal.spawn((
            JOURNAL_RIGHT_BOX_RECT.node(),
            styled_image(&assets.journal_completed_panel, JOURNAL_COMPLETED_PANEL),
            MissionUiView::JournalCompletedListPanel,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        skin_label(
            journal,
            JOURNAL_RIGHT_BOX_RECT,
            "Completed",
            "FusionFallMissionSkinR",
            "Compl_Tab",
            assets,
            Some(MissionUiView::JournalCompletedListTitle),
            None,
        );

        // Active mode follows the inverse source order: the unselected
        // Completed toggle is painted before the selected Active box.
        let completed_tab_style = mission_gui_style("FusionFallMissionSkinR", "Tab2")
            .expect("Retrobution journal Completed tab style");
        journal.spawn((
            skin_style_background_rect(JOURNAL_COMPLETED_TAB_RECT, completed_tab_style).node(),
            styled_image(&assets.journal_completed_tab, JOURNAL_COMPLETED_TAB),
            MissionUiView::JournalCompletedTab,
            JournalTabBackground(JournalListTab::Completed),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        journal
            .spawn((
                Button,
                JOURNAL_COMPLETED_TAB_RECT.node(),
                BackgroundColor(Color::NONE),
                MissionUiControl::JournalCompletedTab,
                MissionUiView::JournalCompletedTab,
                JournalTabButton(JournalListTab::Completed),
                crate::ui::shared::controller::ControllerUiTab(1),
            ))
            .with_children(|tab| {
                skin_label(
                    tab,
                    MissionUiRect::new(0.0, 0.0, 100.0, 30.0),
                    "Completed",
                    "FusionFallMissionSkinR",
                    "Tab2",
                    assets,
                    None,
                    None,
                );
            });
        journal.spawn((
            JOURNAL_RIGHT_BOX_RECT.node(),
            styled_image(&assets.journal_active_panel, JOURNAL_ACTIVE_PANEL),
            MissionUiView::JournalActiveListPanel,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        skin_label(
            journal,
            JOURNAL_RIGHT_BOX_RECT,
            "Active",
            "FusionFallMissionSkinR",
            "Active_Tab",
            assets,
            Some(MissionUiView::JournalActiveListTitle),
            None,
        );
        // `DoWindowRight` draws the category headers and mission rows inside
        // `GUI.BeginScrollView`. The content node keeps the recovered
        // journal-space rectangles and only moves by the scroll offset. Clip
        // vertically only: the selected row's pointer is authored left of the
        // view.
        journal
            .spawn((
                Node {
                    overflow: Overflow::clip_y(),
                    ..JOURNAL_SCROLL_VIEW_RECT.node()
                },
                RelativeCursorPosition::default(),
                JournalScrollViewport,
                FocusPolicy::Pass,
                Pickable::IGNORE,
            ))
            .with_children(|viewport| {
                viewport
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(-JOURNAL_SCROLL_VIEW_RECT.x),
                            top: px(-JOURNAL_SCROLL_VIEW_RECT.y),
                            ..default()
                        },
                        JournalScrollContent,
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ))
                    .with_children(|content| spawn_journal_scroll_list(content, assets));
            });
        // `DoWindowRight` draws this after `EndScrollView`, so its 30 px
        // top/bottom Shadow borders mask the scroll contents at both edges.
        // This full-list overlay must pass ui_focus_system hits to the rows;
        // Pickable::IGNORE alone only affects the separate picking backend.
        journal.spawn((
            MissionUiRect::new(
                JOURNAL_RIGHT_BOX_RECT.x + 5.0,
                JOURNAL_RIGHT_BOX_RECT.y + 32.0,
                350.0,
                571.0,
            )
            .node(),
            styled_image(&assets.journal_right_frame, JOURNAL_RIGHT_FRAME),
            MissionUiView::JournalRightFrame,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        journal.spawn((
            JOURNAL_ALLOW_GROUP_RECT.node(),
            styled_image(&assets.offer_dialog, OFFER_DIALOG),
            MissionUiView::JournalBackground,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        journal.spawn((
            JOURNAL_LEFT_BOX_RECT.node(),
            styled_image(&assets.journal_empty_box, JOURNAL_EMPTY_BOX),
            MissionUiView::JournalEmptySelectionPanel,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        skin_label(
            journal,
            JOURNAL_LEFT_BOX_RECT,
            "No mission selected",
            "FusionFallMissionSkin",
            "EmptyBox",
            assets,
            Some(MissionUiView::JournalEmptySelectionText),
            None,
        );
        journal.spawn((
            MissionUiRect::new(21.0, 31.0, 549.0, 74.0).node(),
            ImageNode::new(assets.journal_world_banner.clone()),
            MissionUiView::JournalMissionBanner,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        skin_label(
            journal,
            // `rectofferdlg + Rect(16, 2, 567, 630)`.
            MissionUiRect::new(24.0, 13.0, 567.0, 630.0),
            "",
            "FusionFallMissionSkin",
            "OfferBack",
            assets,
            Some(MissionUiView::JournalModeTitle),
            None,
        );
        skin_label(
            journal,
            JOURNAL_MISSION_TITLE_RECT,
            "",
            "FusionFallMissionSkin",
            "bigfont16",
            assets,
            Some(MissionUiView::JournalMissionName),
            None,
        );
        skin_label(
            journal,
            JOURNAL_MISSION_DIFFICULTY_RECT,
            "",
            "FusionFallMissionSkin",
            "textArea",
            assets,
            Some(MissionUiView::JournalMissionDifficulty),
            Some(Color::srgb(0.8, 1.0, 1.0)),
        );
        journal.spawn((
            JOURNAL_NPC_FRAME_RECT.node(),
            styled_image(&assets.journal_npc_icon_back, JOURNAL_NPC_ICON_BACK),
            FocusPolicy::Pass,
            Pickable::IGNORE,
            JournalContentRect::standard(JOURNAL_NPC_FRAME_RECT),
        ));
        let journal_portrait_rect = MissionUiRect::new(
            JOURNAL_NPC_FRAME_RECT.x + 3.0,
            JOURNAL_NPC_FRAME_RECT.y + 4.0,
            62.0,
            62.0,
        );
        journal.spawn((
            journal_portrait_rect.node(),
            ImageNode::new(assets.journal_npc_portraits[0].clone()),
            MissionUiView::JournalNpcPortrait,
            FocusPolicy::Pass,
            Pickable::IGNORE,
            JournalContentRect::standard(journal_portrait_rect),
        ));
        skin_label(
            journal,
            JOURNAL_NPC_NAME_RECT,
            "",
            "FusionFallMissionSkin",
            "textArea",
            assets,
            Some(MissionUiView::JournalNpcName),
            Some(Color::srgb(1.0, 1.0, 0.0)),
        );
        skin_label(
            journal,
            JOURNAL_NPC_POSITION_RECT,
            "",
            "FusionFallMissionSkin",
            "bigfont14",
            assets,
            Some(MissionUiView::JournalNpcPosition),
            Some(Color::srgb(0.8, 1.0, 1.0)),
        );
        skin_label(
            journal,
            JOURNAL_OBJECTIVE_HEADER_RECT,
            journal_managed_label(JOURNAL_MISSION_SUMMARY_LABEL),
            "FusionFallMissionSkin",
            "textArea",
            assets,
            Some(MissionUiView::JournalObjectiveHeader),
            Some(Color::srgb(0.0, 0.4, 0.6)),
        );
        skin_label(
            journal,
            JOURNAL_DESCRIPTION_RECT,
            "",
            "FusionFallMissionSkin",
            "label",
            assets,
            Some(MissionUiView::JournalDescription),
            Some(Color::srgb(0.0, 0.19, 0.4)),
        );
        skin_label(
            journal,
            JOURNAL_REWARD_HEADER_RECT,
            journal_managed_label(JOURNAL_REWARD_LABEL),
            "FusionFallMissionSkin",
            "bigfont14",
            assets,
            Some(MissionUiView::JournalRewardHeader),
            Some(Color::srgb(0.0, 0.19, 0.4)),
        );
        skin_label(
            journal,
            MissionUiRect::new(23.0, 301.0, 200.0, 30.0),
            "NANO:",
            "FusionFallMissionSkin",
            "bigfont14",
            assets,
            Some(MissionUiView::JournalNanoLabel),
            Some(Color::srgb(0.0, 0.19, 0.4)),
        );
        journal
            .spawn((
                MissionUiRect::new(13.0, 318.0, 555.0, 245.0).node(),
                MissionUiView::JournalNanoGroup,
                FocusPolicy::Pass,
                Pickable::IGNORE,
            ))
            .with_children(|nano| {
                nano.spawn((
                    MissionUiRect::new(25.0, 10.0, 128.0, 128.0).node(),
                    ImageNode::default(),
                    MissionUiView::JournalNanoPortrait,
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                ));
                skin_label(
                    nano,
                    MissionUiRect::new(10.0, 150.0, 160.0, 20.0),
                    "",
                    "FusionFallMissionSkin",
                    "bigfont14",
                    assets,
                    Some(MissionUiView::JournalNanoName),
                    None,
                );
                skin_label(
                    nano,
                    MissionUiRect::new(10.0, 165.0, 160.0, 20.0),
                    "",
                    "FusionFallMissionSkin",
                    "bigfont8",
                    assets,
                    Some(MissionUiView::JournalNanoAttribute),
                    None,
                );
                skin_label(
                    nano,
                    MissionUiRect::new(10.0, 175.0, 160.0, 50.0),
                    "",
                    "FusionFallMissionSkin",
                    "bigfont8",
                    assets,
                    Some(MissionUiView::JournalNanoDescription),
                    Some(Color::WHITE),
                );
                for skill in 0..3 {
                    let top = skill as f32 * 80.0;
                    nano.spawn((
                        MissionUiRect::new(206.0, top + 5.0, 330.0, 75.0).node(),
                        styled_image(&assets.journal_nano_power, JOURNAL_NANO_POWER),
                        MissionUiView::JournalNanoBox(skill),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ));
                    nano.spawn((
                        MissionUiRect::new(210.0, top + 10.0, 28.0, 28.0).node(),
                        ImageNode::new(assets.journal_nano_skill_back.clone()),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ));
                    nano.spawn((
                        MissionUiRect::new(208.0, top + 8.0, 32.0, 32.0).node(),
                        ImageNode::default(),
                        MissionUiView::JournalNanoSkillIcon(skill),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ));
                    skin_label(
                        nano,
                        MissionUiRect::new(248.0, top + 13.0, 230.0, 20.0),
                        "",
                        "FusionFallMissionSkin",
                        "bigfont14",
                        assets,
                        Some(MissionUiView::JournalNanoSkillName(skill)),
                        None,
                    );
                    skin_label(
                        nano,
                        MissionUiRect::new(248.0, top + 27.0, 230.0, 15.0),
                        "",
                        "FusionFallMissionSkin",
                        "bigfont8",
                        assets,
                        Some(MissionUiView::JournalNanoSkillType(skill)),
                        None,
                    );
                    skin_label(
                        nano,
                        MissionUiRect::new(216.0, top + 45.0, 310.0, 25.0),
                        "",
                        "FusionFallMissionSkin",
                        "label",
                        assets,
                        Some(MissionUiView::JournalNanoSkillDescription(skill)),
                        None,
                    );
                }
            });
        journal.spawn((
            JOURNAL_FM_CARD_RECT.node(),
            styled_image(&assets.journal_reward_box, JOURNAL_REWARD_BOX),
            MissionUiView::JournalRewardCard,
            FocusPolicy::Pass,
            Pickable::IGNORE,
            JournalContentRect::reward(JOURNAL_FM_CARD_RECT),
        ));
        journal.spawn((
            JOURNAL_FM_ICON_RECT.node(),
            stretch(&assets.journal_item_slot),
            MissionUiView::JournalRewardCard,
            FocusPolicy::Pass,
            Pickable::IGNORE,
            JournalContentRect::reward(JOURNAL_FM_ICON_RECT),
        ));
        journal.spawn((
            Button,
            JOURNAL_FM_ICON_RECT.node(),
            stretch(&assets.journal_fm_icon),
            MissionUiControl::JournalRewardIcon,
            MissionUiView::JournalRewardCard,
            JournalContentRect::reward(JOURNAL_FM_ICON_RECT),
        ));
        skin_label(
            journal,
            JOURNAL_FM_AMOUNT_RECT,
            "",
            "FusionFallMissionSkin",
            "bigfont14",
            assets,
            Some(MissionUiView::JournalRewardAmount),
            None,
        );
        skin_label(
            journal,
            JOURNAL_FM_CAPTION_RECT,
            JOURNAL_FUSION_MATTER_LABEL,
            "FusionFallMissionSkin",
            "textArea",
            assets,
            Some(MissionUiView::JournalRewardCard),
            Some(Color::WHITE),
        );
        journal.spawn((
            JOURNAL_TAROS_CARD_RECT.node(),
            styled_image(&assets.journal_reward_box, JOURNAL_REWARD_BOX),
            MissionUiView::JournalCashCard,
            FocusPolicy::Pass,
            Pickable::IGNORE,
            JournalContentRect::cash(JOURNAL_TAROS_CARD_RECT),
        ));
        journal.spawn((
            JOURNAL_TAROS_ICON_RECT.node(),
            stretch(&assets.journal_item_slot),
            MissionUiView::JournalCashCard,
            FocusPolicy::Pass,
            Pickable::IGNORE,
            JournalContentRect::cash(JOURNAL_TAROS_ICON_RECT),
        ));
        journal.spawn((
            Button,
            JOURNAL_TAROS_ICON_RECT.node(),
            stretch(&assets.journal_taros_icon),
            MissionUiControl::JournalRewardIcon,
            MissionUiView::JournalCashCard,
            JournalContentRect::cash(JOURNAL_TAROS_ICON_RECT),
        ));
        skin_label(
            journal,
            JOURNAL_TAROS_AMOUNT_RECT,
            "",
            "FusionFallMissionSkin",
            "bigfont14",
            assets,
            Some(MissionUiView::JournalCashAmount),
            None,
        );
        skin_label(
            journal,
            JOURNAL_TAROS_CAPTION_RECT,
            JOURNAL_TAROS_LABEL,
            "FusionFallMissionSkin",
            "textArea",
            assets,
            Some(MissionUiView::JournalCashCard),
            Some(Color::WHITE),
        );
        journal
            .spawn((
                Button,
                JOURNAL_ACCEPT_RECT.node(),
                styled_image(&assets.accept_button, ACCEPT_BUTTON),
                MissionUiControl::JournalPrimary,
                MissionUiView::JournalPrimary,
                JournalPrimaryButton,
            ))
            .with_children(|button| {
                skin_label(
                    button,
                    MissionUiRect::new(
                        0.0,
                        0.0,
                        JOURNAL_ACCEPT_RECT.width,
                        JOURNAL_ACCEPT_RECT.height,
                    ),
                    "",
                    "FusionFallMissionSkin",
                    "acceptbut",
                    assets,
                    Some(MissionUiView::JournalPrimaryText),
                    Some(Color::WHITE),
                );
            });
        journal_aux_text_button(
            journal,
            JOURNAL_ALLOW_SECONDARY_RECT,
            "DECLINE",
            "RedButton",
            JournalAuxButtonKind::Red,
            MissionUiView::JournalAllowSecondary,
            MissionUiControl::JournalSecondary,
            assets,
        );
        journal_aux_text_button(
            journal,
            JOURNAL_ALLOW_SECONDARY_RECT,
            "CANCEL",
            "CancelButton",
            JournalAuxButtonKind::Cancel,
            MissionUiView::JournalRewardSecondary,
            MissionUiControl::JournalSecondary,
            assets,
        );
        journal_aux_text_button(
            journal,
            JOURNAL_ACTIVE_DELETE_RECT,
            "DELETE MISSION",
            "RedButton",
            JournalAuxButtonKind::Red,
            MissionUiView::JournalActiveDelete,
            MissionUiControl::JournalSecondary,
            assets,
        );
        journal_aux_text_button(
            journal,
            JOURNAL_ACTIVE_MAKE_CURRENT_RECT,
            "MAKE CURRENT MISSION",
            "button",
            JournalAuxButtonKind::Blue,
            MissionUiView::JournalActiveMakeCurrent,
            MissionUiControl::JournalMakeCurrent,
            assets,
        );
        journal.spawn((
            Button,
            JOURNAL_HELP_RECT.node(),
            ImageNode::new(assets.journal_help.clone()),
            MissionUiControl::JournalHelp,
            JournalAuxButtonKind::Help,
        ));
        journal.spawn((
            Button,
            JOURNAL_CLOSE_RECT.node(),
            stretch(&assets.journal_close),
            MissionUiControl::JournalClose,
            JournalCloseButton,
        ));
    });
}

/// Category headers, empty slots and mission rows that `DoWindowRight` draws
/// inside its scroll view, in journal-space coordinates.
pub(super) fn spawn_journal_scroll_list(journal: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    for section in 0..3 {
        journal.spawn((
            Button,
            MissionUiRect::new(613.0, 56.0 + section as f32 * 62.0, 22.0, 22.0).node(),
            ImageNode::new(assets.journal_category_collapse.clone()),
            MissionUiControl::JournalCategoryToggle(section),
            MissionUiView::JournalCategoryToggle(section),
            JournalCategoryToggleButton(section),
        ));
        journal.spawn((
            MissionUiRect::new(613.0, 56.0 + section as f32 * 62.0, 16.0, 16.0).node(),
            ImageNode::new(assets.journal_category_icons[section].clone()),
            MissionUiView::JournalCategoryIcon(section),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        skin_label(
            journal,
            MissionUiRect::new(632.0, 53.0 + section as f32 * 62.0, 250.0, 24.0),
            JOURNAL_CATEGORY_LABELS[section],
            "FusionFallMissionSkinR",
            "bigfont16",
            assets,
            Some(MissionUiView::JournalCategoryHeader(section)),
            None,
        );
        skin_label(
            journal,
            MissionUiRect::new(898.0, 53.0 + section as f32 * 62.0, 60.0, 20.0),
            "",
            "FusionFallMissionSkinR",
            "bigfont16",
            assets,
            Some(MissionUiView::JournalCategoryCount(section)),
            None,
        );
    }
    for slot in 0..6 {
        let mut empty_image = styled_image(&assets.mission_back, MISSION_BACK);
        empty_image.color = Color::srgba(1.0, 1.0, 1.0, 0.5);
        journal.spawn((
            MissionUiRect::new(614.0, 76.0 + slot as f32 * 42.0, 330.0, 27.0).node(),
            empty_image,
            MissionUiView::JournalEmptyRow(slot),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
    }
    for row in 0..JOURNAL_MISSION_ROW_LIMIT {
        journal
            .spawn((
                Button,
                MissionUiRect::new(614.0, 210.0 + row as f32 * 71.0, 330.0, 86.0).node(),
                styled_image(&assets.mission_back, MISSION_BACK),
                MissionUiControl::JournalMissionRow(row),
                MissionUiView::JournalMissionRow(row),
                JournalMissionButton,
            ))
            .with_children(|entry| {
                entry.spawn((
                    MissionUiRect::new(-3.0, -3.0, 336.0, 92.0).node(),
                    styled_image(
                        &assets.journal_selected_mission_back,
                        JOURNAL_SELECTED_MISSION_BACK,
                    ),
                    MissionUiView::JournalMissionRowSelection(row),
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                ));
                entry.spawn((
                    MissionUiRect::new(-3.0, -3.0, 336.0, 62.0).node(),
                    styled_image(
                        &assets.journal_tracked_mission_back,
                        JOURNAL_TRACKED_MISSION_BACK,
                    ),
                    MissionUiView::JournalMissionRowTrackedField(row),
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                ));
                entry
                    .spawn((
                        MissionUiRect::new(-1.0, -10.0, 126.0, 19.0).node(),
                        ImageNode::new(assets.current_mission.clone()),
                        MissionUiView::JournalMissionRowCurrent(row),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ))
                    .with_children(|current| {
                        // The clean texture bakes `CURRENT MISSION` into
                        // the cyan pennant. Preserve its silhouette while
                        // masking only the glyph area and painting keyed
                        // text for every locale.
                        current
                            .spawn((
                                MissionUiRect::new(6.0, 0.0, 106.0, 10.0).node(),
                                BackgroundColor(Color::srgb(203.0 / 255.0, 1.0, 1.0)),
                                FocusPolicy::Pass,
                                Pickable::IGNORE,
                            ))
                            .with_children(|label| {
                                skin_label_with_font_size(
                                    label,
                                    MissionUiRect::new(0.0, -1.0, 106.0, 12.0),
                                    "CURRENT MISSION",
                                    "FusionFallMissionSkin",
                                    "bigfont8",
                                    assets,
                                    None,
                                    Some(Color::srgb(8.0 / 255.0, 60.0 / 255.0, 107.0 / 255.0)),
                                    Some(7.0),
                                );
                            });
                    });
                entry.spawn((
                    MissionUiRect::new(-29.0, 30.0, 30.0, 22.0).node(),
                    ImageNode::new(assets.journal_current_point.clone()),
                    MissionUiView::JournalMissionRowPointer(row),
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                ));
                entry.spawn((
                    MissionUiRect::new(4.0, 10.0, 68.0, 68.0).node(),
                    styled_image(&assets.journal_npc_icon_back, JOURNAL_NPC_ICON_BACK),
                    MissionUiView::JournalMissionRowPortraitFrame(row),
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                ));
                entry.spawn((
                    MissionUiRect::new(7.0, 13.0, 62.0, 62.0).node(),
                    ImageNode::new(assets.journal_npc_portraits[0].clone()),
                    MissionUiView::JournalMissionRowPortrait(row),
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                ));
                entry.spawn((
                    MissionUiRect::new(305.0, 32.0, 22.0, 22.0).node(),
                    ImageNode::new(assets.journal_unchecked.clone()),
                    MissionUiView::JournalMissionRowCheck(row),
                    Button,
                    MissionUiControl::JournalTrackMission(row),
                    FocusPolicy::Block,
                ));
                skin_label(
                    entry,
                    MissionUiRect::new(79.0, 5.0, 200.0, 35.0),
                    "",
                    "FusionFallMissionSkinR",
                    "box",
                    assets,
                    Some(MissionUiView::JournalMissionRowSelectedTitle(row)),
                    None,
                );
                skin_label(
                    entry,
                    MissionUiRect::new(79.0, 35.0, 200.0, 50.0),
                    "",
                    "FusionFallMissionSkinR",
                    "label",
                    assets,
                    Some(MissionUiView::JournalMissionRowSelectedObjective(row)),
                    None,
                );
                skin_label(
                    entry,
                    MissionUiRect::new(9.0, 5.0, 280.0, 25.0),
                    "",
                    "FusionFallMissionSkinR",
                    "bigfont14",
                    assets,
                    Some(MissionUiView::JournalMissionRowUnselectedTitle(row)),
                    None,
                );
                skin_label(
                    entry,
                    MissionUiRect::new(9.0, 25.0, 280.0, 30.0),
                    "",
                    "FusionFallMissionSkinR",
                    "label",
                    assets,
                    Some(MissionUiView::JournalMissionRowUnselectedObjective(row)),
                    None,
                );
            });
    }
}
