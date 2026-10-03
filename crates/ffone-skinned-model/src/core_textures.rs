use super::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterialTextureBinding {
    pub slot: String,
    /// Exact serialized property exists but has no usable name and an
    /// explicit null texture pointer. It is retained only as source-order
    /// provenance and must not become a runtime ShaderLab binding.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unassigned_stale_null: bool,
    /// Unity retained an assigned saved-property texture after the material
    /// switched to a shader that does not declare this slot. The exact bytes
    /// remain provenance, but native rendering must not sample the binding.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ignored_stale_shader_binding: bool,
    /// Exact non-Texture2D payload referenced by this serialized material
    /// slot. Unity 4 used MovieTexture for a small number of wearable VFX.
    /// The native renderer currently uses the declared ShaderLab fallback;
    /// this payload keeps the original media self-contained and auditable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic_texture: Option<DynamicTextureBinding>,
    /// `None` records a declared but deliberately unassigned legacy texture slot.
    pub texture: Option<u32>,
    /// Exact source Texture2D `m_Name` for the referenced published texture.
    /// Null legacy slots keep this `None`; runtime code must never infer it.
    pub source_name: Option<String>,
    /// Exact safe relative PNG URI copied from [`NativeTexture::uri`].
    /// This intentionally duplicates the indexed texture table at the material
    /// boundary so Bevy can resolve external images without reopening the GLB.
    pub uri: Option<String>,
    /// Exact indexed sampler plus its full legacy/native descriptor. Null
    /// legacy slots keep this `None`.
    pub sampler: Option<MaterialTextureSamplerBinding>,
    /// Exact Unity source-chain provenance duplicated at the material boundary
    /// so Bevy can assemble the PNG levels without reopening GLB image extras.
    pub mip_provenance: Option<TextureMipProvenance>,
    /// Every exact source level and external PNG URI. Null legacy slots keep
    /// this `None`; assigned slots must match the indexed [`NativeTexture`].
    pub mip_levels: Option<Vec<NativeTextureMipLevel>>,
    pub scale: [f64; 2],
    pub offset: [f64; 2],
    /// Exact serialized presence of Unity's optional texture pivot extension.
    /// Legacy Unity 4 `m_TexEnvs` commonly omits this value entirely.
    pub pivot: Option<[f64; 2]>,
    /// Exact serialized presence of Unity's optional texture rotation extension.
    /// Runtime consumers apply zero only after this provenance boundary.
    pub rotation: Option<f64>,
    pub color_space: TextureColorSpace,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicTextureBinding {
    pub source_id: String,
    pub source_asset_index: u64,
    pub source_path_id: i64,
    pub source_name: String,
    pub object_type: String,
    pub container: String,
    pub mime_type: String,
    pub looped: bool,
    pub audio_clip_pointer: DynamicTexturePointer,
    pub byte_length: u64,
    pub sha256: String,
    pub data_url: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicTexturePointer {
    pub source_asset_index: u64,
    pub file_id: i64,
    pub path_id: i64,
    pub is_null: bool,
}

/// Closed set of ShaderLab texture defaults used by the audited native model
/// shaders. Publication rejects every other spelling/value instead of guessing
/// how Unity would materialize it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ShaderLabTextureDefault {
    /// Exact ShaderLab right-hand side `"white" {}`.
    BuiltinWhite,
    /// Exact ShaderLab right-hand side `"black" {}`. It remains provenance
    /// until the native runtime has an audited matching built-in image.
    BuiltinBlack,
    /// Exact ShaderLab right-hand side `"gray" {}`.
    BuiltinGray,
    /// Exact ShaderLab right-hand side `"bump" {}`.
    BuiltinBump,
    /// Exact ShaderLab right-hand side `"red" {}`.
    BuiltinRed,
    /// Exact ShaderLab right-hand side `"" {}`. This means no fallback
    /// texture; it must never acquire Bevy's implicit white image.
    Blank,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShaderLabTextureDefaultProperty {
    /// Exact property identifier, preserving spelling and case.
    pub slot: String,
    pub value: ShaderLabTextureDefault,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterialTextureSamplerBinding {
    pub index: u32,
    pub descriptor: NativeSampler,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureColorSpace {
    Srgb,
    Linear,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTexture {
    /// Exact source Texture2D `m_Name`, including original spelling/extension.
    pub source_name: String,
    /// Relative semantic PNG path. Its basename is derived only from `source_name`.
    pub uri: String,
    pub width: u32,
    pub height: u32,
    pub sampler: u32,
    pub mip_provenance: TextureMipProvenance,
    /// Every decoded source mip in original Unity order. Level zero reuses
    /// `uri`; lower levels remain browsable PNGs beside the owning model.
    pub mip_levels: Vec<NativeTextureMipLevel>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextureMipProvenance {
    pub source_texture_format: i32,
    pub source_texture_format_name: String,
    pub source_mip_count: u32,
    pub source_chain_byte_length: u64,
    pub source_chain_sha256: String,
    pub source_chain_complete: bool,
    pub source_layout: TextureSourceMipLayout,
    /// Transform applied to source-decoded RGBA8 rows when publishing PNGs.
    /// Runtime verification reverses it in a temporary hash buffer while the
    /// GPU keeps the published top-left-origin pixels unchanged.
    pub published_pixel_transform: PublishedPixelTransform,
    pub published_policy: PublishedMipPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PublishedMipPolicy {
    BaseLevelOnly,
    ExactSourceLevels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureSourceMipLayout {
    LargestToSmallestContiguous,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTextureMipLevel {
    pub level: u32,
    pub width: u32,
    pub height: u32,
    pub uri: String,
    pub source_byte_offset: u64,
    pub source_byte_length: u64,
    pub source_byte_sha256: String,
    pub decoded_rgba8_byte_length: u64,
    pub decoded_rgba8_sha256: String,
    pub png_byte_length: u64,
    pub png_sha256: String,
}

/// Produces a minimally sanitized PNG filename from an exact Texture2D `m_Name`.
///
/// A known legacy raster extension is replaced with `.png`; spelling, case and
/// every other character are retained unless Windows forbids that character.
pub fn minimal_windows_png_filename(source_name: &str) -> Result<String> {
    validate_true_asset_name("texture", source_name)?;
    let stem = strip_legacy_texture_extension(source_name);
    minimal_windows_filename(stem, "png")
}

/// Produces the lossless collision form used only when two exact Texture2D
/// names collapse after replacing a legacy raster extension with `.png`.
///
/// For example, `fusion_grim.dds` becomes `fusion_grim.dds.png`, remaining
/// distinct from the exact sibling name `fusion_grim` (`fusion_grim.png`).
/// This keeps the original name readable and avoids generated hashes/PathIDs.
pub fn windows_png_filename_preserving_legacy_extension(source_name: &str) -> Result<String> {
    validate_true_asset_name("texture", source_name)?;
    minimal_windows_filename(source_name, "png")
}

pub(super) fn strip_legacy_texture_extension(value: &str) -> &str {
    let Some((stem, extension)) = value.rsplit_once('.') else {
        return value;
    };
    if matches!(
        extension.to_ascii_lowercase().as_str(),
        "bmp" | "dds" | "gif" | "jpeg" | "jpg" | "png" | "psd" | "tga" | "tif" | "tiff"
    ) && !stem.is_empty()
    {
        stem
    } else {
        value
    }
}
