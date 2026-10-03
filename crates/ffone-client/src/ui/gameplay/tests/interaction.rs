use super::*;

#[test]
fn chat_input_defaults_disabled_and_pure_reducer_honors_tutorial_gate() {
    let mut chat = ChatUi {
        active: true,
        input: "tutorial text".to_owned(),
        ..default()
    };
    let original = chat.clone();
    let mut history = ChatInputHistory::default();

    assert!(!chat.input_enabled);
    assert_eq!(
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter),
        Vec::<GameplayUiAction>::new()
    );
    assert_eq!(chat, original);
    assert!(history.sent.is_empty());
}

#[test]
fn nanocom_close_exits_chat_input_state_without_discarding_the_draft() {
    let mut chat = ChatUi {
        input_enabled: true,
        active: true,
        input: "unsent draft".to_owned(),
        ..default()
    };

    chat.close_nanocom_menu();

    assert!(!chat.active);
    assert_eq!(chat.input, "unsent draft");
}

#[test]
fn sent_history_keeps_twenty_and_clamps_source_cursor_boundaries() {
    let mut chat = ChatUi {
        input_enabled: true,
        ..default()
    };
    let mut history = ChatInputHistory::default();
    for index in 0..22 {
        chat.active = true;
        chat.input = format!("message {index}");
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter);
    }
    assert_eq!(history.sent.len(), CHAT_SENT_HISTORY_CAPACITY);
    assert_eq!(history.sent.front().unwrap(), "message 2");
    assert_eq!(history.sent.back().unwrap(), "message 21");
    assert_eq!(history.cursor, CHAT_SENT_HISTORY_CAPACITY);

    chat.active = true;
    for _ in 0..25 {
        reduce_chat_keyboard(
            &mut chat,
            &mut history,
            ChatKeyboardCommand::PreviousHistory,
        );
    }
    assert_eq!(history.cursor, 0);
    assert_eq!(chat.input, "message 2");

    for _ in 0..19 {
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::NextHistory);
    }
    assert_eq!(history.cursor, 19);
    assert_eq!(chat.input, "message 21");
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::NextHistory);
    assert_eq!(history.cursor, CHAT_SENT_HISTORY_CAPACITY);
    assert_eq!(
        chat.input, "message 21",
        "clean CnGuiChat does not clear at the after-history cursor"
    );
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::NextHistory);
    assert_eq!(history.cursor, CHAT_SENT_HISTORY_CAPACITY);
    assert_eq!(chat.input, "message 21");
}

#[test]
fn keyboard_system_reads_text_ignores_controls_and_has_no_movement_dependency() {
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_chat_keyboard_input);
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.visible = true;
        model.chat.visible = true;
        model.chat.input_enabled = false;
        model.chat.active = true;
        model.chat.input = "blocked tutorial text".to_owned();
    }
    app.world_mut()
        .write_message(pressed_key(KeyCode::Enter, Key::Enter, Some("\r")));
    app.update();
    assert!(!app.world().resource::<GameplayUiModel>().chat.active);
    assert!(
        app.world()
            .resource::<GameplayUiModel>()
            .chat
            .input
            .is_empty()
    );
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());

    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .input_enabled = true;
    app.world_mut()
        .write_message(pressed_key(KeyCode::Enter, Key::Enter, Some("\r")));
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::OpenNanocomMenu]
    );

    app.world_mut().write_message(pressed_key(
        KeyCode::KeyA,
        Key::Character("я😀".into()),
        Some("я\n😀\t"),
    ));
    app.update();
    assert_eq!(app.world().resource::<GameplayUiModel>().chat.input, "я😀");
    app.world_mut()
        .write_message(pressed_key(KeyCode::KeyB, Key::Character("б".into()), None));
    app.update();
    assert_eq!(app.world().resource::<GameplayUiModel>().chat.input, "я😀б");
    assert!(
        !app.world().contains_resource::<ButtonInput<KeyCode>>(),
        "chat input consumes KeyboardInput messages without movement/ButtonInput ownership"
    );
}

#[test]
fn keyboard_enter_is_ordered_before_send_button_and_cannot_double_send() {
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(
            Update,
            (
                handle_chat_keyboard_input.before(handle_gameplay_ui_buttons),
                handle_gameplay_ui_buttons,
            ),
        );
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.visible = true;
        model.chat.visible = true;
        model.chat.input_enabled = true;
        model.chat.active = true;
        model.chat.input = "send once".to_owned();
    }
    app.world_mut()
        .spawn((Interaction::Pressed, SendChatButton));
    app.world_mut()
        .write_message(pressed_key(KeyCode::Enter, Key::Enter, Some("\r")));

    app.update();

    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![
            GameplayUiAction::SendChat("send once".to_owned()),
            GameplayUiAction::CloseNanocomMenu,
        ]
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::CloseScreen]
    );
}

#[test]
fn clicking_chat_text_field_activates_input_and_accepts_text() {
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(
            Update,
            (
                handle_chat_keyboard_input.before(handle_gameplay_ui_buttons),
                handle_gameplay_ui_buttons,
            ),
        );
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.visible = true;
        model.chat.visible = true;
        model.chat.input_enabled = true;
    }
    app.world_mut()
        .spawn((Interaction::Pressed, ChatTextFieldButton));

    app.update();
    app.world_mut().write_message(pressed_key(
        KeyCode::KeyA,
        Key::Character("Привет".into()),
        Some("Привет"),
    ));
    app.update();

    assert!(app.world().resource::<GameplayUiModel>().chat.active);
    assert_eq!(
        app.world().resource::<GameplayUiModel>().chat.input,
        "Привет"
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::OpenNanocomMenu]
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::OpenScreen]
    );
}

#[test]
fn clicking_chat_text_field_honors_the_server_input_gate() {
    let mut app = App::new();
    app.init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_gameplay_ui_buttons);
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.visible = true;
        model.chat.visible = true;
        model.chat.input_enabled = false;
    }
    app.world_mut()
        .spawn((Interaction::Pressed, ChatTextFieldButton));

    app.update();

    assert!(!app.world().resource::<GameplayUiModel>().chat.active);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    assert!(app.world().resource::<GameplayUiAudioOutbox>().is_empty());
}

#[test]
fn send_button_records_and_clears_text_without_closing_chat() {
    let mut app = App::new();
    app.init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_gameplay_ui_buttons);
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.chat.input_enabled = true;
        model.chat.active = true;
        model.chat.input = "mouse send".to_owned();
    }
    app.world_mut()
        .spawn((Interaction::Pressed, SendChatButton));

    app.update();

    let model = app.world().resource::<GameplayUiModel>();
    assert!(model.chat.active);
    assert!(model.chat.input.is_empty());
    assert_eq!(
        app.world().resource::<ChatInputHistory>().sent,
        VecDeque::from(["mouse send".to_owned()])
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::SendChat("mouse send".to_owned())]
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::ButtonSound]
    );
}

#[test]
fn controller_start_uses_enter_nanocom_reducer_once_per_press() {
    use super::super::chat_input::handle_chat_keyboard_input;
    let mut app = App::new();
    let mut model = GameplayUiModel::default();
    model.visible = true;
    model.chat.visible = true;
    model.chat.input_enabled = true;
    app.insert_resource(model)
        .insert_resource(GameplayControllerMenuInput(true))
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_chat_keyboard_input);
    app.update();
    assert!(app.world().resource::<GameplayUiModel>().chat.active);
    app.update();
    assert!(app.world().resource::<GameplayUiModel>().chat.active);
    app.world_mut().resource_mut::<GameplayControllerMenuInput>().0 = true;
    app.update();
    assert!(!app.world().resource::<GameplayUiModel>().chat.active);
    assert_eq!(app.world_mut().resource_mut::<GameplayUiOutbox>().drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::OpenNanocomMenu, GameplayUiAction::CloseNanocomMenu]);
}
