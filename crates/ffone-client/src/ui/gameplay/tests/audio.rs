use super::*;

#[test]
fn empty_send_button_keeps_clean_click_audio_without_chat_action() {
    let mut app = App::new();
    app.init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_gameplay_ui_buttons);
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .input_enabled = true;
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .active = true;
    app.world_mut()
        .spawn((Interaction::Pressed, SendChatButton));

    app.update();

    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::ButtonSound]
    );
}

#[test]
fn chat_audio_tabs_and_slide_buttons_emit_only_clean_source_cues() {
    let mut app = App::new();
    app.init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_gameplay_ui_buttons);
    app.world_mut()
        .spawn((Interaction::Pressed, ChatTabButton(ChatChannel::Buddy)));
    app.world_mut()
        .spawn((Interaction::Pressed, MenuChatButton));

    app.update();

    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![
            GameplayUiAudioCue::TabClick01,
            GameplayUiAudioCue::ButtonSound,
            GameplayUiAudioCue::ClickWindowSlideOut,
        ]
    );
}
