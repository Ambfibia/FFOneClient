use super::*;

#[test]
fn mission_dialogue_publishes_localized_chat_once_without_balloon_or_portrait() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content =
        TutorialMissionContent::open(&crate::assets::AssetLocator::open(&root).unwrap()).unwrap();
    let dialogues = content
        .missions()
        .flat_map(|task| {
            [
                task.start_dialogue.as_ref(),
                task.success_dialogue.as_ref(),
                task.failure_dialogue.as_ref(),
            ]
        })
        .flatten()
        .collect::<Vec<_>>();
    assert!(!dialogues.is_empty());
    for locale in ["en", "ru"] {
        let (localization, language) =
            crate::localization::Localization::open(&root, locale).unwrap();
        let bundle: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("localization/{locale}.json"))).unwrap(),
        )
        .unwrap();
        for dialogue in &dialogues {
            let key = format!(
                "content.tabledata.mission.mission_string.{}.str_name_string",
                dialogue.string_id
            );
            assert!(
                bundle["entries"].get(&key).is_some(),
                "missing {locale}: {key}"
            );
            let localized = LocalizedText::new(&key, &dialogue.text);
            let resolved = localization.text(&language, &localized);
            assert!(!resolved.is_empty(), "{locale}: {key}");
        }
    }
    let dialogue = dialogues
        .iter()
        .find(|line| content.gameplay_npc(line.npc_type).is_some())
        .unwrap();
    let npc_type = dialogue.npc_type;
    let localized = LocalizedText::new(
        format!(
            "content.tabledata.mission.mission_string.{}.str_name_string",
            dialogue.string_id
        ),
        &dialogue.text,
    );
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .add_message::<NpcChatEvent>()
        .insert_resource(GameplayUiModel {
            balloon_chat_visible: false,
            ..default()
        })
        .insert_resource(content.clone())
        .insert_resource(LegacyNanoStandRandomStream::with_seed(3))
        .init_resource::<NpcBarkerBubbleRuntime>()
        .add_systems(Update, advance_npc_barker_bubbles);
    let owner = app
        .world_mut()
        .spawn((
            GlobalTransform::IDENTITY,
            NetworkNpcAppearance0104(NpcAppearance0104 {
                npc_id: 77,
                npc_type,
                hp: 100,
                condition_bit_flag: 0,
                position: [0; 3],
                angle: 0,
                barker_type: 0,
            }),
        ))
        .id();
    app.world_mut()
        .resource_mut::<NpcBarkerBubbleRuntime>()
        .request_quest_dialogue(owner, npc_type, localized.clone());
    let mut reader = app
        .world()
        .resource::<Messages<NpcChatEvent>>()
        .get_cursor();
    app.update();
    let received = reader
        .read(app.world().resource::<Messages<NpcChatEvent>>())
        .collect::<Vec<_>>();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].npc_type, npc_type);
    assert_eq!(received[0].message, localized);
    assert!(
        app.world().resource::<NpcBarkerBubbleRuntime>().actors[&owner]
            .lines
            .is_empty()
    );
    app.update();
    assert_eq!(
        reader
            .read(app.world().resource::<Messages<NpcChatEvent>>())
            .count(),
        0
    );
}

#[test]
fn npc_greeting_request_owns_the_exact_tabledata_localization_key() {
    let mut definition = normal_world_npc_definition();
    definition.greeting_string_id = 643;
    definition.greeting = "Exact greeting".to_owned();
    let mut runtime = NpcBarkerBubbleRuntime::default();
    runtime.request_greeting(Entity::from_bits(7), &definition);
    let pending = runtime.pending_greetings.front().unwrap();
    assert_eq!(
        pending.localized.key,
        "content.tabledata.npc.npc_string.643.str_comment"
    );
    assert_eq!(pending.localized.fallback, "Exact greeting");

    definition.greeting = " ".to_owned();
    runtime.request_greeting(Entity::from_bits(8), &definition);
    assert_eq!(runtime.pending_greetings.len(), 1);
    for (index, field) in ["str_name", "str_comment", "str_comment1", "str_comment2"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            localized_tabledata_npc_barker(428, index, "fallback").key,
            format!("content.tabledata.npc.npc_barker.428.{field}")
        );
    }
    assert_eq!(
        localized_tabledata_mission_barker(12, "fallback").key,
        "content.tabledata.mission.mission_string.12.str_name_string"
    );
    assert_eq!(
        localized_tabledata_npc_skill_barker(19, "fallback").key,
        "content.tabledata.skill.skill_string.19.str_comment1"
    );
}

#[test]
fn combat_target_localization_key_tracks_the_current_npc_type() {
    let numbuh_two = localized_tutorial_npc_name(2671, "Numbuh Two");
    let oil_ogre = localized_tutorial_npc_name(2676, "Oil Ogre");
    assert_eq!(numbuh_two.key, "content.npc.2671.name");
    assert_eq!(oil_ogre.key, "content.npc.2676.name");
    assert_ne!(numbuh_two.key, oil_ogre.key);

    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, english) = crate::localization::Localization::open(&asset_root, "en")
        .expect("production localization");
    let (_, russian) = crate::localization::Localization::open(&asset_root, "ru")
        .expect("production Russian localization");
    assert_eq!(localization.text(&english, &oil_ogre), "Oil Ogre");
    assert_eq!(localization.text(&russian, &oil_ogre), "Нефтяной огр");
}
