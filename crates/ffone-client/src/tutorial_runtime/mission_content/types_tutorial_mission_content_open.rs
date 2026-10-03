use super::*;

impl TutorialMissionContent {
    pub fn open(locator: &AssetLocator) -> TutorialMissionContentResult<Self> {
        Self::from_source(locator)
    }

    pub fn from_project_assets(assets: &AssetLocator) -> TutorialMissionContentResult<Self> {
        Self::open(assets)
    }

    pub(super) fn from_source(assets: &impl TutorialAssetSource) -> TutorialMissionContentResult<Self> {
        let (route, bytes) = assets.table()?;
        let document: Value = crate::xdt::from_slice(&bytes).map_err(|error| {
            invalid(format!(
                "invalid tutorial table-set JSON {:?}: {error}",
                route.path
            ))
        })?;

        let root = value_object(&document, "table-set root")?;
        let schema = required_string(root, "schema", "table-set root")?;
        if schema != TABLE_SET_SCHEMA {
            return Err(invalid(format!(
                "unsupported tutorial table-set schema {schema:?}; expected {TABLE_SET_SCHEMA:?}"
            )));
        }
        let tables = required_array(root, "tables", "table-set root")?;
        let matches = tables
            .iter()
            .filter(|table| table.get("name").and_then(Value::as_str) == Some(CONSOLIDATED_TABLE))
            .collect::<Vec<_>>();
        let [table] = matches.as_slice() else {
            return Err(invalid(format!(
                "tutorial content requires exactly one table document named {CONSOLIDATED_TABLE:?}, found {}",
                matches.len()
            )));
        };
        let table = value_object(table, "table-set tables[0]")?;
        let table_key = required_string(table, "key", "table-set tables[0]")?;
        let table_name = required_string(table, "name", "table-set tables[0]")?;
        if table_name != CONSOLIDATED_TABLE {
            return Err(invalid(format!(
                "tutorial table document is {table_name:?}; expected {CONSOLIDATED_TABLE:?}"
            )));
        }
        let value = required_object(table, "value", "table-set tables[0]")?;

        let message_table = required_object(value, "m_pMessageTable", CONSOLIDATED_TABLE)?;
        let message_rows = required_array(message_table, "m_pMessageData", "m_pMessageTable")?;
        let system_messages = extract_system_messages(message_rows)?;

        let npc_table = required_object(value, "m_pNpcTable", CONSOLIDATED_TABLE)?;
        let npc_rows = required_array(npc_table, "m_pNpcData", "m_pNpcTable")?;
        let npc_strings = required_array(npc_table, "m_pNpcStringData", "m_pNpcTable")?;
        let npc_barkers = npc_table
            .get("m_pNpcBarkerData")
            .map(|rows| {
                rows.as_array()
                    .ok_or_else(|| invalid("m_pNpcTable.m_pNpcBarkerData must be an array"))
            })
            .transpose()?
            .map(Vec::as_slice)
            .unwrap_or_default();
        let gameplay_npcs = extract_gameplay_npcs(npc_rows, npc_strings, npc_barkers)?;
        let gameplay_npc_barker_types = extract_gameplay_npc_barker_types(npc_rows)?;
        let gameplay_npc_map_icons = extract_gameplay_npc_map_icons(npc_rows)?;
        let gameplay_npc_sounds = extract_gameplay_npc_sounds(npc_rows)?;
        let gameplay_npc_services = extract_gameplay_npc_services(npc_table, npc_rows)?;
        let npcs = extract_npcs(npc_rows, npc_strings)?;
        let gameplay_vendors = match value.get("m_pVendorTable") {
            Some(table) => {
                let table = value_object(table, "m_pVendorTable")?;
                let rows = required_array(table, "m_pItemData", "m_pVendorTable")?;
                extract_gameplay_vendors(rows)?
            }
            None => BTreeMap::new(),
        };
        // Compact fixtures and authored subsets may intentionally omit the
        // XCom table. When present, every row is strict; when absent, nearest
        // resurrection remains explicitly unresolved.
        let gameplay_xcoms = match value.get("m_pXComTable") {
            Some(table) => {
                let table = value_object(table, "m_pXComTable")?;
                let rows = required_array(table, "m_pXComData", "m_pXComTable")?;
                extract_gameplay_xcoms(rows)?
            }
            None => Vec::new(),
        };
        let gameplay_general_items = match value.get("m_pGeneralItemTable") {
            Some(table) => {
                let table = value_object(table, "m_pGeneralItemTable")?;
                let rows = required_array(table, "m_pItemData", "m_pGeneralItemTable")?;
                let icons = table
                    .get("m_pItemIconData")
                    .map(|icons| {
                        icons.as_array().ok_or_else(|| {
                            invalid("m_pGeneralItemTable.m_pItemIconData must be an array")
                        })
                    })
                    .transpose()?;
                extract_gameplay_general_items(rows, icons)?
            }
            None => BTreeMap::new(),
        };
        let quest_item_names = match value.get("m_pQuestItemTable") {
            Some(table) => {
                let table = value_object(table, "m_pQuestItemTable")?;
                extract_quest_item_names(
                    required_array(table, "m_pItemData", "m_pQuestItemTable")?,
                    required_array(table, "m_pItemStringData", "m_pQuestItemTable")?,
                )?
            }
            None => BTreeMap::new(),
        };
        let mut reward_quest_items = BTreeMap::new();
        if let Some(rows) = value
            .get("m_pMissionTable")
            .and_then(|t| t.get("m_pMissionData"))
            .and_then(Value::as_array)
        {
            for row in rows {
                let row = value_object(row, "mission reward items")?;
                if row.contains_key("m_iSTItemID") {
                    reward_quest_items.insert(
                        required_i32(row, "m_iHTaskID", "mission reward items")?,
                        required_i32_array3(row, "m_iSTItemID", "mission reward items")?,
                    );
                }
            }
        }
        let mut reward_chest_kinds = BTreeMap::new();
        if let Some(rows) = value
            .get("m_pChestItemTable")
            .and_then(|t| t.get("m_pItemData"))
            .and_then(Value::as_array)
        {
            for (index, row) in rows.iter().enumerate() {
                let row = value_object(row, "reward chest")?;
                if let Some(kind) = optional_i32(row, "m_iChestCheck", "reward chest")? {
                    let id = i16::try_from(index)
                        .map_err(|_| invalid("reward chest index exceeds protocol item ID"))?;
                    reward_chest_kinds.insert(id, kind);
                }
            }
        }

        let mission_table = required_object(value, "m_pMissionTable", CONSOLIDATED_TABLE)?;
        let mission_rows = required_array(mission_table, "m_pMissionData", "m_pMissionTable")?;
        let journal_rows = required_array(mission_table, "m_pJournalData", "m_pMissionTable")?;
        let mission_strings =
            required_array(mission_table, "m_pMissionStringData", "m_pMissionTable")?;
        let gameplay_mission_name_strings = mission_strings
            .iter()
            .enumerate()
            .map(|(row_index, row)| {
                let context = format!("m_pMissionTable.m_pMissionStringData[{row_index}]");
                let row = value_object(row, &context)?;
                Ok(required_string(row, "m_pstrNameString", &context)?.to_owned())
            })
            .collect::<TutorialMissionContentResult<Vec<_>>>()?;
        let missions =
            extract_missions(mission_rows, journal_rows, mission_strings, &gameplay_npcs)?;
        // The minimap and in-world indicators ask about each resident NPC every
        // frame. Index the immutable table once; mutable eligibility remains in
        // WorldMissionRuntime and is still evaluated against current state.
        let mut mission_tasks_by_npc = BTreeMap::<i32, Vec<i32>>::new();
        for (&task_id, definition) in &missions {
            let row = &definition.provenance;
            mission_tasks_by_npc
                .entry(row.start_npc_type)
                .or_default()
                .push(task_id);
            if row.terminator_npc_type != row.start_npc_type {
                mission_tasks_by_npc
                    .entry(row.terminator_npc_type)
                    .or_default()
                    .push(task_id);
            }
        }
        let gameplay_guide_nanocom = value
            .get("m_pGuideTable")
            .map(|table| extract_gameplay_guide_nanocom(value_object(table, "m_pGuideTable")?))
            .transpose()?
            .unwrap_or_default();
        let rewards = extract_rewards(mission_table, &missions)?;

        let cut_scene_table = required_object(value, "m_pCutSceneTable", CONSOLIDATED_TABLE)?;
        let scene_rows = required_array(cut_scene_table, "m_SceneData", "m_pCutSceneTable")?;
        let scene_events = extract_scene_events(scene_rows)?;

        let instance_table = required_object(value, "m_pInstanceTable", CONSOLIDATED_TABLE)?;
        let warp_rows = required_array(instance_table, "m_pWarpData", "m_pInstanceTable")?;
        let gameplay_warps = extract_gameplay_warps(warp_rows)?;
        let warps = extract_warps(warp_rows, &npcs, &missions)?;

        let nano_table = required_object(value, "m_pNanoTable", CONSOLIDATED_TABLE)?;
        let nano_rows = required_array(nano_table, "m_pNanoData", "m_pNanoTable")?;
        let nano_icons = required_array(nano_table, "m_pNanoIconData", "m_pNanoTable")?;
        let nano_strings = required_array(nano_table, "m_pNanoStringData", "m_pNanoTable")?;
        let nano_tunes = required_array(nano_table, "m_pNanoTuneData", "m_pNanoTable")?;
        let nano_tune_strings =
            required_array(nano_table, "m_pNanoTuneStringData", "m_pNanoTable")?;
        let installed_icon_paths = assets.installed_icon_paths()?;
        let gameplay_nanos =
            extract_gameplay_nanos(nano_rows, nano_icons, nano_strings, &installed_icon_paths)?;
        let gameplay_nano_tune_fusion_matter = extract_gameplay_nano_tune_fusion_matter(value)?;

        let skill_table = required_object(value, "m_pSkillTable", CONSOLIDATED_TABLE)?;
        let skill_rows = required_array(skill_table, "m_pSkillData", "m_pSkillTable")?;
        let skill_icons = required_array(skill_table, "m_pSkillIconData", "m_pSkillTable")?;
        let skill_strings = skill_table
            .get("m_pSkillStringData")
            .map(|rows| {
                rows.as_array()
                    .ok_or_else(|| invalid("m_pSkillTable.m_pSkillStringData must be an array"))
            })
            .transpose()?
            .map(Vec::as_slice)
            .unwrap_or_default();
        let (gameplay_npc_skill_barkers, gameplay_npc_corruption_barkers) =
            extract_gameplay_npc_skill_barkers(npc_rows, skill_strings)?;
        let gameplay_skills = extract_gameplay_skills(skill_rows, skill_icons)?;
        let gameplay_skill_names = skill_strings
            .iter()
            .enumerate()
            .filter_map(|(id, row)| {
                Some((
                    i16::try_from(id).ok()?,
                    row["m_strName"].as_str()?.to_owned(),
                ))
            })
            .collect();
        let skill_buff_rows = required_array(skill_table, "m_pSkillBuffData", "m_pSkillTable")?;
        let gameplay_skill_buffs = extract_gameplay_skill_buffs(skill_buff_rows, skill_icons)?;
        let gameplay_user_equip_icons =
            extract_gameplay_user_equip_icons(value, &installed_icon_paths)?;
        let gameplay_npc_portrait_icon_paths =
            extract_gameplay_npc_portrait_icon_paths(&gameplay_user_equip_icons)?;
        let gameplay_vendor_item_metadata =
            extract_gameplay_vendor_item_metadata(value, &gameplay_user_equip_icons)?;
        let gameplay_vehicle_speed_classes = extract_gameplay_vehicle_speed_classes(value)?;
        let gameplay_vehicle_equip_types = extract_gameplay_vehicle_equip_types(value)?;
        let journal_nanos = extract_journal_nanos(
            &missions,
            nano_rows,
            nano_icons,
            nano_strings,
            nano_tunes,
            nano_tune_strings,
            skill_rows,
            skill_icons,
        )?;

        Ok(Self {
            provenance: TutorialTableSetProvenance {
                asset_path: route.path.clone(),
                source_path: route.source_path.clone(),
                bytes: route.bytes,
                blake3: route.blake3.clone(),
                schema: schema.to_owned(),
                table_key: table_key.to_owned(),
                table_name: table_name.to_owned(),
            },
            missions,
            mission_tasks_by_npc,
            gameplay_guide_nanocom,
            npcs,
            rewards,
            scene_events,
            warps,
            gameplay_warps,
            gameplay_npcs,
            gameplay_npc_barker_types,
            gameplay_mission_name_strings,
            gameplay_npc_skill_barkers,
            gameplay_npc_corruption_barkers,
            gameplay_npc_portrait_icon_paths,
            gameplay_npc_map_icons,
            gameplay_npc_sounds,
            gameplay_npc_services,
            gameplay_vendors,
            gameplay_vendor_item_metadata,
            gameplay_vehicle_speed_classes,
            gameplay_vehicle_engine_sounds: extract_vehicle_engine_sounds(value),
            gameplay_vehicle_equip_types,
            gameplay_xcoms,
            gameplay_general_items,
            quest_item_names,
            reward_quest_items,
            reward_chest_kinds,
            gameplay_nanos,
            gameplay_nano_tune_fusion_matter,
            gameplay_skills,
            gameplay_skill_names,
            gameplay_skill_buffs,
            gameplay_user_equip_icons,
            journal_nanos,
            system_messages,
        })
    }

    pub fn mission(
        &self,
        task_id: i32,
    ) -> TutorialMissionContentResult<&TutorialMissionDefinition> {
        self.missions
            .get(&task_id)
            .ok_or(TutorialMissionContentError::UnknownMission(task_id))
    }

    /// Exact login/level-up GuideTable copy for raw mentor IDs 1..=5.
    #[must_use]
    pub fn gameplay_guide_nanocom(
        &self,
        raw_mentor: i16,
    ) -> Option<&GameplayGuideNanocomDefinition> {
        self.gameplay_guide_nanocom.get(&raw_mentor)
    }

    pub fn mission_entry(
        &self,
        task_id: i32,
        npc_id: i32,
        npc_position: impl Into<String>,
    ) -> TutorialMissionContentResult<MissionUiEntry> {
        if npc_id <= 0 {
            return Err(invalid(format!(
                "live NPC ID must be positive, got {npc_id}"
            )));
        }
        self.mission_ui_entry(task_id, Some(npc_id), npc_position.into())
    }

    /// Builds the clean source-owned Reward projection used by
    /// `KillMissionNPC`/`CheckMissionItemCount` when there is no terminator.
    /// Runtime NPC ID zero is intentional and is serialized unchanged in the
    /// eventual TASK_END request.
    pub fn automatic_reward_entry(
        &self,
        task_id: i32,
        npc_id: i32,
    ) -> TutorialMissionContentResult<MissionUiEntry> {
        if npc_id < 0 {
            return Err(invalid(format!(
                "automatic reward runtime NPC ID must be zero or positive, got {npc_id}"
            )));
        }
        self.mission_ui_entry(task_id, Some(npc_id), String::new())
    }

    /// Builds a journal-only mission projection from immutable task/NPC
    /// provenance. `None` deliberately distinguishes it from a live NPC
    /// interaction and prevents Journal rows from inventing runtime IDs.
    pub fn journal_entry(
        &self,
        task_id: i32,
        npc_position: impl Into<String>,
    ) -> TutorialMissionContentResult<MissionUiEntry> {
        self.mission_ui_entry(task_id, None, npc_position.into())
    }

    /// Projects active tasks in their source table order while suppressing
    /// duplicate runtime IDs. Any unknown task fails the whole journal closed.
    pub fn active_journal_entries(
        &self,
        active_task_ids: &[i32],
        npc_position: impl Into<String>,
    ) -> TutorialMissionContentResult<Vec<MissionUiEntry>> {
        let active = active_task_ids.iter().copied().collect::<BTreeSet<_>>();
        for task_id in &active {
            self.mission(*task_id)?;
        }
        let npc_position = npc_position.into();
        let mut definitions = self
            .missions
            .values()
            .filter(|mission| active.contains(&mission.provenance.task_id))
            .collect::<Vec<_>>();
        definitions.sort_by_key(|mission| mission.provenance.row_index);
        definitions
            .into_iter()
            .map(|mission| {
                self.mission_ui_entry(mission.provenance.task_id, None, npc_position.clone())
            })
            .collect()
    }

    /// Projects the legacy completed-history list. `cnMissionManager` adds only
    /// terminal tasks (`m_iSUOutgoingTask == 0`) to that list; intermediate
    /// tutorial steps must never appear merely because the native runtime
    /// records them in its broader completion history.
    pub fn completed_journal_entries(
        &self,
        completed_task_ids: &[i32],
        npc_position: impl Into<String>,
    ) -> TutorialMissionContentResult<Vec<MissionUiEntry>> {
        let npc_position = npc_position.into();
        let mut mission_ids = BTreeSet::new();
        let mut entries = Vec::new();
        for &task_id in completed_task_ids {
            let mission = self.mission(task_id)?;
            if mission.provenance.outgoing_task_id != 0
                || !mission_ids.insert(mission.provenance.mission_id)
            {
                continue;
            }
            entries.push(self.mission_ui_entry(task_id, None, npc_position.clone())?);
        }
        Ok(entries)
    }

    pub fn outgoing_task_id(&self, task_id: i32) -> TutorialMissionContentResult<Option<i32>> {
        let outgoing = self.mission(task_id)?.provenance.outgoing_task_id;
        Ok((outgoing != 0).then_some(outgoing))
    }

    pub fn failure_outgoing_task_id(
        &self,
        task_id: i32,
    ) -> TutorialMissionContentResult<Option<i32>> {
        let outgoing = self.mission(task_id)?.provenance.failure_outgoing_task_id;
        Ok((outgoing != 0).then_some(outgoing))
    }

    /// Mirrors the clean mission tree's `parents.GetChildByIndex(0)`: mission
    /// children preserve their serialized TableData row order.
    pub fn is_first_serialized_task(&self, task_id: i32) -> TutorialMissionContentResult<bool> {
        let task = self.mission(task_id)?;
        let first_task = self
            .missions
            .values()
            .filter(|candidate| candidate.provenance.mission_id == task.provenance.mission_id)
            .min_by_key(|candidate| candidate.provenance.row_index)
            .ok_or(TutorialMissionContentError::UnknownMission(
                task.provenance.mission_id,
            ))?;
        Ok(first_task.provenance.task_id == task_id)
    }

    /// Mirrors clean `cnMissionNode.IsFinalTask`: the final task is the last
    /// serialized nonzero task row belonging to the mission, not necessarily
    /// the row whose success outgoing ID is zero.
    pub fn is_final_serialized_task(&self, task_id: i32) -> TutorialMissionContentResult<bool> {
        let task = self.mission(task_id)?;
        let final_task = self
            .missions
            .values()
            .filter(|candidate| candidate.provenance.mission_id == task.provenance.mission_id)
            .max_by_key(|candidate| candidate.provenance.row_index)
            .ok_or(TutorialMissionContentError::UnknownMission(
                task.provenance.mission_id,
            ))?;
        Ok(final_task.provenance.task_id == task_id)
    }

    /// Mirrors `cnMissionNode.GetFinalTask`: mission rewards displayed for any
    /// step come from the terminal task in that task's outgoing chain.
    pub fn final_task_id(&self, task_id: i32) -> TutorialMissionContentResult<i32> {
        Ok(self.final_mission(task_id)?.provenance.task_id)
    }

    /// Resolves the terminal task for a mission ID without inventing a task
    /// order. Exactly one loaded terminal row must own the mission.
    pub fn terminal_task_for_mission_id(
        &self,
        mission_id: i32,
    ) -> TutorialMissionContentResult<&TutorialMissionDefinition> {
        let mut terminals = self.missions.values().filter(|mission| {
            mission.provenance.mission_id == mission_id && mission.provenance.outgoing_task_id == 0
        });
        let terminal = terminals
            .next()
            .ok_or(TutorialMissionContentError::UnknownMission(mission_id))?;
        if terminals.next().is_some() {
            return Err(invalid(format!(
                "mission ID {mission_id} has more than one terminal task"
            )));
        }
        Ok(terminal)
    }

    /// Mirrors `cnMissionNode.GetFinalTask` for PC-load completion flags: the
    /// last serialized task row in the mission, even when its outgoing field
    /// is not the terminal-chain sentinel.
    pub fn final_serialized_task_for_mission_id(
        &self,
        mission_id: i32,
    ) -> TutorialMissionContentResult<&TutorialMissionDefinition> {
        self.missions
            .values()
            .filter(|candidate| candidate.provenance.mission_id == mission_id)
            .max_by_key(|candidate| candidate.provenance.row_index)
            .ok_or(TutorialMissionContentError::UnknownMission(mission_id))
    }

    pub fn mission_row_index(&self, task_id: i32) -> TutorialMissionContentResult<usize> {
        Ok(self.mission(task_id)?.provenance.row_index)
    }

    pub(super) fn mission_ui_entry(
        &self,
        task_id: i32,
        npc_id: Option<i32>,
        npc_position: String,
    ) -> TutorialMissionContentResult<MissionUiEntry> {
        let mission = self.mission(task_id)?;
        let npc_name = self
            .npcs
            .get(&mission.provenance.journal_npc_type)
            .map(|npc| npc.name.as_str())
            .or_else(|| {
                self.gameplay_npcs
                    .get(&mission.provenance.journal_npc_type)
                    .map(|npc| npc.name.as_str())
            })
            .ok_or(TutorialMissionContentError::UnknownNpc(
                mission.provenance.journal_npc_type,
            ))?;
        let rewards = self.reward_ui(mission)?;
        let nano = if mission.mission_type == TutorialMissionType::Nano {
            Some(
                self.journal_nanos
                    .get(&mission.provenance.nano_id)
                    .ok_or_else(|| {
                        invalid(format!(
                            "tutorial mission task {task_id} references unloaded Nano {}",
                            mission.provenance.nano_id
                        ))
                    })?
                    .clone(),
            )
        } else {
            None
        };
        Ok(MissionUiEntry {
            task_id,
            npc_id,
            mission_type: mission.mission_type.xdt_value(),
            task_type: mission.provenance.task_type,
            outgoing_task_id: mission.provenance.outgoing_task_id,
            has_task_reward: mission.provenance.reward_id > 0,
            is_first_mission_task: self.is_first_serialized_task(task_id)?,
            journal_npc_type: mission.provenance.journal_npc_type,
            required_level: mission.provenance.required_level,
            difficulty_type: mission.provenance.difficulty_type,
            nano,
            title: mission.title.clone(),
            npc_name: npc_name.to_owned(),
            npc_position,
            objective: mission.objective.clone(),
            offer_description: mission.offer_description().to_owned(),
            task_description: mission.journal.active_task_description.clone(),
            active_description: mission.active_description(),
            mission_summary: mission.journal.mission_summary.clone(),
            mission_complete_summary: mission.journal.mission_complete_summary.clone(),
            completion_description: mission.completion_description().to_owned(),
            rewards,
        })
    }

    pub fn npc(&self, npc_type: i32) -> TutorialMissionContentResult<&TutorialNpcDefinition> {
        self.npcs
            .get(&npc_type)
            .ok_or(TutorialMissionContentError::UnknownNpc(npc_type))
    }

    pub fn npc_name(&self, npc_type: i32) -> TutorialMissionContentResult<&str> {
        Ok(&self.npc(npc_type)?.name)
    }

    pub fn reward(
        &self,
        reward_id: i32,
    ) -> TutorialMissionContentResult<&TutorialRewardDefinition> {
        self.rewards
            .get(&reward_id)
            .ok_or(TutorialMissionContentError::UnknownReward(reward_id))
    }

    pub fn scene_event(
        &self,
        event: i32,
    ) -> TutorialMissionContentResult<&TutorialSceneEventDefinition> {
        self.scene_events
            .get(&event)
            .ok_or(TutorialMissionContentError::UnknownSceneEvent(event))
    }

    pub fn scene_text_definition(
        &self,
        event: i32,
        line: i32,
    ) -> TutorialMissionContentResult<&TutorialSceneTextDefinition> {
        self.scene_event(event)?.text(line)
    }

    pub fn scene_text(&self, event: i32, line: i32) -> TutorialMissionContentResult<&str> {
        Ok(&self.scene_text_definition(event, line)?.text)
    }

    /// Resolves the exact clean SystemMessage row. Missing compact/source rows
    /// remain unresolved so callers cannot substitute invented modal text.
    #[must_use]
    pub fn system_message_definition(&self, row_id: i32) -> Option<&SystemMessageDefinition> {
        self.system_messages.get(&row_id)
    }

    pub fn warp(&self, npc_type: i32) -> TutorialMissionContentResult<&TutorialWarpDefinition> {
        self.warps
            .get(&npc_type)
            .ok_or(TutorialMissionContentError::UnknownWarp(npc_type))
    }

    /// Looks up an exact normal-world warp row by `m_iWarpNumber`.
    #[must_use]
    pub fn gameplay_warp(&self, warp_id: i32) -> Option<&GameplayWarpDefinition> {
        self.gameplay_warps.get(&warp_id)
    }

    /// Mirrors clean `NpcIconMode.WarpOK`: walk `m_pWarpData` in serialized
    /// order and stop at the first row whose `m_iNpcNumber` matches.
    #[must_use]
    pub fn first_gameplay_warp_for_npc(&self, npc_type: i32) -> Option<&GameplayWarpDefinition> {
        self.gameplay_warps
            .values()
            .filter(|warp| warp.npc_type == npc_type)
            .min_by_key(|warp| warp.row_index)
    }

    /// Returns the row behind the clean normal NPC "WARP" action.
    /// `NpcIconMode.InitMode` exposes that action only for exact service
    /// category 5. Categories 15/16 are transport actions and category 101
    /// adds no warp action, even though all occur in the same outer branch.
    #[must_use]
    pub fn normal_gameplay_warp_for_npc(&self, npc_type: i32) -> Option<&GameplayWarpDefinition> {
        (self.gameplay_npc(npc_type)?.service_category == 5)
            .then(|| self.first_gameplay_warp_for_npc(npc_type))?
    }

    /// Iterates every lossless normal-world warp projection. Callers that
    /// need the visible clean NPC action must use
    /// [`Self::normal_gameplay_warp_for_npc`].
    pub fn gameplay_warps(&self) -> impl ExactSizeIterator<Item = &GameplayWarpDefinition> {
        self.gameplay_warps.values()
    }

    #[must_use]
    pub fn gameplay_npc(&self, npc_type: i32) -> Option<&GameplayNpcUiDefinition> {
        self.gameplay_npcs.get(&npc_type)
    }

    /// Iterates the exact positive `m_pNpcData.m_iNpcNumber` XDT projection in
    /// numeric ID order. The backing `BTreeMap` is intentionally preserved so
    /// tooling and gameplay never invent a second catalog order.
    pub fn gameplay_npcs(&self) -> impl ExactSizeIterator<Item = &GameplayNpcUiDefinition> {
        self.gameplay_npcs.values()
    }

    /// Exact `NpcTableElement.m_iBarkerType` used by the completed-mission
    /// ambient request scan. Only values 1..=4 are renderable in clean code.
    #[must_use]
    pub fn gameplay_npc_barker_type(&self, npc_type: i32) -> Option<i32> {
        self.gameplay_npc_barker_types.get(&npc_type).copied()
    }
}
