use super::*;

#[test]
fn rewardless_npc_steps_of_every_supported_type_skip_the_completion_screen() {
    let root = asset_root();
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();

    // Includes tutorial zone/proximity steps plus ordinary-world object,
    // delivery and escort steps. NpcIconMode keys this behavior only on
    // the current row's reward ID, not on task type or outgoing task ID.
    for task_id in [2253, 2251, 25, 44, 576] {
        let mission = content.mission_entry(task_id, 100, "Test zone").unwrap();
        assert!(!mission.has_task_reward, "task {task_id}");

        let mut app = App::new();
        app.init_resource::<MissionUiModel>()
            .init_resource::<TutorialNativeMechanics>()
            .init_resource::<GameplayUiOutbox>()
            .init_resource::<GameplayUiAudioOutbox>()
            .add_systems(Update, handle_mission_ui_buttons);
        app.world_mut()
            .resource_mut::<MissionUiModel>()
            .show_npc_interaction(NpcInteractionUi {
                npc_id: 100,
                completed_missions: vec![mission],
                ..default()
            });
        app.world_mut()
            .spawn((Interaction::Pressed, MissionUiControl::NpcMissionRow(0)));
        app.update();
        assert!(
            app.world()
                .resource::<MissionUiModel>()
                .npc_icon_mode_visible,
            "task {task_id}"
        );
        assert!(matches!(
            app.world().resource::<MissionUiModel>().journal,
            MissionJournalUi::Hidden
        ));
        assert_eq!(
            app.world_mut()
                .resource_mut::<GameplayUiOutbox>()
                .drain()
                .collect::<Vec<_>>(),
            vec![GameplayUiAction::QuestEnd {
                task_id,
                npc_id: 100,
                box1_choice: 0,
                box2_choice: 0,
            }]
        );
    }
}

#[test]
fn terminal_rewardless_npc_step_also_skips_the_completion_screen() {
    let mut step = mission(703, 100);
    step.task_type = 1;
    step.outgoing_task_id = 0;
    step.has_task_reward = false;
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 100,
        completed_missions: vec![step],
        ..default()
    });

    assert!(model.select_npc_mission(0, &mut outbox));
    assert!(matches!(model.journal, MissionJournalUi::Hidden));
    assert!(matches!(
        model.pending,
        Some(PendingMissionUiRequest::QuestEnd { task_id: 703, .. })
    ));
}

#[test]
fn first_buttercup_nano_talk_submits_task_end_without_the_offer_journal() {
    let root = asset_root();
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let buttercup = content
        .mission_entry(2250, 100, "Pokey Oaks North")
        .unwrap();
    assert_eq!(buttercup.mission_type, 2);
    assert!(buttercup.is_first_mission_task && buttercup.is_intermediate_talk());
    assert!(!buttercup.has_task_reward);

    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 100,
        completed_missions: vec![buttercup],
        ..default()
    });

    // Clean NpcIconMode sends this rewardless TaskEnd directly. The first
    // Nano talk must not open the ACCEPT MISSION journal.
    assert!(model.select_npc_mission(0, &mut outbox));
    assert!(matches!(model.journal, MissionJournalUi::Hidden));
    assert!(model.npc_icon_mode_visible);
    assert!(!model.accept_mission(&mut outbox));
    assert_eq!(
        model.pending,
        Some(PendingMissionUiRequest::QuestEnd {
            task_id: 2250,
            npc_id: 100,
            box1_choice: 0,
            box2_choice: 0,
        })
    );
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::QuestEnd {
            task_id: 2250,
            npc_id: 100,
            box1_choice: 0,
            box2_choice: 0,
        }]
    );
    assert!(model.confirm_quest_end(2250));
    assert!(matches!(model.journal, MissionJournalUi::Hidden));
    assert!(model.npc_icon_mode_visible);
}

#[test]
fn rewarded_terminal_talk_keeps_the_explicit_complete_mission_hand_in() {
    let mut talk = mission(703, 100);
    talk.task_type = 1;
    talk.outgoing_task_id = 0;
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 100,
        completed_missions: vec![talk],
        ..default()
    });

    assert!(model.select_npc_mission(0, &mut outbox));
    assert_eq!(model.pending, None);
    assert!(journal_primary_is_visible(&model));
    assert!(model.complete_mission(&mut outbox));
}

#[test]
fn nanocom_guide_and_exit_do_not_emit_menu_close() {
    let mut model = MissionUiModel {
        enabled: true,
        ..default()
    };
    let mut outbox = GameplayUiOutbox::default();

    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.open_game_guide_from_nanocom(&mut outbox));
    assert!(model.nanocom_main_menu_visible);
    assert!(model.nanocom_foreign_modal_suppressed());
    assert!(!model.nanocom_menu_presented());
    assert!(!model.open_quit_from_nanocom(&mut outbox));

    // Closing GuideMode releases only presentation suppression. The
    // clean NanoCom source latch was never cleared, so the same menu is
    // presented again and can route QuitMenu without an invented reopen.
    model.set_nanocom_foreign_modal_suppressed(false);
    assert!(model.nanocom_menu_presented());
    assert!(model.open_quit_from_nanocom(&mut outbox));
    assert!(model.nanocom_main_menu_visible);
    assert!(model.nanocom_foreign_modal_suppressed());
    assert!(!model.nanocom_menu_presented());

    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenNanocomMenu,
            GameplayUiAction::OpenGameGuideFromNanocom,
            GameplayUiAction::OpenQuitFromNanocom,
        ]
    );
}

#[test]
fn foreign_modal_suppression_blocks_nanocom_controls_and_restores_shared_transition() {
    let model = MissionUiModel {
        enabled: true,
        nanocom_main_menu_visible: true,
        nanocom_foreign_modal_suppressed: true,
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(TutorialNativeMechanics::default())
        .insert_resource(model)
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .init_resource::<GameplayMenuTransition>()
        .add_systems(
            Update,
            (
                handle_mission_ui_buttons,
                sync_gameplay_menu_transition_target,
            )
                .chain(),
        );
    app.world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::NanocomClose));

    app.update();

    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .nanocom_main_menu_visible
    );
    assert!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .next()
            .is_none()
    );
    assert!(!app.world().resource::<GameplayMenuTransition>().open);

    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .set_nanocom_foreign_modal_suppressed(false);
    app.update();

    assert!(app.world().resource::<GameplayMenuTransition>().open);
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .nanocom_menu_presented()
    );
}

#[test]
fn nanocom_lock_matrix_matches_clean_callback_specific_elocks() {
    use crate::tutorial::{MissionStage, TutorialStage};

    let mut native = TutorialNativeMechanics::default();
    native.sync_stable_stage(TutorialStage::Mission(MissionStage::SelectJournal));

    assert!(native.is_locked(TutorialInputLock::Mouse));
    assert!(!tutorial_mission_control_is_locked(
        MissionUiControl::NanocomJournal,
        &native,
    ));
    assert!(!tutorial_mission_control_is_locked(
        MissionUiControl::NanocomClose,
        &native,
    ));
    for control in [
        MissionUiControl::NanocomMyStuff,
        MissionUiControl::NanocomEmail,
        MissionUiControl::NanocomGameGuide,
        MissionUiControl::NanocomMap,
        MissionUiControl::NanocomSettings,
        MissionUiControl::NanocomExitGame,
    ] {
        assert!(
            tutorial_mission_control_is_locked(control, &native),
            "{control:?} must retain its callback-specific lock",
        );
    }

    native.sync_stable_stage(TutorialStage::Mission(MissionStage::NumbuhTwoResponse));
    assert!(tutorial_mission_control_is_locked(
        MissionUiControl::NanocomJournal,
        &native,
    ));
    assert!(!tutorial_mission_control_is_locked(
        MissionUiControl::NanocomClose,
        &native,
    ));
}

#[test]
fn nanocom_journal_other_and_close_have_exact_outbox_order() {
    let mut model = MissionUiModel {
        enabled: true,
        ..default()
    };
    let mut outbox = GameplayUiOutbox::default();

    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.tutorial_observation().nanocom_main_menu_visible);
    assert!(model.open_journal_from_nanocom(
        JournalOtherUi {
            title: "MISSION JOURNAL".to_owned(),
            active_missions: vec![mission(500, 100), mission(501, 100)],
            ..default()
        },
        &mut outbox,
    ));
    assert_eq!(model.viewed_journal_task_id, Some(500));
    assert_eq!(model.selected_journal_task_id, Some(500));
    assert_eq!(
        current_mission(&model).map(|mission| mission.task_id),
        Some(500)
    );
    assert!(model.select_journal_mission(1));
    assert_eq!(model.viewed_journal_task_id, Some(501));
    assert_eq!(
        model.selected_journal_task_id,
        Some(500),
        "browsing an Active-list row must preserve the tracked mission"
    );
    assert_eq!(
        current_mission(&model).map(|mission| mission.task_id),
        Some(501)
    );
    assert!(!model.select_journal_mission(2));
    assert_eq!(
        model.tutorial_observation().journal_mode,
        TutorialJournalMode::Other
    );
    assert!(!model.tutorial_observation().nanocom_main_menu_visible);
    assert!(model.close_journal(&mut outbox));
    assert_eq!(model.viewed_journal_task_id, None);
    assert_eq!(model.selected_journal_task_id, Some(500));
    assert_eq!(model.remembered_active_journal_task_id, Some(501));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenNanocomMenu,
            GameplayUiAction::CloseNanocomMenu,
            GameplayUiAction::OpenMissionJournal,
            GameplayUiAction::CloseMissionJournal,
        ]
    );
}

#[test]
fn task_stop_uses_viewed_not_tracked_and_keeps_other_locked_until_resolution() {
    let mut model = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            title: "MISSION JOURNAL".to_owned(),
            active_missions: vec![mission(500, 100), mission(501, 100), mission(502, 100)],
            ..default()
        }),
        journal_tab: JournalListTab::Active,
        viewed_journal_task_id: Some(501),
        selected_journal_task_id: Some(500),
        npc_icon_mode_visible: false,
        ..default()
    };
    model.nanocom_journal = match &model.journal {
        MissionJournalUi::Other(journal) => journal.clone(),
        _ => unreachable!(),
    };
    let mut outbox = GameplayUiOutbox::default();

    assert!(model.request_task_stop_confirmation(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::RequestTaskStopConfirmation { task_id: 501 }]
    );
    assert!(model.begin_task_stop(501));
    assert!(!model.request_task_stop_confirmation(&mut outbox));
    assert!(!model.select_journal_mission(2));
    assert!(!model.select_journal_tab(JournalListTab::Completed));
    assert!(!model.close_journal(&mut outbox));
    assert_eq!(model.viewed_journal_task_id, Some(501));
    assert_eq!(model.selected_journal_task_id, Some(500));

    assert!(model.reject_pending(501));
    assert_eq!(model.viewed_journal_task_id, Some(501));
    assert_eq!(model.journal_tab, JournalListTab::Active);
    assert!(model.begin_task_stop(501));
    assert!(model.confirm_task_stop(501, &[500, 502]));
    assert!(matches!(model.journal, MissionJournalUi::Other(_)));
    assert_eq!(model.journal_tab, JournalListTab::Active);
    assert_eq!(model.viewed_journal_task_id, Some(500));
    assert_eq!(model.selected_journal_task_id, Some(500));
    assert!(!model.npc_icon_mode_visible);

    let mut fallback = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            active_missions: vec![mission(500, 100), mission(501, 100), mission(502, 100)],
            ..default()
        }),
        viewed_journal_task_id: Some(501),
        selected_journal_task_id: Some(501),
        ..default()
    };
    assert!(fallback.begin_task_stop(501));
    assert!(fallback.confirm_task_stop(501, &[500, 502]));
    assert_eq!(fallback.viewed_journal_task_id, Some(502));
    assert_eq!(fallback.selected_journal_task_id, Some(502));

    let mut empty = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            active_missions: vec![mission(501, 100)],
            ..default()
        }),
        viewed_journal_task_id: Some(501),
        selected_journal_task_id: Some(501),
        ..default()
    };
    assert!(empty.begin_task_stop(501));
    assert!(empty.confirm_task_stop(501, &[]));
    assert_eq!(empty.viewed_journal_task_id, None);
    assert_eq!(empty.selected_journal_task_id, None);
}

#[test]
fn secondary_closes_allow_and_reward_but_tutorial_other_never_requests_stop() {
    use crate::tutorial::{MissionStage, TutorialStage};

    let mut native = TutorialNativeMechanics::default();
    native.sync_stable_stage(TutorialStage::Mission(MissionStage::SelectJournal));
    let model = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            active_missions: vec![mission(500, 100)],
            ..default()
        }),
        viewed_journal_task_id: Some(500),
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(native)
        .insert_resource(model)
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_mission_ui_buttons);
    let button = app
        .world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::JournalSecondary))
        .id();

    app.update();
    assert!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .next()
            .is_none()
    );

    app.world_mut().entity_mut(button).insert(Interaction::None);
    app.update();
    {
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        model.journal = MissionJournalUi::Allow(mission(500, 100));
        model.npc_interaction = Some(npc_with_available());
    }
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    assert!(matches!(
        app.world().resource::<MissionUiModel>().journal,
        MissionJournalUi::Hidden
    ));
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::CloseMissionJournal]
    );

    app.world_mut().entity_mut(button).insert(Interaction::None);
    app.update();
    {
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        model.journal = MissionJournalUi::Reward {
            mission: mission(500, 100),
            box1_choice: 0,
            box2_choice: 0,
        };
        model.npc_interaction = Some(npc_with_available());
    }
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    assert!(matches!(
        app.world().resource::<MissionUiModel>().journal,
        MissionJournalUi::Hidden
    ));
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::CloseMissionJournal]
    );
}

#[test]
fn reward_active_list_keeps_unselected_row_free_of_portrait_decorations() {
    let mut world_mission = mission(451, 2555);
    world_mission.journal_npc_type = 2555;
    world_mission.title = "Spawn Spree".to_owned();
    world_mission.objective = "Deliver item to Computress.".to_owned();
    let reward_model = MissionUiModel {
        enabled: true,
        nanocom_journal: JournalOtherUi {
            active_missions: vec![world_mission.clone()],
            ..default()
        },
        journal: MissionJournalUi::Reward {
            mission: world_mission.clone(),
            box1_choice: 0,
            box2_choice: 0,
        },
        viewed_journal_task_id: Some(451),
        selected_journal_task_id: Some(451),
        ..default()
    };
    let reward_layout = journal_right_layout(&reward_model);
    let reward_row = reward_layout.rows.first().expect("Spawn Spree row");
    assert!(!reward_row.selected);
    assert!(!journal_row_has_selected_decorations(Some(reward_row)));
    assert_eq!(reward_row.rect.width, 330.0);
    assert_eq!(reward_row.rect.height, 56.0);

    let active_model = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            active_missions: vec![world_mission],
            ..default()
        }),
        viewed_journal_task_id: Some(451),
        selected_journal_task_id: Some(451),
        ..default()
    };
    let active_layout = journal_right_layout(&active_model);
    let active_row = active_layout
        .rows
        .first()
        .expect("expanded Spawn Spree row");
    assert!(active_row.selected);
    assert!(journal_row_has_selected_decorations(Some(active_row)));
    assert_eq!(active_row.rect.height, 86.0);
}
