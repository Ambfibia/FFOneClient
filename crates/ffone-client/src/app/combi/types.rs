use super::*;

#[derive(Clone, Debug, Resource)]
pub(in super::super) struct CombiProductionCatalog0104 {
    pub(in super::super) recipes: CombiRecipeTable0104,
    pub(in super::super) items: BTreeMap<(i16, i16), CombiItemMetadata0104>,
    pub(in super::super) icons: BTreeMap<(i16, i16), String>,
}

impl CombiProductionCatalog0104 {
    pub(in super::super) fn open(assets: &AssetLocator, content: &TutorialMissionContent) -> Result<Self, String> {
        let bytes = assets.read(COMBI_RECIPE_TABLE_PATH)?;
        Self::from_table_set_bytes(&bytes, |item| {
            UserEquipCatalogQuery::from_non_empty_item(item)
                .ok()
                .and_then(|query| UserEquipItemCatalog::resolve_icon(content, query))
                .map(|icon| icon.runtime_path().to_owned())
        })
    }

    pub(in super::super) fn from_table_set_bytes(
        bytes: &[u8],
        mut resolve_icon_path: impl FnMut(ItemBase0104) -> Option<String>,
    ) -> Result<Self, String> {
        let recipes = CombiRecipeTable0104::from_table_set_bytes(bytes)
            .map_err(|error| format!("clean Combi recipe table rejected: {error}"))?;
        let document: serde_json::Value = ffone_client::xdt::from_slice(bytes)
            .map_err(|error| format!("invalid Combi TableData JSON: {error}"))?;
        let tables = document
            .get("tables")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "Combi TableData root has no tables array".to_owned())?;
        let mut consolidated = tables.iter().filter(|table| {
            table.get("name").and_then(serde_json::Value::as_str)
                == Some("npc_imports_consolidated")
        });
        let value = consolidated
            .next()
            .and_then(|table| table.get("value"))
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "Combi TableData has no consolidated value object".to_owned())?;
        if consolidated.next().is_some() {
            return Err("Combi TableData has duplicate consolidated tables".to_owned());
        }

        let mut items = BTreeMap::new();
        for (table_name, item_type) in COMBI_ITEM_TABLES_0104 {
            let table = value
                .get(table_name)
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| format!("Combi TableData is missing {table_name}"))?;
            let rows = table
                .get("m_pItemData")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("{table_name}.m_pItemData is not an array"))?;
            let strings = table
                .get("m_pItemStringData")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("{table_name}.m_pItemStringData is not an array"))?;
            for (row_index, row) in rows.iter().enumerate() {
                let item_id = i16::try_from(row_index).map_err(|_| {
                    format!("{table_name} row {row_index} does not fit protocol i16")
                })?;
                let row_context = format!("{table_name}.m_pItemData[{row_index}]");
                let row = row
                    .as_object()
                    .ok_or_else(|| format!("{row_context} is not an object"))?;
                let string_index =
                    usize::try_from(combi_table_i32_0104(row, "m_iItemName", &row_context)?)
                        .map_err(|_| format!("{row_context}.m_iItemName is negative"))?;
                let string_context =
                    format!("{row_context}.m_iItemName -> m_pItemStringData[{string_index}]");
                let string = strings
                    .get(string_index)
                    .and_then(serde_json::Value::as_object)
                    .ok_or_else(|| format!("{string_context} is missing or not an object"))?;
                let item = ItemBase0104 {
                    item_type,
                    item_id,
                    option: 0,
                    time_limit: 0,
                };
                let icon_path = resolve_icon_path(item);
                let metadata = CombiItemMetadata0104 {
                    name: combi_table_string_0104(string, "m_strName", &string_context)?.to_owned(),
                    description: combi_table_string_0104(string, "m_strComment", &string_context)?
                        .to_owned(),
                    minimum_level: combi_table_i32_0104(row, "m_iMinReqLev", &row_context)?,
                    required_gender: combi_table_i32_0104(row, "m_iReqSex", &row_context)?,
                    rarity: combi_table_i32_0104(row, "m_iRarity", &row_context)?,
                    // These two fields are serialized cnGuiCombi values. The
                    // clean managed owner never rewrites them from TableData.
                    rarity_label: COMBI_SERIALIZED_RARITY_LABEL_0104.to_owned(),
                    mentor: combi_table_i32_0104(row, "m_iMentor", &row_context)?,
                    // Consolidated recovery omits this schema field on rows
                    // where the clean deserializer leaves its integer at the
                    // managed default zero.
                    cashable: combi_table_i32_default_zero_0104(row, "m_iCashAble", &row_context)?,
                    item_price: combi_table_i32_0104(row, "m_iItemPrice", &row_context)?,
                    point_rating: combi_table_i32_0104(row, "m_iPointRat", &row_context)?,
                    group_rating: combi_table_i32_0104(row, "m_iGroupRat", &row_context)?,
                    defense_rating: combi_table_i32_0104(row, "m_iDefenseRat", &row_context)?,
                    delay_time: combi_table_i32_0104(row, "m_iDelayTime", &row_context)?,
                    equip_type: combi_table_i32_0104(row, "m_iEquipType", &row_context)?,
                    target_mode: combi_table_i32_0104(row, "m_iTargetMode", &row_context)?,
                    type_label: clean_type_label(item_type).to_owned(),
                    trade_label: COMBI_SERIALIZED_TRADE_LABEL_0104.to_owned(),
                    icon_path,
                };
                if items.insert((item_type, item_id), metadata).is_some() {
                    return Err(format!(
                        "duplicate Combi item metadata for type {item_type} row {item_id}"
                    ));
                }
            }
        }
        let mut icons = BTreeMap::new();
        for (table_name, item_type) in EMAIL_ITEM_TABLES_0104 {
            if let Some(rows) = value.get(table_name)
                .and_then(|table| table.get("m_pItemData"))
                .and_then(serde_json::Value::as_array)
            {
                for row_index in 1..rows.len() {
                    let item_id = i16::try_from(row_index)
                        .map_err(|_| format!("{table_name} item ID exceeds protocol i16"))?;
                    if let Some(path) = resolve_icon_path(ItemBase0104 {
                        item_type, item_id, option: 0, time_limit: 0,
                    }) {
                        icons.insert((item_type, item_id), path);
                    }
                }
            }
        }
        Ok(Self { recipes, items, icons })
    }
}

impl CombiItemCatalog0104 for CombiProductionCatalog0104 {
    fn resolve_icon(&self, item_type: i16, item_id: i16) -> Option<String> {
        self.icons.get(&(item_type, item_id)).cloned()
    }
    fn resolve(&self, item_type: i16, item_id: i16) -> Option<CombiItemMetadata0104> {
        self.items.get(&(item_type, item_id)).cloned()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in super::super) enum PendingCombiSystemAction0104 {
    Resolve(CombiSystemModal0104),
    Informational,
}

#[derive(Clone, Debug)]
pub(in super::super) struct PendingCombiSystemMessage0104 {
    pub(in super::super) request: SystemMessageRequest,
    pub(in super::super) action: PendingCombiSystemAction0104,
}

#[derive(Debug, Resource)]
pub(in super::super) struct CombiProductionShell0104 {
    pub(in super::super) pending_outputs: VecDeque<CombiProductionOutput0104>,
    pub(in super::super) pending_system_messages: BTreeMap<u64, PendingCombiSystemMessage0104>,
    pub(in super::super) next_system_message_id: u64,
    pub(in super::super) lease: Option<CombiModeLease0104>,
    pub(in super::super) cursor_was_locked: Option<bool>,
    pub(in super::super) ui_mode_audio: Option<Entity>,
    pub(in super::super) inventory_mode_active: bool,
    pub(in super::super) camera_npc_table_id: Option<i32>,
    pub(in super::super) last_drag_inventory_index: Option<usize>,
    pub(in super::super) last_selection_change: Option<CombiSelectionChange0104>,
    pub(in super::super) last_animation: Option<(i32, CombiNpcAnimation0104)>,
    pub(in super::super) first_use_checks: Vec<i32>,
}

impl Default for CombiProductionShell0104 {
    fn default() -> Self {
        Self {
            pending_outputs: VecDeque::new(),
            pending_system_messages: BTreeMap::new(),
            next_system_message_id: COMBI_SYSTEM_MESSAGE_ID_BASE_0104,
            lease: None,
            cursor_was_locked: None,
            ui_mode_audio: None,
            inventory_mode_active: false,
            camera_npc_table_id: None,
            last_drag_inventory_index: None,
            last_selection_change: None,
            last_animation: None,
            first_use_checks: Vec::new(),
        }
    }
}

impl CombiProductionShell0104 {
    pub(in super::super) fn push_output(&mut self, output: CombiProductionOutput0104) {
        self.pending_outputs.push_back(output);
    }

    pub(in super::super) fn next_request_id(&mut self) -> u64 {
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < COMBI_SYSTEM_MESSAGE_ID_BASE_0104 {
            self.next_system_message_id = COMBI_SYSTEM_MESSAGE_ID_BASE_0104;
        }
        request_id
    }

    pub(in super::super) fn clear_owned_messages(&mut self, messages: &mut SystemMessageUiModel) {
        for request_id in self
            .pending_system_messages
            .keys()
            .copied()
            .collect::<Vec<_>>()
        {
            messages.remove(request_id);
        }
        self.pending_system_messages.clear();
    }

    pub(in super::super) fn stop_ui_mode_audio(&mut self, commands: &mut Commands) {
        if let Some(entity) = self.ui_mode_audio.take() {
            commands.entity(entity).despawn();
        }
    }

    pub(in super::super) fn reset(&mut self, commands: &mut Commands, messages: &mut SystemMessageUiModel) {
        self.stop_ui_mode_audio(commands);
        self.clear_owned_messages(messages);
        *self = Self::default();
    }
}

#[derive(SystemParam)]
pub(in super::super) struct CombiProductionOwners0104<'w, 's> {
    pub(in super::super) runtime: ResMut<'w, CombiProductionRuntime0104>,
    pub(in super::super) state: ResMut<'w, CombiUiState0104>,
    pub(in super::super) projection: ResMut<'w, CombiModeProjection0104>,
    pub(in super::super) outbox: ResMut<'w, CombiUiOutbox0104>,
    pub(in super::super) frames: ResMut<'w, CombiNetworkFrameInbox0104>,
    pub(in super::super) shell: ResMut<'w, CombiProductionShell0104>,
    pub(in super::super) inventory: ResMut<'w, LocalInventoryRuntime>,
    pub(in super::super) system_messages: ResMut<'w, SystemMessageUiModel>,
    pub(in super::super) content: Res<'w, TutorialMissionContent>,
    pub(in super::super) user_equip: ResMut<'w, UserEquipUiState>,
    pub(in super::super) option_runtime: Res<'w, OptionProductionRuntime>,
    pub(in super::super) audio_catalog: Res<'w, NativeAudioCatalog>,
    pub(in super::super) status: ResMut<'w, RuntimeStatus>,
    pub(in super::super) cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}
