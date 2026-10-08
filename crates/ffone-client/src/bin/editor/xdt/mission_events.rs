//! The message and speaker are a single authoring action on a stage card.
use super::*;

pub(super) const PAIRS: &[(&str, &str)] = &[
    ("m_iSTMessageTextID", "m_iSTMessageSendNPC"),
    ("m_iSUMessagetextID", "m_iSUMessageSendNPC"),
    ("m_iFMessageTextID", "m_iFMessageSendNPC"),
    ("m_iSTEmailTextID", "m_iSTEmailSendNPC"),
    ("m_iSUEmailTextID", "m_iSUEmailSendNPC"),
    ("m_iFEmailTextID", "m_iFEmailSendNPC"),
    ("m_iSTDialogBubble", "m_iSTDialogBubbleNPCID"),
    ("m_iSUDialogBubble", "m_iSUDialogBubbleNPCID"),
    ("m_iFDialogBubble", "m_iFDialogBubbleNPCID"),
];

pub(super) fn speaker(field: &str) -> Option<&'static str> {
    PAIRS.iter().find(|pair| pair.0 == field).map(|pair| pair.1)
}

pub(super) fn message_type(field: &str) -> Option<&'static str> {
    match field {
        "m_iSTMessageTextID" => Some("m_iSTMessageType"),
        "m_iSUMessagetextID" => Some("m_iSUMessageType"),
        "m_iFMessageTextID" => Some("m_iFMessageType"),
        "m_iSTEmailTextID" => Some("m_iSTMessageType"),
        "m_iSUEmailTextID" => Some("m_iSUMessageType"),
        "m_iFEmailTextID" => Some("m_iFMessageType"),
        _ => None,
    }
}

pub(super) fn enable_nanocom(owner: &mut Value, field: &str) {
    if mail_fields::is_override(field) {return;}
    if let Some(route) = message_type(field) {
        if owner[route].as_i64().unwrap_or(0) & 4 != 0 {
            return;
        }
        if owner[field].as_i64().is_some_and(|id| id > 0) {
            owner[route] = Value::from(owner[route].as_i64().unwrap_or(0) | 2);
        }
    }
}

impl XdtEditor {
    pub(super) fn enable_edited_nanocom(&mut self) {
        let ids: BTreeSet<_> = mission_publish::changed_tasks(&self.base, &self.document)
            .into_iter()
            .filter_map(|task| task["m_iHTaskID"].as_i64())
            .collect();
        for table in self.document["tables"].as_array_mut().into_iter().flatten() {
            let old_rows=self.base["tables"].as_array().into_iter().flatten().find(|old|old["name"]==table["name"])
                .and_then(|old|old["value"]["m_pMissionTable"]["m_pMissionData"].as_array());
            for task in table["value"]["m_pMissionTable"]["m_pMissionData"]
                .as_array_mut()
                .into_iter()
                .flatten()
            {
                if ids.contains(&task["m_iHTaskID"].as_i64().unwrap_or(0)) {
                    let old=old_rows.and_then(|rows|rows.iter().find(|old|old["m_iHTaskID"]==task["m_iHTaskID"]));
                    for (field, _) in PAIRS {
                        if let Some(route)=message_type(field) {
                            if old.map(|old|&old[*field])!=Some(&task[*field])&&old.and_then(|old|old.get(route))==task.get(route) {
                                enable_nanocom(task, field);
                            }
                        }
                    }
                }
            }
        }
    }
    pub(super) fn edit_event(&mut self, row: usize, field: &str) -> Result<(), String> {
        self.edit_message(row, field, true)
    }
    fn edit_message(&mut self, row: usize, field: &str, nanocom: bool) -> Result<(), String> {
        let speaker = speaker(field).ok_or("Unknown message channel")?;
        self.row = Some(row);
        self.workspace.selected = BTreeSet::from([row]);
        self.array_slot = None;
        let owner = self
            .rows()
            .get(row)
            .cloned()
            .ok_or("Missing mission task")?;
        let locale = self
            .workspace
            .event_locales
            .get(&(row, field.into()))
            .copied()
            .unwrap_or(1);
        self.start_mission_text_edit(field, true)?;
        if nanocom {
            if let (Some(origin), Some(route)) = (&self.workspace.quick, message_type(field)) {
                if let Some(value) = self
                    .document
                    .pointer_mut(&self.tables[origin.table].pointer)
                    .and_then(|rows| rows.get_mut(origin.row))
                {
                    value[route] = Value::from(value[route].as_i64().unwrap_or(0) | 2);
                }
            }
        }
        // Provide a useful initial speaker. This belongs to the creation frame,
        // so cancellation and undo also restore the original owner record.
        if owner[speaker].as_i64().unwrap_or(0) == 0 {
            let preferred = if field.starts_with("m_iST") {
                "m_iHNPCID"
            } else {
                "m_iHTerminatorNPCID"
            };
            let npc = owner[preferred]
                .as_i64()
                .filter(|id| *id > 0)
                .or_else(|| owner["m_iHJournalNPCID"].as_i64().filter(|id| *id > 0))
                .unwrap_or(0);
            if let Some(origin) = &self.workspace.quick {
                if let Some(value) = self
                    .document
                    .pointer_mut(&self.tables[origin.table].pointer)
                    .and_then(|rows| rows.get_mut(origin.row))
                {
                    value[speaker] = Value::from(npc);
                }
            }
        }
        self.begin(Focus::Locale(locale));
        Ok(())
    }

    pub(super) fn edit_email(
        &mut self,
        row: usize,
        field: &str,
        slot: Option<usize>,
    ) -> Result<(), String> {
        if slot.is_none() {
            let field=mail_fields::email_field(self.rows().get(row).ok_or("Missing mission stage")?,field);
            self.edit_message(row, field, false)?;
            let origin = self.workspace.quick.as_ref().ok_or("Missing email owner")?;
            let owner = self
                .document
                .pointer_mut(&self.tables[origin.table].pointer)
                .and_then(|v| v.get_mut(origin.row))
                .ok_or("Missing email owner")?;
            if let Some(route) = message_type(field) {
                owner[route] = Value::from(owner[route].as_i64().unwrap_or(0) | 4);
            }
        } else {
            self.row = Some(row);
            self.workspace.selected = BTreeSet::from([row]);
            self.array_slot = slot;
            self.start_mission_text_edit(field, true)?;
        }
        Ok(())
    }

    pub(super) fn sync_mission_text_links(&mut self) {
        // Direct table edits must publish their text too, even when the string ID is unchanged.
        for table in &self.tables {
            if !table.label.ends_with("/m_pMissionStringData"){continue;}
            let rows=self.document.pointer(&table.pointer).and_then(Value::as_array);
            let old=self.base.pointer(&table.pointer);
            for (i,row) in rows.into_iter().flatten().enumerate() {
                let Some(text)=row["m_pstrNameString"].as_str()else{continue;};
                if old.and_then(|v|v.get(i)).and_then(|v|v["m_pstrNameString"].as_str())==Some(text){continue;}
                self.workspace.locale_drafts.entry(format!("content.tabledata.mission.mission_string.{i}.str_name_string")).or_insert_with(||[Some(text.to_owned()),Some(text.to_owned())]);
            }
        }
        let before: BTreeMap<_, _> = ffone_client::localization::mission_text_links(&self.base)
            .into_iter()
            .collect();
        for (key, source) in ffone_client::localization::mission_text_links(&self.document) {
            let changed = before.get(&key) != Some(&source)
                || self.workspace.locale_drafts.contains_key(&source);
            let values: [Option<String>; 2] = std::array::from_fn(|i| {
                if !changed && self.workspace.locale_base[i].contains_key(&key) {
                    return None;
                }
                self.workspace
                    .locale_drafts
                    .get(&source)
                    .and_then(|value| value[i].clone())
                    .or_else(|| self.workspace.locale_base[i].get(&source).cloned())
            });
            if values.iter().any(Option::is_some) {
                let entry = self
                    .workspace
                    .locale_drafts
                    .entry(key)
                    .or_insert_with(|| [None, None]);
                for i in 0..2 {
                    if let Some(value) = &values[i] {
                        entry[i] = Some(value.clone());
                    }
                }
            }
        }
    }
}
