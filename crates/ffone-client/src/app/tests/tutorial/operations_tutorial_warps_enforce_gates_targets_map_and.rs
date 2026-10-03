use super::*;

#[test]
fn tutorial_warps_enforce_gates_targets_map_and_one_shot_chain() {
    let content = runtime_test_mission_content();
    let mut runtime = TutorialMissionRuntime::default();
    let mut transform = Transform::default();
    let mut controller = LegacyPlayerController::from_baseline_table();
    let attendant = content.warp(2694).unwrap();
    let portal = content.warp(2695).unwrap();
    let exit = content.warp(2696).unwrap();

    assert!(!tutorial_warp_is_available(attendant, &runtime));
    assert!(!tutorial_warp_is_available(portal, &runtime));
    assert!(tutorial_warp_is_available(exit, &runtime));
    assert!(
        apply_tutorial_warp(
            &content,
            &mut runtime,
            Some(1),
            2696,
            exit.provenance.warp_id,
            exit.required_task_id,
            exit.target,
            &mut transform,
            &mut controller,
        )
        .is_err()
    );
    assert_eq!(transform.translation, Vec3::ZERO);
    assert!(
        apply_tutorial_warp(
            &content,
            &mut runtime,
            Some(0),
            2694,
            attendant.provenance.warp_id + 1,
            attendant.required_task_id,
            attendant.target,
            &mut transform,
            &mut controller,
        )
        .is_err()
    );

    controller.velocity = Vec3::new(3.0, 4.0, 5.0);
    controller.grounded = true;
    controller.jumping = true;
    controller.set_auto_run(true);
    assert!(runtime.start_task(&content, 2251).unwrap());
    apply_tutorial_warp(
        &content,
        &mut runtime,
        Some(0),
        2694,
        attendant.provenance.warp_id,
        attendant.required_task_id,
        attendant.target,
        &mut transform,
        &mut controller,
    )
    .unwrap();
    assert_eq!(runtime.active_tasks, vec![2251]);
    assert!(!runtime.completed_tasks.contains(&2251));
    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(-595.73, -90.66, 745.45), 0.000_1)
    );
    assert!(!controller.grounded);
    assert!(!controller.jumping);
    assert_eq!(controller.velocity, Vec3::ZERO);
    assert!(!controller.is_auto_running());
    assert!(!tutorial_warp_is_available(portal, &runtime));
    let first_trigger = [(
        2376,
        true,
        ProtocolPosition::new([59_500, 74_500, -9_200]).to_native(),
    )];
    assert_eq!(
        tutorial_location_task_completion_candidate(
            &content,
            &runtime,
            transform.translation,
            &first_trigger,
        ),
        Some(2251)
    );
    let first_completion = runtime.complete_task(&content, 2251).unwrap();
    assert_eq!(first_completion.outgoing_task_id, Some(2252));
    assert!(first_completion.outgoing_started);
    assert_eq!(runtime.active_tasks, vec![2252]);
    assert!(tutorial_warp_is_available(portal, &runtime));
    assert!(
        apply_tutorial_warp(
            &content,
            &mut runtime,
            Some(0),
            2694,
            attendant.provenance.warp_id,
            attendant.required_task_id,
            attendant.target,
            &mut transform,
            &mut controller,
        )
        .is_err()
    );

    apply_tutorial_warp(
        &content,
        &mut runtime,
        Some(0),
        2695,
        portal.provenance.warp_id,
        portal.required_task_id,
        portal.target,
        &mut transform,
        &mut controller,
    )
    .unwrap();
    assert_eq!(runtime.active_tasks, vec![2252]);
    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(-596.12, -133.0, 985.67), 0.000_1)
    );

    let second_trigger = [(
        2378,
        true,
        ProtocolPosition::new([58_700, 98_400, -13_300]).to_native(),
    )];
    assert_eq!(
        tutorial_location_task_completion_candidate(
            &content,
            &runtime,
            transform.translation,
            &second_trigger,
        ),
        Some(2252)
    );
    let second_completion = runtime.complete_task(&content, 2252).unwrap();
    assert_eq!(second_completion.outgoing_task_id, Some(2253));
    assert!(second_completion.outgoing_started);
    assert_eq!(runtime.active_tasks, vec![2253]);
    apply_tutorial_warp(
        &content,
        &mut runtime,
        Some(0),
        2696,
        exit.provenance.warp_id,
        exit.required_task_id,
        exit.target,
        &mut transform,
        &mut controller,
    )
    .unwrap();
    assert_eq!(runtime.active_tasks, vec![2253]);
    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(-907.65, 10.43, 715.58), 0.000_1)
    );
    assert!(
        apply_tutorial_warp(
            &content,
            &mut runtime,
            Some(0),
            2696,
            exit.provenance.warp_id,
            exit.required_task_id,
            exit.target,
            &mut transform,
            &mut controller,
        )
        .is_err()
    );
    assert_eq!(runtime.completed_warp_ids, vec![253, 254, 255]);
}

#[test]
fn tutorial_hides_shared_shard_npcs_and_restores_them_in_world() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .add_systems(Update, sync_tutorial_network_npc_visibility);
    let npc = app
        .world_mut()
        .spawn((
            NetworkNpc0104 {
                npc_id: 100,
                npc_type: 2674,
            },
            Visibility::Inherited,
        ))
        .id();

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(npc),
        Some(&Visibility::Hidden)
    );
    assert!(
        app.world()
            .get::<TutorialSuppressedNetworkNpc>(npc)
            .is_some()
    );

    // Lifecycle upserts can reinsert Inherited while the tutorial is live;
    // the presentation owner must suppress that duplicate again next frame.
    app.world_mut()
        .entity_mut(npc)
        .insert(Visibility::Inherited);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(npc),
        Some(&Visibility::Hidden)
    );

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::World);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(npc),
        Some(&Visibility::Inherited)
    );
    assert!(
        app.world()
            .get::<TutorialSuppressedNetworkNpc>(npc)
            .is_none()
    );
}

#[test]
fn tutorial_preupdate_always_suppresses_home_auto_run_toggle() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .insert_resource(tutorial_gate_test_native_mechanics())
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(PreUpdate, suppress_locked_tutorial_ui_shortcuts);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Home);

    app.update();

    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Home)
    );
}

#[test]
fn tutorial_enter_shortcut_uses_the_clean_menu_lock_only() {
    let enter_survives = |stage: TutorialStage, key: KeyCode| {
        let mut native = TutorialNativeMechanics::default();
        native.sync_stable_stage(stage);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<ClientState>()
            .insert_resource(native)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(PreUpdate, suppress_locked_tutorial_ui_shortcuts);
        app.world_mut()
            .resource_mut::<NextState<ClientState>>()
            .set(ClientState::Tutorial);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);

        app.update();

        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(key)
    };

    assert!(enter_survives(
        TutorialStage::Mission(MissionStage::OpenMenu),
        KeyCode::Enter,
    ));
    assert!(enter_survives(
        TutorialStage::Mission(MissionStage::SelectJournal),
        KeyCode::NumpadEnter,
    ));
    assert!(!enter_survives(
        TutorialStage::Mission(MissionStage::NumbuhTwoResponse),
        KeyCode::Enter,
    ));
}

#[test]
fn dexter_ship_spawn_waits_for_its_effect_library_and_remains_single_instance() {
    assert!(!dexter_ship_spawn_ready(
        ClientState::TutorialIntro,
        false,
        false
    ));
    assert!(dexter_ship_spawn_ready(
        ClientState::TutorialIntro,
        true,
        false
    ));
    assert!(!dexter_ship_spawn_ready(
        ClientState::TutorialIntro,
        true,
        true
    ));
    assert!(!dexter_ship_spawn_ready(ClientState::World, true, false));
}
