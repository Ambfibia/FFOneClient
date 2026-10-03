use super::*;

pub(super) const CHARACTER_REGISTRY_SCHEMA: &str = "ffone.semantic-character-registry.v2";

pub(super) const CATALOG_ROW_HEIGHT: f32 = 58.0;

pub(super) const CATALOG_ROW_GAP: f32 = 4.0;

pub(super) const EDITOR_CATALOG_WIDTH: f32 = 310.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CatalogKind {
    Npc,
    Nano,
    Equipment,
}

impl CatalogKind {
    pub(super) const fn title(self) -> &'static str {
        match self {
            Self::Npc => "NPC",
            Self::Nano => "NANO",
            Self::Equipment => "",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct EditorCatalogEntry {
    pub(super) kind: CatalogKind,
    pub(super) display_name: String,
    pub(super) semantic_id: String,
    pub(super) logical_name: String,
    pub(super) glb: String,
    pub(super) icon_path: Option<String>,
    pub(super) npc_visual: Option<NetworkNpcVisualDefinition0104>,
    pub(super) hnpc_visual: Option<NetworkHnpcVisualDefinition0104>,
    pub(super) animations: Vec<String>,
    pub(super) network_id: Option<i64>,
    pub(super) table_index: Option<usize>,
    pub(super) scale: Option<f32>,
    pub(super) height_server_units: Option<i64>,
    pub(super) level: Option<i64>,
    pub(super) style: Option<i64>,
    pub(super) team_or_set: Option<i64>,
    pub(super) texture_main: Option<String>,
    pub(super) texture_sub: Option<String>,
    pub(super) native_extension: bool,
}

#[derive(Debug, Resource)]
pub(super) struct EditorCatalog {
    pub(super) entries: Vec<EditorCatalogEntry>,
    pub(super) hnpc: Option<HnpcRuntimeCatalog>,
    pub(super) npc_count: usize,
    pub(super) nano_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CharacterRegistryDocument {
    pub(super) schema: String,
    pub(super) models: Vec<CharacterRegistryModel>,
}

#[allow(dead_code)]
impl EditorCatalog {
    pub(super) fn open(locator: &AssetLocator) -> Result<Self, String> {
        let registry: CharacterRegistryDocument = locator.read_character_models()?;
        if registry.schema != CHARACTER_REGISTRY_SCHEMA {
            return Err(format!(
                "character registry uses {:?}; expected {CHARACTER_REGISTRY_SCHEMA:?}",
                registry.schema
            ));
        }
        let content = TutorialMissionContent::open(locator).map_err(|error| error.to_string())?;
        let npc_visuals = NetworkNpcVisualCatalog0104::open(locator)?;
        let base_rig = NativePlayerRigCatalog::open(locator.root())?;
        let hnpc = HnpcRuntimeCatalog::open(locator, &base_rig)?;
        let nano_portraits = GameplayNanoPortraitCatalog::open(locator)?;
        let models_by_glb = registry
            .models
            .into_iter()
            .map(|model| (model.glb.clone(), model))
            .collect::<BTreeMap<_, _>>();

        let mut entries =
            Vec::with_capacity(content.gameplay_npcs().len() + content.gameplay_nanos().len());
        for npc in content.gameplay_npcs() {
            let visual = npc_visuals.get(npc.npc_type).cloned();
            let hnpc_visual = npc_visuals.get_hnpc(npc.npc_type).cloned();
            let hnpc_rig = hnpc_visual
                .as_ref()
                .and_then(|definition| hnpc.appearance(definition.appearance_index))
                .and_then(|appearance| appearance.look.as_ref())
                .map(|look| hnpc.rig_catalog().gender(look.gender))
                .transpose()?;
            let model = visual
                .as_ref()
                .and_then(|definition| models_by_glb.get(&definition.glb));
            entries.push(EditorCatalogEntry {
                kind: CatalogKind::Npc,
                display_name: npc.name.clone(),
                semantic_id: model
                    .map(|model| model.id.clone())
                    .unwrap_or_else(|| format!("xdt/npc/{}", npc.npc_type)),
                logical_name: visual
                    .as_ref()
                    .map(|definition| definition.logical_name.clone())
                    .unwrap_or_else(|| format!("xdt_npc_{}", npc.npc_type)),
                glb: visual
                    .as_ref()
                    .map(|definition| definition.glb.clone())
                    .or_else(|| hnpc_rig.map(|rig| rig.skeleton_glb.clone()))
                    .unwrap_or_default(),
                icon_path: content
                    .gameplay_npc_portrait_icon_path(npc.npc_type)
                    .map(str::to_owned),
                npc_visual: visual.clone(),
                hnpc_visual,
                animations: model
                    .map(|model| model.animations.clone())
                    .or_else(|| {
                        hnpc_rig.map(|rig| rig.clips.iter().map(|clip| clip.name.clone()).collect())
                    })
                    .unwrap_or_default(),
                network_id: Some(i64::from(npc.npc_type)),
                table_index: None,
                scale: visual.as_ref().map(|definition| definition.table_scale),
                height_server_units: Some(i64::from(npc.height_server_units)),
                level: Some(i64::from(npc.npc_level)),
                style: Some(i64::from(npc.npc_style)),
                team_or_set: Some(i64::from(npc.team)),
                texture_main: visual
                    .as_ref()
                    .and_then(|definition| definition.main_texture.as_ref())
                    .map(|texture| texture.true_name.clone()),
                texture_sub: visual
                    .as_ref()
                    .and_then(|definition| definition.sub_texture.as_ref())
                    .map(|texture| texture.true_name.clone()),
                native_extension: false,
            });
        }
        let npc_count = entries.len();
        for nano in content.gameplay_nanos() {
            let glb = nano_portraits.model_path(nano.nano_id).unwrap_or_default();
            let model = models_by_glb.get(glb);
            entries.push(EditorCatalogEntry {
                kind: CatalogKind::Nano,
                display_name: nano.name.clone(),
                semantic_id: model
                    .map(|model| model.id.clone())
                    .unwrap_or_else(|| format!("xdt/nano/{}", nano.nano_id)),
                logical_name: model
                    .map(|model| model.logical_name.clone())
                    .unwrap_or_else(|| format!("xdt_nano_{}", nano.nano_id)),
                glb: glb.to_owned(),
                icon_path: nano.icon_path.clone(),
                npc_visual: None,
                hnpc_visual: None,
                animations: model
                    .map(|model| model.animations.clone())
                    .unwrap_or_default(),
                network_id: Some(i64::from(nano.nano_id)),
                table_index: None,
                scale: None,
                height_server_units: None,
                level: None,
                style: Some(i64::from(nano.style)),
                team_or_set: Some(i64::from(nano.sort_number)),
                texture_main: None,
                texture_sub: None,
                native_extension: false,
            });
        }
        // The browser lists native models, including inactive/event Nanos with
        // no network row. Keep the named Van Kleiss row over its old event alias.
        let van_kleiss = entries
            .iter()
            .find(|entry| entry.kind == CatalogKind::Nano && entry.display_name == "Van Kleiss")
            .map(|entry| entry.glb.clone());
        if let Some(glb) = van_kleiss.filter(|glb| !glb.is_empty()) {
            entries.retain(|entry| {
                entry.kind != CatalogKind::Nano
                    || entry.glb != glb
                    || entry.display_name == "Van Kleiss"
            });
        }
        let represented = entries
            .iter()
            .filter(|entry| entry.kind == CatalogKind::Nano)
            .map(|entry| entry.glb.clone())
            .collect::<BTreeSet<_>>();
        let mut extensions = models_by_glb
            .values()
            .filter(|model| model.category == "nano" && !represented.contains(&model.glb))
            .collect::<Vec<_>>();
        extensions.sort_by(|a, b| a.id.cmp(&b.id));
        for model in extensions {
            locator.require_file(&model.glb)?;
            let (name, icon) = match model.logical_name.as_str() {
                "nano_holonano" => ("Unstable Nano", "holo-nano"),
                "nano_upgrade" => ("Upgrade", "upgrade"),
                "nano_ghostfreak" => ("Ghostfreak", "ghostfreak"),
                "nano_ben" => ("Ben Tennyson", "ben-10"),
                "nano_flapjack" => ("Flapjack", "flapjack"),
                "nano_johnnybravo" => ("Johnny Bravo", "johnny-bravo"),
                _ => (model.logical_name.as_str(), "missing"),
            };
            let icon = format!("icons/entities/nanos/nanoicon_{icon}.png");
            locator.require_file(&icon)?;
            entries.push(EditorCatalogEntry {
                kind: CatalogKind::Nano,
                display_name: name.to_owned(),
                semantic_id: model.id.clone(),
                logical_name: model.logical_name.clone(),
                glb: model.glb.clone(),
                icon_path: Some(icon),
                npc_visual: None,
                hnpc_visual: None,
                animations: model.animations.clone(),
                network_id: None,
                table_index: None,
                scale: None,
                height_server_units: None,
                level: None,
                style: None,
                team_or_set: None,
                texture_main: None,
                texture_sub: None,
                native_extension: true,
            });
        }
        let nano_count = entries.len() - npc_count;
        if npc_count == 0 || nano_count == 0 {
            return Err("production XDT projection must contain NPC and Nano rows".to_owned());
        }
        Ok(Self {
            hnpc: Some(hnpc),
            entries,
            npc_count,
            nano_count,
        })
    }

    pub(super) fn from_documents(
        table_set: &Value,
        registry_models: Vec<CharacterRegistryModel>,
    ) -> Result<Self, String> {
        let mut models_by_key = BTreeMap::<(CatalogKind, String), CharacterRegistryModel>::new();
        for model in registry_models {
            let kind = match model.category.as_str() {
                "npc" => CatalogKind::Npc,
                "nano" => CatalogKind::Nano,
                _ => continue,
            };
            models_by_key.insert((kind, model.logical_name.clone()), model);
        }

        let table = table_set
            .get("tables")
            .and_then(Value::as_array)
            .and_then(|tables| {
                let matches = tables
                    .iter()
                    .filter(|table| {
                        table.get("name").and_then(Value::as_str) == Some(CONSOLIDATED_TABLE)
                    })
                    .collect::<Vec<_>>();
                (matches.len() == 1).then_some(matches[0])
            })
            .ok_or_else(|| format!("table-set must contain one {CONSOLIDATED_TABLE:?}"))?;

        let mut entries = Vec::new();
        let mut consumed_models = BTreeSet::<(CatalogKind, String)>::new();
        Self::append_npc_rows(table, &models_by_key, &mut consumed_models, &mut entries)?;
        Self::append_nano_rows(table, &models_by_key, &mut consumed_models, &mut entries)?;

        for ((kind, logical_name), model) in models_by_key {
            if consumed_models.contains(&(kind, logical_name.clone())) {
                continue;
            }
            entries.push(EditorCatalogEntry {
                kind,
                display_name: humanize_name(&logical_name),
                semantic_id: model.id,
                logical_name,
                glb: model.glb,
                icon_path: None,
                npc_visual: None,
                hnpc_visual: None,
                animations: model.animations,
                network_id: None,
                table_index: None,
                scale: (kind == CatalogKind::Npc).then_some(1.0),
                height_server_units: None,
                level: None,
                style: None,
                team_or_set: None,
                texture_main: None,
                texture_sub: None,
                native_extension: true,
            });
        }

        entries.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.native_extension.cmp(&right.native_extension))
                .then_with(|| {
                    left.network_id
                        .unwrap_or(i64::MAX)
                        .cmp(&right.network_id.unwrap_or(i64::MAX))
                })
                .then_with(|| left.display_name.cmp(&right.display_name))
        });
        let npc_count = entries
            .iter()
            .filter(|entry| entry.kind == CatalogKind::Npc)
            .count();
        let nano_count = entries.len() - npc_count;
        if npc_count == 0 || nano_count == 0 {
            return Err("editor catalog must resolve both NPC and Nano models".to_owned());
        }
        Ok(Self {
            hnpc: None,
            entries,
            npc_count,
            nano_count,
        })
    }

    pub(super) fn append_npc_rows(
        table: &Value,
        models: &BTreeMap<(CatalogKind, String), CharacterRegistryModel>,
        consumed: &mut BTreeSet<(CatalogKind, String)>,
        entries: &mut Vec<EditorCatalogEntry>,
    ) -> Result<(), String> {
        let npc_table = table
            .pointer("/value/m_pNpcTable")
            .ok_or_else(|| "consolidated table has no m_pNpcTable".to_owned())?;
        let rows = array(npc_table, "m_pNpcData")?;
        let meshes = array(npc_table, "m_pNpcMeshData")?;
        for (row_index, row) in rows.iter().enumerate() {
            let network_id = integer(row, "m_iNpcNumber").unwrap_or_default();
            if network_id <= 0 || integer(row, "m_iHNpc").unwrap_or_default() != 0 {
                continue;
            }
            let Some(mesh_index) = positive_index(row, "m_iMesh") else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_index) else {
                continue;
            };
            let Some(logical_name) = native_string(mesh, "m_pstrMMeshModelString") else {
                continue;
            };
            let key = (CatalogKind::Npc, logical_name.to_owned());
            let Some(model) = models.get(&key) else {
                continue;
            };
            consumed.insert(key);
            entries.push(EditorCatalogEntry {
                kind: CatalogKind::Npc,
                display_name: humanize_name(logical_name),
                semantic_id: model.id.clone(),
                logical_name: model.logical_name.clone(),
                glb: model.glb.clone(),
                icon_path: None,
                npc_visual: None,
                hnpc_visual: None,
                animations: model.animations.clone(),
                network_id: Some(network_id),
                table_index: Some(row_index),
                scale: number(row, "m_fScale").filter(|value| *value > 0.0),
                height_server_units: integer(row, "m_iHeight").filter(|value| *value > 0),
                level: integer(row, "m_iNpcLevel"),
                style: integer(row, "m_iNpcStyle"),
                team_or_set: integer(row, "m_iTeam"),
                texture_main: native_string(mesh, "m_pstrMTextureString").map(str::to_owned),
                texture_sub: native_string(mesh, "m_pstrMTextureString2").map(str::to_owned),
                native_extension: false,
            });
        }
        Ok(())
    }

    pub(super) fn append_nano_rows(
        table: &Value,
        models: &BTreeMap<(CatalogKind, String), CharacterRegistryModel>,
        consumed: &mut BTreeSet<(CatalogKind, String)>,
        entries: &mut Vec<EditorCatalogEntry>,
    ) -> Result<(), String> {
        let nano_table = table
            .pointer("/value/m_pNanoTable")
            .ok_or_else(|| "consolidated table has no m_pNanoTable".to_owned())?;
        let rows = array(nano_table, "m_pNanoData")?;
        let meshes = array(nano_table, "m_pNanoMeshData")?;
        for (row_index, row) in rows.iter().enumerate() {
            let network_id = integer(row, "m_iNanoNumber").unwrap_or_default();
            if network_id <= 0 {
                continue;
            }
            let Some(mesh_index) = positive_index(row, "m_iMesh") else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_index) else {
                continue;
            };
            let Some(logical_name) = native_string(mesh, "m_pstrMMeshModelString") else {
                continue;
            };
            let key = (CatalogKind::Nano, logical_name.to_owned());
            let Some(model) = models.get(&key) else {
                continue;
            };
            consumed.insert(key);
            entries.push(EditorCatalogEntry {
                kind: CatalogKind::Nano,
                display_name: humanize_name(logical_name),
                semantic_id: model.id.clone(),
                logical_name: model.logical_name.clone(),
                glb: model.glb.clone(),
                icon_path: None,
                npc_visual: None,
                hnpc_visual: None,
                animations: model.animations.clone(),
                network_id: Some(network_id),
                table_index: Some(row_index),
                scale: None,
                height_server_units: None,
                level: None,
                style: integer(row, "m_iStyle"),
                team_or_set: integer(row, "m_iNanoSet"),
                texture_main: native_string(mesh, "m_pstrMTextureString").map(str::to_owned),
                texture_sub: native_string(mesh, "m_pstrMTextureString2").map(str::to_owned),
                native_extension: false,
            });
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub(super) fn positive_index(value: &Value, key: &str) -> Option<usize> {
    usize::try_from(integer(value, key)?).ok()
}

#[derive(Component)]
pub(super) struct CatalogSlot(pub(super) usize);

#[derive(Component)]
pub(super) struct CatalogScroll;

#[derive(Component)]
pub(super) struct CatalogScrollbar;

#[derive(Component)]
pub(super) struct CatalogScrollThumb;

pub(super) fn scroll_catalog_list(
    mut wheel_events: MessageReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    state: Res<EditorState>,
    mut filter: Local<Option<(CatalogKind, String)>>,
    mut drag_offset: Local<Option<f32>>,
    mut viewport: Single<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        With<CatalogScroll>,
    >,
    track: Single<(&RelativeCursorPosition, &ComputedNode), With<CatalogScrollbar>>,
) {
    let (cursor, scroll, computed) = &mut *viewport;
    let current_filter = (state.kind, state.search.clone());
    if filter.as_ref() != Some(&current_filter) {
        scroll.y = 0.0;
        *drag_offset = None;
        *filter = Some(current_filter);
    }
    let max_scroll = ((computed.content_size() - computed.size())
        * computed.inverse_scale_factor())
    .max(Vec2::ZERO)
    .y;
    let mut position = scroll.y.clamp(0.0, max_scroll);
    for event in wheel_events.read() {
        // Geometric hover stays true over child buttons, images and labels.
        if !cursor.cursor_over() && !track.0.cursor_over() {
            continue;
        }
        let delta = match event.unit {
            MouseScrollUnit::Line => -event.y * 44.0,
            MouseScrollUnit::Pixel => -event.y,
        };
        position = (position + delta).clamp(0.0, max_scroll.max(0.0));
    }
    let height = track.1.size().y * track.1.inverse_scale_factor();
    let (thumb_top, thumb_height) = catalog_scroll_thumb(height, max_scroll, position);
    if buttons.just_pressed(MouseButton::Left)
        && track.0.cursor_over()
        && let Some(cursor) = track.0.normalized
    {
        let y = (cursor.y + 0.5) * height;
        *drag_offset = Some(if (thumb_top..=thumb_top + thumb_height).contains(&y) {
            y - thumb_top
        } else {
            thumb_height * 0.5
        });
    }
    if !buttons.pressed(MouseButton::Left) {
        *drag_offset = None;
    }
    if let (Some(offset), Some(cursor)) = (*drag_offset, track.0.normalized) {
        let travel = height - thumb_height;
        if travel > 0.0 {
            position = (((cursor.y + 0.5) * height - offset) / travel).clamp(0.0, 1.0) * max_scroll;
        }
    }
    scroll.y = position;
}

pub(super) fn catalog_scroll_thumb(height: f32, max_scroll: f32, position: f32) -> (f32, f32) {
    if height <= 0.0 || max_scroll <= 0.0 {
        return (0.0, height.max(0.0));
    }
    let thumb = (height * height / (height + max_scroll)).clamp(30.0_f32.min(height), height);
    (
        (height - thumb) * (position / max_scroll).clamp(0.0, 1.0),
        thumb,
    )
}

pub(super) fn bind_catalog_scrollbar(
    viewport: Single<(&ScrollPosition, &ComputedNode), With<CatalogScroll>>,
    track: Single<&ComputedNode, With<CatalogScrollbar>>,
    mut thumb: Single<(&mut Node, &mut BackgroundColor), With<CatalogScrollThumb>>,
) {
    let max_scroll = ((viewport.1.content_size().y - viewport.1.size().y)
        * viewport.1.inverse_scale_factor())
    .max(0.0);
    let (top, height) = catalog_scroll_thumb(
        track.size().y * track.inverse_scale_factor(),
        max_scroll,
        viewport.0.y,
    );
    thumb.0.top = px(top);
    thumb.0.height = px(height);
    thumb.1.0 = if max_scroll > 0.0 {
        Color::srgb(0.22, 0.65, 0.75)
    } else {
        Color::srgb(0.06, 0.15, 0.18)
    };
}
