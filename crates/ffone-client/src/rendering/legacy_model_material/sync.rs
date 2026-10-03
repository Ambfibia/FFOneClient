//! Per-frame sky rimlight, anisotropy, outline visibility and culling synchronisation.

use super::LegacyModelMaterial;
use super::material_uniforms::LegacyOutlineCompanion;
use super::samplers::{
    LEGACY_TEXTURE_ANISOTROPY, LEGACY_TEXTURE_ANISOTROPY_DISABLED,
    LEGACY_TEXTURE_ANISOTROPY_ENABLED, apply_native_anisotropy, native_sampler_baseline,
};
use crate::{movement::LegacyOrbitCamera, option_ui::OptionUiModel, world::NativeWorldCatalog};
use bevy::{
    camera::visibility::NoFrustumCulling, image::ImageSampler, mesh::skinning::SkinnedMesh,
    prelude::*,
};
use std::sync::atomic::Ordering;

pub(super) fn sync_legacy_sky_rimlight(
    catalog: Option<Res<NativeWorldCatalog>>,
    cameras: Query<&GlobalTransform, With<LegacyOrbitCamera>>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
) {
    let Some(catalog) = catalog else {
        return;
    };
    let Ok(camera) = cameras.single() else {
        return;
    };
    let sampled = catalog
        .sample_ambience(camera.translation(), false)
        .sky_color;
    let sky_color = LinearRgba::new(sampled[0], sampled[1], sampled[2], sampled[3]);
    sync_legacy_sky_rimlight_assets(&mut materials, sky_color);
}

pub(super) fn sync_legacy_sky_rimlight_assets(
    materials: &mut Assets<LegacyModelMaterial>,
    sky_color: LinearRgba,
) -> usize {
    // `Assets::iter_mut` queues a Modified event as soon as each item is
    // yielded, even when its value is left untouched. Walking the complete
    // material catalog that way made Bevy re-upload every world material and
    // rebuild all of their bind groups on every frame. Discover the genuinely
    // stale assets through immutable access, then emit changes only for them.
    let stale_materials = materials
        .iter()
        .filter_map(|(id, material)| {
            (material.uniform.rim_effect.z > 0.5
                && material.uniform.rim_effect.z < 1.5
                && material.uniform.rim_color != sky_color)
                .then_some(id)
        })
        .collect::<Vec<_>>();
    let changed = stale_materials.len();
    for id in stale_materials {
        if let Some(mut material) = materials.get_mut(id) {
            material.uniform.rim_color = sky_color;
        }
    }
    changed
}

/// Publishes the committed `ANISOTROPIC FILTERING` option to the legacy
/// sampler builder and re-stamps textures that are already resident.
///
/// Bevy bakes the sampler into the `Image` asset, so a live toggle only reaches
/// textures loaded afterwards unless the resident ones are updated here.
pub(super) fn sync_legacy_texture_anisotropy(
    options: Option<Res<OptionUiModel>>,
    mut images: ResMut<Assets<Image>>,
) {
    // A runtime without OptionUiModel (GPU acceptance/editor tooling) keeps the
    // filtered default rather than inventing a persisted setting.
    let desired = options
        .as_deref()
        .map_or(LEGACY_TEXTURE_ANISOTROPY_ENABLED, |options| {
            if options.persisted_options.graphics.anisotropic_filtering {
                LEGACY_TEXTURE_ANISOTROPY_ENABLED
            } else {
                LEGACY_TEXTURE_ANISOTROPY_DISABLED
            }
        });
    let previous = LEGACY_TEXTURE_ANISOTROPY.swap(desired, Ordering::Relaxed);
    if previous == desired {
        return;
    }
    // Only native model samplers carry their reversible baseline. UI images and
    // render targets must not be admitted merely because their filters match.
    let stale = images.iter().filter_map(|(id, image)| {
        let ImageSampler::Descriptor(descriptor) = &image.sampler else { return None; };
        native_sampler_baseline(descriptor).map(|_| id)
    }).collect::<Vec<_>>();
    for id in stale {
        if let Some(mut image) = images.get_mut(id)
            && let ImageSampler::Descriptor(descriptor) = &mut image.sampler {
            apply_native_anisotropy(descriptor, desired);
        }
    }
}

pub(super) fn legacy_outline_visibility(enabled: bool) -> Visibility {
    if enabled {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

pub(super) fn sync_legacy_outline_visibility(
    options: Option<Res<OptionUiModel>>,
    mut outlines: Query<&mut Visibility, With<LegacyOutlineCompanion>>,
) {
    // A runtime without OptionUiModel (GPU acceptance/editor tooling) owns
    // its explicit companion visibility. Production always provides the
    // option model and still applies the live Toon Outline setting here.
    let Some(options) = options.as_deref() else {
        return;
    };
    let enabled = options.persisted_options.graphics.toon_shading;
    let desired = legacy_outline_visibility(enabled);
    for mut visibility in &mut outlines {
        if *visibility != desired {
            *visibility = desired;
        }
    }
}

/// glTF supplies only the undeformed mesh AABB. FusionFall equipment can move
/// completely outside that box after its joint palette and inverse bind poses
/// are evaluated, so Bevy's ordinary static-AABB frustum test is not valid for
/// these entities. Keep the exact authored skin and camera; opt the skinned
/// surface out until the runtime has a proven per-frame deformed-AABB updater.
pub(super) fn disable_static_frustum_culling_for_skinned_meshes(
    mut commands: Commands,
    query: Query<Entity, (With<SkinnedMesh>, Without<NoFrustumCulling>)>,
) {
    for entity in &query {
        try_insert_streamed_entity(&mut commands, entity, NoFrustumCulling);
    }
}

/// Scene streaming may despawn a glTF entity after a discovery query has read
/// it but before Bevy applies this system's deferred commands. Treat that as a
/// normal cancelled conversion instead of escalating the stale entity ID into
/// a command-buffer panic.
pub(super) fn try_insert_streamed_entity(commands: &mut Commands, entity: Entity, bundle: impl Bundle) {
    commands.entity(entity).try_insert(bundle);
}
