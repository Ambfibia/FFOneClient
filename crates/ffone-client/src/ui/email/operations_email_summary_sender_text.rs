use super::*;

pub(super) fn email_summary_sender_text(model: &EmailUiModel, message: &EmailSummary) -> LocalizedText {
    if model
        .buddies
        .iter()
        .any(|buddy| buddy.pc_uid == message.from_pc_uid && buddy.name_check_flag != 1)
    {
        email_player_label_text(message.from_pc_uid)
    } else {
        email_passthrough_text(message.sender_label())
    }
}

pub(super) fn email_recipient_text(model: &EmailUiModel) -> LocalizedText {
    let player_label = format!("Player {}", model.draft.recipient_pc_uid);
    if model.draft.recipient_pc_uid > 0 && model.draft.recipient_name == player_label {
        email_player_label_text(model.draft.recipient_pc_uid)
    } else {
        email_passthrough_text(model.draft.recipient_name.clone())
    }
}

pub(super) fn email_day_text(sent: EmailSystemTime, now: EmailSystemTime) -> LocalizedText {
    let days = sent.legacy_day_difference(now);
    if days <= 0 {
        LocalizedText::new("ui.email.date.today", "today")
    } else {
        LocalizedText::new("ui.email.date.days", "{days} day").with_arg("days", days.to_string())
    }
}

pub(super) fn email_page_number_text(page: impl ToString) -> LocalizedText {
    LocalizedText::new("ui.email.page.number", "{page}").with_arg("page", page.to_string())
}

pub(super) fn email_detail_taros_text(amount: i32) -> LocalizedText {
    LocalizedText::new("ui.email.detail.taros", "TAROS : {amount}")
        .with_arg("amount", amount.to_string())
}

pub(super) fn email_compose_taros_text(amount: i32) -> LocalizedText {
    LocalizedText::new("ui.email.compose.taros", "{amount}").with_arg("amount", amount.to_string())
}

pub(super) fn email_postage_amount_text(amount: i32) -> LocalizedText {
    LocalizedText::new("ui.email.compose.postage.amount", "{amount}")
        .with_arg("amount", amount.to_string())
}

pub(super) fn email_calculator_value_text(amount: i64) -> LocalizedText {
    LocalizedText::new("ui.email.calculator.value", "{amount}")
        .with_arg("amount", amount.to_string())
}

pub(super) fn email_calculator_digit_text(digit: u8) -> LocalizedText {
    LocalizedText::new("ui.email.calculator.digit", "{digit}").with_arg("digit", digit.to_string())
}

#[cfg(test)]
pub(super) fn email_text(model: &EmailUiModel, role: EmailUiTextRole) -> String {
    email_fallback_text(&email_localized_text(model, role))
}
