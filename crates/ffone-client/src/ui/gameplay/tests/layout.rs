use super::*;

#[test]
fn current_objective_localizes_then_preserves_clean_progress_composition() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, mut language) =
        Localization::open(&asset_root, "en").expect("open production localization");
    let objective = CurrentObjectiveUi {
        visible: true,
        task_id: Some(2248),
        title: "Transmitter Critters".to_owned(),
        body: "Defeat the Oil Ogre.".to_owned(),
        remaining_time_seconds: Some(42),
        enemies: vec![
            CurrentObjectiveProgressUi {
                content_id: 2676,
                name: "Oil Ogre".to_owned(),
                complete: 2,
                needed: 5,
            },
            CurrentObjectiveProgressUi {
                content_id: 2675,
                name: "Cyberus".to_owned(),
                complete: 1,
                needed: 1,
            },
        ],
        quest_items: vec![CurrentObjectiveProgressUi {
            content_id: 10,
            name: "Blossom's ribbon".to_owned(),
            complete: 3,
            needed: 4,
        }],
    };

    let english = localized_current_objective_body(&objective, &localization, &language);
    assert_eq!(
        localization.text(&language, &english),
        "Defeat the Oil Ogre.\nRemaining time: 42\nOil Ogre : 2/5\nCyberus : 1/1\n\nBlossom's ribbon : 3/4\n"
    );

    localization.select(&mut language, "ru");
    let russian = localized_current_objective_body(&objective, &localization, &language);
    assert_eq!(
        localization.text(&language, &russian),
        "Победите нефтяного огра.\nОставшееся время: 42\nНефтяной огр : 2/5\nСайберус : 1/1\n\nЛента Цветика : 3/4\n"
    );
}

#[test]
fn chat_control_a_uses_physical_key_in_russian_layout_without_inserting_text() {
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<ChatInputHistory>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, handle_chat_keyboard_input);
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.visible = true;
        model.chat.input_enabled = true;
        model.chat.active = true;
        model.chat.input = "текст".into();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ControlRight);
    app.world_mut().write_message(pressed_key(
        KeyCode::KeyA,
        Key::Character("ф".into()),
        Some("ф"),
    ));
    app.update();
    let chat = &app.world().resource::<GameplayUiModel>().chat;
    assert_eq!(chat.input, "текст");
    assert_eq!(chat.edit.range(), 0..5);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ControlRight);
    app.world_mut().write_message(pressed_key(
        KeyCode::KeyN,
        Key::Character("новый".into()),
        Some("новый"),
    ));
    app.update();
    assert_eq!(
        app.world().resource::<GameplayUiModel>().chat.input,
        "новый"
    );
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
}

#[test]
fn exact_retrobution_rectangles_are_stable() {
    assert_eq!(
        PLAYER_FRAME_RECT,
        GameplayUiRect::new(0.0, 6.0, 255.0, 66.0)
    );
    assert_eq!(PLAYER_HEALTH_RECT.width, 186.0);
    assert_eq!(
        PLAYER_COMBAT_TOGGLE_RECT,
        GameplayUiRect::new(260.0, 6.0, 29.0, 29.0)
    );
    assert_eq!(COMBAT_TARGET_MOB_WIDTH, 290.0);
    assert_eq!(COMBAT_TARGET_NPC_WIDTH, 255.0);
    assert_eq!(COMBAT_TARGET_FRAME_HEIGHT, 66.0);
    assert_eq!(COMBAT_TARGET_MOB_GROUP_HEIGHT, 76.0);
    assert_eq!(
        COMBAT_TARGET_AFFINITY_EFFECT_RECT,
        GameplayUiRect::new(6.0, -13.0, 58.0, 72.0)
    );
    assert_eq!(
        COMBAT_TARGET_AFFINITY_ICON_RECT,
        GameplayUiRect::new(25.0, 55.0, 21.0, 21.0)
    );
    assert_eq!(
        COMBAT_TARGET_SKILL_RECT,
        GameplayUiRect::new(260.0, 7.0, 26.0, 26.0)
    );
    assert_eq!(
        COMBAT_DANGER_RECT,
        GameplayUiRect::new(0.0, 0.0, 302.0, 90.0)
    );
    assert_eq!(
        MINIMAP_GROUP_RECT,
        GameplayUiRect::new(1093.0, -4.0, 187.0, 183.0)
    );
    assert_eq!(
        MINIMAP_MAP_RECT,
        GameplayUiRect::new(26.0, 17.0, 148.0, 148.0)
    );
    assert_eq!(
        MINIMAP_NAME_RECT,
        GameplayUiRect::new(42.0, 121.0, 120.0, 60.0)
    );
    assert_eq!(
        MINIMAP_FUSION_METER_RECT,
        GameplayUiRect::new(12.0, 3.0, 176.0, 176.0)
    );
    assert_eq!(
        CURRENT_OBJECTIVE_RECT,
        GameplayUiRect::new(29.0, 186.0, 148.0, 264.0)
    );
    assert_eq!(
        TUTORIAL_TEXT_RECT,
        GameplayUiRect::new(0.0, 180.0, 1280.0, 60.0)
    );
    let active_chat = chat_layout(Vec2::new(CHAT_DEFAULT_WIDTH, CHAT_DEFAULT_HEIGHT), true);
    let inactive_chat = chat_layout(active_chat.size, false);
    assert_eq!(active_chat.size, Vec2::new(440.0, 150.0));
    assert_eq!(
        active_chat.background,
        GameplayUiRect::new(0.0, 14.0, 440.0, 113.0)
    );
    assert_eq!(CHAT_JEFFE_14_FONT_SIZE, 14.0);
    assert_eq!(CHAT_JEFFE_14_LINE_HEIGHT, 13.71);
    assert_eq!(CHAT_JEFFE_14_VERTICAL_SCALE, 0.7);
    assert_eq!(CHAT_INPUT_FONT_SIZE, 11.0);
    assert_eq!(CHAT_CHALET_SMALL_LINE_HEIGHT, 12.072);
    assert_eq!(CHAT_CHALET_SMALL_Y_OFFSET, 3.0);
    assert_eq!(NPC_BARKER_JEFFE_FONT_SIZE, 14.0);
    assert_eq!(NPC_BARKER_JEFFE_LINE_HEIGHT, 13.71);
    assert_eq!(NPC_BARKER_JEFFE_VERTICAL_SCALE, 0.7);
    assert_eq!(
        active_chat.tabs,
        [
            GameplayUiRect::new(-40.0, 1.0, 135.0, 14.0),
            GameplayUiRect::new(230.0, 1.0, 135.0, 14.0),
            GameplayUiRect::new(95.0, 1.0, 135.0, 14.0),
        ]
    );
    assert_eq!(
        active_chat.entry,
        GameplayUiRect::new(-8.0, 117.0, 462.0, 40.0)
    );
    assert_eq!(
        active_chat.log,
        GameplayUiRect::new(24.0, 18.0, 416.0, 97.0)
    );
    assert_eq!(
        active_chat.empty_state,
        GameplayUiRect::new(27.5, 33.5, 385.0, 60.0)
    );
    assert_eq!(
        [
            active_chat.input,
            active_chat.send,
            active_chat.menu,
            active_chat.emote,
            active_chat.resize,
        ],
        [
            GameplayUiRect::new(75.0, 124.0, 295.0, 23.0),
            GameplayUiRect::new(365.0, 124.0, 69.0, 23.0),
            GameplayUiRect::new(6.0, 124.0, 31.0, 19.0),
            GameplayUiRect::new(41.0, 124.0, 31.0, 19.0),
            GameplayUiRect::new(411.0, 18.0, 25.0, 25.0),
        ]
    );
    assert_eq!(
        inactive_chat.input,
        GameplayUiRect::new(40.0, 124.0, 385.0, 23.0)
    );
    let compact_chat = chat_layout(Vec2::new(CHAT_MIN_WIDTH, CHAT_MIN_HEIGHT), true);
    assert_eq!(compact_chat.tabs[ChatChannel::Group.index()].x, 182.0);
    assert_eq!(compact_chat.tabs[ChatChannel::Buddy.index()].x, 71.0);
    assert_eq!(compact_chat.resize.x, 296.0);
    assert_eq!(compact_chat.log.height, 82.0);
    assert_eq!(
        clamped_chat_size(Vec2::new(f32::INFINITY, -100.0)),
        Vec2::new(CHAT_DEFAULT_WIDTH, CHAT_MIN_HEIGHT)
    );
    assert_eq!(CHAT_INPUT_PADDING, 5.0);
    assert_eq!(CHAT_INPUT_FONT_SIZE, 11.0);
    assert_eq!(CHAT_CHALET_SMALL_LINE_HEIGHT, 12.072);
    assert_eq!(
        ACTIVE_NANO_INFO_RECT,
        GameplayUiRect::new(101.0, 42.0, 154.0, 29.0)
    );
    assert_eq!(
        ACTIVE_NANO_NAME_RECT,
        GameplayUiRect::new(20.0, 0.0, 100.0, 20.0)
    );
    assert_eq!(
        ACTIVE_NANO_SKILL_RECT,
        GameplayUiRect::new(125.0, 2.0, 26.0, 26.0)
    );
    assert_eq!(
        ACTIVE_NANO_COOLDOWN_RECT,
        GameplayUiRect::new(124.0, 2.0, 26.0, 26.0)
    );
    assert_eq!(
        ACTIVE_NANO_STAMINA_RECT,
        GameplayUiRect::new(18.0, 19.0, 104.0, 4.0)
    );
    assert_eq!(
        NANO_EMPTY_SLOTS_RECT,
        GameplayUiRect::new(1037.0, 673.0, 223.0, 37.0)
    );
    assert_eq!(
        NANO_SLOT_KEY_RECTS,
        [
            GameplayUiRect::new(1055.0, 684.0, 36.0, 36.0),
            GameplayUiRect::new(1133.0, 684.0, 36.0, 36.0),
            GameplayUiRect::new(1209.0, 684.0, 36.0, 36.0),
        ]
    );
    assert_eq!(
        NANO_AFFINITY_BACK_RECTS,
        [
            GameplayUiRect::new(7.0, 16.0, 56.0, 72.0),
            GameplayUiRect::new(83.0, 16.0, 56.0, 72.0),
            GameplayUiRect::new(160.0, 16.0, 56.0, 72.0),
        ]
    );
    assert_eq!(
        NANO_GUMBALL_RECTS,
        [
            GameplayUiRect::new(-3.0, -27.0, 71.0, 104.0),
            GameplayUiRect::new(74.0, -27.0, 71.0, 104.0),
            GameplayUiRect::new(151.0, -27.0, 71.0, 104.0),
        ]
    );
    assert_eq!(
        NANO_PORTRAIT_RECTS,
        [
            GameplayUiRect::new(8.0, 20.0, 64.0, 64.0),
            GameplayUiRect::new(85.0, 20.0, 64.0, 64.0),
            GameplayUiRect::new(162.0, 20.0, 64.0, 64.0),
        ]
    );
    assert_eq!(
        NANO_ANIMATED_PORTRAIT_RECTS,
        [
            GameplayUiRect::new(0.0, -24.0, 72.0, 108.0),
            GameplayUiRect::new(77.0, -24.0, 72.0, 108.0),
            GameplayUiRect::new(154.0, -24.0, 72.0, 108.0),
        ]
    );
    assert_eq!(
        NANO_SKILL_RECTS,
        [
            GameplayUiRect::new(-5.0, 5.0, 26.0, 26.0),
            GameplayUiRect::new(72.0, 5.0, 26.0, 26.0),
            GameplayUiRect::new(149.0, 5.0, 26.0, 26.0),
        ]
    );
    assert_eq!(
        NANO_COOLDOWN_RECTS,
        [
            GameplayUiRect::new(-6.0, 5.0, 26.0, 26.0),
            GameplayUiRect::new(71.0, 5.0, 26.0, 26.0),
            GameplayUiRect::new(148.0, 5.0, 26.0, 26.0),
        ]
    );
    assert_eq!(
        NANO_STAMINA_FILL_RECTS[0],
        GameplayUiRect::new(23.0, 18.0, 40.0, 4.0)
    );
    assert_eq!(
        NANO_BATTERY_COUNTER_RECT,
        GameplayUiRect::new(-81.0, 26.0, 76.0, 29.0)
    );
    assert_eq!(
        WEAPON_BATTERY_COUNTER_RECT,
        GameplayUiRect::new(-81.0, 61.0, 76.0, 29.0)
    );
    assert_eq!(
        skill_icon_asset_path(10),
        "ui/en/gameplay/nano/icons/skill/skillicon_10.png"
    );
    assert_eq!(TUTORIAL_MOUSE_RECT.y, 720.0 - 150.0 - 200.0);
    assert_eq!(TUTORIAL_RIGHT_RECT.y, 720.0 - 70.0 - 35.0 - 200.0);
}

#[test]
fn potion_and_boost_counter_text_preserves_primary_digits_and_height_adapter() {
    assert_eq!(battery_counter_text(0), "0000");
    assert_eq!(battery_counter_text(1), "0001");
    assert_eq!(battery_counter_text(16), "0016");
    assert_eq!(battery_counter_text(600), "0600");
    assert_eq!(battery_counter_text(9999), "9999");
    assert_eq!(battery_counter_text(10_000), "10000");
    assert_eq!(BATTERY_COUNTER_FONT_SIZE, 9.0);
    assert_eq!(BATTERY_COUNTER_LINE_HEIGHT, 13.71);
    assert_eq!(BATTERY_COUNTER_TEXT_Y_OFFSET, 1.0);
}

#[test]
fn potion_and_boost_labels_stay_inside_their_counter_art_at_ui_scales() {
    for (art, label) in [
        (NANO_BATTERY_COUNTER_RECT, NANO_BATTERY_LABEL_RECT),
        (WEAPON_BATTERY_COUNTER_RECT, WEAPON_BATTERY_LABEL_RECT),
    ] {
        assert_eq!(label, art);
        for viewport_height in [681.0, 720.0, 1080.0] {
            let scale = gameplay_ui_scale(viewport_height);
            let four_digit_width = 24.0 * scale; // jeffe.otf at the retained 9 px size.
            let digit_left = (label.x + label.width * 0.5) * scale - four_digit_width * 0.5;
            let digit_right = digit_left + four_digit_width;
            let digit_top = (label.y + BATTERY_COUNTER_TEXT_Y_OFFSET) * scale;
            assert!(digit_left > (art.x + 23.0) * scale); // icon's rightmost bright pixel
            assert!(digit_right <= (art.x + art.width) * scale);
            assert!(digit_top + BATTERY_COUNTER_LINE_HEIGHT * scale
                <= (art.y + art.height) * scale);
        }
    }
    assert!(
        NANO_BATTERY_LABEL_RECT.y + NANO_BATTERY_LABEL_RECT.height < WEAPON_BATTERY_COUNTER_RECT.y
    );
}

#[test]
fn tutorial_cues_keep_1264_by_681_layout_and_animate_all_directions() {
    const HEIGHT: f32 = 681.0;
    const BASE_PHASE: f32 = 0.125;
    assert_eq!(
        TutorialIllustrationCue::mouse().reference_rect(HEIGHT),
        GameplayUiRect::new(80.0, 331.0, 116.0, 154.0)
    );
    for (direction, expected) in [
        (
            TutorialArrowDirection::Left,
            GameplayUiRect::new(0.0, 376.0, 90.0, 69.0),
        ),
        (
            TutorialArrowDirection::Right,
            GameplayUiRect::new(186.0, 376.0, 88.0, 69.0),
        ),
        (
            TutorialArrowDirection::Up,
            GameplayUiRect::new(107.0, 261.0, 69.0, 89.0),
        ),
        (
            TutorialArrowDirection::Down,
            GameplayUiRect::new(107.0, 481.0, 69.0, 89.0),
        ),
    ] {
        assert_eq!(
            TutorialArrowCue::mouse(direction).reference_rect(HEIGHT, BASE_PHASE),
            expected
        );
    }

    let at = |direction| TutorialArrowCue::at(direction, 100.0, 200.0);
    assert_eq!(
        at(TutorialArrowDirection::Left).reference_rect(HEIGHT, 0.0),
        GameplayUiRect::new(110.0, 200.0, 90.0, 69.0)
    );
    assert_eq!(
        at(TutorialArrowDirection::Right).reference_rect(HEIGHT, 0.0),
        GameplayUiRect::new(90.0, 200.0, 88.0, 69.0)
    );
    assert_eq!(
        at(TutorialArrowDirection::Up).reference_rect(HEIGHT, 0.0),
        GameplayUiRect::new(100.0, 210.0, 69.0, 89.0)
    );
    assert_eq!(
        at(TutorialArrowDirection::Down).reference_rect(HEIGHT, 0.0),
        GameplayUiRect::new(100.0, 190.0, 69.0, 89.0)
    );
}

#[test]
fn tutorial_cues_scale_the_animated_rect_around_each_legacy_pivot() {
    fn approximate(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.001,
            "{actual} differs from {expected}"
        );
    }

    let viewport = Vec2::new(1920.0, 1080.0);
    let scale = gameplay_ui_scale(viewport.y);
    approximate(scale, 1.476_562_5);

    let mouse = TutorialIllustrationCue::mouse().render_rect(viewport, scale);
    for (actual, expected) in [
        (mouse.x, 118.125),
        (mouse.y, 563.203_1),
        (mouse.width, 171.281_25),
        (mouse.height, 227.390_63),
    ] {
        approximate(actual, expected);
    }

    let top_right =
        TutorialArrowCue::at(TutorialArrowDirection::Right, viewport.x - 280.0, 50.0)
            .with_scale_pivot(TutorialCueScalePivot::TopRight)
            .render_rect(viewport, scale, 0.0);
    for (actual, expected) in [
        (top_right.x, 1491.796_9),
        (top_right.y, 73.828_125),
        (top_right.width, 129.937_5),
        (top_right.height, 101.882_81),
    ] {
        approximate(actual, expected);
    }

    let menu_pivot = TutorialCueScalePivot::Point(Vec2::new(1440.0, 540.0));
    let menu = TutorialArrowCue::at(TutorialArrowDirection::Right, 1194.0, 499.0)
        .with_scale_pivot(menu_pivot)
        .render_rect(viewport, scale, 0.0);
    approximate(menu.x, 1062.0);
    approximate(menu.y, 479.460_94);
}

#[test]
fn tutorial_text_uses_live_width_and_the_approved_replacement_font() {
    let layout = tutorial_instruction_layout(TUTORIAL_TEXT_RECT.y);
    assert_eq!(layout.width, percent(100));
    assert_eq!(layout.height, px(60));
    assert_eq!(layout.align_items, AlignItems::Center);
    assert_eq!(layout.justify_content, JustifyContent::Center);
    assert_eq!(TUTORIAL_INSTRUCTION_FONT_SIZE, 16.0);
    assert!(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/game")
            .join(TUTORIAL_INSTRUCTION_FONT_PATH)
            .is_file()
    );
}

#[test]
fn test_ser_world_position_maps_to_legacy_tile_16_crop() {
    // OpenFusion server coordinates [632032, 187177, -5500] are centi-units.
    let samples = minimap_tiles(632032.0 / 100.0, 187177.0 / 100.0, 8.0);
    assert_eq!(samples.len(), 1);
    let sample = samples[0];
    assert_eq!(sample.tile_number, 16);
    assert!((sample.source.x - 12.08).abs() < 0.2, "{sample:?}");
    assert!((sample.source.y - 12.06).abs() < 0.2, "{sample:?}");
    assert!((sample.source.width - 64.0).abs() < 0.01, "{sample:?}");
    assert!((sample.source.height - 64.0).abs() < 0.01, "{sample:?}");
    assert_eq!(
        sample.destination,
        GameplayUiRect::new(0.0, 0.0, 148.0, 148.0)
    );
}

#[test]
fn minimap_ratio_four_is_the_exact_zoomed_in_half_width() {
    let ratio_eight = minimap_tiles(6320.32, 1871.77, 8.0);
    let ratio_four = minimap_tiles(6320.32, 1871.77, 4.0);
    assert_eq!(ratio_eight.len(), 1);
    assert_eq!(ratio_four.len(), 1);
    assert!((ratio_eight[0].source.width - 64.0).abs() < 0.01);
    assert!((ratio_four[0].source.width - 32.0).abs() < 0.01);
    assert!((ratio_four[0].source.height - 32.0).abs() < 0.01);
    assert_eq!(
        ratio_four[0].destination,
        GameplayUiRect::new(0.0, 0.0, 148.0, 148.0)
    );
}

#[test]
fn waypoint_center_and_exact_height_boundaries_match_legacy() {
    let player = Vec3::new(4096.0, 10.0, 4096.0);
    let centered = minimap_waypoint(player, player, 8.0, false, 0.0).unwrap();
    assert_eq!(centered.icon, MinimapWaypointIcon::InRange);
    assert!((centered.left - 65.5).abs() <= f32::EPSILON);
    assert!((centered.top - 65.5).abs() <= f32::EPSILON);
    assert_eq!(centered.rotation_degrees, 0.0);
    assert_eq!(centered.alpha, 1.0);

    for y in [5.0, -5.0] {
        let sample = minimap_waypoint(
            player,
            Vec3::new(player.x, player.y + y, player.z),
            8.0,
            false,
            0.0,
        )
        .unwrap();
        assert_eq!(sample.icon, MinimapWaypointIcon::InRange);
    }
    assert_eq!(
        minimap_waypoint(
            player,
            Vec3::new(player.x, player.y + 5.01, player.z),
            8.0,
            false,
            0.0,
        )
        .unwrap()
        .icon,
        MinimapWaypointIcon::Above
    );
    assert_eq!(
        minimap_waypoint(
            player,
            Vec3::new(player.x, player.y - 5.01, player.z),
            8.0,
            false,
            0.0,
        )
        .unwrap()
        .icon,
        MinimapWaypointIcon::Below
    );
}

#[test]
fn waypoint_rejects_non_finite_or_non_positive_scale() {
    assert!(minimap_waypoint(Vec3::ZERO, Vec3::ZERO, 0.0, false, 0.0).is_none());
    assert!(
        minimap_waypoint(Vec3::ZERO, Vec3::new(f32::NAN, 0.0, 0.0), 8.0, false, 0.0,).is_none()
    );
}

#[test]
fn gameplay_ui_scale_matches_ffguiutility_default_formula() {
    assert_eq!(gameplay_ui_scale(681.0), 1.0);
    assert_eq!(gameplay_ui_scale(720.0), 1.0);
    assert!((gameplay_ui_scale(768.0) - 1.05).abs() < f32::EPSILON);
    assert!((gameplay_ui_scale(1008.0) - 1.378_125).abs() < 0.000_001);
    assert_eq!(gameplay_ui_scale(0.0), 1.0);
    assert_eq!(gameplay_ui_scale(f32::NAN), 1.0);
}
