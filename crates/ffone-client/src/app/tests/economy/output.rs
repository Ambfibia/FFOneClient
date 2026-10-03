use super::*;

#[test]
fn combi_attempt_modal_matches_source_table_and_semantic_copy() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let assets = AssetLocator::open(root).unwrap();
    let bytes = assets.read(COMBI_RECIPE_TABLE_PATH).unwrap();
    let document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let consolidated = document["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|table| table["name"] == "npc_imports_consolidated")
        .unwrap();
    let source = &consolidated["value"]["m_pMessageTable"]["m_pMessageData"][254];
    let localized = CombiSystemModal0104::AttemptConfirmation.localized_text();
    assert_eq!(source["m_iButtonType"].as_i64(), Some(15));
    assert_eq!(
        SystemMessageButtonType::try_from(15),
        Ok(SystemMessageButtonType::CombinationConfirm)
    );
    assert_eq!(
        source["m_szString"].as_str(),
        Some(localized.fallback.as_str())
    );
    assert_eq!(localized.key, "ui.combi.modal.attempt_confirmation");
}

#[test]
fn email_active_npc_letters_use_start_copy_without_requiring_mentor_email() {
    let content = runtime_test_mission_content();
    let catalog = runtime_test_email_catalog_0104();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&5_i16.to_le_bytes());
    let mut guide = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    guide.load_pc_state(&load);
    // Dexter's task 198 has no mentor invitation in any column. Task 255
    // has mentor copy but no start-mail flag and must not become active mail.
    let messages = email_guide_messages_0104(&catalog, &content, &guide, [198, 255], []).unwrap();
    let [message] = messages.as_slice() else {
        panic!("expected Dexter's active letter: {messages:?}");
    };
    assert_eq!(message.mode, 1);
    assert_eq!(message.mission_task_id, 198);
    assert_eq!(message.sender_npc_id, 728);
    let row = catalog
        .guide_rows
        .iter()
        .find(|row| row.task_id == 198)
        .unwrap();
    assert!(row.mentor_copy.iter().all(Option::is_none));
    assert_eq!(message.content, row.start_copy);
    assert_eq!(message.content_string_id, row.start_string_id);
    assert!(
        email_guide_messages_0104(&catalog, &content, &guide, [], [])
            .unwrap()
            .is_empty()
    );
}
