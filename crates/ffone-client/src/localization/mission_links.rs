//! Mission labels follow numeric TableData links, including newly authored journals.
use serde_json::Value;

/// Semantic task key -> canonical mission string key. Accepts either the shared
/// server document or the named client view; text content is never an identity.
pub fn mission_text_links(document: &Value) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let tables: Vec<&Value> = if let Some(tables) = document["tables"].as_array() {
        tables.iter().map(|table| &table["value"]).collect()
    } else {
        vec![document]
    };
    for table in tables {
        let missions = &table["m_pMissionTable"];
        for task in missions["m_pMissionData"].as_array().into_iter().flatten() {
            let Some(id) = task["m_iHTaskID"].as_i64().filter(|id| *id > 0) else { continue };
            let mut fields = vec![("title", task["m_iHMissionName"].as_u64()),
                ("objective", task["m_iHCurrentObjective"].as_u64())];
            if let Some(journal) = task["m_iSTJournalIDAdd"].as_u64()
                .and_then(|index| missions["m_pJournalData"].get(index as usize)) {
                for (field, column) in [
                    ("offer_description", "m_iDetaileMissionDesc"),
                    ("task_description", "m_iDetailedTaskDesc"),
                    ("mission_summary", "m_iMissionSummary"),
                    ("mission_complete_summary", "m_iMissionCompleteSummary"),
                    ("completion_description", "m_iDetaileMissionCompleteSummary"),
                ] {
                    fields.push((field, journal[column].as_u64().filter(|index| *index > 0)));
                }
            }
            for (field, index) in fields {
                let Some(index) = index else { continue };
                if missions["m_pMissionStringData"].get(index as usize).is_some() {
                    result.push((format!("content.mission.task.{id}.{field}"),
                        format!("content.tabledata.mission.mission_string.{index}.str_name_string")));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journals_and_shared_titles_follow_ids_in_both_document_formats() {
        let root = serde_json::json!({"m_pMissionTable": {
            "m_pMissionData": [{"m_iHTaskID": 5252, "m_iHMissionName": 1,
                "m_iHCurrentObjective": 2, "m_iSTJournalIDAdd": 1}],
            "m_pJournalData": [{}, {"m_iDetailedTaskDesc": 3}],
            "m_pMissionStringData": [{}, {}, {}, {}]
        }});
        let links = mission_text_links(&root);
        assert!(links.contains(&("content.mission.task.5252.task_description".into(),
            "content.tabledata.mission.mission_string.3.str_name_string".into())));
        assert_eq!(links, mission_text_links(&serde_json::json!({"tables": [{"value": root}]})));
    }
}
