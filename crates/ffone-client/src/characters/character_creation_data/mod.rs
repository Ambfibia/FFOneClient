//! Verified native character-creation data used by the Bevy UI and preview.
//!
//! The loader accepts only the published character-creation root contracts and
//! verifies every referenced model/texture directly from their owner hashes.
//! Unity bundles and `.ffclient` projects are deliberately not runtime inputs.

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::{color::LinearRgba, prelude::Resource};
use ffone_protocol::{
    CharacterEquipSlot0104, EquippedItem0104, FixedUtf16, OnItem0104, OnItemIndex0104,
    PcAppearance0104, PcStyle0104,
};
use ffone_runtime_contracts::{
    AvatarItemCategory, AvatarItemLookup, AvatarItemVisual, AvatarModelReference,
    AvatarTextureReference, CHARACTER_CREATION_APPEARANCE_PATH,
    CHARACTER_CREATION_APPEARANCE_SCHEMA, CHARACTER_CREATION_AVATAR_ITEMS_PATH,
    CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA, CHARACTER_CREATION_NAME_WHEEL_PATH,
    CHARACTER_CREATION_NAME_WHEEL_SCHEMA, CHARACTER_CREATION_RUNTIME_TEXTURES_PATH,
    CHARACTER_CREATION_RUNTIME_TEXTURES_SCHEMA, CharacterAppearanceCategory,
    CharacterCreationAppearance, CharacterCreationAssetReference, CharacterCreationAvatarItems,
    CharacterCreationChoice, CharacterCreationNameWheel, CharacterCreationRuntimeTextures,
    CharacterGender as DataGender, CharacterPaletteColor, CharacterRuntimeTextureContract,
    NativeLookupStatus, PlayerRigGender,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    assets::AssetLocator,
    character_creation_ui::{
        AppearanceField, CharacterAppearance, CharacterCreationOptionCounts,
        CharacterGender as UiGender, CharacterNameLists, GeneratedCharacterName,
    },
    network::CharacterSummary,
    player_preview::{
        NativePlayerLook, NativePlayerPartAssembly, NativePlayerPartKind, NativePlayerPartLook,
        NativePlayerTexture,
    },
    player_shared_rig::NativePlayerRigCatalog,
    tutorial_player_presentation::PlayerWeaponAnimationProfile,
};

#[cfg(test)]
mod tests;

mod constants;
mod assets_normalize_relative_path;
mod validation;
mod types;
mod state;
mod textures;
mod input;
mod collision;
mod operations;
mod animation;

use constants::PROTOCOL_0104;
use assets_normalize_relative_path::{
    PLAYER_ITEM_SET_CATALOG_SCHEMA, PLAYER_ITEM_SET_CATALOG_PATH, PlayerItemRouteCatalog,
    NativeAssetKind, normalize_relative_path
};
pub use validation::CharacterCreationDataError;
use validation::validate_documents;
pub use types::{
    CharacterCreationDataResult, CharacterCreationData, CharacterCreationDataResource
};
use types::LegacyHatPolicy;
pub use state::ResolvedCreatorSelection;
use textures::{
    PartTextureRule, compatible_shared_texture_contract, validate_runtime_texture_contract
};
use input::read_json;
use collision::strip_native_collision_suffix;
use operations::{
    data_gender, protocol_gender, positive_style, palette_color, compose_legacy_last_name,
    to_u8, to_i8, to_i16, invalid
};
use animation::player_rig_gender;
