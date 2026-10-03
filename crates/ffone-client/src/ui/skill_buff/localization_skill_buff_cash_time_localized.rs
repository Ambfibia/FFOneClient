use super::*;

#[must_use]
pub fn skill_buff_cash_time_localized(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.skill_buff.cash_time", "{time}").with_arg("time", value)
}
