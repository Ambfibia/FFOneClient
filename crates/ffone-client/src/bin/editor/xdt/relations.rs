//! Declared routes mirror native mission/NPC/Nano consumers; other IDs remain candidates.
use super::*;

pub(super) struct Reference {
    pub table: usize,
    pub row: usize,
    pub field: String,
    pub value: i64,
    pub target_table: usize,
    pub target_row: Option<usize>,
}
pub(super) fn optional_index(field: &str) -> bool {
    field == "m_iBarkerNumber" || mission_journal::is_link(field)
        || matches!(field, "m_iSTDialogBubble" | "m_iSUDialogBubble" | "m_iFDialogBubble"
            | "m_iSTMessageTextID" | "m_iSUMessagetextID" | "m_iFMessageTextID")
}
pub(super) fn rule_for(table:&str,field:&str,row:&serde_json::Map<String,Value>,slot:Option<usize>)->Option<(String,Option<&'static str>)>{
    if table.ends_with("/m_pRewardData"){
        if let Some(types)=mission_reward::type_field(field){
            let code=row.get(types)?.get(slot.unwrap_or(0))?.as_i64()?;
            let target=mission_reward::item_table(code)?;
            return Some((format!("{}/{target}/m_pItemData",table.split('/').next()?),Some("m_iItemNumber")));
        }
    }
    rule(table,field)
}
pub(super) fn rule(table: &str, field: &str) -> Option<(String, Option<&'static str>)> {
    if table.contains("/m_pAnimationTable/") {
        return None;
    }
    let group = table.rsplit_once('/')?.0;
    let leaf = table.rsplit('/').next()?;
    if leaf == "m_pJournalData" && mission_journal::TEXT_FIELDS.contains(&field) {
        return Some((format!("{group}/m_pMissionStringData"), None));
    }
    if leaf == "m_pMissionData" && mission_journal::is_link(field) {
        return Some((format!("{group}/m_pJournalData"), None));
    }
    if leaf == "m_pItemData" && group.ends_with("ItemTable") {
        let target = match field {
            "m_iItemName" => Some("m_pItemStringData"),
            "m_iIcon" => Some("m_pItemIconData"),
            _ => None,
        };
        if let Some(target) = target {
            return Some((format!("{group}/{target}"), None));
        }
    }
    // These are zero-based indexes, not equality with the target row's icon/name fields.
    let indexed = match (leaf, field) {
        ("m_pNpcData", "m_iIcon1") => Some("m_pNpcIconData"),
        ("m_pNpcData", "m_iNpcName") => Some("m_pNpcStringData"),
        ("m_pNpcData", "m_iMesh") => Some("m_pNpcMeshData"),
        ("m_pNpcData", "m_iBarkerNumber") => Some("m_pNpcBarkerData"),
        ("m_pNanoData", "m_iIcon1") => Some("m_pNanoIconData"),
        ("m_pNanoData", "m_iNanoName") => Some("m_pNanoStringData"),
        ("m_pNanoData", "m_iMesh") => Some("m_pNanoMeshData"),
        (
            "m_pMissionData",
            "m_iHMissionName"
            | "m_iHCurrentObjective"
            | "m_iSTMessageTextID"
            | "m_iSUMessagetextID"
            | "m_iFMessageTextID",
        ) => Some("m_pMissionStringData"),
        (
            "m_pMissionData",
            "m_iSTDialogBubble" | "m_iSUDialogBubble" | "m_iFDialogBubble",
        ) => Some("m_pMissionStringData"),
        _ => None,
    };
    if let Some(target) = indexed {
        return Some((format!("{group}/{target}"), None));
    }
    let root = table.split('/').next()?;
    if leaf == "m_pMissionData" {
        if matches!(field,"m_iCSUItemID"|"m_iCSTItemID"|"m_iSTItemID"|"m_iSUItem"|"m_iFItemID"|"m_iDelItemID") {
            return Some((format!("{root}/m_pQuestItemTable/m_pItemData"),Some("m_iItemNumber")));
        }
        if field == "m_iCSTReqMission" {
            return Some((table.to_owned(), Some("m_iHMissionID")));
        }
        let npc = matches!(
            field,
            "m_iHNPCID"
                | "m_iHTerminatorNPCID"
                | "m_iHJournalNPCID"
                | "m_iCSUDEFNPCID"
                | "m_iSTMessageSendNPC"
                | "m_iSUMessageSendNPC"
                | "m_iFMessageSendNPC"
                | "m_iSTDialogBubbleNPCID"
                | "m_iSUDialogBubbleNPCID"
                | "m_iFDialogBubbleNPCID"
                | "m_iCSUEnemyID"
                | "m_iSTGrantWayPoint"
        );
        if npc {
            return Some((
                format!("{root}/m_pNpcTable/m_pNpcData"),
                Some("m_iNpcNumber"),
            ));
        }
        if matches!(
            field,
            "m_iCSTTrigger" | "m_iSUOutgoingTask" | "m_iFOutgoingTask"
        ) {
            return Some((table.to_owned(), Some("m_iHTaskID")));
        }
        if field == "m_iSUReward" {
            return Some((format!("{group}/m_pRewardData"), Some("m_iMissionRewardID")));
        }
        if matches!(field, "m_iSTNanoID" | "m_iCSTRReqNano") {
            return Some((
                format!("{root}/m_pNanoTable/m_pNanoData"),
                Some("m_iNanoNumber"),
            ));
        }
    }
    None
}
pub(super) fn collect(document: &Value, tables: &[Table]) -> Vec<Reference> {
    let names: BTreeMap<_, _> = tables
        .iter()
        .enumerate()
        .map(|(i, t)| (t.label.as_str(), i))
        .collect();
    let mut ids = BTreeMap::new();
    for (t, table) in tables.iter().enumerate() {
        for (r, row) in document
            .pointer(&table.pointer)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            if let Some(row) = row.as_object() {
                for (field, value) in row {
                    if let Some(id) = value.as_i64() {
                        ids.entry((t, field.as_str(), id)).or_insert(r);
                    }
                }
            }
        }
    }
    let mut result = Vec::new();
    for (t, table) in tables.iter().enumerate() {
        for (r, row) in document
            .pointer(&table.pointer)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let Some(row) = row.as_object() else { continue };
            for (field, value) in row {
                let values: Vec<(Option<usize>, i64)> = if let Some(id) = value.as_i64() {
                    vec![(None, id)]
                } else {
                    value
                        .as_array()
                        .into_iter()
                        .flatten()
                        .enumerate()
                        .filter_map(|(i, v)| v.as_i64().map(|id| (Some(i), id)))
                        .collect()
                };
                for (slot, value) in values {
                    let Some((target,identity))=rule_for(&table.label,field,row,slot)else{continue};
                    let Some(&target_table)=names.get(target.as_str())else{continue};
                    let field=slot.map_or_else(||field.clone(),|i|format!("{field}[{i}]"));
                    if value < 0
                        || (identity.is_some() && value == 0)
                        || optional_index(&field) && value == 0
                    {
                        continue;
                    }
                    let target_row = if let Some(identity) = identity {
                        ids.get(&(target_table, identity, value)).copied()
                    } else {
                        let length = document
                            .pointer(&tables[target_table].pointer)
                            .and_then(Value::as_array)
                            .map_or(0, Vec::len);
                        usize::try_from(value).ok().filter(|i| *i < length)
                    };
                    result.push(Reference {
                        table: t,
                        row: r,
                        field,
                        value,
                        target_table,
                        target_row,
                    });
                }
            }
        }
    }
    result
}
pub(super) fn validate_changes(base: &Value, draft: &Value) -> Result<(), String> {
    fn tables(document: &Value) -> Vec<Table> {
        let mut result = Vec::new();
        if let Some(tables) = document["tables"].as_array() {
            fn walk(value: &Value, pointer: String, label: String, result: &mut Vec<Table>) {
                if let Some(rows) = value.as_array() {
                    if rows.iter().all(Value::is_object) {
                        result.push(Table { pointer, label });
                    }
                } else if let Some(fields) = value.as_object() {
                    for (key, v) in fields {
                        walk(
                            v,
                            format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")),
                            format!("{label}/{key}"),
                            result,
                        );
                    }
                }
            }
            for (i, t) in tables.iter().enumerate() {
                walk(
                    &t["value"],
                    format!("/tables/{i}/value"),
                    t["name"].as_str().unwrap_or("").into(),
                    &mut result,
                );
            }
        }
        result
    }
    let old_tables = tables(base);
    let new_tables = tables(draft);
    let old = collect(base, &old_tables);
    let new = collect(draft, &new_tables);
    let by_source: BTreeMap<_, _> = new
        .iter()
        .map(|r| {
            (
                (
                    new_tables[r.table].label.as_str(),
                    r.row,
                    r.field.as_str(),
                    r.value,
                ),
                r,
            )
        })
        .collect();
    for reference in &old {
        let Some(old_row) = reference.target_row else {
            continue;
        };
        if rule(
            &old_tables[reference.table].label,
            reference.field.split('[').next().unwrap_or(""),
        )
        .is_some_and(|(_, id)| id.is_none())
        {
            let target = &old_tables[reference.target_table];
            let Some(next_target) = new_tables.iter().find(|t| t.label == target.label) else {
                continue;
            };
            let before = base
                .pointer(&target.pointer)
                .and_then(Value::as_array)
                .unwrap();
            let after = draft
                .pointer(&next_target.pointer)
                .and_then(Value::as_array)
                .unwrap();
            if let Some(next_ref) = by_source.get(&(
                old_tables[reference.table].label.as_str(),
                reference.row,
                reference.field.as_str(),
                reference.value,
            )) {
                if next_ref.target_row.is_some_and(|i| {
                    before.get(old_row) != after.get(i)
                        && (after.len() < before.len()
                            || after.iter().any(|row| Some(row) == before.get(old_row)))
                }) {
                    return Err(format!(
                        "Changing table order shifts referenced indexes: {} #{}. Update {} #{} {} explicitly.",
                        target.label,
                        old_row + 1,
                        old_tables[reference.table].label,
                        reference.row + 1,
                        reference.field
                    ));
                }
            }
        }
    }
    drop(by_source);
    let missing: BTreeSet<_> = old
        .iter()
        .filter(|r| r.target_row.is_none())
        .map(|r| {
            (
                old_tables[r.table].label.clone(),
                r.row,
                r.field.clone(),
                r.value,
            )
        })
        .collect();
    for r in new {
        if r.target_row.is_none()
            && !missing.contains(&(
                new_tables[r.table].label.clone(),
                r.row,
                r.field.clone(),
                r.value,
            ))
        {
            return Err(format!(
                "Broken reference: {} #{} {}={} → {}",
                new_tables[r.table].label,
                r.row + 1,
                r.field,
                r.value,
                new_tables[r.target_table].label
            ));
        }
    }
    Ok(())
}
