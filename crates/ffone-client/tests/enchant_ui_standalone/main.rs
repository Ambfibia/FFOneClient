//! Focused clean-Retrobution Enchant contract tests through the production
//! crate export.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use ffone_client::enchant_ui::*;
use ffone_protocol::ItemBase0104;

#[path = "operations.rs"]
mod operations;
#[path = "view.rs"]
mod view;
#[path = "models.rs"]
mod models;
#[path = "layout.rs"]
mod layout;
#[path = "interaction.rs"]
mod interaction;
#[path = "codec.rs"]
mod codec;
#[path = "assets.rs"]
mod assets;
#[path = "state.rs"]
mod state;
#[path = "commands.rs"]
mod commands;
#[path = "validation.rs"]
mod validation;
#[path = "audio.rs"]
mod audio;

use operations::{item, presentation, selectable};
use view::attach_ready_weapon;
use models::awaiting_weapon_model;
