//! Localization keys and source-text resolution for mission UI labels.

use super::system_dialog::TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT;
use crate::localization::{Language, Localization, LocalizedText, localized_tutorial_mission_text};
use bevy::prelude::*;

pub(super) const JOURNAL_MISSION_OFFER_LABEL: &str = "MISSION OFFER";
pub(super) const JOURNAL_MISSION_DETAILS_LABEL: &str = "MISSION DETAILS";
pub(super) const JOURNAL_MY_NOTES_LABEL: &str = "MY NOTES";
pub(super) const JOURNAL_MISSION_SUMMARY_LABEL: &str = "MISSION SUMMARY";
pub(super) const JOURNAL_MISSION_COMPLETION_LABEL: &str = "MISSION COMPLETION";
pub(super) const JOURNAL_REWARD_LABEL: &str = "REWARD";
pub(super) const JOURNAL_FUSION_MATTER_LABEL: &str = "FUSION MATTER";
pub(super) const JOURNAL_TAROS_LABEL: &str = "TAROS";
pub(super) const NANOCOM_MENU_LABELS: [&str; 7] = [
    "MY STUFF",
    "JOURNAL",
    "E-MAIL",
    "MAP",
    "SETTINGS",
    "GAME GUIDE",
    "EXIT GAME",
];

pub(super) const MISSION_UI_SOURCE_KEYS: &[(&str, &str)] = &[
    ("Active missions:", "ui.mission.npc.active_missions"),
    ("Available missions:", "ui.mission.npc.available_missions"),
    ("WARP", "ui.mission.npc.warp"),
    ("CLOSE", "ui.mission.common.close"),
    ("Mission Journal", "ui.mission.journal.title"),
    ("Active Mission List", "ui.mission.journal.active_list"),
    ("Active", "ui.mission.journal.active"),
    ("Completed", "ui.mission.journal.completed"),
    ("NANO MISSION", "ui.mission.journal.category.nano"),
    ("GUIDE MISSION", "ui.mission.journal.category.guide"),
    ("WORLD MISSIONS", "ui.mission.journal.category.world"),
    ("CURRENT MISSION", "ui.mission.journal.current"),
    ("No mission selected", "ui.mission.journal.no_selection"),
    ("Guide", "ui.mission.type.guide"),
    ("Nano", "ui.mission.type.nano"),
    ("Quest", "ui.mission.type.quest"),
    ("Easy", "ui.mission.difficulty.easy"),
    ("Normal", "ui.mission.difficulty.normal"),
    ("Hard", "ui.mission.difficulty.hard"),
    (JOURNAL_MISSION_OFFER_LABEL, "ui.mission.journal.offer"),
    (JOURNAL_MISSION_DETAILS_LABEL, "ui.mission.journal.details"),
    (JOURNAL_MY_NOTES_LABEL, "ui.mission.journal.notes"),
    (JOURNAL_MISSION_SUMMARY_LABEL, "ui.mission.journal.summary"),
    (
        JOURNAL_MISSION_COMPLETION_LABEL,
        "ui.mission.journal.completion",
    ),
    ("MISSION OFFER:", "ui.mission.journal.offer_header"),
    ("MISSION DETAILS:", "ui.mission.journal.details_header"),
    ("MY NOTES:", "ui.mission.journal.notes_header"),
    ("MISSION SUMMARY:", "ui.mission.journal.summary_header"),
    (JOURNAL_REWARD_LABEL, "ui.mission.journal.reward"),
    ("REWARD:", "ui.mission.journal.reward_header"),
    ("NANO:", "ui.mission.journal.nano_header"),
    (
        JOURNAL_FUSION_MATTER_LABEL,
        "ui.mission.reward.fusion_matter",
    ),
    (JOURNAL_TAROS_LABEL, "ui.mission.reward.taros"),
    ("ACCEPT MISSION", "ui.mission.journal.accept"),
    ("COMPLETE MISSION", "ui.mission.journal.complete"),
    ("DECLINE", "ui.mission.journal.decline"),
    ("DELETE MISSION", "ui.mission.journal.delete"),
    ("MAKE CURRENT MISSION", "ui.mission.journal.make_current"),
    ("MY STUFF", "ui.mission.nanocom.inventory"),
    ("JOURNAL", "ui.mission.nanocom.journal"),
    ("E-MAIL", "ui.mission.nanocom.email"),
    ("MAP", "ui.mission.nanocom.map"),
    ("SETTINGS", "ui.mission.nanocom.settings"),
    ("GAME GUIDE", "ui.mission.nanocom.guide"),
    ("EXIT GAME", "ui.mission.nanocom.exit"),
    ("LEAVE GROUP", "ui.mission.chat.leave_group"),
    ("WARP AWAY", "ui.mission.chat.warp_away"),
    ("HOP ON VEHICLE", "ui.mission.chat.vehicle"),
    (
        TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT,
        "ui.mission.tutorial_exit.prompt",
    ),
    ("OKAY", "ui.mission.common.okay"),
    ("CANCEL", "ui.common.cancel"),
    ("ENTER STORE", "ui.mission.npc.service.store"),
    ("NANO STATION", "ui.mission.npc.service.nano_station"),
    ("BANK", "ui.mission.npc.service.bank"),
    ("LOCAL BANK", "ui.mission.npc.service.local_bank"),
    (" GUIDE CHANGER", "ui.mission.npc.service.guide_changer"),
    (" WARP TO PAST", "ui.mission.npc.service.past_warp"),
    (" WARP", "ui.mission.npc.service.transportation"),
    (" START RACE", "ui.mission.npc.service.race"),
    (" CANCEL RACE", "ui.mission.npc.service.race_cancel"),
    (" RACE RANK", "ui.mission.npc.service.race_rank"),
    (" COMBINE ITEMS", "ui.mission.npc.service.combine"),
    (" BARBER", "ui.mission.npc.service.barber"),
    (" ENCHANT ITEMS", "ui.mission.npc.service.enchant"),
    ("RULES", "ui.mission.npc.service.rules"),
    ("Numbuh Two", "content.npc.2671.name"),
    ("Buttercup", "content.npc.2672.name"),
    ("Dexter", "content.npc.2673.name"),
    ("Infected zone", "content.location.infected_zone"),
    (
        "Fusion Buttercup's Lair",
        "content.location.fusion_buttercup_lair",
    ),
    ("Tech Square", "content.location.tech_square"),
    ("Pokey Oaks North", "content.location.pokey_oaks_north"),
];

pub(super) fn mission_ui_localized_text(source: impl Into<String>) -> LocalizedText {
    let source = source.into();
    if let Some((_, key)) = MISSION_UI_SOURCE_KEYS
        .iter()
        .find(|(candidate, _)| *candidate == source)
    {
        LocalizedText::new(*key, source)
    } else {
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", source)
    }
}

pub(super) fn mission_content_text(task_id: i32, field: &str, fallback: &str) -> LocalizedText {
    localized_tutorial_mission_text(task_id, field, fallback)
}

pub(super) fn nano_content_text(nano_id: i32, field: &str, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.nano.{nano_id}.{field}"),
        fallback.to_owned(),
    )
}

pub(super) fn nano_skill_content_text(tune_id: i32, field: &str, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.nano_tune.{tune_id}.{field}"),
        fallback.to_owned(),
    )
}

pub(super) fn resolve_mission_ui_text(
    localization: Option<&Localization>,
    language: Option<&Language>,
    localized: &LocalizedText,
) -> String {
    match (localization, language) {
        (Some(localization), Some(language)) => localization.text(language, localized),
        _ => localized
            .args
            .iter()
            .fold(localized.fallback.clone(), |text, (name, value)| {
                text.replace(&format!("{{{name}}}"), value)
            }),
    }
}
