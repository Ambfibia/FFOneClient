use super::*;

pub(super) fn opened_player_model() -> (
    EmailUiModel,
    EmailUiOutbox,
    EmailTransportOutbox,
    EmailUiAudioOutbox,
) {
    let mut model = EmailUiModel::default();
    let mut actions = EmailUiOutbox::default();
    open_email_ui(&mut model, &mut actions, vec![], 1_000);
    model.folder = EmailFolder::Player;
    model.player_messages = vec![summary(7, "Hello")];
    model.selected_row = Some(0);
    model.read_message = Some(EmailReadMessage {
        email_index: 7,
        content: "Body".to_owned(),
        items: [
            EmailWireItem {
                item_type: 7,
                item_id: 100,
                option: 1,
                time_limit: 0,
            },
            EmailWireItem::default(),
            EmailWireItem::default(),
            EmailWireItem::default(),
        ],
        cash: 250,
    });
    (
        model,
        actions,
        EmailTransportOutbox::default(),
        EmailUiAudioOutbox::default(),
    )
}
