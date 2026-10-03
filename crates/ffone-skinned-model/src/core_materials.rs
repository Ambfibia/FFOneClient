use super::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeMaterial {
    /// Exact source material `m_Name`; spelling and case are never normalized.
    pub name: String,
    /// Exact serialized Unity Shader object name.
    pub serialized_shader_name: String,
    /// Exact name parsed from the ShaderLab `Shader "..."` declaration.
    pub declared_shader_name: String,
    /// Runtime shader-family identity. It currently equals declaredShaderName,
    /// but stays explicit so no consumer ever substitutes serializedShaderName.
    pub legacy_shader_name: String,
    /// Fully resolved effective legacy render queue, not an unknown/default marker.
    pub render_queue: i32,
    /// Source order is authoritative and must be preserved.
    pub colors: Vec<MaterialColorProperty>,
    /// Source order is authoritative and must be preserved.
    pub floats: Vec<MaterialFloatProperty>,
    /// Every ShaderLab `2D` property in exact `Properties` source order,
    /// including properties absent from Unity's `m_SavedProperties` table.
    /// This keeps an unused blank declaration such as `_SpecMap = "" {}`
    /// distinct from an explicit built-in fallback.
    pub shader_texture_defaults: Vec<ShaderLabTextureDefaultProperty>,
    /// Source order is authoritative and must be preserved. Unity may retain
    /// a null saved-property entry after a material changes to a shader that
    /// no longer declares that slot; such a stale-null binding is provenance
    /// only and has no ShaderLab fallback or runtime texture.
    pub texture_bindings: Vec<MaterialTextureBinding>,
    /// Compiled pass order is authoritative and must be preserved.
    pub passes: Vec<MaterialPass>,
    /// `_BumpMap`/`_ShaderMap` standard glTF references exist only to make the
    /// Bevy loader apply their exact linear color space and retain handles.
    /// Native legacy rendering remains authoritative.
    pub standard_texture_refs_are_loader_hints: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialColorProperty {
    pub name: String,
    pub value: [f64; 4],
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialFloatProperty {
    pub name: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterialPass {
    /// Legacy passes may be unnamed; order remains authoritative either way.
    pub name: Option<String>,
    pub blend: MaterialBlendState,
    pub cull: MaterialCullMode,
    pub z_write: bool,
    pub z_test: MaterialCompareFunction,
    pub alpha_test: MaterialAlphaTestState,
    /// RGBA bit mask (`R=1`, `G=2`, `B=4`, `A=8`).
    pub color_mask: u8,
    pub outline: MaterialOutlineState,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterialBlendState {
    pub enabled: bool,
    pub source_color: MaterialBlendFactor,
    pub destination_color: MaterialBlendFactor,
    pub color_operation: MaterialBlendOperation,
    pub source_alpha: MaterialBlendFactor,
    pub destination_alpha: MaterialBlendFactor,
    pub alpha_operation: MaterialBlendOperation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MaterialBlendFactor {
    Zero,
    One,
    SourceColor,
    OneMinusSourceColor,
    DestinationColor,
    OneMinusDestinationColor,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationAlpha,
    OneMinusDestinationAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SourceAlphaSaturate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MaterialBlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
    Minimum,
    Maximum,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MaterialCullMode {
    Off,
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MaterialCompareFunction {
    Disabled,
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "camelCase", deny_unknown_fields)]
pub enum MaterialAlphaTestState {
    Disabled,
    Enabled {
        compare: MaterialCompareFunction,
        reference: MaterialAlphaReference,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "source", rename_all = "camelCase", deny_unknown_fields)]
pub enum MaterialAlphaReference {
    Literal { value: f64 },
    FloatProperty { name: String, resolved_value: f64 },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "camelCase", deny_unknown_fields)]
pub enum MaterialOutlineState {
    Disabled,
    WorldSpace { width: f64, color: [f64; 4] },
    ScreenSpace { width: f64, color: [f64; 4] },
}
