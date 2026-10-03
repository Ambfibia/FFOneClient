use super::*;

#[test]
fn mission_failure_outgoing_and_final_serialized_row_remain_source_owned() {
    let mut document = compact_document();
    let rows = mission_rows_mut(&mut document);
    rows.iter_mut()
        .find(|row| row["m_iHTaskID"] == json!(2248))
        .unwrap()["m_iFOutgoingTask"] = json!(2250);
    // Clean IsFinalTask is based on child/source order even when the last
    // row still owns a positive (and potentially dangling) success edge.
    rows.iter_mut()
        .find(|row| row["m_iHTaskID"] == json!(2254))
        .unwrap()["m_iSUOutgoingTask"] = json!(99_999);

    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();
    assert_eq!(content.failure_outgoing_task_id(2248).unwrap(), Some(2250));
    assert_eq!(content.failure_outgoing_task_id(2249).unwrap(), None);
    assert!(content.is_first_serialized_task(2250).unwrap());
    assert!(!content.is_first_serialized_task(2253).unwrap());
    assert!(!content.is_first_serialized_task(2254).unwrap());
    assert!(!content.is_final_serialized_task(2253).unwrap());
    assert!(content.is_final_serialized_task(2254).unwrap());
    assert_eq!(content.outgoing_task_id(2254).unwrap(), Some(99_999));
}
