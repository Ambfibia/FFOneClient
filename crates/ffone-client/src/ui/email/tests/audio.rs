use super::*;

#[test]
fn open_reproduces_cursor_sound_refresh_and_inventory_mode_order() {
    let mut model = EmailUiModel::default();
    let mut actions = EmailUiOutbox::default();
    open_email_ui(
        &mut model,
        &mut actions,
        vec![EmailGuideMessage::default()],
        500,
    );
    assert!(model.visible);
    assert_eq!(model.folder, EmailFolder::Guide);
    assert_eq!(model.selected_row, Some(0));
    assert_eq!(
        actions.0.into_iter().collect::<Vec<_>>(),
        vec![
            EmailUiAction::StartUiModeSound,
            EmailUiAction::SetCursorLocked(false),
            EmailUiAction::RefreshGuideEmail {
                event_group: 15,
                event_function: 7,
            },
            EmailUiAction::SetInventoryMailMode {
                event_group: 11,
                event_function: 0,
                value: 4,
            },
        ]
    );
}

#[test]
fn read_success_is_authoritative_and_plays_arrival_sound() {
    let (mut model, mut actions, mut transport, mut audio) = opened_player_model();
    model.send_in_flight = true;
    let read = EmailReadMessage {
        email_index: 7,
        content: "Authoritative server body".to_owned(),
        items: [EmailWireItem::default(); 4],
        cash: 99,
    };
    apply_email_reply(
        EmailReply::ReadSuccess(read.clone()),
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio,
    );
    assert_eq!(model.read_message, Some(read));
    assert!(!model.send_in_flight);
    assert_eq!(audio.pop(), Some(EmailUiAudioCue::EmailArrived));
}
