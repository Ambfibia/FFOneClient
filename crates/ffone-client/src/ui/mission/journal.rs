//! Mission journal rows, scrolling, content layout and hover presentation.

use super::assets::MissionUiAssets;
use super::components::{
    JournalAuxButtonKind, JournalCategoryToggleButton, JournalCloseButton, JournalContentRect,
    JournalPrimaryButton, JournalTabBackground, JournalTabButton, MissionUiControl, MissionUiView,
};
use super::geometry::{JOURNAL_SCROLL_VIEW_RECT, JOURNAL_SCROLL_WHEEL_LINE};
use super::layout::journal_right_layout;
use super::model::{JournalListTab, MissionJournalUi, MissionUiEntry, MissionUiModel};
use super::widgets::mission_text_state_color;
use crate::{
    gameplay_nano_portraits::{
        GameplayNanoPortraitCatalog, JournalNanoPortraitImage, JournalNanoPortraitRequest,
    },
    gui_skin::gui_style as mission_gui_style,
};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    ui::RelativeCursorPosition,
};

pub(super) fn current_mission(model: &MissionUiModel) -> Option<&MissionUiEntry> {
    match &model.journal {
        MissionJournalUi::Allow(mission) | MissionJournalUi::Reward { mission, .. } => {
            Some(mission)
        }
        MissionJournalUi::Other(_) => model
            .viewed_journal_task_id
            .and_then(|task_id| {
                journal_rows(model)
                    .iter()
                    .find(|mission| mission.task_id == task_id)
            })
            .or_else(|| journal_rows(model).first()),
        MissionJournalUi::Hidden => None,
    }
}

pub(super) fn sync_journal_nano_portrait_request(
    model: Res<MissionUiModel>,
    catalog: Option<Res<GameplayNanoPortraitCatalog>>,
    mut request: ResMut<JournalNanoPortraitRequest>,
) {
    let desired = (model.enabled && !matches!(model.journal, MissionJournalUi::Hidden))
        .then(|| {
            let nano_id = i16::try_from(current_mission(&model)?.nano.as_ref()?.nano_id).ok()?;
            let model_path = catalog.as_ref()?.model_path(nano_id)?;
            Some((nano_id, model_path))
        })
        .flatten();
    if request.desired() == desired {
        return;
    }
    if let Some((nano_id, model_path)) = desired {
        request.set(nano_id, model_path);
    } else {
        request.clear();
    }
}

pub(super) fn journal_rows(model: &MissionUiModel) -> &[MissionUiEntry] {
    match &model.journal {
        MissionJournalUi::Other(other) => match model.journal_tab {
            JournalListTab::Active => &other.active_missions,
            JournalListTab::Completed => &other.completed_missions,
        },
        MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. } => {
            &model.nanocom_journal.active_missions
        }
        MissionJournalUi::Hidden => &[],
    }
}

pub(super) fn journal_is_completed(model: &MissionUiModel) -> bool {
    matches!(model.journal, MissionJournalUi::Other(_))
        && model.journal_tab == JournalListTab::Completed
}

/// `DoWindowRight` scroll view hosting [`spawn_journal_scroll_list`].
#[derive(Component)]
pub(super) struct JournalScrollViewport;

/// Journal-space content of [`JournalScrollViewport`]; only its offset moves.
#[derive(Component)]
pub(super) struct JournalScrollContent;

/// The list the journal shows, `(is_journal_book, tab)`. Another list starts
/// scrolled to the top.
pub(super) fn journal_scroll_list(model: &MissionUiModel) -> Option<(bool, JournalListTab)> {
    if !model.enabled {
        return None;
    }
    match model.journal {
        MissionJournalUi::Hidden => None,
        MissionJournalUi::Other(_) => Some((true, model.journal_tab)),
        MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. } => {
            Some((false, JournalListTab::Active))
        }
    }
}

pub(super) fn journal_max_scroll(model: &MissionUiModel) -> f32 {
    (journal_right_layout(model).content_height - JOURNAL_SCROLL_VIEW_RECT.height).max(0.0)
}

/// Mouse-wheel scrolling of the journal list, clamped to its content like the
/// clean `GUI.BeginScrollView`.
pub(super) fn scroll_mission_journal(
    wheel_events: Option<MessageReader<MouseWheel>>,
    mut model: ResMut<MissionUiModel>,
    viewports: Query<&RelativeCursorPosition, With<JournalScrollViewport>>,
    mut contents: Query<&mut Node, With<JournalScrollContent>>,
    mut shown_list: Local<Option<(bool, JournalListTab)>>,
) {
    let list = journal_scroll_list(&model);
    let hovered = list.is_some() && viewports.iter().any(RelativeCursorPosition::cursor_over);
    let mut offset = if *shown_list == list {
        model.journal_scroll
    } else {
        0.0
    };
    *shown_list = list;
    if let Some(mut wheel_events) = wheel_events {
        for event in wheel_events.read() {
            if hovered {
                offset -= match event.unit {
                    MouseScrollUnit::Line => event.y * JOURNAL_SCROLL_WHEEL_LINE,
                    MouseScrollUnit::Pixel => event.y,
                };
            }
        }
    }
    if list.is_none() {
        offset = 0.0;
    } else if offset != model.journal_scroll || model.is_changed() {
        offset = offset.clamp(0.0, journal_max_scroll(&model));
    }
    if model.journal_scroll != offset {
        model.journal_scroll = offset;
    }
    let top = px(-JOURNAL_SCROLL_VIEW_RECT.y - offset);
    for node in &mut contents {
        node.map_unchanged(|node| &mut node.top).set_if_neq(top);
    }
}

pub(super) fn bind_journal_content_layout(
    model: Res<MissionUiModel>,
    mut content_nodes: Query<(&JournalContentRect, &mut Node)>,
) {
    if !model.is_changed() {
        return;
    }
    let fusion_matter = current_mission(&model).map_or(0, |mission| mission.rewards.fusion_matter);
    let has_left_content =
        !matches!(model.journal, MissionJournalUi::Other(_)) || current_mission(&model).is_some();
    for (content, mut node) in &mut content_nodes {
        let mut rect = match (&model.journal, model.journal_tab) {
            (MissionJournalUi::Other(_), JournalListTab::Completed) => content.completed_rect,
            (MissionJournalUi::Other(_), JournalListTab::Active) => content.active_rect,
            _ => content.offer_rect,
        };
        if content.cash_element && fusion_matter <= 0 {
            rect = rect.translated(-260.0, 0.0);
        }
        node.display = if has_left_content {
            Display::Flex
        } else {
            Display::None
        };
        node.left = px(rect.x);
        node.top = px(rect.y);
        node.width = px(rect.width);
        node.height = px(rect.height);
    }
}

pub(super) fn bind_journal_nano_portrait_image(
    portrait: Res<JournalNanoPortraitImage>,
    mut views: Query<(&MissionUiView, &mut ImageNode)>,
) {
    if !portrait.is_changed() {
        return;
    }
    for (view, mut image) in &mut views {
        if *view == MissionUiView::JournalNanoPortrait {
            image.image = portrait.0.clone().unwrap_or_default();
        }
    }
}

pub(super) fn bind_journal_mission_button_hover(
    model: Res<MissionUiModel>,
    assets: Res<MissionUiAssets>,
    mut buttons: Query<(&Interaction, &MissionUiControl, &mut ImageNode), Changed<Interaction>>,
) {
    for (interaction, control, mut image) in &mut buttons {
        match *control {
            MissionUiControl::JournalMissionRow(_) => image.color = Color::WHITE,
            MissionUiControl::JournalTrackMission(row) => {
                let tracked = journal_right_layout(&model)
                    .rows
                    .get(row)
                    .is_some_and(|layout| layout.tracked);
                let hovered = matches!(interaction, Interaction::Hovered | Interaction::Pressed);
                image.image = match (tracked, hovered) {
                    (true, true) => assets.journal_current_check_over.clone(),
                    (true, false) => assets.journal_current_check.clone(),
                    (false, true) => assets.journal_unchecked_over.clone(),
                    (false, false) => assets.journal_unchecked.clone(),
                };
            }
            _ => {}
        }
    }
}

pub(super) fn bind_journal_close_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &mut ImageNode),
        (With<JournalCloseButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut image) in &mut buttons {
        image.image = if journal_close_uses_hover_asset(interaction) {
            assets.journal_close_over.clone()
        } else {
            assets.journal_close.clone()
        };
    }
}

pub(super) fn bind_journal_primary_hover(
    mut buttons: Query<
        (&Interaction, &Children),
        (With<JournalPrimaryButton>, Changed<Interaction>),
    >,
    mut text_colors: Query<&mut TextColor>,
) {
    let style = mission_gui_style("FusionFallMissionSkin", "acceptbut")
        .expect("Retrobution journal accept button style");
    for (interaction, children) in &mut buttons {
        let state = match interaction {
            Interaction::Hovered => "hover",
            Interaction::Pressed => "active",
            Interaction::None => "normal",
        };
        let color = mission_text_state_color(style, state);
        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                **text_color = color;
            }
        }
    }
}

pub(super) fn bind_journal_aux_button_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (
            &Interaction,
            &JournalAuxButtonKind,
            Option<&Children>,
            &mut ImageNode,
        ),
        Changed<Interaction>,
    >,
    mut text_colors: Query<&mut TextColor>,
) {
    for (interaction, kind, children, mut image) in &mut buttons {
        let state = match interaction {
            Interaction::Hovered => "hover",
            Interaction::Pressed => "active",
            Interaction::None => "normal",
        };
        let (style_name, normal, hover) = match kind {
            JournalAuxButtonKind::Blue => ("button", &assets.blue_button, &assets.blue_button_over),
            JournalAuxButtonKind::Red => ("RedButton", &assets.red_button, &assets.red_button_over),
            JournalAuxButtonKind::Cancel => {
                ("CancelButton", &assets.cancel_normal, &assets.blue_button)
            }
            JournalAuxButtonKind::Help => {
                image.image = if matches!(interaction, Interaction::Hovered) {
                    assets.journal_help_over.clone()
                } else {
                    assets.journal_help.clone()
                };
                continue;
            }
        };
        image.image = if matches!(interaction, Interaction::Hovered) {
            hover.clone()
        } else {
            normal.clone()
        };
        let style = mission_gui_style("FusionFallMissionSkin", style_name)
            .expect("Retrobution journal auxiliary button style");
        let color = mission_text_state_color(style, state);
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut text_color) = text_colors.get_mut(child) {
                    **text_color = color;
                }
            }
        }
    }
}

pub(super) fn bind_journal_tab_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<(&Interaction, &JournalTabButton, &Children), Changed<Interaction>>,
    mut backgrounds: Query<(&JournalTabBackground, &mut ImageNode)>,
    mut text_colors: Query<&mut TextColor>,
) {
    for (interaction, tab, children) in &mut buttons {
        let (state, hovered) = match interaction {
            Interaction::Hovered => ("hover", true),
            Interaction::Pressed => ("active", false),
            Interaction::None => ("normal", false),
        };
        for (background_tab, mut image) in &mut backgrounds {
            if background_tab.0 != tab.0 {
                continue;
            }
            image.image = match (tab.0, hovered) {
                (JournalListTab::Active, true) => assets.journal_active_tab_over.clone(),
                (JournalListTab::Active, false) => assets.journal_active_tab.clone(),
                (JournalListTab::Completed, true) => assets.journal_completed_tab_over.clone(),
                (JournalListTab::Completed, false) => assets.journal_completed_tab.clone(),
            };
            image.color = if matches!(interaction, Interaction::Pressed) {
                Color::NONE
            } else {
                Color::WHITE
            };
        }

        let style_name = match tab.0 {
            JournalListTab::Active => "Tab1",
            JournalListTab::Completed => "Tab2",
        };
        let style = mission_gui_style("FusionFallMissionSkinR", style_name)
            .expect("Retrobution journal tab style");
        let color = mission_text_state_color(style, state);
        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                **text_color = color;
            }
        }
    }
}

pub(super) fn bind_journal_category_toggle_hover(
    model: Res<MissionUiModel>,
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &JournalCategoryToggleButton, &mut ImageNode),
        Changed<Interaction>,
    >,
) {
    for (interaction, marker, mut image) in &mut buttons {
        let expanded = model
            .completed_category_expanded
            .get(marker.0)
            .copied()
            .unwrap_or(false);
        let hovered = matches!(interaction, Interaction::Hovered);
        image.image = match (expanded, hovered) {
            (true, true) => assets.journal_category_collapse_over.clone(),
            (true, false) => assets.journal_category_collapse.clone(),
            (false, true) => assets.journal_category_expand_over.clone(),
            (false, false) => assets.journal_category_expand.clone(),
        };
    }
}

pub(super) fn journal_close_uses_hover_asset(interaction: &Interaction) -> bool {
    matches!(interaction, Interaction::Hovered)
}
