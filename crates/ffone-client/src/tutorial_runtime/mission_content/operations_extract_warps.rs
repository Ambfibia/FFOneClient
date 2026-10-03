use super::*;

pub(super) fn extract_npcs(
    rows: &[Value],
    strings: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, TutorialNpcDefinition>> {
    let required = TUTORIAL_MISSION_NPC_TYPES
        .into_iter()
        .collect::<BTreeSet<_>>();
    let tutorial_actor_types = tutorial_actor_npc_types();
    let mut npcs = BTreeMap::new();

    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pNpcData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
        // Keep every XDT row instantiated by tutorial choreography so target HUD
        // labels resolve through m_iNpcName -> m_pNpcStringData as in the
        // original client. Only mission-facing NPCs remain mandatory below,
        // which keeps compact/custom table sets valid.
        if !required.contains(&npc_type) && !tutorial_actor_types.contains(&npc_type) {
            continue;
        }
        let name_string_id = required_i32(row, "m_iNpcName", &context)?;
        let name_row_index = usize::try_from(name_string_id).map_err(|_| {
            invalid(format!(
                "{context}.m_iNpcName is negative: {name_string_id}"
            ))
        })?;
        let name = indexed_string(
            strings,
            name_string_id,
            "m_strName",
            &format!("{context}.m_iNpcName"),
        )?;
        let definition = TutorialNpcDefinition {
            name,
            provenance: TutorialNpcRowProvenance {
                row_index,
                npc_type,
                npc_class: required_i32(row, "m_iNpcType", &context)?,
                name_string_id,
                name_row_index,
            },
        };
        if npcs.insert(npc_type, definition).is_some() {
            return Err(invalid(format!(
                "duplicate tutorial NPC type {npc_type} in m_pNpcData"
            )));
        }
    }

    for npc_type in required {
        if !npcs.contains_key(&npc_type) {
            return Err(invalid(format!(
                "missing tutorial NPC type {npc_type} in m_pNpcData"
            )));
        }
    }
    Ok(npcs)
}

pub(super) fn extract_gameplay_warps(
    rows: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, GameplayWarpDefinition>> {
    let mut warps = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pWarpData[{row_index}]");
        let row = value_object(row, &context)?;
        let definition = GameplayWarpDefinition {
            row_index,
            warp_id: required_i32(row, "m_iWarpNumber", &context)?,
            npc_type: required_i32(row, "m_iNpcNumber", &context)?,
            target: TutorialWarpTarget {
                map_id: required_i32(row, "m_iToMapNum", &context)?,
                x: required_i32(row, "m_iToX", &context)?,
                y: required_i32(row, "m_iToY", &context)?,
                z: required_i32(row, "m_iToZ", &context)?,
            },
            warp_group_type: required_i32(row, "m_iWarpGroupType", &context)?,
            limit_level: required_i32(row, "m_iLimit_Level", &context)?,
            limit_task_id: required_i32(row, "m_iLimit_TaskID", &context)?,
            limit_item_id: required_i32(row, "m_iLimit_ItemID", &context)?,
            limit_item_type: required_i32(row, "m_iLimit_ItemType", &context)?,
            limit_use_item_id: required_i32(row, "m_iLimit_UseItemID", &context)?,
            limit_use_item_type: required_i32(row, "m_iLimit_UseItemType", &context)?,
            mission_id: required_i32(row, "m_iMissionID", &context)?,
            is_instance: required_i32(row, "m_iIsInstance", &context)?,
            cost: required_i32(row, "m_iCost", &context)?,
        };
        if warps.insert(definition.warp_id, definition).is_some() {
            return Err(invalid(format!(
                "duplicate gameplay warp ID {} in m_pWarpData",
                definition.warp_id
            )));
        }
    }

    let guide = warps.get(&GUIDE_FIRST_WARP_ID).ok_or_else(|| {
        invalid(format!(
            "missing clean Guide first-change warp ID {GUIDE_FIRST_WARP_ID}"
        ))
    })?;
    let zero_gates = [
        guide.warp_group_type,
        guide.limit_level,
        guide.limit_task_id,
        guide.limit_item_id,
        guide.limit_item_type,
        guide.limit_use_item_id,
        guide.limit_use_item_type,
        guide.mission_id,
        guide.is_instance,
        guide.cost,
    ];
    if guide.npc_type != GUIDE_FIRST_WARP_NPC_TYPE
        || guide.target != GUIDE_FIRST_WARP_TARGET
        || zero_gates != [0; 10]
    {
        return Err(invalid(format!(
            "Guide first-change warp contradicts source contract: expected \
             npc/target/zero-gates={GUIDE_FIRST_WARP_NPC_TYPE}/{GUIDE_FIRST_WARP_TARGET:?}/{:?}, \
             found {}/{:?}/{zero_gates:?}",
            [0; 10], guide.npc_type, guide.target
        )));
    }
    Ok(warps)
}

pub(super) fn extract_warps(
    rows: &[Value],
    npcs: &BTreeMap<i32, TutorialNpcDefinition>,
    missions: &BTreeMap<i32, TutorialMissionDefinition>,
) -> TutorialMissionContentResult<BTreeMap<i32, TutorialWarpDefinition>> {
    let required = TUTORIAL_WARP_NPC_TYPES.into_iter().collect::<BTreeSet<_>>();
    let mut warps = BTreeMap::new();

    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pWarpData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
        if !required.contains(&npc_type) {
            continue;
        }
        let raw_limit_task_id = required_i32(row, "m_iLimit_TaskID", &context)?;
        let target = TutorialWarpTarget {
            map_id: required_i32(row, "m_iToMapNum", &context)?,
            x: required_i32(row, "m_iToX", &context)?,
            y: required_i32(row, "m_iToY", &context)?,
            z: required_i32(row, "m_iToZ", &context)?,
        };
        let provenance = TutorialWarpRowProvenance {
            row_index,
            npc_type,
            warp_id: required_i32(row, "m_iWarpNumber", &context)?,
            raw_limit_task_id,
            mission_id: required_i32(row, "m_iMissionID", &context)?,
            is_instance: required_i32(row, "m_iIsInstance", &context)?,
            cost: required_i32(row, "m_iCost", &context)?,
        };
        if raw_limit_task_id != 0 && !missions.contains_key(&raw_limit_task_id) {
            return Err(invalid(format!(
                "{context} references missing tutorial task gate {raw_limit_task_id}"
            )));
        }
        let label = npcs
            .get(&npc_type)
            .ok_or_else(|| invalid(format!("{context} has no tutorial NPC name source")))?
            .name
            .clone();
        let definition = TutorialWarpDefinition {
            label,
            target,
            required_task_id: (raw_limit_task_id != 0).then_some(raw_limit_task_id),
            provenance,
        };
        if warps.insert(npc_type, definition).is_some() {
            return Err(invalid(format!(
                "duplicate tutorial warp for NPC type {npc_type}"
            )));
        }
    }

    for expected in EXPECTED_WARPS {
        let warp = warps.get(&expected.npc_type).ok_or_else(|| {
            invalid(format!(
                "missing tutorial warp for NPC type {}",
                expected.npc_type
            ))
        })?;
        if warp.provenance.warp_id != expected.warp_id
            || warp.provenance.raw_limit_task_id != expected.required_task_id
            || warp.target != expected.target
        {
            return Err(invalid(format!(
                "tutorial warp for NPC type {} contradicts source contract: \
                 expected id/task/target={}/{}/{:?}, found {}/{}/{:?}",
                expected.npc_type,
                expected.warp_id,
                expected.required_task_id,
                expected.target,
                warp.provenance.warp_id,
                warp.provenance.raw_limit_task_id,
                warp.target
            )));
        }
    }
    Ok(warps)
}

pub(super) fn indexed_string(
    rows: &[Value],
    source_id: i32,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<String> {
    let value = indexed_string_allow_empty(rows, source_id, field, context)?;
    if value.trim().is_empty() {
        return Err(invalid(format!(
            "{context} resolves to an empty {field} string"
        )));
    }
    Ok(value)
}

/// Resolves a source string while preserving a legitimately empty value.
///
/// `NpcStringTable.m_strComment2` is optional voice ownership data in the
/// clean table. Many valid NPC rows reference an existing string row whose
/// comment is empty; that means "no move voice", not malformed TableData.
/// Identity, mission and presentation strings continue to use
/// [`indexed_string`] and therefore remain non-empty.
pub(super) fn indexed_string_allow_empty(
    rows: &[Value],
    source_id: i32,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<String> {
    let index = usize::try_from(source_id)
        .map_err(|_| invalid(format!("{context} has negative string ID {source_id}")))?;
    let row = rows.get(index).ok_or_else(|| {
        invalid(format!(
            "{context} references missing string row {source_id}"
        ))
    })?;
    let row = value_object(row, &format!("{context} -> string[{source_id}]"))?;
    let value = required_string(row, field, context)?;
    Ok(value.to_owned())
}

pub(super) fn required_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<&'a Vec<Value>> {
    object
        .get(field)
        .ok_or_else(|| invalid(format!("{context} has no {field}")))?
        .as_array()
        .ok_or_else(|| invalid(format!("{context}.{field} must be an array")))
}

pub(super) fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<&'a str> {
    object
        .get(field)
        .ok_or_else(|| invalid(format!("{context} has no {field}")))?
        .as_str()
        .ok_or_else(|| invalid(format!("{context}.{field} must be a string")))
}

pub(super) fn required_i32(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<i32> {
    let value = object
        .get(field)
        .ok_or_else(|| invalid(format!("{context} has no {field}")))?
        .as_i64()
        .ok_or_else(|| invalid(format!("{context}.{field} must be an integer")))?;
    i32::try_from(value).map_err(|_| invalid(format!("{context}.{field} is outside i32: {value}")))
}

pub(super) fn required_i32_array2(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<[i32; 2]> {
    let values = required_array(object, field, context)?;
    let [first, second] = values.as_slice() else {
        return Err(invalid(format!(
            "{context}.{field} must contain exactly two integers"
        )));
    };
    let parse = |value: &Value, index: usize| {
        let value = value
            .as_i64()
            .ok_or_else(|| invalid(format!("{context}.{field}[{index}] must be an integer")))?;
        i32::try_from(value).map_err(|_| {
            invalid(format!(
                "{context}.{field}[{index}] is outside i32: {value}"
            ))
        })
    };
    Ok([parse(first, 0)?, parse(second, 1)?])
}

pub(super) fn required_i32_array3(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<[i32; 3]> {
    let values = required_array(object, field, context)?;
    let [first, second, third] = values.as_slice() else {
        return Err(invalid(format!(
            "{context}.{field} must contain exactly three integers"
        )));
    };
    let parse = |value: &Value, index: usize| {
        let value = value
            .as_i64()
            .ok_or_else(|| invalid(format!("{context}.{field}[{index}] must be an integer")))?;
        i32::try_from(value).map_err(|_| {
            invalid(format!(
                "{context}.{field}[{index}] is outside i32: {value}"
            ))
        })
    };
    Ok([parse(first, 0)?, parse(second, 1)?, parse(third, 2)?])
}

pub(super) fn required_i32_array4(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<[i32; 4]> {
    let values = required_array(object, field, context)?;
    let [first, second, third, fourth] = values.as_slice() else {
        return Err(invalid(format!(
            "{context}.{field} must contain exactly four integers"
        )));
    };
    let parse = |value: &Value, index: usize| {
        let value = value
            .as_i64()
            .ok_or_else(|| invalid(format!("{context}.{field}[{index}] must be an integer")))?;
        i32::try_from(value).map_err(|_| {
            invalid(format!(
                "{context}.{field}[{index}] is outside i32: {value}"
            ))
        })
    };
    Ok([
        parse(first, 0)?,
        parse(second, 1)?,
        parse(third, 2)?,
        parse(fourth, 3)?,
    ])
}

pub(super) fn required_i32_array5(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<[i32; 5]> {
    let values = required_array(object, field, context)?;
    let [first, second, third, fourth, fifth] = values.as_slice() else {
        return Err(invalid(format!(
            "{context}.{field} must contain exactly five integers"
        )));
    };
    let parse = |value: &Value, index: usize| {
        let value = value
            .as_i64()
            .ok_or_else(|| invalid(format!("{context}.{field}[{index}] must be an integer")))?;
        i32::try_from(value).map_err(|_| {
            invalid(format!(
                "{context}.{field}[{index}] is outside i32: {value}"
            ))
        })
    };
    Ok([
        parse(first, 0)?,
        parse(second, 1)?,
        parse(third, 2)?,
        parse(fourth, 3)?,
        parse(fifth, 4)?,
    ])
}

pub(super) fn invalid(detail: impl Into<String>) -> TutorialMissionContentError {
    TutorialMissionContentError::Invalid(detail.into())
}

pub(super) fn extract_vehicle_engine_sounds(root: &Map<String, Value>) -> BTreeMap<i16, String> {
    let mut result = BTreeMap::new();
    let Some(table) = root.get("m_pVehicleItemTable") else {
        return result;
    };
    let (Some(items), Some(sounds)) = (
        table.get("m_pItemData").and_then(Value::as_array),
        table.get("m_pItemSoundData").and_then(Value::as_array),
    ) else {
        return result;
    };
    for (index, item) in items.iter().enumerate() {
        let Some(sound) = item
            .get("m_iSound1")
            .and_then(Value::as_u64)
            .and_then(|i| sounds.get(i as usize))
            .and_then(|s| s.get("m_pstrSoundString1"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let name = sound.trim().rsplit(['/', '\\']).next().unwrap_or_default();
        let name = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
        if !name.is_empty() && !name.eq_ignore_ascii_case("null") {
            if let Ok(id) = i16::try_from(index) {
                result.insert(id, name.to_owned());
            }
        }
    }
    result
}
