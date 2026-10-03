//!
//! The legacy client rendered one independent player instance per occupied
//! account slot, targeted each camera at the instance-local `Bip01 Head`, and
//! copied those cameras into the four exact IMGUI rectangles. This module
//! reproduces that ownership model with native Bevy scenes and readable GLB /
//! PNG assets only.

use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState},
    camera::{Viewport, visibility::RenderLayers},
    color::LinearRgba,
    prelude::*,
    render::render_resource::TextureFormat,
    window::PrimaryWindow,
};

use crate::{
    attachment::{
        LegacyPlayerAttachmentSlot, player_attachment_socket_full_path,
        standard_player_attachment_placement,
    },
    character_scene::{NativeSceneRole, native_scene_container_transform},
    character_selection_ui::{
        CharacterSelectionLayout, CharacterSelectionUiModel, LegacySelectionRect,
    },
    coordinates::unity_to_native_vector,
    gameplay_ui::GameplayPortraitImage,
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyMaterialPassCompanion, LegacyModelMaterial,
        PendingLegacyModelMaterial, load_legacy_main_texture_replacement_with_contract,
    },
    player_preview::{NATIVE_PLAYER_PREVIEW_CAMERA_ORDER, NativePlayerLook, NativePlayerPartKind},
    player_shared_rig::{
        NativePlayerBodyShape, NativePlayerRigAssetCache, NativePlayerRigBones,
        NativePlayerRigCatalog, NativePlayerRigSpawnRequest, NativePlayerRigStatus,
        spawn_native_player_rig,
    },
};

#[cfg(test)]
mod tests;

mod state_update_character_selection_portrait_status;
mod state_sync_character_selection_portrait_cameras;
mod animation;
mod materials;
mod textures;
mod constants;
mod models;
mod types;
mod operations;

pub use state_update_character_selection_portrait_status::{
    CHARACTER_SELECTION_PORTRAIT_COUNT, CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE,
    CHARACTER_SELECTION_PORTRAIT_X, CHARACTER_SELECTION_PORTRAIT_WIDTH,
    CHARACTER_SELECTION_PORTRAIT_HEIGHT, CHARACTER_SELECTION_PORTRAIT_Y,
    CHARACTER_SELECTION_PORTRAIT_CAMERA_DISTANCE, CHARACTER_SELECTION_PORTRAIT_CAMERA_HEIGHT,
    CHARACTER_SELECTION_PORTRAIT_CAMERA_FOV_DEGREES, CHARACTER_SELECTION_PORTRAIT_CAMERA_NEAR,
    CHARACTER_SELECTION_PORTRAIT_CAMERA_FAR, CharacterSelectionPortraitStatus,
    CharacterSelectionPortraitSlot, CharacterSelectionPortraitLight,
    NativeCharacterSelectionPortraitsPlugin, CharacterSelectionPortraitsSet
};
use state_update_character_selection_portrait_status::{
    CharacterSelectionPortraitCamera, CharacterSelectionPortraitsRuntime
};
#[cfg(test)]
use state_update_character_selection_portrait_status::{
    CharacterSelectionPortraitPart, PendingCharacterSelectionPortraitAttachment,
    bind_character_selection_portrait_attachments
};
use state_sync_character_selection_portrait_cameras::sync_character_selection_portrait_cameras;
pub use state_sync_character_selection_portrait_cameras::character_selection_portrait_rect;
pub use animation::{CHARACTER_SELECTION_PORTRAIT_HEAD_BONE, GAMEPLAY_PLAYER_PORTRAIT_NECK_BONE};
use animation::CharacterSelectionPortraitRig;
pub use materials::CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS;
use materials::{
    CharacterSelectionPortraitMaterialBound, sync_character_selection_portrait_render_layers
};
pub use textures::{
    GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_WIDTH, GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_HEIGHT
};
use textures::{remember_texture, ActorSkinTextureRole, actor_skin_texture_role};
pub use constants::{
    GAMEPLAY_PLAYER_PORTRAIT_CAMERA_DISTANCE, GAMEPLAY_PLAYER_PORTRAIT_CAMERA_HEIGHT,
    GAMEPLAY_PLAYER_PORTRAIT_CAMERA_YAW_DEGREES, GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FOV_DEGREES,
    GAMEPLAY_PLAYER_PORTRAIT_CAMERA_NEAR, GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FAR
};
pub use models::{CharacterSelectionPortraitsModel, GameplayPlayerPortraitModel};
use types::GameplayPlayerPortraitCamera;
use operations::{
    setup_gameplay_player_portrait_camera, visit_portrait_hierarchy,
    gameplay_player_portrait_camera_transform, uses_global_skin_secondary, half_tint,
    multiply_rgb
};
