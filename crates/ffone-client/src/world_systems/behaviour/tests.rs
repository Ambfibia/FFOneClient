use bevy::camera::visibility::SetViewVisibility;

use crate::world_behaviour::*;

mod materials;
mod operations;
mod state;
mod models;
mod collision;
mod animation;
mod localization;
mod codec;
mod pod_glow;
mod traversal;

use materials::{
    assert_material_animation_upload_contract, linear_material_curve, material_clip
};
use operations::trigger_volume;
