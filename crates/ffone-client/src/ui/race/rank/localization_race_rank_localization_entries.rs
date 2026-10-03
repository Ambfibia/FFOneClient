use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankLocalizationEntry {
    pub key: &'static str,
    pub en: &'static str,
    pub ru: &'static str,
}

pub const RACE_RANK_LOCALIZATION_ENTRIES: [RaceRankLocalizationEntry; 28] = [
    RaceRankLocalizationEntry {
        key: RACE_RANK_TITLE_KEY,
        en: "RANKINGS",
        ru: "РЕЙТИНГИ",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_LOCATIONS_KEY,
        en: "LOCATIONS",
        ru: "ЛОКАЦИИ",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_LOCATION_LABEL_KEY,
        en: "LOCATION:",
        ru: "ЛОКАЦИЯ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_HEADER_RANK_KEY,
        en: "RANK",
        ru: "МЕСТО",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_HEADER_PLAYER_KEY,
        en: "PLAYER",
        ru: "ИГРОК",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_HEADER_TOP_SCORE_KEY,
        en: "TOP SCORE",
        ru: "ЛУЧШИЙ СЧЁТ",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_NO_SCORE_KEY,
        en: "No score registered yet.",
        ru: "Результатов пока нет.",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_PERIOD_TODAY_KEY,
        en: "today",
        ru: "сегодня",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_PERIOD_WEEK_KEY,
        en: "this week",
        ru: "за неделю",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_PERIOD_MONTH_KEY,
        en: "this month",
        ru: "за месяц",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_PERIOD_ALL_TIME_KEY,
        en: "all time",
        ru: "за всё время",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_BEST_TODAY_KEY,
        en: "MY BEST SCORE TODAY:",
        ru: "МОЙ ЛУЧШИЙ СЧЁТ СЕГОДНЯ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_BEST_WEEK_KEY,
        en: "MY BEST SCORE THIS WEEK:",
        ru: "МОЙ ЛУЧШИЙ СЧЁТ ЗА НЕДЕЛЮ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_BEST_MONTH_KEY,
        en: "MY BEST SCORE THIS MONTH:",
        ru: "МОЙ ЛУЧШИЙ СЧЁТ ЗА МЕСЯЦ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_BEST_ALL_TIME_KEY,
        en: "MY ALL TIME BEST SCORE:",
        ru: "МОЙ ЛУЧШИЙ СЧЁТ ЗА ВСЁ ВРЕМЯ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_TOP_TODAY_KEY,
        en: " TODAY'S TOP 10:",
        ru: " ТОП-10 ЗА СЕГОДНЯ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_TOP_WEEK_KEY,
        en: " THIS WEEK'S TOP 10:",
        ru: " ТОП-10 ЗА НЕДЕЛЮ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_TOP_MONTH_KEY,
        en: " THIS MONTH'S TOP 10:",
        ru: " ТОП-10 ЗА МЕСЯЦ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_TOP_ALL_TIME_KEY,
        en: " ALL TIME TOP 10:",
        ru: " ТОП-10 ЗА ВСЁ ВРЕМЯ:",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_NPC_NAME_KEY,
        en: "{name}",
        ru: "{name}",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_LOCATION_NAME_KEY,
        en: "{location}",
        ru: "{location}",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_AREA_NAME_KEY,
        en: "{area}",
        ru: "{area}",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_ROW_NAME_KEY,
        en: "{location}",
        ru: "{location}",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_ROW_AREA_KEY,
        en: "{area}",
        ru: "{area}",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_PAGE_KEY,
        en: "{first} - {last} of {total} Locations",
        ru: "{first} - {last} из {total} локаций",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_VALUE_KEY,
        en: "{rank}",
        ru: "{rank}",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_PLAYER_VALUE_KEY,
        en: "{player}",
        ru: "{player}",
    },
    RaceRankLocalizationEntry {
        key: RACE_RANK_SCORE_VALUE_KEY,
        en: "{score}",
        ru: "{score}",
    },
];

pub(super) fn localized_fallback(localized: &LocalizedText) -> String {
    localized
        .args
        .iter()
        .fold(localized.fallback.clone(), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
}

pub(super) fn race_rank_location_name_localized(value: &str) -> LocalizedText {
    LocalizedText::new(RACE_RANK_LOCATION_NAME_KEY, "{location}").with_arg("location", value)
}

pub(super) fn race_rank_area_name_localized(value: &str) -> LocalizedText {
    LocalizedText::new(RACE_RANK_AREA_NAME_KEY, "{area}").with_arg("area", value)
}

pub(super) fn race_rank_npc_name_localized(value: &str) -> LocalizedText {
    LocalizedText::new(RACE_RANK_NPC_NAME_KEY, "{name}").with_arg("name", value)
}

pub(super) fn race_rank_row_localized(part: RaceRankRowPart, value: &str) -> LocalizedText {
    match part {
        RaceRankRowPart::Name => {
            LocalizedText::new(RACE_RANK_ROW_NAME_KEY, "{location}").with_arg("location", value)
        }
        RaceRankRowPart::Area => {
            LocalizedText::new(RACE_RANK_ROW_AREA_KEY, "{area}").with_arg("area", value)
        }
        RaceRankRowPart::Icon
        | RaceRankRowPart::Selection
        | RaceRankRowPart::Outline
        | RaceRankRowPart::Point => unreachable!("non-text row part"),
    }
}

pub(super) fn race_rank_score_localized(column: RaceRankScoreColumn, value: &str) -> LocalizedText {
    match column {
        RaceRankScoreColumn::Rank => {
            LocalizedText::new(RACE_RANK_VALUE_KEY, "{rank}").with_arg("rank", value)
        }
        RaceRankScoreColumn::Player => {
            LocalizedText::new(RACE_RANK_PLAYER_VALUE_KEY, "{player}").with_arg("player", value)
        }
        RaceRankScoreColumn::Score => {
            LocalizedText::new(RACE_RANK_SCORE_VALUE_KEY, "{score}").with_arg("score", value)
        }
    }
}
