use crate::pc2pc_ui::*;
use crate::user_equip_ui::UserEquipIconRef;
use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

mod constants;
mod operations;
mod state;
mod assets;
mod types;
mod models;
mod layout;
mod commands;
mod interaction;
mod output;
mod validation;
mod codec;
mod audio;

use constants::{LOCAL_ID, REMOTE_ID};
use operations::{item, identity, local_envelope, remote_envelope, capabilities_with_popup};
use state::runtime;
use assets::Catalog;
use types::AllowEquip;
use models::model_with;
use codec::offer_reply_frame;
