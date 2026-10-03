use bevy::text::LineHeight;

use crate::combi_ui::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

mod constants;
mod assets;
mod input;
mod operations;
mod layout;
mod view;
mod state;
mod commands;
mod frame;
mod output;
mod localization;
mod materials;
mod interaction;
mod textures;

use constants::CLEAN_TABLE_SET;
use assets::FixtureCatalog;
use operations::{item, metadata, fixture, awaiting_machine, carried};
use frame::pointer_frame;
use interaction::pointer_app;
use textures::png_dimensions;
