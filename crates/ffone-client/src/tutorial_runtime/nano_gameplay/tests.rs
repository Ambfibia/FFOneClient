use crate::tutorial_nano_gameplay::*;
use bevy::asset::{AssetApp, AssetPlugin};

use crate::legacy_model_material::{
    LegacyModelMaterialParams, LegacyModelTextures, LegacyShaderKind,
};

mod operations;
mod assets;
mod models;
mod materials;
mod commands;
mod systems;
mod state;
mod animation;
mod validation;
mod audio;

use operations::{mechanics_app, demo_actor, activate_for_test, target};
use assets::asset_app;
use models::gltf_with_gameplay_clips;
