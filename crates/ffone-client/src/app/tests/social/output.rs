use super::*;

#[test]
fn world_login_enqueues_exact_computress_mail_nanocom_copy() {
    let content = runtime_test_mission_content();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    load.as_bytes_mut()[ffone_protocol::PcLoadData0104::MENTOR_OFFSET
        ..ffone_protocol::PcLoadData0104::MENTOR_OFFSET + 2]
        .copy_from_slice(&5_i16.to_le_bytes());
    let mut guide = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    guide.load_pc_state(&load);
    let mut mission = WorldMissionRuntime::default();
    mission.seed(&load, &content).unwrap();
    let nano_bank = NanoFreeTuningBank0104::default();
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let owned_nanos = BTreeSet::new();
    let quest_inventory = inventory.quest_inventory.as_ref().unwrap();
    let mail_task = content
        .missions()
        .find(|candidate| {
            matches!(
                candidate.mission_type,
                TutorialMissionType::Guide | TutorialMissionType::Nano
            ) && mission.can_start_task(candidate, 100, 5, &owned_nanos, quest_inventory, &content)
        })
        .expect("published TableData must have an available Guide/Nano mail task");
    let catalog = EmailProductionCatalog0104 {
        items: BTreeMap::new(),
        guide_rows: vec![EmailGuideTableRow0104 {
            task_id: mail_task.provenance.task_id,
            start_message_type: 0,
            start_sender: 0,
            start_copy: String::new(),
            start_string_id: 0,
            subject_string_id: 0,
            mentor_string_ids: [0; 5],
            subject: mail_task.title.clone(),
            mentor_copy: [None, None, None, None, Some("Computress mail".to_owned())],
        }],
    };
    let mut messages = NanocomMessageUiModel::default();
    assert!(enqueue_login_guide_nanocom_0104(
        &catalog,
        &content,
        &mission,
        &guide,
        &nano_bank,
        &inventory,
        100,
        &mut messages,
    ));
    let active = &messages.active().unwrap().request;
    assert_eq!(active.compact_title_localized().key, "content.npc.730.name");
    assert_eq!(
        active.compact_body_localized(10.0).key,
        "content.tabledata.guide.guide_string.19.sz_string"
    );
    assert_eq!(
        active.compact_icon_path.as_deref(),
        Some(GUIDE_COMPUTRESS_ICON_PATH)
    );
    assert!(
        active
            .voice_true_name
            .as_deref()
            .is_some_and(|name| name.starts_with("Computress_CommOut0"))
    );
}

#[test]
fn confirmed_world_mission_edge_enqueues_source_owned_nanocom_copy() {
    let content = runtime_test_mission_content();
    let mission = content
        .missions()
        .find(|mission| {
            mission.mission_type != TutorialMissionType::Nano
                && mission
                    .start_nanocom_message
                    .as_ref()
                    .is_some_and(|message| {
                        content
                            .gameplay_npc_portrait_icon_path(message.npc_type)
                            .is_some()
                    })
        })
        .expect("published normal mission must own a start NanoCom line");
    let source = mission.start_nanocom_message.as_ref().unwrap();
    let event =
        WorldMissionServerEvent0104::TaskStartSuccess(ffone_protocol::PcTaskStartSuccess0104 {
            task_id: mission.provenance.task_id,
            remaining_time: 0,
        });
    let mut messages = NanocomMessageUiModel::default();
    assert!(enqueue_world_mission_nanocom(
        &event,
        &content,
        &mut messages
    ));
    let active = &messages.active().unwrap().request;
    assert_eq!(
        active.compact_title_localized().key,
        format!("content.npc.{}.name", source.npc_type)
    );
    assert_eq!(
        active.compact_body_localized(10.0).key,
        format!(
            "content.tabledata.mission.mission_string.{}.str_name_string",
            source.string_id
        )
    );
}
