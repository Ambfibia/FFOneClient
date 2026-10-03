use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::option_ui::*;

mod assets;
mod operations;
mod layout;
mod frame;
mod containers;
mod localization;
mod interaction;
mod audio;
mod commands;
mod textures;
mod state;
mod validation;

use assets::asset_root;
use operations::cue_timeline;
use layout::assert_sliced_border;
