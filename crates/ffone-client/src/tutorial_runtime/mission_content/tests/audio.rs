use super::*;

#[test]
fn gameplay_npc_referenced_empty_move_voice_is_a_valid_silent_owner() {
    let mut document = compact_document();
    npc_rows_mut(&mut document)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iComment"] = json!(7);
    document["tables"][0]["value"]["m_pNpcTable"]["m_pNpcStringData"][7]["m_strComment2"] =
        json!("");

    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();
    assert_eq!(content.gameplay_npc(664).unwrap().move_voice_owner, "");

    let mut missing_row = compact_document();
    npc_rows_mut(&mut missing_row)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iComment"] = json!(999);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing_row).assets)
            .unwrap_err()
            .to_string()
            .contains("references missing string row 999")
    );
}

#[test]
fn gameplay_minimap_sound_gate_is_required_and_contradictions_fail_closed() {
    let mut missing = compact_document();
    npc_rows_mut(&mut missing)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("m_iSound");
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing).assets)
            .unwrap_err()
            .to_string()
            .contains("has no m_iSound")
    );

    let mut contradictory = compact_document();
    let row = npc_rows_mut(&mut contradictory)
        .iter()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap();
    let mut duplicate = row.clone();
    duplicate["m_iSound"] = json!(2);
    npc_rows_mut(&mut contradictory).push(duplicate);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(contradictory).assets)
            .unwrap_err()
            .to_string()
            .contains("contradictory m_iSound rows for gameplay NPC type 664")
    );
}
