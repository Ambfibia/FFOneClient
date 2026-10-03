use super::*;

#[test]
fn transportation_service_icons_match_clean_primary_bytes() {
    // Verified primary PNG export; encoded bytes follow the current
    // publisher while dimensions and pixels retain the service icons.
    const NPCICON_MONKEY_SOURCE_BYTES: usize = 2_058;
    const NPCICON_MONKEY_SOURCE_SHA256: &str =
        "460CF57F93AE39FD3BDD03AA866FA4C0171E90F932D34684EC86CD80AA15B576";
    const NPCICON_SCAMP_SOURCE_BYTES: usize = 1_270;
    const NPCICON_SCAMP_SOURCE_SHA256: &str =
        "1AF2FB6CDFA82F9F8799EB3D339B4E13D60A745E98BA9266E093FEC4314EEC9F";
    let cases = [
        (
            NPCICON_MONKEY,
            include_bytes!("../../../../../../assets/game/ui/en/gameplay/journal/npcicon_10.png")
                .as_slice(),
            NPCICON_MONKEY_SOURCE_BYTES,
            NPCICON_MONKEY_SOURCE_SHA256,
        ),
        (
            NPCICON_SCAMP,
            include_bytes!("../../../../../../assets/game/ui/en/gameplay/journal/npcicon_11.png")
                .as_slice(),
            NPCICON_SCAMP_SOURCE_BYTES,
            NPCICON_SCAMP_SOURCE_SHA256,
        ),
    ];
    for (path, bytes, expected_len, expected_sha256) in cases {
        assert_eq!(bytes.len(), expected_len, "{path} source byte count");
        assert_eq!(
            format!("{:X}", Sha256::digest(bytes)),
            expected_sha256,
            "{path} source hash"
        );
        let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .unwrap_or_else(|error| panic!("{path} must remain a valid PNG: {error}"));
        assert_eq!((image.width(), image.height()), (38, 33), "{path} size");
    }
}

#[test]
fn utility_service_icons_match_clean_primary_bytes() {
    let cases = [
        (
            NPCICON_RANK,
            include_bytes!("../../../../../../assets/game/ui/en/gameplay/journal/npcicon_08.png")
                .as_slice(),
            2_082,
            "951126E355A64C5CC038B96FF74C8109705A482C949E98667A55E7B671AB7D5A",
        ),
        (
            NPCICON_COMBINE,
            include_bytes!("../../../../../../assets/game/ui/en/gameplay/journal/npcicon_20.png")
                .as_slice(),
            2_371,
            "D301439CF1A871372EF2B79786E831F501E990A3FE9CC05D8D216F81BE6D0AF0",
        ),
        (
            NPCICON_ENCHANT,
            include_bytes!("../../../../../../assets/game/ui/en/gameplay/journal/npcicon_22.png")
                .as_slice(),
            2_528,
            "FEE52CE0DC01E379CA2BF426AF69A85E5FD58966B79B64AE477CC336B7EC85B8",
        ),
        (
            NPCICON_RULE,
            include_bytes!("../../../../../../assets/game/ui/en/gameplay/journal/npcicon_23.png")
                .as_slice(),
            2_528,
            "FEE52CE0DC01E379CA2BF426AF69A85E5FD58966B79B64AE477CC336B7EC85B8",
        ),
    ];
    for (path, bytes, expected_len, expected_sha256) in cases {
        assert_eq!(bytes.len(), expected_len, "{path} source byte count");
        assert_eq!(
            format!("{:X}", Sha256::digest(bytes)),
            expected_sha256,
            "{path} source hash"
        );
        let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .unwrap_or_else(|error| panic!("{path} must remain a valid PNG: {error}"));
        assert_eq!((image.width(), image.height()), (38, 33), "{path} size");
    }
}

pub(super) fn mission(task_id: i32, npc_id: i32) -> MissionUiEntry {
    MissionUiEntry {
        task_id,
        npc_id: Some(npc_id),
        mission_type: 3,
        task_type: 5,
        outgoing_task_id: 0,
        has_task_reward: true,
        is_first_mission_task: false,
        journal_npc_type: 2671,
        required_level: 1,
        difficulty_type: 0,
        nano: None,
        title: format!("Mission {task_id}"),
        npc_name: "Numbuh Two".to_owned(),
        npc_position: "Pokey Oaks North".to_owned(),
        objective: "Tutorial mission objective".to_owned(),
        offer_description: "Tutorial mission offer".to_owned(),
        task_description: "Tutorial mission task".to_owned(),
        active_description: "Tutorial mission objective".to_owned(),
        mission_summary: format!("Mission {task_id} summary"),
        mission_complete_summary: format!("Mission {task_id} completed notes"),
        completion_description: "Tutorial mission complete".to_owned(),
        rewards: MissionUiRewards {
            cash: 0,
            fusion_matter: 75,
        },
    }
}

#[test]
fn multiline_mission_rows_move_following_rows_and_grow_dialogue() {
    let mut model=MissionUiModel::default();
    model.npc_interaction=Some(NpcInteractionUi {available_missions:vec![mission(500,100),mission(501,100)],..default()});
    let normal=NpcMissionRowHeights::default();let mut wrapped=normal.clone();wrapped.0[0]=80.;
    assert_eq!(npc_rows(&model,&wrapped)[1].1-npc_rows(&model,&normal)[1].1,48.);
    assert_eq!(quest_list_end(&model,&wrapped)-quest_list_end(&model,&normal),48.);
}

pub(super) fn npc_with_available() -> NpcInteractionUi {
    NpcInteractionUi {
        npc_id: 100,
        name: "Numbuh Two".to_owned(),
        available_missions: vec![mission(500, 100)],
        ..default()
    }
}

#[test]
fn mission_single_line_text_keeps_source_padding_without_glyph_clipping() {
    let style = mission_gui_style("FusionFallInteractionSkin", "Sel_Bar").unwrap();
    let outer = MissionUiRect::new(0.0, 50.0, 328.0, 20.0);
    assert_eq!(
        mission_text_content_rect(outer, style),
        MissionUiRect::new(10.0, 54.0, 312.0, 10.0)
    );
    assert_eq!(
        mission_text_layout_rect(
            outer,
            style,
            &(
                TextFont {
                    font_size: (12.0).into(),
                    ..default()
                },
                LineHeight::Px(13.710_000_04)
            ),
        ),
        MissionUiRect::new(10.0, 54.0, 312.0, 13.710_000_04)
    );
}

#[test]
fn journal_managed_literals_and_close_states_are_exact() {
    assert_eq!(JOURNAL_MISSION_OFFER_LABEL, "MISSION OFFER");
    assert_eq!(JOURNAL_MISSION_DETAILS_LABEL, "MISSION DETAILS");
    assert_eq!(JOURNAL_MY_NOTES_LABEL, "MY NOTES");
    assert_eq!(JOURNAL_MISSION_SUMMARY_LABEL, "MISSION SUMMARY");
    assert_eq!(
        journal_managed_label(JOURNAL_MISSION_SUMMARY_LABEL),
        "MISSION SUMMARY:"
    );
    assert_eq!(JOURNAL_REWARD_LABEL, "REWARD");
    assert_eq!(journal_managed_label(JOURNAL_REWARD_LABEL), "REWARD:");
    let mut model = MissionUiModel {
        journal: MissionJournalUi::Allow(mission(500, 100)),
        ..default()
    };
    assert_eq!(journal_objective_label(&model), JOURNAL_MISSION_OFFER_LABEL);
    model.journal = MissionJournalUi::Other(JournalOtherUi::default());
    assert_eq!(
        journal_objective_label(&model),
        JOURNAL_MISSION_DETAILS_LABEL
    );
    model.journal_tab = JournalListTab::Completed;
    assert_eq!(journal_objective_label(&model), JOURNAL_MY_NOTES_LABEL);
    assert_eq!(JOURNAL_FUSION_MATTER_LABEL, "FUSION MATTER");
    assert!(journal_close_uses_hover_asset(&Interaction::Hovered));
    assert!(!journal_close_uses_hover_asset(&Interaction::Pressed));
    assert!(!journal_close_uses_hover_asset(&Interaction::None));
    assert!(MissionUiRewards::default().is_empty());
    assert!(
        !MissionUiRewards {
            cash: 0,
            fusion_matter: 75,
        }
        .is_empty()
    );
}

#[test]
fn delayed_oil_ogre_accept_is_ignored_until_mouse_unlocks() {
    use crate::tutorial::{MissionStage, TutorialStage};

    let mut app = App::new();
    app.init_resource::<MissionUiModel>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_mission_ui_buttons);

    {
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        model.show_npc_interaction(NpcInteractionUi {
            npc_id: 1005,
            name: "Numbuh Two".to_owned(),
            available_missions: vec![mission(2248, 1005)],
            ..default()
        });
    }
    {
        let world = app.world_mut();
        world.resource_scope(|world, mut model: Mut<MissionUiModel>| {
            let mut outbox = world.resource_mut::<GameplayUiOutbox>();
            assert!(model.select_npc_mission(0, &mut outbox));
            outbox.drain().for_each(drop);
        });
        world
            .resource_mut::<TutorialNativeMechanics>()
            .sync_stable_stage(TutorialStage::Mission(MissionStage::NumbuhTwoDelay));
    }

    let accept = app
        .world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::JournalPrimary))
        .id();
    app.update();
    assert_eq!(
        app.world().resource::<MissionUiModel>().pending,
        None,
        "the legacy bBackground/Mouse lock must swallow an early ACCEPT"
    );
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    assert!(app.world().resource::<GameplayUiAudioOutbox>().is_empty());

    app.world_mut()
        .resource_mut::<TutorialNativeMechanics>()
        .sync_stable_stage(TutorialStage::Mission(MissionStage::AcceptMission));
    app.world_mut().entity_mut(accept).insert(Interaction::None);
    app.update();
    app.world_mut()
        .entity_mut(accept)
        .insert(Interaction::Pressed);
    app.update();

    assert_eq!(
        app.world().resource::<MissionUiModel>().pending,
        Some(PendingMissionUiRequest::TaskStart {
            task_id: 2248,
            npc_id: 1005,
        })
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::TaskStart {
            task_id: 2248,
            npc_id: 1005,
        }]
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::MissionAccepted]
    );
}

#[test]
fn tutorial_exit_system_message_47_contract_is_exact() {
    assert_eq!(TUTORIAL_EXIT_SYSTEM_MESSAGE_ID, 47);
    assert_eq!(TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE, 2);
    assert_eq!(
        TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT,
        "Are you sure you want to skip the tutorial? "
    );
    assert!(TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT.ends_with(' '));
    assert_eq!(TUTORIAL_EXIT_KEY, KeyCode::Backquote);
    assert_eq!(SYSTEM_DIALOG_OVERLAY_ALPHA, 0.75);
    assert_eq!(
        SYSTEM_DIALOG_PANEL_RECT,
        MissionUiRect::new(0.0, 0.0, 600.0, 164.0)
    );
    assert_eq!(
        SYSTEM_DIALOG_CONTENT_RECT,
        MissionUiRect::new(140.0, 20.0, 420.0, 104.0)
    );
    assert_eq!(
        SYSTEM_DIALOG_OKAY_RECT,
        MissionUiRect::new(454.0, 124.0, 110.0, 25.0)
    );
    assert_eq!(
        SYSTEM_DIALOG_CANCEL_RECT,
        MissionUiRect::new(44.0, 124.0, 150.0, 25.0)
    );
    assert_eq!(
        source_style_border(SYSTEM_DIALOG_BOX),
        Some(BorderRect {
            min_inset: Vec2::new(40.0, 16.0),
            max_inset: Vec2::new(40.0, 40.0)
        })
    );
    assert_eq!(
        source_style_border(CHARACTER_SELECTION_BLUE_BUTTON_PATH),
        Some(BorderRect::all(5.0))
    );
    assert_eq!(
        source_style_border(CHARACTER_SELECTION_CANCEL_NORMAL_PATH),
        Some(BorderRect::all(5.0))
    );
    assert_eq!(
        system_dialog_button_source_path(
            MissionUiControl::SystemDialogOkay,
            &Interaction::Hovered
        ),
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH
    );
    assert_eq!(
        system_dialog_button_source_path(
            MissionUiControl::SystemDialogCancel,
            &Interaction::None
        ),
        CHARACTER_SELECTION_CANCEL_NORMAL_PATH
    );
    assert_eq!(
        system_dialog_button_source_path(
            MissionUiControl::SystemDialogCancel,
            &Interaction::Pressed
        ),
        CHARACTER_SELECTION_BLUE_BUTTON_PATH
    );
}

#[test]
fn tutorial_exit_gate_requires_ready_gameplay_and_allows_no_blocker() {
    let ready = TutorialExitDialogGate {
        tutorial_active: true,
        world_ready: true,
        startup_ready: true,
        ..default()
    };
    assert!(ready.allows_open());
    assert!(
        !TutorialExitDialogGate {
            delay_active: true,
            ..ready
        }
        .allows_open()
    );
    assert!(
        !TutorialExitDialogGate {
            system_popup_active: true,
            ..ready
        }
        .allows_open()
    );
    assert!(
        !TutorialExitDialogGate {
            event_scene_active: true,
            ..ready
        }
        .allows_open()
    );
    assert!(
        !TutorialExitDialogGate {
            tutorial_active: false,
            ..ready
        }
        .allows_open()
    );
    assert!(
        !TutorialExitDialogGate {
            world_ready: false,
            ..ready
        }
        .allows_open()
    );
    assert!(
        !TutorialExitDialogGate {
            startup_ready: false,
            ..ready
        }
        .allows_open()
    );
}

#[test]
fn tutorial_exit_cancel_and_confirmation_are_exactly_once_and_forgery_safe() {
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    assert!(model.open_tutorial_exit_dialog());
    assert!(!model.open_tutorial_exit_dialog());
    assert!(model.cancel_tutorial_exit_dialog());
    assert!(!model.cancel_tutorial_exit_dialog());
    assert!(outbox.is_empty());

    model.system_dialog = Some(TutorialSystemDialogUi {
        message_id: 999,
        button_type: TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE,
        text: TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT,
    });
    assert!(!model.confirm_tutorial_exit_dialog(&mut outbox));
    assert!(!model.cancel_tutorial_exit_dialog());
    model.system_dialog = None;

    assert!(model.open_tutorial_exit_dialog());
    assert!(model.confirm_tutorial_exit_dialog(&mut outbox));
    assert!(!model.confirm_tutorial_exit_dialog(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::ConfirmTutorialExit {
            message_id: 47,
            button_type: 2,
        }]
    );
    assert!(!model.take_tutorial_exit_confirmation(999, 2));
    assert!(model.take_tutorial_exit_confirmation(47, 2));
    assert!(!model.take_tutorial_exit_confirmation(47, 2));
    assert!(model.open_tutorial_exit_dialog());
}

#[test]
fn all_seven_clean_nanocom_rows_are_live_and_typed() {
    assert_eq!(
        NANOCOM_MENU_LABELS,
        [
            "MY STUFF",
            "JOURNAL",
            "E-MAIL",
            "MAP",
            "SETTINGS",
            "GAME GUIDE",
            "EXIT GAME",
        ]
    );
    assert_eq!(
        (0..NANOCOM_MENU_LABELS.len())
            .map(nanocom_control)
            .collect::<Vec<_>>(),
        vec![
            MissionUiControl::NanocomMyStuff,
            MissionUiControl::NanocomJournal,
            MissionUiControl::NanocomEmail,
            MissionUiControl::NanocomMap,
            MissionUiControl::NanocomSettings,
            MissionUiControl::NanocomGameGuide,
            MissionUiControl::NanocomExitGame,
        ]
    );
    assert_eq!(
        MenuButtonStyle::HudRed.skin_and_style(),
        ("FusionFallHUDSkin", "RedButton")
    );
    assert_eq!(
        skin_style_border("FusionFallHUDSkin", "RedButton"),
        Some(BorderRect {
            min_inset: Vec2::new(8.0, 5.0),
            max_inset: Vec2::new(8.0, 5.0)
        })
    );
}

#[test]
fn warp_away_counts_down_once_then_owns_the_clean_channel_12_cooldown() {
    let mut model = MissionUiModel {
        enabled: true,
        nanocom_main_menu_visible: true,
        ..default()
    };
    let mut outbox = GameplayUiOutbox::default();

    assert!(model.warp_away_available());
    assert!(model.begin_warp_away(&mut outbox));
    assert!(!model.begin_warp_away(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::WarpAwayStarted]
    );
    assert_eq!(model.warp_away_display_seconds(), Some(20));

    model.advance_warp_away(19.4, &mut outbox);
    assert_eq!(model.warp_away_display_seconds(), Some(1));
    assert!(outbox.is_empty());
    model.advance_warp_away(0.61, &mut outbox);
    assert_eq!(model.warp_away_countdown_seconds(), None);
    assert!(model.warp_away_request_pending());
    assert_eq!(model.warp_away_cooldown_seconds(), 60.0);
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::RequestWarpAway]
    );

    assert!(model.complete_warp_away());
    assert!(!model.complete_warp_away());
    model.advance_warp_away(60.0, &mut outbox);
    assert_eq!(model.warp_away_cooldown_seconds(), 0.0);
    assert!(model.warp_away_available());
}

#[test]
fn allow_accept_waits_for_ack_then_returns_to_same_npc() {
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    model.show_npc_interaction(npc_with_available());

    assert!(model.select_npc_mission(0, &mut outbox));
    assert_eq!(
        model.tutorial_observation().journal_mode,
        TutorialJournalMode::Allow
    );
    assert!(!model.tutorial_observation().npc_icon_mode_visible);
    assert!(model.accept_mission(&mut outbox));
    assert!(!model.accept_mission(&mut outbox));
    assert!(matches!(
        model.pending,
        Some(PendingMissionUiRequest::TaskStart {
            task_id: 500,
            npc_id: 100
        })
    ));
    assert_eq!(
        model.tutorial_observation().journal_mode,
        TutorialJournalMode::Allow
    );
    assert!(!model.confirm_task_start(999));
    assert!(model.confirm_task_start(500));
    assert_eq!(
        model.tutorial_observation().journal_mode,
        TutorialJournalMode::Hidden
    );
    assert!(model.is_interacting_with(100));

    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenMissionAllow {
                task_id: 500,
                npc_id: 100,
            },
            GameplayUiAction::TaskStart {
                task_id: 500,
                npc_id: 100,
            },
        ]
    );
}

#[test]
fn reward_complete_uses_choices_and_failure_only_unlocks() {
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    let mut npc = npc_with_available();
    npc.available_missions.clear();
    npc.completed_missions.push(mission(700, 100));
    model.show_npc_interaction(npc);

    assert!(model.select_npc_mission(0, &mut outbox));
    let MissionJournalUi::Reward {
        box1_choice,
        box2_choice,
        ..
    } = &mut model.journal
    else {
        panic!("completed mission must open Reward");
    };
    *box1_choice = 2;
    *box2_choice = 1;
    assert!(model.complete_mission(&mut outbox));
    assert!(model.reject_pending(700));
    assert_eq!(
        model.tutorial_observation().journal_mode,
        TutorialJournalMode::Reward
    );
    assert!(model.complete_mission(&mut outbox));
    assert!(model.confirm_quest_end(700));
    assert!(model.is_interacting_with(100));

    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenMissionReward {
                task_id: 700,
                npc_id: 100,
            },
            GameplayUiAction::QuestEnd {
                task_id: 700,
                npc_id: 100,
                box1_choice: 2,
                box2_choice: 1,
            },
            GameplayUiAction::QuestEnd {
                task_id: 700,
                npc_id: 100,
                box1_choice: 2,
                box2_choice: 1,
            },
        ]
    );
}

#[test]
fn intermediate_talk_row_advances_without_opening_completion_screen() {
    let mut talk = mission(701, 100);
    talk.task_type = 1;
    talk.outgoing_task_id = 702;
    talk.has_task_reward = false;
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 100,
        completed_missions: vec![talk],
        ..default()
    });

    assert!(model.select_npc_mission(0, &mut outbox));
    assert!(matches!(model.journal, MissionJournalUi::Hidden));
    assert_eq!(
        model.pending,
        Some(PendingMissionUiRequest::QuestEnd {
            task_id: 701,
            npc_id: 100,
            box1_choice: 0,
            box2_choice: 0,
        })
    );
    assert!(
        !journal_primary_is_visible(&model),
        "an intermediate conversation must not open the completion screen"
    );
    assert!(
        !model.complete_mission(&mut outbox),
        "the selected talk row already owns its exact TASK_END request"
    );
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::QuestEnd {
            task_id: 701,
            npc_id: 100,
            box1_choice: 0,
            box2_choice: 0,
        }]
    );
}

#[test]
fn rejected_intermediate_talk_keeps_the_npc_dialog_list_visible() {
    let mut talk = mission(701, 100);
    talk.task_type = 1;
    talk.outgoing_task_id = 702;
    talk.has_task_reward = false;
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 100,
        completed_missions: vec![talk],
        ..default()
    });

    assert!(model.select_npc_mission(0, &mut outbox));
    assert!(model.npc_icon_mode_visible);
    assert!(model.reject_pending(701));
    assert!(model.npc_icon_mode_visible);
    assert!(matches!(model.journal, MissionJournalUi::Hidden));
    assert!(model.select_npc_mission(0, &mut outbox));
}

#[test]
fn transportation_registration_notice_fills_the_top_letterbox_until_npc_init() {
    let mut model = MissionUiModel::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 7,
        ..default()
    });
    assert!(model.npc_top_notice().is_none());
    model.show_transportation_registration_notice(1);
    assert_eq!(
        model.npc_top_notice().map(|text| text.key.as_str()),
        Some("ui.transportation.registered.warp")
    );
    model.show_transportation_registration_notice(2);
    assert_eq!(
        model.npc_top_notice().map(|text| text.key.as_str()),
        Some("ui.transportation.registered.hub")
    );

    let locator = AssetLocator::open(asset_root()).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_resource(model)
        .init_resource::<GameplayMenuTransition>()
        .insert_resource(TutorialMissionContent::open(&locator).unwrap())
        .init_resource::<NpcMissionRowHeights>()
        .add_systems(Update, bind_mission_ui);
    let assets = MissionUiAssets::load(app.world().resource::<AssetServer>());
    app.insert_resource(assets);
    let bar = app
        .world_mut()
        .spawn((
            MissionUiView::NpcTopNotice,
            Node {
                display: Display::None,
                ..default()
            },
        ))
        .id();
    let text = app
        .world_mut()
        .spawn((
            MissionUiView::NpcTopNoticeText,
            Node::default(),
            LocalizedText::new("ui.content.passthrough", "{text}"),
        ))
        .id();
    app.update();
    assert_eq!(app.world().get::<Node>(bar).unwrap().display, Display::Flex);
    assert_eq!(
        app.world().get::<LocalizedText>(text).unwrap().key,
        "ui.transportation.registered.hub"
    );
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .show_npc_interaction(NpcInteractionUi {
            npc_id: 8,
            ..default()
        });
    app.update();
    assert_eq!(app.world().get::<Node>(bar).unwrap().display, Display::None);

    let (localization, en) = Localization::open(&asset_root(), "en").unwrap();
    let (_, ru) = Localization::open(&asset_root(), "ru").unwrap();
    for transportation_type in [1, 2] {
        let mut model = MissionUiModel::default();
        model.show_transportation_registration_notice(transportation_type);
        let notice = model.npc_top_notice().unwrap();
        assert_ne!(
            localization.text(&en, notice),
            localization.text(&ru, notice)
        );
    }
}
