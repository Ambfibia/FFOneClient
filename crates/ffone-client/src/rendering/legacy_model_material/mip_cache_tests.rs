use crate::legacy_model_material::ExactMipPngBytes;
use crate::legacy_model_material::exact_mip_cache::{
    ExactMipChainCache, ExactMipChainCacheEntry, ExactMipChainKey, ExactMipLevelAssets,
};
use crate::legacy_model_material::exact_mip_chain::complete_exact_mip_entry;
use bevy::{asset::AssetPath, prelude::*};

#[test]
fn completed_mip_cache_releases_staging_and_does_not_pin_the_render_image() {
    let mut images = Assets::<Image>::default();
    let mut pngs = Assets::<ExactMipPngBytes>::default();
    let source = images.add(Image::default());
    let encoded = pngs.add(ExactMipPngBytes {
        bytes: vec![1, 2, 3],
    });
    let Handle::Strong(source_handle) = &source else {
        unreachable!()
    };
    let source_weak = std::sync::Arc::downgrade(source_handle);
    let Handle::Strong(encoded_handle) = &encoded else {
        unreachable!()
    };
    let encoded_weak = std::sync::Arc::downgrade(encoded_handle);
    let key = ExactMipChainKey {
        model_source: AssetPath::from("models/a.glb"),
        binding_digest: [0; 32],
    };
    let mut cache = ExactMipChainCache::default();
    cache.entries.insert(
        key.clone(),
        ExactMipChainCacheEntry {
            levels: vec![ExactMipLevelAssets {
                path: AssetPath::from("textures/a.png"),
                image: source,
                png_bytes: encoded,
            }],
            assembled: None,
            source_material: Handle::<StandardMaterial>::default().id(),
        },
    );
    let output = cache.shared_images.insert(Image::default(), &mut images);
    complete_exact_mip_entry(&mut cache, &key, &output);
    assert!(source_weak.upgrade().is_none());
    assert!(encoded_weak.upgrade().is_none());
    assert!(cache.entries[&key].levels.is_empty());
    assert!(
        cache.entries[&key]
            .assembled
            .as_ref()
            .unwrap()
            .upgrade()
            .is_some()
    );
    drop(output);
    assert!(
        cache.entries[&key]
            .assembled
            .as_ref()
            .unwrap()
            .upgrade()
            .is_none()
    );
}

#[test]
fn assembled_mip_images_share_only_when_all_native_image_fields_match() {
    let mut images = Assets::<Image>::default();
    let mut cache = ExactMipChainCache::default();
    let image = Image::default();
    let first = cache.shared_images.insert(image.clone(), &mut images);
    let second = cache.shared_images.insert(image.clone(), &mut images);
    assert_eq!(first.id(), second.id());
    let mut different = image;
    different.sampler = bevy::image::ImageSampler::nearest();
    let third = cache.shared_images.insert(different, &mut images);
    assert_ne!(first.id(), third.id());
}

#[test]
fn source_unload_cancels_partial_mips_and_releases_completed_slot_leases() {
    use super::{
        LegacyModelMaterial, LegacyStaticMaterialCache,
        shared_assets::maintain_shared_static_assets,
    };
    let mut app = App::new();
    app.init_resource::<LegacyStaticMaterialCache>()
        .init_resource::<ExactMipChainCache>()
        .add_message::<AssetEvent<Image>>()
        .add_message::<AssetEvent<Mesh>>()
        .add_message::<AssetEvent<StandardMaterial>>()
        .add_message::<AssetEvent<LegacyModelMaterial>>()
        .add_systems(Update, maintain_shared_static_assets);
    let mut images = Assets::<Image>::default();
    let mut pngs = Assets::<ExactMipPngBytes>::default();
    let source = Assets::<StandardMaterial>::default()
        .add(StandardMaterial::default())
        .id();
    let complete = images.add(Image::default());
    let partial = images.add(Image::default());
    let encoded = pngs.add(ExactMipPngBytes {
        bytes: vec![1, 2, 3],
    });
    let weak = |handle: &Handle<Image>| match handle {
        Handle::Strong(h) => std::sync::Arc::downgrade(h),
        _ => unreachable!(),
    };
    let complete_weak = weak(&complete);
    let partial_weak = weak(&partial);
    let mut cache = app.world_mut().resource_mut::<ExactMipChainCache>();
    cache.pending_materials.insert(source, vec![complete]);
    cache.entries.insert(
        ExactMipChainKey {
            model_source: AssetPath::from("models/cancelled.glb"),
            binding_digest: [1; 32],
        },
        ExactMipChainCacheEntry {
            source_material: source,
            assembled: None,
            levels: vec![ExactMipLevelAssets {
                path: AssetPath::from("textures/pending.png"),
                image: partial,
                png_bytes: encoded,
            }],
        },
    );
    app.update();
    assert!(
        complete_weak.upgrade().is_some(),
        "completed slots survive while another slot loads"
    );
    assert!(partial_weak.upgrade().is_some());
    app.world_mut()
        .write_message(AssetEvent::<StandardMaterial>::Removed { id: source });
    app.update();
    assert!(complete_weak.upgrade().is_none());
    assert!(partial_weak.upgrade().is_none());
    let cache = app.world().resource::<ExactMipChainCache>();
    assert!(cache.entries.is_empty());
    assert!(cache.pending_materials.is_empty());
}
