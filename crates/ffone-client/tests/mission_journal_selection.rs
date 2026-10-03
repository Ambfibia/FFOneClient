use bevy::prelude::*;
use ffone_client::{gameplay_ui::GameplayUiOutbox, mission_ui::*};

fn mission(task_id: i32, npc_id: i32) -> MissionUiEntry {
    MissionUiEntry {
        task_id,
        npc_id: Some(npc_id),
        mission_type: 3,
        ..default()
    }
}

#[test]
fn active_journal_selection_survives_close_reopen_and_reordered_refresh() {
    let mut model = MissionUiModel::default();
    model.enabled = true;
    let mut outbox = GameplayUiOutbox::default();
    model.set_active_journal_missions(vec![mission(500, 100), mission(501, 100)]);
    assert!(model.open_journal_from_shortcut(&mut outbox));
    assert!(model.select_journal_mission(1));
    model.selected_journal_task_id = Some(501);
    assert!(model.close_journal(&mut outbox));
    model.set_active_journal_missions(vec![
        mission(502, 100),
        mission(501, 100),
        mission(500, 100),
    ]);
    assert!(model.open_journal_from_shortcut(&mut outbox));
    assert_eq!(model.viewed_journal_task_id, Some(501));
    assert_eq!(model.selected_journal_task_id, Some(501));
    model.set_active_journal_missions(vec![mission(500, 100), mission(501, 100)]);
    assert_eq!(model.viewed_journal_task_id, Some(501));
}

#[test]
fn active_journal_refresh_only_falls_back_when_viewed_quest_disappears() {
    let mut model = MissionUiModel::default();
    model.enabled = true;
    let mut outbox = GameplayUiOutbox::default();
    model.set_active_journal_missions(vec![
        mission(500, 100),
        mission(501, 100),
        mission(502, 100),
    ]);
    model.set_completed_journal_missions(vec![mission(700, 100)]);
    assert!(model.open_journal_from_shortcut(&mut outbox));
    assert!(model.select_journal_mission(1));
    // Browsing remains distinct from the tracked quest for direct model callers.
    assert_eq!(model.selected_journal_task_id, Some(500));
    assert!(model.select_journal_tab(JournalListTab::Completed));
    assert!(model.toggle_completed_category(0));
    model.set_active_journal_missions(vec![
        mission(502, 100),
        mission(501, 100),
        mission(500, 100),
    ]);
    assert_eq!(model.viewed_journal_task_id, Some(700));
    assert_eq!(model.completed_category_expanded, [false, true, true]);
    assert!(model.select_journal_tab(JournalListTab::Active));
    assert_eq!(model.viewed_journal_task_id, Some(501));
    model.set_active_journal_missions(vec![mission(502, 100), mission(500, 100)]);
    assert_eq!(model.viewed_journal_task_id, Some(500));
    assert!(model.begin_task_stop(500));
    assert!(model.confirm_task_stop(500, &[502]));
    assert!(model.close_journal(&mut outbox));
    assert!(model.open_journal_from_shortcut(&mut outbox));
    assert_eq!(model.viewed_journal_task_id, Some(502));
    model.set_active_journal_missions(vec![]);
    assert_eq!(model.viewed_journal_task_id, None);
    assert!(model.nanocom_journal.active_missions.is_empty());
    assert!(model.close_journal(&mut outbox));
    assert!(model.open_journal_from_shortcut(&mut outbox));
    assert_eq!(model.selected_journal_task_id, None);
}

#[test]
fn active_journal_choice_survives_completed_history_and_npc_reward() {
    let mut model = MissionUiModel::default();
    model.enabled = true;
    let mut outbox = GameplayUiOutbox::default();
    model.set_active_journal_missions(vec![mission(500, 100), mission(501, 100)]);
    model.set_completed_journal_missions(vec![mission(700, 100)]);
    assert!(model.open_journal_from_shortcut(&mut outbox));
    assert!(model.select_journal_mission(1));
    assert!(model.select_journal_tab(JournalListTab::Completed));
    assert!(model.close_journal(&mut outbox));
    assert!(model.open_automatic_reward(mission(700, 100)));
    assert!(model.close_journal(&mut outbox));
    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.open_journal_from_nanocom(model.nanocom_journal.clone(), &mut outbox));
    assert_eq!(model.viewed_journal_task_id, Some(501));
}
