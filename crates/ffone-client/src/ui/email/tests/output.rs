use super::*;

#[test]
fn dynamic_server_player_and_numeric_copy_uses_templates_and_arguments() {
    let mut model = EmailUiModel {
        visible: true,
        folder: EmailFolder::Player,
        player_messages: vec![summary(7, "Server-authored subject")],
        selected_row: Some(0),
        current_local_time: EmailSystemTime {
            year: 2026,
            month: 7,
            day: 8,
            ..default()
        },
        ..default()
    };
    model.player_messages[0].send_time.day = 7;
    model.buddies.push(EmailBuddy {
        pc_uid: model.player_messages[0].from_pc_uid,
        name_check_flag: 0,
        ..default()
    });
    model.read_message = Some(EmailReadMessage {
        email_index: 7,
        content: "Server-authored body".to_owned(),
        cash: 250,
        ..default()
    });
    model.draft.subject = "Player-authored subject".to_owned();
    model.draft.content = "Player-authored body".to_owned();
    model.draft.cash = 75;

    let sender = email_localized_text(&model, EmailUiTextRole::RowFrom(0));
    assert_eq!(sender.key, "ui.email.player_label");
    assert_eq!(sender.fallback, "Player {pc_uid}");
    assert_eq!(
        sender.args["pc_uid"],
        model.player_messages[0].from_pc_uid.to_string()
    );

    let subject = email_localized_text(&model, EmailUiTextRole::RowSubject(0));
    assert_eq!(subject.key, "ui.content.passthrough");
    assert_eq!(subject.fallback, "{text}");
    assert_eq!(subject.args["text"], "Server-authored subject");

    let body = email_localized_text(&model, EmailUiTextRole::DetailBody);
    assert_eq!(body.key, "ui.content.passthrough");
    assert_eq!(body.args["text"], "Server-authored body");

    let date = email_localized_text(&model, EmailUiTextRole::RowDate(0));
    assert_eq!(date.key, "ui.email.date.days");
    assert_eq!(date.fallback, "{days} day");
    assert_eq!(date.args["days"], "1");

    let selection = email_localized_text(&model, EmailUiTextRole::PageSelection);
    assert_eq!(selection.key, "ui.email.page.selection");
    assert_eq!(selection.args["page"], "1");
    assert_eq!(selection.args["row"], "1");

    let taros = email_localized_text(&model, EmailUiTextRole::DetailTaros);
    assert_eq!(taros.fallback, "TAROS : {amount}");
    assert_eq!(taros.args["amount"], "250");
    assert_eq!(email_fallback_text(&taros), "TAROS : 250");

    for role in [
        EmailUiTextRole::ComposeSubject,
        EmailUiTextRole::ComposeBody,
    ] {
        let localized = email_localized_text(&model, role);
        assert_eq!(localized.key, "ui.content.passthrough");
        assert_eq!(localized.fallback, "{text}");
    }
    let postage = email_localized_text(&model, EmailUiTextRole::ComposePostage);
    assert_eq!(postage.key, "ui.email.compose.postage.amount");
    assert_eq!(postage.fallback, "{amount}");
    assert_eq!(postage.args["amount"], "50");
}
