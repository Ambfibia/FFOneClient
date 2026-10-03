use super::*;

pub(super) fn extract_missions(
    rows: &[Value],
    journal_rows: &[Value],
    strings: &[Value],
    npcs: &BTreeMap<i32, GameplayNpcUiDefinition>,
) -> TutorialMissionContentResult<BTreeMap<i32, TutorialMissionDefinition>> {
    let required = TUTORIAL_MISSION_TASK_IDS
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut missions = BTreeMap::new();

    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pMissionData[{row_index}]");
        let row = value_object(row, &context)?;
        let task_id = required_i32(row, "m_iHTaskID", &context)?;
        if task_id == 0 {
            continue;
        }
        if task_id < 0 {
            return Err(invalid(format!(
                "{context}.m_iHTaskID must be zero or positive, found {task_id}"
            )));
        }
        let provenance = TutorialMissionRowProvenance {
            row_index,
            task_id,
            mission_id: required_i32(row, "m_iHMissionID", &context)?,
            title_string_id: required_i32(row, "m_iHMissionName", &context)?,
            objective_string_id: required_i32(row, "m_iHCurrentObjective", &context)?,
            mission_type: required_i32(row, "m_iHMissionType", &context)?,
            nano_id: required_i32(row, "m_iSTNanoID", &context)?,
            journal_row_id: required_i32(row, "m_iSTJournalIDAdd", &context)?,
            required_level: required_i32(row, "m_iCTRReqLvMin", &context)?,
            required_level_max: required_i32(row, "m_iCTRReqLvMax", &context)?,
            repeat_flag: required_i32(row, "m_iRepeatflag", &context)?,
            barker_text_ids: if row.contains_key("m_iHBarkerTextID") {
                required_i32_array4(row, "m_iHBarkerTextID", &context)?
            } else {
                [0; 4]
            },
            required_missions: required_i32_array2(row, "m_iCSTReqMission", &context)?,
            required_nanos: required_i32_array5(row, "m_iCSTRReqNano", &context)?,
            required_guide: required_i32(row, "m_iCSTReqGuide", &context)?,
            start_item_ids: required_i32_array3(row, "m_iCSTItemID", &context)?,
            start_item_counts: required_i32_array3(row, "m_iCSTItemNumNeeded", &context)?,
            start_message_npc_type: optional_i32(row, "m_iSTMessageSendNPC", &context)?
                .unwrap_or_default(),
            start_message_type: optional_i32(row, "m_iSTMessageType", &context)?
                .unwrap_or_default(),
            start_message_string_id: optional_i32(row, "m_iSTMessageTextID", &context)?
                .unwrap_or_default(),
            trigger_task_id: required_i32(row, "m_iCSTTrigger", &context)?,
            grant_timer: required_i32(row, "m_iSTGrantTimer", &context)?,
            completion_check_timer: required_i32(row, "m_iCSUCheckTimer", &context)?,
            completion_enemy_ids: required_i32_array3(row, "m_iCSUEnemyID", &context)?,
            completion_enemy_counts: required_i32_array3(row, "m_iCSUNumToKill", &context)?,
            completion_item_ids: required_i32_array3(row, "m_iCSUItemID", &context)?,
            completion_item_counts: required_i32_array3(row, "m_iCSUItemNumNeeded", &context)?,
            escort_def_npc_type: required_i32(row, "m_iCSUDEFNPCID", &context)?,
            required_instance_id: required_i32(row, "m_iRequireInstanceID", &context)?,
            difficulty_type: required_i32(row, "m_iHDifficultyType", &context)?,
            journal_npc_type: required_i32(row, "m_iHJournalNPCID", &context)?,
            start_npc_type: required_i32(row, "m_iHNPCID", &context)?,
            terminator_npc_type: required_i32(row, "m_iHTerminatorNPCID", &context)?,
            grant_waypoint_npc_type: required_i32(row, "m_iSTGrantWayPoint", &context)?,
            task_type: required_i32(row, "m_iHTaskType", &context)?,
            outgoing_task_id: required_i32(row, "m_iSUOutgoingTask", &context)?,
            success_message_npc_type: optional_i32(row, "m_iSUMessageSendNPC", &context)?
                .unwrap_or_default(),
            success_message_type: optional_i32(row, "m_iSUMessageType", &context)?
                .unwrap_or_default(),
            success_message_string_id: optional_i32(row, "m_iSUMessagetextID", &context)?
                .unwrap_or_default(),
            failure_outgoing_task_id: required_i32(row, "m_iFOutgoingTask", &context)?,
            failure_message_npc_type: optional_i32(row, "m_iFMessageSendNPC", &context)?
                .unwrap_or_default(),
            failure_message_type: optional_i32(row, "m_iFMessageType", &context)?
                .unwrap_or_default(),
            failure_message_string_id: optional_i32(row, "m_iFMessageTextID", &context)?
                .unwrap_or_default(),
            reward_id: required_i32(row, "m_iSUReward", &context)?,
        };
        let mission_type = TutorialMissionType::try_from(provenance.mission_type)
            .map_err(|error| invalid(format!("{context}.m_iHMissionType: {error}")))?;
        let title = indexed_string(
            strings,
            provenance.title_string_id,
            "m_pstrNameString",
            &format!("{context}.m_iHMissionName"),
        )?;
        let objective = indexed_string(
            strings,
            provenance.objective_string_id,
            "m_pstrNameString",
            &format!("{context}.m_iHCurrentObjective"),
        )?;
        let journal = extract_mission_journal_text(
            journal_rows,
            strings,
            provenance.journal_row_id,
            &format!("{context}.m_iSTJournalIDAdd"),
        )?;
        let start_nanocom_message = extract_mission_nanocom_message(
            strings,
            provenance.start_message_npc_type,
            provenance.start_message_type,
            provenance.start_message_string_id,
            &format!("{context}.m_iSTMessageTextID"),
        )?;
        let success_nanocom_message = extract_mission_nanocom_message(
            strings,
            provenance.success_message_npc_type,
            provenance.success_message_type,
            provenance.success_message_string_id,
            &format!("{context}.m_iSUMessagetextID"),
        )?;
        let failure_nanocom_message = extract_mission_nanocom_message(
            strings,
            provenance.failure_message_npc_type,
            provenance.failure_message_type,
            provenance.failure_message_string_id,
            &format!("{context}.m_iFMessageTextID"),
        )?;
        if !npcs.contains_key(&provenance.journal_npc_type) {
            return Err(invalid(format!(
                "{context} references journal NPC type {}, which is missing from m_pNpcData",
                provenance.journal_npc_type
            )));
        }
        let definition = TutorialMissionDefinition {
            title,
            objective,
            mission_type,
            journal,
            start_nanocom_message,
            success_nanocom_message,
            failure_nanocom_message,
            start_dialogue: extract_mission_dialogue(
                row,
                strings,
                "m_iSTDialogBubble",
                "m_iSTDialogBubbleNPCID",
                &context,
            )?,
            success_dialogue: extract_mission_dialogue(
                row,
                strings,
                "m_iSUDialogBubble",
                "m_iSUDialogBubbleNPCID",
                &context,
            )?,
            failure_dialogue: extract_mission_dialogue(
                row,
                strings,
                "m_iFDialogBubble",
                "m_iFDialogBubbleNPCID",
                &context,
            )?,
            provenance,
        };
        if missions.insert(task_id, definition).is_some() {
            return Err(invalid(format!(
                "duplicate mission task {task_id} in m_pMissionData"
            )));
        }
    }

    for task_id in required {
        if !missions.contains_key(&task_id) {
            return Err(invalid(format!(
                "missing tutorial mission task {task_id} in m_pMissionData"
            )));
        }
    }

    let mut title_by_mission = BTreeMap::<i32, (i32, &str)>::new();
    for mission in missions.values() {
        let key = mission.provenance.mission_id;
        let value = (mission.provenance.title_string_id, mission.title.as_str());
        if let Some(previous) = title_by_mission.insert(key, value)
            && previous != value
        {
            return Err(invalid(format!(
                "mission ID {key} has contradictory title sources: {previous:?} versus {value:?}"
            )));
        }
    }
    for mission in missions.values() {
        let outgoing = mission.provenance.outgoing_task_id;
        if outgoing == 0 {
            continue;
        }
        // The published primary-derived table contains a small number of
        // exact dangling outgoing IDs (for example 1197 -> 1200). Clean
        // `GetTask` returns null for those and simply cannot continue the
        // chain; retaining the source ID is more faithful than rejecting the
        // complete normal-world catalog at load time.
        let Some(next) = missions.get(&outgoing) else {
            continue;
        };
        if next.provenance.mission_id != mission.provenance.mission_id {
            return Err(invalid(format!(
                "task {} points across mission IDs to task {outgoing}",
                mission.provenance.task_id
            )));
        }
    }

    Ok(missions)
}

pub(super) fn extract_mission_journal_text(
    rows: &[Value],
    strings: &[Value],
    source_id: i32,
    context: &str,
) -> TutorialMissionContentResult<TutorialMissionJournalText> {
    let row_index = usize::try_from(source_id)
        .map_err(|_| invalid(format!("{context} has negative journal row ID {source_id}")))?;
    let row = rows.get(row_index).ok_or_else(|| {
        invalid(format!(
            "{context} references missing m_pJournalData row {source_id}"
        ))
    })?;
    let row_context = format!("{context} -> m_pJournalData[{source_id}]");
    let row = value_object(row, &row_context)?;
    let provenance = TutorialMissionJournalProvenance {
        row_index,
        offer_description_string_id: required_i32(row, "m_iDetaileMissionDesc", &row_context)?,
        active_task_description_string_id: required_i32(row, "m_iDetailedTaskDesc", &row_context)?,
        mission_summary_string_id: required_i32(row, "m_iMissionSummary", &row_context)?,
        mission_complete_summary_string_id: required_i32(
            row,
            "m_iMissionCompleteSummary",
            &row_context,
        )?,
        completion_description_string_id: required_i32(
            row,
            "m_iDetaileMissionCompleteSummary",
            &row_context,
        )?,
    };

    Ok(TutorialMissionJournalText {
        offer_description: indexed_string(
            strings,
            provenance.offer_description_string_id,
            "m_pstrNameString",
            &format!("{row_context}.m_iDetaileMissionDesc"),
        )?,
        active_task_description: indexed_string(
            strings,
            provenance.active_task_description_string_id,
            "m_pstrNameString",
            &format!("{row_context}.m_iDetailedTaskDesc"),
        )?,
        mission_summary: indexed_string(
            strings,
            provenance.mission_summary_string_id,
            "m_pstrNameString",
            &format!("{row_context}.m_iMissionSummary"),
        )?,
        mission_complete_summary: indexed_string(
            strings,
            provenance.mission_complete_summary_string_id,
            "m_pstrNameString",
            &format!("{row_context}.m_iMissionCompleteSummary"),
        )?,
        completion_description: indexed_string(
            strings,
            provenance.completion_description_string_id,
            "m_pstrNameString",
            &format!("{row_context}.m_iDetaileMissionCompleteSummary"),
        )?,
        provenance,
    })
}

pub(super) fn extract_rewards(
    mission_table: &Map<String, Value>,
    missions: &BTreeMap<i32, TutorialMissionDefinition>,
) -> TutorialMissionContentResult<BTreeMap<i32, TutorialRewardDefinition>> {
    let required = missions
        .values()
        .map(|mission| mission.provenance.reward_id)
        .filter(|reward_id| *reward_id != 0)
        .collect::<BTreeSet<_>>();
    if required.is_empty() {
        return Ok(BTreeMap::new());
    }

    let rows = required_array(mission_table, "m_pRewardData", "m_pMissionTable")?;
    let mut rewards = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pRewardData[{row_index}]");
        let row = value_object(row, &context)?;
        let reward_id = required_i32(row, "m_iMissionRewardID", &context)?;
        if !required.contains(&reward_id) {
            continue;
        }
        let primary_types = required_i32_array4(row, "m_iMissionRewarItemType", &context)?;
        let primary_ids = required_i32_array4(row, "m_iMissionRewardItemID", &context)?;
        let secondary_types = required_i32_array4(row, "m_iMissionRewardItemType2", &context)?;
        let secondary_ids = required_i32_array4(row, "m_iMissionRewardItemID2", &context)?;
        let definition = TutorialRewardDefinition {
            reward_id,
            cash: required_i32(row, "m_iCash", &context)?,
            fusion_matter: required_i32(row, "m_iFusionMatter", &context)?,
            box1_choice: required_i32(row, "m_iBox1Choice", &context)?,
            box2_choice: required_i32(row, "m_iBox2Choice", &context)?,
            primary_items: std::array::from_fn(|index| TutorialRewardItem {
                item_type: primary_types[index],
                item_id: primary_ids[index],
            }),
            secondary_items: std::array::from_fn(|index| TutorialRewardItem {
                item_type: secondary_types[index],
                item_id: secondary_ids[index],
            }),
        };
        if definition.cash < 0
            || definition.fusion_matter < 0
            || definition.box1_choice < 0
            || definition.box2_choice < 0
        {
            return Err(invalid(format!(
                "{context} contains a negative tutorial reward quantity"
            )));
        }
        if rewards.insert(reward_id, definition).is_some() {
            return Err(invalid(format!(
                "duplicate tutorial reward {reward_id} in m_pRewardData"
            )));
        }
    }
    for reward_id in required {
        if !rewards.contains_key(&reward_id) {
            return Err(invalid(format!(
                "missing tutorial reward {reward_id} in m_pRewardData"
            )));
        }
    }
    Ok(rewards)
}

pub(super) fn extract_scene_events(
    rows: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, TutorialSceneEventDefinition>> {
    let mut events = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_SceneData[{row_index}]");
        let row = value_object(row, &context)?;
        let event = required_i32(row, "m_iEvent", &context)?;
        if event < 0 {
            return Err(invalid(format!("{context}.m_iEvent is negative: {event}")));
        }
        let declared_element_count = required_i32(row, "m_iElementCnt", &context)?;
        if declared_element_count < 0 {
            return Err(invalid(format!(
                "{context}.m_iElementCnt is negative: {declared_element_count}"
            )));
        }
        let elements = required_array(row, "m_TextElement", &context)?;
        let declared_usize = usize::try_from(declared_element_count).map_err(|_| {
            invalid(format!(
                "{context}.m_iElementCnt cannot be represented as usize: {declared_element_count}"
            ))
        })?;
        if elements.len() != declared_usize {
            return Err(invalid(format!(
                "{context}.m_iElementCnt declares {declared_element_count} elements, but m_TextElement contains {}",
                elements.len()
            )));
        }

        let mut texts = BTreeMap::new();
        for (element_index, element) in elements.iter().enumerate() {
            let element_context = format!("{context}.m_TextElement[{element_index}]");
            let element = value_object(element, &element_context)?;
            let line = required_i32(element, "m_iLine", &element_context)?;
            if line < 0 {
                return Err(invalid(format!(
                    "{element_context}.m_iLine is negative: {line}"
                )));
            }
            let text = required_string(element, "m_strText", &element_context)?;
            if text.is_empty() {
                return Err(invalid(format!(
                    "{element_context}.m_strText must not be empty"
                )));
            }
            let definition = TutorialSceneTextDefinition {
                text: text.to_owned(),
                provenance: TutorialSceneTextProvenance {
                    event_row_index: row_index,
                    element_index,
                    event,
                    line,
                },
            };
            if texts.insert(line, definition).is_some() {
                return Err(invalid(format!(
                    "duplicate cut-scene event {event} line {line} in {context}"
                )));
            }
        }

        for expected_line in 1..=declared_element_count {
            if !texts.contains_key(&expected_line) {
                return Err(invalid(format!(
                    "{context} is not contiguous: missing line {expected_line} of {declared_element_count}"
                )));
            }
        }
        if texts.keys().next().copied().unwrap_or(1) != 1
            || texts.keys().next_back().copied().unwrap_or(0) != declared_element_count
        {
            return Err(invalid(format!(
                "{context} line range must be exactly 1..={declared_element_count}"
            )));
        }

        let definition = TutorialSceneEventDefinition {
            provenance: TutorialSceneEventProvenance {
                row_index,
                event,
                declared_element_count,
            },
            texts,
        };
        if events.insert(event, definition).is_some() {
            return Err(invalid(format!(
                "duplicate cut-scene event {event} in m_SceneData"
            )));
        }
    }
    Ok(events)
}

pub(super) fn extract_gameplay_user_equip_icons(
    root: &Map<String, Value>,
    installed_icon_paths: &BTreeSet<String>,
) -> TutorialMissionContentResult<BTreeMap<(u8, u8, i32, u8), GameplayUserEquipIconDefinition>> {
    let mut catalog = BTreeMap::new();

    for spec in OPTIONAL_USER_EQUIP_ITEM_TABLES {
        let Some(table) = root.get(spec.table_key) else {
            continue;
        };
        let table = value_object(table, spec.table_key)?;
        append_user_equip_icon_table(&mut catalog, table, spec, installed_icon_paths)?;
    }

    for spec in EXISTING_USER_EQUIP_ICON_SUBTABLES {
        let Some(table) = root.get(spec.table_key) else {
            continue;
        };
        let table = value_object(table, spec.table_key)?;
        append_user_equip_icon_table(&mut catalog, table, spec, installed_icon_paths)?;
    }

    Ok(catalog)
}

pub(super) fn extract_gameplay_npc_portrait_icon_paths(
    icons: &BTreeMap<(u8, u8, i32, u8), GameplayUserEquipIconDefinition>,
) -> TutorialMissionContentResult<BTreeMap<i32, String>> {
    let mut paths = BTreeMap::new();
    for icon in icons.values().filter(|icon| {
        // Exact Npc TableData route used by NpcMoveController.SetMobIcon:
        // table 10, item subtable 0, icon subtable 3.
        icon.item_table == 10 && icon.item_subtable == 0 && icon.icon_subtable == 3
    }) {
        if icon.declared_item_number <= 0 {
            continue;
        }
        let Some(path) = icon.icon_path.as_ref() else {
            continue;
        };
        if let Some(previous) = paths.insert(icon.declared_item_number, path.clone())
            && previous != *path
        {
            return Err(invalid(format!(
                "contradictory target portrait icons for gameplay NPC type {}: {previous:?} vs {path:?}",
                icon.declared_item_number
            )));
        }
    }
    Ok(paths)
}

pub(super) fn extract_gameplay_vendor_item_metadata(
    root: &Map<String, Value>,
    icons: &BTreeMap<(u8, u8, i32, u8), GameplayUserEquipIconDefinition>,
) -> TutorialMissionContentResult<BTreeMap<(i16, i16), GameplayVendorItemMetadataDefinition>> {
    let mut metadata = BTreeMap::new();
    for spec in VENDOR_ITEM_TABLES {
        let Some(table) = root.get(spec.table_key) else {
            continue;
        };
        let table = value_object(table, spec.table_key)?;
        // Compact authored fixtures intentionally carry only the icon route.
        // A real metadata table is recognized by its clean string subtable;
        // once present, every consumed field below is strict.
        let Some(strings) = table.get("m_pItemStringData") else {
            continue;
        };
        let strings = strings.as_array().ok_or_else(|| {
            invalid(format!(
                "{}.m_pItemStringData must be an array",
                spec.table_key
            ))
        })?;
        let rows = required_array(table, "m_pItemData", spec.table_key)?;
        for (row_index, row) in rows.iter().enumerate() {
            let context = format!("{}.m_pItemData[{row_index}]", spec.table_key);
            let row = value_object(row, &context)?;
            let item_id = i16::try_from(row_index).map_err(|_| {
                invalid(format!(
                    "{context} clean TableData row index does not fit protocol i16"
                ))
            })?;
            let string_index = required_i32(row, "m_iItemName", &context)?;
            let string_index = usize::try_from(string_index)
                .map_err(|_| invalid(format!("{context}.m_iItemName must be non-negative")))?;
            let string_context =
                format!("{context}.m_iItemName -> m_pItemStringData[{string_index}]");
            let string_row = strings.get(string_index).ok_or_else(|| {
                invalid(format!(
                    "{context}.m_iItemName references missing string row {string_index}"
                ))
            })?;
            let string_row = value_object(string_row, &string_context)?;
            let name = required_string(string_row, "m_strName", &string_context)?.to_owned();
            let description =
                required_string(string_row, "m_strComment", &string_context)?.to_owned();
            let level = if spec.requires_level {
                required_i32(row, "m_iMinReqLev", &context)?
            } else {
                0
            };
            let general_item_type = spec
                .is_general
                .then(|| required_i32(row, "m_iItemType", &context))
                .transpose()?;
            let battery_recharge = spec
                .is_general
                .then(|| required_i32(row, "m_iBatteryRecharge", &context))
                .transpose()?;
            let stack_size = if spec.is_general {
                optional_i32(row, "m_iStackNumber", &context)?
            } else {
                None
            };
            if stack_size.is_some_and(|stack_size| stack_size < 0) {
                return Err(invalid(format!(
                    "{context}.m_iStackNumber must be non-negative"
                )));
            }
            let icon_path = icons
                .get(&(spec.item_table, 0, i32::from(item_id), 2))
                .and_then(|icon| icon.icon_path.clone());
            let text_group = match spec.item_type {
                0 => "weapon_item",
                1 => "shirts_item",
                2 => "pants_item",
                3 => "shoes_item",
                4 => "hat_item",
                5 => "glass_item",
                6 => "back_item",
                7 => "general_item",
                9 => "chest_item",
                10 => "vehicle_item",
                _ => unreachable!("fixed item table contract"),
            };
            let definition = GameplayVendorItemMetadataDefinition {
                try_on_gender: optional_i32(row, "m_iReqSex", &context)?.unwrap_or(0),
                try_on_guide: optional_i32(row, "m_iMentor", &context)?.unwrap_or(0),
                text_key_prefix: format!(
                    "content.tabledata.{text_group}.item_string.{string_index}"
                ),
                name,
                description,
                level,
                buy_price: required_i32(row, "m_iItemPrice", &context)?,
                sell_price: required_i32(row, "m_iItemSellPrice", &context)?,
                sellable: required_i32(row, "m_iSellAble", &context)? != 0,
                general_item_type,
                battery_recharge,
                stack_size,
                icon_path,
                point_rating: if spec.requires_level {
                    required_i32(row, "m_iPointRat", &context)?
                } else {
                    0
                },
                group_rating: if spec.requires_level {
                    required_i32(row, "m_iGroupRat", &context)?
                } else {
                    0
                },
                defense_rating: if spec.requires_level {
                    required_i32(row, "m_iDefenseRat", &context)?
                } else {
                    0
                },
                equip_type: spec
                    .requires_level
                    .then(|| required_i32(row, "m_iEquipType", &context))
                    .transpose()?,
                target_mode: spec
                    .requires_level
                    .then(|| required_i32(row, "m_iTargetMode", &context))
                    .transpose()?,
                rarity: spec
                    .requires_level
                    .then(|| required_i32(row, "m_iRarity", &context))
                    .transpose()?,
                tradeable: required_i32(row, "m_iTradeAble", &context)? != 0,
            };
            if metadata
                .insert((spec.item_type, item_id), definition)
                .is_some()
            {
                return Err(invalid(format!(
                    "duplicate VendorMode metadata for item type {} row {item_id}",
                    spec.item_type
                )));
            }
        }
    }
    Ok(metadata)
}

pub(super) fn extract_gameplay_vehicle_speed_classes(
    root: &Map<String, Value>,
) -> TutorialMissionContentResult<BTreeMap<i16, i32>> {
    let Some(table) = root.get("m_pVehicleItemTable") else {
        return Ok(BTreeMap::new());
    };
    let table = value_object(table, "m_pVehicleItemTable")?;
    let rows = required_array(table, "m_pItemData", "m_pVehicleItemTable")?;

    // Compact source subsets may own only the vehicle icon route. Once the
    // clean Int32 speed field is present, require it for every table-26 row;
    // a partial projection must fail instead of inventing a fallback value.
    let has_speed_projection = rows.iter().any(|row| {
        row.as_object()
            .is_some_and(|row| row.contains_key("m_iUp_runSpeed"))
    });
    if !has_speed_projection {
        return Ok(BTreeMap::new());
    }

    let mut speed_classes = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pVehicleItemTable.m_pItemData[{row_index}]");
        let row = value_object(row, &context)?;
        let item_id = i16::try_from(row_index).map_err(|_| {
            invalid(format!(
                "{context} clean TableData row index does not fit protocol i16"
            ))
        })?;
        speed_classes.insert(item_id, required_i32(row, "m_iUp_runSpeed", &context)?);
    }
    Ok(speed_classes)
}

pub(super) fn extract_gameplay_vehicle_equip_types(
    root: &Map<String, Value>,
) -> TutorialMissionContentResult<BTreeMap<i16, i32>> {
    let Some(table) = root.get("m_pVehicleItemTable") else {
        return Ok(BTreeMap::new());
    };
    let table = value_object(table, "m_pVehicleItemTable")?;
    let rows = required_array(table, "m_pItemData", "m_pVehicleItemTable")?;
    let has_projection = rows.iter().any(|row| {
        row.as_object()
            .is_some_and(|row| row.contains_key("m_iEquipType"))
    });
    if !has_projection {
        return Ok(BTreeMap::new());
    }

    let mut equip_types = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pVehicleItemTable.m_pItemData[{row_index}]");
        let row = value_object(row, &context)?;
        let item_id = i16::try_from(row_index).map_err(|_| {
            invalid(format!(
                "{context} clean TableData row index does not fit protocol i16"
            ))
        })?;
        equip_types.insert(item_id, required_i32(row, "m_iEquipType", &context)?);
    }
    Ok(equip_types)
}
