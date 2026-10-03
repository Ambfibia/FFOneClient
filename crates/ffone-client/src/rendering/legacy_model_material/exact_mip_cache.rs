//! Exact mip-chain keys, level assets and the shared chain cache.

use super::{ExactMipPngBytes, shared_assets};
use bevy::{asset::AssetPath, prelude::*};
use std::collections::HashMap;

/// Runtime proof that every assigned published texture binding was validated
/// before the legacy material replaced the glTF fallback. `baseLevelOnly`
/// bindings are valid and therefore leave both multi-mip counters at zero.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExactMipChainApplied {
    pub assigned_texture_bindings: u32,
    pub mip_chains_applied: u32,
    pub mip_levels: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ExactMipChainKey {
    pub(super) model_source: AssetPath<'static>,
    pub(super) binding_digest: [u8; 32],
}

#[derive(Debug, Clone)]
pub(super) struct ExactMipLevelAssets {
    pub(super) path: AssetPath<'static>,
    pub(super) image: Handle<Image>,
    pub(super) png_bytes: Handle<ExactMipPngBytes>,
}

#[derive(Debug, Clone)]
pub(super) struct ExactMipChainCacheEntry {
    pub(super) levels: Vec<ExactMipLevelAssets>,
    pub(super) assembled: Option<std::sync::Weak<bevy::asset::StrongHandle>>,
    pub(super) source_material: AssetId<StandardMaterial>,
}

#[derive(Resource, Debug, Default)]
pub(super) struct ExactMipChainCache {
    pub(super) entries: HashMap<ExactMipChainKey, ExactMipChainCacheEntry>,
    pub(super) shared_images: shared_assets::SharedAssets<Image>,
    // A material can need several asynchronous chains. Keep already completed
    // slots alive until all slots have been admitted into the final material.
    pub(super) pending_materials: HashMap<AssetId<StandardMaterial>, Vec<Handle<Image>>>,
}
