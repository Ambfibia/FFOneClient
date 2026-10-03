use std::collections::HashMap;

use bevy::{asset::AssetPath, prelude::Resource};
use ffone_skinned_model::{MaterialTextureBinding, TextureColorSpace};
use serde::Deserialize;

use super::{
    LegacyGltfTextureBinding, exact_f32, exact_sampler_descriptor, exact_vec2_f32,
    is_safe_relative_png_uri, validate_exact_mip_metadata,
};

const PUBLISHED_MANIFEST_PATH: &str = "objects/legacy-static-exact-mips.json";
const PUBLISHED_MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../../assets/game/objects/legacy-static-exact-mips.json");
const MANIFEST_SCHEMA: &str = "ffone.legacy-static-exact-mips.v1";
const PRIMARY_SOURCE_ALIAS: &str = "primary";
const PRIMARY_ASSET: &str = "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a";
const PRIMARY_CONTAINER: &str = "Tutorial.resourceFile";
const PRIMARY_CONTAINER_BYTE_LENGTH: u64 = 27_003_826;
const PRIMARY_CONTAINER_SHA256: &str =
    "49a684ff4236848d0b882a5d725ffbe99d5cfb5d2090dc8705350e97dd3fd024";

#[derive(Resource, Debug)]
pub(super) struct LegacyStaticExactMipManifest {
    pub(super) source: AssetPath<'static>,
    pub(super) bindings: HashMap<String, LegacyGltfTextureBinding>,
}

impl LegacyStaticExactMipManifest {
    pub(super) fn from_published() -> Result<Self, String> {
        Self::from_bytes(PUBLISHED_MANIFEST_BYTES)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let published: PublishedManifest = serde_json::from_slice(bytes)
            .map_err(|error| format!("cannot parse {PUBLISHED_MANIFEST_PATH}: {error}"))?;
        published.validate_source()?;

        let mut bindings = HashMap::with_capacity(published.textures.len());
        let mut previous_texture_id: Option<&str> = None;
        for entry in &published.textures {
            if previous_texture_id.is_some_and(|previous| previous >= entry.texture_id.as_str()) {
                return Err("textures must be sorted by unique textureId".to_owned());
            }
            previous_texture_id = Some(&entry.texture_id);

            entry.validate_provenance(&published.source.asset)?;
            let binding = runtime_binding(&entry.binding).map_err(|error| {
                format!(
                    "texture {} has an invalid binding: {error}",
                    entry.texture_id
                )
            })?;
            bindings.insert(entry.texture_id.clone(), binding);
        }
        if bindings.is_empty() {
            return Err("textures must contain at least one exact mip binding".to_owned());
        }

        Ok(Self {
            // Entry URIs are relative to `assets/game/objects`, the directory
            // containing the published manifest.
            source: AssetPath::from(PUBLISHED_MANIFEST_PATH),
            bindings,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublishedManifest {
    schema: String,
    source: PublishedSource,
    extractor: PublishedTool,
    publisher: PublishedTool,
    textures: Vec<PublishedTexture>,
}

impl PublishedManifest {
    fn validate_source(&self) -> Result<(), String> {
        if self.schema != MANIFEST_SCHEMA {
            return Err(format!(
                "schema {:?} is not {MANIFEST_SCHEMA:?}",
                self.schema
            ));
        }
        if self.source.alias != PRIMARY_SOURCE_ALIAS
            || self.source.asset != PRIMARY_ASSET
            || self.source.relative_container_path != PRIMARY_CONTAINER
            || self.source.byte_length != PRIMARY_CONTAINER_BYTE_LENGTH
            || self.source.sha256 != PRIMARY_CONTAINER_SHA256
        {
            return Err("primary source identity does not match the audited container".to_owned());
        }
        self.extractor.validate("extractor")?;
        self.publisher.validate("publisher")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublishedSource {
    alias: String,
    asset: String,
    relative_container_path: String,
    byte_length: u64,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishedTool {
    name: String,
    version: String,
    command: String,
}

impl PublishedTool {
    fn validate(&self, role: &str) -> Result<(), String> {
        if self.name.trim().is_empty()
            || self.version.trim().is_empty()
            || self.command.trim().is_empty()
        {
            return Err(format!("{role} provenance is incomplete"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublishedTexture {
    texture_id: String,
    source_object_path_id: u32,
    binding: MaterialTextureBinding,
}

impl PublishedTexture {
    fn validate_provenance(&self, source_asset: &str) -> Result<(), String> {
        let expected_texture_id = format!("{source_asset}:{}", self.source_object_path_id);
        if self.texture_id != expected_texture_id {
            return Err(format!(
                "textureId {:?} contradicts sourceObjectPathId {}",
                self.texture_id, self.source_object_path_id
            ));
        }
        if self.binding.slot != "_MainTex"
            || self.binding.texture != Some(self.source_object_path_id)
        {
            return Err(format!(
                "texture {} is not its exact _MainTex binding",
                self.texture_id
            ));
        }
        Ok(())
    }
}

fn runtime_binding(binding: &MaterialTextureBinding) -> Result<LegacyGltfTextureBinding, String> {
    validate_static_binding(binding)?;
    let scale = exact_vec2_f32(binding.scale, &binding.slot, "scale").map_err(|error| error.0)?;
    let offset =
        exact_vec2_f32(binding.offset, &binding.slot, "offset").map_err(|error| error.0)?;
    let pivot = binding
        .pivot
        .map(|value| exact_vec2_f32(value, &binding.slot, "pivot"))
        .transpose()
        .map_err(|error| error.0)?;
    let rotation = binding
        .rotation
        .map(|value| exact_f32(value, &binding.slot, "rotation"))
        .transpose()
        .map_err(|error| error.0)?;

    Ok(LegacyGltfTextureBinding {
        slot: binding.slot.clone(),
        texture_index: binding
            .texture
            .map(|value| usize::try_from(value).expect("u32 fits usize on Bevy targets")),
        source_name: binding.source_name.clone(),
        uri: binding.uri.clone(),
        sampler: binding.sampler.clone(),
        mip_provenance: binding.mip_provenance.clone(),
        mip_levels: binding.mip_levels.clone(),
        color_space: binding.color_space,
        scale,
        offset,
        pivot,
        rotation,
    })
}

fn validate_static_binding(binding: &MaterialTextureBinding) -> Result<(), String> {
    if binding.unassigned_stale_null || binding.ignored_stale_shader_binding {
        return Err("published static binding cannot be stale or ignored".to_owned());
    }
    if binding.dynamic_texture.is_some() {
        return Err("published static binding cannot be dynamic".to_owned());
    }
    let (Some(source_name), Some(uri), Some(sampler), Some(provenance), Some(levels)) = (
        binding.source_name.as_deref(),
        binding.uri.as_deref(),
        binding.sampler.as_ref(),
        binding.mip_provenance.as_ref(),
        binding.mip_levels.as_deref(),
    ) else {
        return Err("published static binding has incomplete texture metadata".to_owned());
    };
    if binding.texture.is_none() || !is_safe_relative_png_uri(uri) {
        return Err("published static binding has no texture or has an unsafe PNG URI".to_owned());
    }
    if sampler.descriptor.name != source_name {
        return Err("published static sampler name contradicts sourceName".to_owned());
    }
    if binding.color_space != TextureColorSpace::Srgb {
        return Err("published static _MainTex must be typed as sRGB".to_owned());
    }
    validate_exact_mip_metadata(&binding.slot, uri, provenance, levels).map_err(|error| error.0)?;
    exact_sampler_descriptor(&binding.slot, &sampler.descriptor)?;
    Ok(())
}

#[cfg(test)]
mod tests;
