use super::*;

#[test]
fn tutorial_warp_buttons_show_the_action_instead_of_the_destination() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for (locale, expected) in [("en", "WARP"), ("ru", "ПЕРЕМЕСТИТЬСЯ")] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        for npc_type in 2694..=2696 {
            for label in [
                "Infected zone",
                "Fusion Buttercup's Lair",
                "Tech Square",
                "",
            ] {
                let row = NpcUtilityRow {
                    label,
                    warp_npc_type: Some(npc_type),
                    service: None,
                };
                assert_eq!(
                    localization.text(&language, &row.localized_text()),
                    expected
                );
            }
        }
    }
}

#[test]
fn accept_button_locales_share_the_native_replacement_jeffe_font() {
    const REPLACEMENT_JEFFE_SHA256: &str =
        "F8D41844AD2092D9998E51B8CBEF5B65B3CE6DB276C93949ECECAE227674C3E1";

    let root = asset_root();
    let (localization, mut language) = Localization::open(&root, "en").unwrap();
    let label = mission_ui_localized_text("ACCEPT MISSION");
    assert_eq!(localization.text(&language, &label), "ACCEPT MISSION");
    localization.select(&mut language, "ru");
    assert_eq!(localization.text(&language, &label), "ПРИНЯТЬ МИССИЮ");

    let accept_style = mission_gui_effective_font("FusionFallMissionSkin", "acceptbut")
        .expect("acceptbut must retain its recovered source font role");
    assert_eq!(accept_style.path_id, JEFFE_14_SOURCE_FONT_PATH_ID);
    assert_eq!(OPTION_JEFFE_FONT_PATH, "fonts/jeffe.otf");

    let replacement_font = include_bytes!("../../../../../../assets/game/fonts/jeffe.otf");
    assert_eq!(
        format!("{:X}", Sha256::digest(replacement_font)),
        REPLACEMENT_JEFFE_SHA256,
        "mission UI must use the approved Cyrillic replacement, never the original Unity JEFFE Font"
    );
}

#[test]
fn nanocom_blocks_avatar_controls_but_keeps_its_chat_input_available() {
    let mut model = MissionUiModel {
        enabled: true,
        ..default()
    };
    let mut outbox = GameplayUiOutbox::default();

    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.gameplay_input_blocked());
    assert!(!model.chat_input_blocked());

    assert!(model.close_nanocom_menu(&mut outbox));
    model.show_npc_interaction(npc_with_available());
    assert!(model.gameplay_input_blocked());
    assert!(model.chat_input_blocked());
}

#[test]
fn talk_click_cannot_press_a_newly_revealed_npc_mission_row() {
    let mut model = MissionUiModel::default();
    model.show_npc_interaction(npc_with_available());
    let mut app = App::new();
    app.insert_resource(TutorialNativeMechanics::default())
        .insert_resource(model)
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_mission_ui_buttons);
    let row = app
        .world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::NpcMissionRow(0)))
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);

    app.update();

    let model = app.world().resource::<MissionUiModel>();
    assert!(model.npc_icon_mode_visible);
    assert!(matches!(model.journal, MissionJournalUi::Hidden));
    assert!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .next()
            .is_none()
    );

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.world_mut().entity_mut(row).insert(Interaction::None);
    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.world_mut().entity_mut(row).insert(Interaction::Pressed);
    app.update();

    let model = app.world().resource::<MissionUiModel>();
    assert!(!model.npc_icon_mode_visible);
    assert!(matches!(model.journal, MissionJournalUi::Allow(_)));
}

#[test]
fn chapter_five_select_journal_accepts_click_while_camera_mouse_is_locked() {
    use crate::tutorial::{MissionStage, TutorialStage};

    let mut native = TutorialNativeMechanics::default();
    native.sync_stable_stage(TutorialStage::Mission(MissionStage::SelectJournal));
    let model = MissionUiModel {
        enabled: true,
        nanocom_main_menu_visible: true,
        nanocom_journal: JournalOtherUi {
            title: "MISSION JOURNAL".to_owned(),
            active_missions: vec![mission(500, 100)],
            ..default()
        },
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(native)
        .insert_resource(model)
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_mission_ui_buttons);
    app.world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::NanocomJournal));

    app.update();

    let model = app.world().resource::<MissionUiModel>();
    assert!(!model.nanocom_main_menu_visible);
    assert_eq!(
        model.tutorial_observation().journal_mode,
        TutorialJournalMode::Other,
    );
    assert_eq!(model.viewed_journal_task_id, Some(500));
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![
            GameplayUiAction::CloseNanocomMenu,
            GameplayUiAction::OpenMissionJournal,
        ],
    );
}

#[test]
fn active_row_press_browses_until_make_current_button_is_pressed() {
    let model = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            title: "MISSION JOURNAL".to_owned(),
            active_missions: vec![mission(500, 100), mission(501, 100)],
            ..default()
        }),
        journal_tab: JournalListTab::Active,
        viewed_journal_task_id: Some(500),
        selected_journal_task_id: Some(500),
        ..default()
    };
    let mut app = App::new();
    app.init_resource::<TutorialNativeMechanics>()
        .insert_resource(model)
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_mission_ui_buttons);
    app.world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::JournalMissionRow(1)));

    app.update();

    let model = app.world().resource::<MissionUiModel>();
    assert_eq!(model.viewed_journal_task_id, Some(501));
    assert_eq!(model.selected_journal_task_id, Some(500));
    assert_eq!(
        current_mission(model).map(|mission| mission.task_id),
        Some(501)
    );
    assert!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .next()
            .is_none(),
        "the clean channel-12 selection is client-local and emits no gameplay packet"
    );

    app.world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::JournalMakeCurrent));
    app.update();
    let model = app.world().resource::<MissionUiModel>();
    assert_eq!(model.viewed_journal_task_id, Some(501));
    assert_eq!(model.selected_journal_task_id, Some(501));
    assert!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .next()
            .is_none()
    );
}

#[test]
fn long_completed_journal_scrolls_rows_through_the_clipped_source_view() {
    let mut model = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            title: "MISSION JOURNAL".to_owned(),
            ..default()
        }),
        ..default()
    };
    model.set_completed_journal_missions(
        (0..20).map(|index| mission(700 + index, 100)).collect(),
    );
    assert!(model.select_journal_tab(JournalListTab::Completed));
    let layout = journal_right_layout(&model);
    // `DoWindowRight`: 140 px plus 71 px per expanded row, plus the
    // selected row's extra 30 px.
    assert_eq!(layout.content_height, 140.0 + 20.0 * 71.0 + 30.0);
    assert!(layout.rows.len() <= JOURNAL_MISSION_ROW_LIMIT);
    let max_scroll = layout.content_height - JOURNAL_SCROLL_VIEW_RECT.height;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<MouseWheel>()
        .insert_resource(model)
        .add_systems(Update, scroll_mission_journal);
    let viewport = app
        .world_mut()
        .spawn((
            JournalScrollViewport,
            RelativeCursorPosition {
                cursor_over: true,
                normalized: Some(Vec2::ZERO),
            },
        ))
        .id();
    let content = app
        .world_mut()
        .spawn((JournalScrollContent, Node::default()))
        .id();
    let wheel = |app: &mut App, lines: f32| {
        app.world_mut().write_message(MouseWheel {
            phase: bevy::input::touch::TouchPhase::Moved,
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: lines,
            window: Entity::PLACEHOLDER,
        });
        app.update();
        let scroll = app.world().resource::<MissionUiModel>().journal_scroll;
        assert_eq!(
            app.world().get::<Node>(content).unwrap().top,
            px(-JOURNAL_SCROLL_VIEW_RECT.y - scroll)
        );
        scroll
    };
    assert_eq!(wheel(&mut app, 0.0), 0.0);
    assert_eq!(wheel(&mut app, -1.0), JOURNAL_SCROLL_WHEEL_LINE);
    assert_eq!(wheel(&mut app, -100.0), max_scroll);
    let layout = journal_right_layout(app.world().resource::<MissionUiModel>());
    assert_eq!(layout.rows.last().map(|row| row.mission.task_id), Some(719));
    assert!(layout.rows.len() <= JOURNAL_MISSION_ROW_LIMIT);

    app.world_mut()
        .get_mut::<RelativeCursorPosition>(viewport)
        .unwrap()
        .cursor_over = false;
    assert_eq!(wheel(&mut app, 3.0), max_scroll);
    assert!(
        app.world_mut()
            .resource_mut::<MissionUiModel>()
            .select_journal_tab(JournalListTab::Active)
    );
    assert_eq!(wheel(&mut app, 0.0), 0.0);
}
