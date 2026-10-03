use super::*;

#[test]
fn npc_dialog_suppresses_gameplay_chrome_without_erasing_its_base_state() {
    let mut mission_ui = MissionUiModel::default();
    assert!(gameplay_chrome_visible(true, Some(&mission_ui)));
    assert!(!gameplay_chrome_visible(false, Some(&mission_ui)));

    mission_ui.npc_icon_mode_visible = true;
    assert!(!gameplay_chrome_visible(true, Some(&mission_ui)));

    mission_ui.npc_icon_mode_visible = false;
    assert!(gameplay_chrome_visible(true, Some(&mission_ui)));
    assert!(gameplay_chrome_visible(true, None));
}

#[test]
fn journal_tabs_preserve_gui_style_overflow_and_shadow_state() {
    let active = mission_gui_style("FusionFallMissionSkinR", "Tab1").unwrap();
    assert_eq!(
        skin_style_background_rect(JOURNAL_ACTIVE_TAB_RECT, active),
        MissionUiRect::new(597.0, 11.0, 120.0, 32.0)
    );
    assert_eq!(active.states["normal"].background.path_id, 126);
    assert_eq!(active.states["hover"].background.path_id, 210);

    let completed = mission_gui_style("FusionFallMissionSkinR", "Tab2").unwrap();
    assert_eq!(
        skin_style_background_rect(JOURNAL_COMPLETED_TAB_RECT, completed),
        MissionUiRect::new(650.0, 11.0, 160.0, 30.0)
    );
    assert_eq!(completed.states["normal"].background.path_id, 98);
    assert_eq!(completed.states["hover"].background.path_id, 299);

    let window = mission_gui_style("FusionFallMissionSkinR", "window").unwrap();
    assert_eq!(window.states["normal"].background.path_id, 470);
    assert_eq!(window.states["onNormal"].background.path_id, 412);
    assert_eq!(JOURNAL_RIGHT_FRAME, "ui/en/gameplay/journal/shadow.png");
}

#[test]
fn linked_npc_mode_blocks_gameplay_and_keeps_one_subtarget_through_journal() {
    let mut model = MissionUiModel::default();
    assert!(!model.gameplay_input_blocked());
    assert_eq!(model.npc_subtarget_actor_id(), None);

    model.show_npc_interaction(npc_with_available());
    assert!(model.gameplay_input_blocked());
    assert_eq!(model.npc_subtarget_actor_id(), Some(100));

    let mut outbox = GameplayUiOutbox::default();
    assert!(model.select_npc_mission(0, &mut outbox));
    assert!(matches!(model.journal, MissionJournalUi::Allow(_)));
    assert!(model.gameplay_input_blocked());
    assert_eq!(model.npc_subtarget_actor_id(), Some(100));

    assert!(model.close_journal(&mut outbox));
    assert_eq!(model.npc_subtarget_actor_id(), Some(100));
    assert!(
        !outbox
            .drain()
            .any(|action| matches!(action, GameplayUiAction::NpcIconClose { .. })),
        "returning to the NPC menu must not request farewell"
    );
    assert!(model.close_npc_interaction(&mut outbox));
    assert!(!model.close_npc_interaction(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![GameplayUiAction::NpcIconClose { npc_id: 100 }]
    );
    assert!(!model.gameplay_input_blocked());
    assert_eq!(model.npc_subtarget_actor_id(), None);
}

#[test]
fn nanocom_settings_preserves_mode_then_menu_close_event_order() {
    let mut model = MissionUiModel {
        enabled: true,
        ..default()
    };
    let mut outbox = GameplayUiOutbox::default();

    assert!(!model.open_option_from_nanocom(&mut outbox));
    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.open_option_from_nanocom(&mut outbox));
    assert!(!model.nanocom_main_menu_visible);
    assert!(!model.open_option_from_nanocom(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenNanocomMenu,
            GameplayUiAction::OpenOptionFromNanocomSettings,
            GameplayUiAction::CloseNanocomMenu,
        ]
    );
}

#[test]
fn nanocom_email_and_map_preserve_mode_then_close_order() {
    let mut model = MissionUiModel {
        enabled: true,
        ..default()
    };
    let mut outbox = GameplayUiOutbox::default();

    model.notify_new_email();
    assert!(model.nanocom_new_mail_notice_visible);
    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.open_email_from_nanocom(&mut outbox));
    assert!(!model.nanocom_main_menu_visible);
    assert!(!model.nanocom_new_mail_notice_visible);
    assert!(!model.open_email_from_nanocom(&mut outbox));

    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.open_world_map_from_nanocom(&mut outbox));
    assert!(!model.nanocom_main_menu_visible);

    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenNanocomMenu,
            GameplayUiAction::OpenEmailFromNanocom,
            GameplayUiAction::CloseNanocomMenu,
            GameplayUiAction::OpenNanocomMenu,
            GameplayUiAction::OpenWorldMapFromNanocom,
            GameplayUiAction::CloseNanocomMenu,
        ]
    );
}

#[test]
fn completed_tab_uses_final_history_selection_sort_and_expand_contract() {
    let mut active = mission(500, 100);
    active.mission_type = 3;
    let mut first_raw_completed = mission(700, 100);
    first_raw_completed.mission_type = 3;
    first_raw_completed.required_level = 5;
    first_raw_completed.title = "Zulu World".to_owned();
    let mut nano = mission(701, 100);
    nano.mission_type = 2;
    nano.required_level = 2;
    nano.title = "Nano History".to_owned();
    let mut earlier_world = mission(702, 100);
    earlier_world.mission_type = 3;
    earlier_world.required_level = 1;
    earlier_world.title = "Alpha World".to_owned();

    let mut model = MissionUiModel {
        enabled: true,
        journal: MissionJournalUi::Other(JournalOtherUi {
            title: "MISSION JOURNAL".to_owned(),
            active_missions: vec![active],
            ..default()
        }),
        viewed_journal_task_id: Some(500),
        selected_journal_task_id: Some(500),
        ..default()
    };
    model.set_completed_journal_missions(vec![first_raw_completed, nano, earlier_world]);

    assert!(model.select_journal_tab(JournalListTab::Completed));
    assert_eq!(model.viewed_journal_task_id, Some(700));
    assert_eq!(
        current_mission(&model).map(|mission| mission.task_id),
        Some(700),
        "the source selects the first raw completed mission before display sorting"
    );
    let layout = journal_right_layout(&model);
    assert!(layout.completed);
    assert_eq!(layout.category_counts, [1, 0, 2]);
    assert!(layout.empty_rows.is_empty());
    assert_eq!(
        layout
            .rows
            .iter()
            .map(|row| row.mission.task_id)
            .collect::<Vec<_>>(),
        vec![701, 702, 700]
    );
    assert!(layout.rows[2].selected);
    assert!(layout.rows.iter().all(|row| !row.tracked));
    assert_eq!(layout.header_tops, [56.0, 167.0, 207.0]);

    assert!(model.toggle_completed_category(0));
    assert_eq!(model.completed_category_expanded, [false, true, true]);
    let collapsed = journal_right_layout(&model);
    assert_eq!(collapsed.category_counts, [1, 0, 2]);
    assert_eq!(
        collapsed
            .rows
            .iter()
            .map(|row| row.mission.task_id)
            .collect::<Vec<_>>(),
        vec![702, 700]
    );

    assert!(model.select_journal_mission(0));
    assert_eq!(model.viewed_journal_task_id, Some(702));
    assert_eq!(
        model.selected_journal_task_id,
        Some(500),
        "completed history must never retarget the active mission"
    );
    assert!(model.select_journal_tab(JournalListTab::Active));
    assert_eq!(model.viewed_journal_task_id, Some(500));
    assert_eq!(model.completed_category_expanded, [true; 3]);
    assert!(!model.toggle_completed_category(0));
    assert!(model.select_journal_tab(JournalListTab::Completed));
    assert_eq!(model.viewed_journal_task_id, Some(700));
}

#[test]
fn npc_close_respects_end_mode_lock_before_requesting_farewell() {
    use crate::{
        tutorial::{MissionStage, TutorialStage},
        tutorial_logic::TutorialIntent,
    };
    let mut native = TutorialNativeMechanics::default();
    native.sync_stable_stage(TutorialStage::Mission(MissionStage::NumbuhTwoResponse));
    native.apply_intent(TutorialIntent::PopInputFilter);
    native.apply_intent(TutorialIntent::LockUi);
    assert!(!native.is_locked(TutorialInputLock::Mouse));
    let mut model = MissionUiModel::default();
    model.show_npc_interaction(npc_with_available());
    let mut app = App::new();
    app.insert_resource(native)
        .insert_resource(model)
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_mission_ui_buttons);
    let button = app
        .world_mut()
        .spawn((Interaction::Pressed, MissionUiControl::NpcClose))
        .id();
    app.update();
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .npc_icon_mode_visible
    );
    assert!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .next()
            .is_none()
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::ButtonSound]
    );
    app.world_mut()
        .resource_mut::<TutorialNativeMechanics>()
        .apply_intent(TutorialIntent::UnlockUi);
    app.world_mut().entity_mut(button).insert(Interaction::None);
    app.update();
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::NpcIconClose { npc_id: 100 }]
    );
}
