use super::*;

pub const NANO_FREE_TUNING_ACQUIRED_LOCALIZATION_KEY: &str = "ui.nano_free_tuning.acquired";

pub const NANO_FREE_TUNING_TITLE_LOCALIZATION_KEY: &str = "ui.nano_free_tuning.title";

pub const NANO_FREE_TUNING_SELECT_LOCALIZATION_KEY: &str = "ui.nano_free_tuning.select";

pub const NANO_FREE_TUNING_BANG_LOCALIZATION_KEY: &str = "ui.nano_free_tuning.bang";

pub const NANO_FREE_TUNING_NANO_NAME_LOCALIZATION_KEY: &str = "ui.nano_free_tuning.nano_name";

pub const NANO_FREE_TUNING_POWER_NAME_LOCALIZATION_KEY: &str = "ui.nano_free_tuning.power.name";

pub const NANO_FREE_TUNING_POWER_TYPE_LOCALIZATION_KEY: &str = "ui.nano_free_tuning.power.type";

pub const NANO_FREE_TUNING_POWER_DESCRIPTION_LOCALIZATION_KEY: &str =
    "ui.nano_free_tuning.power.description";

pub(super) fn nano_free_tuning_localized_text(role: NanoFreeTuningTextRole, copy: &str) -> LocalizedText {
    match role {
        NanoFreeTuningTextRole::AcquiredPrefix => LocalizedText::new(
            NANO_FREE_TUNING_ACQUIRED_LOCALIZATION_KEY,
            NANO_FREE_TUNING_COPY_ACQUIRED,
        ),
        NanoFreeTuningTextRole::Bang => LocalizedText::new(
            NANO_FREE_TUNING_BANG_LOCALIZATION_KEY,
            NANO_FREE_TUNING_COPY_BANG,
        ),
        NanoFreeTuningTextRole::NanoName => {
            LocalizedText::new(NANO_FREE_TUNING_NANO_NAME_LOCALIZATION_KEY, " {name} ")
                .with_arg("name", copy)
        }
        NanoFreeTuningTextRole::PowerName(_) => {
            LocalizedText::new(NANO_FREE_TUNING_POWER_NAME_LOCALIZATION_KEY, "{name}")
                .with_arg("name", copy)
        }
        NanoFreeTuningTextRole::PowerType(_) => {
            LocalizedText::new(NANO_FREE_TUNING_POWER_TYPE_LOCALIZATION_KEY, "{type}")
                .with_arg("type", copy)
        }
        NanoFreeTuningTextRole::PowerDescription(_) => LocalizedText::new(
            NANO_FREE_TUNING_POWER_DESCRIPTION_LOCALIZATION_KEY,
            "{description}",
        )
        .with_arg("description", copy),
    }
}
