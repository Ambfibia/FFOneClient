use super::*;

pub const TRANSPORTATION_UI_Z_INDEX: i32 = 8_200;

pub const TRANSPORTATION_ROUTE_STRIDE: f32 = 82.0;

pub const TRANSPORTATION_ROUTE_CONTENT_PADDING: f32 = 10.0;

pub const TRANSPORTATION_ASSET_ROOT: &str = "ui/en/transportation";

pub const TRANSPORTATION_LEFT_BACK_PATH: &str = "ui/en/transportation/skin/left_back.png";

pub const TRANSPORTATION_RIGHT_BACK_PATH: &str = "ui/en/transportation/skin/right_back.png";

pub const TRANSPORTATION_LEFT_BOX_PATH: &str = "ui/en/transportation/skin/left_box.png";

pub const TRANSPORTATION_ROUTE_ROW_PATH: &str = "ui/en/transportation/skin/route_row.png";

pub const TRANSPORTATION_ROUTE_SELECTED_PATH: &str = "ui/en/transportation/skin/route_selected.png";

pub const TRANSPORTATION_ICON_BOX_PATH: &str = "ui/en/transportation/skin/icon_box.png";

pub const TRANSPORTATION_MONKEY_PATH: &str = "ui/en/transportation/decor/monkey_pilot.png";

pub const TRANSPORTATION_BUBBLE_PATH: &str = "ui/en/transportation/decor/bubble.png";

pub const TRANSPORTATION_TAROS_PATH: &str = "ui/en/transportation/decor/taros.png";

pub const TRANSPORTATION_REGISTERED_WARP_PATH: &str =
    "ui/en/transportation/markers/registered_warp.png";

pub const TRANSPORTATION_UNREGISTERED_WARP_PATH: &str =
    "ui/en/transportation/markers/unregistered_warp.png";

pub const TRANSPORTATION_SELECTED_WARP_PATH: &str =
    "ui/en/transportation/markers/selected_warp.png";

pub const TRANSPORTATION_REGISTERED_WYVERN_PATH: &str =
    "ui/en/transportation/markers/registered_wyvern.png";

pub const TRANSPORTATION_UNREGISTERED_WYVERN_PATH: &str =
    "ui/en/transportation/markers/unregistered_wyvern.png";

pub const TRANSPORTATION_SELECTED_WYVERN_PATH: &str =
    "ui/en/transportation/markers/selected_wyvern.png";

pub const TRANSPORTATION_START_LABEL_PATH: &str = "ui/en/transportation/markers/start_label.png";

pub const TRANSPORTATION_FALLBACK_ROUTE_PATH: &str =
    "ui/en/transportation/markers/fallback_route.png";

pub const RETROBUTION_TRANSPORTATION_TABLE_OBJECT_PATH_ID: i64 = 7;

pub const RETROBUTION_WORLD_NAME_OBJECT_PATH_ID: i64 = 8;

pub const RETROBUTION_TRANSPORT_MODE_GAME_OBJECT_PATH_ID: i64 = 1_345;

pub const RETROBUTION_CN_TRANS_COMPONENT_PATH_ID: i64 = 1_583;

pub const RETROBUTION_TRANSPORT_SKIN_PATH_ID: i64 = 1_384;

pub const RETROBUTION_TRANSPORT_SKIN_DEFAULT_FONT_PATH_ID: i64 = 1_018;

pub const RETROBUTION_TRANSPORT_BIGFONT14_PATH_ID: i64 = 903;

pub const RETROBUTION_TRANSPORT_BIGFONT16_PATH_ID: i64 = 1_012;

pub const RETROBUTION_TRANSPORTATION_ROUTE_COUNT: usize = 115;

pub const TRANSPORTATION_SEMANTIC_ASSET_FILES: usize = 23;

pub const TRANSPORTATION_SEMANTIC_ASSET_BYTES: u64 = 70_026;

/// SHA-256 over every sorted UTF-8 semantic path, NUL, little-endian `u64`
/// length, then the exact converted PNG bytes.
pub const TRANSPORTATION_SEMANTIC_ASSET_SET_SHA256: &str =
    "8438ddce15a9dc398f573a8b8f2598033cab51a7e292c9772479361f478c14e9";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportationCatalogProvenance {
    pub asset_path: String,
    pub source_path: String,
    pub bytes: u64,
    pub blake3: String,
    pub sha256: String,
    pub schema: String,
    pub table_key: String,
    pub table_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportationRouteDefinition {
    pub row_index: usize,
    pub vehicle_id: i32,
    pub npc_id: i32,
    pub local_string: i32,
    pub move_type: i32,
    pub start_location: i32,
    pub end_location: i32,
    pub cost: i32,
    pub speed: i32,
    pub mesh: i32,
    pub sound: i32,
    pub route_number: i32,
}

#[derive(Clone, Debug, Resource)]
pub struct TransportationCatalog {
    pub provenance: TransportationCatalogProvenance,
    pub(super) npc_classes: BTreeMap<i32, i32>,
    pub(super) routes: Vec<TransportationRouteDefinition>,
    pub(super) warp_locations: Vec<TransportationLocation>,
    pub(super) broomstick_locations: Vec<TransportationLocation>,
}

impl TransportationCatalog {
    pub fn open(locator: &AssetLocator) -> Result<Self, TransportationCatalogError> {
        let bytes =
            locator
                .read(TABLE_SET_PATH)
                .map_err(|detail| TransportationCatalogError::Io {
                    path: TABLE_SET_PATH.to_owned(),
                    detail,
                })?;
        let hash = blake3::hash(&bytes).to_hex().to_string();
        Self::from_table_set_bytes_with_provenance(
            &bytes,
            TABLE_SET_PATH.to_owned(),
            TABLE_SET_PATH.to_owned(),
            bytes.len() as u64,
            hash,
        )
    }

    pub fn from_project_assets(assets: &AssetLocator) -> Result<Self, TransportationCatalogError> {
        Self::open(assets)
    }

    pub fn from_table_set_bytes(bytes: &[u8]) -> Result<Self, TransportationCatalogError> {
        Self::from_table_set_bytes_with_provenance(
            bytes,
            "<memory>/xdt.json".to_owned(),
            "<memory>".to_owned(),
            bytes.len() as u64,
            blake3::hash(bytes).to_hex().to_string(),
        )
    }

    pub(super) fn from_table_set_bytes_with_provenance(
        bytes: &[u8],
        asset_path: String,
        source_path: String,
        declared_bytes: u64,
        declared_blake3: String,
    ) -> Result<Self, TransportationCatalogError> {
        let document: Value = crate::xdt::from_slice(bytes)
            .map_err(|error| invalid(format!("invalid transportation table-set JSON: {error}")))?;
        let root = value_object(&document, "table-set root")?;
        let schema = required_string(root, "schema", "table-set root")?;
        if schema != TABLE_SET_SCHEMA {
            return Err(invalid(format!(
                "unsupported table-set schema {schema:?}; expected {TABLE_SET_SCHEMA:?}"
            )));
        }
        let tables = required_array(root, "tables", "table-set root")?;
        let table = tables
            .iter()
            .filter_map(Value::as_object)
            .find(|table| {
                table
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name == CONSOLIDATED_TABLE)
            })
            .ok_or_else(|| invalid(format!("missing table document {CONSOLIDATED_TABLE:?}")))?;
        let table_key = required_string(table, "key", "table document")?;
        let table_name = required_string(table, "name", "table document")?;
        let value = required_object(table, "value", "table document")?;

        let npc_table = required_object(value, "m_pNpcTable", CONSOLIDATED_TABLE)?;
        let npc_rows = required_array(npc_table, "m_pNpcData", "m_pNpcTable")?;
        let mut npc_classes = BTreeMap::new();
        for (row_index, row) in npc_rows.iter().enumerate() {
            let context = format!("m_pNpcData[{row_index}]");
            let row = value_object(row, &context)?;
            let npc_number = required_i32(row, "m_iNpcNumber", &context)?;
            let npc_class = required_i32(row, "m_iNpcType", &context)?;
            if let Some(previous) = npc_classes.insert(npc_number, npc_class)
                && previous != npc_class
            {
                return Err(invalid(format!(
                    "contradictory NPC class for m_iNpcNumber {npc_number}"
                )));
            }
        }

        let transportation = required_object(value, "m_pTransportationTable", CONSOLIDATED_TABLE)?;
        let icon_rows = required_array(transportation, "m_pTransIcon", "m_pTransportationTable")?;
        let mut icon_numbers = Vec::with_capacity(icon_rows.len());
        for (row_index, row) in icon_rows.iter().enumerate() {
            let context = format!("m_pTransIcon[{row_index}]");
            let row = value_object(row, &context)?;
            icon_numbers.push(required_i32(row, "m_iIconNumber", &context)?);
        }

        let warp_locations = extract_locations(
            required_array(
                transportation,
                "m_pTransportationWarpLocation",
                "m_pTransportationTable",
            )?,
            required_array(
                transportation,
                "m_pTransportationWarpString",
                "m_pTransportationTable",
            )?,
            &icon_numbers,
            "m_pTransportationWarpLocation",
        )?;
        let broomstick_locations = extract_locations(
            required_array(
                transportation,
                "m_pBroomstickLocation",
                "m_pTransportationTable",
            )?,
            required_array(
                transportation,
                "m_pBroomstickString",
                "m_pTransportationTable",
            )?,
            &icon_numbers,
            "m_pBroomstickLocation",
        )?;
        let route_rows = required_array(
            transportation,
            "m_pTransportationData",
            "m_pTransportationTable",
        )?;
        let mut routes = Vec::with_capacity(route_rows.len());
        for (row_index, row) in route_rows.iter().enumerate() {
            let context = format!("m_pTransportationData[{row_index}]");
            let row = value_object(row, &context)?;
            routes.push(TransportationRouteDefinition {
                row_index,
                vehicle_id: required_i32(row, "m_iVehicleID", &context)?,
                npc_id: required_i32(row, "m_iNPCID", &context)?,
                local_string: required_i32(row, "m_iLocalString", &context)?,
                move_type: required_i32(row, "m_iMoveType", &context)?,
                start_location: required_i32(row, "m_iStartLocation", &context)?,
                end_location: required_i32(row, "m_iEndLocation", &context)?,
                cost: required_i32(row, "m_iCost", &context)?,
                speed: required_i32(row, "m_iSpeed", &context)?,
                mesh: required_i32(row, "m_iMesh", &context)?,
                sound: required_i32(row, "m_iSound", &context)?,
                route_number: required_i32(row, "m_iRouteNum", &context)?,
            });
        }

        Ok(Self {
            provenance: TransportationCatalogProvenance {
                asset_path,
                source_path,
                bytes: declared_bytes,
                blake3: declared_blake3,
                sha256: format!("{:x}", Sha256::digest(bytes)),
                schema: schema.to_owned(),
                table_key: table_key.to_owned(),
                table_name: table_name.to_owned(),
            },
            npc_classes,
            routes,
            warp_locations,
            broomstick_locations,
        })
    }

    #[must_use]
    pub fn npc_class(&self, npc_table_id: i32) -> Option<i32> {
        self.npc_classes.get(&npc_table_id).copied()
    }

    #[must_use]
    pub fn routes(&self) -> &[TransportationRouteDefinition] {
        &self.routes
    }

    #[must_use]
    pub fn warp_locations(&self) -> &[TransportationLocation] {
        &self.warp_locations
    }

    #[must_use]
    pub fn broomstick_locations(&self) -> &[TransportationLocation] {
        &self.broomstick_locations
    }

    /// Exact `NpcIconMode.CheckTransport` projection. Categories 15 and 16
    /// register the location owned by the interacting NPC before the WARP row
    /// can open mode 19; every other category has no registration request.
    #[must_use]
    pub fn registration_intent(
        &self,
        npc_instance_id: i32,
        npc_table_id: i32,
    ) -> Option<TransportationRegistrationIntent> {
        let (transportation_type, locations) = match self.npc_class(npc_table_id)? {
            15 => (
                TransportationService::Warp as i32,
                self.warp_locations.as_slice(),
            ),
            16 => (
                TransportationService::Wyvern as i32,
                self.broomstick_locations.as_slice(),
            ),
            _ => return None,
        };
        let location_id = locations
            .iter()
            .find(|location| location.npc_id == npc_table_id)?
            .location_id;
        (location_id > 0).then_some(TransportationRegistrationIntent {
            transportation_type,
            npc_id: npc_instance_id,
            location_id,
        })
    }

    pub(super) fn project(
        &self,
        context: &TransportationOpenContext,
    ) -> Result<TransportationProjection, TransportationModelError> {
        context.validate()?;
        let service = match context.target {
            TransportationTarget::ItemUse { .. } => TransportationService::ItemUse,
            TransportationTarget::Npc { npc_table_id, .. }
                if self.npc_class(npc_table_id) == Some(16) =>
            {
                TransportationService::Wyvern
            }
            TransportationTarget::Npc { .. } => TransportationService::Warp,
        };
        let mut routes = Vec::new();
        let mut start_position = match context.target {
            TransportationTarget::Npc { npc_position, .. } => {
                TransportationUiPoint::new(npc_position.x, npc_position.z)
            }
            // Exact clean bug: item-use stores Transform.position.y, not z.
            TransportationTarget::ItemUse { .. } => {
                TransportationUiPoint::new(context.player.position.x, context.player.position.y)
            }
        };

        match service {
            TransportationService::Warp | TransportationService::Wyvern => {
                let TransportationTarget::Npc { npc_table_id, .. } = context.target else {
                    unreachable!("service was derived from target");
                };
                let locations = match service {
                    TransportationService::Warp => &self.warp_locations,
                    TransportationService::Wyvern => &self.broomstick_locations,
                    TransportationService::ItemUse => unreachable!(),
                };
                for definition in self
                    .routes
                    .iter()
                    .filter(|route| route.npc_id == npc_table_id)
                {
                    let start = indexed_location(
                        locations,
                        definition.start_location,
                        definition.row_index,
                        "start",
                    )?;
                    let end = indexed_location(
                        locations,
                        definition.end_location,
                        definition.row_index,
                        "end",
                    )?;
                    start_position = start.position;
                    routes.push(project_route(
                        definition.clone(),
                        end,
                        service,
                        &context.player.unlocks,
                    ));
                }
            }
            TransportationService::ItemUse => {
                for location in self
                    .broomstick_locations
                    .iter()
                    .filter(|location| matches!(location.table_zone, 2 | 3))
                {
                    let definition = TransportationRouteDefinition {
                        row_index: usize::MAX,
                        vehicle_id: 1,
                        npc_id: 0,
                        local_string: 0,
                        move_type: 0,
                        start_location: 0,
                        end_location: location.location_id,
                        cost: 0,
                        speed: 0,
                        mesh: 0,
                        sound: 0,
                        route_number: 0,
                    };
                    routes.push(project_route(
                        definition,
                        location,
                        service,
                        &context.player.unlocks,
                    ));
                }
            }
        }

        Ok(TransportationProjection {
            service,
            routes,
            start_position,
        })
    }
}

pub(super) fn project_route(
    definition: TransportationRouteDefinition,
    end: &TransportationLocation,
    service: TransportationService,
    unlocks: &TransportationUnlocks,
) -> TransportationRoute {
    let registered = match service {
        TransportationService::Warp => unlocks.warp_registered(definition.end_location),
        TransportationService::Wyvern | TransportationService::ItemUse => {
            unlocks.wyvern_registered(definition.end_location)
        }
    };
    let icon_path = usize::try_from(end.icon_number)
        .ok()
        .and_then(|index| TRANSPORTATION_TRANSPORT_ICON_PATHS.get(index))
        .copied()
        .unwrap_or(TRANSPORTATION_FALLBACK_ROUTE_PATH);
    let map_marker_path = match (service, registered) {
        (TransportationService::Wyvern, true) => TRANSPORTATION_REGISTERED_WYVERN_PATH,
        (TransportationService::Wyvern, false) => TRANSPORTATION_UNREGISTERED_WYVERN_PATH,
        (_, true) => TRANSPORTATION_REGISTERED_WARP_PATH,
        (_, false) => TRANSPORTATION_UNREGISTERED_WARP_PATH,
    };
    TransportationRoute {
        definition,
        name: end.name.clone(),
        name_localization_key: format!(
            "content.tabledata.transportation.{}.{}.str_location_name",
            if service == TransportationService::Warp { "transportation_warp_string" } else { "broomstick_string" },
            end.row_index,
        ),
        region: end.region.clone(),
        position: end.position,
        icon_number: end.icon_number,
        icon_path,
        map_marker_path,
        registered,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransportationRoute {
    pub definition: TransportationRouteDefinition,
    pub name: String,
    pub name_localization_key: String,
    pub region: String,
    pub position: TransportationUiPoint,
    pub icon_number: i32,
    pub icon_path: &'static str,
    pub map_marker_path: &'static str,
    pub registered: bool,
}

#[derive(Clone, Debug, Default, Resource, PartialEq, Eq)]
pub enum TransportationPresentationAssetStatus {
    #[default]
    Loading,
    Ready,
    Failed {
        asset_path: &'static str,
    },
}

pub(super) fn transportation_presentation_asset_paths() -> Vec<&'static str> {
    let mut paths = vec![
        WORLD_MAP_BACKDROP_PATH,
        WORLD_MAP_CLOSE_PATH,
        WORLD_MAP_CLOSE_HOVER_PATH,
        WORLD_MAP_FREEZONE_PATHS[1],
        WORLD_MAP_DARKLAND_PATHS[1],
        WORLD_MAP_PAYZONE_PATHS[3],
        WORLD_MAP_LINE_PATH,
        WORLD_MAP_LINE_EFFECT_LARGE_PATH,
        WORLD_MAP_LINE_EFFECT_SMALL_PATH,
        TRANSPORTATION_LEFT_BACK_PATH,
        TRANSPORTATION_RIGHT_BACK_PATH,
        TRANSPORTATION_LEFT_BOX_PATH,
        TRANSPORTATION_ROUTE_ROW_PATH,
        TRANSPORTATION_ROUTE_SELECTED_PATH,
        TRANSPORTATION_ICON_BOX_PATH,
        TRANSPORTATION_BLUE_BUTTON_PATH,
        TRANSPORTATION_BLUE_BUTTON_HOVER_PATH,
        TRANSPORTATION_SCROLL_UP_PATH,
        TRANSPORTATION_SCROLL_BAR_PATH,
        TRANSPORTATION_SCROLL_THUMB_PATH,
        TRANSPORTATION_SCROLL_DOWN_PATH,
        TRANSPORTATION_MONKEY_PATH,
        TRANSPORTATION_BUBBLE_PATH,
        TRANSPORTATION_TAROS_PATH,
        TRANSPORTATION_REGISTERED_WARP_PATH,
        TRANSPORTATION_UNREGISTERED_WARP_PATH,
        TRANSPORTATION_SELECTED_WARP_PATH,
        TRANSPORTATION_REGISTERED_WYVERN_PATH,
        TRANSPORTATION_UNREGISTERED_WYVERN_PATH,
        TRANSPORTATION_SELECTED_WYVERN_PATH,
        TRANSPORTATION_START_LABEL_PATH,
        TRANSPORTATION_FALLBACK_ROUTE_PATH,
    ];
    paths.extend(TRANSPORTATION_TRANSPORT_ICON_PATHS);
    paths
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationPresentationRouteRow(pub usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationRouteLayer;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationDynamicRoute;

pub(super) fn transportation_route_name_text(route: &TransportationRoute) -> LocalizedText {
    LocalizedText::new(
        &route.name_localization_key,
        &route.name,
    )
}

pub(super) fn transportation_route_region_text(region: &str) -> LocalizedText {
    let key = match region {
        "The Suburbs" => "suburbs",
        "Downtown" => "downtown",
        "The Wilds" => "wilds",
        "The Darklands" => "darklands",
        "The Future" => "future",
        _ => "unknown",
    };
    LocalizedText::new(format!("content.transportation.region.{key}"), region)
}

pub(super) fn update_transportation_asset_status(
    asset_server: Res<AssetServer>,
    assets: Res<TransportationPresentationAssets>,
    mut status: ResMut<TransportationPresentationAssetStatus>,
) {
    let next = if let Some(asset_path) = assets.failed_path(&asset_server) {
        TransportationPresentationAssetStatus::Failed { asset_path }
    } else if assets.all_loaded(&asset_server) {
        TransportationPresentationAssetStatus::Ready
    } else {
        TransportationPresentationAssetStatus::Loading
    };
    if *status != next {
        *status = next;
    }
}
