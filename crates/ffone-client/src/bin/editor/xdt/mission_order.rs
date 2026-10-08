//! Explicit entry/ending changes preserve task IDs and splice the normal chain.
use super::*;

impl XdtEditor {
    pub(super) fn set_stage_boundary(
        &mut self,
        selected: usize,
        first: bool,
    ) -> Result<(), String> {
        let mission =
            self.rows().get(selected).ok_or("Missing mission stage")?["m_iHMissionID"].clone();
        let count = self
            .rows()
            .iter()
            .filter(|r| r["m_iHMissionID"] == mission)
            .count();
        self.set_stage_position(selected, if first { 0 } else { count.saturating_sub(1) })
    }
    pub(super) fn set_stage_position(
        &mut self,
        selected: usize,
        position: usize,
    ) -> Result<(), String> {
        if !self.mission_table() {
            return Err("Select a mission task".into());
        }
        let source = self.rows();
        let stage = source.get(selected).ok_or("Missing mission stage")?;
        let mission = stage["m_iHMissionID"]
            .as_i64()
            .filter(|id| *id > 0)
            .ok_or("Select a mission task")?;
        let id = stage["m_iHTaskID"]
            .as_i64()
            .filter(|id| *id > 0)
            .ok_or("Select a mission task")?;
        let slots: Vec<_> = source
            .iter()
            .enumerate()
            .filter(|(_, r)| r["m_iHMissionID"].as_i64() == Some(mission))
            .map(|(i, _)| i)
            .collect();
        let by_id: BTreeMap<_, _> = slots
            .iter()
            .filter_map(|i| source[*i]["m_iHTaskID"].as_i64().map(|id| (id, *i)))
            .collect();
        let original_first = slots[0];
        let mut chain = Vec::new();
        let mut visited = BTreeSet::new();
        let mut row = original_first;
        loop {
            if !visited.insert(row) {
                return Err(
                    "Connect a valid main stage chain before changing its boundaries".into(),
                );
            }
            chain.push(row);
            let next = source[row]["m_iSUOutgoingTask"].as_i64().unwrap_or(0);
            if next <= 0 {
                break;
            }
            row = *by_id
                .get(&next)
                .ok_or("Connect a valid main stage chain before changing its boundaries")?;
        }
        if !chain.contains(&selected)
            && (stage["m_iSUOutgoingTask"].as_i64().unwrap_or(0) > 0
                || slots
                    .iter()
                    .any(|i| source[*i]["m_iSUOutgoingTask"].as_i64() == Some(id)))
        {
            return Err("Select a stage in the main chain or connect it first".into());
        }
        let was_in_chain = chain.contains(&selected);
        chain.retain(|i| *i != selected);
        chain.insert(position.min(chain.len()), selected);
        let new_first = chain[0];
        let mut rows = source.to_vec();
        if was_in_chain {
            let successor = source[selected]["m_iSUOutgoingTask"].clone();
            for i in &slots {
                if rows[*i]["m_iSUOutgoingTask"].as_i64() == Some(id) {
                    rows[*i]["m_iSUOutgoingTask"] = successor.clone();
                }
            }
        }
        for (position, i) in chain.iter().enumerate() {
            rows[*i]["m_iSUOutgoingTask"] = chain
                .get(position + 1)
                .map(|next| source[*next]["m_iHTaskID"].clone())
                .unwrap_or(Value::from(0));
        }
        if new_first != original_first {
            // Entry requirements belong to the mission opening. A continuation
            // usually has no giver or prerequisites and cannot be offered alone.
            for (field, value) in source[original_first].as_object().into_iter().flatten() {
                if field == "m_iHNPCID"
                    || field.starts_with("m_iCST")
                    || field.starts_with("m_iCTR")
                {
                    rows[new_first][field] = value.clone();
                }
            }
            if rows[original_first]["m_iHNPCID"] == rows[new_first]["m_iHNPCID"] {
                rows[original_first]["m_iHNPCID"] = Value::from(0);
            }
        }
        let order: Vec<_> = chain
            .iter()
            .copied()
            .chain(slots.iter().copied().filter(|i| !chain.contains(i)))
            .collect();
        let ordered: Vec<_> = order.iter().map(|i| rows[*i].clone()).collect();
        for (slot, value) in slots.iter().zip(ordered) {
            rows[*slot] = value;
        }
        let new_selected = rows
            .iter()
            .position(|r| r["m_iHTaskID"].as_i64() == Some(id))
            .unwrap();
        let mut next = self.document.clone();
        *next
            .pointer_mut(&self.tables[self.table].pointer)
            .ok_or("Missing mission table")? = Value::Array(rows);
        self.commit_document(next)?;
        self.row = Some(new_selected);
        self.workspace.selected = BTreeSet::from([new_selected]);
        self.workspace.pending_link = None;
        self.workspace.selected_edge = None;
        self.workspace.diagnostics.clear();
        self.workspace.event_locales.clear();
        self.workspace.field_scrolls.clear();
        self.picker_field = None;
        self.focus = None;
        self.rebuild_links();
        self.workspace.view_request = Some(false);
        self.revision += 1;
        Ok(())
    }
}
