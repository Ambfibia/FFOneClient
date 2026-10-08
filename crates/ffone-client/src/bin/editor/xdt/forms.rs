//! Explicit record drafts. Switching fields never inserts an incomplete record.
use super::*;

pub(super) struct Draft {
    pub value: Value,
    pub required: BTreeSet<String>,
    pub template: usize,
    pub name: Option<NewName>,
    pub placeholder: bool,
}
pub(super) struct NewName {
    pub field: String,
    pub table: usize,
    pub key: String,
    pub row: Value,
    pub text: String,
}

impl XdtEditor {
    pub(super) fn rebuild_summaries(&mut self) {
        self.summaries = self
            .tables
            .iter()
            .map(|t| {
                self.document
                    .pointer(&t.pointer)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(r, _)| self.record_name_uncached(t, r))
                    .collect()
            })
            .collect();
    }
    fn record_name_uncached(&self, table: &Table, r: usize) -> String {
        let Some(row) = self.document.pointer(&table.pointer).and_then(|v| v.get(r)) else {
            return String::new();
        };
        if let Some(name) = mission_journal::record_name(self, table, row) { return name; }
        for key in [
            "m_strName",
            "m_pstrNameString",
            "name",
            "m_strTitle",
            "m_pstrMMeshModelString",
            "m_pstrFMeshModelString",
            "path",
        ] {
            if let Some(name) = row
                .get(key)
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                return name.to_owned();
            }
        }
        for field in [
            "m_iNpcName",
            "m_iNanoName",
            "m_iHMissionName",
            "m_iItemName",
            "m_iSkillName",
            "m_iTuneName",
        ] {
            let Some(index) = row.get(field).and_then(Value::as_u64) else {
                continue;
            };
            let target = relations::rule(&table.label, field)
                .map(|(t, _)| t)
                .or_else(|| {
                    let (group, _) = table.label.rsplit_once('/')?;
                    let kind = if field == "m_iItemName" {
                        "Item"
                    } else if field == "m_iSkillName" {
                        "Skill"
                    } else {
                        "NanoTune"
                    };
                    Some(format!("{group}/m_p{kind}StringData"))
                });
            if let Some(strings) =
                target.and_then(|name| self.tables.iter().find(|t| t.label == name))
            {
                if let Some(name) = self
                    .document
                    .pointer(&strings.pointer)
                    .and_then(|v| v.get(index as usize))
                    .and_then(|v| v.get("m_strName").or_else(|| v.get("m_pstrNameString")))
                    .and_then(Value::as_str)
                    .filter(|s| !s.trim().is_empty())
                {
                    return name.to_owned();
                }
            }
        }
        let id = schema::identity(&table.label)
            .and_then(|key| row.get(key))
            .map(display);
        id.map(|id| format!("ID {id}"))
            .unwrap_or_else(|| format!("#{}", r + 1))
    }
    pub(super) fn record_name(&self, t: usize, r: usize) -> String {
        self.summaries
            .get(t)
            .and_then(|rows| rows.get(r))
            .cloned()
            .unwrap_or_else(|| format!("#{}", r + 1))
    }
    pub(super) fn value_label(&self, row: usize, field: &str) -> String {
        let value = self.rows().get(row).and_then(|r| r.get(field));
        if let Some(reference) = self
            .references
            .iter()
            .find(|r| r.table == self.table && r.row == row && r.field == field)
        {
            if let Some(target) = reference.target_row {
                return format!(
                    "{} · {}",
                    self.record_name(reference.target_table, target),
                    reference.value
                );
            }
        }
        value.map(display).unwrap_or_else(|| "—".into())
    }
    pub(super) fn start_draft(&mut self, duplicate: bool) -> Result<(), String> {
        let template = self
            .row
            .filter(|r| *r < self.rows().len())
            .or_else(|| {
                self.rows().iter().position(|r| {
                    schema::identity(&self.tables[self.table].label)
                        .and_then(|f| r.get(f))
                        .and_then(Value::as_i64)
                        .is_some_and(|n| n > 0)
                })
            })
            .or_else(|| (!self.rows().is_empty()).then_some(0))
            .ok_or("No record schema: select a populated table")?;
        if duplicate && self.row.is_none() {
            return Err("Select a record to duplicate".into());
        }
        let mut value = self.rows()[template].clone();
        let fields = schema::required(&self.tables[self.table].label);
        let mut required: BTreeSet<String> = fields.iter().map(|s| (*s).to_owned()).collect();
        // Allocate only verified identity fields, never foreign keys or classification numbers.
        if let Some(key) = schema::identity(&self.tables[self.table].label) {
            let max = self
                .rows()
                .iter()
                .filter_map(|r| r.get(key).and_then(Value::as_i64))
                .max()
                .unwrap_or(0);
            let next = max.checked_add(1).ok_or("ID range exhausted")?;
            value[key] = Value::from(next);
            required.insert(key.to_owned());
        } else if fields.is_empty() {
            // No verified identity: the complete typed template is the creation contract.
            required.extend(
                value
                    .as_object()
                    .into_iter()
                    .flat_map(|r| r.iter())
                    .filter(|(_, v)| {
                        !v.is_null() && !v.as_str().is_some_and(|s| s.trim().is_empty())
                    })
                    .map(|(k, _)| k.clone()),
            );
        }
        self.draft = Some(Draft {
            value,
            required,
            template,
            name: None,
            placeholder: false,
        });
        if !duplicate {
            if self.tables[self.table].label.ends_with("/m_pJournalData") {
                self.draft.as_mut().unwrap().value["m_iDetailedTaskDesc"] = Value::from(0);
            }
            if self.tables[self.table]
                .label
                .ends_with("/m_pMissionTable/m_pMissionData")
            {
                let draft=self.draft.as_mut().unwrap();
                let task=draft.value["m_iHTaskID"].clone();
                draft.value=mission_fields::defaults(&draft.value);
                draft.value["m_iHTaskID"]=task;
                let max = self
                    .rows()
                    .iter()
                    .filter_map(|r| r.get("m_iHMissionID").and_then(Value::as_i64))
                    .max()
                    .unwrap_or(0);
                self.draft.as_mut().unwrap().value["m_iHMissionID"] =
                    Value::from(max.checked_add(1).ok_or("Mission ID range exhausted")?);
            }
            if let Some(field) = schema::name_field(&self.tables[self.table].label) {
                if self.reference_target(field).is_some() {
                    self.own_name(field.to_owned())?;
                    if self.mission_table() {
                        let draft = self.draft.as_mut().unwrap();
                        draft.value["m_iHCurrentObjective"] = draft.value["m_iHMissionName"].clone();
                    }
                }
            }
        }
        self.focus = None;
        self.picker_field = None;
        self.advanced = false;
        self.status.clear();
        self.revision += 1;
        Ok(())
    }
    pub(super) fn start_placeholder_draft(&mut self) -> Result<(), String> {
        if !self.tables[self.table].label.ends_with("/m_pNpcTable/m_pNpcData") {
            return Err("Select the NPC table".into());
        }
        let template = self.rows().iter().position(|r| r["m_iNpcNumber"].as_i64() == Some(1401))
            .or_else(|| (0..self.rows().len()).find(|r| self.record_name(self.table, *r).eq_ignore_ascii_case("Location A256")))
            .ok_or("Missing empty NPC template: Location A256 (1401)")?;
        let selected = self.row.replace(template);
        let result = self.start_draft(false);
        self.row = selected;
        result?;
        self.draft.as_mut().unwrap().placeholder = true;
        Ok(())
    }
    pub(super) fn apply_draft_field(&mut self, field: String) -> bool {
        let result = self
            .draft
            .as_mut()
            .ok_or("No creation draft".to_owned())
            .and_then(|draft| {
                let old = draft.value.get(&field).ok_or("Missing field".to_owned())?;
                let value = parse_cell(old, &self.edit)
                    .map_err(|_| format!("{field}: {}", schema::type_error(old)))?;
                draft.value[&field] = value;
                Ok(())
            });
        if let Err(error) = result {
            self.status = error;
            self.revision += 1;
            return false;
        }
        self.focus = None;
        self.status.clear();
        self.revision += 1;
        true
    }
    pub(super) fn draft_errors(&self) -> Vec<(String, String)> {
        let Some(draft) = &self.draft else {
            return Vec::new();
        };
        let mut errors = Vec::new();
        if let Some(name) = &draft.name {
            let text = if self.focus == Some(Focus::NewName) {
                &self.edit
            } else {
                &name.text
            };
            if text.trim().is_empty() {
                errors.push((name.field.clone(), "required".into()));
            }
        }
        for field in &draft.required {
            let Some(value) = draft.value.get(field) else {
                errors.push((field.clone(), "required".into()));
                continue;
            };
            if value.is_null() || value.as_str().is_some_and(|s| s.trim().is_empty()) {
                errors.push((field.clone(), "required".into()));
            } else if let Some(error) =
                schema::invalid(&self.tables[self.table].label, field, value)
            {
                errors.push((field.clone(), error.into()));
            }
        }
        for (field, value) in draft.value.as_object().into_iter().flatten() {
            if draft.name.as_ref().is_some_and(|n| n.field == *field) {
                continue;
            }
            if let Some(error) = schema::invalid(&self.tables[self.table].label, field, value) {
                if !errors.iter().any(|(f, _)| f == field) {
                    errors.push((field.clone(), error.into()));
                }
            }
            if let Some((t, id)) = self.reference_target(field) {
                // The owned text is inserted together with the draft; another text
                // field may already point at that same pending row.
                if id.is_none() && draft.name.as_ref().is_some_and(|name|
                    name.table == t && draft.value.get(&name.field) == Some(value)) {
                    continue;
                }
                let found = if let Some(id) = id {
                    self.document
                        .pointer(&self.tables[t].pointer)
                        .and_then(Value::as_array)
                        .is_some_and(|rows| rows.iter().any(|r| r.get(id) == Some(value)))
                } else {
                    value.as_u64().is_some_and(|i| {
                        self.document
                            .pointer(&self.tables[t].pointer)
                            .and_then(Value::as_array)
                            .is_some_and(|rows| (i as usize) < rows.len())
                    })
                };
                // Optional routes use non-positive sentinels in accepted data.
                if !found && value.as_i64().is_some_and(|v| v > 0) {
                    errors.push((field.clone(), "reference".into()));
                }
                if matches!(
                    field.as_str(),
                    "m_iNpcName" | "m_iNanoName" | "m_iHMissionName" | "m_iHCurrentObjective"
                ) {
                    let named = value
                        .as_u64()
                        .and_then(|i| {
                            self.document
                                .pointer(&self.tables[t].pointer)
                                .and_then(|rows| rows.get(i as usize))
                        })
                        .and_then(|row| {
                            row.get("m_strName").or_else(|| row.get("m_pstrNameString"))
                        })
                        .and_then(Value::as_str)
                        .is_some_and(|name| !name.trim().is_empty());
                    if !named {
                        errors.push((field.clone(), "reference".into()));
                    }
                }
            }
        }
        errors
    }
    pub(super) fn create_draft(&mut self) -> Result<(), String> {
        if !self.apply() { return Err(self.status.clone()); }
        self.validate_text_draft()?;
        let errors = self.draft_errors();
        if !errors.is_empty() {
            return Err(format!("Required parameters: {}", errors.iter().map(|(f, _)| f.as_str()).collect::<Vec<_>>().join(", ")));
        }
        let draft = self.draft.as_ref().ok_or("No creation draft")?;
        let mut next = self.document.clone();
        if let Some(name) = &draft.name {
            let target = next.pointer_mut(&self.tables[name.table].pointer).and_then(Value::as_array_mut).ok_or("Missing text table")?;
            if draft.value[&name.field].as_u64() != Some(target.len() as u64) {
                return Err("Text table changed while creating the record".into());
            }
            let mut text = name.row.clone();
            text[&name.key] = Value::from(name.text.clone());
            target.push(text);
        }
        let rows = next.pointer_mut(&self.tables[self.table].pointer).and_then(Value::as_array_mut).ok_or("Missing owner table")?;
        let row = self.workspace.quick_existing.unwrap_or(rows.len());
        let value = draft.value.clone();
        if self.workspace.quick_existing.is_some(){*rows.get_mut(row).ok_or("Missing shared text")?=value.clone();}
        else{rows.push(value.clone());}
        let link=schema::identity(&self.tables[self.table].label).and_then(|id|value.get(id)).cloned().unwrap_or_else(||Value::from(row));
        if let Some(origin) = self.workspace.quick.as_ref().filter(|_|self.workspace.creation.last().is_none_or(|frame|frame.draft.is_none())) {
            let owner = next.pointer_mut(&self.tables[origin.table].pointer).and_then(|v| v.get_mut(origin.row)).ok_or("Missing owner")?;
            if let Some(slot) = origin.slot { *owner.get_mut(&origin.field).and_then(|v| v.get_mut(slot)).ok_or("Missing list element")? = link.clone(); }
            else { owner[&origin.field] = link.clone(); }
            mission_events::enable_nanocom(owner, &origin.field);
        }
        let translations=self.workspace.quick_text.as_ref().map(|text|{
            if text.values.iter().all(|v|v.is_empty()) {
                let fallback=draft.name.as_ref().map(|n|n.text.clone()).or_else(||value["m_pstrNameString"].as_str().map(str::to_owned)).unwrap_or_default();
                [fallback.clone(),fallback]
            }else{text.values.clone()}
        });
        self.commit_document(next)?;
        let aliases=self.quick_text_aliases(&value,row);
        if let Some(translations)=translations{self.attach_text_history(aliases,translations)?;}
        if self.workspace.quick.is_some() { self.finish_creation(Some(link)); }
        else {
            self.draft = None;
            self.row = Some(row);
            if self.mission_table() { self.mission_filter = value["m_iHMissionID"].as_i64(); }
            self.search.clear();
            self.refresh();
            self.offset = self.filtered.iter().position(|r| *r == row).unwrap_or(0);
        }
        self.picker_field = None;
        self.status.clear();
        Ok(())
    }
    pub(super) fn reference_target(&self, field: &str) -> Option<(usize, Option<&'static str>)> {
        self.reference_target_at(field,self.array_slot)
    }
    pub(super) fn reference_target_at(&self,field:&str,slot:Option<usize>)->Option<(usize,Option<&'static str>)>{
        let value=self.draft.as_ref().map(|d|&d.value).or_else(||self.row.and_then(|r|self.rows().get(r)))?;
        let (name, id) = relations::rule_for(&self.tables.get(self.table)?.label, field,value.as_object()?,slot)?;
        Some((self.tables.iter().position(|t| t.label == name)?, id))
    }
    pub(super) fn reference_candidates(&self, table:usize, field:&str)->Vec<usize> {
        let Some((_,_))=self.reference_target(field) else {return Vec::new();};
        let Some(rows)=self.document.pointer(&self.tables[table].pointer).and_then(Value::as_array) else {return Vec::new();};
        let mut seen=BTreeSet::new();
        let own=self.field_value("m_iHMissionID").and_then(Value::as_i64);
        let existing:BTreeSet<_>=self.field_value(mission_graph::REQUIRE).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_i64).collect();
        let needle=self.reference_search.to_lowercase();
        let mut candidates: Vec<_> = rows.iter().enumerate().filter(|(_,row)| {
            if matches!(field,"m_iSUOutgoingTask"|"m_iFOutgoingTask") {
                return row["m_iHMissionID"].as_i64()==own && row["m_iHTaskID"].as_i64().is_some_and(|id|id>0);
            }
            if field!=mission_graph::REQUIRE {return true;}
            let Some(group)=row["m_iHMissionID"].as_i64().filter(|id|*id>0) else {return false;};
            Some(group)!=own && !existing.contains(&group) && seen.insert(group)
        }).filter(|(r,_)|self.matches_search(table,*r,&needle))
            .map(|(r,_)|r).collect();
        candidates.sort_by_key(|r|self.search_rank(table,*r,&needle));
        candidates
    }
    pub(super) fn own_name(&mut self, field: String) -> Result<(), String> {
        if schema::name_field(&self.tables[self.table].label) != Some(field.as_str()) {
            return Err("Only the record name supports inline text creation".into());
        }
        let (t, id) = self.reference_target(&field).ok_or("Missing name route")?;
        if id.is_some() {
            return Err("Name must use an indexed text record".into());
        }
        let draft = self.draft.as_mut().ok_or("No creation draft")?;
        let rows = self
            .document
            .pointer(&self.tables[t].pointer)
            .and_then(Value::as_array)
            .ok_or("Missing text table")?;
        let template = draft.value[&field]
            .as_u64()
            .and_then(|i| rows.get(i as usize))
            .or_else(|| rows.first())
            .ok_or("Missing text schema")?;
        let mut row = template.clone();
        let key = if row.get("m_strName").is_some() {
            "m_strName"
        } else {
            "m_pstrNameString"
        };
        // A new name starts with empty optional prose, preserving all numeric metadata.
        for value in row.as_object_mut().into_iter().flat_map(|r| r.values_mut()) {
            if value.is_string() {
                *value = Value::String(String::new());
            }
        }
        draft.value[&field] = Value::from(rows.len());
        draft.name = Some(NewName {
            field,
            table: t,
            key: key.to_owned(),
            row,
            text: String::new(),
        });
        self.picker_field = None;
        self.focus = None;
        self.revision += 1;
        Ok(())
    }
    pub(super) fn existing_name(&mut self) -> Result<(), String> {
        let draft = self.draft.as_ref().ok_or("No creation draft")?;
        let field = draft.name.as_ref().ok_or("No inline name")?.field.clone();
        let value = self.rows()[draft.template][&field].clone();
        let draft = self.draft.as_mut().unwrap();
        draft.value[&field] = value;
        draft.name = None;
        self.begin(Focus::Draft(field));
        Ok(())
    }
    pub(super) fn pick_reference(
        &mut self,
        field: String,
        t: usize,
        row: usize,
    ) -> Result<(), String> {
        let (target, id) = self
            .reference_target(&field)
            .ok_or("Unknown reference route")?;
        if target != t {
            return Err("Wrong reference table".into());
        }
        let target_row = self
            .document
            .pointer(&self.tables[t].pointer)
            .and_then(|v| v.get(row))
            .ok_or("Missing referenced record")?;
        let mut value = id
            .and_then(|id| target_row.get(id))
            .cloned()
            .unwrap_or_else(|| Value::from(row));
        if let Some(slot) = self.array_slot {
            let mut array = self
                .field_value(&field)
                .cloned()
                .ok_or("Missing list field")?;
            *array.get_mut(slot).ok_or("Missing list element")? = value;
            value = array;
        }
        if let Some(draft) = &mut self.draft {
            if draft.name.as_ref().is_some_and(|n| n.field == field) {
                draft.name = None;
            }
            draft.value[&field] = value;
        } else {
            let r = self.row.ok_or("Select a record")?;
            let mut rows = self.rows().to_vec();
            rows[r][&field] = value;
            self.change(rows)?;
        }
        self.focus = None;
        self.array_slot = None;
        self.picker_field = None;
        self.status.clear();
        self.revision += 1;
        Ok(())
    }
}
