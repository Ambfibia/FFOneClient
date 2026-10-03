use crate::bank_ui::*;
use bevy::asset::AssetPlugin;
use ffone_protocol::WirePayload;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
use tempfile::tempdir;

mod constants;
mod operations;
mod output;
mod state;
mod assets;
mod types;
mod layout;
mod interaction;
mod validation;
mod localization;

use constants::{OWNER_PC_ID, NPC_ID};
use operations::{item, empty, open_with, projection};
use output::write_item;
use state::runtime_with;
use assets::{AllCatalog, MissingCatalog};
use types::AllowEquip;
use interaction::{pointer_test_app, pointer_edge};
