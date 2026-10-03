//! Parsing of glTF material extras into pending model, static-world and water materials.

use super::expected_passes::{refine_exact_shader_kind, validate_typed_source_passes};
use super::material_uniforms::LegacyWaterMaterialUniform;
use super::params::{LegacyGltfTextureBinding, LegacyModelMaterialParams};
use super::pipeline_state::{linear_rgba, named_float, named_vec4};
use super::render_plan::LegacyPassPlan;
use super::shader_kind::{
    LegacyMaterialExtrasError, LegacyShaderKind, legacy_shader_uses_fixed_function_fog,
};
use super::static_shader_plan::{
    legacy_static_named_float, legacy_static_named_vec4, legacy_static_shader_plan, static_vec2,
};
use super::validation::{exact_f32, exact_vec2_f32, validate_runtime_texture_binding};
use bevy::prelude::*;
use ffone_skinned_model::{
    MaterialAlphaReference, MaterialAlphaTestState, MaterialPass, MaterialTextureBinding,
    ShaderLabTextureDefaultProperty,
};
use serde_json::Value;
use std::collections::HashSet;

/// Parsed, typed material metadata attached to a glTF primitive before its
/// standard fallback material is replaced.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct PendingLegacyModelMaterial {
    pub true_name: String,
    pub serialized_shader_name: String,
    pub declared_shader_name: String,
    pub params: LegacyModelMaterialParams,
    pub shader_texture_defaults: Vec<ShaderLabTextureDefaultProperty>,
    pub texture_bindings: Vec<LegacyGltfTextureBinding>,
    pub source_render_queue: i32,
    pub source_passes: Vec<MaterialPass>,
    pub source_pass_count: usize,
}

impl PendingLegacyModelMaterial {
    pub fn from_gltf_extras(
        gltf_material_name: Option<&str>,
        extras: &str,
    ) -> Result<Self, LegacyMaterialExtrasError> {
        let root: Value = serde_json::from_str(extras).map_err(|error| {
            LegacyMaterialExtrasError(format!("invalid material extras: {error}"))
        })?;
        let ffone = root
            .get("ffone")
            .and_then(Value::as_object)
            .ok_or_else(|| LegacyMaterialExtrasError("material extras.ffone is missing".into()))?;
        let shader_name = ffone
            .get("legacyShaderName")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                LegacyMaterialExtrasError("extras.ffone.legacyShaderName is missing".into())
            })?;
        let serialized_shader_name = ffone
            .get("serializedShaderName")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                LegacyMaterialExtrasError(
                    "extras.ffone.serializedShaderName is missing or empty".into(),
                )
            })?;
        let declared_shader_name = ffone
            .get("declaredShaderName")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                LegacyMaterialExtrasError(
                    "extras.ffone.declaredShaderName is missing or empty".into(),
                )
            })?;
        if shader_name != declared_shader_name {
            return Err(LegacyMaterialExtrasError(
                "extras.ffone.legacyShaderName contradicts declaredShaderName".into(),
            ));
        }
        let shader = LegacyShaderKind::classify_exact(declared_shader_name)
            .map_err(|error| LegacyMaterialExtrasError(error.to_string()))?;
        let true_name = ffone
            .get("name")
            .and_then(Value::as_str)
            .or(gltf_material_name)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| LegacyMaterialExtrasError("exact material name is missing".into()))?
            .to_owned();
        if ffone
            .get("standardTextureRefsAreLoaderHints")
            .and_then(Value::as_bool)
            != Some(true)
        {
            return Err(LegacyMaterialExtrasError(
                "extras.ffone.standardTextureRefsAreLoaderHints must be true".into(),
            ));
        }
        let mut params = LegacyModelMaterialParams::from_shader_name(declared_shader_name)
            .map_err(|error| LegacyMaterialExtrasError(error.to_string()))?;

        if let Some(value) = named_vec4(ffone.get("colors"), "_Color") {
            params.base_color = linear_rgba(value);
        }
        if shader == LegacyShaderKind::HologramSolidAdditive
            && let Some(value) = named_vec4(ffone.get("colors"), "_ColorA")
        {
            params.base_color = linear_rgba(value);
        }
        if shader == LegacyShaderKind::HologramSolidAdditive
            && let Some(value) = named_vec4(ffone.get("colors"), "_ColorB")
        {
            params.tint_color = linear_rgba(value);
        }
        if shader == LegacyShaderKind::HologramSolidAdditive {
            if let Some(value) = named_vec4(ffone.get("colors"), "_OverlayRColor") {
                params.ambient_color = linear_rgba(value);
            }
            if let Some(value) = named_vec4(ffone.get("colors"), "_OverlayGColor") {
                params.emission = linear_rgba(value);
            }
            if let Some(value) = named_vec4(ffone.get("colors"), "_OverlayBColor") {
                params.rim_color = linear_rgba(value);
            }
        }
        if let Some(value) = named_vec4(ffone.get("colors"), "_TintColor") {
            params.tint_color = linear_rgba(value);
        }
        // Retrobution assigns the replacement Fusion Matter program to
        // materials created for the old shader. Unity keeps stale saved
        // `_AmbColor`/`_Emission` records, but the replacement program does
        // not declare or read them; its `_BubblesTint`/`_ElectricityTint`
        // defaults remain authoritative unless those exact properties exist.
        if shader != LegacyShaderKind::FusionMatterLightDir
            && let Some(value) = named_vec4(ffone.get("colors"), "_AmbColor")
        {
            params.ambient_color = linear_rgba(value);
        }
        if shader != LegacyShaderKind::FusionMatterLightDir
            && let Some(value) = named_vec4(ffone.get("colors"), "_Emission")
        {
            params.emission = linear_rgba(value);
        }
        if let Some(value) = named_vec4(ffone.get("colors"), "_FusionTint") {
            params.tint_color = linear_rgba(value);
        }
        if let Some(value) = named_vec4(ffone.get("colors"), "_BubblesTint") {
            params.ambient_color = linear_rgba(value);
        }
        if let Some(value) = named_vec4(ffone.get("colors"), "_ElectricityTint") {
            params.emission = linear_rgba(value);
        }
        if let Some(value) = named_vec4(ffone.get("colors"), "_RimColor") {
            params.rim_color = linear_rgba(value);
        }
        if let Some(value) = named_vec4(ffone.get("colors"), "_OutlineColor") {
            params.outline_color = linear_rgba(value);
        }
        if let Some(value) = named_float(ffone.get("floats"), "_Cutoff") {
            params.alpha_cutoff = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_Outline") {
            params.outline_width = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_FatFactor") {
            params.fat_factor = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_Speed") {
            params.fusion_speed = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_RimPower") {
            params.rim_power = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_RimIntensity") {
            params.rim_intensity = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_ShadowStrength") {
            params.shadow_strength = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_EmissionStrength") {
            params.emission_strength = value;
        }
        if let Some(value) = named_float(ffone.get("floats"), "_Transparency") {
            params.transparency = value;
        }
        if shader == LegacyShaderKind::HologramSolidAdditive {
            if let Some(value) = named_float(ffone.get("floats"), "_MasterOpacity") {
                params.emission_strength = value;
            }
            if let Some(value) = named_float(ffone.get("floats"), "_BaseOpacity") {
                params.transparency = value;
            }
            if let Some(value) = named_float(ffone.get("floats"), "_OverlayROpacity") {
                params.rim_power = value;
            }
            if let Some(value) = named_float(ffone.get("floats"), "_OverlayGOpacity") {
                params.rim_intensity = value;
            }
            if let Some(value) = named_float(ffone.get("floats"), "_OverlayBOpacity") {
                params.shadow_strength = value;
            }
            if let Some(value) = named_float(ffone.get("floats"), "_MaskAlphaOpacity") {
                params.alpha_cutoff = value;
            }
        }
        if shader == LegacyShaderKind::SkinnedToonRimTransparent
            && named_float(ffone.get("floats"), "_UseOverlays").is_some_and(|value| value != 0.0)
        {
            return Err(LegacyMaterialExtrasError(
                "transparent rim material enables unsupported overlay textures".into(),
            ));
        }

        let shader_texture_defaults =
            serde_json::from_value::<Vec<ShaderLabTextureDefaultProperty>>(
                ffone.get("shaderTextureDefaults").cloned().ok_or_else(|| {
                    LegacyMaterialExtrasError(
                        "extras.ffone.shaderTextureDefaults is missing".into(),
                    )
                })?,
            )
            .map_err(|error| {
                LegacyMaterialExtrasError(format!(
                    "extras.ffone.shaderTextureDefaults is not the typed native schema: {error}"
                ))
            })?;
        let mut shader_default_slots = HashSet::new();
        for property in &shader_texture_defaults {
            if property.slot.is_empty() || !shader_default_slots.insert(property.slot.as_str()) {
                return Err(LegacyMaterialExtrasError(format!(
                    "ShaderLab texture default slot {:?} is empty or duplicated",
                    property.slot
                )));
            }
        }

        let raw_bindings = ffone
            .get("textureBindings")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                LegacyMaterialExtrasError("extras.ffone.textureBindings is missing".into())
            })?;
        for binding in raw_bindings {
            let object = binding.as_object().ok_or_else(|| {
                LegacyMaterialExtrasError("texture binding is not an object".into())
            })?;
            for required in [
                "texture",
                "sourceName",
                "uri",
                "sampler",
                "mipProvenance",
                "mipLevels",
            ] {
                if !object.contains_key(required) {
                    return Err(LegacyMaterialExtrasError(format!(
                        "texture binding omits required exact field {required}"
                    )));
                }
            }
        }
        let typed_bindings = serde_json::from_value::<Vec<MaterialTextureBinding>>(Value::Array(
            raw_bindings.clone(),
        ))
        .map_err(|error| {
            LegacyMaterialExtrasError(format!(
                "extras.ffone.textureBindings is not the typed native binding schema: {error}"
            ))
        })?;
        let mut texture_bindings = Vec::with_capacity(typed_bindings.len());
        let mut binding_slots = HashSet::new();
        for binding in typed_bindings {
            validate_runtime_texture_binding(&binding)?;
            // A declared null property still owns its UV transform and can receive
            // an XDT/ActorSkinCombiner texture later. Only undeclared stale slots
            // are discarded; no texture payload is loaded for an unassigned slot.
            if binding.ignored_stale_shader_binding
                || binding.unassigned_stale_null
                    && !shader_default_slots.contains(binding.slot.as_str())
            {
                continue;
            }
            if !binding_slots.insert(binding.slot.clone()) {
                return Err(LegacyMaterialExtrasError(format!(
                    "texture binding slot {} occurs more than once",
                    binding.slot
                )));
            }
            if !shader_default_slots.contains(binding.slot.as_str()) && binding.texture.is_some() {
                return Err(LegacyMaterialExtrasError(format!(
                    "assigned texture binding {} has no exact ShaderLab default",
                    binding.slot
                )));
            }
            let texture_index = binding
                .texture
                .map(|index| usize::try_from(index).expect("u32 fits usize on Bevy targets"));
            let scale = exact_vec2_f32(binding.scale, &binding.slot, "scale")?;
            let offset = exact_vec2_f32(binding.offset, &binding.slot, "offset")?;
            let pivot = binding
                .pivot
                .map(|pivot| exact_vec2_f32(pivot, &binding.slot, "pivot"))
                .transpose()?;
            let rotation = binding
                .rotation
                .map(|rotation| exact_f32(rotation, &binding.slot, "rotation"))
                .transpose()?;
            if binding.slot == "_MainTex"
                || shader == LegacyShaderKind::HologramSolidAdditive && binding.slot == "_MaskTex"
            {
                params.uv_scale = scale;
                params.uv_offset = offset;
                params.uv_pivot = pivot.unwrap_or(Vec2::ZERO);
                params.uv_rotation_degrees = rotation.unwrap_or(0.0);
            }
            texture_bindings.push(LegacyGltfTextureBinding {
                slot: binding.slot,
                texture_index,
                source_name: binding.source_name,
                uri: binding.uri,
                sampler: binding.sampler,
                mip_provenance: binding.mip_provenance,
                mip_levels: binding.mip_levels,
                color_space: binding.color_space,
                scale,
                offset,
                pivot,
                rotation,
            });
        }

        let source_passes =
            serde_json::from_value::<Vec<MaterialPass>>(ffone.get("passes").cloned().ok_or_else(
                || LegacyMaterialExtrasError("extras.ffone.passes is missing".into()),
            )?)
            .map_err(|error| {
                LegacyMaterialExtrasError(format!(
                    "extras.ffone.passes is not the typed native pass schema: {error}"
                ))
            })?;
        params.shader = refine_exact_shader_kind(declared_shader_name, shader, &source_passes);

        // A serialized Unity Material only stores overridden values. When
        // `_Cutoff` is absent, the publisher resolves the exact ShaderLab
        // property default into the typed pass and binds that pass into the
        // model's semantic proof. Use that resolved FloatProperty value for
        // the runtime uniform too; the generic 0.9 compatibility default is
        // only valid for older metadata that has no typed source pass.
        if named_float(ffone.get("floats"), "_Cutoff").is_none()
            && matches!(
                params.shader,
                LegacyShaderKind::AdditiveTwoSided
                    | LegacyShaderKind::AdditiveTestTwoSidedQueue3011
                    | LegacyShaderKind::SrcAlphaAdditiveTestTwoSided
            )
        {
            if let Some(MaterialAlphaReference::FloatProperty {
                name,
                resolved_value,
            }) = source_passes
                .first()
                .and_then(|pass| match &pass.alpha_test {
                    MaterialAlphaTestState::Enabled { reference, .. } => Some(reference),
                    MaterialAlphaTestState::Disabled => None,
                })
            {
                if name == "_Cutoff" && resolved_value.is_finite() {
                    let cutoff = *resolved_value as f32;
                    if cutoff.is_finite() {
                        params.alpha_cutoff = cutoff;
                    }
                }
            }
        }

        let source_render_queue = ffone
            .get("renderQueue")
            .and_then(Value::as_i64)
            .and_then(|queue| i32::try_from(queue).ok())
            .ok_or_else(|| {
                LegacyMaterialExtrasError(
                    "extras.ffone.renderQueue is missing or is not an i32".into(),
                )
            })?;
        let expected_queue = params
            .render_plan()
            .passes
            .first()
            .expect("closed shader set always has a pass")
            .render_mode
            .source_queue;
        if source_render_queue != expected_queue {
            return Err(LegacyMaterialExtrasError(format!(
                "typed renderQueue {source_render_queue} contradicts {} queue {expected_queue}",
                params.shader.exact_name()
            )));
        }
        validate_typed_source_passes(&params, &source_passes)?;
        let source_pass_count = source_passes.len();
        match ffone.get("nativeSurfaceStyle").and_then(Value::as_str) {
            None if ffone.get("nativeSurfaceStyle").is_none() => {},
            Some("cel") if matches!(params.shader, LegacyShaderKind::OpaqueNormal
                | LegacyShaderKind::AlphaBlendNormal | LegacyShaderKind::TransparentNormal) => {
                params.native_cel_shading = true;
            },
            _ => return Err(LegacyMaterialExtrasError("unsupported nativeSurfaceStyle contract".into())),
        }

        match ffone.get("nativeOutline") {
            None => {},
            Some(Value::Bool(enabled)) if params.native_cel_shading => {
                params.native_outline = *enabled;
            },
            _ => return Err(LegacyMaterialExtrasError("nativeOutline requires a boolean and nativeSurfaceStyle=cel".into())),
        }

        match ffone.get("nativeInkRim") {
            None => {},
            Some(Value::Bool(enabled)) if params.native_cel_shading
                && params.shader == LegacyShaderKind::OpaqueNormal => {
                params.native_ink_rim = *enabled;
            },
            _ => return Err(LegacyMaterialExtrasError("nativeInkRim requires a boolean and opaque nativeSurfaceStyle=cel".into())),
        }

        Ok(Self {
            true_name,
            serialized_shader_name: serialized_shader_name.to_owned(),
            declared_shader_name: declared_shader_name.to_owned(),
            params,
            shader_texture_defaults,
            texture_bindings,
            source_render_queue,
            source_passes,
            source_pass_count,
        })
    }
}

/// Compatibility metadata emitted by the first native static-world exporter.
///
/// That exporter predates the strict logical-model material contract and puts
/// its fields directly in `materials[*].extras`. Rejecting it leaves Bevy's
/// PBR/transparent fallback active for every tutorial prop, even though the
/// export still carries the source shader, texture and render-state identity.
/// Keep this adapter separate so strict logical models remain fail-closed.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct PendingLegacyStaticWorldMaterial {
    pub(super) source_material_id: String,
    pub(super) true_name: String,
    pub(super) base_texture_id: Option<String>,
    pub(super) params: LegacyModelMaterialParams,
    pub(super) render_passes: Vec<LegacyPassPlan>,
    pub(super) has_exact_base_color: bool,
    pub(super) has_exact_ambient_color: bool,
    pub(super) has_exact_emission: bool,
    pub(super) has_glow_mask: bool,
}

impl PendingLegacyStaticWorldMaterial {
    pub(crate) fn from_gltf_extras(
        gltf_material_name: Option<&str>,
        extras: &str,
    ) -> Result<Self, LegacyMaterialExtrasError> {
        let root: Value = serde_json::from_str(extras).map_err(|error| {
            LegacyMaterialExtrasError(format!("invalid material extras: {error}"))
        })?;
        if root.get("ffone").is_some() {
            return Err(LegacyMaterialExtrasError(
                "material uses the strict extras.ffone schema".into(),
            ));
        }
        let source_material_id = root
            .get("ffoneSourceMaterialId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                LegacyMaterialExtrasError(
                    "legacy static extras.ffoneSourceMaterialId is missing".into(),
                )
            })?
            .to_owned();
        let shader_name = root
            .get("shaderName")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                LegacyMaterialExtrasError("legacy static extras.shaderName is missing".into())
            })?;
        let (shader, render_passes) = legacy_static_shader_plan(shader_name)
            .map_err(|error| LegacyMaterialExtrasError(error.to_string()))?;
        let has_glow_mask = legacy_static_glow_shader(shader_name);

        if has_glow_mask {
            let texture_contract = root
                .get("runtimeTextureContract")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    LegacyMaterialExtrasError(
                        "normal_glow runtime texture contract is missing".into(),
                    )
                })?;
            for (field, expected) in [
                ("schema", "ffone.legacy-glow-texture-bindings.v1"),
                ("emissiveTexture", "_BumpMap"),
                ("semantic", "constantColor(0.5)-lerp-by-texture-alpha"),
            ] {
                if texture_contract.get(field).and_then(Value::as_str) != Some(expected) {
                    return Err(LegacyMaterialExtrasError(format!(
                        "normal_glow runtime texture contract field {field:?} is not {expected:?}"
                    )));
                }
            }
            let assigned = root
                .get("exactTextureSlots")
                .and_then(Value::as_array)
                .and_then(|slots| {
                    slots
                        .iter()
                        .find(|slot| slot.get("name").and_then(Value::as_str) == Some("_BumpMap"))
                })
                .and_then(|slot| slot.get("textureId"))
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty());
            if !assigned {
                return Err(LegacyMaterialExtrasError(
                    "normal_glow exact _BumpMap texture slot is unassigned".into(),
                ));
            }
        } else if root
            .pointer("/runtimeTextureContract/schema")
            .and_then(Value::as_str)
            == Some("ffone.legacy-glow-texture-bindings.v1")
        {
            return Err(LegacyMaterialExtrasError(format!(
                "non-glow static shader {shader_name:?} declares a glow texture contract"
            )));
        }

        let true_name = gltf_material_name
            .filter(|name| !name.is_empty())
            .unwrap_or(source_material_id.as_str())
            .to_owned();
        let mut params = LegacyModelMaterialParams::for_shader(shader);
        params.glow_mask = has_glow_mask;
        params.fixed_function_fog = legacy_shader_uses_fixed_function_fog(shader_name);
        let main_texture_slot = root
            .get("exactTextureSlots")
            .and_then(Value::as_array)
            .and_then(|slots| {
                slots
                    .iter()
                    .find(|slot| slot.get("name").and_then(Value::as_str) == Some("_MainTex"))
            });
        let base_texture_id = main_texture_slot
            .and_then(|slot| slot.get("textureId"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        if let Some(slot) = main_texture_slot {
            if let Some(scale) = static_vec2(slot.get("scale")) {
                params.uv_scale = scale;
            }
            if let Some(offset) = static_vec2(slot.get("offset")) {
                params.uv_offset = offset;
            }
            if let Some(pivot) = static_vec2(slot.get("pivot")) {
                params.uv_pivot = pivot;
            }
            if let Some(rotation) = slot
                .get("rotation")
                .and_then(Value::as_f64)
                .map(|value| value as f32)
                .filter(|value| value.is_finite())
            {
                params.uv_rotation_degrees = rotation;
            }
        }

        // New exports include these exact saved values. Older static-world
        // GLBs expose `_Color` only through glTF's baseColorFactor; their
        // audited compatibility defaults are supplied after Bevy loads it.
        let mut base_color = legacy_static_named_vec4(root.get("colors"), "_Color");
        let mut ambient_color = legacy_static_named_vec4(root.get("colors"), "_AmbColor");
        let mut emission = legacy_static_named_vec4(root.get("colors"), "_Emission");
        if shader_name == "RetroLit" {
            // Primary Foster interior: Lighting On, Ambient [_Color],
            // SeparateSpecular On, texture * primary DOUBLE. Its explicit
            // black _SpecColor removes the separate-specular term and its
            // unit material alpha makes the alpha combiner texture-only.
            // Those values let us reuse the proven family-6 color program;
            // other RetroLit variants must not silently lose specular/alpha.
            let specular =
                legacy_static_named_vec4(root.get("colors"), "_SpecColor").unwrap_or([1.0; 4]);
            let color = base_color.unwrap_or([1.0; 4]);
            if specular[..3] != [0.0; 3] || color[3] != 1.0 {
                return Err(LegacyMaterialExtrasError(
                    "RetroLit requires the audited zero-specular, unit-alpha material contract"
                        .into(),
                ));
            }
            base_color = Some(color);
            ambient_color = Some(color);
            emission = Some(emission.unwrap_or([0.0; 4]));
        }
        if let Some(value) = base_color {
            params.base_color = linear_rgba(value);
        }
        if let Some(value) = ambient_color {
            params.ambient_color = linear_rgba(value);
        }
        if let Some(value) = emission {
            params.emission = linear_rgba(value);
        }
        if let Some(value) = legacy_static_named_float(root.get("floats"), "_Cutoff") {
            params.alpha_cutoff = value;
        }

        Ok(Self {
            source_material_id,
            true_name,
            base_texture_id,
            params,
            render_passes,
            has_exact_base_color: base_color.is_some(),
            has_exact_ambient_color: ambient_color.is_some(),
            has_exact_emission: emission.is_some(),
            has_glow_mask,
        })
    }
}

pub(super) fn legacy_static_glow_shader(name: &str) -> bool {
    matches!(
        name,
        "normal_glow_blendSrcalphaInvsrcalpha"
            | "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff"
            | "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD"
            | "normal_glow_blendOneOne_zwriteOff_vertexColorAD"
    )
}

/// Exact `ffWater` material values published by the native static-world
/// exporter. `ffPoison` deliberately uses the same source shader with a
/// different gradient, Fresnel map, palette and wave motion.
#[derive(Component, Debug, Clone, PartialEq)]
pub(super) struct PendingLegacyWaterMaterial {
    pub(super) source_material_id: String,
    pub(super) true_name: String,
    pub(super) uniform: LegacyWaterMaterialUniform,
}

impl PendingLegacyWaterMaterial {
    pub(super) fn from_gltf_extras(
        gltf_material_name: Option<&str>,
        extras: &str,
    ) -> Result<Self, LegacyMaterialExtrasError> {
        let root: Value = serde_json::from_str(extras).map_err(|error| {
            LegacyMaterialExtrasError(format!("invalid material extras: {error}"))
        })?;
        if root.get("shaderName").and_then(Value::as_str) != Some("ffWater") {
            return Err(LegacyMaterialExtrasError(
                "material is not the exact ffWater shader".into(),
            ));
        }
        let source_material_id = root
            .get("ffoneSourceMaterialId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                LegacyMaterialExtrasError("ffWater extras.ffoneSourceMaterialId is missing".into())
            })?
            .to_owned();
        let true_name = gltf_material_name
            .filter(|name| !name.is_empty())
            .unwrap_or(source_material_id.as_str())
            .to_owned();
        if !matches!(true_name.as_str(), "ffWater" | "ffPoison") {
            return Err(LegacyMaterialExtrasError(format!(
                "unsupported ffWater material identity {true_name:?}"
            )));
        }
        let texture_contract = root
            .get("runtimeTextureContract")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                LegacyMaterialExtrasError("ffWater runtime texture contract is missing".into())
            })?;
        let expected_contract = [
            ("schema", "ffone.legacy-water-texture-bindings.v1"),
            ("baseColorTexture", "_ReflectiveColor"),
            ("normalTexture", "_BumpMap"),
            ("emissiveTexture", "_Fresnel"),
        ];
        for (field, expected) in expected_contract {
            if texture_contract.get(field).and_then(Value::as_str) != Some(expected) {
                return Err(LegacyMaterialExtrasError(format!(
                    "ffWater runtime texture contract field {field:?} is not {expected:?}"
                )));
            }
        }
        for required in ["_BumpMap", "_Fresnel", "_ReflectiveColor"] {
            let assigned = root
                .get("exactTextureSlots")
                .and_then(Value::as_array)
                .and_then(|slots| {
                    slots
                        .iter()
                        .find(|slot| slot.get("name").and_then(Value::as_str) == Some(required))
                })
                .and_then(|slot| slot.get("textureId"))
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty());
            if !assigned {
                return Err(LegacyMaterialExtrasError(format!(
                    "ffWater exact texture slot {required} is unassigned"
                )));
            }
        }
        let required_color = |name: &str| {
            legacy_static_named_vec4(root.get("colors"), name).ok_or_else(|| {
                LegacyMaterialExtrasError(format!("ffWater saved color {name} is missing"))
            })
        };
        let required_float = |name: &str| {
            legacy_static_named_float(root.get("floats"), name).ok_or_else(|| {
                LegacyMaterialExtrasError(format!("ffWater saved float {name} is missing"))
            })
        };
        let color = required_color("_Color")?;
        let wave_speed = required_color("WaveSpeed")?;
        let refr_color = required_color("_RefrColor")?;
        let horizon_color = required_color("_HorizonColor")?;
        let foam_color = required_color("_FoamColor")?;
        let wave_scale = required_float("_WaveScale")?;
        let reflection_distortion = required_float("_ReflDistort")?;
        let refraction_distortion = required_float("_RefrDistort")?;
        for (name, values) in [
            ("_Color", color),
            ("WaveSpeed", wave_speed),
            ("_RefrColor", refr_color),
            ("_HorizonColor", horizon_color),
            ("_FoamColor", foam_color),
        ] {
            if !values.iter().all(|value| value.is_finite()) {
                return Err(LegacyMaterialExtrasError(format!(
                    "ffWater saved color {name} is non-finite"
                )));
            }
        }
        if ![wave_scale, reflection_distortion, refraction_distortion]
            .iter()
            .all(|value| value.is_finite())
        {
            return Err(LegacyMaterialExtrasError(
                "ffWater saved float is non-finite".into(),
            ));
        }

        let poison_marker = if true_name == "ffPoison" { 1.0 } else { 0.0 };
        Ok(Self {
            source_material_id,
            true_name,
            uniform: LegacyWaterMaterialUniform {
                color: linear_rgba(color),
                wave_speed: Vec4::from_array(wave_speed),
                refr_color: linear_rgba(refr_color),
                horizon_color: linear_rgba(horizon_color),
                foam_color: linear_rgba(foam_color),
                water_params: Vec4::new(
                    wave_scale,
                    reflection_distortion,
                    refraction_distortion,
                    poison_marker,
                ),
            },
        })
    }
}
