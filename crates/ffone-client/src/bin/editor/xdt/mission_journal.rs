//! Journal IDs are stable array indexes; their six fields reference mission text.
use super::*;

impl XdtEditor {
    pub(super) fn start_journal_edit(&mut self, field: &str) -> Result<(), String> {
        if !is_link(field) { return Err("Select a journal entry".into()); }
        let (table, _) = self.reference_target(field).ok_or("Missing journal table")?;
        let id = self.field_value(field).and_then(Value::as_u64).filter(|id| *id > 0)
            .ok_or("Select a journal entry")? as usize;
        let value = self.document.pointer(&self.tables[table].pointer).and_then(|v| v.get(id))
            .cloned().ok_or("Missing journal entry")?;
        self.start_quick(field)?;
        self.workspace.quick_existing = Some(id);
        self.draft.as_mut().unwrap().value = value;
        self.focus = None;
        self.picker_field = None;
        Ok(())
    }
}

pub(super) const TEXT_FIELDS: &[&str] = &[
    "m_iMissionSummary",
    "m_iDetaileMissionDesc",
    "m_iTaskSummary",
    "m_iDetailedTaskDesc",
    "m_iMissionCompleteSummary",
    "m_iDetaileMissionCompleteSummary",
];

pub(super) fn is_link(field: &str) -> bool {
    matches!(
        field,
        "m_iSTJournalIDAdd" | "m_iSUJournaliDAdd" | "m_iFJournalIDAdd"
    )
}

pub(super) fn record_name(e: &XdtEditor, table: &Table, row: &Value) -> Option<String> {
    if !table.label.ends_with("/m_pJournalData") {
        return None;
    }
    let (group, _) = table.label.rsplit_once('/')?;
    let strings = e
        .tables
        .iter()
        .find(|t| t.label == format!("{group}/m_pMissionStringData"))?;
    let texts = e.document.pointer(&strings.pointer)?;
    let mut parts = Vec::new();
    for field in [
        "m_iMissionSummary",
        "m_iDetailedTaskDesc",
        "m_iDetaileMissionDesc",
        "m_iTaskSummary",
        "m_iMissionCompleteSummary",
        "m_iDetaileMissionCompleteSummary",
    ] {
        let Some(id) = row[field].as_u64().filter(|id| *id > 0) else {
            continue;
        };
        let Some(text) = texts
            .get(id as usize)
            .and_then(|v| v["m_pstrNameString"].as_str())
        else {
            continue;
        };
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !text.is_empty() && !parts.contains(&text) {
            parts.push(text);
        }
        if parts.len() == 2 {
            break;
        }
    }
    (!parts.is_empty()).then(|| parts.join(" — ").chars().take(160).collect())
}
