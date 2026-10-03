//! Binding of the journal frame, selected mission details, rewards and Nano reward card.

use super::super::assets::{ACTIVE_DIALOG, END_DIALOG, OFFER_DIALOG};
use super::super::components::MissionUiView;
use super::super::geometry::{
    JOURNAL_ACTIVE_GROUP_RECT, JOURNAL_ALLOW_GROUP_RECT, JOURNAL_COMPLETED_GROUP_RECT,
    JOURNAL_COMPLETED_NANO_GROUP_RECT, JOURNAL_COMPLETED_NANO_LABEL_RECT,
    JOURNAL_REWARD_GROUP_RECT,
};
use super::super::labels::{
    JOURNAL_MISSION_COMPLETION_LABEL, JOURNAL_MISSION_OFFER_LABEL, mission_content_text,
    mission_ui_localized_text, nano_content_text, nano_skill_content_text, resolve_mission_ui_text,
};
use super::super::layout::{
    MissionUiRect, journal_primary_is_visible, journal_primary_rect, journal_primary_text_rect,
};
use super::super::model::{JournalListTab, MissionJournalUi};
use super::super::widgets::{
    journal_managed_label, journal_objective_label, mission_text_font, mission_text_layout_rect,
    styled_image,
};
use super::MissionUiBindContext;
use crate::{gui_skin::gui_style as mission_gui_style, localization::LocalizedText};
use bevy::prelude::*;

/// Journal window frame, selected mission details, rewards and primary button.
pub(super) fn bind_journal_detail_view(
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
        localization,
        language,
        mission,
        nano,
        completed_journal,
        other_journal,
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
        MissionUiView::JournalRoot => {
            if let Some(node) = &mut node {
                node.display = if journal_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalBackdrop => {
            if let Some(node) = &mut node {
                node.display = if journal_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalBackground => {
            if let Some(image) = &mut image {
                *image.as_mut() = match &model.journal {
                    MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. } => {
                        styled_image(&assets.offer_dialog, OFFER_DIALOG)
                    }
                    MissionJournalUi::Other(_) if completed_journal => {
                        styled_image(&assets.end_dialog, END_DIALOG)
                    }
                    MissionJournalUi::Other(_) => {
                        styled_image(&assets.active_dialog, ACTIVE_DIALOG)
                    }
                    MissionJournalUi::Hidden => {
                        styled_image(&assets.offer_dialog, OFFER_DIALOG)
                    }
                };
            }
            if let Some(node) = &mut node {
                *node.as_mut() = match &model.journal {
                    MissionJournalUi::Allow(_) => JOURNAL_ALLOW_GROUP_RECT.node(),
                    MissionJournalUi::Reward { .. } => JOURNAL_REWARD_GROUP_RECT.node(),
                    MissionJournalUi::Other(_) if completed_journal => {
                        JOURNAL_COMPLETED_GROUP_RECT.node()
                    }
                    MissionJournalUi::Other(_) => JOURNAL_ACTIVE_GROUP_RECT.node(),
                    MissionJournalUi::Hidden => JOURNAL_ALLOW_GROUP_RECT.node(),
                };
                node.display = if journal_visible && (!other_journal || mission.is_some()) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalEmptySelectionPanel
        | MissionUiView::JournalEmptySelectionText => {
            if let Some(node) = &mut node {
                node.display = if journal_visible && other_journal && mission.is_none() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalMissionBanner => {
            if let Some(node) = &mut node {
                node.display = if journal_visible && mission.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                node.top = px(if other_journal { 31.0 } else { 29.0 });
            }
            if let Some(image) = &mut image {
                image.image = mission.map_or_else(
                    || assets.journal_world_banner.clone(),
                    |mission| assets.journal_banner(mission.mission_type).clone(),
                );
            }
        }
        MissionUiView::JournalWindowTitle => {
            if let Some(node) = &mut node {
                node.display = if matches!(model.journal, MissionJournalUi::Other(_)) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalModeTitle => {
            if let Some(node) = &mut node {
                node.display = if matches!(
                    model.journal,
                    MissionJournalUi::Allow(_) | MissionJournalUi::Reward { .. }
                ) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            set_text!(match model.journal {
                MissionJournalUi::Allow(_) => {
                    mission_ui_localized_text(JOURNAL_MISSION_OFFER_LABEL)
                }
                MissionJournalUi::Reward { .. } => {
                    mission_ui_localized_text(JOURNAL_MISSION_COMPLETION_LABEL)
                }
                MissionJournalUi::Hidden | MissionJournalUi::Other(_) => {
                    mission_ui_localized_text("")
                }
            });
        }
        MissionUiView::JournalMissionName => {
            set_text!(mission.map_or_else(
                || mission_ui_localized_text(""),
                |mission| {
                    let kind = match mission.mission_type {
                        1 => "Guide",
                        2 => "Nano",
                        _ => "Quest",
                    };
                    let kind = resolve_mission_ui_text(
                        localization.as_deref(),
                        language.as_deref(),
                        &mission_ui_localized_text(kind),
                    );
                    let title = resolve_mission_ui_text(
                        localization.as_deref(),
                        language.as_deref(),
                        &mission_content_text(mission.task_id, "title", &mission.title),
                    );
                    LocalizedText::new("ui.mission.journal.mission_name", "{kind} : {title}")
                        .with_arg("kind", kind)
                        .with_arg("title", title)
                },
            ));
        }
        MissionUiView::JournalMissionDifficulty => {
            set_text!(mission.map_or_else(
                || mission_ui_localized_text(""),
                |mission| {
                    let difficulty = match mission.difficulty_type {
                        0 => "Easy",
                        2 => "Hard",
                        _ => "Normal",
                    };
                    let difficulty = resolve_mission_ui_text(
                        localization.as_deref(),
                        language.as_deref(),
                        &mission_ui_localized_text(difficulty),
                    );
                    LocalizedText::new(
                        "ui.mission.journal.level_difficulty",
                        "Level {level} {difficulty}",
                    )
                    .with_arg("level", mission.required_level.to_string())
                    .with_arg("difficulty", difficulty)
                },
            ));
        }
        MissionUiView::JournalNpcName => {
            set_text!(mission.map_or_else(
                || mission_ui_localized_text(""),
                |mission| LocalizedText::new(
                    format!("content.npc.{}.name", mission.journal_npc_type),
                    mission.npc_name.clone(),
                ),
            ));
        }
        MissionUiView::JournalNpcPosition => {
            set_text!(mission.map_or_else(
                || mission_ui_localized_text(""),
                |mission| mission_ui_localized_text(mission.npc_position.clone()),
            ));
        }
        MissionUiView::JournalNpcPortrait => {
            if let Some(node) = &mut node {
                node.display = if mission.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            if let Some(image) = &mut image {
                image.image = mission.map_or_else(
                    || assets.journal_npc_portraits[0].clone(),
                    |mission| {
                        assets.journal_npc_portrait(
                            &content,
                            &asset_server,
                            mission.journal_npc_type,
                        )
                    },
                );
            }
        }
        MissionUiView::JournalObjectiveHeader => {
            set_text!(mission_ui_localized_text(journal_managed_label(
                journal_objective_label(&model),
            )));
        }
        MissionUiView::JournalDescription => {
            set_text!(mission.map_or_else(
                || mission_ui_localized_text(""),
                |mission| match &model.journal {
                    MissionJournalUi::Allow(_) => mission_content_text(
                        mission.task_id,
                        "offer_description",
                        &mission.offer_description,
                    ),
                    MissionJournalUi::Reward { .. } => mission_content_text(
                        mission.task_id,
                        "completion_description",
                        &mission.completion_description,
                    ),
                    MissionJournalUi::Other(_)
                        if model.journal_tab == JournalListTab::Completed =>
                    {
                        mission_content_text(
                            mission.task_id,
                            "mission_complete_summary",
                            &mission.mission_complete_summary,
                        )
                    }
                    MissionJournalUi::Other(_)
                        if model.selected_journal_task_id == Some(mission.task_id) =>
                    {
                        let objective = resolve_mission_ui_text(
                            localization.as_deref(),
                            language.as_deref(),
                            &mission_content_text(
                                mission.task_id,
                                "objective",
                                &mission.objective,
                            ),
                        );
                        let details = resolve_mission_ui_text(
                            localization.as_deref(),
                            language.as_deref(),
                            &mission_content_text(
                                mission.task_id,
                                "task_description",
                                &mission.task_description,
                            ),
                        );
                        LocalizedText::new(
                            "ui.mission.journal.active_description",
                            "{objective}\n\n{details}",
                        )
                        .with_arg("objective", objective)
                        .with_arg("details", details)
                    }
                    MissionJournalUi::Other(_) => mission_content_text(
                        mission.task_id,
                        "task_description",
                        &mission.task_description,
                    ),
                    MissionJournalUi::Hidden => mission_ui_localized_text(""),
                },
            ));
        }
        MissionUiView::JournalRewardHeader => {
            if let Some(node) = &mut node {
                node.display = if nano.is_none()
                    && mission.is_some_and(|mission| !mission.rewards.is_empty())
                {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalRewardCard => {
            if let Some(node) = &mut node {
                node.display = if nano.is_none()
                    && mission.is_some_and(|mission| mission.rewards.fusion_matter > 0)
                {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalCashCard => {
            if let Some(node) = &mut node {
                node.display = if nano.is_none()
                    && mission.is_some_and(|mission| mission.rewards.cash > 0)
                {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::JournalCashAmount => {
            let cash = mission.map_or(0, |mission| mission.rewards.cash);
            if let Some(node) = &mut node {
                node.display = if nano.is_none() && cash > 0 {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            set_text!(mission_ui_localized_text(if cash > 0 {
                cash.to_string()
            } else {
                String::new()
            }));
        }
        MissionUiView::JournalRewardAmount => {
            let fusion_matter = mission.map_or(0, |mission| mission.rewards.fusion_matter);
            if let Some(node) = &mut node {
                node.display = if nano.is_none() && fusion_matter > 0 {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            set_text!(mission_ui_localized_text(if fusion_matter > 0 {
                fusion_matter.to_string()
            } else {
                String::new()
            }));
        }

        MissionUiView::JournalPrimary => {
            if let Some(node) = &mut node {
                node.display = if journal_primary_is_visible(&model) {
                    Display::Flex
                } else {
                    Display::None
                };
                let rect = journal_primary_rect(&model.journal);
                node.left = px(rect.x);
                node.top = px(rect.y);
                node.width = px(rect.width);
                node.height = px(rect.height);
            }
            if let Some(image) = &mut image {
                image.color = if model.pending.is_some() {
                    Color::srgba(0.55, 0.55, 0.55, 0.8)
                } else {
                    Color::WHITE
                };
            }
        }

        MissionUiView::JournalPrimaryText => {
            if let Some(node) = &mut node {
                // The clean reward branch changes the complete button from
                // the offer branch's 218x85 Rect to 233x70. Keep the
                // MiddleCenter text child on that same live Rect; leaving
                // the spawned offer height here pushes/clips the reward
                // caption at the lower edge.
                let rect = journal_primary_text_rect(&model.journal);
                node.left = px(rect.x);
                node.top = px(rect.y);
                node.width = px(rect.width);
                node.height = px(rect.height);
            }
            set_text!(match model.journal {
                MissionJournalUi::Allow(_) => mission_ui_localized_text("ACCEPT MISSION"),
                MissionJournalUi::Reward { .. } => {
                    mission_ui_localized_text("COMPLETE MISSION")
                }
                MissionJournalUi::Hidden | MissionJournalUi::Other(_) => {
                    mission_ui_localized_text("")
                }
            });
        }
        _ => {}
    }
}

/// Journal Nano reward card views.
pub(super) fn bind_journal_nano_view(
    context: &MissionUiBindContext<'_>,
    view: &MissionUiView,
    node: Option<Mut<Node>>,
    localized_text: Option<Mut<LocalizedText>>,
    image: Option<Mut<ImageNode>>,
) {
    let MissionUiBindContext {
        assets,
        asset_server,
        nano,
        completed_journal,
        active_journal,
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
        MissionUiView::JournalNanoLabel => {
            if let Some(node) = &mut node {
                node.display = if nano.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                let outer = if completed_journal {
                    JOURNAL_COMPLETED_NANO_LABEL_RECT
                } else if active_journal {
                    MissionUiRect::new(24.0, 318.0, 200.0, 30.0)
                } else {
                    MissionUiRect::new(23.0, 301.0, 200.0, 30.0)
                };
                let style = mission_gui_style("FusionFallMissionSkin", "bigfont14")
                    .expect("Retrobution Nano label style");
                let font = mission_text_font(&assets, "FusionFallMissionSkin", "bigfont14");
                let content = mission_text_layout_rect(outer, style, &font);
                node.left = px(content.x);
                node.top = px(content.y);
                node.width = px(content.width);
                node.height = px(content.height);
            }
        }
        MissionUiView::JournalNanoGroup => {
            if let Some(node) = &mut node {
                node.display = if nano.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                let rect = if completed_journal {
                    JOURNAL_COMPLETED_NANO_GROUP_RECT
                } else if active_journal {
                    MissionUiRect::new(14.0, 335.0, 555.0, 245.0)
                } else {
                    MissionUiRect::new(13.0, 318.0, 555.0, 245.0)
                };
                node.left = px(rect.x);
                node.top = px(rect.y);
            }
        }
        MissionUiView::JournalNanoPortrait => {}
        MissionUiView::JournalNanoName => {
            set_text!(nano.map_or_else(
                || mission_ui_localized_text(""),
                |nano| nano_content_text(nano.nano_id, "name", &nano.name),
            ));
        }
        MissionUiView::JournalNanoAttribute => {
            set_text!(nano.map_or_else(
                || mission_ui_localized_text(""),
                |nano| nano_content_text(nano.nano_id, "attribute", &nano.attribute),
            ));
        }
        MissionUiView::JournalNanoDescription => {
            set_text!(nano.map_or_else(
                || mission_ui_localized_text(""),
                |nano| nano_content_text(nano.nano_id, "description", &nano.description),
            ));
        }
        MissionUiView::JournalNanoBox(_) => {}
        MissionUiView::JournalNanoSkillIcon(slot) => {
            if let Some(image) = &mut image {
                image.image = nano.and_then(|nano| nano.skills.get(slot)).map_or_else(
                    Handle::default,
                    |skill| {
                        asset_server.load(format!(
                            "ui/en/gameplay/nano/icons/skill/skillicon_{:02}.png",
                            skill.icon_number
                        ))
                    },
                );
            }
        }
        MissionUiView::JournalNanoSkillName(slot) => {
            set_text!(nano.and_then(|nano| nano.skills.get(slot)).map_or_else(
                || mission_ui_localized_text(""),
                |skill| nano_skill_content_text(skill.tune_id, "name", &skill.name),
            ));
        }
        MissionUiView::JournalNanoSkillType(slot) => {
            set_text!(
                nano.and_then(|nano| nano.skills.get(slot)).map_or_else(
                    || mission_ui_localized_text(""),
                    |skill| nano_skill_content_text(
                        skill.tune_id,
                        "type_label",
                        &skill.type_label,
                    ),
                )
            );
        }
        MissionUiView::JournalNanoSkillDescription(slot) => {
            set_text!(nano.and_then(|nano| nano.skills.get(slot)).map_or_else(
                || mission_ui_localized_text(""),
                |skill| nano_skill_content_text(
                    skill.tune_id,
                    "description",
                    &skill.description,
                ),
            ));
        }
        _ => {}
    }
}
