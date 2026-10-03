use super::*;

pub(super) fn append_user_equip_icon_table(
    catalog: &mut BTreeMap<(u8, u8, i32, u8), GameplayUserEquipIconDefinition>,
    table: &Map<String, Value>,
    spec: UserEquipIconTableSpec,
    installed_icon_paths: &BTreeSet<String>,
) -> TutorialMissionContentResult<()> {
    let Some(icon_rows) = table.get(spec.icon_rows_key) else {
        if spec.require_icon_rows_when_table_present {
            return Err(invalid(format!(
                "{} has no required {}",
                spec.table_key, spec.icon_rows_key
            )));
        }
        return Ok(());
    };
    let icon_rows = icon_rows.as_array().ok_or_else(|| {
        invalid(format!(
            "{}.{} must be an array",
            spec.table_key, spec.icon_rows_key
        ))
    })?;
    let mut icons = Vec::with_capacity(icon_rows.len());
    for (icon_row_index, icon) in icon_rows.iter().enumerate() {
        let context = format!(
            "{}.{}[{icon_row_index}]",
            spec.table_key, spec.icon_rows_key
        );
        let icon = value_object(icon, &context)?;
        let raw_icon_type = required_i32(icon, "m_iIconType", &context)?;
        let icon_type = u8::try_from(raw_icon_type).map_err(|_| {
            invalid(format!(
                "{context}.m_iIconType is outside u8: {raw_icon_type}"
            ))
        })?;
        let raw_icon_number = required_i32(icon, "m_iIconNumber", &context)?;
        let icon_number = u32::try_from(raw_icon_number).map_err(|_| {
            invalid(format!(
                "{context}.m_iIconNumber must be non-negative, found {raw_icon_number}"
            ))
        })?;
        let icon_path = avatar_util_semantic_icon_path(icon_type, icon_number)
            .filter(|path| installed_icon_paths.contains(path.as_str()));
        icons.push((icon_type, icon_number, icon_path));
    }

    let rows = required_array(table, spec.item_rows_key, spec.table_key)?;
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("{}.{}[{row_index}]", spec.table_key, spec.item_rows_key);
        let row = value_object(row, &context)?;
        let declared_item_number = required_i32(row, spec.declared_item_field, &context)?;
        if declared_item_number < 0 {
            return Err(invalid(format!(
                "{context}.{} must be non-negative, found {declared_item_number}",
                spec.declared_item_field
            )));
        }
        let icon_row_id = required_i32(row, spec.item_icon_field, &context)?;
        let icon_index = usize::try_from(icon_row_id).map_err(|_| {
            invalid(format!(
                "{context}.{} must be non-negative, found {icon_row_id}",
                spec.item_icon_field
            ))
        })?;
        let (icon_type, icon_number, icon_path) =
            icons.get(icon_index).cloned().ok_or_else(|| {
                invalid(format!(
                    "{context}.{} references missing {} row {icon_row_id}",
                    spec.item_icon_field, spec.icon_rows_key
                ))
            })?;
        let item_row_id = i32::try_from(row_index).map_err(|_| {
            invalid(format!(
                "{context} row index does not fit the clean TableData i32"
            ))
        })?;
        let definition = GameplayUserEquipIconDefinition {
            item_table: spec.item_table,
            item_subtable: spec.item_subtable,
            item_row_id,
            declared_item_number,
            icon_subtable: spec.icon_subtable,
            icon_row_id,
            icon_type,
            icon_number,
            icon_path,
        };
        let key = (
            spec.item_table,
            spec.item_subtable,
            item_row_id,
            spec.icon_subtable,
        );
        if catalog.insert(key, definition).is_some() {
            return Err(invalid(format!(
                "duplicate UserEquip TableData route ({}, {}, {}, {})",
                spec.item_table, spec.item_subtable, item_row_id, spec.icon_subtable
            )));
        }
    }
    Ok(())
}

pub(super) fn primary_nano_icon_slug(icon_number: u32) -> Option<&'static str> {
    Some(match icon_number {
        0 => "samurai-jack",
        1 => "eduardo",
        2 => "mojo-jojo",
        3 => "numbuh-five",
        4 => "mandark",
        5 => "fourarms",
        6 => "billy",
        7 => "blossom",
        8 => "dexter",
        9 => "ed",
        10 => "grim",
        11 => "juniper-lee",
        12 => "mandy",
        13 => "aku",
        14 => "bloo",
        15 => "coco",
        16 => "courage",
        17 => "dee-dee",
        18 => "demongo",
        19 => "edd",
        20 => "eddy",
        21 => "humongosaur",
        22 => "hex",
        23 => "him",
        24 => "mac",
        25 => "megas",
        26 => "numbuh-one",
        27 => "numbuh-four",
        28 => "numbuh-two",
        29 => "numbuh-three",
        30 => "swampfire",
        31 => "prof-utonium",
        32 => "vilgax",
        33 => "wilt",
        34 => "buttercup",
        35 => "bubbles",
        36 => "missing",
        37 => "ben-10",
        38 => "johnny-bravo",
        39 => "cheese",
        40 => "finn",
        41 => "flapjack",
        42 => "holo-nano",
        43 => "belladonna",
        44 => "computress",
        45 => "runty",
        46 => "finn",
        47 => "coop",
        48 => "panini",
        49 => "jack-olantern",
        _ => return None,
    })
}

pub(super) fn extract_gameplay_general_items(
    rows: &[Value],
    icon_rows: Option<&Vec<Value>>,
) -> TutorialMissionContentResult<BTreeMap<i16, GameplayGeneralItemUiDefinition>> {
    let mut items = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pGeneralItemTable.m_pItemData[{row_index}]");
        let row = value_object(row, &context)?;
        let raw_item_id = required_i32(row, "m_iItemNumber", &context)?;
        let item_id = i16::try_from(raw_item_id).map_err(|_| {
            invalid(format!(
                "{context}.m_iItemNumber {raw_item_id} does not fit protocol i16"
            ))
        })?;
        if item_id < 0 {
            return Err(invalid(format!(
                "{context}.m_iItemNumber must be non-negative, found {item_id}"
            )));
        }
        let item_type = required_i32(row, "m_iItemType", &context)?;
        let link_skill = optional_i32(row, "m_iLinkSkill", &context)?;
        let stim_pack_attribute = optional_i32(row, "m_iStimPackAttri", &context)?;
        let icon_index = optional_i32(row, "m_iIcon", &context)?;
        let (icon_type, icon_number, icon_path) = match (icon_index, icon_rows) {
            (Some(icon_index), Some(icon_rows)) => {
                let icon_index = usize::try_from(icon_index).map_err(|_| {
                    invalid(format!(
                        "{context}.m_iIcon must be non-negative, found {icon_index}"
                    ))
                })?;
                let icon_context = format!("{context}.m_iIcon -> m_pItemIconData[{icon_index}]");
                let icon = icon_rows.get(icon_index).ok_or_else(|| {
                    invalid(format!(
                        "{context}.m_iIcon references missing icon row {icon_index}"
                    ))
                })?;
                let icon = value_object(icon, &icon_context)?;
                let raw_icon_type = required_i32(icon, "m_iIconType", &icon_context)?;
                let raw_icon_number = required_i32(icon, "m_iIconNumber", &icon_context)?;
                let icon_type = u8::try_from(raw_icon_type).map_err(|_| {
                    invalid(format!(
                        "{icon_context}.m_iIconType is outside u8: {raw_icon_type}"
                    ))
                })?;
                let icon_number = u32::try_from(raw_icon_number).map_err(|_| {
                    invalid(format!(
                        "{icon_context}.m_iIconNumber must be non-negative, found {raw_icon_number}"
                    ))
                })?;
                let icon_path = general_item_semantic_icon_path(icon_type, icon_number);
                (Some(icon_type), Some(icon_number), icon_path)
            }
            // Compact authored fixtures may intentionally omit the optional
            // icon table. The semantic subtype remains usable by Resurrect.
            _ => (None, None, None),
        };
        let definition = GameplayGeneralItemUiDefinition {
            item_id,
            item_type,
            link_skill,
            stim_pack_attribute,
            icon_type,
            icon_number,
            icon_path,
        };
        if let Some(previous) = items.insert(item_id, definition.clone())
            && previous != definition
        {
            return Err(invalid(format!(
                "contradictory GeneralItem rows for item {item_id}: {previous:?} vs {definition:?}"
            )));
        }
    }
    Ok(items)
}

pub(super) fn extract_quest_item_names(
    rows: &[Value],
    strings: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, String>> {
    let mut names = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pQuestItemTable.m_pItemData[{row_index}]");
        let row = value_object(row, &context)?;
        let item_id = required_i32(row, "m_iItemNumber", &context)?;
        if item_id <= 0 {
            continue;
        }
        let name_id = required_i32(row, "m_iItemName", &context)?;
        let name = indexed_string(
            strings,
            name_id,
            "m_strName",
            &format!("{context}.m_iItemName"),
        )?;
        if let Some(previous) = names.insert(item_id, name.clone())
            && previous != name
        {
            return Err(invalid(format!(
                "contradictory quest-item names for item {item_id}: {previous:?} vs {name:?}"
            )));
        }
    }
    Ok(names)
}

pub(super) fn optional_i32(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<Option<i32>> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    let value = value
        .as_i64()
        .ok_or_else(|| invalid(format!("{context}.{field} must be an integer")))?;
    i32::try_from(value)
        .map(Some)
        .map_err(|_| invalid(format!("{context}.{field} is outside i32: {value}")))
}

pub(super) fn optional_xdt_f32(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<Option<XdtF32>> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    let value = value
        .as_f64()
        .ok_or_else(|| invalid(format!("{context}.{field} must be a number")))?
        as f32;
    if !value.is_finite() {
        return Err(invalid(format!(
            "{context}.{field} must be finite, found {value}"
        )));
    }
    Ok(Some(XdtF32::from_value(value)))
}

pub(super) fn extract_system_messages(
    rows: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, SystemMessageDefinition>> {
    let mut definitions = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        // Compact table subsets retain direct-index identity with null holes.
        // A hole is an unresolved source row, never a request for fallback
        // text. The production Retrobution table contains concrete rows.
        if row.is_null() {
            continue;
        }
        let row_id = i32::try_from(row_index).map_err(|_| {
            invalid(format!(
                "m_pMessageTable.m_pMessageData row index {row_index} is outside i32"
            ))
        })?;
        let context = format!("m_pMessageTable.m_pMessageData[{row_index}]");
        let row = value_object(row, &context)?;
        let raw_button_type = required_i32(row, "m_iButtonType", &context)?;
        let runtime_button_type =
            SystemMessageButtonType::try_from(raw_button_type).map_err(|error| {
                invalid(format!(
                    "{context}.m_iButtonType cannot enter the native runtime: {error}"
                ))
            })?;
        definitions.insert(
            row_id,
            SystemMessageDefinition {
                row_id,
                exact_text: required_string(row, "m_szString", &context)?.to_owned(),
                raw_button_type,
                runtime_button_type,
            },
        );
    }
    Ok(definitions)
}

pub(super) fn extract_gameplay_xcoms(
    rows: &[Value],
) -> TutorialMissionContentResult<Vec<GameplayXcomDefinition>> {
    rows.iter()
        .enumerate()
        .map(|(row_index, row)| {
            let context = format!("m_pXComData[{row_index}]");
            let row = value_object(row, &context)?;
            Ok(GameplayXcomDefinition {
                row_index: i32::try_from(row_index).map_err(|_| {
                    invalid(format!("{context} index does not fit the protocol i32"))
                })?,
                xcom_number: required_i32(row, "m_iXcomNumber", &context)?,
                zone: required_i32(row, "m_iZone", &context)?,
                position: [
                    required_i32(row, "m_iXpos", &context)?,
                    required_i32(row, "m_iYpos", &context)?,
                    required_i32(row, "m_iZpos", &context)?,
                ],
            })
        })
        .collect()
}

pub(super) fn extract_gameplay_npc_services(
    npc_table: &Map<String, Value>,
    npc_rows: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, (i32, String)>> {
    let Some(service_rows) = npc_table.get("m_pNpcServiceData") else {
        return Ok(BTreeMap::new());
    };
    let service_rows = service_rows
        .as_array()
        .ok_or_else(|| invalid("m_pNpcTable.m_pNpcServiceData must be an array"))?;
    let mut services = BTreeMap::new();
    for (row_index, row) in npc_rows.iter().enumerate() {
        let context = format!("m_pNpcTable.m_pNpcData[{row_index}]");
        let row = value_object(row, &context)?;
        let Some(service_number) = optional_i32(row, "m_iServiceNumber", &context)? else {
            continue;
        };
        let npc_number = required_i32(row, "m_iNpcNumber", &context)?;
        if npc_number <= 0 {
            continue;
        }
        let service_index = usize::try_from(service_number).map_err(|_| {
            invalid(format!(
                "{context}.m_iServiceNumber must be non-negative, found {service_number}"
            ))
        })?;
        let service_context =
            format!("{context}.m_iServiceNumber -> m_pNpcServiceData[{service_index}]");
        let service = service_rows.get(service_index).ok_or_else(|| {
            invalid(format!(
                "{context}.m_iServiceNumber references missing service row {service_index}"
            ))
        })?;
        let service = value_object(service, &service_context)?;
        let text = required_string(service, "m_strService", &service_context)?.to_owned();
        if let Some(previous) = services.insert(npc_number, (service_number, text.clone()))
            && previous != (service_number, text.clone())
        {
            return Err(invalid(format!(
                "contradictory NPC service rows for NPC {npc_number}: {previous:?} vs {text:?}"
            )));
        }
    }
    Ok(services)
}

pub(super) fn extract_gameplay_vendors(
    rows: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, Vec<GameplayVendorItemDefinition>>> {
    let mut vendors: BTreeMap<i32, Vec<GameplayVendorItemDefinition>> = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pVendorTable.m_pItemData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_number = required_i32(row, "m_iNpcNumber", &context)?;
        if npc_number <= 0 {
            return Err(invalid(format!(
                "{context}.m_iNpcNumber must be positive, found {npc_number}"
            )));
        }
        let raw_item_type = required_i32(row, "m_iItemType", &context)?;
        let item_type = i16::try_from(raw_item_type).map_err(|_| {
            invalid(format!(
                "{context}.m_iItemType {raw_item_type} does not fit protocol i16"
            ))
        })?;
        if item_type < 0 {
            return Err(invalid(format!(
                "{context}.m_iItemType must be non-negative, found {item_type}"
            )));
        }
        let raw_item_id = required_i32(row, "m_iitemID", &context)?;
        let item_id = i16::try_from(raw_item_id).map_err(|_| {
            invalid(format!(
                "{context}.m_iitemID {raw_item_id} does not fit protocol i16"
            ))
        })?;
        if item_id <= 0 {
            return Err(invalid(format!(
                "{context}.m_iitemID must be positive, found {item_id}"
            )));
        }
        let sort_number = required_i32(row, "m_iSortNumber", &context)?;
        if sort_number < 0 {
            return Err(invalid(format!(
                "{context}.m_iSortNumber must be non-negative, found {sort_number}"
            )));
        }
        vendors
            .entry(npc_number)
            .or_default()
            .push(GameplayVendorItemDefinition {
                row_index,
                npc_number,
                sort_number,
                item_type,
                item_id,
                sell_cost: required_i32(row, "m_iSellCost", &context)?,
            });
    }
    // The four placed early-world guide shops have the same service and
    // guide-specific wares as their later-world counterparts, but no rows of
    // their own in the 0104 VendorTable. Keep their own NPC identities.
    for (early, later) in [(643, 644), (645, 646), (647, 648), (649, 650)] {
        if !vendors.contains_key(&early)
            && let Some(rows) = vendors.get(&later)
        {
            vendors.insert(
                early,
                rows.iter()
                    .cloned()
                    .map(|mut item| {
                        item.npc_number = early;
                        item
                    })
                    .collect(),
            );
        }
    }
    Ok(vendors)
}

pub(super) fn extract_gameplay_npc_map_icons(
    rows: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, i32>> {
    let mut map_icons = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pNpcData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
        if npc_type <= 0 {
            continue;
        }
        let Some(map_icon) = optional_i32(row, "m_iMapIcon", &context)? else {
            continue;
        };
        if let Some(previous) = map_icons.insert(npc_type, map_icon)
            && previous != map_icon
        {
            return Err(invalid(format!(
                "contradictory m_iMapIcon rows for gameplay NPC type {npc_type}: {previous} vs {map_icon}"
            )));
        }
    }
    Ok(map_icons)
}

pub(super) fn extract_gameplay_npc_sounds(rows: &[Value]) -> TutorialMissionContentResult<BTreeMap<i32, i32>> {
    let mut sounds = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pNpcData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
        if npc_type <= 0 {
            continue;
        }
        let sound = required_i32(row, "m_iSound", &context)?;
        if let Some(previous) = sounds.insert(npc_type, sound)
            && previous != sound
        {
            return Err(invalid(format!(
                "contradictory m_iSound rows for gameplay NPC type {npc_type}: {previous} vs {sound}"
            )));
        }
    }
    Ok(sounds)
}

pub(super) fn extract_gameplay_npc_barker_types(
    rows: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, i32>> {
    let mut barker_types = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pNpcData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
        if npc_type <= 0 {
            continue;
        }
        let barker_type = optional_i32(row, "m_iBarkerType", &context)?.unwrap_or_default();
        if let Some(previous) = barker_types.insert(npc_type, barker_type)
            && previous != barker_type
        {
            return Err(invalid(format!(
                "contradictory m_iBarkerType rows for gameplay NPC type {npc_type}: {previous} vs {barker_type}"
            )));
        }
    }
    Ok(barker_types)
}

pub(super) fn extract_gameplay_npc_skill_barkers(
    npc_rows: &[Value],
    skill_strings: &[Value],
) -> TutorialMissionContentResult<(
    BTreeMap<(i32, i16), GameplayNpcSkillBarkerDefinition>,
    BTreeMap<i32, GameplayNpcSkillBarkerDefinition>,
)> {
    let mut skills = BTreeMap::new();
    let mut corruption = BTreeMap::new();
    if skill_strings.is_empty() {
        return Ok((skills, corruption));
    }
    for (row_index, row) in npc_rows.iter().enumerate() {
        let context = format!("m_pNpcData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
        if npc_type <= 0 {
            continue;
        }

        // This order is the exact else-if chain in FindNpcSkillString.
        for (skill_field, string_field) in [
            ("m_iMegaType", "m_iMegaString"),
            ("m_iActiveSkill1", "m_iActiveSkill1String"),
            ("m_iActiveSkill2", "m_iActiveSkill2String"),
            ("m_iSupportSkill", "m_iSupportSkillString"),
        ] {
            let raw_skill_id = optional_i32(row, skill_field, &context)?.unwrap_or_default();
            let skill_id = i16::try_from(raw_skill_id).map_err(|_| {
                invalid(format!(
                    "{context}.{skill_field} {raw_skill_id} does not fit protocol i16"
                ))
            })?;
            let string_id = optional_i32(row, string_field, &context)?.unwrap_or_default();
            let text = indexed_string_allow_empty(
                skill_strings,
                string_id,
                "m_strComment1",
                &format!("{context}.{string_field}"),
            )?;
            if !text.is_empty() && text != " " {
                skills
                    .entry((npc_type, skill_id))
                    .or_insert(GameplayNpcSkillBarkerDefinition { string_id, text });
            }
        }

        let string_id = optional_i32(row, "m_iCorruptionString", &context)?.unwrap_or_default();
        let text = indexed_string_allow_empty(
            skill_strings,
            string_id,
            "m_strComment1",
            &format!("{context}.m_iCorruptionString"),
        )?;
        if !text.is_empty() && text != " " {
            corruption.insert(
                npc_type,
                GameplayNpcSkillBarkerDefinition { string_id, text },
            );
        }
    }
    Ok((skills, corruption))
}

pub(super) fn extract_gameplay_npcs(
    rows: &[Value],
    strings: &[Value],
    barkers: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, GameplayNpcUiDefinition>> {
    let mut definitions = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pNpcData[{row_index}]");
        let row = value_object(row, &context)?;
        let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
        if npc_type <= 0 {
            continue;
        }
        let name_string_id = required_i32(row, "m_iNpcName", &context)?;
        let name = indexed_string(
            strings,
            name_string_id,
            "m_strName",
            &format!("{context}.m_iNpcName"),
        )?;
        let greeting = indexed_string_allow_empty(
            strings,
            name_string_id,
            "m_strComment",
            &format!("{context}.m_iNpcName"),
        )?;
        let barker_string_id = optional_i32(row, "m_iBarkerNumber", &context)?.unwrap_or_default();
        if barker_string_id < 0 {
            return Err(invalid(format!(
                "{context}.m_iBarkerNumber must be zero or positive, found {barker_string_id}"
            )));
        }
        let barker = (barker_string_id > 0)
            .then(|| {
                Ok(GameplayNpcBarkerDefinition {
                    string_id: barker_string_id,
                    lines: [
                        indexed_string_allow_empty(
                            barkers,
                            barker_string_id,
                            "m_strName",
                            &format!("{context}.m_iBarkerNumber"),
                        )?,
                        indexed_string_allow_empty(
                            barkers,
                            barker_string_id,
                            "m_strComment",
                            &format!("{context}.m_iBarkerNumber"),
                        )?,
                        indexed_string_allow_empty(
                            barkers,
                            barker_string_id,
                            "m_strComment1",
                            &format!("{context}.m_iBarkerNumber"),
                        )?,
                        indexed_string_allow_empty(
                            barkers,
                            barker_string_id,
                            "m_strComment2",
                            &format!("{context}.m_iBarkerNumber"),
                        )?,
                    ],
                })
            })
            .transpose()?;
        let max_hp = required_i32(row, "m_iHP", &context)?;
        if max_hp < 0 {
            return Err(invalid(format!(
                "{context}.m_iHP must be zero or positive, found {max_hp}"
            )));
        }
        let npc_level = required_i32(row, "m_iNpcLevel", &context)?;
        if npc_level < 0 {
            return Err(invalid(format!(
                "{context}.m_iNpcLevel must be zero or positive, found {npc_level}"
            )));
        }
        let npc_style = required_i32(row, "m_iNpcStyle", &context)?;
        if !(0..=2).contains(&npc_style) {
            return Err(invalid(format!(
                "{context}.m_iNpcStyle must be 0, 1, or 2, found {npc_style}"
            )));
        }
        let radius_server_units = required_i32(row, "m_iRadius", &context)?;
        let height_server_units = required_i32(row, "m_iHeight", &context)?;
        let sight_range_server_units = required_i32(row, "m_iSightRange", &context)?;
        for (field, value) in [
            ("m_iRadius", radius_server_units),
            ("m_iHeight", height_server_units),
            ("m_iSightRange", sight_range_server_units),
        ] {
            if value < 0 {
                return Err(invalid(format!(
                    "{context}.{field} must be zero or positive, found {value}"
                )));
            }
        }
        let npc_class = required_i32(row, "m_iNpcType", &context)?;
        let move_voice_owner = match optional_i32(row, "m_iComment", &context)? {
            Some(string_id) => indexed_string_allow_empty(
                strings,
                string_id,
                "m_strComment2",
                &format!("{context}.m_iComment"),
            )?
            .trim()
            .to_owned(),
            None => String::new(),
        };
        let mesh_id = optional_i32(row, "m_iMesh", &context)?;
        if mesh_id.is_some_and(|mesh_id| mesh_id < 0) {
            return Err(invalid(format!(
                "{context}.m_iMesh must be zero or positive, found {mesh_id:?}"
            )));
        }
        let table_scale = optional_xdt_f32(row, "m_fScale", &context)?;
        if table_scale.is_some_and(|scale| scale.value() <= 0.0) {
            return Err(invalid(format!(
                "{context}.m_fScale must be positive, found {:?}",
                table_scale.map(XdtF32::value)
            )));
        }
        let attack_range_server_units = optional_i32(row, "m_iAtkRange", &context)?;
        if attack_range_server_units.is_some_and(|range| range < 0) {
            return Err(invalid(format!(
                "{context}.m_iAtkRange must be zero or positive, found {attack_range_server_units:?}"
            )));
        }
        let definition = GameplayNpcUiDefinition {
            npc_type,
            name,
            greeting_string_id: name_string_id,
            greeting,
            barker,
            team: required_i32(row, "m_iTeam", &context)?,
            npc_level,
            npc_style,
            attack_effect: required_i32(row, "m_iEffect", &context)?,
            service_category: npc_class,
            service_number: optional_i32(row, "m_iServiceNumber", &context)?,
            npc_class,
            ai_type: required_i32(row, "m_iAiType", &context)?,
            mesh_id,
            table_scale,
            attack_range_server_units,
            move_voice_owner,
            radius_server_units,
            height_server_units,
            sight_range_server_units,
            max_hp,
        };
        if let Some(previous) = definitions.insert(npc_type, definition.clone())
            && previous != definition
        {
            return Err(invalid(format!(
                "contradictory duplicate gameplay NPC type {npc_type} in m_pNpcData"
            )));
        }
    }
    Ok(definitions)
}
