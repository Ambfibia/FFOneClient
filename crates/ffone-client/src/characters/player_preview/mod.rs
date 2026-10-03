//! Native modular player preview used by character selection and creation.
//!
//! Every part is an audited GLB under `assets/game/characters/player`. The
//! preview composes the independently skinned source parts at their unchanged
//! authored origin. No Unity object, bundle, runtime, or fallback NPC is used.

use bevy::{
    animation::RepeatAnimation,
    asset::{LoadState, RecursiveDependencyLoadState},
    camera::{RenderTarget, Viewport, visibility::RenderLayers},
    color::LinearRgba,
    prelude::*,
    render::render_resource::TextureFormat,
    window::PrimaryWindow,
    window::WindowRef,
};
use ffone_runtime_contracts::{CharacterRuntimeTextureContract, PlayerRigGender};
use ffone_skinned_model::{PublishedMipPolicy, TextureColorSpace};

use crate::{
    attachment::{
        LegacyPlayerAttachmentSlot, player_attachment_socket_full_path,
        standard_player_attachment_placement,
    },
    character_creation_ui::{
        CHARACTER_CREATION_APPEARANCE_WIDTH, CharacterCreationLayout, CharacterCreationUiModel,
    },
    character_scene::{NativeSceneRole, native_scene_container_transform},
    character_selection_ui::{
        CHARACTER_SELECTION_PREVIEW_HEIGHT, CHARACTER_SELECTION_PREVIEW_WIDTH,
        CharacterSelectionLayout, CharacterSelectionUiModel,
    },
    gameplay_ui::GAMEPLAY_UI_CAMERA_ORDER,
    legacy_model_material::{
        LegacyMaterialApplied, LegacyMaterialMetadataError, LegacyMaterialPassCompanion,
        LegacyModelMaterial, PendingLegacyModelMaterial,
        load_character_runtime_texture_with_contract,
    },
    player_appearance_material::{
        ActorSkinTextureRole, NativePlayerMaterialBinding, bind_native_player_look_material,
    },
    player_shared_rig::{
        NativePlayerBodyShape, NativePlayerBodyShapePlayback, NativePlayerRigAssetCache,
        NativePlayerRigBones, NativePlayerRigCatalog, NativePlayerRigSpawnRequest,
        NativePlayerRigStand1Playback, NativePlayerRigStatus, spawn_native_player_rig,
    },
    tutorial_player_presentation::PlayerWeaponAnimationProfile,
};

#[cfg(test)]
use crate::player_appearance_material::{
    actor_skin_texture_role, half_tint, multiply_rgb, uses_global_skin_secondary,
};

use crate::scene_hierarchy::is_descendant_of;

#[cfg(test)]
mod tests;

mod materials;
mod constants;
mod state;
mod types;
mod textures;
mod models;
mod validation;
mod operations;
mod animation_recover_native_player_preview_id;
mod systems;
mod inventory_target;

pub use materials::NATIVE_PLAYER_PREVIEW_RENDER_LAYER;
use materials::{
    NativePlayerPreviewMaterialBound, NativePlayerPreviewMaterialBaseline,
    bind_native_player_preview_render_layers
};
pub use constants::{
    NATIVE_PLAYER_PREVIEW_CAMERA_ORDER, NATIVE_PLAYER_CREATION_CAMERA_DISTANCE,
    NATIVE_PLAYER_CREATION_CAMERA_HEIGHT, NATIVE_PLAYER_CREATION_CAMERA_MIN_DISTANCE,
    NATIVE_PLAYER_CREATION_CAMERA_MAX_DISTANCE
};
pub use state::{
    NATIVE_PLAYER_INVENTORY_PREVIEW_CAMERA_ORDER, NATIVE_PLAYER_SELECTION_CAMERA_DISTANCE,
    NATIVE_PLAYER_SELECTION_CAMERA_HEIGHT, NATIVE_PLAYER_INVENTORY_CAMERA_DISTANCE,
    NATIVE_PLAYER_INVENTORY_CAMERA_HEIGHT, NATIVE_PLAYER_INVENTORY_PREVIEW_WIDTH,
    NATIVE_PLAYER_INVENTORY_PREVIEW_HEIGHT, NativePlayerPreviewStatus,
    NativePlayerInventoryPreviewImage
};
use state::{NativePlayerPreviewRuntime, update_native_player_preview_status};
pub use types::{
    NativePlayerPreviewStage, NativePlayerPartKind, NativePlayerPartAssembly,
    NativePlayerPartLook, NativePlayerLook, NativePlayerPreviewSet, NativePlayerPreviewCamera,
    NativePlayerPreviewPlugin, NativePlayerTryOnPreviewImage, NativePlayerBarberPreviewImage
};
use types::{
    NativePlayerPreviewRoot, NativePlayerPreviewPart, PendingNativePlayerPreviewAttachment,
    NativePlayerPreviewLight, NativePlayerPreviewGeometry, NativePlayerBindingDiagnostics
};
pub use textures::NativePlayerTexture;
use textures::validate_player_texture;
pub use models::NativePlayerPreviewModel;
pub use validation::{NativePlayerPreviewAudit, NativePlayerPreviewSurfaceAudit};
use validation::{validate_asset_path, validate_exact_route, validate_palette_color};
pub use operations::prewarm_native_player_look;
use operations::{
    setup_native_player_preview_stage, rebuild_native_player_preview,
    bind_native_player_preview_attachments, bind_native_player_preview_materials,
    native_player_preview_camera_distance_and_height, native_player_preview_viewport, ancestor_part
};
#[cfg(test)]
use operations::superseded_pending_root;
use animation_recover_native_player_preview_id::recover_native_player_preview_idle_animation;
use systems::sync_native_player_preview_transform;
