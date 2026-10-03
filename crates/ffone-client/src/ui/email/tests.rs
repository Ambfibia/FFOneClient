use crate::email_ui::*;
use bevy::asset::AssetPlugin;

mod operations;
mod models;
mod layout;
mod codec;
mod audio;
mod interaction;
mod state;
mod localization;
mod output;
mod validation;

use operations::summary;
use models::opened_player_model;
