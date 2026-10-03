use super::*;

#[test]
fn mapped_normal_warp_system_message_dismissal_ends_npc_icon_mode_only() {
    let mut app = App::new();
    app.init_resource::<SystemMessageUiOutbox>()
        .init_resource::<NormalNpcWarpRuntime>()
        .init_resource::<MissionUiModel>()
        .init_resource::<RuntimeStatus>()
        .add_systems(Update, consume_normal_warp_system_message_outbox);
    {
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        model.enabled = true;
        model.npc_icon_mode_visible = true;
        model.npc_interaction = Some(NpcInteractionUi {
            npc_id: 91,
            ..default()
        });
    }
    let mapped_request = NORMAL_NPC_WARP_SYSTEM_MESSAGE_ID_BASE;
    app.world_mut()
        .resource_mut::<NormalNpcWarpRuntime>()
        .pending_system_messages
        .insert(mapped_request, 115);
    app.world_mut()
        .resource_mut::<SystemMessageUiOutbox>()
        .push(SystemMessageUiAction::Chosen {
            request_id: mapped_request,
            button_type: SystemMessageButtonType::Ok,
            choice: SystemMessageChoice::Primary,
        });

    app.update();

    {
        let model = app.world().resource::<MissionUiModel>();
        assert!(model.npc_interaction.is_none());
        assert!(!model.npc_icon_mode_visible);
    }
    assert!(
        app.world()
            .resource::<NormalNpcWarpRuntime>()
            .pending_system_messages
            .is_empty()
    );
    {
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        model.enabled = true;
        model.npc_icon_mode_visible = true;
        model.npc_interaction = Some(NpcInteractionUi {
            npc_id: 92,
            ..default()
        });
    }
    let still_pending = mapped_request + 1;
    app.world_mut()
        .resource_mut::<NormalNpcWarpRuntime>()
        .pending_system_messages
        .insert(still_pending, 173);
    app.world_mut()
        .resource_mut::<SystemMessageUiOutbox>()
        .push(SystemMessageUiAction::Chosen {
            request_id: u64::MAX,
            button_type: SystemMessageButtonType::Ok,
            choice: SystemMessageChoice::Primary,
        });

    app.update();

    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .npc_interaction
            .is_some()
    );
    assert_eq!(
        app.world().resource::<SystemMessageUiOutbox>().len(),
        1,
        "unrelated owners keep their action"
    );
    assert!(
        app.world()
            .resource::<NormalNpcWarpRuntime>()
            .pending_system_messages
            .contains_key(&still_pending)
    );
}

#[test]
fn selected_mission_smart_indicator_uses_grant_waypoint_and_npc_radius() {
    let content = runtime_test_mission_content();
    let mut mission = TutorialMissionRuntime::default();

    assert!(tutorial_selected_waypoint_npc_types(&mission, &content).is_empty());
    mission.start_task(&content, 2248).unwrap();
    assert_eq!(
        tutorial_selected_waypoint_npc_types(&mission, &content),
        BTreeSet::from([2374])
    );
    mission.complete_task(&content, 2248).unwrap();
    let selected = tutorial_selected_waypoint_npc_types(&mission, &content);
    assert_eq!(selected, BTreeSet::from([2671]));
    assert!((tutorial_smart_indicator_scale(&content, 2671).unwrap() - 2.4).abs() < f32::EPSILON);
    let mut actor = TutorialActor {
        id: 1005,
        npc_type: 2671,
        team: 1,
        hp: 100,
        max_hp: 100,
        damaged: false,
        interacting: false,
        invulnerable: false,
    };
    assert!(
        (tutorial_actor_smart_indicator_scale(&content, &actor, &selected).unwrap() - 2.4).abs()
            < f32::EPSILON
    );
    actor.interacting = true;
    assert_eq!(
        tutorial_actor_smart_indicator_scale(&content, &actor, &selected),
        None
    );
    mission.complete_task(&content, 2249).unwrap();
    assert!(tutorial_selected_waypoint_npc_types(&mission, &content).is_empty());
}

#[test]
fn active_category_thirteen_opens_mode_sixteen_as_cancel_not_rank() {
    let catalog = RaceRankCatalog::embedded().unwrap();
    let mut model = RaceModeModel::default();
    let mut production = RaceProductionRuntime::default();
    production.player.ring_race_active = true;
    production.player.current_ep_id = 15;
    open_race_mode_from_npc(
        &mut model,
        &catalog,
        &mut production,
        RaceEcomType::Start,
        9_001,
        6_501,
        "M_SACTRace1".to_owned(),
        800,
        true,
    )
    .unwrap();
    assert_eq!(production.active_game_mode, Some(RACE_MODE_GAME_MODE_ID));
    assert_ne!(production.active_game_mode, Some(RACE_RANK_GAME_MODE_ID));
    assert_eq!(model.phase(), RaceModePhase::AwaitingCancel);
    assert!(matches!(
        model.pop_output(),
        Some(RaceModeOutput::Effect(RaceModeEffect::SetCursorLocked(
            false
        )))
    ));
    assert!(matches!(
        model.pop_output(),
        Some(RaceModeOutput::Request(RaceRequestIntent::Cancel {
            i_start_ecom_id: 9_001,
            ..
        }))
    ));
    assert!(!RACE_RANK_HTTP_TRANSPORT_AVAILABLE);
    assert_eq!(RaceRankModel::default().phase(), RaceRankPhase::Hidden);
}

#[test]
fn npc_icon_dialog_hides_gameplay_hud_until_mode_closes() {
    let mut model = MissionUiModel::default();
    model.npc_icon_mode_visible = true;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<TutorialChoreographyPresentation>()
        .insert_resource(model)
        .add_systems(PostUpdate, sync_tutorial_choreography_visibility);
    let hud = app
        .world_mut()
        .spawn((GameplayHud, Visibility::Inherited))
        .id();

    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Hidden
    );

    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .npc_icon_mode_visible = false;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Inherited
    );
}
