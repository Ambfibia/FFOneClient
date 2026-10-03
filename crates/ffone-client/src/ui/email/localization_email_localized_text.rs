use super::*;

pub(super) fn email_localized_text(model: &EmailUiModel, role: EmailUiTextRole) -> LocalizedText {
    match role {
        EmailUiTextRole::FolderEmpty => {
            let is_empty = match model.folder {
                EmailFolder::Guide => model.visible_guide_messages().is_empty(),
                EmailFolder::Player => model.player_messages.is_empty(),
            };
            if is_empty {
                LocalizedText::new(
                    "ui.email.folder.empty",
                    "There are no messages in this folder.",
                )
            } else {
                email_passthrough_text("")
            }
        }
        EmailUiTextRole::RowFrom(row) => match model.folder {
            EmailFolder::Guide => model
                .visible_guide_messages()
                .get(row)
                .map(EmailGuideMessage::localized_sender)
                .unwrap_or_else(|| email_passthrough_text("")),
            EmailFolder::Player => model
                .player_messages
                .get(row)
                .map(|message| email_summary_sender_text(model, message))
                .unwrap_or_else(|| email_passthrough_text("")),
        },
        EmailUiTextRole::RowSubject(row) => match model.folder {
            EmailFolder::Guide => model
                .visible_guide_messages()
                .get(row)
                .map(EmailGuideMessage::localized_subject)
                .unwrap_or_else(|| email_passthrough_text("")),
            EmailFolder::Player => model
                .player_messages
                .get(row)
                .map(|message| email_passthrough_text(message.truncated_subject()))
                .unwrap_or_else(|| email_passthrough_text("")),
        },
        EmailUiTextRole::RowDate(row) => {
            if model.folder == EmailFolder::Player {
                model
                    .player_messages
                    .get(row)
                    .map(|message| email_day_text(message.send_time, model.current_local_time))
                    .unwrap_or_else(|| email_passthrough_text(""))
            } else {
                email_passthrough_text("")
            }
        }
        EmailUiTextRole::PageSelection => model
            .selected_row
            .map(|row| {
                let page = match model.folder {
                    EmailFolder::Guide => model.guide_page + 1,
                    EmailFolder::Player => model.player_page.max(0) as usize,
                };
                email_page_selection_text(page, row + 1)
            })
            .unwrap_or_else(|| email_passthrough_text("")),
        EmailUiTextRole::PageNumber => match model.folder {
            EmailFolder::Guide if model.guide_messages.is_empty() => email_page_number_text(0),
            EmailFolder::Guide => email_page_number_text(model.guide_page + 1),
            EmailFolder::Player => email_page_number_text(model.player_page),
        },
        EmailUiTextRole::DetailFrom => match model.folder {
            EmailFolder::Guide => model
                .selected_guide()
                .map(EmailGuideMessage::localized_sender)
                .unwrap_or_else(|| email_passthrough_text("")),
            EmailFolder::Player => model
                .selected_summary()
                .map(|message| email_summary_sender_text(model, message))
                .unwrap_or_else(|| email_passthrough_text("")),
        },
        EmailUiTextRole::DetailSubject => match model.folder {
            EmailFolder::Guide => model
                .selected_guide()
                .map(EmailGuideMessage::localized_subject)
                .unwrap_or_else(|| email_passthrough_text("")),
            EmailFolder::Player => model
                .selected_summary()
                .map(|message| email_passthrough_text(message.subject.clone()))
                .unwrap_or_else(|| email_passthrough_text("")),
        },
        EmailUiTextRole::DetailReceived => model
            .selected_summary()
            .map(|message| email_day_text(message.send_time, model.current_local_time))
            .unwrap_or_else(|| email_passthrough_text("")),
        EmailUiTextRole::DetailBody => match model.folder {
            EmailFolder::Guide => model
                .selected_guide()
                .map(EmailGuideMessage::localized_content)
                .unwrap_or_else(|| {
                    LocalizedText::new(
                        "ui.email.detail.no_selection",
                        "There is no message selected.",
                    )
                }),
            EmailFolder::Player => model
                .read_message
                .as_ref()
                .map(|message| email_passthrough_text(message.content.clone()))
                .unwrap_or_else(|| {
                    LocalizedText::new(
                        "ui.email.detail.no_selection",
                        "There is no message selected.",
                    )
                }),
        },
        EmailUiTextRole::DetailTaros => email_detail_taros_text(
            model
                .read_message
                .as_ref()
                .map(|message| message.cash)
                .unwrap_or(0),
        ),
        EmailUiTextRole::ComposeTo => email_recipient_text(model),
        EmailUiTextRole::ComposeSubject => email_passthrough_text(model.draft.subject.clone()),
        EmailUiTextRole::ComposeBody => email_passthrough_text(model.draft.content.clone()),
        EmailUiTextRole::ComposeTaros => email_compose_taros_text(model.draft.cash),
        EmailUiTextRole::ComposePostage => email_postage_amount_text(model.draft.postage()),
        EmailUiTextRole::CalculatorValue => email_calculator_value_text(model.calculator_value),
    }
}
