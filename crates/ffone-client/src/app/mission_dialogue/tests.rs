use super::*;
use ffone_protocol::{
    PcTaskEndSuccess0104, PcTaskFailure0104, PcTaskStartSuccess0104, RewardItemReply0104,
};

#[test]
fn inventory_full_notice_survives_missing_npc_and_uses_existing_localized_popup() {
    let content = TutorialMissionContent::open(
        &AssetLocator::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut mission = WorldMissionRuntime::default();
    mission
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: 451,
                remaining_time: 0,
            }),
            &content,
        )
        .unwrap();
    mission
        .record_request(PendingMissionUiRequest::QuestEnd {
            task_id: 451,
            npc_id: 2555,
            box1_choice: 0,
            box2_choice: 0,
        })
        .unwrap();
    mission
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndFailure(PcTaskFailure0104 {
                task_id: 451,
                error_code: 13,
            }),
            &content,
        )
        .unwrap();
    let mut app = App::new();
    app.insert_resource(content)
        .insert_resource(mission)
        .init_resource::<NpcBarkerBubbleRuntime>()
        .init_resource::<GameplayAudioRuntime>()
        .init_resource::<SystemMessageUiModel>()
        .add_systems(Update, present_world_mission_dialogue);
    app.update();
    let popup = app
        .world()
        .resource::<SystemMessageUiModel>()
        .current()
        .unwrap();
    assert_eq!(
        popup.localized.key,
        "content.tabledata.message.message.12.sz_string"
    );
    assert_eq!(popup.text, "Inventory is full.");
    app.update();
    assert_eq!(app.world().resource::<SystemMessageUiModel>().len(), 1);
}

#[test]
fn mission_voice_uses_table_owner_and_only_terminal_completion() {
    let content = TutorialMissionContent::open(
        &AssetLocator::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    )
    .unwrap();
    for task in content.missions() {
        assert_eq!(mission_voice(task, WorldMissionDialogueEdge::Success), None);
        assert_eq!(mission_voice(task, WorldMissionDialogueEdge::Failure), None);
        assert_eq!(
            mission_voice(task, WorldMissionDialogueEdge::Complete),
            Some((
                task.provenance.terminator_npc_type,
                LegacyNpcVoiceCue::QuestCompleted
            ))
        );
        assert_eq!(
            mission_voice(task, WorldMissionDialogueEdge::Start),
            Some((
                task.provenance.start_npc_type,
                if task.provenance.task_type == 6 {
                    LegacyNpcVoiceCue::QuestStartEscort
                } else {
                    LegacyNpcVoiceCue::QuestAccepted
                }
            ))
        );
    }
}

#[test]
fn full_level_four_fusion_matter_accepts_reward_and_end_after_inventory_retry() {
    let mut world = World::new();
    let mut npc_query =
        bevy::ecs::system::SystemState::<Query<&NetworkNpcAppearance0104>>::new(&mut world);
    let npcs = npc_query.get(&world).unwrap();
    let content = TutorialMissionContent::open(
        &AssetLocator::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    )
    .unwrap();
    let task_id = 451;
    let mission_id = content.mission(task_id).unwrap().provenance.mission_id;
    let mut mission = WorldMissionRuntime::default();
    mission
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id,
                remaining_time: 0,
            }),
            &content,
        )
        .unwrap();
    let mut ui = MissionUiModel::default();
    assert!(ui.open_automatic_reward(content.automatic_reward_entry(task_id, 2555).unwrap()));
    let mut outbox = GameplayUiOutbox::default();
    assert!(ui.complete_mission(&mut outbox));
    mission.record_request(ui.pending.unwrap()).unwrap();
    let bridge = NetworkBridge::start();
    let mut inventory = LocalInventoryRuntime {
        snapshot: Some(InventoryRuntime0104::from_pc_load(
            77,
            &ffone_protocol::PcLoadData0104::zeroed(),
        )),
        quest_inventory: Some(
            [ItemBase0104 {
                item_type: 0,
                item_id: 0,
                option: 0,
                time_limit: 0,
            }; ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT],
        ),
    };
    let mut messages = NanocomMessageUiModel::default();
    let mut audio = GameplayAudioRuntime::default();
    let cap = legacy_avatar_max_fusion_matter(4, 0);
    assert!(cap > 0);
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_level: 4,
            fusion_matter: cap,
            max_fusion_matter: cap,
            ..default()
        },
        ..default()
    };
    let failure = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_TASK_END_FAIL,
        flags: 0,
        checksum: 0,
        payload: PcTaskFailure0104 {
            task_id,
            error_code: 13,
        }
        .encode(),
    };
    assert!(
        apply_world_mission_network_frame(
            &failure,
            &bridge,
            &content,
            &npcs,
            &mut inventory,
            None,
            false,
            &mut mission,
            &mut ui,
            &mut messages,
            &mut audio,
            &mut runtime
        )
        .unwrap()
    );
    assert_eq!(mission.take_inventory_full_notices(), vec![task_id]);
    assert!(ui.pending.is_none());
    assert!(mission.active_task_ids().contains(&task_id));
    assert!(ui.complete_mission(&mut outbox));
    mission.record_request(ui.pending.unwrap()).unwrap();
    let reward = RewardItemReply0104 {
        candy: 30,
        fusion_matter: cap,
        nano_battery: 0,
        weapon_battery: 0,
        pack_padding: [0; 3],
        fatigue: 0,
        fatigue_level: 0,
        npc_type_id: 0,
        task_id,
        items: vec![ffone_protocol::ItemReward0104 {
            inventory_location: 1,
            slot: 0,
            item: ItemBase0104 {
                item_type: 5,
                item_id: 24,
                option: 1,
                time_limit: 0,
            },
        }],
    };
    for frame in [
        DecodedFrame {
            packet_type: packet::P_FE2CL_REP_REWARD_ITEM,
            flags: 0,
            checksum: 0,
            payload: reward.encode().unwrap(),
        },
        DecodedFrame {
            packet_type: packet::P_FE2CL_REP_PC_TASK_END_SUCC,
            flags: 0,
            checksum: 0,
            payload: PcTaskEndSuccess0104 { task_id }.encode(),
        },
    ] {
        assert!(
            apply_world_mission_network_frame(
                &frame,
                &bridge,
                &content,
                &npcs,
                &mut inventory,
                None,
                false,
                &mut mission,
                &mut ui,
                &mut messages,
                &mut audio,
                &mut runtime
            )
            .unwrap()
        );
    }
    assert_eq!(runtime.fusion_matter, cap);
    assert_eq!(runtime.candy, 30);
    assert_eq!(inventory.snapshot().unwrap().inventory()[0].item_id, 24);
    assert!(ui.pending.is_none());
    assert!(!mission.active_task_ids().contains(&task_id));
    assert!(mission.completed_mission_ids().contains(&mission_id));
}

#[test]
fn world_reward_currency_gains_reach_chat_only_with_event_messages() {
    let mut world = World::new();
    let mut npc_query =
        bevy::ecs::system::SystemState::<Query<&NetworkNpcAppearance0104>>::new(&mut world);
    let npcs = npc_query.get(&world).unwrap();
    let content = TutorialMissionContent::open(
        &AssetLocator::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    )
    .unwrap();
    let bridge = NetworkBridge::start();
    let mut inventory = LocalInventoryRuntime {
        snapshot: Some(InventoryRuntime0104::from_pc_load(
            77,
            &ffone_protocol::PcLoadData0104::zeroed(),
        )),
        quest_inventory: Some(
            [ItemBase0104 {
                item_type: 0,
                item_id: 0,
                option: 0,
                time_limit: 0,
            }; ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT],
        ),
    };
    let mut mission = WorldMissionRuntime::default();
    let mut ui = MissionUiModel::default();
    let mut messages = NanocomMessageUiModel::default();
    let mut audio = GameplayAudioRuntime::default();
    let mut notices = ffone_client::gameplay_ui::rewards::RewardNotices::default();
    let mut runtime = RuntimeStatus::default();
    runtime.candy = 100;
    runtime.fusion_matter = 40;
    // A mob kill reward: absolute post-state totals and no item.
    for (event_messages, candy, fusion_matter) in
        [(true, 125, 52), (false, 150, 60), (true, 150, 60)]
    {
        let frame = DecodedFrame {
            packet_type: packet::P_FE2CL_REP_REWARD_ITEM,
            flags: 0,
            checksum: 0,
            payload: RewardItemReply0104 {
                candy,
                fusion_matter,
                nano_battery: 0,
                weapon_battery: 0,
                pack_padding: [0; 3],
                fatigue: 100,
                fatigue_level: 1,
                npc_type_id: 0,
                task_id: 0,
                items: Vec::new(),
            }
            .encode()
            .unwrap(),
        };
        assert!(
            apply_world_mission_network_frame(
                &frame,
                &bridge,
                &content,
                &npcs,
                &mut inventory,
                Some(&mut notices),
                event_messages,
                &mut mission,
                &mut ui,
                &mut messages,
                &mut audio,
                &mut runtime
            )
            .unwrap()
        );
    }
    let lines = &runtime.chat.world_chat_lines;
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].text, "You earned 25 Taros for a total 125 taros.");
    assert_eq!(lines[0].kind, ChatLineKind::Receive);
    let taros = lines[0].localized.as_ref().unwrap();
    assert_eq!(taros.key, "ui.hud.chat.reward.taros");
    assert_eq!(taros.args["amount"], "25");
    assert_eq!(taros.args["total"], "125");
    assert_eq!(
        lines[1].text,
        "You collected 12 Fusion Matter for a total 52 Fusion Matter."
    );
    assert_eq!(
        lines[1].localized.as_ref().unwrap().key,
        "ui.hud.chat.reward.fusion_matter"
    );
    assert_eq!((runtime.candy, runtime.fusion_matter), (150, 60));
}

#[test]
fn tutorial_bubbles_speak_only_outside_the_cutscene_their_edge_opens() {
    use ffone_client::tutorial::InfectionStage;

    let content = TutorialMissionContent::open(
        &AssetLocator::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    )
    .unwrap();
    let bubble = |task_id: i32, stage: InfectionStage| {
        tutorial_mission_bubble(
            content.mission(task_id).unwrap(),
            WorldMissionDialogueEdge::Complete,
            TutorialStage::Infection(stage).metadata().scene,
        )
        .map(|dialogue| (dialogue.npc_type, dialogue.string_id))
    };
    // Buttercup's 2250 line is voiced in the interactive ButtercupDelay.
    assert_eq!(
        bubble(2250, InfectionStage::ButtercupDelay),
        Some((2672, 11684))
    );
    // Dexter's 2253 edge opens InfectionB, which subtitles the same line.
    assert_eq!(bubble(2253, InfectionStage::DexterCutscene), None);
    for task_id in [2248, 2249, 2251, 2252, 2254] {
        assert_eq!(bubble(task_id, InfectionStage::ButtercupDelay), None);
    }
}

#[test]
fn tutorial_task_edges_reach_the_nanocom_including_outgoing_starts() {
    #[derive(Clone, Copy)]
    enum Edge {
        Start(i32),
        End(i32),
    }
    let content = TutorialMissionContent::open(
        &AssetLocator::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut app = App::new();
    app.insert_resource(content)
        .init_resource::<TutorialMissionRuntime>()
        .init_resource::<TutorialSession>()
        .init_resource::<NanocomMessageUiModel>()
        .init_resource::<NpcBarkerBubbleRuntime>()
        .add_systems(Update, present_tutorial_mission_dialogue);
    // The Oil Ogre kill ends 2248; 2251 and 2252 chain into Dexter's calls.
    for edge in [
        Edge::Start(2248),
        Edge::End(2248),
        Edge::End(2249),
        Edge::Start(2250),
        Edge::Start(2250),
        Edge::End(2250),
        Edge::End(2251),
        Edge::End(2252),
        Edge::End(2253),
    ] {
        app.world_mut()
            .resource_scope(|world, mut runtime: Mut<TutorialMissionRuntime>| {
                let content = world.resource::<TutorialMissionContent>();
                match edge {
                    Edge::Start(task_id) => {
                        runtime.start_task(content, task_id).unwrap();
                    }
                    Edge::End(task_id) => {
                        runtime.complete_task(content, task_id).unwrap();
                    }
                }
            });
        app.update();
    }

    assert!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .dialogue_edges
            .is_empty()
    );
    let mission_string =
        |id: i32| format!("content.tabledata.mission.mission_string.{id}.str_name_string");
    let queued = app
        .world()
        .resource::<NanocomMessageUiModel>()
        .queued()
        .iter()
        .map(|queued| {
            (
                queued.request.kind,
                queued.request.compact_title_localized().key,
                queued.request.compact_body_localized(10.0).key,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        queued,
        vec![
            (
                NanocomMessageKind::Npc,
                "content.npc.2671.name".to_owned(),
                mission_string(11673)
            ),
            (
                NanocomMessageKind::Nano,
                ffone_client::nanocom_message_ui::NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY
                    .to_owned(),
                mission_string(11683)
            ),
            (
                NanocomMessageKind::Npc,
                "content.npc.2673.name".to_owned(),
                mission_string(11689)
            ),
            (
                NanocomMessageKind::Npc,
                "content.npc.2673.name".to_owned(),
                mission_string(11692)
            ),
        ],
        "each accepted tutorial edge presents its NanoCom row exactly once"
    );
}
