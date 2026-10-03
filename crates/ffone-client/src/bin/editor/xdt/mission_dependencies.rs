//! Mission links always write mission IDs into the target's prerequisites.
use super::*;
use mission_graph::REQUIRE;

pub(super) fn first_rows(rows: &[Value]) -> BTreeMap<i64, usize> {
    let mut first = BTreeMap::new();
    for (row, value) in rows.iter().enumerate() {
        if let Some(id) = value["m_iHMissionID"].as_i64().filter(|id| *id > 0) {
            first.entry(id).or_insert(row);
        }
    }
    first
}

// The server takes the first native row as the mission entry point. Requirements
// repeated on later stages are not additional opening links on the diagram.
pub(super) fn opening_edges(rows: &[Value]) -> Vec<mission_graph::Edge> {
    let first = first_rows(rows);
    mission_graph::edges(rows, false)
        .into_iter()
        .filter(|e| first.get(&e.to) == Some(&e.row))
        .collect()
}

pub(super) fn clear(rows: &mut [Value], row: usize, slot: usize) -> Result<(), String> {
    let value = rows.get(row).ok_or("Missing target mission")?;
    let group = value["m_iHMissionID"].clone();
    let required = value[REQUIRE]
        .get(slot)
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)
        .ok_or("Missing prerequisite slot")?;
    for value in rows.iter_mut().filter(|r| r["m_iHMissionID"] == group) {
        for prerequisite in value[REQUIRE].as_array_mut().into_iter().flatten() {
            if prerequisite.as_i64() == Some(required) {
                *prerequisite = Value::from(0);
            }
        }
    }
    Ok(())
}

impl XdtEditor {
    pub(super) fn connect_canvas(
        &mut self,
        source: usize,
        field: &str,
        target: usize,
    ) -> Result<(), String> {
        if field == REQUIRE {
            self.connect_missions(source, target, None)
        } else {
            self.connect_stage(source, field, target)
        }
    }

    pub(super) fn connect_missions(
        &mut self,
        source: usize,
        target: usize,
        replace: Option<(usize, usize)>,
    ) -> Result<(), String> {
        let mut rows = self.rows().to_vec();
        let from = rows
            .get(source)
            .and_then(|v| v["m_iHMissionID"].as_i64())
            .filter(|id| *id > 0)
            .ok_or("Missing source mission")?;
        let to = rows
            .get(target)
            .and_then(|v| v["m_iHMissionID"].as_i64())
            .filter(|id| *id > 0)
            .ok_or("Missing target mission")?;
        if from == to {
            return Err("Mission prerequisite would create a cycle".into());
        }
        // Work on a copy: a full target, invalid ID or cycle must retain the old link.
        if let Some((row, slot)) = replace {
            if rows
                .get(row)
                .is_some_and(|r| r["m_iHMissionID"].as_i64() == Some(to))
            {
                return Ok(());
            }
            clear(&mut rows, row, slot)?;
        }
        let first = first_rows(&rows)[&to];
        let slots = rows[first][REQUIRE]
            .as_array_mut()
            .ok_or("Missing prerequisite slots")?;
        if !slots.iter().any(|id| id.as_i64() == Some(from)) {
            let slot = slots
                .iter_mut()
                .find(|id| id.as_i64() == Some(0))
                .ok_or("All prerequisite slots are occupied")?;
            *slot = Value::from(from);
        }
        self.change(rows)
    }

    pub(super) fn retarget_canvas_edge(
        &mut self,
        row: usize,
        field: &str,
        slot: usize,
        target: usize,
    ) -> Result<(), String> {
        if field != REQUIRE {
            return self.connect_stage(row, field, target);
        }
        let from = self
            .rows()
            .get(row)
            .and_then(|v| v[REQUIRE].get(slot))
            .and_then(Value::as_i64)
            .ok_or("Missing source mission")?;
        let source = first_rows(self.rows())
            .get(&from)
            .copied()
            .ok_or("Missing source mission")?;
        self.connect_missions(source, target, Some((row, slot)))
    }
}
