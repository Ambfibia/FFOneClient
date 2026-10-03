use crate::vendor_ui::*;
use bevy::asset::AssetPlugin;
use ffone_protocol::PcLoadData0104;
use std::path::Path;
use tempfile::tempdir;

mod constants;
mod operations;
mod output;
mod state;
mod projection;
mod assets;
mod types;
mod commands;
mod layout;
mod interaction;
mod validation;
mod audio;

use constants::{OWNER_PC_ID, NPC_ID, TABLE_VENDOR_ID};
use operations::{item, empty, projection};
use output::write_item;
use state::runtime_with;
use projection::session;
use assets::TestCatalog;
use types::TestEligibility;
