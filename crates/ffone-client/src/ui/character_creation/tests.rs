use std::path::{Path, PathBuf};

use crate::character_creation_ui::*;
use sha2::{Digest, Sha256};

mod assets;
mod operations;
mod localization;
mod layout;
mod materials;
mod frame;
mod interaction;
mod validation;
mod models;
mod audio;
mod containers;
mod systems;

use assets::project_asset;
use operations::names;
use localization::insert_test_localization;
use audio::creation_audio_test_app;
