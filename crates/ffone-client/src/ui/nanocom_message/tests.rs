use crate::nanocom_message_ui::*;
use bevy::asset::AssetPlugin;
use std::path::Path;
use tempfile::tempdir;

mod operations;
mod output;
mod layout;
mod containers;
mod localization;
mod assets;
mod audio;
mod interaction;
mod state;
mod systems;
mod commands;

use operations::{passive, buddy, assert_close};
