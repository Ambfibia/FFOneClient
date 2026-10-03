//! Editable native palettes and face images shared by creation and gameplay.
use bevy::prelude::*;
use ffone_runtime_contracts::{
    CharacterCreationAssetReference, CharacterPaletteColor, CharacterRuntimeTextureContract,
    CharacterRuntimeTextureSource,
};
use ffone_skinned_model::{
    NativeSampler, PublishedMipPolicy, SamplerMagFilter, SamplerMinFilter, SamplerWrapMode,
    TextureColorSpace,
};
use serde::Deserialize;

pub const CUSTOMIZATION_PATH: &str = "data/character_creation/customization.json";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCustomization {
    pub schema: String,
    pub skin: Vec<[f32; 4]>,
    pub hair: Vec<[f32; 4]>,
    pub eye: Vec<[f32; 4]>,
    pub creation_skin_count: usize,
    pub textures: Vec<CustomizationTexture>,
}

impl CharacterCustomization {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "ffone.character-customization.v1"
            || self.creation_skin_count == 0
            || self.creation_skin_count > self.skin.len()
            || [&self.skin, &self.hair, &self.eye].iter().any(|palette| {
                palette.is_empty()
                    || palette.len() > 127
                    || palette
                        .iter()
                        .flatten()
                        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            })
        {
            return Err("invalid native customization palettes".into());
        }
        for texture in &self.textures {
            if texture.width == 0
                || texture.height == 0
                || texture.filter != "linear"
                || texture.wrap != "repeat"
            {
                return Err(format!("invalid customization image {}", texture.true_name));
            }
        }
        Ok(())
    }

    pub fn ui_palettes(&self) -> [Vec<Color>; 3] {
        [
            &self.skin[..self.creation_skin_count],
            self.hair.as_slice(),
            self.eye.as_slice(),
        ]
        .map(|palette| {
            palette
                .iter()
                .map(|c| Color::srgba(c[0], c[1], c[2], c[3]))
                .collect()
        })
    }
}

pub fn native_palette(values: &[[f32; 4]]) -> Vec<CharacterPaletteColor> {
    values
        .iter()
        .enumerate()
        .map(|(i, &rgba)| CharacterPaletteColor {
            code: i as u8 + 1,
            rgba,
        })
        .collect()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomizationTexture {
    pub true_name: String,
    pub image: CharacterCreationAssetReference,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub filter: String,
    pub wrap: String,
}

impl CustomizationTexture {
    /// Bridge the native image into the existing player material API. No source
    /// identity participates in runtime selection or semantic asset equality.
    pub fn runtime_contract(&self) -> CharacterRuntimeTextureContract {
        CharacterRuntimeTextureContract {
            true_name: self.true_name.clone(),
            native_asset: self.image.clone(),
            native_png_sha256: self.sha256.clone(),
            source: CharacterRuntimeTextureSource {
                asset: String::new(),
                container_route: String::new(),
                path_id: 0,
                width: self.width,
                height: self.height,
                texture_format: 0,
                texture_format_name: String::new(),
                complete_image_size: u64::from(self.width) * u64::from(self.height) * 4,
                source_chain_sha256: String::new(),
                mip_map: false,
                source_mip_count: 1,
                image_count: 1,
                texture_dimension: 2,
            },
            usage_color_space: TextureColorSpace::Srgb,
            usage_color_space_source: "native player color image".into(),
            sampler: NativeSampler {
                name: self.true_name.clone(),
                mag_filter: SamplerMagFilter::Linear,
                min_filter: SamplerMinFilter::Linear,
                wrap_s: SamplerWrapMode::Repeat,
                wrap_t: SamplerWrapMode::Repeat,
                legacy_filter_mode: 1,
                legacy_wrap_mode: 0,
                anisotropy_level: 1,
                mip_map_bias: 0.0,
            },
            published_mip_policy: PublishedMipPolicy::BaseLevelOnly,
        }
    }
}
