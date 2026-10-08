//! Semantic authoring views over the original document. Blocks never rewrite hidden fields.
use super::*;

pub(super) struct Block {
    pub key: &'static str,
    pub fields: Vec<String>,
    pub configured: bool,
    pub core: bool,
}

pub(super) fn group(table: &str, field: &str) -> Option<&'static str> {
    let leaf = table.rsplit('/').next()?;
    if table.contains("/m_pAnimationTable/") {
        return None;
    }
    if schema::identity(table) == Some(field)
        || schema::name_field(table) == Some(field)
        || leaf == "m_pMissionData" && field == "m_iHMissionID"
    {
        return Some("identity");
    }
    match leaf {
        "m_pNpcData" | "m_pNanoData" => match field {
            "m_iMesh" | "m_iIcon1" | "m_fScale" | "m_iHeight" | "m_iRadius" | "m_iNanoSet" => {
                Some("appearance")
            }
            "m_iHP" | "m_iHPRegen" | "m_iNpcLevel" | "m_iNpcStyle" | "m_iStyle" | "m_iPower"
            | "m_iProtection" | "m_iAccuracy" | "m_iDodge" | "m_iAtkRange" | "m_iAtkRate"
            | "m_iAtkAngle" | "m_iCombatRange" | "m_iNanoBattery1" => Some("combat"),
            "m_iWalkSpeed" | "m_iRunSpeed" | "m_iSwimSpeed" | "m_iJumpDistance"
            | "m_iJumpHeight" | "m_iSightRange" | "m_iIdleRange" => Some("movement"),
            "m_iBarkerNumber" | "m_iComment" => Some("dialogue"),
            // Other AI/skill codes remain available in All fields, without invented behavior.
            "m_iNpcType" => Some("other"),
            _ => None,
        },
        "m_pMissionData" => {
            if matches!(field, "m_iHDifficultyType" | "m_iHMissionType" | "m_iHTaskType" | "m_iHMissionVisibility") {
                Some("identity")
            } else if matches!(field, "m_iCSUEnemyID" | "m_iCSUNumToKill") {
                Some("defeat")
            } else if matches!(field, "m_iCSUItemID" | "m_iCSUItemNumNeeded") {
                Some("collect")
            } else if field.starts_with("m_iCSUDEF") || field == "m_iCSUDEPNPCFollow" {
                Some("defend")
            } else if field == "m_iCSUCheckTimer" {
                Some("timing")
            } else if field.starts_with("m_iCST")
                || field.starts_with("m_iCTR")
                || field == "m_iRequireInstanceID"
            {
                Some("conditions")
            } else if field == "m_iHCurrentObjective"
                || matches!(
                    field,
                    "m_iHNPCID" | "m_iHTerminatorNPCID" | "m_iHJournalNPCID"
                )
            {
                Some("objective")
            } else if field.starts_with("m_iST") {
                Some("start")
            } else if field.starts_with("m_iSU") {
                Some("success")
            } else if field.starts_with("m_iF") {
                Some("failure")
            } else {
                None
            }
        }
        "m_pItemData" if table.contains("ItemTable/") => match field {
            "m_iMinReqLev" | "m_iReqSex" | "m_iEquipLoc" | "m_iEquipType" => Some("requirements"),
            "m_iItemPrice" | "m_iItemSellPrice" | "m_iSellAble" | "m_iTradeAble"
            | "m_iCashAble" | "m_iStackNumber" => Some("economy"),
            "m_iMesh" | "m_iIcon" | "m_iRarity" => Some("appearance"),
            "m_iAtkRange" | "m_iAtkRate" | "m_iAtkAngle" | "m_iBatteryDrain" | "m_ibattery"
            | "m_iPointRat" | "m_iGroupRat" | "m_iDefenseRat" => Some("combat"),
            f if f.starts_with("m_iUp_") => Some("bonuses"),
            _ => None,
        },
        "m_pRewardData" => Some("reward"),
        _ => None,
    }
}

pub(super) fn neutral(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(v) => !v,
        Value::Number(n) => n.as_f64() == Some(0.),
        Value::String(s) => s.trim().is_empty(),
        Value::Array(a) => a.iter().all(neutral),
        Value::Object(o) => o.values().all(neutral),
    }
}

pub(super) fn blocks(table: &str, columns: &[String], value: &Value, advanced: bool) -> Vec<Block> {
    let mut grouped: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let semantic = columns.iter().any(|f| group(table, f).is_some());
    for field in columns {
        if value.get(field).is_none() {
            continue;
        }
        let key = if semantic {
            match group(table, field) {
                Some(key) => key,
                None if advanced || schema::required(table).contains(&field.as_str()) => "other",
                None => continue,
            }
        } else {
            if !advanced && !schema::basic(table, columns).contains(field) {
                continue;
            }
            schema::section(field)
        };
        grouped.entry(key).or_default().push(field.clone());
    }
    [
        "identity",
        "objective",
        "defeat",
        "collect",
        "defend",
        "combat",
        "requirements",
        "appearance",
        "movement",
        "dialogue",
        "conditions",
        "timing",
        "start",
        "success",
        "failure",
        "economy",
        "bonuses",
        "reward",
        "mission",
        "other",
    ]
    .into_iter()
    .filter_map(|key| {
        grouped.remove(key).map(|fields| Block {
            configured: fields.iter().any(|f| !neutral(&value[f])),
            fields: {
                let mut fields = schema::ordered_fields(fields);
                let preferred = [
                    schema::identity(table).unwrap_or(""),
                    schema::name_field(table).unwrap_or(""),
                    "m_iHCurrentObjective",
                    "m_iHP",
                    "m_iNpcLevel",
                    "m_iNpcStyle",
                    "m_iStyle",
                    "m_iNanoBattery1",
                    "m_iMesh",
                    "m_iIcon1",
                    "m_iIcon",
                    "m_fScale",
                ];
                fields.sort_by_key(|f| {
                    preferred
                        .iter()
                        .position(|p| *p == f)
                        .unwrap_or(preferred.len())
                });
                if table.ends_with("/m_pMissionData") {
                    // Keep each speaker immediately before its message in the
                    // start/success/failure sections, independently of suffix sorting.
                    for pair in super::mission_events::PAIRS {
                        if let (Some(speaker), Some(text)) = (fields.iter().position(|f| f == pair.1),
                            fields.iter().position(|f| f == pair.0)) {
                            let text = fields.remove(text);
                            let speaker = fields.iter().position(|f| f == pair.1).unwrap_or(speaker);
                            fields.insert(speaker + 1, text);
                        }
                    }
                }
                fields
            },
            core: matches!(key, "identity" | "objective" | "reward"),
            key,
        })
    })
    .collect()
}

pub(super) struct TextTarget {
    pub table: usize,
    pub row: usize,
    pub key: String,
    pub users: usize,
}

impl XdtEditor {
    pub(super) fn mission_table(&self) -> bool {
        self.tables
            .get(self.table)
            .is_some_and(|t| t.label.ends_with("/m_pMissionTable/m_pMissionData"))
    }
    pub(super) fn clear_reference(&mut self, field: &str) -> Result<(), String> {
        let (_, id) = self
            .reference_target(field)
            .ok_or("Unknown reference route")?;
        if id.is_none() && !relations::optional_index(field) {
            return Err("Indexed references cannot be cleared".into());
        }
        let mut value = self.field_value(field).cloned().ok_or("Missing field")?;
        if let Some(slot) = self.array_slot {
            *value.get_mut(slot).ok_or("Missing list element")? = Value::from(0);
        } else if value.is_number() {
            value = Value::from(0);
        } else {
            return Err("Select a list element".into());
        }
        if let Some(draft) = &mut self.draft {
            draft.value[field] = value;
        } else {
            let row = self.row.ok_or("Select a record")?;
            let mut rows = self.rows().to_vec();
            rows[row][field] = value;
            self.change(rows)?;
        }
        self.focus = None;
        self.picker_field = None;
        self.array_slot = None;
        self.revision += 1;
        Ok(())
    }
    pub(super) fn field_value(&self, field: &str) -> Option<&Value> {
        self.draft
            .as_ref()
            .map(|d| &d.value)
            .or_else(|| self.row.and_then(|r| self.rows().get(r)))?
            .get(field)
    }

    pub(super) fn begin_element(&mut self, field: String, slot: usize) -> Result<(), String> {
        let value = self
            .field_value(&field)
            .and_then(|v| v.get(slot))
            .ok_or("Missing list element")?
            .clone();
        self.begin(if self.draft.is_some() {
            Focus::Draft(field)
        } else {
            Focus::Cell(field)
        });
        self.array_slot = Some(slot);
        self.edit = display(&value);
        self.cursor = self.edit.len();
        self.anchor = 0;
        if self.picker_field.as_deref().is_some_and(|field|self.reference_target(field).is_some()) {
            self.begin(Focus::ReferenceSearch);
        }
        Ok(())
    }

    pub(super) fn apply_element(&mut self, field: &str, slot: usize) -> bool {
        let result = (|| {
            let mut array = self
                .field_value(field)
                .cloned()
                .ok_or("Missing field".to_owned())?;
            let old = array
                .get_mut(slot)
                .ok_or("Missing list element".to_owned())?;
            let parsed =
                parse_cell(old, &self.edit).map_err(|_| schema::type_error(old).to_owned())?;
            *old = parsed;
            if let Some(draft) = &mut self.draft {
                draft.value[field] = array;
            } else {
                let row = self.row.ok_or("Select a record".to_owned())?;
                let mut rows = self.rows().to_vec();
                rows[row][field] = array;
                self.change(rows)?;
            }
            Ok::<_, String>(())
        })();
        self.revision += 1;
        match result {
            Ok(()) => {
                self.array_slot = None;
                self.focus = None;
                self.status.clear();
                true
            }
            Err(error) => {
                self.status = error;
                false
            }
        }
    }
    pub(super) fn authoring_blocks(&self, value: &Value) -> Vec<Block> {
        let mut result = blocks(
            &self.tables[self.table].label,
            &self.columns,
            value,
            self.advanced,
        );
        if self.mission_table() {
            for (group,route,_,text,npc) in mail_fields::PAIRS {
                if value[*route].as_i64().unwrap_or(0)&6==6 {
                    if let Some(block)=result.iter_mut().find(|b|b.key==*group) {
                        for field in [*npc,*text] {if !block.fields.iter().any(|f|f==field){block.fields.push(field.into());}}
                    }
                }
            }
            if let Some(identity) = result.iter_mut().find(|b| b.key == "identity") {
                if !identity.fields.iter().any(|f| f == mission_visibility::FIELD) {
                    identity.fields.push(mission_visibility::FIELD.into());
                }
            }
        }
        if let Some(draft) = &self.draft {
            let missing: Vec<_> = draft
                .required
                .iter()
                .filter(|f| value.get(*f).is_some() && !result.iter().any(|b| b.fields.contains(f)))
                .cloned()
                .collect();
            if !missing.is_empty() {
                if let Some(other) = result.iter_mut().find(|b| b.key == "other") {
                    other.fields.extend(missing);
                } else {
                    result.push(Block {
                        key: "other",
                        configured: missing.iter().any(|f| !neutral(&value[f])),
                        fields: missing,
                        core: false,
                    });
                }
            }
        }
        result
    }
    pub(super) fn authoring_fields(&self, value: &Value) -> Vec<String> {
        self.authoring_blocks(value)
            .into_iter()
            .filter(|b| self.block_open(b))
            .flat_map(|b| b.fields)
            .filter(|f| {
                !self
                    .draft
                    .as_ref()
                    .and_then(|d| d.name.as_ref())
                    .is_some_and(|n| n.field == *f)
            })
            .collect()
    }
    pub(super) fn block_open(&self, block: &Block) -> bool {
        if self.mission_table() && self.draft.is_none() {
            return self.block_overrides.get(&(self.table,self.row,false,block.key.into())).copied().unwrap_or(false);
        }
        self.block_overrides
            .get(&(
                self.table,
                self.row,
                self.draft.is_some(),
                block.key.to_owned(),
            ))
            .copied()
            .unwrap_or(
                block.core
                    || block.configured
                    || self.advanced
                    || self
                        .draft
                        .as_ref()
                        .is_some_and(|d| block.fields.iter().any(|f| d.required.contains(f))),
            )
    }

    pub(super) fn text_target(&self, field: &str) -> Option<TextTarget> {
        let (table, id) = self.reference_target(field)?;
        if id.is_some() || !self.tables[table].label.ends_with("StringData") {
            return None;
        }
        let owner = self
            .draft
            .as_ref()
            .map(|d| &d.value)
            .or_else(|| self.row.and_then(|r| self.rows().get(r)))?;
        if self
            .draft
            .as_ref()
            .and_then(|d| d.name.as_ref())
            .is_some_and(|n| n.field == field)
        {
            return None;
        }
        let row = usize::try_from(owner.get(field)?.as_u64()?).ok()?;
        let target = self
            .document
            .pointer(&self.tables[table].pointer)?
            .get(row)?;
        let key = ["m_strName", "m_pstrNameString"]
            .into_iter()
            .find(|key| target.get(key).is_some_and(Value::is_string))?;
        let users = self
            .references
            .iter()
            .filter(|r| r.target_table == table && r.target_row == Some(row))
            .map(|r| (r.table, r.row))
            .collect::<BTreeSet<_>>()
            .len();
        Some(TextTarget {
            table,
            row,
            key: key.to_owned(),
            users,
        })
    }

    pub(super) fn edit_shared_text(&mut self, field: &str) -> Result<(), String> {
        if self.draft.is_some() {
            return Err("Finish or cancel the new record first".into());
        }
        let target = self.text_target(field).ok_or("Missing referenced text")?;
        self.navigate(target.table, target.row);
        self.begin(Focus::Cell(target.key));
        Ok(())
    }

    pub(super) fn copy_text(&mut self, field: &str) -> Result<(), String> {
        if self.draft.is_some() {
            return Err("Finish or cancel the new record first".into());
        }
        let owner = self.row.ok_or("Select a record")?;
        let target = self.text_target(field).ok_or("Missing referenced text")?;
        let mut next = self.document.clone();
        let rows = next
            .pointer_mut(&self.tables[target.table].pointer)
            .and_then(Value::as_array_mut)
            .ok_or("Missing text table")?;
        let before = rows.clone();
        let copied = rows
            .get(target.row)
            .cloned()
            .ok_or("Missing referenced text")?;
        let new_row = rows.len();
        rows.push(copied);
        validate_rows(&before, rows)?;
        next.pointer_mut(&self.tables[self.table].pointer)
            .and_then(|v| v.get_mut(owner))
            .ok_or("Missing owner")?[field] = Value::from(new_row);
        relations::validate_changes(&self.document, &next)?;
        self.undo.push(Change {
            pointer: String::new(),
            before: self.document.clone(),
            after: next.clone(),
        });
        self.redo.clear();
        self.document = next;
        self.reindex();
        self.navigate(target.table, new_row);
        self.begin(Focus::Cell(target.key));
        Ok(())
    }
}
