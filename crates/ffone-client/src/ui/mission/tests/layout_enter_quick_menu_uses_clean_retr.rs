use super::*;

#[test]
fn exact_reference_rectangles_are_stable() {
    assert_eq!(
        JOURNAL_WINDOW_RECT,
        MissionUiRect::new(122.0, 33.0, 1036.0, 654.0)
    );
    assert_eq!(
        JOURNAL_ACCEPT_RECT,
        MissionUiRect::new(347.0, 562.0, 218.0, 85.0)
    );
    assert_eq!(
        JOURNAL_COMPLETE_RECT,
        MissionUiRect::new(327.0, 562.0, 233.0, 70.0)
    );
    assert_eq!(JOURNAL_REWARD_GROUP_RECT, JOURNAL_ALLOW_GROUP_RECT);
    assert_eq!(
        JOURNAL_MISSION_TITLE_RECT,
        MissionUiRect::new(44.0, 33.0, 317.0, 47.0)
    );
    assert_eq!(
        JOURNAL_FM_CARD_RECT,
        MissionUiRect::new(30.0, 336.0, 250.0, 76.0)
    );
    assert_eq!(
        JOURNAL_FM_ICON_RECT,
        MissionUiRect::new(40.0, 342.0, 64.0, 64.0)
    );
    assert_eq!(
        JOURNAL_MISSION_TITLE_RECT.translated(1.0, 7.0),
        MissionUiRect::new(45.0, 40.0, 317.0, 47.0)
    );
    assert_eq!(
        NPC_SINGLE_MISSION_RECT,
        MissionUiRect::new(794.0, 268.5, 332.0, 183.0)
    );
    assert_eq!(
        NANOCOM_MENU_RECT,
        MissionUiRect::new(1102.0, 112.0, 178.0, 294.0)
    );
    assert_eq!(
        NANOCOM_CLOSE_RECT,
        MissionUiRect::new(153.0, 42.0, 23.0, 23.0)
    );
    assert_eq!(
        NANOCOM_JOURNAL_RECT,
        MissionUiRect::new(11.0, 100.0, 156.0, 26.0)
    );
}

#[test]
fn npc_service_dialog_uses_original_heights_and_routes_vendor() {
    let mut model = MissionUiModel::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 77,
        name: "Numbuh Two".to_owned(),
        ..default()
    });
    assert_eq!(
        npc_utility_panel_height(&model),
        114.0,
        "a mission acknowledgement must return to the legacy single CLOSE window"
    );

    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 77,
        name: "Shopkeeper".to_owned(),
        services: vec![NpcServiceUiEntry::original(NpcServiceKind::Vendor)],
        ..default()
    });
    assert_eq!(npc_utility_panel_height(&model), 171.0);

    model
        .npc_interaction
        .as_mut()
        .expect("interaction")
        .services
        .push(NpcServiceUiEntry::original(NpcServiceKind::Bank));
    assert_eq!(npc_utility_panel_height(&model), 228.0);

    model
        .npc_interaction
        .as_mut()
        .expect("interaction")
        .services
        .truncate(1);
    let mut outbox = GameplayUiOutbox::default();
    assert!(model.activate_npc_utility(0, &mut outbox));
    assert!(!model.npc_icon_mode_visible);
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::NpcService {
            npc_id: 77,
            service: NpcServiceKind::Vendor,
        }]
    );
}

#[test]
fn journal_serialized_skin_borders_are_exact() {
    assert_eq!(
        source_style_border(MISSION_TOP),
        Some(BorderRect {
            min_inset: Vec2::new(20.0, 0.0),
            max_inset: Vec2::new(45.0, 0.0)
        })
    );
    assert_eq!(
        source_style_border(MISSION_BODY),
        Some(BorderRect {
            min_inset: Vec2::new(10.0, 0.0),
            max_inset: Vec2::new(10.0, 0.0)
        })
    );
    assert_eq!(
        source_style_border(MISSION_BUTTON),
        Some(BorderRect::all(5.0))
    );
    assert_eq!(
        source_style_border(NPC_MULTI_WINDOW),
        Some(BorderRect {
            min_inset: Vec2::new(20.0, 40.0),
            max_inset: Vec2::new(45.0, 20.0)
        })
    );
    assert_eq!(
        source_style_border(OFFER_DIALOG),
        Some(BorderRect {
            min_inset: Vec2::new(38.0, 0.0),
            max_inset: Vec2::new(27.0, 0.0)
        })
    );
    assert_eq!(
        source_style_border(JOURNAL_REWARD_BOX),
        Some(BorderRect::all(2.0))
    );
    assert_eq!(
        source_style_border(JOURNAL_WINDOW),
        Some(BorderRect::all(5.0))
    );
    assert_eq!(
        source_style_border(JOURNAL_RIGHT_FRAME),
        Some(BorderRect {
            min_inset: Vec2::new(0.0, 30.0),
            max_inset: Vec2::new(0.0, 30.0)
        })
    );
    assert_eq!(source_style_border(JOURNAL_CLOSE), None);
}

#[test]
fn enter_quick_menu_uses_clean_retrobution_chat_rects_and_styles() {
    assert_eq!(
        CHAT_QUICK_MENU_RECT,
        MissionUiRect::new(0.0, 1.0, 170.0, 127.0)
    );
    assert_eq!(CHAT_WINDOW_HEIGHT, 150.0);
    assert_eq!(CHAT_EMOTE_CONTAINER_HEIGHT, 235.0);
    assert_eq!(
        chat_quick_menu_logical_rect(681.0),
        MissionUiRect::new(0.0, 297.0, 170.0, 127.0)
    );
    assert_eq!(
        CHAT_QUICK_GROUP_RECT,
        MissionUiRect::new(10.0, 22.0, 155.0, 25.0)
    );
    assert_eq!(
        CHAT_QUICK_WARP_RECT,
        MissionUiRect::new(10.0, 52.0, 155.0, 25.0)
    );
    assert_eq!(
        CHAT_QUICK_VEHICLE_RECT,
        MissionUiRect::new(10.0, 82.0, 155.0, 25.0)
    );
    assert_eq!(
        MenuButtonStyle::ChatRed.skin_and_style(),
        ("FusionFallChatSkin", "RedButton")
    );
    assert_eq!(
        MenuButtonStyle::ChatBlue.skin_and_style(),
        ("FusionFallChatSkin", "BlueButton")
    );
    assert_eq!(
        skin_style_border("FusionFallChatSkin", "EmoteBox"),
        Some(BorderRect {
            min_inset: Vec2::new(20.0, 20.0),
            max_inset: Vec2::new(5.0, 20.0)
        })
    );

    let hud_button = mission_gui_style("FusionFallHUDSkin", "button").unwrap();
    let hud_exit = mission_gui_style("FusionFallHUDSkin", "RedButton").unwrap();
    let chat_red = mission_gui_style("FusionFallChatSkin", "RedButton").unwrap();
    let warp_countdown = mission_gui_style("FusionFallChatSkin", "BigFont70").unwrap();
    assert_eq!(hud_button.alignment, 4);
    assert_eq!(hud_exit.alignment, 4);
    assert_eq!(chat_red.alignment, 4);
    assert_eq!(warp_countdown.alignment, 4);
    assert_eq!(JEFFE_14_SOURCE_FONT_PATH_ID, 903);
    assert_eq!(JEFFE_14_REPLACEMENT_FONT_SIZE, 12.0);
    assert_eq!(JEFFE_14_LINE_HEIGHT, 13.710_000_04);
    assert_eq!(CENTERED_MENU_JEFFE_FONT_SIZE, 14.0);
    assert_eq!(CENTERED_MENU_JEFFE_VERTICAL_SCALE, 0.7);
    assert_eq!(
        mission_gui_effective_font("FusionFallHUDSkin", "button")
            .unwrap()
            .path_id,
        JEFFE_14_SOURCE_FONT_PATH_ID
    );
    assert_eq!(
        mission_gui_effective_font("FusionFallHUDSkin", "RedButton")
            .unwrap()
            .path_id,
        JEFFE_14_SOURCE_FONT_PATH_ID
    );
    assert_eq!(
        mission_gui_effective_font("FusionFallChatSkin", "BigFont70")
            .unwrap()
            .path_id,
        1_129
    );
    assert_eq!(
        mission_text_content_rect(MissionUiRect::new(0.0, 0.0, 156.0, 26.0), hud_button),
        MissionUiRect::new(10.0, 3.0, 140.0, 17.0)
    );
    assert_eq!(
        mission_text_content_rect(MissionUiRect::new(0.0, 0.0, 156.0, 26.0), hud_exit),
        MissionUiRect::new(10.0, 4.0, 140.0, 16.0)
    );
    assert_eq!(
        mission_text_content_rect(MissionUiRect::new(0.0, 0.0, 155.0, 25.0), chat_red),
        MissionUiRect::new(2.0, 2.0, 151.0, 21.0)
    );
    assert_eq!(
        nanocom_button_source_path(MenuButtonStyle::HudBlue, &Interaction::Hovered),
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH
    );
    assert_eq!(
        nanocom_button_source_path(MenuButtonStyle::HudBlue, &Interaction::Pressed),
        CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        "Unity GUIStyle.active returns to the serialized normal background"
    );
    assert_eq!(
        nanocom_button_source_path(MenuButtonStyle::HudRed, &Interaction::Hovered),
        CHARACTER_SELECTION_RED_BUTTON_OVER_PATH
    );
    assert_eq!(menu_interaction_state(&Interaction::Pressed), "active");
}

#[test]
fn available_first_nano_task_starts_and_later_talk_ends_directly() {
    let root = asset_root();
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let first = content
        .mission_entry(2250, 100, "Pokey Oaks North")
        .unwrap();
    let later = content.mission_entry(2253, 100, "Fusion Lair").unwrap();
    assert!(!later.is_first_mission_task);

    let mut available_model = MissionUiModel::default();
    let mut available_outbox = GameplayUiOutbox::default();
    available_model.show_npc_interaction(NpcInteractionUi {
        npc_id: 100,
        available_missions: vec![first],
        ..default()
    });
    assert!(available_model.select_npc_mission(0, &mut available_outbox));
    available_outbox.drain().for_each(drop);
    assert!(available_model.accept_mission(&mut available_outbox));
    assert!(matches!(
        available_model.pending,
        Some(PendingMissionUiRequest::TaskStart { task_id: 2250, .. })
    ));

    let mut later_model = MissionUiModel::default();
    let mut later_outbox = GameplayUiOutbox::default();
    later_model.show_npc_interaction(NpcInteractionUi {
        npc_id: 100,
        completed_missions: vec![later],
        ..default()
    });
    assert!(later_model.select_npc_mission(0, &mut later_outbox));
    assert!(matches!(
        later_model.pending,
        Some(PendingMissionUiRequest::QuestEnd { task_id: 2253, .. })
    ));
}

#[test]
fn completed_assets_and_left_rectangles_match_retrobution() {
    assert_eq!(END_DIALOG, "ui/en/gameplay/mission/journal/enddlg.png");
    assert_eq!(
        JOURNAL_COMPLETED_PANEL,
        "ui/en/gameplay/journal/compl_tab.png"
    );
    assert_eq!(JOURNAL_ACTIVE_TAB, "ui/en/gameplay/journal/inventab.png");
    assert_eq!(
        JOURNAL_ACTIVE_TAB_OVER,
        "ui/en/gameplay/journal/inventabover.png"
    );
    assert_eq!(JOURNAL_COMPLETED_TAB, "ui/en/shared/nanotab.png");
    assert_eq!(
        JOURNAL_COMPLETED_TAB_OVER,
        "ui/en/gameplay/journal/nanotabover.png"
    );
    assert_eq!(
        JOURNAL_COMPLETED_BANNER_RECT,
        MissionUiRect::new(21.0, 31.0, 549.0, 74.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_MISSION_TITLE_RECT,
        MissionUiRect::new(45.0, 40.0, 317.0, 47.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_MISSION_DIFFICULTY_RECT,
        MissionUiRect::new(45.0, 86.0, 317.0, 20.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_NPC_FRAME_RECT,
        MissionUiRect::new(23.0, 120.0, 68.0, 68.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_NPC_NAME_RECT,
        MissionUiRect::new(99.0, 118.0, 300.0, 20.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_NPC_POSITION_RECT,
        MissionUiRect::new(99.0, 136.0, 300.0, 20.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_NOTES_HEADER_RECT,
        MissionUiRect::new(35.0, 198.0, 500.0, 18.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_NOTES_RECT,
        MissionUiRect::new(35.0, 218.0, 500.0, 100.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_REWARD_HEADER_RECT,
        MissionUiRect::new(22.0, 343.0, 300.0, 20.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_REWARD_VIEW_RECT,
        MissionUiRect::new(21.0, 368.0, 577.0, 245.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_NANO_LABEL_RECT,
        MissionUiRect::new(24.0, 345.0, 200.0, 30.0)
    );
    assert_eq!(
        JOURNAL_COMPLETED_NANO_GROUP_RECT,
        MissionUiRect::new(14.0, 362.0, 555.0, 245.0)
    );
    assert_eq!(
        source_style_border(END_DIALOG),
        Some(BorderRect {
            min_inset: Vec2::new(38.0, 0.0),
            max_inset: Vec2::new(27.0, 0.0)
        })
    );
    assert_eq!(
        source_style_border(JOURNAL_COMPLETED_PANEL),
        Some(BorderRect {
            min_inset: Vec2::new(0.0, 35.0),
            max_inset: Vec2::new(0.0, 35.0)
        })
    );
}
