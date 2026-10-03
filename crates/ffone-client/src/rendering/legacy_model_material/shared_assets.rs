//! Content sharing for immutable static render assets, with no scene/entity merging.
//!
//! Hashes select candidates only. Full Bevy value equality is the authority,
//! including sampler, color space, mip layout, vertex attributes and render state.
//! Weak handles let streamed assets unload; animations must clone before writing.

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Weak},
};

use bevy::{
    asset::{AssetEvent, StrongHandle},
    prelude::*,
};

use super::{LegacyModelMaterial, LegacyStaticMaterialCache, image_is_fully_opaque};

// Allocation ownership is not part of native material value equality. Cloning
// a value creates a private candidate; only explicit admission makes it shared.
#[derive(Debug, Default)]
pub(super) enum MaterialSharingState {
    #[default]
    Unadmitted,
    Shared,
    Private,
}

impl Clone for MaterialSharingState {
    fn clone(&self) -> Self {
        Self::Private
    }
}

#[derive(Resource)]
pub(crate) struct ModelMaterialSharing(pub bool);

impl Default for ModelMaterialSharing {
    fn default() -> Self {
        Self(
            !(std::env::var_os("FFONE_PERF_MODEL_MATERIAL_BASELINE").is_some()
                && (std::env::var_os("FFONE_PERF_OUTPUT").is_some()
                    || std::env::var_os("FFONE_PERF_ASSET_AUDIT").is_some())),
        )
    }
}

impl PartialEq for MaterialSharingState {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Detach only materials admitted to a shared immutable cache. Bind immediately
/// through a mutable ECS handle, so later writers in this update see the copy.
pub fn make_legacy_material_unique(
    handle: &mut Handle<LegacyModelMaterial>,
    materials: &mut Assets<LegacyModelMaterial>,
) {
    let Some(material) = materials.get(&*handle) else {
        return;
    };
    match material.sharing_state {
        MaterialSharingState::Shared => {
            let private = material.clone();
            *handle = materials.add(private);
        }
        MaterialSharingState::Unadmitted => {
            // A writer can run before final render-order admission. Keep that
            // material private so a later admission cannot replace its binding.
            materials.get_mut_untracked(&*handle).unwrap().sharing_state =
                MaterialSharingState::Private;
        }
        MaterialSharingState::Private => {}
    }
}

/// Diagnostic A/B switch. Production always defaults to sharing; this never
/// changes visibility, geometry, texture quality or streaming distances.
#[derive(Resource, Debug, Clone, Copy)]
pub struct StaticAssetSharing {
    pub enabled: bool,
}

impl Default for StaticAssetSharing {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub struct StaticAssetSharingStatistics {
    pub meshes_hashed: usize,
    pub mesh_reuses: usize,
    pub images_hashed: usize,
    pub image_reuses: usize,
    pub material_reuses: usize,
    pub opacity_scans: usize,
}

pub fn static_asset_sharing_statistics(world: &World) -> StaticAssetSharingStatistics {
    let Some(cache) = world.get_resource::<LegacyStaticMaterialCache>() else {
        return Default::default();
    };
    StaticAssetSharingStatistics {
        meshes_hashed: cache.shared.meshes.hashed,
        mesh_reuses: cache.shared.meshes.hits,
        images_hashed: cache.shared.images.hashed,
        image_reuses: cache.shared.images.hits,
        material_reuses: cache.shared.materials.hits,
        opacity_scans: cache.shared.opacity_scans,
    }
}

pub(super) trait SharedValue: Asset + PartialEq {
    fn fingerprint(&self) -> [u8; 32];
    fn shareable(&self) -> bool {
        true
    }
}

impl SharedValue for Image {
    fn fingerprint(&self) -> [u8; 32] {
        *blake3::hash(self.data.as_deref().unwrap_or_default()).as_bytes()
    }

    fn shareable(&self) -> bool {
        use bevy::render::render_resource::TextureUsages;
        self.data.is_some()
            && !self
                .texture_descriptor
                .usage
                .intersects(TextureUsages::RENDER_ATTACHMENT | TextureUsages::STORAGE_BINDING)
    }
}

impl SharedValue for Mesh {
    fn fingerprint(&self) -> [u8; 32] {
        let mut hash = blake3::Hasher::new();
        for (_, values) in self.attributes() {
            hash.update(values.get_bytes());
        }
        hash.update(self.get_index_buffer_bytes().unwrap_or_default());
        *hash.finalize().as_bytes()
    }
}

impl SharedValue for LegacyModelMaterial {
    fn fingerprint(&self) -> [u8; 32] {
        // Uniform variants may share a bucket, but never bypass full equality.
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.render_mode.hash(&mut hash);
        self.gpu_uv_animation.hash(&mut hash);
        for texture in [
            &self.base_texture,
            &self.toon_ramp,
            &self.bump_texture,
            &self.effect_map,
        ] {
            texture.as_ref().map(Handle::id).hash(&mut hash);
        }
        let mut digest = [0; 32];
        digest[..8].copy_from_slice(&hash.finish().to_le_bytes());
        digest
    }
}

#[derive(Debug)]
pub(super) struct SharedAssets<A: SharedValue> {
    candidates: HashMap<[u8; 32], Vec<Weak<StrongHandle>>>,
    fingerprints: HashMap<AssetId<A>, (Weak<StrongHandle>, [u8; 32])>,
    pub hits: usize,
    pub hashed: usize,
}

impl<A: SharedValue> Default for SharedAssets<A> {
    fn default() -> Self {
        Self {
            candidates: HashMap::new(),
            fingerprints: HashMap::new(),
            hits: 0,
            hashed: 0,
        }
    }
}

impl<A: SharedValue> SharedAssets<A> {
    pub fn intern(&mut self, source: &Handle<A>, assets: &Assets<A>) -> Handle<A> {
        let Handle::Strong(strong) = source else {
            return source.clone();
        };
        let Some(value) = assets.get(source) else {
            return source.clone();
        };
        if !value.shareable() {
            return source.clone();
        }
        let fingerprint = if let Some((_, fingerprint)) = self.fingerprints.get(&source.id()) {
            *fingerprint
        } else {
            let fingerprint = value.fingerprint();
            self.hashed += 1;
            // Admission-time housekeeping only; no full-catalog walk in idle frames.
            if self.hashed % 256 == 0 {
                self.prune();
            }
            self.fingerprints
                .insert(source.id(), (Arc::downgrade(strong), fingerprint));
            fingerprint
        };
        let candidates = self.candidates.entry(fingerprint).or_default();
        candidates.retain(|candidate| candidate.strong_count() > 0);
        for candidate in candidates.iter().filter_map(Weak::upgrade) {
            let handle = Handle::Strong(candidate);
            if handle.id() == source.id() {
                return source.clone();
            }
            if assets
                .get(&handle)
                .is_some_and(|candidate| candidate == value)
            {
                self.hits += 1;
                return handle;
            }
        }
        candidates.push(Arc::downgrade(strong));
        source.clone()
    }

    pub fn insert(&mut self, value: A, assets: &mut Assets<A>) -> Handle<A> {
        if !value.shareable() {
            return assets.add(value);
        }
        let fingerprint = value.fingerprint();
        if let Some(candidates) = self.candidates.get(&fingerprint) {
            for candidate in candidates.iter().filter_map(Weak::upgrade) {
                let handle = Handle::Strong(candidate);
                if assets
                    .get(&handle)
                    .is_some_and(|candidate| candidate == &value)
                {
                    self.hits += 1;
                    return handle;
                }
            }
        }
        let handle = assets.add(value);
        self.intern(&handle, assets)
    }

    fn invalidate(&mut self, id: AssetId<A>) {
        // Equality still verifies candidates after edits. Rehash when next admitted.
        self.fingerprints.remove(&id);
    }

    fn prune(&mut self) {
        self.fingerprints
            .retain(|_, (handle, _)| handle.strong_count() > 0);
        self.candidates.retain(|_, handles| {
            handles.retain(|handle| handle.strong_count() > 0);
            !handles.is_empty()
        });
    }
}

#[derive(Debug, Default)]
pub(super) struct SharedStaticAssets {
    pub meshes: SharedAssets<Mesh>,
    pub images: SharedAssets<Image>,
    pub materials: SharedAssets<LegacyModelMaterial>,
    opacity: HashMap<AssetId<Image>, (Weak<StrongHandle>, bool)>,
    opacity_scans: usize,
}

impl SharedStaticAssets {
    pub fn opaque(&mut self, handle: &Handle<Image>, image: &Image) -> bool {
        if let Some((_, opaque)) = self.opacity.get(&handle.id()) {
            return *opaque;
        }
        self.opacity_scans += 1;
        let opaque = image_is_fully_opaque(image).unwrap_or(false);
        if let Handle::Strong(strong) = handle {
            if self.opacity.len() % 256 == 0 {
                self.opacity
                    .retain(|_, (handle, _)| handle.strong_count() > 0);
            }
            self.opacity
                .insert(handle.id(), (Arc::downgrade(strong), opaque));
        }
        opaque
    }
}

pub(super) fn maintain_shared_static_assets(
    mut cache: ResMut<LegacyStaticMaterialCache>,
    mut mip_cache: Option<ResMut<super::ExactMipChainCache>>,
    mut images: MessageReader<AssetEvent<Image>>,
    mut meshes: MessageReader<AssetEvent<Mesh>>,
    mut materials: MessageReader<AssetEvent<LegacyModelMaterial>>,
    mut sources: MessageReader<AssetEvent<StandardMaterial>>,
) {
    for event in images.read() {
        if let AssetEvent::Modified { id } | AssetEvent::Removed { id } = event {
            if let Some(mip_cache) = mip_cache.as_mut() {
                mip_cache.shared_images.invalidate(*id);
            }
            cache.shared.images.invalidate(*id);
            cache.shared.opacity.remove(id);
        }
    }
    for event in meshes.read() {
        if let AssetEvent::Modified { id } | AssetEvent::Removed { id } = event {
            cache.shared.meshes.invalidate(*id);
        }
    }
    for event in materials.read() {
        if let AssetEvent::Modified { id } | AssetEvent::Removed { id } = event {
            cache.shared.materials.invalidate(*id);
        }
    }
    let retired: std::collections::HashSet<_> = sources
        .read()
        .filter_map(|event| match event {
            AssetEvent::Removed { id } | AssetEvent::Modified { id } => Some(*id),
            _ => None,
        })
        .collect();
    if !retired.is_empty() {
        if let Some(mip_cache) = mip_cache.as_mut() {
            for id in &retired {
                mip_cache.pending_materials.remove(id);
            }
            // Cancelled/failed asynchronous admissions also own staging images.
            // Retire them when their source material unloads, not in an idle scan.
            mip_cache
                .entries
                .retain(|_, entry| !retired.contains(&entry.source_material));
        }
        cache
            .entries
            .retain(|key, _| !retired.contains(&key.standard_material));
    }
}

#[cfg(test)]
mod tests;
