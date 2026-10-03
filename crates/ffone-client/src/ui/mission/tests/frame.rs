use super::*;

#[test]
fn quest_frame_keeps_its_minimum_and_fits_wrapped_rows_on_small_viewport() {
    let mut app = App::new();
    app.init_resource::<MissionUiModel>()
        .init_resource::<NpcMissionRowHeights>()
        .init_resource::<GameplayMenuTransition>()
        .add_systems(Update, update_mission_ui_layout);
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(1264, 681),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));
    let quest = app
        .world_mut()
        .spawn((MissionUiView::NpcQuestRoot, Node::default(), UiTransform::default()))
        .id();
    app.update();
    assert_eq!(app.world().get::<Node>(quest).unwrap().height, px(183.0));

    app.world_mut().resource_mut::<MissionUiModel>().npc_interaction = Some(NpcInteractionUi {
        available_missions: vec![MissionUiEntry::default()],
        ..default()
    });
    app.world_mut().resource_mut::<NpcMissionRowHeights>().0[0] = 800.0;
    app.update();
    assert_eq!(app.world().get::<Node>(quest).unwrap().height, px(951.0));
    let scale = app.world().get::<UiTransform>(quest).unwrap().scale.y;
    assert!((scale * 951.0 - 681.0).abs() < 0.01);
}

#[test]
fn rewardless_hand_in_keeps_npc_presentation_visible_on_every_pending_frame() {
    for accepted in [true, false] {
        let locator = AssetLocator::open(asset_root()).unwrap();
        let content = TutorialMissionContent::open(&locator).unwrap();
        let mission = content.mission_entry(2253, 100, "Test zone").unwrap();
        assert!(!mission.has_task_reward);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<Font>()
            .init_resource::<MissionUiModel>()
            .init_resource::<GameplayMenuTransition>()
            .init_resource::<TutorialNativeMechanics>()
            .init_resource::<GameplayUiOutbox>()
            .init_resource::<GameplayUiAudioOutbox>()
            .insert_resource(content)
            .init_resource::<NpcMissionRowHeights>()
            .add_systems(Update, (handle_mission_ui_buttons, bind_mission_ui).chain());
        let assets = MissionUiAssets::load(app.world().resource::<AssetServer>());
        app.insert_resource(assets);
        let letterbox = app
            .world_mut()
            .spawn((MissionUiView::NpcLetterboxRoot, Node::default()))
            .id();
        let quest = app
            .world_mut()
            .spawn((MissionUiView::NpcQuestRoot, Node::default()))
            .id();
        let utility = app
            .world_mut()
            .spawn((MissionUiView::NpcWarpRoot, Node::default()))
            .id();
        let journal = app
            .world_mut()
            .spawn((MissionUiView::JournalRoot, Node::default()))
            .id();
        let button = app
            .world_mut()
            .spawn((Interaction::None, MissionUiControl::NpcMissionRow(0)))
            .id();
        app.world_mut()
            .resource_mut::<MissionUiModel>()
            .show_npc_interaction(NpcInteractionUi {
                npc_id: 100,
                completed_missions: vec![mission],
                ..default()
            });

        app.update();
        assert_eq!(
            app.world().get::<Node>(letterbox).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<Node>(quest).unwrap().display,
            Display::Flex
        );
        // Exercise real input and binding, including another press while
        // a world reply is delayed. The local tutorial uses the same model.
        for interaction in [
            Interaction::Pressed,
            Interaction::None,
            Interaction::Pressed,
        ] {
            *app.world_mut().get_mut::<Interaction>(button).unwrap() = interaction;
            app.update();
            let model = app.world().resource::<MissionUiModel>();
            assert!(model.pending.is_some());
            assert!(model.gameplay_input_blocked());
            assert_eq!(model.npc_subtarget_actor_id(), Some(100));
            assert_eq!(
                app.world().get::<Node>(letterbox).unwrap().display,
                Display::Flex
            );
            assert_eq!(
                app.world().get::<Node>(quest).unwrap().display,
                Display::Flex
            );
            assert_eq!(
                app.world().get::<Node>(utility).unwrap().display,
                Display::None
            );
            assert_eq!(
                app.world().get::<Node>(journal).unwrap().display,
                Display::None
            );
        }
        assert_eq!(
            app.world_mut()
                .resource_mut::<GameplayUiOutbox>()
                .drain()
                .collect::<Vec<_>>(),
            vec![GameplayUiAction::QuestEnd {
                task_id: 2253,
                npc_id: 100,
                box1_choice: 0,
                box2_choice: 0,
            }]
        );
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        {
            let mut model = app.world_mut().resource_mut::<MissionUiModel>();
            if accepted {
                assert!(model.confirm_quest_end(2253));
            } else {
                assert!(model.reject_pending(2253));
            }
        }
        app.update();
        assert_eq!(
            app.world().get::<Node>(letterbox).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<Node>(journal).unwrap().display,
            Display::None
        );
        // Success changes only the menu contents, failure keeps the row
        // available for retry; neither toggles the shared NPC backdrop.
        assert_eq!(
            app.world().get::<Node>(quest).unwrap().display,
            if accepted {
                Display::None
            } else {
                Display::Flex
            }
        );
        assert_eq!(
            app.world().get::<Node>(utility).unwrap().display,
            if accepted {
                Display::Flex
            } else {
                Display::None
            }
        );
        assert!(app.world().resource::<MissionUiModel>().pending.is_none());
    }
}
