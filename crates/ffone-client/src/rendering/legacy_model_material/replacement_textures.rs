//! NPC/character replacement textures and their runtime contracts.

use super::metadata::PendingLegacyModelMaterial;
use super::params::LegacyGltfTextureBinding;
use super::samplers::exact_sampler_descriptor;
use super::shader_kind::LegacyShaderKind;
use bevy::{
    asset::AssetPath,
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use ffone_runtime_contracts::CharacterRuntimeTextureContract;
use ffone_skinned_model::{NativeSampler, PublishedMipPolicy, TextureColorSpace};
use serde_json::Value;

/// Table texture role selected for one legacy NPC material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyNpcTextureRole {
    Main,
    Sub,
}

/// Recovers `NpcMoveController.SetupNPC`'s `main`/`sub` material assignment,
/// including the malformed clean-client Fusion Flapjack material whose true
/// name is `back_fusiontentacles-main-link_a.dds` even though its exact shader
/// and XDT Texture2 slot identify it as the Fusion overlay.
///
/// Exact `sub` names retain first priority. The shader fallback is deliberately
/// narrow: it applies only when Texture2 exists and the material is the typed
/// `Skin_FusionEffect_blendSrcalphaInvsrcalpha` family. Ordinary `main` names
/// and every non-Fusion material keep the Unity case-sensitive branch.
pub fn legacy_npc_texture_role(
    true_name: &str,
    shader: LegacyShaderKind,
    has_main: bool,
    has_sub: bool,
) -> Option<LegacyNpcTextureRole> {
    if has_sub && true_name.contains("sub") {
        Some(LegacyNpcTextureRole::Sub)
    } else if has_sub && shader == LegacyShaderKind::FusionEffect {
        Some(LegacyNpcTextureRole::Sub)
    } else if has_main && true_name.contains("main") {
        Some(LegacyNpcTextureRole::Main)
    } else {
        None
    }
}

/// The recovered SetupNPC loop writes Renderer.material, which is slot zero.
/// Multi-material renderers retain authored face, glass and accessory textures.
/// Older publications without this ownership field retain compatibility behavior.
pub fn native_npc_table_texture_writable(extras: Option<&bevy::gltf::GltfExtras>) -> bool {
    extras
        .and_then(|extras| serde_json::from_str::<Value>(&extras.value).ok())
        .and_then(|value| {
            value
                .get("npcTableTextureWritable")
                .and_then(Value::as_bool)
        })
        .unwrap_or(true)
}

/// Loads a runtime-selected `_MainTex` replacement with the exact sampler and
/// color-space interpretation retained by the audited glTF material binding.
///
/// This is used by modular player assembly when eye, skin or clothing choices
/// replace the authored default PNG without reintroducing Unity at runtime.
pub fn load_legacy_main_texture_replacement(
    asset_server: &AssetServer,
    pending: &PendingLegacyModelMaterial,
    path: impl Into<AssetPath<'static>>,
) -> Result<Handle<Image>, String> {
    let binding = pending
        .texture_bindings
        .iter()
        .find(|binding| binding.slot == "_MainTex")
        .ok_or_else(|| {
            format!(
                "material {} has no typed _MainTex binding for a native replacement",
                pending.true_name
            )
        })?;
    let sampler = binding
        .sampler
        .as_ref()
        .ok_or_else(|| format!("texture binding {} has no exact sampler", binding.slot))?;
    let image_sampler = ImageSampler::Descriptor(exact_sampler_descriptor(
        &binding.slot,
        &sampler.descriptor,
    )?);
    let is_srgb = binding.color_space == TextureColorSpace::Srgb;
    let path = path.into();
    Ok(asset_server
        .load_builder()
        .with_settings::<ImageLoaderSettings>(move |settings| {
            settings.is_srgb = is_srgb;
            settings.sampler = image_sampler.clone();
        })
        .load::<Image>(path))
}

/// Loads a table-selected NPC `_MainTex` with the exact sampler retained by
/// the selected source Texture2D rather than by the prefab's authored texture.
///
/// `NpcMoveController.SetupNPC` loads `texture/<table-name>.dds` and assigns
/// that Texture object to materials named `main`/`sub`. The replacement
/// Texture2D therefore owns the sampler when the prefab binding is null or
/// points at a different texture. Offline appearance tools use this boundary;
/// production callers must still provide audited source Texture2D metadata.
pub fn load_legacy_main_texture_replacement_with_sampler(
    asset_server: &AssetServer,
    pending: &PendingLegacyModelMaterial,
    path: impl Into<AssetPath<'static>>,
    sampler: &NativeSampler,
) -> Result<Handle<Image>, String> {
    let binding = pending
        .texture_bindings
        .iter()
        .find(|binding| binding.slot == "_MainTex")
        .ok_or_else(|| {
            format!(
                "material {} has no typed _MainTex binding for an NPC table replacement",
                pending.true_name
            )
        })?;
    let image_sampler = ImageSampler::Descriptor(exact_sampler_descriptor(&binding.slot, sampler)?);
    let is_srgb = binding.color_space == TextureColorSpace::Srgb;
    let path = path.into();
    Ok(asset_server
        .load_builder()
        .with_settings::<ImageLoaderSettings>(move |settings| {
            settings.is_srgb = is_srgb;
            settings.sampler = image_sampler.clone();
        })
        .load::<Image>(path))
}

/// Loads a dynamic `ActorSkinCombiner` `_MainTex` using the audited source
/// Texture2D contract. Player face, hair, skin and a few starter equipment
/// materials intentionally serialize a null `_MainTex`; in that case the
/// selected texture, rather than the material, is the sampler authority.
pub fn load_legacy_main_texture_replacement_with_contract(
    asset_server: &AssetServer,
    pending: &PendingLegacyModelMaterial,
    path: impl Into<AssetPath<'static>>,
    contract: &CharacterRuntimeTextureContract,
) -> Result<Handle<Image>, String> {
    let binding = pending
        .texture_bindings
        .iter()
        .find(|binding| binding.slot == "_MainTex")
        .ok_or_else(|| {
            format!(
                "material {} has no typed _MainTex slot for a native replacement",
                pending.true_name
            )
        })?;
    validate_runtime_replacement_contract(binding, contract)?;
    load_validated_character_runtime_texture(asset_server, path, contract)
}

/// Loads a character runtime texture with its audited sampler and color-space
/// contract before any GLB can request the same external PNG. Bevy identifies
/// an image asset by path, so a plain prewarm load would permanently win over
/// the later glTF settings request and make material validation fail.
pub fn load_character_runtime_texture_with_contract(
    asset_server: &AssetServer,
    path: impl Into<AssetPath<'static>>,
    contract: &CharacterRuntimeTextureContract,
) -> Result<Handle<Image>, String> {
    validate_character_runtime_texture_contract(contract)?;
    load_validated_character_runtime_texture(asset_server, path, contract)
}

pub(super) fn load_validated_character_runtime_texture(
    asset_server: &AssetServer,
    path: impl Into<AssetPath<'static>>,
    contract: &CharacterRuntimeTextureContract,
) -> Result<Handle<Image>, String> {
    let image_sampler =
        ImageSampler::Descriptor(exact_sampler_descriptor("_MainTex", &contract.sampler)?);
    let is_srgb = contract.usage_color_space == TextureColorSpace::Srgb;
    let path = path.into();
    Ok(asset_server
        .load_builder()
        .with_settings::<ImageLoaderSettings>(move |settings| {
            settings.is_srgb = is_srgb;
            settings.sampler = image_sampler.clone();
        })
        .load::<Image>(path))
}

pub(super) fn validate_runtime_replacement_contract(
    binding: &LegacyGltfTextureBinding,
    contract: &CharacterRuntimeTextureContract,
) -> Result<(), String> {
    if binding.color_space != contract.usage_color_space {
        return Err(format!(
            "texture binding {} color space contradicts audited runtime texture {}",
            binding.slot, contract.true_name
        ));
    }
    validate_character_runtime_texture_contract(contract)?;
    let exact_source_chain = binding
        .mip_provenance
        .as_ref()
        .is_some_and(|mips| mips.source_chain_sha256 == contract.source.source_chain_sha256);
    if exact_source_chain && let Some(material_sampler) = binding.sampler.as_ref() {
        let left = &material_sampler.descriptor;
        let right = &contract.sampler;
        if left.mag_filter != right.mag_filter
            || left.min_filter != right.min_filter
            || left.wrap_s != right.wrap_s
            || left.wrap_t != right.wrap_t
            || left.legacy_filter_mode != right.legacy_filter_mode
            || left.legacy_wrap_mode != right.legacy_wrap_mode
            || left.anisotropy_level != right.anisotropy_level
            || left.mip_map_bias != right.mip_map_bias
        {
            return Err(format!(
                "material {} _MainTex sampler contradicts runtime texture {}",
                material_sampler.descriptor.name, contract.true_name
            ));
        }
    }
    // ActorSkinCombiner.Generate() replaces `_MainTex` after cloning the
    // authored material. In particular, every `sub` material receives
    // `secondaryTextures[j]`, even when the material serialized another
    // `_MainTex` (starter shirt `sub` serializes the shirt texture and is then
    // assigned `m_skin`). Therefore source-chain identity is evidence about the
    // replacement only when the authored binding names that same Texture2D.
    // The replacement Texture2D owns its own sampler. The authored sampler is
    // acceptance evidence only when the binding already names that same
    // Texture2D; comparing it to a different runtime replacement would reject
    // valid AttachGO assignments such as rigid helmet glass materials.
    if exact_source_chain
        && !contract
            .usage_color_space_source
            .starts_with("all_hnpc ActorSkinCombiner")
        && let Some(mips) = binding.mip_provenance.as_ref()
        && (mips.source_chain_sha256 != contract.source.source_chain_sha256
            || mips.source_texture_format != contract.source.texture_format
            || mips.source_mip_count != contract.source.source_mip_count)
    {
        return Err(format!(
            "material _MainTex mip provenance contradicts runtime texture {}",
            contract.true_name
        ));
    }
    Ok(())
}

pub(super) fn validate_character_runtime_texture_contract(
    contract: &CharacterRuntimeTextureContract,
) -> Result<(), String> {
    let source_mip_chain_is_consistent = if contract.source.mip_map {
        contract.source.source_mip_count > 1
    } else {
        contract.source.source_mip_count == 1
    };
    if contract.sampler.name != contract.true_name
        || !source_mip_chain_is_consistent
        || contract.published_mip_policy != PublishedMipPolicy::BaseLevelOnly
        || contract.sampler.mip_map_bias != 0.0
    {
        return Err(format!(
            "runtime texture {} has an invalid exact sampler/mip contract",
            contract.true_name
        ));
    }
    Ok(())
}
