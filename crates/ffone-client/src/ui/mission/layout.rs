//! Row measurement and computed layout of the NPC letterbox and mission journal.

use super::components::MissionUiView;
use super::geometry::{JOURNAL_ACCEPT_RECT, JOURNAL_COMPLETE_RECT, JOURNAL_SCROLL_VIEW_RECT};
use super::journal::{current_mission, journal_is_completed, journal_rows};
use super::labels::mission_ui_localized_text;
use super::model::{MissionJournalUi, MissionUiEntry, MissionUiModel};
use crate::{gameplay_ui::NpcServiceKind, localization::LocalizedText};
use bevy::{prelude::*, text::TextLayoutInfo};

pub(super) const NPC_MISSION_ROW_LIMIT: usize = 4;
pub(super) const NPC_UTILITY_ROW_LIMIT: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MissionUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl MissionUiRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn center(self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub const fn translated(self, x: f32, y: f32) -> Self {
        Self::new(self.x + x, self.y + y, self.width, self.height)
    }

    pub fn scale_about(self, pivot: Vec2, scale: f32) -> Self {
        let center = pivot + (self.center() - pivot) * scale;
        Self::new(
            center.x - self.width * scale * 0.5,
            center.y - self.height * scale * 0.5,
            self.width * scale,
            self.height * scale,
        )
    }

    pub(super) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.x),
            top: px(self.y),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

pub(super) fn apply_scaled_group(
    node: &mut Node,
    transform: &mut UiTransform,
    visual: MissionUiRect,
    unscaled_size: Vec2,
    scale: f32,
) {
    node.left = px(visual.x + (visual.width - unscaled_size.x) * 0.5);
    node.top = px(visual.y + (visual.height - unscaled_size.y) * 0.5);
    node.width = px(unscaled_size.x);
    node.height = px(unscaled_size.y);
    transform.scale = Vec2::splat(scale);
}

pub(super) fn journal_primary_rect(journal: &MissionJournalUi) -> MissionUiRect {
    if matches!(journal, MissionJournalUi::Reward { .. }) {
        JOURNAL_COMPLETE_RECT
    } else {
        JOURNAL_ACCEPT_RECT
    }
}

pub(super) fn journal_primary_text_rect(journal: &MissionJournalUi) -> MissionUiRect {
    let rect = journal_primary_rect(journal);
    MissionUiRect::new(0.0, 0.0, rect.width, rect.height)
}

pub(super) fn journal_primary_is_visible(model: &MissionUiModel) -> bool {
    matches!(
        model.journal,
        MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. }
    )
}

/// Row entities recycled for the rows intersecting the scroll view: its 558 px
/// meet at most nine 56 px rows on their 71 px stride.
pub(super) const JOURNAL_MISSION_ROW_LIMIT: usize = 10;

#[derive(Resource, Clone)]
pub(super) struct NpcMissionRowHeights(pub(super) [f32; NPC_MISSION_ROW_LIMIT]);

impl Default for NpcMissionRowHeights {
    fn default() -> Self { Self([32.0; NPC_MISSION_ROW_LIMIT]) }
}

pub(super) fn measure_npc_mission_rows(
    texts: Query<(&MissionUiView, &TextLayoutInfo, &ComputedNode)>,
    mut heights: ResMut<NpcMissionRowHeights>,
) {
    for (view, layout, node) in &texts {
        if let MissionUiView::NpcMissionRowText(slot) = *view {
            let height = (layout.size.y * node.inverse_scale_factor() + 14.0).ceil().max(32.0);
            if heights.0[slot] != height { heights.0[slot] = height; }
        }
    }
}

pub(super) fn npc_rows<'a>(model: &'a MissionUiModel, heights: &NpcMissionRowHeights) -> Vec<(&'a MissionUiEntry, f32)> {
    let Some(interaction) = model.npc_interaction.as_ref() else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    let mut y = 50.0;
    if !interaction.completed_missions.is_empty() {
        y += 25.0;
        for mission in &interaction.completed_missions {
            if rows.len() == NPC_MISSION_ROW_LIMIT {
                return rows;
            }
            rows.push((mission, y));
            y += heights.0[rows.len() - 1];
        }
    }
    if !interaction.available_missions.is_empty() {
        y += 25.0;
        for mission in &interaction.available_missions {
            if rows.len() == NPC_MISSION_ROW_LIMIT {
                return rows;
            }
            rows.push((mission, y));
            y += heights.0[rows.len() - 1];
        }
    }
    rows
}

#[derive(Clone, Copy)]
pub(super) struct NpcUtilityRow<'a> {
    pub(super) label: &'a str,
    pub(super) warp_npc_type: Option<i32>,
    pub(super) service: Option<NpcServiceKind>,
}

impl NpcUtilityRow<'_> {
    pub(super) fn localized_text(&self) -> LocalizedText {
        match self.warp_npc_type {
            Some(2694..=2696) => LocalizedText::new("ui.mission.npc.warp", "WARP"),
            _ => mission_ui_localized_text(self.label),
        }
    }
}

pub(super) fn npc_utility_rows(model: &MissionUiModel) -> Vec<NpcUtilityRow<'_>> {
    let Some(interaction) = model.npc_interaction.as_ref() else {
        return Vec::new();
    };
    interaction
        .services
        .iter()
        .map(|entry| NpcUtilityRow {
            label: &entry.label,
            warp_npc_type: None,
            service: Some(entry.service),
        })
        .chain(interaction.warp.iter().map(|warp| NpcUtilityRow {
            label: &warp.label,
            warp_npc_type: Some(warp.npc_type),
            service: None,
        }))
        .take(NPC_UTILITY_ROW_LIMIT)
        .collect()
}

pub(super) fn npc_utility_panel_height(model: &MissionUiModel) -> f32 {
    match npc_utility_rows(model).len() + 1 {
        0 | 1 => 114.0,
        2 => 171.0,
        _ => 228.0,
    }
}

pub(super) fn quest_list_end(model: &MissionUiModel, heights: &NpcMissionRowHeights) -> f32 {
    let Some(interaction) = model.npc_interaction.as_ref() else {
        return 50.0;
    };
    let completed = interaction
        .completed_missions
        .len()
        .min(NPC_MISSION_ROW_LIMIT);
    let available = interaction
        .available_missions
        .len()
        .min(NPC_MISSION_ROW_LIMIT.saturating_sub(completed));
    50.0 + if completed > 0 {
        25.0 + heights.0[..completed].iter().sum::<f32>()
    } else {
        0.0
    } + if available > 0 {
        25.0 + heights.0[completed..completed + available].iter().sum::<f32>()
    } else {
        0.0
    }
}

pub(super) fn quest_bottom_height(model: &MissionUiModel) -> f32 {
    if !npc_utility_rows(model).is_empty() {
        131.0
    } else {
        76.0
    }
}

// `cnMissionJournal.SortMission`: 1 Guide, 2 Nano, 3 Normal/world.
pub(super) const JOURNAL_CATEGORY_TYPES: [i32; 3] = [2, 1, 3];
pub(super) const JOURNAL_CATEGORY_CAPACITIES: [usize; 3] = [1, 1, 4];
pub(super) const JOURNAL_CATEGORY_LABELS: [&str; 3] = ["NANO MISSION", "GUIDE MISSION", "WORLD MISSIONS"];

pub(super) struct JournalRowLayout<'a> {
    pub(super) mission: &'a MissionUiEntry,
    pub(super) rect: MissionUiRect,
    pub(super) selected: bool,
    pub(super) tracked: bool,
}

pub(super) struct JournalRightLayout<'a> {
    pub(super) completed: bool,
    /// Height of the `DoWindowRight` scroll-view content.
    pub(super) content_height: f32,
    pub(super) header_tops: [f32; 3],
    pub(super) category_counts: [usize; 3],
    pub(super) rows: Vec<JournalRowLayout<'a>>,
    pub(super) empty_rows: Vec<MissionUiRect>,
}

pub(super) fn journal_right_layout(model: &MissionUiModel) -> JournalRightLayout<'_> {
    let source_rows = journal_rows(model);
    let completed = journal_is_completed(model);
    let selected_task = matches!(model.journal, MissionJournalUi::Other(_))
        .then(|| current_mission(model).map(|mission| mission.task_id))
        .flatten();
    let tracked_task = (!completed)
        .then_some(model.selected_journal_task_id)
        .flatten();
    let mut header_tops = [0.0; 3];
    let mut category_counts = [0; 3];
    let mut rows = Vec::new();
    let mut empty_rows = Vec::new();
    // `RightBox` (597,11) + scroll view (11,40).
    let scroll_x = JOURNAL_SCROLL_VIEW_RECT.x;
    let scroll_y = JOURNAL_SCROLL_VIEW_RECT.y;
    let visible_top = scroll_y + model.journal_scroll;
    let visible_bottom = visible_top + JOURNAL_SCROLL_VIEW_RECT.height;
    let mut y = 5.0;
    for (section, mission_type) in JOURNAL_CATEGORY_TYPES.into_iter().enumerate() {
        header_tops[section] = scroll_y + y;
        y += if completed { 30.0 } else { 20.0 };
        let mut category = source_rows
            .iter()
            .filter(|mission| mission.mission_type == mission_type)
            .collect::<Vec<_>>();
        if completed {
            category.sort_by(|left, right| {
                (left.required_level, left.title.as_str(), left.task_id).cmp(&(
                    right.required_level,
                    right.title.as_str(),
                    right.task_id,
                ))
            });
        } else {
            category.truncate(JOURNAL_CATEGORY_CAPACITIES[section]);
        }
        category_counts[section] = category.len();
        if !completed || model.completed_category_expanded[section] {
            for mission in category {
                let selected = selected_task == Some(mission.task_id);
                let tracked = tracked_task == Some(mission.task_id);
                if tracked {
                    // `RenderActive` draws CurrentMissionTex for SelectMission,
                    // independently of which row is expanded, then advances 10 px.
                    y += 10.0;
                }
                let rect = MissionUiRect::new(
                    scroll_x + 6.0,
                    scroll_y + y,
                    330.0,
                    if selected { 86.0 } else { 56.0 },
                );
                // Row entities are recycled for the rows the view can show.
                if rect.y < visible_bottom && rect.y + rect.height > visible_top {
                    rows.push(JournalRowLayout {
                        mission,
                        rect,
                        selected,
                        tracked,
                    });
                }
                y += if selected { 101.0 } else { 71.0 };
            }
        }
        if completed {
            if section < 2 {
                y += 10.0;
            }
        } else {
            for _ in category_counts[section]..JOURNAL_CATEGORY_CAPACITIES[section] {
                empty_rows.push(MissionUiRect::new(
                    scroll_x + 6.0,
                    scroll_y + y,
                    330.0,
                    27.0,
                ));
                y += 42.0;
            }
        }
    }
    JournalRightLayout {
        completed,
        // The source sizes its view 110 (active) or 140 (completed) px plus
        // the rows, i.e. 45/25 px past this running y, which also covers the
        // expanded row and tracked banner that the source left out.
        content_height: y + if completed { 25.0 } else { 45.0 },
        header_tops,
        category_counts,
        rows,
        empty_rows,
    }
}

pub(super) fn journal_row_has_selected_decorations(layout: Option<&JournalRowLayout<'_>>) -> bool {
    layout.is_some_and(|layout| layout.selected)
}
