//! Standalone clean-Retrobution UserEquip publication contracts.
//!
//! This harness intentionally reads the shipped files without booting Bevy. It
//! keeps localization publication, semantic-key coverage, and exact primary
//! texture bytes independently auditable from the production UI module.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[path = "constants.rs"]
mod constants;
#[path = "containers.rs"]
mod containers;
#[path = "textures.rs"]
mod textures;
#[path = "projects.rs"]
mod projects;
#[path = "operations.rs"]
mod operations;
#[path = "state.rs"]
mod state;
#[path = "localization.rs"]
mod localization;
#[path = "collision.rs"]
mod collision;
#[path = "assets.rs"]
mod assets;
#[path = "commands.rs"]
mod commands;
#[path = "input.rs"]
mod input;
#[path = "layout.rs"]
mod layout;

use constants::{
    PRIMARY_MAIN_ARCHIVE_BYTES, PRIMARY_MAIN_ARCHIVE_SHA256, PRIMARY_ICONS_ARCHIVE_BYTES,
    PRIMARY_ICONS_ARCHIVE_SHA256, ALTERNATE_ICONS_ARCHIVE_BYTES,
    ALTERNATE_ICONS_ARCHIVE_SHA256, USER_EQUIP_SOURCE, REDEEM_SOURCE, MAIN_SOURCE, APP_SOURCE,
    EXTENDED_NANO_ICONS, PRIMARY_TEXTURES, PRIMARY_NANO_READY_TEXTURES
};
use containers::{TextBundle, open_bundle};
use textures::PrimaryTextureContract;
use projects::workspace_root;
use operations::{semantic_primary_nano_slug, placeholders, sha256_lower};
use state::inventory_keys_in_source;
