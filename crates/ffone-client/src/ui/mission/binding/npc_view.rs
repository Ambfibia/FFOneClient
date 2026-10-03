//! Binding of NPC letterbox, quest list and utility panel views.

use super::super::assets::{MISSION_BOTTOM, MISSION_BOTTOM_MULTI, NPC_MULTI_WINDOW, NPC_WINDOW};
use super::super::components::MissionUiView;
use super::super::labels::{mission_content_text, mission_ui_localized_text};
use super::super::layout::{NPC_MISSION_ROW_LIMIT, npc_utility_panel_height, quest_bottom_height};
use super::super::widgets::styled_image;
use super::MissionUiBindContext;
use crate::localization::{LocalizedText, localized_tabledata_npc_name};
use bevy::prelude::*;

/// NPC letterbox, quest list and utility panel views.
pub(super) fn bind_npc_view(
    context: &MissionUiBindContext<'_>,
    view: &MissionUiView,
    node: Option<Mut<Node>>,
    localized_text: Option<Mut<LocalizedText>>,
    image: Option<Mut<ImageNode>>,
) {
    let MissionUiBindContext {
        model,
        heights,
        assets,
        rows,
        utility_rows,
        list_end,
        interaction,
        quest_visible,
        npc_utility_visible,
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
        MissionUiView::NpcLetterboxRoot => {
            if let Some(node) = &mut node {
                node.display = if model.enabled && model.npc_letterbox_visible() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::NpcTopNotice => {
            if let Some(node) = &mut node {
                node.display = if model.npc_top_notice.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::NpcTopNoticeText => {
            if let Some(notice) = &model.npc_top_notice {
                set_text!(notice.clone());
            }
        }
        MissionUiView::NpcQuestRoot => {
            if let Some(node) = &mut node {
                node.display = if quest_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::NpcWarpRoot => {
            if let Some(node) = &mut node {
                node.display = if npc_utility_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            if let Some(image) = &mut image {
                *image.as_mut() = if !utility_rows.is_empty() {
                    styled_image(&assets.npc_multi_window, NPC_MULTI_WINDOW)
                } else {
                    styled_image(&assets.npc_window, NPC_WINDOW)
                };
            }
        }
        MissionUiView::NpcName => {
            set_text!(interaction.map_or_else(
                || mission_ui_localized_text(""),
                |npc| {
                    if npc.npc_type > 0 {
                        localized_tabledata_npc_name(npc.npc_type, &npc.name)
                    } else {
                        mission_ui_localized_text(npc.name.clone())
                    }
                },
            ));
        }
        MissionUiView::NpcCompletedBody => {
            if let Some(node) = &mut node {
                let completed = interaction.map_or(0, |npc| {
                    npc.completed_missions.len().min(NPC_MISSION_ROW_LIMIT)
                });
                node.display = if quest_visible && completed > 0 {
                    Display::Flex
                } else {
                    Display::None
                };
                node.top = px(50.0);
                node.height = px(25.0 + heights.0[..completed].iter().sum::<f32>());
            }
        }
        MissionUiView::NpcAvailableBody => {
            if let Some(node) = &mut node {
                let completed = interaction.map_or(0, |npc| {
                    npc.completed_missions.len().min(NPC_MISSION_ROW_LIMIT)
                });
                let available = interaction.map_or(0, |npc| {
                    npc.available_missions
                        .len()
                        .min(NPC_MISSION_ROW_LIMIT.saturating_sub(completed))
                });
                let top = 50.0
                    + if completed > 0 {
                        25.0 + heights.0[..completed].iter().sum::<f32>()
                    } else {
                        0.0
                    };
                node.display = if quest_visible && available > 0 {
                    Display::Flex
                } else {
                    Display::None
                };
                node.top = px(top);
                node.height = px(25.0 + heights.0[completed..completed + available].iter().sum::<f32>());
            }
        }
        MissionUiView::NpcQuestUtilityButton => {
            if let Some(node) = &mut node {
                node.display = if quest_visible && !utility_rows.is_empty() {
                    Display::Flex
                } else {
                    Display::None
                };
                node.top = px(list_end + 13.0);
            }
        }
        MissionUiView::NpcQuestUtilityText => {
            set_text!(
                utility_rows
                    .first()
                    .map_or_else(|| mission_ui_localized_text(""), |row| row.localized_text(),)
            );
        }
        MissionUiView::NpcQuestUtilityIcon => {
            if let Some(image) = &mut image {
                image.image = utility_rows.first().map_or_else(
                    || assets.npcicon_warp.clone(),
                    |row| {
                        row.service.map_or_else(
                            || assets.npcicon_warp.clone(),
                            |service| assets.npc_service_icon(service).clone(),
                        )
                    },
                );
            }
        }
        MissionUiView::NpcQuestCloseButton => {
            if let Some(node) = &mut node {
                node.top = px(list_end + if !utility_rows.is_empty() { 67.0 } else { 12.0 });
            }
        }
        MissionUiView::NpcUtilityButton(slot) => {
            if let Some(node) = &mut node {
                node.display = if npc_utility_visible && utility_rows.get(slot).is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                let total = utility_rows.len() + 1;
                node.top = px(npc_utility_panel_height(&model) - total as f32 * 54.0
                    + slot as f32 * 54.0
                    - 10.0);
            }
        }
        MissionUiView::NpcUtilityText(slot) => {
            set_text!(
                utility_rows
                    .get(slot)
                    .map_or_else(|| mission_ui_localized_text(""), |row| row.localized_text(),)
            );
        }
        MissionUiView::NpcUtilityIcon(slot) => {
            if let Some(image) = &mut image {
                image.image = utility_rows.get(slot).map_or_else(
                    || assets.npcicon_warp.clone(),
                    |row| {
                        row.service.map_or_else(
                            || assets.npcicon_warp.clone(),
                            |service| assets.npc_service_icon(service).clone(),
                        )
                    },
                );
            }
        }
        MissionUiView::NpcCloseButton => {
            if let Some(node) = &mut node {
                let total = utility_rows.len() + 1;
                node.top = px(npc_utility_panel_height(&model) - total as f32 * 54.0
                    + utility_rows.len() as f32 * 54.0
                    - 10.0);
            }
        }
        MissionUiView::NpcCompletedHeader => {
            if let Some(node) = &mut node {
                node.display = if quest_visible
                    && interaction.is_some_and(|npc| !npc.completed_missions.is_empty())
                {
                    Display::Flex
                } else {
                    Display::None
                };
                node.top = px(50.0);
            }
        }
        MissionUiView::NpcAvailableHeader => {
            if let Some(node) = &mut node {
                let completed_offset = interaction.map_or(0.0, |npc| {
                    if npc.completed_missions.is_empty() {
                        0.0
                    } else {
                        25.0 + heights.0[..npc.completed_missions.len().min(NPC_MISSION_ROW_LIMIT)]
                            .iter().sum::<f32>()
                    }
                });
                node.display = if quest_visible
                    && interaction.is_some_and(|npc| !npc.available_missions.is_empty())
                {
                    Display::Flex
                } else {
                    Display::None
                };
                node.top = px(50.0 + completed_offset);
            }
        }
        MissionUiView::NpcMissionRow(slot) => {
            if let Some(node) = &mut node {
                node.display = if quest_visible && rows.get(slot).is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some((_, y)) = rows.get(slot) {
                    node.top = px(*y);
                    node.height = px(heights.0[slot] - 5.0);
                }
            }
        }
        MissionUiView::NpcMissionRowIcon(slot) => {
            if let Some(image) = &mut image {
                image.image = rows.get(slot).map_or_else(
                    || assets.npc_world_mission_icon.clone(),
                    // Tutorial world and nano missions both use
                    // pMissionIcon[0] (`world_icon`). Guide-specific
                    // icons are selected only when m_iHMissionType=1.
                    |_| assets.npc_world_mission_icon.clone(),
                );
            }
        }
        MissionUiView::NpcMissionRowText(slot) => {
            set_text!(rows.get(slot).map_or_else(
                || mission_ui_localized_text(""),
                |(mission, _)| mission_content_text(mission.task_id, "title", &mission.title,),
            ));
        }
        MissionUiView::NpcQuestBottom => {
            if let Some(node) = &mut node {
                node.top = px(list_end);
                node.height = px(quest_bottom_height(&model));
            }
            if let Some(image) = &mut image {
                *image.as_mut() = if !utility_rows.is_empty() {
                    styled_image(&assets.mission_bottom_multi, MISSION_BOTTOM_MULTI)
                } else {
                    styled_image(&assets.mission_bottom, MISSION_BOTTOM)
                };
            }
        }
        _ => {}
    }
}
