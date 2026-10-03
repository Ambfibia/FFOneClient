use crate::app::gameplay_ui_actions::*;

#[test]
fn tutorial_quest_pages_remain_open_when_world_actions_are_drained() {
    use ffone_client::mission_ui::{MissionJournalUi, MissionUiEntry, MissionUiModel, NpcInteractionUi};

    for reward in [false, true] {
        let mission = MissionUiEntry {
            task_id: if reward { 2249 } else { 2248 },
            npc_id: Some(1005),
            has_task_reward: reward,
            ..Default::default()
        };
        let mut interaction = NpcInteractionUi { npc_id: 1005, npc_type: 2671, ..Default::default() };
        if reward {
            interaction.completed_missions.push(mission);
        } else {
            interaction.available_missions.push(mission);
        }
        let mut model = MissionUiModel::default();
        model.show_npc_interaction(interaction);
        let mut outbox = GameplayUiOutbox::default();
        assert!(model.select_npc_mission(0, &mut outbox));
        outbox.push(GameplayUiAction::SendChat("world-owned chat".to_owned()));

        // This is the world's production drain predicate during Tutorial.
        // Sending OpenMissionAllow/Reward to that consumer rejects NPC 1005
        // as absent from the network world and clears the freshly opened page.
        assert_eq!(
            outbox.drain_matching(|action| !tutorial_owns_gameplay_action(action)),
            vec![GameplayUiAction::SendChat("world-owned chat".to_owned())],
        );
        let local = outbox.drain_matching(tutorial_owns_gameplay_action);
        assert_eq!(local.len(), 1);
        assert!(if reward {
            matches!(local[0], GameplayUiAction::OpenMissionReward { task_id: 2249, npc_id: 1005 })
                && matches!(model.journal, MissionJournalUi::Reward { .. })
        } else {
            matches!(local[0], GameplayUiAction::OpenMissionAllow { task_id: 2248, npc_id: 1005 })
                && matches!(model.journal, MissionJournalUi::Allow(_))
        });
        assert!(!model.npc_icon_mode_visible);
        assert!(outbox.drain().next().is_none());
    }
}

#[test]
fn routing_preserves_cross_domain_action_order() {
    let routed = [
        GameplayUiAction::WarpAwayStarted,
        GameplayUiAction::ToggleMenuChat,
        GameplayUiAction::OpenNanocomMenu,
        GameplayUiAction::OpenMissionJournal,
        GameplayUiAction::NpcIconClose { npc_id: 42 },
    ]
    .into_iter()
    .map(RoutedGameplayUiAction::from)
    .collect::<Vec<_>>();

    assert!(matches!(routed[0], RoutedGameplayUiAction::Warp(_)));
    assert!(matches!(routed[1], RoutedGameplayUiAction::Chat(_)));
    assert!(matches!(routed[2], RoutedGameplayUiAction::Mode(_)));
    assert!(matches!(routed[3], RoutedGameplayUiAction::Mission(_)));
    assert!(matches!(routed[4], RoutedGameplayUiAction::Npc(_)));
}
