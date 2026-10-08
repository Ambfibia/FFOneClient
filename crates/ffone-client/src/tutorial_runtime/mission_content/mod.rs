//! Verified XDT-backed content used by the Retrobution tutorial mission UI.
//!
//! This module owns no Bevy systems and does not contain fallback mission text.
//! It reads the stable gameplay table-set root and projects only the source
//! rows required by the tutorial.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs,
};

use bevy::prelude::Resource;
use ffone_protocol::{ItemBase0104, PcWarpUseNpcRequest0104};
use serde_json::{Map, Value};

use crate::{
    assets::{AssetLocator, TABLE_SET_PATH},
    mission_ui::{MissionUiEntry, MissionUiRewards},
    system_message_ui::SystemMessageButtonType,
    tutorial_actors::tutorial_actor_npc_types,
    vendor_ui::{VendorIconRef, VendorItemCatalog0104, VendorItemMetadata0104},
    world_map::{
        WorldMapCatalog, WorldMapCatalogLookup, WorldMapMissionAvailability,
        WorldMapNpcCatalogEntry,
    },
};

#[cfg(test)]
mod tests;

mod constants;
mod assets_clean_nearest_xcom_index;
mod validation;
mod types_gameplay_warp_definition;
mod types_tutorial_mission_content;
mod types_tutorial_mission_content_open;
mod types_tutorial_mission_content_gameplay_mission_na;
mod commands;
mod state;
mod input;
mod operations_extract_journal_nanos;
mod operations_extract_missions;
mod mission_visibility;
mod operations_extract_gameplay_npcs;
mod operations_extract_warps;
mod containers;

pub use constants::{
    TUTORIAL_MISSION_TASK_IDS, TUTORIAL_MISSION_NPC_TYPES, TUTORIAL_WARP_NPC_TYPES
};
pub use mission_visibility::{MissionMarkerSurface, MissionMarkerVisibility};
use constants::{
    TABLE_SET_SCHEMA, CONSOLIDATED_TABLE, OPTIONAL_USER_EQUIP_ITEM_TABLES,
    EXISTING_USER_EQUIP_ICON_SUBTABLES, VENDOR_ITEM_TABLES, EXPECTED_WARPS,
    GUIDE_FIRST_WARP_ID, GUIDE_FIRST_WARP_NPC_TYPE, GUIDE_FIRST_WARP_TARGET
};
use assets_clean_nearest_xcom_index::{
    TutorialAssetSource, avatar_util_semantic_icon_path, nano_ready_semantic_icon_path,
    general_item_semantic_icon_path
};
pub use assets_clean_nearest_xcom_index::{GameplayWarpItemLookup, clean_nearest_xcom_index};
pub use validation::{TutorialMissionContentError, GameplayWarpResolveError};
pub use types_gameplay_warp_definition::{
    TutorialMissionContentResult, TutorialTableSetProvenance, TutorialMissionType,
    TutorialMissionRowProvenance, TutorialMissionJournalProvenance,
    TutorialMissionJournalText, TutorialMissionDefinition, TutorialMissionDialogue,
    TutorialMissionNanocomMessage, GameplayGuideNanocomDefinition, TutorialRewardItem,
    TutorialRewardDefinition, TutorialSceneTextProvenance, TutorialSceneTextDefinition,
    TutorialNpcRowProvenance, TutorialNpcDefinition, TutorialWarpTarget,
    TutorialWarpRowProvenance, TutorialWarpDefinition, GameplayWarpDefinition,
    GameplayWarpWorldPosition, GameplayWarpServerPosition, GameplayWarpGroupMember,
    GameplayWarpEligibilityInput, GameplayWarpDenial, GameplayWarpEligibility, XdtF32,
    GameplayNpcUiDefinition, GameplayNpcBarkerDefinition, GameplayNpcSkillBarkerDefinition,
    GameplayNpcMinimapDefinition, GameplayXcomDefinition, GameplayGeneralItemUiDefinition,
    GameplayVendorItemDefinition
};
use types_gameplay_warp_definition::GameplayVendorItemMetadataDefinition;
pub use types_tutorial_mission_content::{
    GameplayUserEquipItemDetail, GameplayUserEquipIconDefinition, GameplayNanoUiDefinition,
    GameplaySkillUiDefinition, GameplaySkillBuffUiDefinition, TutorialNanoJournalSkillUi,
    TutorialNanoJournalUi, SystemMessageDefinition, TutorialMissionContent
};
use types_tutorial_mission_content_gameplay_mission_na::{
    UserEquipIconTableSpec, VendorItemTableSpec, ExpectedWarp
};
pub use commands::{TutorialSceneEventProvenance, TutorialSceneEventDefinition};
pub use state::GameplayWarpInventoryLocation;
use input::collect_relative_files;
use operations_extract_journal_nanos::{
    extract_gameplay_nanos, extract_gameplay_nano_tune_fusion_matter, extract_gameplay_skills,
    extract_gameplay_skill_buffs, extract_journal_nanos, extract_mission_dialogue,
    extract_mission_nanocom_message, extract_gameplay_guide_nanocom
};
#[cfg(test)]
use operations_extract_journal_nanos::merged_nano_named_icon_slug;
use operations_extract_missions::{
    extract_missions, extract_rewards, extract_scene_events, extract_gameplay_user_equip_icons,
    extract_gameplay_npc_portrait_icon_paths, extract_gameplay_vendor_item_metadata,
    extract_gameplay_vehicle_speed_classes, extract_gameplay_vehicle_equip_types
};
use operations_extract_gameplay_npcs::{
    append_user_equip_icon_table, primary_nano_icon_slug, extract_gameplay_general_items,
    extract_quest_item_names, optional_i32, extract_system_messages, extract_gameplay_xcoms,
    extract_gameplay_npc_services, extract_gameplay_vendors, extract_gameplay_npc_map_icons,
    extract_gameplay_npc_sounds, extract_gameplay_npc_barker_types,
    extract_gameplay_npc_skill_barkers, extract_gameplay_npcs
};
use operations_extract_warps::{
    extract_npcs, extract_gameplay_warps, extract_warps, indexed_string,
    indexed_string_allow_empty, required_array, required_string, required_i32,
    required_i32_array2, required_i32_array3, required_i32_array4, required_i32_array5,
    invalid, extract_vehicle_engine_sounds
};
use containers::{indexed_table_object, value_object, required_object};
