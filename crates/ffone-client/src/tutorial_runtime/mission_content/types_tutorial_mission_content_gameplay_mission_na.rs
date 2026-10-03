use super::*;

impl TutorialMissionContent {


    /// Exact row used by the clean `P_FE2CL_REP_BARKER` handler.
    #[must_use]
    pub fn gameplay_mission_name_string(&self, string_id: i32) -> Option<&str> {
        usize::try_from(string_id)
            .ok()
            .and_then(|index| self.gameplay_mission_name_strings.get(index))
            .map(String::as_str)
    }

    /// Mirrors `NpcMoveController.FindNpcSkillString` for an ordinary skill
    /// READY packet, including its Mega/Active1/Active2/Support priority.
    #[must_use]
    pub fn gameplay_npc_skill_barker(
        &self,
        npc_type: i32,
        skill_id: i16,
    ) -> Option<&GameplayNpcSkillBarkerDefinition> {
        self.gameplay_npc_skill_barkers.get(&(npc_type, skill_id))
    }

    /// Exact `m_iCorruptionString -> SkillStringTable.m_strComment1` row.
    #[must_use]
    pub fn gameplay_npc_corruption_barker(
        &self,
        npc_type: i32,
    ) -> Option<&GameplayNpcSkillBarkerDefinition> {
        self.gameplay_npc_corruption_barkers.get(&npc_type)
    }

    /// Resolves `NpcTableElement.m_iIcon1 -> m_pNpcIconData ->
    /// AvatarUtil.GetIconName` for the target HUD portrait. The route remains
    /// absent when TableData has no icon row, the icon type is unsupported,
    /// or the exact native semantic asset is not installed.
    #[must_use]
    pub fn gameplay_npc_portrait_icon_path(&self, npc_type: i32) -> Option<&str> {
        self.gameplay_npc_portrait_icon_paths
            .get(&npc_type)
            .map(String::as_str)
    }

    #[must_use]
    pub fn quest_item_name(&self, item_id: i32) -> Option<&str> {
        self.quest_item_names.get(&item_id).map(String::as_str)
    }

    pub fn reward_quest_item(&self, task_id: i32, npc_type: i32) -> Option<i32> {
        let row = &self.mission(task_id).ok()?.provenance;
        let index = row
            .completion_enemy_ids
            .iter()
            .enumerate()
            .find_map(|(i, id)| {
                (*id == npc_type && row.completion_enemy_counts[i] == 0).then_some(i)
            })?;
        self.reward_quest_items
            .get(&task_id)
            .map(|items| items[index])
            .filter(|id| *id > 0)
    }

    pub fn reward_is_crate(&self, item_id: i16) -> bool {
        self.reward_chest_kinds.get(&item_id) == Some(&0)
    }

    /// Clean `ChestItemElement.m_iChestCheck`: `0` is a C.R.A.T.E, `1` an E.G.G.
    #[must_use]
    pub fn reward_chest_kind(&self, item_id: i16) -> Option<i32> {
        self.reward_chest_kinds.get(&item_id).copied()
    }

    /// Returns the exact clean `NpcTableElement.m_iMapIcon` when that field is
    /// present in the loaded TableData row. Compact/authored table subsets may
    /// omit it; callers must treat that as an unresolved catalog row.
    #[must_use]
    pub fn gameplay_npc_map_icon(&self, npc_type: i32) -> Option<i32> {
        self.gameplay_npc_map_icons.get(&npc_type).copied()
    }

    /// Resolves every immutable TableData field read by the clean gameplay
    /// minimap. A compact row without `m_iMapIcon` remains unresolved rather
    /// than receiving an inferred generic NPC/mob texture.
    #[must_use]
    pub fn gameplay_npc_minimap(&self, npc_type: i32) -> Option<GameplayNpcMinimapDefinition> {
        let npc = self.gameplay_npc(npc_type)?;
        Some(GameplayNpcMinimapDefinition {
            npc_type,
            npc_class: npc.npc_class,
            sound: *self.gameplay_npc_sounds.get(&npc_type)?,
            map_icon: self.gameplay_npc_map_icon(npc_type)?,
        })
    }

    /// Exact clean VendorTable service discovery used by
    /// `NpcIconMode.CheckVendor`.
    #[must_use]
    pub fn gameplay_vendor_items(&self, npc_type: i32) -> Option<&[GameplayVendorItemDefinition]> {
        self.gameplay_vendors.get(&npc_type).map(Vec::as_slice)
    }

    #[must_use]
    pub fn gameplay_npc_is_vendor(&self, npc_type: i32) -> bool {
        self.gameplay_vendor_items(npc_type)
            .is_some_and(|rows| !rows.is_empty())
    }

    /// Exact `m_iServiceNumber -> m_pNpcServiceData[].m_strService`.
    /// An empty string is a valid source value and remains distinguishable
    /// from a missing/unresolved service row.
    #[must_use]
    pub fn gameplay_npc_service(&self, npc_type: i32) -> Option<&str> {
        self.gameplay_npc_services
            .get(&npc_type)
            .map(|(_, text)| text.as_str())
    }

    pub fn gameplay_npc_service_localized(
        &self,
        npc_type: i32,
    ) -> Option<crate::localization::LocalizedText> {
        self.gameplay_npc_services
            .get(&npc_type)
            .map(|(number, text)| {
                crate::localization::LocalizedText::new(
                    format!("tabledata.npc.service.{number}.str_service"),
                    text,
                )
            })
    }

    /// Returns the clean array index for the closest XCom in the explicitly
    /// supplied zone. Callers must not conflate an instance map with zone zero.
    #[must_use]
    pub fn xcom_position(&self, row_index: i32) -> Option<[i32; 3]> {
        self.gameplay_xcoms
            .iter()
            .find(|row| row.row_index == row_index)
            .map(|row| row.position)
    }

    pub fn nearest_xcom_index(&self, zone: i32, position: [i32; 3]) -> Option<i32> {
        clean_nearest_xcom_index(&self.gameplay_xcoms, zone, position)
    }

    /// Resolves `ItemBase.item_id` through the clean GeneralItem table.
    #[must_use]
    pub fn general_item_type(&self, item_id: i16) -> Option<i32> {
        self.gameplay_general_items
            .get(&item_id)
            .map(|item| item.item_type)
    }

    #[must_use]
    pub fn general_item_stim_pack_attribute(&self, item_id: i16) -> Option<i32> {
        self.general_item(item_id)?.stim_pack_attribute
    }

    #[must_use]
    pub fn general_item(&self, item_id: i16) -> Option<&GameplayGeneralItemUiDefinition> {
        self.gameplay_general_items.get(&item_id)
    }

    #[must_use]
    pub fn general_item_icon_path(&self, item_id: i16) -> Option<&str> {
        self.general_item(item_id)?.icon_path.as_deref()
    }

    #[must_use]
    pub fn gameplay_nano(&self, nano_id: i16) -> Option<&GameplayNanoUiDefinition> {
        self.gameplay_nanos.get(&nano_id)
    }

    pub fn gameplay_nanos(&self) -> impl ExactSizeIterator<Item = &GameplayNanoUiDefinition> {
        self.gameplay_nanos.values()
    }

    /// Exact `m_pAvatarTable.m_pAvatarGrowData[level].m_iReqBlob_NanoTune`.
    /// This is intentionally distinct from the Nano-creation Fusion Matter
    /// threshold stored beside it in `m_iReqBlob_NanoCreate`.
    #[must_use]
    pub fn gameplay_nano_tune_fusion_matter(&self, level: i32) -> Option<i32> {
        self.gameplay_nano_tune_fusion_matter.get(&level).copied()
    }

    #[must_use]
    pub fn gameplay_skill_name(&self, skill_id: i16) -> Option<&str> {
        self.gameplay_skill_names.get(&skill_id).map(String::as_str)
    }

    pub fn gameplay_skill(&self, skill_id: i16) -> Option<&GameplaySkillUiDefinition> {
        self.gameplay_skills.get(&skill_id)
    }

    /// Exact clean table-26 `m_iEquipType` used only after the shard confirms
    /// `PC_VEHICLE_ON_SUCC`: 1 selects board presentation and 2/3 select the
    /// scooter family. The protocol item ID is the serialized row index.
    #[must_use]
    pub fn gameplay_vehicle_equip_type(&self, item_id: i16) -> Option<i32> {
        self.gameplay_vehicle_equip_types.get(&item_id).copied()
    }

    pub fn gameplay_vehicle_engine_sound(&self, item_id: i16) -> Option<&str> {
        self.gameplay_vehicle_engine_sounds
            .get(&item_id)
            .map(String::as_str)
    }

    pub fn gameplay_vehicle_speed(&self, item_id: i16) -> Option<i32> {
        self.gameplay_vehicle_speed_classes.get(&item_id).copied()
    }

    #[must_use]
    pub fn gameplay_skill_buff(&self, buff_id: i32) -> Option<&GameplaySkillBuffUiDefinition> {
        self.gameplay_skill_buffs.get(&buff_id)
    }

    pub fn gameplay_skill_buffs(
        &self,
    ) -> impl ExactSizeIterator<Item = &GameplaySkillBuffUiDefinition> {
        self.gameplay_skill_buffs.values()
    }

    /// Resolves the exact four-part clean TableData route. Missing tables,
    /// rows, icon subtables, unsupported icon types, and uninstalled native
    /// assets remain explicit misses.
    #[must_use]
    pub fn gameplay_user_equip_icon(
        &self,
        item_table: u8,
        item_subtable: u8,
        item_row_id: i32,
        icon_subtable: u8,
    ) -> Option<&GameplayUserEquipIconDefinition> {
        self.gameplay_user_equip_icons
            .get(&(item_table, item_subtable, item_row_id, icon_subtable))
    }

    /// Semantic bundle keys follow the item's resolved text row, which can differ
    /// from its protocol identity. Callers keep language switching in LocalizedText.
    pub fn gameplay_try_on_allowed(
        &self,
        item: ffone_protocol::ItemBase0104,
        gender: i32,
        guide: i32,
    ) -> bool {
        if !(0..=6).contains(&item.item_type) || !matches!(gender, 1 | 2) {
            return false;
        }
        let appearance = ((item.option as u32 >> 16) & 0xffff) as i16;
        let id = if appearance > 0 {
            appearance
        } else {
            item.item_id
        };
        self.gameplay_vendor_item_metadata
            .get(&(item.item_type, id))
            .is_some_and(|m| {
                (m.try_on_gender <= 0 || m.try_on_gender == gender)
                    && (m.try_on_guide <= 0 || m.try_on_guide == guide)
            })
    }

    /// Inventory availability follows the base wire item's requirements, even
    /// when a combined item draws a different appearance icon.
    pub fn gameplay_inventory_item_available(
        &self,
        item: ItemBase0104,
        level: i32,
        gender: i32,
        guide: i32,
    ) -> bool {
        self.gameplay_vendor_item_metadata
            .get(&(item.item_type, item.item_id))
            .is_none_or(|metadata| {
                level >= metadata.level
                    && (metadata.try_on_gender <= 0 || metadata.try_on_gender == gender)
                    && (metadata.try_on_guide <= 0 || metadata.try_on_guide == guide)
            })
    }

    /// Appearance icon is independent of the base item's statistics and economy metadata.
    pub fn gameplay_item_display_icon(&self, item: ItemBase0104) -> Option<&str> {
        let appearance = ((item.option as u32) >> 16) as u16;
        let id = if ((0..=6).contains(&item.item_type) || item.item_type == 10) && appearance > 0 {
            i16::try_from(appearance).ok()?
        } else {
            item.item_id
        };
        self.gameplay_vendor_item_metadata
            .get(&(item.item_type, id))?
            .icon_path
            .as_deref()
    }

    pub fn gameplay_user_equip_item_text(
        &self,
        item_type: i16,
        item_id: i16,
    ) -> Option<(
        crate::localization::LocalizedText,
        crate::localization::LocalizedText,
    )> {
        let item = self
            .gameplay_vendor_item_metadata
            .get(&(item_type, item_id))?;
        Some((
            crate::localization::LocalizedText::new(
                format!("{}.str_name", item.text_key_prefix),
                &item.name,
            ),
            crate::localization::LocalizedText::new(
                format!("{}.str_comment", item.text_key_prefix),
                &item.description,
            ),
        ))
    }

    #[must_use]
    pub fn gameplay_user_equip_item_detail(
        &self,
        item_type: i16,
        item_id: i16,
    ) -> Option<GameplayUserEquipItemDetail> {
        let item = self
            .gameplay_vendor_item_metadata
            .get(&(item_type, item_id))?;
        Some(GameplayUserEquipItemDetail {
            name: item.name.clone(),
            description: item.description.clone(),
            level: item.level,
            point_rating: item.point_rating,
            group_rating: item.group_rating,
            defense_rating: item.defense_rating,
            equip_type: item.equip_type,
            target_mode: item.target_mode,
            rarity: item.rarity,
            tradeable: item.tradeable,
            vehicle_speed_class: self.gameplay_vehicle_speed_classes.get(&item_id).copied(),
        })
    }

    #[must_use]
    pub fn journal_nano(&self, nano_id: i32) -> Option<&TutorialNanoJournalUi> {
        self.journal_nanos.get(&nano_id)
    }

    pub fn missions(&self) -> impl ExactSizeIterator<Item = &TutorialMissionDefinition> {
        self.missions.values()
    }

    pub fn missions_for_npc(
        &self,
        npc_type: i32,
    ) -> impl Iterator<Item = &TutorialMissionDefinition> {
        self.mission_tasks_by_npc
            .get(&npc_type)
            .into_iter()
            .flatten()
            .map(|task_id| &self.missions[task_id])
    }

    pub fn npcs(&self) -> impl ExactSizeIterator<Item = &TutorialNpcDefinition> {
        self.npcs.values()
    }

    pub fn rewards(&self) -> impl ExactSizeIterator<Item = &TutorialRewardDefinition> {
        self.rewards.values()
    }

    pub fn scene_events(&self) -> impl ExactSizeIterator<Item = &TutorialSceneEventDefinition> {
        self.scene_events.values()
    }

    pub fn warps(&self) -> impl ExactSizeIterator<Item = &TutorialWarpDefinition> {
        self.warps.values()
    }

    pub(super) fn reward_ui(
        &self,
        mission: &TutorialMissionDefinition,
    ) -> TutorialMissionContentResult<MissionUiRewards> {
        let reward_id = self
            .final_mission(mission.provenance.task_id)?
            .provenance
            .reward_id;
        if reward_id == 0 {
            return Ok(MissionUiRewards::default());
        }
        let reward = self.reward(reward_id)?;
        // Clean `cnMissionJournal.CheckComplete` only serializes selection
        // bitmasks when a reward row actually requires a choice. A fixed item
        // (`m_iBox*Choice == 0`) is granted by the server and must never make
        // the mission row disappear merely because its optional detail card
        // is presentation-only. The primary Retrobution table has fixed item
        // rewards (including reward 106 for task 451), but no choice rows.
        if reward.box1_choice > 0 || reward.box2_choice > 0 {
            return Err(invalid(format!(
                "tutorial reward {reward_id} requires item choices; exact selection projection is required before rendering it"
            )));
        }
        Ok(MissionUiRewards {
            cash: reward.cash,
            fusion_matter: reward.fusion_matter,
        })
    }

    pub(super) fn final_mission(
        &self,
        task_id: i32,
    ) -> TutorialMissionContentResult<&TutorialMissionDefinition> {
        let mut current = self.mission(task_id)?;
        let mut visited = BTreeSet::new();
        loop {
            let current_task_id = current.provenance.task_id;
            if !visited.insert(current_task_id) {
                return Err(invalid(format!(
                    "tutorial mission chain starting at task {task_id} contains a cycle at task {current_task_id}"
                )));
            }
            let outgoing = current.provenance.outgoing_task_id;
            if outgoing == 0 {
                return Ok(current);
            }
            current = self.mission(outgoing)?;
        }
    }

}

impl VendorItemCatalog0104 for TutorialMissionContent {
    fn resolve(&self, item: ItemBase0104) -> Option<VendorItemMetadata0104> {
        let metadata = self
            .gameplay_vendor_item_metadata
            .get(&(item.item_type, item.item_id))?;
        Some(VendorItemMetadata0104 {
            name: metadata.name.clone(),
            level: metadata.level,
            buy_price: metadata.buy_price,
            sell_price: metadata.sell_price,
            sellable: metadata.sellable,
            general_item_type: metadata.general_item_type,
            battery_recharge: metadata.battery_recharge,
            stack_size: metadata.stack_size,
            icon: self
                .gameplay_item_display_icon(item)
                .and_then(|path| VendorIconRef::new(path.to_owned()).ok()),
        })
    }

    fn vehicle_speed_class(&self, item: ItemBase0104) -> Option<i32> {
        // Clean ItemType vehicle is 10. Table 26 is indexed by protocol item
        // id; its serialized m_iItemNumber is not the lookup contract.
        if item.item_type != 10 {
            return None;
        }
        self.gameplay_vehicle_speed_classes
            .get(&item.item_id)
            .copied()
    }
}

impl WorldMapCatalog for TutorialMissionContent {
    fn npc(&self, npc_type: i32) -> WorldMapCatalogLookup<WorldMapNpcCatalogEntry> {
        let Some(npc) = self.gameplay_npc(npc_type) else {
            return WorldMapCatalogLookup::Missing;
        };
        let Some(map_icon) = self.gameplay_npc_map_icon(npc_type) else {
            return WorldMapCatalogLookup::Missing;
        };
        WorldMapCatalogLookup::Unique(WorldMapNpcCatalogEntry {
            display_name: npc.name.clone(),
            map_icon,
            // FFOne does not yet own a normal-world mission-manager snapshot.
            // TableData alone cannot prove either availability predicate used
            // by clean WorldMapMode, so this adapter deliberately fails closed.
            mission: WorldMapMissionAvailability::None,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct UserEquipIconTableSpec {
    pub(super) table_key: &'static str,
    pub(super) item_table: u8,
    pub(super) item_rows_key: &'static str,
    pub(super) declared_item_field: &'static str,
    pub(super) item_subtable: u8,
    pub(super) item_icon_field: &'static str,
    pub(super) icon_rows_key: &'static str,
    pub(super) icon_subtable: u8,
    /// Gear/chest tables are optional as whole tables, but incomplete tables
    /// are invalid. Existing broader content tables retain their prior API:
    /// an optional extra icon subtable may be absent without rejecting them.
    pub(super) require_icon_rows_when_table_present: bool,
}

#[derive(Clone, Copy)]
pub(super) struct VendorItemTableSpec {
    pub(super) table_key: &'static str,
    pub(super) item_type: i16,
    pub(super) item_table: u8,
    pub(super) requires_level: bool,
    pub(super) is_general: bool,
}

#[derive(Clone, Copy)]
pub(super) struct ExpectedWarp {
    pub(super) npc_type: i32,
    pub(super) warp_id: i32,
    pub(super) required_task_id: i32,
    pub(super) target: TutorialWarpTarget,
}
