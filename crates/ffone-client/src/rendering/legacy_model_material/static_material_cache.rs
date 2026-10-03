//! Admission cache for immutable static-world materials.

use super::exact_mip_cache::ExactMipChainApplied;
use super::render_plan::LegacyPassKind;
use super::shared_assets::SharedStaticAssets;
use super::{LegacyModelMaterial, shared_assets};
use bevy::{asset::AssetPath, prelude::*};
use std::collections::HashMap;

/// Static world GLBs are instantiated many times inside one streamed tile.
/// Their glTF material handles and authored FFOne material metadata are
/// identical, so keep one native pass set per source glTF material while that
/// source asset is live. The weak content cache also shares exact resources
/// across different model paths without merging their placement/behavior owners.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct LegacyStaticMaterialCacheKey {
    pub(super) model_source: AssetPath<'static>,
    pub(super) standard_material: AssetId<StandardMaterial>,
}

#[derive(Debug, Clone)]
pub(super) struct CachedLegacyStaticMaterial {
    pub(super) passes: Vec<(LegacyPassKind, Handle<LegacyModelMaterial>)>,
    pub(super) mip_proof: ExactMipChainApplied,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct LegacyStaticMaterialCache {
    pub(super) entries: HashMap<LegacyStaticMaterialCacheKey, CachedLegacyStaticMaterial>,
    pub(super) shared: SharedStaticAssets,
}

impl LegacyStaticMaterialCache {
    pub(crate) fn admit_immutable(
        &mut self,
        material: LegacyModelMaterial,
        assets: &mut Assets<LegacyModelMaterial>,
    ) -> Handle<LegacyModelMaterial> {
        let handle = self.shared.materials.insert(material, assets);
        if let Some(material) = assets.get_mut_untracked(&handle) {
            material.sharing_state = shared_assets::MaterialSharingState::Shared;
        }
        handle
    }
}
