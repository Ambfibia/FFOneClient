use super::*;

#[test]
fn tutorial_buttercup_mission_dialogue_reaches_bubble_and_chat_in_both_locales() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content =
        TutorialMissionContent::open(&crate::assets::AssetLocator::open(&root).unwrap())
            .unwrap();
    let definition = content
        .mission(2250)
        .unwrap()
        .success_dialogue
        .clone()
        .unwrap();
    assert_eq!(definition.npc_type, 2672);
    let localized = LocalizedText::new(
        format!(
            "content.tabledata.mission.mission_string.{}.str_name_string",
            definition.string_id
        ),
        &definition.text,
    );
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(content)
        .init_resource::<GameplayUiModel>()
        .init_resource::<LegacyNanoStandRandomStream>()
        .init_resource::<NpcBarkerBubbleRuntime>()
        .add_message::<NpcChatEvent>()
        .add_systems(Update, advance_npc_barker_bubbles);
    let owner = app
        .world_mut()
        .spawn((
            GlobalTransform::IDENTITY,
            TutorialActor {
                id: 1007,
                npc_type: 2672,
                team: 1,
                hp: 100,
                max_hp: 100,
                damaged: false,
                interacting: true,
                invulnerable: false,
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<NpcBarkerBubbleRuntime>()
        .request_quest_dialogue(owner, definition.npc_type, localized);
    app.update();
    let runtime = app.world().resource::<NpcBarkerBubbleRuntime>();
    let line = &runtime.actors[&owner]
        .lines
        .front()
        .expect("tutorial mission bubble")
        .localized;
    for locale in ["en", "ru"] {
        let (localization, language) =
            crate::localization::Localization::open(&root, locale).unwrap();
        let bundle: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("localization/{locale}.json"))).unwrap(),
        )
        .unwrap();
        assert!(
            bundle["entries"].get(&line.key).is_some(),
            "missing {locale}: {}",
            line.key
        );
        assert!(!localization.text(&language, line).trim().is_empty());
    }
    let events = app.world().resource::<Messages<NpcChatEvent>>();
    let mut cursor = events.get_cursor();
    let rows = cursor.read(events).collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].npc_type, 2672);
    assert_eq!(rows[0].message, *line);
}

#[test]
fn npc_barker_uses_clean_string_chance_distance_and_lifetime_boundaries() {
    let display = GameplayUiModel::default();
    assert!(!display.npc_names_visible);
    assert!(display.balloon_chat_visible);

    assert!(!chat_string_is_visible(""));
    assert!(!chat_string_is_visible(" "));
    assert!(chat_string_is_visible("  "));
    assert!(chat_string_is_visible("Hello"));

    assert!(barker_attempt_succeeds(9, 9.999));
    assert!(!barker_attempt_succeeds(10, 9.0));
    assert!(!barker_attempt_succeeds(0, 10.0));
    assert!(!barker_attempt_succeeds(0, f32::NAN));
    assert_eq!(barker_lifetime(20.0), 7.0);
    assert_eq!(barker_lifetime(40.0), 8.0);
}

#[test]
fn speech_bubble_anchor_converts_physical_node_size_to_logical_viewport_space() {
    let target_scale_factor = 1.5;
    let viewport = Vec2::new(420.0, 260.0);

    for logical_size in [Vec2::new(100.0, 30.0), Vec2::new(300.0, 40.0)] {
        let mut computed = ComputedNode::default();
        computed.size = logical_size * target_scale_factor;
        computed.inverse_scale_factor = target_scale_factor.recip();

        let recovered_size = speech_bubble_logical_size(&computed);
        assert!((recovered_size - logical_size).length() < 0.000_1);

        let top_left = speech_bubble_top_left(viewport, recovered_size, 0.0);
        let physical_center_x = top_left.x * target_scale_factor + computed.size().x * 0.5;
        let physical_bottom_y = (top_left.y + recovered_size.y) * target_scale_factor;
        assert!((physical_center_x - viewport.x * target_scale_factor).abs() < 0.000_1);
        assert!((physical_bottom_y - viewport.y * target_scale_factor).abs() < 0.000_1);
    }

    // Lifetime is derived from the unscaled IMGUI height, not the 60
    // physical pixels produced by this target scale factor.
    assert_eq!(barker_lifetime(40.0), 8.0);
}

#[test]
fn player_freechat_bubble_rejects_source_empty_cases_and_replaces_per_actor() {
    let owner = Entity::from_bits(17);
    let mut runtime = PlayerFreeChatBubbleRuntime::default();
    runtime.request_message(owner, "");
    runtime.request_message(owner, " ");
    assert!(runtime.pending.is_empty());

    runtime.request_message(owner, "first");
    runtime.request_message(owner, "second");
    while let Some(pending) = runtime.pending.pop_front() {
        runtime.replace(pending.owner, pending.message, 12.0);
    }
    let line = runtime.actors.get(&owner).expect("one player bubble");
    assert_eq!(runtime.actors.len(), 1);
    assert_eq!(line.localized.key, "ui.content.passthrough");
    assert_eq!(line.localized.fallback, "{text}");
    assert_eq!(line.localized.args.get("text").unwrap(), "second");
    assert_eq!(line.started_at, 12.0);
    assert_eq!(line.max_lifetime, NPC_BARKER_MIN_LIFETIME_SECONDS);
}

#[test]
fn computress_2555_autonomous_barker_uses_production_tabledata_row_51() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(&asset_root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let definition = content
        .gameplay_npc(2555)
        .expect("production Computress NPC 2555");
    let barker = definition
        .barker
        .clone()
        .expect("Computress NPC 2555 must own its autonomous Barker row");

    assert_eq!(barker.string_id, 51);
    assert_eq!(
        barker.lines,
        [
            "I am Computress. Welcome to Sector V. ".to_owned(),
            "Dexlabs communication is offline.".to_owned(),
            "The time machine must be repaired.".to_owned(),
            "Once you have obtained a few Nanos, please proceed to the Cul-de-Sac.".to_owned(),
        ]
    );

    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .add_message::<NpcChatEvent>()
        .insert_resource(GameplayUiModel::default())
        .insert_resource(content)
        // Seed 3 produces the clean-order draws 7/100 (success), then
        // field index 3. This exercises the periodic path, not Greeting.
        .insert_resource(LegacyNanoStandRandomStream::with_seed(3))
        .insert_resource(NpcBarkerBubbleRuntime::default())
        .add_systems(Update, advance_npc_barker_bubbles);
    app.world_mut().spawn((
        GlobalTransform::IDENTITY,
        LegacyAvatarActionState::default(),
    ));
    let npc = app
        .world_mut()
        .spawn((
            GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 9.0)),
            NetworkNpcAppearance0104(NpcAppearance0104 {
                npc_id: 2555,
                npc_type: 2555,
                hp: 439,
                condition_bit_flag: 0,
                position: [0; 3],
                angle: 0,
                barker_type: 0,
            }),
        ))
        .id();
    app.world_mut()
        .resource_mut::<NpcBarkerBubbleRuntime>()
        .actors
        .insert(
            npc,
            NpcBarkerActorState {
                npc_type: 2555,
                f_barker_time: 0.0,
                lines: VecDeque::new(),
            },
        );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(16));
    app.update();

    let runtime = app.world().resource::<NpcBarkerBubbleRuntime>();
    let actor = runtime.actors.get(&npc).expect("live Computress state");
    let localized = &actor
        .lines
        .front()
        .expect("nearby Computress autonomous Barker attempt")
        .localized;
    assert_eq!(
        localized.key,
        "content.tabledata.npc.npc_barker.51.str_comment2"
    );
    assert_eq!(localized.fallback, barker.lines[3]);
    assert_eq!(
        app.world()
            .resource::<LegacyNanoStandRandomStream>()
            .draw_count(),
        2
    );
    assert_eq!(actor.f_barker_time, 16.0);

    let mut cursor = app
        .world()
        .resource::<Messages<NpcChatEvent>>()
        .get_cursor();
    let events = cursor
        .read(app.world().resource::<Messages<NpcChatEvent>>())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].npc_type, 2555);
    assert_eq!(events[0].message, *localized);

    // Turning off balloons must still publish click speech exactly once.
    let definition = app
        .world()
        .resource::<TutorialMissionContent>()
        .gameplay_npc(2555)
        .unwrap()
        .clone();
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .balloon_chat_visible = false;
    app.world_mut()
        .resource_mut::<NpcBarkerBubbleRuntime>()
        .request_greeting(npc, &definition);
    app.update();
    let events = cursor
        .read(app.world().resource::<Messages<NpcChatEvent>>())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].message,
        localized_tabledata_npc_greeting(definition.greeting_string_id, &definition.greeting)
    );
    assert_eq!(
        app.world().resource::<NpcBarkerBubbleRuntime>().actors[&npc]
            .lines
            .len(),
        1
    );
    app.update();
    assert_eq!(
        cursor
            .read(app.world().resource::<Messages<NpcChatEvent>>())
            .count(),
        0
    );
}

#[test]
fn every_gameplay_hud_text_entity_has_semantic_ownership() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<CircularMinimapTileMaterial>()
        .init_asset::<FusionMatterMeterMaterial>()
        .add_systems(Startup, spawn_gameplay_hud);
    app.update();

    let (bubble_layer, parent, z_index, pickable) = app
        .world_mut()
        .query_filtered::<
            (Entity, Option<&ChildOf>, &GlobalZIndex, &Pickable),
            With<NpcBarkerBubbleLayer>,
        >()
        .single(app.world())
        .expect("PrintName barker pass must own one resident UI layer");
    assert!(parent.is_none(), "barker layer must not inherit HUD hiding");
    assert_eq!(
        *z_index,
        GlobalZIndex(NPC_BARKER_GLOBAL_Z_INDEX),
        "every ordinary UI root must cover the world Barker pass"
    );
    assert!(NPC_BARKER_GLOBAL_Z_INDEX < 0);
    assert_eq!(*pickable, Pickable::IGNORE);
    let gameplay_hud = app
        .world_mut()
        .query_filtered::<Entity, With<GameplayHud>>()
        .single(app.world())
        .expect("gameplay HUD root");
    assert_ne!(bubble_layer, gameplay_hud);

    let mut texts = app.world_mut().query::<(&Text, Option<&LocalizedText>)>();
    let rows = texts.iter(app.world()).collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|(_, localized)| {
        localized.is_some_and(|localized| !localized.key.trim().is_empty())
    }));

    let mut dynamic_texts = app.world_mut().query_filtered::<&LocalizedText, Or<(
        With<NanoBatteryCounterText>,
        With<WeaponBatteryCounterText>,
        With<PlayerNameText>,
        With<PlayerLevelText>,
        With<MinimapNameText>,
        With<MinimapNameShadow>,
        With<CombatTargetInfoLevel>,
    )>>();
    let dynamic_rows = dynamic_texts.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(dynamic_rows.len(), 7);
    assert!(dynamic_rows.iter().all(|localized| {
        localized.key == "ui.content.passthrough"
            && localized.fallback == "{text}"
            && localized.args.contains_key("text")
    }));

    let mut battery_label_parents = Vec::new();
    let mut battery_texts =
        app.world_mut().query_filtered::<(
            &ChildOf,
            (&TextFont, &LineHeight),
            &UiTransform,
            &LocalizedText,
        ), Or<(With<NanoBatteryCounterText>, With<WeaponBatteryCounterText>)>>(
        );
    for (parent, font, transform, localized) in battery_texts.iter(app.world()) {
        assert_eq!(
            font.0.font_size.eval(Vec2::ZERO, 16.0),
            BATTERY_COUNTER_FONT_SIZE
        );
        assert_eq!((*font.1), LineHeight::Px(BATTERY_COUNTER_LINE_HEIGHT));
        assert_eq!(
            *transform,
            UiTransform::from_translation(Val2::px(0.0, BATTERY_COUNTER_TEXT_Y_OFFSET))
        );
        assert_eq!(localized.args.get("text").map(String::as_str), Some("0000"));
        battery_label_parents.push(parent.parent());
    }
    assert_eq!(battery_label_parents.len(), 2);
    for parent in battery_label_parents {
        let node = app
            .world()
            .get::<Node>(parent)
            .expect("battery text must remain inside its source label Rect");
        assert_eq!(node.align_items, AlignItems::Start);
        assert_eq!(node.justify_content, JustifyContent::Center);
    }

    let active_nano_name = app
        .world_mut()
        .query_filtered::<&LocalizedText, With<ActiveNanoInfoName>>()
        .single(app.world())
        .expect("active Nano HUD must own one localized name");
    assert_eq!(active_nano_name.key, "ui.gameplay.nano_info.name");
    assert_eq!(active_nano_name.fallback, "{name}");
    assert_eq!(
        active_nano_name.args.get("name").map(String::as_str),
        Some("")
    );

    let mut objective_texts = app.world_mut().query_filtered::<
        &LocalizedText,
        Or<(With<CurrentObjectiveTitle>, With<CurrentObjectiveBody>)>,
    >();
    let objective_rows = objective_texts.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(objective_rows.len(), 2);
    assert!(objective_rows.iter().any(|localized| {
        localized.key == "ui.hud.current_objective.title" && localized.args.len() == 1
    }));
    assert!(objective_rows.iter().any(|localized| {
        localized.key == "ui.hud.current_objective.body"
            && localized.fallback == "{objective}{progress}"
            && localized.args.len() == 2
    }));
}

#[test]
fn active_nano_info_binds_name_stamina_skill_and_cooldown_as_a_separate_surface() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<CircularMinimapTileMaterial>()
        .init_asset::<FusionMatterMeterMaterial>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<NanoWheelTransientUi>()
        .add_systems(Startup, spawn_gameplay_hud)
        .add_systems(Update, bind_active_nano_info);
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.visible = true;
        model.nanos[1] = NanoSlotUi {
            nano_id: Some(1),
            name: "Buttercup".to_owned(),
            skill_id: Some(1),
            style: Some(1),
            model_path: None,
            nano_icon_number: Some(34),
            skill_icon_number: Some(10),
            active_skill: true,
            stamina_fraction: 0.5,
            active: true,
        };
        app.world_mut().resource_mut::<NanoWheelTransientUi>().slots[1]
            .active_skill_cooldown_remaining = Some(1.0);
    }
    app.update();

    let root = app
        .world_mut()
        .query_filtered::<&Node, With<ActiveNanoInfoRoot>>()
        .single(app.world())
        .expect("active Nano info root");
    assert_eq!(root.display, Display::Flex);
    let stamina = app
        .world_mut()
        .query_filtered::<&Node, With<ActiveNanoInfoStaminaFill>>()
        .single(app.world())
        .expect("active Nano stamina fill");
    assert_eq!(stamina.width, px(52.0));
    let name = app
        .world_mut()
        .query_filtered::<&LocalizedText, With<ActiveNanoInfoName>>()
        .single(app.world())
        .expect("active Nano name");
    assert_eq!(name.key, "content.nano.1.name");
    assert_eq!(name.fallback, "Buttercup");

    let visible_images = app
        .world_mut()
        .query::<(&ActiveNanoInfoImage, &Node)>()
        .iter(app.world())
        .filter(|(_, node)| node.display == Display::Flex)
        .map(|(marker, _)| marker.0)
        .collect::<Vec<_>>();
    assert!(visible_images.contains(&ActiveNanoInfoLayer::SkillIcon));
    assert!(visible_images.contains(&ActiveNanoInfoLayer::CooldownBase));
    assert!(visible_images.contains(&ActiveNanoInfoLayer::CooldownRotating));
}

pub(super) fn pressed_key(key_code: KeyCode, logical_key: Key, text: Option<&str>) -> KeyboardInput {
    KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window: Entity::PLACEHOLDER,
    }
}

#[test]
fn enter_menu_uses_retrobution_half_second_easing() {
    assert_eq!(menu_slide_offset(true, 1.0), 1.0);
    assert_eq!(menu_slide_offset(true, 0.0), 0.0);
    assert_eq!(menu_slide_offset(false, 1.0), 0.0);
    assert_eq!(menu_slide_offset(false, 0.0), 1.0);

    let mut transition = GameplayMenuTransition::default();
    transition.set_open(true);
    transition.advance(0.25);
    assert_eq!(transition.remaining, 0.5);
    transition.advance(0.25);
    assert_eq!(transition.remaining, 0.0);
    assert!(transition.open);
}

#[test]
fn chat_editing_preserves_history_and_sends_the_edited_draft() {
    let mut chat = ChatUi {
        input_enabled: true,
        ..default()
    };
    let mut history = ChatInputHistory::default();
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter);
    reduce_chat_keyboard(
        &mut chat,
        &mut history,
        ChatKeyboardCommand::Text("привет😀"),
    );
    reduce_chat_keyboard(
        &mut chat,
        &mut history,
        ChatKeyboardCommand::Edit {
            key: KeyCode::ArrowLeft,
            control: false,
            shift: true,
        },
    );
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Text("!"));
    assert_eq!(chat.input, "привет!");
    let actions = reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter);
    assert_eq!(
        actions,
        vec![
            GameplayUiAction::SendChat("привет!".into()),
            GameplayUiAction::CloseNanocomMenu
        ]
    );
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter);
    reduce_chat_keyboard(
        &mut chat,
        &mut history,
        ChatKeyboardCommand::PreviousHistory,
    );
    assert_eq!(chat.edit.cursor, 7);
    reduce_chat_keyboard(
        &mut chat,
        &mut history,
        ChatKeyboardCommand::Edit {
            key: KeyCode::KeyA,
            control: true,
            shift: false,
        },
    );
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Backspace);
    assert!(chat.input.is_empty());
    assert_eq!(history.sent.front().unwrap(), "привет!");
}

#[test]
fn chat_utf16_limit_is_exact_for_cyrillic_and_surrogate_pairs() {
    let mut cyrillic = "я".repeat(CHAT_INPUT_UTF16_LIMIT);
    append_chat_text_utf16(&mut cyrillic, "б");
    assert_eq!(cyrillic.encode_utf16().count(), CHAT_INPUT_UTF16_LIMIT);
    assert!(!cyrillic.ends_with('б'));

    let mut accepts_pair = "a".repeat(CHAT_INPUT_UTF16_LIMIT - 2);
    append_chat_text_utf16(&mut accepts_pair, "😀x");
    assert_eq!(accepts_pair.encode_utf16().count(), CHAT_INPUT_UTF16_LIMIT);
    assert!(accepts_pair.ends_with('😀'));
    assert!(!accepts_pair.ends_with('x'));

    let mut rejects_partial_pair = "a".repeat(CHAT_INPUT_UTF16_LIMIT - 1);
    append_chat_text_utf16(&mut rejects_partial_pair, "😀x");
    assert_eq!(
        rejects_partial_pair.encode_utf16().count(),
        CHAT_INPUT_UTF16_LIMIT - 1
    );
    assert!(!rejects_partial_pair.contains('😀'));
    assert!(!rejects_partial_pair.ends_with('x'));

    let mut oversized = format!("{}😀x", "я".repeat(CHAT_INPUT_UTF16_LIMIT - 1));
    truncate_chat_input_utf16(&mut oversized);
    assert_eq!(oversized.encode_utf16().count(), CHAT_INPUT_UTF16_LIMIT - 1);
    assert!(!oversized.contains('😀'));
    assert!(!oversized.ends_with('x'));
}

#[test]
fn chat_backspace_removes_one_unicode_scalar() {
    let mut chat = ChatUi {
        input_enabled: true,
        active: true,
        input: "я😀".to_owned(),
        ..default()
    };
    let mut history = ChatInputHistory::default();

    assert!(
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Backspace)
            .is_empty()
    );
    assert_eq!(chat.input, "я");
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Backspace);
    assert!(chat.input.is_empty());
}

#[test]
fn chat_enter_opens_then_sends_exact_untrimmed_text_and_closes() {
    let mut chat = ChatUi {
        input_enabled: true,
        ..default()
    };
    let mut history = ChatInputHistory::default();

    assert_eq!(
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter),
        vec![GameplayUiAction::OpenNanocomMenu]
    );
    assert!(chat.active);
    assert_eq!(history.cursor, 0);

    let message = "  Привет 😀  ";
    reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Text(message));
    assert_eq!(
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter),
        vec![
            GameplayUiAction::SendChat(message.to_owned()),
            GameplayUiAction::CloseNanocomMenu,
        ]
    );
    assert!(!chat.active);
    assert!(chat.input.is_empty());
    assert_eq!(history.sent, VecDeque::from([message.to_owned()]));
    assert_eq!(history.cursor, 1);
}

#[test]
fn chat_enter_does_not_send_empty_but_preserves_whitespace_message() {
    let mut chat = ChatUi {
        input_enabled: true,
        active: true,
        ..default()
    };
    let mut history = ChatInputHistory::default();

    assert_eq!(
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter),
        vec![GameplayUiAction::CloseNanocomMenu]
    );
    assert!(history.sent.is_empty());

    chat.active = true;
    chat.input = "   ".to_owned();
    assert_eq!(
        reduce_chat_keyboard(&mut chat, &mut history, ChatKeyboardCommand::Enter),
        vec![
            GameplayUiAction::SendChat("   ".to_owned()),
            GameplayUiAction::CloseNanocomMenu,
        ]
    );
    assert_eq!(history.sent, VecDeque::from(["   ".to_owned()]));
}

#[test]
fn inactive_send_and_emote_controls_are_inert() {
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
        .spawn((Interaction::Pressed, SendChatButton));
    app.world_mut().spawn((Interaction::Pressed, EmoteButton));

    app.update();

    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    assert!(app.world().resource::<GameplayUiAudioOutbox>().is_empty());
}

#[test]
fn combat_target_matchup_reproduces_retrobution_cyclic_affinities() {
    assert_eq!(
        combat_target_matchup(0, Some(0)),
        Some(CombatTargetMatchup::Tie)
    );
    assert_eq!(
        combat_target_matchup(1, Some(0)),
        Some(CombatTargetMatchup::Win)
    );
    assert_eq!(
        combat_target_matchup(2, Some(0)),
        Some(CombatTargetMatchup::Lose)
    );
    assert_eq!(
        combat_target_matchup(0, Some(2)),
        Some(CombatTargetMatchup::Win)
    );
    assert_eq!(combat_target_matchup(-1, Some(0)), None);
    assert_eq!(combat_target_matchup(1, Some(3)), None);
    assert_eq!(combat_target_matchup(1, None), None);
    assert_eq!(
        CombatTargetMatchup::ALL.map(CombatTargetMatchup::size),
        [
            Vec2::new(20.0, 16.0),
            Vec2::new(17.0, 12.0),
            Vec2::new(15.0, 16.0),
        ]
    );
}

pub(super) fn normal_world_npc_definition() -> GameplayNpcUiDefinition {
    GameplayNpcUiDefinition {
        npc_type: 1,
        name: "Fusion Eduardo".to_owned(),
        greeting_string_id: 1,
        greeting: " ".to_owned(),
        barker: None,
        team: 2,
        npc_level: 3,
        npc_style: 0,
        attack_effect: 0,
        service_category: 0,
        service_number: Some(0),
        npc_class: 0,
        ai_type: 2,
        mesh_id: None,
        table_scale: None,
        attack_range_server_units: None,
        move_voice_owner: String::new(),
        radius_server_units: 170,
        height_server_units: 450,
        sight_range_server_units: 400,
        max_hp: 1260,
    }
}

pub(super) fn normal_world_npc_appearance(npc_type: i32, hp: i32) -> NetworkNpcAppearance0104 {
    NetworkNpcAppearance0104(NpcAppearance0104 {
        npc_id: 64,
        npc_type,
        hp,
        condition_bit_flag: 0,
        position: [0; 3],
        angle: 0,
        barker_type: 0,
    })
}

#[test]
fn normal_world_target_hud_uses_only_proven_gameplay_npc_fields() {
    let definition = normal_world_npc_definition();
    let appearance = normal_world_npc_appearance(definition.npc_type, 630);
    let target = GameplayHudNpc::network(&appearance, &definition, None).unwrap();

    assert_eq!(target.name.fallback(), "Fusion Eduardo");
    let localized = target.name.localized();
    assert_eq!(localized.key, "content.npc.1.name");
    assert!(localized.args.is_empty());
    assert_eq!(target.hp_fraction, Some(0.5));
    assert_eq!(target.height, 4.5);
    assert_eq!(target.projection_origin_y, 0.0);
    assert_eq!(target.level, Some(3));
    assert_eq!(target.affinity_style, Some(0));
    assert_eq!(target.portrait_icon_path, None);
    assert!(
        target
            .projected_world_position(Vec3::new(2.0, 10.0, 4.0), 0.5)
            .abs_diff_eq(Vec3::new(2.0, 12.25, 4.0), 0.000_01)
    );
}
