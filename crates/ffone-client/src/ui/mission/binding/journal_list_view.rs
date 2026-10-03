//! Binding of journal list panels, tabs, category headers and mission rows.

use super::super::components::MissionUiView;
use super::super::journal::current_mission;
use super::super::labels::{mission_content_text, mission_ui_localized_text};
use super::super::layout::{
    JOURNAL_CATEGORY_CAPACITIES, MissionUiRect, journal_row_has_selected_decorations,
};
use super::super::model::MissionJournalUi;
use super::super::widgets::{mission_text_font, mission_text_layout_rect};
use super::MissionUiBindContext;
use crate::{gui_skin::gui_style as mission_gui_style, localization::LocalizedText};
use bevy::prelude::*;

/// Journal list panels, tabs, category headers and mission rows.
pub(super) fn bind_journal_list_view(
    context: &MissionUiBindContext<'_>,
    view: &MissionUiView,
    node: Option<Mut<Node>>,
    localized_text: Option<Mut<LocalizedText>>,
    image: Option<Mut<ImageNode>>,
) {
    let MissionUiBindContext {
        model,
        assets,
        content,
        asset_server,
        journal_layout,
        completed_journal,
        active_journal,
        journal_visible,
        ..
    } = *context;
    let mut node = node;
    let mut localized_text = localized_text;
    let mut image = image;
    macro_rules! set_text {
        ($localized:expr) => {
            if let Some(component) = &mut localized_text {
                **component = $localized;
            }
        };
    }
    match *view {
        MissionUiView::JournalAllowListPanel | MissionUiView::JournalAllowListTitle => {
            if let Some(node) = &mut node {
                node.display = if journal_visible
                    && matches!(
                        model.journal,
                        MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. }
                    ) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalActiveListPanel | MissionUiView::JournalActiveListTitle => {
            if let Some(node) = &mut node {
                node.display = if journal_visible && active_journal {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalCompletedListPanel | MissionUiView::JournalCompletedListTitle => {
            if let Some(node) = &mut node {
                node.display = if journal_visible && completed_journal {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalActiveTab => {
            if let Some(node) = &mut node {
                node.display = if journal_visible && completed_journal {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalCompletedTab => {
            if let Some(node) = &mut node {
                node.display = if journal_visible && active_journal {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalRightFrame => {
            if let Some(node) = &mut node {
                node.display = if journal_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalCategoryHeader(section) => {
            if let Some(node) = &mut node {
                node.display = if journal_visible {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some(top) = journal_layout.header_tops.get(section) {
                    node.top = px(*top);
                }
                node.left = px(if completed_journal { 657.0 } else { 632.0 });
                node.width = px(if completed_journal { 225.0 } else { 250.0 });
            }
        }
        MissionUiView::JournalCategoryCount(section) => {
            if let Some(node) = &mut node {
                node.display = if journal_visible {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some(top) = journal_layout.header_tops.get(section) {
                    node.top = px(*top);
                }
            }
            set_text!(journal_layout.category_counts.get(section).map_or_else(
                || mission_ui_localized_text(""),
                |count| {
                    mission_ui_localized_text(if completed_journal {
                        count.to_string()
                    } else {
                        format!("{count}/{}", JOURNAL_CATEGORY_CAPACITIES[section])
                    })
                },
            ));
        }
        MissionUiView::JournalCategoryIcon(section) => {
            if let Some(node) = &mut node {
                node.display = if journal_visible {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some(top) = journal_layout.header_tops.get(section) {
                    node.top = px(*top + if completed_journal { 3.0 } else { 0.0 });
                }
                node.left = px(if completed_journal { 638.0 } else { 613.0 });
            }
        }
        MissionUiView::JournalCategoryToggle(section) => {
            if let Some(node) = &mut node {
                node.display = if journal_visible && completed_journal {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some(top) = journal_layout.header_tops.get(section) {
                    node.top = px(*top);
                }
            }
            if let Some(image) = &mut image {
                image.image = if model
                    .completed_category_expanded
                    .get(section)
                    .copied()
                    .unwrap_or(false)
                {
                    assets.journal_category_collapse.clone()
                } else {
                    assets.journal_category_expand.clone()
                };
            }
        }
        MissionUiView::JournalEmptyRow(slot) => {
            if let Some(node) = &mut node {
                if let Some(rect) = journal_layout.empty_rows.get(slot) {
                    node.display = if journal_visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    node.left = px(rect.x);
                    node.top = px(rect.y);
                    node.width = px(rect.width);
                    node.height = px(rect.height);
                } else {
                    node.display = Display::None;
                }
            }
        }
        MissionUiView::JournalMissionRow(row) => {
            if let Some(node) = &mut node {
                if let Some(layout) = journal_layout.rows.get(row) {
                    node.display = if journal_visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    node.left = px(layout.rect.x);
                    node.top = px(layout.rect.y);
                    node.width = px(layout.rect.width);
                    node.height = px(layout.rect.height);
                } else {
                    node.display = Display::None;
                }
            }
        }
        MissionUiView::JournalMissionRowSelectedTitle(row)
        | MissionUiView::JournalMissionRowSelectedObjective(row)
        | MissionUiView::JournalMissionRowUnselectedTitle(row)
        | MissionUiView::JournalMissionRowUnselectedObjective(row) => {
            let layout = journal_layout.rows.get(row);
            if let Some(node) = &mut node {
                let wants_selected = matches!(
                    *view,
                    MissionUiView::JournalMissionRowSelectedTitle(_)
                        | MissionUiView::JournalMissionRowSelectedObjective(_)
                );
                node.display = if layout.is_some_and(|layout| layout.selected == wants_selected)
                {
                    Display::Flex
                } else {
                    Display::None
                };
                let (outer, style_name) = match *view {
                    MissionUiView::JournalMissionRowSelectedTitle(_) => (
                        MissionUiRect::new(
                            79.0,
                            5.0,
                            200.0,
                            if completed_journal { 30.0 } else { 35.0 },
                        ),
                        "box",
                    ),
                    MissionUiView::JournalMissionRowSelectedObjective(_) => {
                        (MissionUiRect::new(79.0, 35.0, 200.0, 50.0), "label")
                    }
                    MissionUiView::JournalMissionRowUnselectedTitle(_) => (
                        MissionUiRect::new(
                            9.0,
                            5.0,
                            280.0,
                            if completed_journal { 30.0 } else { 25.0 },
                        ),
                        "bigfont14",
                    ),
                    MissionUiView::JournalMissionRowUnselectedObjective(_) => {
                        (MissionUiRect::new(9.0, 25.0, 280.0, 30.0), "label")
                    }
                    _ => unreachable!(),
                };
                let style = mission_gui_style("FusionFallMissionSkinR", style_name)
                    .expect("Retrobution journal row text style");
                let font = mission_text_font(&assets, "FusionFallMissionSkinR", style_name);
                let content = mission_text_layout_rect(outer, style, &font);
                node.left = px(content.x);
                node.top = px(content.y);
                node.width = px(content.width);
                node.height = px(content.height);
            }
            set_text!(layout.map_or_else(
                || mission_ui_localized_text(""),
                |layout| {
                    if matches!(
                        *view,
                        MissionUiView::JournalMissionRowSelectedTitle(_)
                            | MissionUiView::JournalMissionRowUnselectedTitle(_)
                    ) {
                        mission_content_text(
                            layout.mission.task_id,
                            "title",
                            &layout.mission.title,
                        )
                    } else if completed_journal {
                        mission_content_text(
                            layout.mission.task_id,
                            "mission_summary",
                            &layout.mission.mission_summary,
                        )
                    } else {
                        mission_content_text(
                            layout.mission.task_id,
                            "objective",
                            &layout.mission.objective,
                        )
                    }
                },
            ));
        }
        MissionUiView::JournalAllowSecondary => {
            if let Some(node) = &mut node {
                node.display = if matches!(model.journal, MissionJournalUi::Allow(_)) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalRewardSecondary => {
            if let Some(node) = &mut node {
                node.display = if matches!(model.journal, MissionJournalUi::Reward { .. }) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalActiveDelete => {
            if let Some(node) = &mut node {
                node.display = if active_journal {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            if let Some(image) = &mut image {
                image.color = if model.pending.is_some() {
                    Color::srgba(0.55, 0.55, 0.55, 0.8)
                } else {
                    Color::WHITE
                };
            }
        }
        MissionUiView::JournalActiveMakeCurrent => {
            if let Some(node) = &mut node {
                node.display = if active_journal
                    && current_mission(&model).is_some_and(|mission| {
                        model.selected_journal_task_id != Some(mission.task_id)
                    }) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalMissionRowSelection(row)
        | MissionUiView::JournalMissionRowPointer(row)
        | MissionUiView::JournalMissionRowPortraitFrame(row) => {
            if let Some(node) = &mut node {
                node.display =
                    if journal_row_has_selected_decorations(journal_layout.rows.get(row)) {
                        Display::Flex
                    } else {
                        Display::None
                    };
            }
        }
        MissionUiView::JournalMissionRowCurrent(row) => {
            if let Some(node) = &mut node {
                node.display = if journal_layout
                    .rows
                    .get(row)
                    .is_some_and(|layout| layout.tracked)
                {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalMissionRowTrackedField(row) => {
            if let Some(node) = &mut node {
                node.display = if journal_layout
                    .rows
                    .get(row)
                    .is_some_and(|layout| layout.tracked && !layout.selected)
                {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalMissionRowPortrait(row) => {
            let layout = journal_layout.rows.get(row);
            if let Some(node) = &mut node {
                node.display = if journal_row_has_selected_decorations(layout) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            if let Some(image) = &mut image {
                image.image = layout.map_or_else(
                    || assets.journal_npc_portraits[0].clone(),
                    |layout| {
                        assets.journal_npc_portrait(
                            &content,
                            &asset_server,
                            layout.mission.journal_npc_type,
                        )
                    },
                );
            }
        }
        MissionUiView::JournalMissionRowCheck(row) => {
            if let Some(node) = &mut node {
                if !completed_journal && let Some(layout) = journal_layout.rows.get(row) {
                    node.display = Display::Flex;
                    node.top = px(if layout.selected { 32.0 } else { 20.0 });
                } else {
                    node.display = Display::None;
                }
            }
            if let Some(image) = &mut image {
                image.image = if journal_layout
                    .rows
                    .get(row)
                    .is_some_and(|layout| layout.tracked)
                {
                    assets.journal_current_check.clone()
                } else {
                    assets.journal_unchecked.clone()
                };
            }
        }
        _ => {}
    }
}
