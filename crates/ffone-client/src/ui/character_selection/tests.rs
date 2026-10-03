use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::character_selection_ui::*;

mod operations;
mod assets;
mod localization;
mod layout;
mod containers;
mod state;
mod frame;
mod models;
mod materials;
mod animation;
mod interaction;
mod textures;
mod systems;

use operations::{occupied, only_entity};
use assets::project_asset;
use localization::insert_test_localization;
use layout::assert_node_rect;
