use std::path::{Path, PathBuf};

use serde_json::json;
use tempfile::TempDir;

use crate::tutorial_mission_content::*;
use crate::vendor_ui::VendorSystemMessageId0104;

mod types;
mod operations_compact_document;
mod operations_compact_fixture_builds_ui_entries_and_source;
mod operations_real_assets_game_table_set_has_exact_tutoria;
mod assets;
mod input;
mod containers;
mod audio;
mod state;

use types::Fixture;
use operations_compact_document::{
    mission_row, journal_row, compact_document, compact_item_icon_table,
    compact_user_equip_document, mission_rows_mut, npc_rows_mut, warp_rows_mut,
    push_gameplay_warp_row
};
