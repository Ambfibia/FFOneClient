//! Clean Retrobution ordinary-world mission indicator contracts.
//!
//! `cnMissionManager.StartSelectMissionTask` resolves the selected task's
//! `m_iSTGrantWayPoint` through the first matching row in the ordered
//! `clientnpc.asset` `MiniMapNpc.m_pElements` array. It does not use the
//! nearest or currently streamed NPC. `cnMissionManager.RefreshQuestSymbol`
//! separately projects ES668/ES865/ES866 over the live near-NPC list.
//!
//! The raw Unity container and the patched extraction cache are offline
//! evidence only. Runtime code opens the validated native JSON catalog under
//! `assets/game` and has no legacy-build dependency.

use std::{collections::BTreeSet, fmt::Write as _};

use bevy::prelude::{Resource, Vec3};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{assets::AssetLocator, coordinates::unity_to_native_vector};

pub const CLIENT_NPC_WAYPOINT_CATALOG_PATH: &str = "data/missions/client-npc-waypoints.json";
pub const CLIENT_NPC_WAYPOINT_CATALOG_SCHEMA: &str = "ffone.client-npc-waypoint-catalog.v1";
pub const CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT: usize = 2_903;
pub const CLIENT_NPC_WAYPOINT_CATALOG_BYTES: u64 = 298_059;
pub const CLIENT_NPC_WAYPOINT_CATALOG_SHA256: &str =
    "535A28EC8B83AB5FE3F5BAAB4F884DD77D6BBB9FC5A43FFC0BB11B62252C6A1A";

pub const CLIENT_NPC_WAYPOINT_PRIMARY_CONTAINER: &str = "TableData.resourceFile";
pub const CLIENT_NPC_WAYPOINT_PRIMARY_CONTAINER_BYTES: u64 = 784_963;
pub const CLIENT_NPC_WAYPOINT_PRIMARY_CONTAINER_SHA256: &str =
    "6D4CEA151152E2AB75B7D16590BDA00FBA172600A163E0B3A5318564DF0D7E3B";
pub const CLIENT_NPC_WAYPOINT_PRIMARY_ROUTE: &str = "clientnpc.asset";
pub const CLIENT_NPC_WAYPOINT_PRIMARY_PATH_ID: i64 = 9;
pub const CLIENT_NPC_WAYPOINT_PRIMARY_SCRIPT_PATH_ID: i64 = 3;

/// `cnMissionManager.GetNearList` radius in legacy Unity client-world units.
pub const WORLD_MISSION_INDICATOR_REFRESH_RANGE: f32 = 5_000.0;
pub const WORLD_SMART_INDICATOR_EFFECT_ID: i32 = 668;
pub const WORLD_ADVANCE_SYMBOL_EFFECT_ID: i32 = 865;
pub const WORLD_NEW_SYMBOL_EFFECT_ID: i32 = 866;
pub const WORLD_SMART_INDICATOR_EFFECT_PRIORITY: i32 = 1;
pub const WORLD_QUEST_SYMBOL_EFFECT_PRIORITY: i32 = 0;
pub const WORLD_QUEST_SYMBOL_SCALE: f32 = 1.0;
pub const WORLD_QUEST_SYMBOL_ROOT_HEIGHT_FACTOR: f32 = 0.85;

/// Exact clean `NpcMoveController.MakeGameIcon` selection plus the requested
/// static-service presentation extension.
///
/// Types 13/14 swap the race start/end marker with ring-race state. Type 17
/// swaps its TableData effect for ES811 when this Recall Point is registered.
/// All remaining non-zero, non-30 effects follow the ordinary branch.
///
/// The Bank object (686) retains its static-service icon. SCAMPER's ES683
/// belongs only to category 15 transport operators, not carts, props or
/// category 111 objects carrying the same table effect.
#[must_use]
pub const fn world_npc_game_icon_effect(
    npc_type: i32,
    npc_class: i32,
    table_effect_id: i32,
    ring_race_active: bool,
    rxcom_registered_here: bool,
) -> Option<i32> {
    let table_effect_id = match (npc_type, table_effect_id) {
        (686, 0) => 679,
        _ => table_effect_id,
    };
    if table_effect_id == 683 && npc_class != 15 {
        return None;
    }
    let effect_id = match npc_class {
        0 | 6 => -1,
        13 if !ring_race_active => table_effect_id,
        13 => -1,
        14 if ring_race_active => table_effect_id,
        14 => -1,
        17 if rxcom_registered_here => 811,
        17 => table_effect_id,
        _ if table_effect_id != 0 => table_effect_id,
        _ => -1,
    };
    // ES0 and ES106 are inactive EffectEmitterController shells with `go=0`,
    // no particles and no reproducible visual node. Clean instantiates the
    // empty GameObject, so native visual parity is to render no icon.
    if effect_id <= 0 || matches!(effect_id, 30 | 106) {
        None
    } else {
        Some(effect_id)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientNpcWaypointObjectProvenance {
    pub name: String,
    pub class_id: i32,
    pub type_id: i32,
    pub path_id: i64,
    pub script_path_id: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientNpcWaypointProvenance {
    pub alias: String,
    pub build: String,
    pub relative_container_path: String,
    pub container_size: u64,
    pub container_sha256: String,
    pub route: String,
    pub serialized_asset: String,
    pub object: ClientNpcWaypointObjectProvenance,
    pub coordinate_space: String,
    pub lookup_semantics: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientNpcWaypointRow {
    pub row_index: u32,
    pub npc_type: i32,
    /// Exact `MiniMapNpcElement.kPos` in legacy Unity client-world units.
    pub client_position: [f32; 3],
}

impl ClientNpcWaypointRow {
    #[must_use]
    pub fn client_position_vec3(self) -> Vec3 {
        Vec3::from_array(self.client_position)
    }

    #[must_use]
    pub fn native_position(self) -> Vec3 {
        unity_to_native_vector(self.client_position_vec3())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClientNpcWaypointDocument {
    schema: String,
    source: ClientNpcWaypointProvenance,
    row_count: usize,
    rows: Vec<ClientNpcWaypointRow>,
}

/// Ordered native copy of clean `clientnpc.asset`.
///
/// Duplicate NPC types are intentional. `first_matching_type` therefore uses
/// a linear source-order lookup just like `WorldDataContainer.GetClientNpcPos`.
#[derive(Clone, Debug, Resource)]
pub struct ClientNpcWaypointCatalog {
    provenance: ClientNpcWaypointProvenance,
    rows: Box<[ClientNpcWaypointRow]>,
}

impl ClientNpcWaypointCatalog {
    pub fn open(locator: &AssetLocator) -> Result<Self, String> {
        let bytes = locator.read(CLIENT_NPC_WAYPOINT_CATALOG_PATH)?;
        if bytes.len() as u64 != CLIENT_NPC_WAYPOINT_CATALOG_BYTES {
            return Err(format!(
                "client NPC waypoint catalog length mismatch: expected \
                 {CLIENT_NPC_WAYPOINT_CATALOG_BYTES}, found {}",
                bytes.len()
            ));
        }
        let actual_sha256 = sha256_upper(&bytes);
        if actual_sha256 != CLIENT_NPC_WAYPOINT_CATALOG_SHA256 {
            return Err(format!(
                "client NPC waypoint catalog SHA256 mismatch: expected \
                 {CLIENT_NPC_WAYPOINT_CATALOG_SHA256}, found {actual_sha256}"
            ));
        }
        Self::from_json_bytes(&bytes)
    }

    pub fn from_project_assets(locator: &AssetLocator) -> Result<Self, String> {
        Self::open(locator)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, String> {
        let document = serde_json::from_slice::<ClientNpcWaypointDocument>(bytes)
            .map_err(|error| format!("invalid client NPC waypoint catalog JSON: {error}"))?;
        Self::from_document(document)
    }

    fn from_document(document: ClientNpcWaypointDocument) -> Result<Self, String> {
        if document.schema != CLIENT_NPC_WAYPOINT_CATALOG_SCHEMA {
            return Err(format!(
                "client NPC waypoint catalog must use schema \
                 {CLIENT_NPC_WAYPOINT_CATALOG_SCHEMA:?}, found {:?}",
                document.schema
            ));
        }
        let expected_provenance = expected_provenance();
        if document.source != expected_provenance {
            return Err(format!(
                "client NPC waypoint provenance differs from clean primary evidence: \
                 expected {expected_provenance:?}, found {:?}",
                document.source
            ));
        }
        if document.row_count != CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT
            || document.rows.len() != CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT
        {
            return Err(format!(
                "client NPC waypoint catalog must contain exactly \
                 {CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT} rows, declared {}, found {}",
                document.row_count,
                document.rows.len()
            ));
        }
        for (expected_index, row) in document.rows.iter().enumerate() {
            if row.row_index as usize != expected_index {
                return Err(format!(
                    "client NPC waypoint row {expected_index} declares source index {}",
                    row.row_index
                ));
            }
            if !row.client_position_vec3().is_finite() {
                return Err(format!(
                    "client NPC waypoint row {expected_index} has non-finite client position {:?}",
                    row.client_position
                ));
            }
        }
        Ok(Self {
            provenance: document.source,
            rows: document.rows.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn provenance(&self) -> &ClientNpcWaypointProvenance {
        &self.provenance
    }

    #[must_use]
    pub fn rows(&self) -> &[ClientNpcWaypointRow] {
        &self.rows
    }

    #[must_use]
    pub fn first_matching_type(&self, npc_type: i32) -> Option<&ClientNpcWaypointRow> {
        self.rows.iter().find(|row| row.npc_type == npc_type)
    }
}

fn sha256_upper(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        write!(&mut encoded, "{byte:02X}").expect("writing to a String cannot fail");
    }
    encoded
}

fn expected_provenance() -> ClientNpcWaypointProvenance {
    ClientNpcWaypointProvenance {
        alias: "primary".to_owned(),
        build: "retrobution-20260613".to_owned(),
        relative_container_path: CLIENT_NPC_WAYPOINT_PRIMARY_CONTAINER.to_owned(),
        container_size: CLIENT_NPC_WAYPOINT_PRIMARY_CONTAINER_BYTES,
        container_sha256: CLIENT_NPC_WAYPOINT_PRIMARY_CONTAINER_SHA256.to_owned(),
        route: CLIENT_NPC_WAYPOINT_PRIMARY_ROUTE.to_owned(),
        serialized_asset: "CustomAssetBundle-1dca92eecee4742d985b799d8226666d".to_owned(),
        object: ClientNpcWaypointObjectProvenance {
            name: "clientnpc".to_owned(),
            class_id: 114,
            type_id: -4,
            path_id: CLIENT_NPC_WAYPOINT_PRIMARY_PATH_ID,
            script_path_id: CLIENT_NPC_WAYPOINT_PRIMARY_SCRIPT_PATH_ID,
        },
        coordinate_space: "legacy-unity-client-world-units".to_owned(),
        lookup_semantics: "first-ordered-matching-npc-type".to_owned(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientNpcWaypointPreserveReason {
    MissingNpcType,
    ZeroSourcePosition { source_row_index: u32 },
}

/// State mutation emitted by clean `StartSelectMissionTask` semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClientNpcWaypointUpdate {
    /// No selected task, or its grant waypoint is non-positive.
    Clear,
    /// A positive grant was present, but clean sent no replacement event.
    PreservePrevious {
        npc_type: i32,
        reason: ClientNpcWaypointPreserveReason,
    },
    Set {
        npc_type: i32,
        source_row_index: u32,
        native_position: Vec3,
    },
}

/// Resolves the first selected task's raw `m_iSTGrantWayPoint`.
///
/// Clean deliberately leaves the previous waypoint untouched when a positive
/// type is absent from `clientnpc.asset` or resolves to `Vector3.zero`.
#[must_use]
pub fn resolve_client_npc_waypoint_update(
    grant_waypoint_npc_type: Option<i32>,
    catalog: &ClientNpcWaypointCatalog,
) -> ClientNpcWaypointUpdate {
    let Some(npc_type) = grant_waypoint_npc_type.filter(|npc_type| *npc_type > 0) else {
        return ClientNpcWaypointUpdate::Clear;
    };
    let Some(row) = catalog.first_matching_type(npc_type) else {
        return ClientNpcWaypointUpdate::PreservePrevious {
            npc_type,
            reason: ClientNpcWaypointPreserveReason::MissingNpcType,
        };
    };
    if row.client_position_vec3() == Vec3::ZERO {
        return ClientNpcWaypointUpdate::PreservePrevious {
            npc_type,
            reason: ClientNpcWaypointPreserveReason::ZeroSourcePosition {
                source_row_index: row.row_index,
            },
        };
    }
    ClientNpcWaypointUpdate::Set {
        npc_type,
        source_row_index: row.row_index,
        native_position: row.native_position(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveMissionWaypointTask {
    pub task_id: i32,
    pub mission_id: i32,
    pub grant_waypoint_npc_type: i32,
}

/// The two distinct clean projections owned by manual mission selection.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectedMissionWaypointProjection {
    /// First active task in active-list order for the selected mission.
    pub current_task_id: Option<i32>,
    /// Raw grant value from that first task, including zero.
    pub current_grant_waypoint_npc_type: Option<i32>,
    /// Positive grants from every active task in the selected mission.
    pub smart_indicator_npc_types: BTreeSet<i32>,
}

/// Reproduces `GetSelectedActiveMission` plus the separate ES668 scan.
#[must_use]
pub fn project_selected_mission_waypoints(
    selected_mission_id: Option<i32>,
    active_tasks: &[ActiveMissionWaypointTask],
) -> SelectedMissionWaypointProjection {
    let Some(selected_mission_id) = selected_mission_id else {
        return SelectedMissionWaypointProjection::default();
    };
    let mut projection = SelectedMissionWaypointProjection::default();
    for task in active_tasks
        .iter()
        .filter(|task| task.mission_id == selected_mission_id)
    {
        if projection.current_task_id.is_none() {
            projection.current_task_id = Some(task.task_id);
            projection.current_grant_waypoint_npc_type = Some(task.grant_waypoint_npc_type);
        }
        if task.grant_waypoint_npc_type > 0 {
            projection
                .smart_indicator_npc_types
                .insert(task.grant_waypoint_npc_type);
        }
    }
    projection
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldMissionIndicatorSymbol {
    Advance,
    New,
}

impl WorldMissionIndicatorSymbol {
    #[must_use]
    pub const fn effect_id(self) -> i32 {
        match self {
            Self::Advance => WORLD_ADVANCE_SYMBOL_EFFECT_ID,
            Self::New => WORLD_NEW_SYMBOL_EFFECT_ID,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesiredWorldSmartIndicator {
    pub effect_id: i32,
    pub scale: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DesiredWorldMissionIndicators {
    pub smart: Option<DesiredWorldSmartIndicator>,
    pub quest_symbol: Option<WorldMissionIndicatorSymbol>,
}

/// Inputs already reduced from active mission/task state for one live NPC.
///
/// Mission Finder is intentionally absent. Clean uses it only while drawing
/// minimap/world-map icons; it never gates ES668, ES865, or ES866 in 3D.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMissionIndicatorEligibilityInput {
    /// Whether this NPC was returned by the current 5000-unit near-list scan.
    pub in_refresh_near_list: bool,
    /// Exact `NpcTableElement.m_iNpcType`/native `npc_class`.
    pub npc_class: i32,
    /// Any selected-mission active task grants a waypoint to this NPC type.
    pub selected_waypoint_target: bool,
    /// `NpcIconMode` currently force-deletes ES668 during interaction.
    pub smart_force_deleted: bool,
    /// Exact `m_iRadius` protocol centiunits.
    pub radius_server_units: i32,
    /// Any active task ID occurs in this NPC type's terminate-task list.
    pub has_active_terminating_task: bool,
    /// Grant/repeat/completed/active/start-condition checks all passed.
    pub has_new_mission_available: bool,
}

/// Clean does not touch an NPC that is outside the near list or belongs to an
/// outer-skipped class, so callers must distinguish that from reconciling to
/// an empty desired set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorldMissionIndicatorRefresh {
    PreservePrevious,
    Reconcile(DesiredWorldMissionIndicators),
}

#[must_use]
pub const fn npc_class_refreshes_world_mission_indicators(npc_class: i32) -> bool {
    !matches!(npc_class, 0 | 25)
}

#[must_use]
pub const fn npc_class_supports_world_quest_symbols(npc_class: i32) -> bool {
    !matches!(npc_class, 0 | 25 | 100 | 101 | 105 | 110 | 111)
}

#[must_use]
pub fn world_smart_indicator_scale(radius_server_units: i32) -> f32 {
    radius_server_units as f32 * 0.01 * 2.0
}

/// Pure ES668/ES865/ES866 projection for one clean refresh visit.
///
/// Advance wins over New and intentionally does not depend on objective
/// completion. The caller owns attachment resolution and named-effect diffing.
#[must_use]
pub fn project_world_mission_indicator_refresh(
    input: WorldMissionIndicatorEligibilityInput,
) -> WorldMissionIndicatorRefresh {
    if !input.in_refresh_near_list || !npc_class_refreshes_world_mission_indicators(input.npc_class)
    {
        return WorldMissionIndicatorRefresh::PreservePrevious;
    }

    let smart = (input.selected_waypoint_target && !input.smart_force_deleted).then(|| {
        DesiredWorldSmartIndicator {
            effect_id: WORLD_SMART_INDICATOR_EFFECT_ID,
            scale: world_smart_indicator_scale(input.radius_server_units),
        }
    });
    let quest_symbol = if !npc_class_supports_world_quest_symbols(input.npc_class) {
        None
    } else if input.has_active_terminating_task {
        Some(WorldMissionIndicatorSymbol::Advance)
    } else if input.has_new_mission_available {
        Some(WorldMissionIndicatorSymbol::New)
    } else {
        None
    };

    WorldMissionIndicatorRefresh::Reconcile(DesiredWorldMissionIndicators {
        smart,
        quest_symbol,
    })
}

#[cfg(test)]
mod tests;
