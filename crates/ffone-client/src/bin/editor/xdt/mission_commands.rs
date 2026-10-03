//! Data mutations are checked before committing one document history entry.
use super::*;
use mission_workspace::{Command, Context};

pub(super) fn next_id(rows: &[Value], key: &str, max: i64) -> Result<i64, String> {
    let next = rows.iter().filter_map(|r| r[key].as_i64()).max().unwrap_or(0)
        .checked_add(1).ok_or("ID range exhausted")?;
    if next > max { return Err("ID range exhausted".into()); }
    Ok(next.max(1))
}

impl XdtEditor {
    pub(super) fn arrange_mission_canvas(&mut self)->Result<(),String>{
        let visible:BTreeSet<_>=mission_canvas::scene(self).nodes.iter().map(|node|node.layout_key(self)).collect();
        let before=self.workspace.positions.clone();
        self.workspace.positions.retain(|key,_|!visible.contains(key));
        if before!=self.workspace.positions{
            self.undo.push(Change{pointer:"@mission-layout".into(),before:serde_json::to_value(before).unwrap(),after:serde_json::to_value(&self.workspace.positions).unwrap()});
            self.redo.clear();self.workspace.layout_dirty=true;self.save_layout()?;
        }
        self.workspace.view_request=Some(true);
        Ok(())
    }
    pub(super) fn validate_document_change(&self, before: &Value, next: &Value) -> Result<(), String> {
        for table in &self.tables {
            let Some(after) = next.pointer(&table.pointer).and_then(Value::as_array) else { continue };
            let old = before.pointer(&table.pointer).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
            if old == after { continue; }
            validate_rows(old, after)?;
            let identity = schema::identity(&table.label);
            for (i, value) in after.iter().enumerate() {
                let original = identity.and_then(|id| old.iter().find(|v| v.get(id) == value.get(id)))
                    .or_else(|| if identity.is_none() { old.get(i) } else { None });
                for field in schema::required(&table.label) {
                    if value.get(*field).is_none() && (original.is_none() || original.is_some_and(|v| v.get(*field).is_some())) {
                        return Err(format!("Required parameters: {field}"));
                    }
                }
                for (field, v) in value.as_object().into_iter().flatten() {
                    if original.and_then(|v| v.get(field)) != Some(v) {
                        if let Some(error) = schema::invalid(&table.label, field, v) { return Err(format!("{field}: {error}")); }
                    }
                }
            }
            if table.label.ends_with("/m_pMissionData") { mission_graph::validate(old, after)?; }
        }
        relations::validate_changes(before, next)
    }
    pub(super) fn commit_document(&mut self, next: Value) -> Result<(), String> {
        self.validate_document_change(&self.document, &next)?;
        if self.document == next { return Ok(()); }
        self.undo.push(Change { pointer:String::new(), before:self.document.clone(), after:next.clone() });
        self.redo.clear();
        self.document = next;
        self.reindex();
        self.refresh();
        Ok(())
    }
    pub(super) fn append_mission(&mut self, new_group: bool, duplicate: bool, name: &str, objective: &str) -> Result<usize, String> {
        if !self.mission_table() { return Err("Select a mission task".into()); }
        let source_row = self.workspace.pending_link.as_ref()
            .filter(|(_, field)| field != mission_graph::REQUIRE && !new_group)
            .map(|(row, _)| *row).or(self.row)
            .or_else(|| self.rows().iter().position(|r| r["m_iHTaskID"].as_i64().is_some_and(|id| id > 0)));
        let template = source_row.and_then(|r| self.rows().get(r))
            .ok_or("Missing mission schema")?.clone();
        let task_id = next_id(self.rows(), "m_iHTaskID", i32::MAX as i64)?;
        let mission_id = if new_group { next_id(self.rows(), "m_iHMissionID", 2048)? } else {
            template["m_iHMissionID"].as_i64().filter(|id| *id > 0).ok_or("Select a mission task")?
        };
        let mut value = if duplicate { template.clone() } else { mission_fields::defaults(&template) };
        if !new_group && !duplicate {
            for (field, previous) in template.as_object().into_iter().flatten() {
                if authoring::group(&self.tables[self.table].label, field) == Some("identity") || field == "m_iHJournalNPCID" {
                    value[field] = previous.clone();
                }
            }
        }
        value["m_iHTaskID"] = Value::from(task_id);
        value["m_iHMissionID"] = Value::from(mission_id);
        value["m_iSUOutgoingTask"] = Value::from(0);
        value["m_iFOutgoingTask"] = Value::from(0);
        let (strings, _) = self.reference_target("m_iHMissionName").ok_or("Missing text table")?;
        let mut next = self.document.clone();
        let texts = next.pointer_mut(&self.tables[strings].pointer).and_then(Value::as_array_mut).ok_or("Missing text table")?;
        let text_template = texts.first().cloned().ok_or("Missing text schema")?;
        if new_group {
            let mut text = text_template.clone();
            for value in text.as_object_mut().into_iter().flat_map(|v| v.values_mut()) { if value.is_string() { *value = Value::from(""); } }
            text["m_pstrNameString"] = Value::from(name);
            value["m_iHMissionName"] = Value::from(texts.len());
            texts.push(text);
        } else { value["m_iHMissionName"] = template["m_iHMissionName"].clone(); }
        if !duplicate {
            let mut text = text_template;
            for v in text.as_object_mut().into_iter().flat_map(|v| v.values_mut()) { if v.is_string() { *v = Value::from(""); } }
            text["m_pstrNameString"] = Value::from(objective);
            value["m_iHCurrentObjective"] = Value::from(texts.len());
            texts.push(text);
        }
        let rows = next.pointer_mut(&self.tables[self.table].pointer).and_then(Value::as_array_mut).ok_or("Missing mission table")?;
        let row = rows.len();
        rows.push(value);
        if let Some((source, field)) = self.workspace.pending_link.clone() {
            if field == mission_graph::REQUIRE && new_group {
                rows[row][&field][0] = rows[source]["m_iHMissionID"].clone();
            } else if field == mission_graph::REQUIRE {
                return Err("Select a target mission".into());
            } else if !new_group {
                let old = rows[source][&field].clone();
                rows[source][&field] = Value::from(task_id);
                rows[row][&field] = old;
            }
        } else if !new_group && !duplicate {
            let source = source_row.ok_or("Select a mission task")?;
            rows[row]["m_iSUOutgoingTask"] = rows[source]["m_iSUOutgoingTask"].clone();
            rows[source]["m_iSUOutgoingTask"] = Value::from(task_id);
        }
        self.commit_document(next)?;
        self.workspace.pending_link = None;
        self.workspace.selected_edge = None;
        self.workspace.expanded.insert(mission_id);
        self.row = Some(row);
        self.mission_filter = None;
        self.search.clear();
        self.workspace.selected = BTreeSet::from([row]);
        self.workspace.view_request=Some(false);
        self.refresh();
        Ok(row)
    }
    pub(super) fn connect_stage(&mut self, source: usize, field: &str, target: usize) -> Result<(), String> {
        let mut rows = self.rows().to_vec();
        let to = rows.get(target).ok_or("Missing target stage")?;
        let id = to["m_iHTaskID"].as_i64().filter(|id| *id > 0).ok_or("Missing target stage")?;
        let group = to["m_iHMissionID"].clone();
        let from = rows.get_mut(source).ok_or("Missing source stage")?;
        if !matches!(field, "m_iSUOutgoingTask" | "m_iFOutgoingTask") { return Err("Unsupported graph edge".into()); }
        if from["m_iHMissionID"] != group { return Err("Transitions must stay within the same mission".into()); }
        if source == target && field == "m_iSUOutgoingTask" { return Err("Success cannot return to the same stage".into()); }
        from[field] = Value::from(id);
        let mut next = self.document.clone();
        *next.pointer_mut(&self.tables[self.table].pointer).unwrap() = Value::Array(rows);
        self.commit_document(next)
    }
    pub(super) fn delete_mission_stage(&mut self) -> Result<(), String> {
        let row = self.row.ok_or("Select a row")?;
        let mut next = self.document.clone();
        let rows = next.pointer_mut(&self.tables[self.table].pointer).and_then(Value::as_array_mut).ok_or("Missing mission table")?;
        let id = rows[row]["m_iHTaskID"].clone();
        for value in rows.iter_mut() {
            for field in ["m_iSUOutgoingTask", "m_iFOutgoingTask", "m_iCSTTrigger"] {
                if value.get(field) == Some(&id) { value[field] = Value::from(0); }
            }
        }
        rows.remove(row);
        self.commit_document(next)?;
        self.row = None;
        self.workspace.selected.clear();
        self.delete_confirm = false;
        Ok(())
    }
    pub(super) fn mission_command(&mut self, command: Command, l: &Localization, lang: &Language) -> Result<(), String> {
        use Command::*;
        let tr = |key: &str, en: &str| presentation::tr(l, lang, key, en);
        let anchor = self.workspace.context.take().map(|(at, _)| at).unwrap_or(Vec2::new(260., 150.));
        match command {
            NewMission | NewStage | Duplicate | InsertStage => {
                if matches!(command, InsertStage) {
                    let (source, field, _) = self.workspace.selected_edge.clone().ok_or("Select a transition")?;
                    self.workspace.pending_link = Some((source, field));
                }
                self.append_mission(matches!(command, NewMission), matches!(command, Duplicate),
                    &tr("mission.new_title", "New mission"), &tr("mission.new_objective", "Complete the conversation"))?;
                self.graph_stages = true;
            }
            ToggleMission(id) => {
                if !self.workspace.expanded.remove(&id) { self.workspace.expanded.insert(id); }
                self.row=self.rows().iter().position(|v|v["m_iHMissionID"].as_i64()==Some(id));
                self.workspace.view_request=Some(false);
            }
            ToggleStage(id) => {
                if !self.workspace.collapsed_stages.remove(&id) { self.workspace.collapsed_stages.insert(id); }
                self.workspace.layout_dirty = true;
                self.save_layout()?;
            }
            Select(row) => {
                if let Some((source, field)) = self.workspace.pending_link.clone() {
                    self.connect_canvas(source, &field, row)?;
                    self.workspace.pending_link = None;
                }
                self.row = Some(row);
                self.workspace.selected = BTreeSet::from([row]);
                self.picker_field = None;
                self.rebuild_links();
                self.workspace.view_request=Some(false);
            }
            ToggleCatalog => self.workspace.catalog_hidden = !self.workspace.catalog_hidden,
            Tables => self.workspace.enabled=false,
            CatalogPage(down) => {self.workspace.catalog_offset=if down {self.workspace.catalog_offset+60}else{self.workspace.catalog_offset.saturating_sub(60)};}
            Collapse => {
                if let Some(v) = self.row.and_then(|r| self.rows().get(r)).cloned() {
                    for block in self.authoring_blocks(&v) {
                        self.block_overrides.insert((self.table, self.row, false, block.key.into()), false);
                    }
                }
            }
            Validate => self.mission_diagnostics(),
            Fit => self.workspace.view_request=Some(true),
            Center => self.workspace.view_request=Some(false),
            Arrange => self.arrange_mission_canvas()?,
            Zoom(delta) => self.graph_zoom = (self.graph_zoom + delta).clamp(0.5, 1.5),
            ResetZoom => {self.graph_zoom=1.;self.workspace.view_request=Some(false);}
            Connect(row, field) => { self.workspace.pending_link = Some((row, field)); self.workspace.selected_edge = None; }
            Edge(row, field, slot) => self.workspace.selected_edge = Some((row, field, slot)),
            RemoveEdge => { let (row, field, slot) = self.workspace.selected_edge.clone().ok_or("Select a transition")?; self.graph_remove(row, &field, slot)?; self.workspace.selected_edge = None; }
            Delete => { self.delete_confirm = true; self.workspace.context = Some((anchor, Context::Node(self.row.ok_or("Select a row")?))); }
            ConfirmDelete => self.delete_mission_stage()?,
            QuickCreate(field) => self.start_quick(&field)?,
            QuickPlaceholder(field) => self.start_quick_kind(&field, true)?,
            EditJournal(field) => self.start_journal_edit(&field)?,
            InspectField(row, field) => {
                self.row = Some(row);
                self.workspace.selected = BTreeSet::from([row]);
                self.column_search = field.clone();
                self.begin(Focus::Cell(field));
                self.rebuild_links();
            }
            OpenMission(row) => {
                self.row = Some(row); self.graph_stages = true;
                self.workspace.selected = BTreeSet::from([row]);
                self.workspace.view_request = Some(true); self.rebuild_links();
            }
            CopyMissionId(row) => {
                let value = self.rows().get(row).and_then(|v| v.get("m_iHMissionID")).map(display).ok_or("Missing target mission")?;
                arboard::Clipboard::new().and_then(|mut c| c.set_text(value)).map_err(|e| e.to_string())?;
            }
            CopyId => {
                let value = self.row.and_then(|r| self.rows().get(r)).and_then(|v| v.get("m_iHTaskID")).map(display).ok_or("Select a row")?;
                arboard::Clipboard::new().and_then(|mut c| c.set_text(value)).map_err(|e| e.to_string())?;
            }
            Help(field) => self.workspace.help = Some(field),
        }
        self.revision += 1;
        Ok(())
    }
    pub(super) fn mission_diagnostics(&mut self) {
        let mut issues = Vec::new();
        for (r, row) in self.rows().iter().enumerate().filter(|(_, v)| v["m_iHTaskID"].as_i64().is_some_and(|n| n > 0)) {
            for (field, value) in row.as_object().into_iter().flatten() {
                if let Some(error) = schema::invalid(&self.tables[self.table].label, field, value) {
                    issues.push((r, field.clone(), error.into()));
                }
            }
            if row["m_iHNPCID"].as_i64() == Some(0) { issues.push((r, "m_iHNPCID".into(), "mission_no_giver".into())); }
        }
        for reference in self.references.iter().filter(|r| r.table == self.table && r.target_row.is_none()) {
            issues.push((reference.row, reference.field.clone(), "reference".into()));
        }
        self.workspace.diagnostics = issues;
    }
}
