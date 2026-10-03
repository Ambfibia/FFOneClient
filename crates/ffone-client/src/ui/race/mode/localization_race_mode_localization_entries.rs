use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceModeLocalizationEntry {
    pub key: &'static str,
    pub en: &'static str,
    pub ru: &'static str,
}

/// Authored bundle entries required by the reachable clean `OnEndGUI` tree.
/// The unreachable `StartWindow` and `OnFailGUI` copies are intentionally
/// absent from this contract.
pub const RACE_MODE_LOCALIZATION_ENTRIES: [RaceModeLocalizationEntry; 19] = [
    RaceModeLocalizationEntry {
        key: RACE_MODE_TIME_KEY,
        en: RACE_COPY_TIME,
        ru: "ВРЕМЯ",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_PODS_KEY,
        en: RACE_COPY_PODS,
        ru: "КАПСУЛЫ",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_SCORE_KEY,
        en: RACE_COPY_SCORE,
        ru: "СЧЁТ",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_MY_BEST_KEY,
        en: RACE_COPY_MY_BEST,
        ru: "МОЙ ЛУЧШИЙ РЕЗУЛЬТАТ",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_REWARD_KEY,
        en: RACE_COPY_REWARD,
        ru: "НАГРАДА:",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_ACCEPT_KEY,
        en: RACE_COPY_ACCEPT,
        ru: "ПРИНЯТЬ",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_INVENTORY_FULL_KEY,
        en: RACE_COPY_INVENTORY_FULL,
        ru: "ИНВЕНТАРЬ ПОЛОН!",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_TIME_VALUE_KEY,
        en: "{time}",
        ru: "{time}",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_PODS_VALUE_KEY,
        en: "{pods}",
        ru: "{pods}",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_SCORE_VALUE_KEY,
        en: "{score}",
        ru: "{score}",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_FUSION_MATTER_KEY,
        en: "{amount}\nFUSION MATTER",
        ru: "{amount}\nМАТЕРИЯ ФЬЮЖН",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_ITEM_NAME_KEY,
        en: "{name}",
        ru: "{name}",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_ITEM_LEVEL_KEY,
        en: "LEVEL {level}",
        ru: "УРОВЕНЬ {level}",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_RATING_NONE_KEY,
        en: "",
        ru: "",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_RATING_GENIUS_KEY,
        en: "Genius!",
        ru: "Гениально!",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_RATING_AWESOME_KEY,
        en: "Awesome!",
        ru: "Потрясающе!",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_RATING_GOOD_KEY,
        en: "Good",
        ru: "Хорошо",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_RATING_NOT_BAD_KEY,
        en: "Not Bad",
        ru: "Неплохо",
    },
    RaceModeLocalizationEntry {
        key: RACE_MODE_RATING_BLEH_KEY,
        en: "Bleh!",
        ru: "Так себе!",
    },
];

#[must_use]
pub fn race_rank_localized(rank: i32) -> LocalizedText {
    match race_filled_star_count(rank) {
        5 => LocalizedText::new(RACE_MODE_RATING_GENIUS_KEY, "Genius!"),
        4 => LocalizedText::new(RACE_MODE_RATING_AWESOME_KEY, "Awesome!"),
        3 => LocalizedText::new(RACE_MODE_RATING_GOOD_KEY, "Good"),
        2 => LocalizedText::new(RACE_MODE_RATING_NOT_BAD_KEY, "Not Bad"),
        1 => LocalizedText::new(RACE_MODE_RATING_BLEH_KEY, "Bleh!"),
        _ => LocalizedText::new(RACE_MODE_RATING_NONE_KEY, ""),
    }
}

#[must_use]
pub fn race_time_localized(total_seconds: i32) -> LocalizedText {
    LocalizedText::new(RACE_MODE_TIME_VALUE_KEY, "{time}")
        .with_arg("time", race_time_copy(total_seconds))
}

#[must_use]
pub fn race_pods_localized(pods: i32) -> LocalizedText {
    LocalizedText::new(RACE_MODE_PODS_VALUE_KEY, "{pods}").with_arg("pods", pods.to_string())
}

#[must_use]
pub fn race_score_localized(score: i32) -> LocalizedText {
    LocalizedText::new(RACE_MODE_SCORE_VALUE_KEY, "{score}")
        .with_arg("score", race_score_copy(score))
}

#[must_use]
pub fn race_fusion_matter_localized(amount: i32) -> LocalizedText {
    LocalizedText::new(RACE_MODE_FUSION_MATTER_KEY, "{amount}\nFUSION MATTER")
        .with_arg("amount", race_score_copy(amount))
}

#[must_use]
pub fn race_reward_item_name_localized(name: &str) -> LocalizedText {
    LocalizedText::new(RACE_MODE_ITEM_NAME_KEY, "{name}").with_arg("name", name)
}

#[must_use]
pub fn race_reward_item_level_localized(level: i32) -> LocalizedText {
    LocalizedText::new(RACE_MODE_ITEM_LEVEL_KEY, "LEVEL {level}")
        .with_arg("level", level.to_string())
}

pub(super) fn initial_race_mode_localized(role: RaceModeTextRole) -> LocalizedText {
    match role {
        RaceModeTextRole::RankCopy => LocalizedText::new(RACE_MODE_RATING_NONE_KEY, ""),
        RaceModeTextRole::CurrentTime | RaceModeTextRole::BestTime => {
            LocalizedText::new(RACE_MODE_TIME_VALUE_KEY, "{time}").with_arg("time", "")
        }
        RaceModeTextRole::CurrentPods | RaceModeTextRole::BestPods => {
            LocalizedText::new(RACE_MODE_PODS_VALUE_KEY, "{pods}").with_arg("pods", "")
        }
        RaceModeTextRole::CurrentScore | RaceModeTextRole::BestScore => {
            LocalizedText::new(RACE_MODE_SCORE_VALUE_KEY, "{score}").with_arg("score", "")
        }
        RaceModeTextRole::ItemName => {
            LocalizedText::new(RACE_MODE_ITEM_NAME_KEY, "{name}").with_arg("name", "")
        }
        RaceModeTextRole::ItemLevel => {
            LocalizedText::new(RACE_MODE_ITEM_LEVEL_KEY, "LEVEL {level}").with_arg("level", "")
        }
        RaceModeTextRole::InventoryFull => {
            LocalizedText::new(RACE_MODE_INVENTORY_FULL_KEY, RACE_COPY_INVENTORY_FULL)
        }
        RaceModeTextRole::FusionMatter => {
            LocalizedText::new(RACE_MODE_FUSION_MATTER_KEY, "{amount}\nFUSION MATTER")
                .with_arg("amount", "")
        }
    }
}

pub(super) fn localized_fallback(localized: &LocalizedText) -> String {
    localized
        .args
        .iter()
        .fold(localized.fallback.clone(), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
}
