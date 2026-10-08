//! Mission-level marker policy, applied to every stage in one undo step.
use super::*;

pub(super) const FIELD: &str = "m_iHMissionVisibility";

pub(super) fn propagate(before: &[Value], after: &mut [Value]) -> Result<(), String> {
    let mut policies = BTreeMap::new();
    for row in after.iter() {
        let Some(value) = row.get(FIELD) else {
            continue;
        };
        let old = before
            .iter()
            .find(|old| old["m_iHTaskID"] == row["m_iHTaskID"]);
        if old.is_some_and(|old| old.get(FIELD) == Some(value)) {
            continue;
        }
        let mission = row["m_iHMissionID"].as_i64().ok_or("Missing mission ID")?;
        let policy = value
            .as_i64()
            .filter(|n| (0..=3).contains(n))
            .ok_or("Invalid mission secrecy")?;
        if policies
            .insert(mission, policy)
            .is_some_and(|previous| previous != policy)
        {
            return Err(format!("Mission {mission} has conflicting secrecy changes"));
        }
    }
    for row in after {
        if let Some(policy) = row["m_iHMissionID"]
            .as_i64()
            .and_then(|id| policies.get(&id))
        {
            row[FIELD] = Value::from(*policy);
        }
    }
    Ok(())
}

impl XdtEditor {
    pub(super) fn set_mission_visibility(&mut self, value: i64) -> Result<(), String> {
        if !(0..=3).contains(&value) || !self.mission_table() {
            return Err("Invalid mission secrecy".into());
        }
        if let Some(draft) = &mut self.draft {
            draft.value[FIELD] = Value::from(value);
        } else {
            let row = self.row.ok_or("Select a mission stage")?;
            let mut rows = self.rows().to_vec();
            rows[row][FIELD] = Value::from(value);
            self.change(rows)?;
        }
        self.focus = None;
        self.picker_field = None;
        self.revision += 1;
        Ok(())
    }
}
