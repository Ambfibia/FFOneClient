use super::*;

#[test]
fn npc_chat_receives_localized_speech_once_with_independent_filters_and_history_cap() {
    use ffone_client::{gameplay_ui::NpcChatEvent, localization::localized_tabledata_npc_greeting};

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for locale in ["en", "ru"] {
        let locator = AssetLocator::open(&root).unwrap();
        let content = TutorialMissionContent::open(&locator).unwrap();
        let (localization, language) = Localization::open(&root, locale).unwrap();
        let npc = content.gameplay_npc(2555).unwrap();
        let event = NpcChatEvent {
            npc_type: npc.npc_type,
            message: localized_tabledata_npc_greeting(npc.greeting_string_id, &npc.greeting),
        };
        let sender = localization.text(
            &language,
            &ffone_client::localization::localized_tabledata_npc_name(npc.npc_type, &npc.name),
        );
        let message = localization.text(&language, &event.message);
        assert!(!message.is_empty());
        let expected = format!("{sender}: {message}");
        let mut app = App::new();
        app.add_message::<NpcChatEvent>()
            .insert_resource(State::new(ClientState::World))
            .init_resource::<OptionProductionRuntime>()
            .init_resource::<RuntimeStatus>()
            .init_resource::<TutorialMissionRuntime>()
            .insert_resource(content)
            .insert_resource(localization)
            .insert_resource(language)
            .add_systems(Update, receive_npc_chat);
        app.world_mut().resource_mut::<RuntimeStatus>().player_id = Some(1);
        app.world_mut()
            .resource_mut::<OptionProductionRuntime>()
            .options
            .display
            .balloon = false;
        app.world_mut().write_message(event.clone());
        app.update();
        let runtime = app.world().resource::<RuntimeStatus>();
        assert_eq!(runtime.chat.world_chat_lines.len(), 1);
        let line = &runtime.chat.world_chat_lines[0];
        assert_eq!(line.kind, ChatLineKind::Npc);
        assert_eq!(line.text, expected);
        assert_eq!(
            app.world().resource::<Localization>().text(
                app.world().resource::<Language>(),
                line.localized.as_ref().unwrap()
            ),
            expected
        );
        assert!(runtime.chat.world_group_chat_lines.is_empty());
        assert!(runtime.chat.world_buddy_chat_lines.is_empty());
        app.update();
        assert_eq!(
            app.world()
                .resource::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .len(),
            1
        );

        app.world_mut()
            .resource_mut::<OptionProductionRuntime>()
            .options
            .display
            .npc_messages_in_chat = false;
        app.world_mut().write_message(event.clone());
        app.update();
        app.world_mut()
            .resource_mut::<OptionProductionRuntime>()
            .options
            .display
            .npc_messages_in_chat = true;
        app.update();
        assert_eq!(
            app.world()
                .resource::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .len(),
            1
        );
        for _ in 0..60 {
            app.world_mut().write_message(event.clone());
        }
        app.update();
        assert_eq!(
            app.world()
                .resource::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .len(),
            TUTORIAL_CHAT_HISTORY_LIMIT
        );

        app.insert_resource(State::new(ClientState::Tutorial));
        app.world_mut().write_message(event.clone());
        app.update();
        assert_eq!(
            app.world()
                .resource::<TutorialMissionRuntime>()
                .chat_lines
                .last()
                .unwrap()
                .text,
            expected
        );
        app.world_mut().resource_mut::<RuntimeStatus>().player_id = None;
        app.world_mut().write_message(event);
        app.update();
        assert_eq!(
            app.world()
                .resource::<TutorialMissionRuntime>()
                .chat_lines
                .len(),
            1
        );
    }
}

#[test]
fn server_message_routes_localized_system_line_to_all_and_group_only() {
    const RESPONSE: &str = "Commands available to you: help, access, population";

    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, mut language) = Localization::open(&asset_root, "en").unwrap();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_MOTD_LOGIN,
        flags: 0,
        checksum: 0,
        payload: ffone_protocol::ServerMessage0104 {
            message_type: 1,
            message: FixedUtf16::from_str(RESPONSE).unwrap(),
        }
        .encode(),
    };

    let mut runtime = RuntimeStatus::default();
    assert_eq!(
        apply_server_message_frame_0104(&frame, &mut runtime, &localization, &language,),
        Ok(true)
    );
    let expected = ChatLineUi::system(format!("Gamemaster: {RESPONSE}"));
    assert_eq!(runtime.chat.world_chat_lines, vec![expected.clone()]);
    assert_eq!(runtime.chat.world_group_chat_lines, vec![expected]);
    assert!(runtime.chat.world_buddy_chat_lines.is_empty());
    assert_eq!(runtime.chat.world_chat_alerts, [false; 3]);

    localization.select(&mut language, "ru");
    let mut russian_runtime = RuntimeStatus::default();
    assert_eq!(
        apply_server_message_frame_0104(&frame, &mut russian_runtime, &localization, &language,),
        Ok(true)
    );
    assert_eq!(
        russian_runtime.chat.world_chat_lines,
        vec![ChatLineUi::system(format!(
            "\u{0413}\u{0435}\u{0439}\u{043c}\u{043c}\u{0430}\u{0441}\u{0442}\u{0435}\u{0440}: {RESPONSE}"
        ))]
    );

    let before = russian_runtime.chat.world_chat_lines.clone();
    let malformed = DecodedFrame {
        payload: vec![0; ffone_protocol::ServerMessage0104::SIZE - 1],
        ..frame
    };
    assert!(
            apply_server_message_frame_0104(
                &malformed,
                &mut russian_runtime,
                &localization,
                &language,
            )
            .unwrap_err()
            .contains("payload must be 1026 bytes, got 1025")
        );
    assert_eq!(russian_runtime.chat.world_chat_lines, before);
}

#[test]
fn help_and_redeem_server_replies_use_actual_en_ru_bundles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for (locale, expected_help, expected_repeat) in [
        ("en", "Available commands", "/redeem: You have already redeemed this code item"),
        ("ru", "Доступные команды", "/redeem: Вы уже получили награду по этому коду"),
    ] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        let mut runtime = RuntimeStatus::default();
        for (reply, expected) in [("Available commands", expected_help),
            ("/redeem: You have already redeemed this code item", expected_repeat)] {
            let frame = DecodedFrame { packet_type: packet::P_FE2CL_PC_MOTD_LOGIN,
                flags: 0, checksum: 0,
                payload: ffone_protocol::ServerMessage0104 { message_type: 0,
                    message: FixedUtf16::from_str(reply).unwrap() }.encode() };
            assert_eq!(apply_server_message_frame_0104(&frame, &mut runtime, &localization, &language), Ok(true));
            assert!(runtime.chat.world_chat_lines.last().unwrap().text.ends_with(expected));
        }
        assert_eq!(runtime.chat.world_chat_lines, runtime.chat.world_group_chat_lines);
        assert!(runtime.chat.world_buddy_chat_lines.is_empty());
    }
}
