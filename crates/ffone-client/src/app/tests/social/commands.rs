use super::*;

#[test]
fn nanocom_messages_reach_chat_with_clean_event_types_and_filters() {
    use ffone_client::nanocom_message_ui::{
        NANOCOM_COMPACT_TITLE, NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY, NANOCOM_INVITATION_FALLBACK,
        NANOCOM_INVITATION_LOCALIZATION_KEY,
    };

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for locale in ["en", "ru"] {
        let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
        let (localization, language) = Localization::open(&root, locale).unwrap();
        let (task_id, source) = content
            .missions()
            .filter(|mission| mission.mission_type != TutorialMissionType::Nano)
            .find_map(|mission| {
                let source = mission.start_nanocom_message.clone()?;
                content
                    .gameplay_npc(source.npc_type)
                    .is_some()
                    .then_some((mission.provenance.task_id, source))
            })
            .expect("a production mission start edge owns a type-9 NanoCom notice");
        let npc = content.gameplay_npc(source.npc_type).unwrap();
        let sender = localization.text(
            &language,
            &LocalizedText::new(
                format!("content.npc.{}.name", source.npc_type),
                npc.name.clone(),
            ),
        );
        let body = localization.text(
            &language,
            &LocalizedText::new(
                format!(
                    "content.tabledata.mission.mission_string.{}.str_name_string",
                    source.string_id
                ),
                source.text.clone(),
            ),
        );
        let expected = format!("{sender}: {body}");
        let invite = format!(
            "{}: {}",
            localization.text(
                &language,
                &LocalizedText::new(
                    NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY,
                    NANOCOM_COMPACT_TITLE
                )
            ),
            localization.text(
                &language,
                &LocalizedText::new(
                    NANOCOM_INVITATION_LOCALIZATION_KEY,
                    NANOCOM_INVITATION_FALLBACK
                )
                .with_arg("name", "Dexter")
            ),
        );
        let enqueue = |app: &mut App| {
            assert!(enqueue_mission_nanocom(
                task_id,
                MissionNanocomEdge::Start,
                &content,
                &mut app.world_mut().resource_mut::<NanocomMessageUiModel>(),
            ));
        };

        let mut app = App::new();
        app.insert_resource(State::new(ClientState::CharacterSelect))
            .init_resource::<OptionProductionRuntime>()
            .init_resource::<RuntimeStatus>()
            .init_resource::<TutorialMissionRuntime>()
            .init_resource::<NanocomMessageUiModel>()
            .insert_resource(localization)
            .insert_resource(language)
            .add_systems(Update, receive_nanocom_chat);

        // A notice queued before World is entered waits for the gameplay chat.
        enqueue(&mut app);
        app.update();
        assert!(
            app.world()
                .resource::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .is_empty()
        );
        assert!(
            app.world()
                .resource::<NanocomMessageUiModel>()
                .has_chat_echo()
        );

        app.insert_resource(State::new(ClientState::World));
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
        assert!(
            !app.world()
                .resource::<NanocomMessageUiModel>()
                .has_chat_echo()
        );

        // Idle frames neither repeat the line nor dirty either owner.
        app.world_mut().clear_trackers();
        app.update();
        assert_eq!(
            app.world()
                .resource::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .len(),
            1
        );
        assert!(!app.world().resource_ref::<RuntimeStatus>().is_changed());
        assert!(
            !app.world()
                .resource_ref::<NanocomMessageUiModel>()
                .is_changed()
        );

        // NPC/events-in-chat drops passive copy at receipt; invitations are
        // clean type 1 and are shared by ALL and GROUP.
        app.world_mut()
            .resource_mut::<OptionProductionRuntime>()
            .options
            .display
            .npc_messages_in_chat = false;
        enqueue(&mut app);
        app.world_mut()
            .resource_mut::<NanocomMessageUiModel>()
            .enqueue_buddy_invite(1, "Dexter");
        app.update();
        app.world_mut()
            .resource_mut::<OptionProductionRuntime>()
            .options
            .display
            .npc_messages_in_chat = true;
        app.update();
        let runtime = app.world().resource::<RuntimeStatus>();
        assert_eq!(runtime.chat.world_chat_lines.len(), 2);
        let line = runtime.chat.world_chat_lines.last().unwrap();
        assert_eq!(line.kind, ChatLineKind::System);
        assert_eq!(line.text, invite);
        assert_eq!(runtime.chat.world_group_chat_lines, vec![line.clone()]);
        assert!(runtime.chat.world_buddy_chat_lines.is_empty());

        app.insert_resource(State::new(ClientState::Tutorial));
        enqueue(&mut app);
        app.update();
        let tutorial = &app.world().resource::<TutorialMissionRuntime>().chat_lines;
        assert_eq!(tutorial.len(), 1);
        assert_eq!(tutorial[0].kind, ChatLineKind::Npc);
        assert_eq!(tutorial[0].text, expected);
        assert_eq!(
            app.world()
                .resource::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .len(),
            2
        );
    }
}
