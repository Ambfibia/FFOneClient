use std::time::Duration;

use bevy::time::TimeUpdateStrategy;

use crate::legacy_model_material::{
    LegacyModelMaterialParams, LegacyModelTextures, LegacyShaderKind,
};
use crate::network_world_runtime::*;

mod operations_shared_npc_appear_effect_waits_for_xdt_bindi;
mod operations_hnpc_player_rigs_use_the_same_weapon_profile;
mod textures;
mod animation;
mod assets;
mod state;
mod collision;
mod models;
mod codec;
mod audio;

use operations_shared_npc_appear_effect_waits_for_xdt_bindi::{table_set, remote_pc_test_look};
use textures::texture_catalog;
use assets::registry;
