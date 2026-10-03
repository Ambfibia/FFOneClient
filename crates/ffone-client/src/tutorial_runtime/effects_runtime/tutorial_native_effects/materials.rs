use super::*;

/// Exact `UnityEngine.ParticleRenderMode` values from Retrobution's Unity 2.x
/// `UnityEngine.dll`. The serialized field retained its old
/// `m_StretchParticles` name even for the horizontal and vertical billboard
/// modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LegacyParticleRenderMode {
    Billboard,
    SortedBillboard,
    HorizontalBillboard,
    VerticalBillboard,
}

impl TryFrom<i64> for LegacyParticleRenderMode {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Billboard),
            2 => Ok(Self::SortedBillboard),
            4 => Ok(Self::HorizontalBillboard),
            5 => Ok(Self::VerticalBillboard),
            other => Err(format!(
                "unsupported UnityEngine.ParticleRenderMode {other}"
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct MaterialAnimationPlan {
    pub(super) duration: f32,
    pub(super) curves: Vec<MaterialCurvePlan>,
}

#[derive(Clone, Debug)]
pub(super) struct MaterialCurvePlan {
    pub(super) node_name: String,
    pub(super) property: MaterialAnimatedProperty,
    pub(super) curve: AnimationCurve,
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub(super) enum MaterialAnimatedProperty {
    BaseRed,
    BaseGreen,
    BaseBlue,
    BaseAlpha,
    AmbientRed,
    AmbientGreen,
    AmbientBlue,
    AmbientAlpha,
    EmissionRed,
    EmissionGreen,
    EmissionBlue,
    EmissionAlpha,
    UvScaleX,
    UvScaleY,
    UvOffsetX,
    UvOffsetY,
    UvRotationDegrees,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize)]
pub(super) enum ParticleBlendMode {
    ZeroOneMinusSrcAlpha,
    OneOne,
    OneOneMinusSrcColor,
    OneMinusDstColorOne,
    OneMinusSrcColorDstAlpha,
    OneMinusSrcColorOneMinusSrcColor,
    SrcColorOne,
    OneMinusSrcAlphaOne,
    SrcAlphaOne,
    SrcAlphaOneMinusSrcColor,
    DstAlphaOneMinusSrcColor,
}

pub(super) fn compile_material_animation(
    effect_id: i32,
    closure: &TutorialEffectClosureFile,
) -> Result<Option<MaterialAnimationPlan>, TutorialNativeClosureBlockerReason> {
    if !matches!(
        effect_id,
        15 | 391
            | 425
            | 547
            | 548
            | 601
            | 602
            | 626
            | 627
            | 705
            | 734
            | 736
            | 739
            | 740
            | 741
            | 751
            | 833
            | 834
    ) {
        return Ok(None);
    }
    let clips = closure
        .objects
        .iter()
        .filter(|object| object.object_type == "AnimationClip" && object.name == "nif-default")
        .collect::<Vec<_>>();
    let [clip] = clips.as_slice() else {
        return Err(invalid(format!(
            "effect {effect_id} expected one nif-default AnimationClip, found {}",
            clips.len()
        )));
    };
    let mut curves = Vec::new();
    let mut duration = 0.0_f32;
    for source in array(&clip.value, "m_FloatCurves").map_err(invalid)? {
        if integer(source, "classID").map_err(invalid)? != 21 {
            return Err(invalid(format!(
                "effect {effect_id} float curve does not target Material class 21"
            )));
        }
        let attribute = source
            .get("attribute")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                invalid(format!(
                    "effect {effect_id} material curve has no attribute"
                ))
            })?;
        let property = match attribute {
            "_Color.r" => Some(MaterialAnimatedProperty::BaseRed),
            "_Color.g" => Some(MaterialAnimatedProperty::BaseGreen),
            "_Color.b" => Some(MaterialAnimatedProperty::BaseBlue),
            "_Color.a" => Some(MaterialAnimatedProperty::BaseAlpha),
            "_AmbColor.r" => Some(MaterialAnimatedProperty::AmbientRed),
            "_AmbColor.g" => Some(MaterialAnimatedProperty::AmbientGreen),
            "_AmbColor.b" => Some(MaterialAnimatedProperty::AmbientBlue),
            "_AmbColor.a" => Some(MaterialAnimatedProperty::AmbientAlpha),
            "_Emission.r" => Some(MaterialAnimatedProperty::EmissionRed),
            "_Emission.g" => Some(MaterialAnimatedProperty::EmissionGreen),
            "_Emission.b" => Some(MaterialAnimatedProperty::EmissionBlue),
            "_Emission.a" => Some(MaterialAnimatedProperty::EmissionAlpha),
            "_MainTex.scale.x" => Some(MaterialAnimatedProperty::UvScaleX),
            "_MainTex.scale.y" => Some(MaterialAnimatedProperty::UvScaleY),
            "_MainTex.offset.x" => Some(MaterialAnimatedProperty::UvOffsetX),
            "_MainTex.offset.y" => Some(MaterialAnimatedProperty::UvOffsetY),
            "_MainTex.rotation" => Some(MaterialAnimatedProperty::UvRotationDegrees),
            // The exact native shader intentionally has no specular term, so
            // serialized _SpecColor curves cannot alter its rendered result.
            "_SpecColor.r" | "_SpecColor.g" | "_SpecColor.b" | "_SpecColor.a" => None,
            other => {
                return Err(invalid(format!(
                    "effect {effect_id} has unsupported material animation property {other:?}"
                )));
            }
        };
        let Some(property) = property else {
            continue;
        };
        let node_name = source
            .get("path")
            .and_then(JsonValue::as_str)
            .filter(|path| !path.is_empty())
            .and_then(|path| path.rsplit('/').next())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                invalid(format!(
                    "effect {effect_id} material curve path has no exact terminal node name"
                ))
            })?
            .to_owned();
        let curve = parse_material_curve(source, "curve").map_err(invalid)?;
        let curve_duration = curve.0.last().map_or(0.0, |key| key.time);
        duration = duration.max(curve_duration);
        curves.push(MaterialCurvePlan {
            node_name,
            property,
            curve,
        });
    }
    if curves.is_empty() || !duration.is_finite() || duration <= 0.0 {
        return Err(invalid(format!(
            "effect {effect_id} has no reproducible native material animation curves"
        )));
    }
    Ok(Some(MaterialAnimationPlan { duration, curves }))
}

pub(super) fn parse_material_curve(value: &JsonValue, field: &str) -> Result<AnimationCurve, String> {
    parse_curve_with_modes(value, field, true)
}

pub(super) fn particle_blend_mode(shader_name: &str) -> Option<ParticleBlendMode> {
    Some(match shader_name {
        "particle_blendZeroInvsrcalpha_zwriteOff_cullOff" => {
            ParticleBlendMode::ZeroOneMinusSrcAlpha
        }
        "particle_blendOneOne_zwriteOff_cullOff" => ParticleBlendMode::OneOne,
        "particle_blendOneInvsrccolor_zwriteOff_cullOff" => ParticleBlendMode::OneOneMinusSrcColor,
        "particle_blendInvdestcolorOne_zwriteOff_cullOff" => ParticleBlendMode::OneMinusDstColorOne,
        "particle_blendInvsrccolorDestalpha_zwriteOff_cullOff" => {
            ParticleBlendMode::OneMinusSrcColorDstAlpha
        }
        "particle_blendInvsrccolorInvsrccolor_zwriteOff_cullOff" => {
            ParticleBlendMode::OneMinusSrcColorOneMinusSrcColor
        }
        "particle_blendSrccolorOne_zwriteOff_cullOff" => ParticleBlendMode::SrcColorOne,
        "particle_blendInvsrcalphaOne_zwriteOff_cullOff" => ParticleBlendMode::OneMinusSrcAlphaOne,
        "particle_blendSrcalphaOne_zwriteOff_cullOff" => ParticleBlendMode::SrcAlphaOne,
        "particle_blendSrcalphaInvsrccolor_zwriteOff_cullOff" => {
            ParticleBlendMode::SrcAlphaOneMinusSrcColor
        }
        "particle_blendDestalphaInvsrccolor_zwriteOff_cullOff" => {
            ParticleBlendMode::DstAlphaOneMinusSrcColor
        }
        _ => return None,
    })
}

pub(super) fn exact_shader_name(shader: &TutorialUnityObjectProof) -> Result<String, String> {
    if let Some(name) = shader
        .value
        .get("m_Name")
        .and_then(JsonValue::as_str)
        .filter(|name| !name.is_empty())
    {
        return Ok(name.to_owned());
    }
    if !shader.name.is_empty() {
        return Ok(shader.name.clone());
    }
    let script = shader
        .value
        .get("m_Script")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "Shader has no exact name or source text".to_owned())?;
    const PREFIX: &str = "Shader \"";
    if !script.starts_with(PREFIX) || script.matches(PREFIX).count() != 1 {
        return Err("Shader source has no unique leading Shader name token".to_owned());
    }
    let remainder = &script[PREFIX.len()..];
    let end = remainder
        .find('"')
        .ok_or_else(|| "Shader source leading name is unterminated".to_owned())?;
    let name = &remainder[..end];
    if name.is_empty() || !remainder[end + 1..].trim_start().starts_with('{') {
        return Err("Shader source leading name/header is malformed".to_owned());
    }
    Ok(name.to_owned())
}

#[derive(Debug, Clone, Copy, ShaderType)]
pub(super) struct TutorialParticleUniform {
    pub(super) tint: Vec4,
    pub(super) uv_scale_offset: Vec4,
    pub(super) legacy_gamma_accumulation_gain: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(TutorialParticlePipelineKey)]
#[type_path = "ffone_client::tutorial_effects_runtime::tutorial_native_effects"]
pub(super) struct TutorialParticleMaterial {
    #[uniform(0)]
    pub(super) uniform: TutorialParticleUniform,
    #[texture(1)]
    #[sampler(2)]
    pub(super) texture: Handle<Image>,
    pub(super) blend_mode: ParticleBlendMode,
}

impl Material for TutorialParticleMaterial {
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn depth_bias(&self) -> f32 {
        // Particle shaders belong to the same transparent queue as mesh
        // effects. Without this bias, depth-writing world materials can be
        // submitted after a foreground particle and paint over it as the
        // camera changes their relative origins. Keep the normal depth test.
        mesh_effect_render_queue_sort_bias(3_003)
    }

    fn vertex_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("tutorial_native_effects.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn fragment_shader() -> ShaderRef {
        Self::vertex_shader()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
        }
        let (src_factor, dst_factor) = match key.bind_group_data.blend_mode {
            ParticleBlendMode::ZeroOneMinusSrcAlpha => {
                (BlendFactor::Zero, BlendFactor::OneMinusSrcAlpha)
            }
            ParticleBlendMode::OneOne => (BlendFactor::One, BlendFactor::One),
            ParticleBlendMode::OneOneMinusSrcColor => (BlendFactor::One, BlendFactor::OneMinusSrc),
            ParticleBlendMode::OneMinusDstColorOne => (BlendFactor::OneMinusDst, BlendFactor::One),
            ParticleBlendMode::OneMinusSrcColorDstAlpha => {
                (BlendFactor::OneMinusSrc, BlendFactor::DstAlpha)
            }
            ParticleBlendMode::OneMinusSrcColorOneMinusSrcColor => {
                (BlendFactor::OneMinusSrc, BlendFactor::OneMinusSrc)
            }
            ParticleBlendMode::SrcColorOne => (BlendFactor::Src, BlendFactor::One),
            ParticleBlendMode::OneMinusSrcAlphaOne => {
                (BlendFactor::OneMinusSrcAlpha, BlendFactor::One)
            }
            ParticleBlendMode::SrcAlphaOne => (BlendFactor::SrcAlpha, BlendFactor::One),
            ParticleBlendMode::SrcAlphaOneMinusSrcColor => {
                (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrc)
            }
            ParticleBlendMode::DstAlphaOneMinusSrcColor => {
                (BlendFactor::DstAlpha, BlendFactor::OneMinusSrc)
            }
        };
        let additive = BlendComponent {
            src_factor,
            dst_factor,
            operation: BlendOperation::Add,
        };
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState {
                    color: additive,
                    alpha: additive,
                });
                target.write_mask = ColorWrites::RED | ColorWrites::GREEN | ColorWrites::BLUE;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ParticleMaterialCacheKey {
    pub(super) texture: (String, i64, String),
    pub(super) blend_mode: u8,
    pub(super) tint: [u32; 4],
    pub(super) uv_scale_offset: [u32; 4],
    pub(super) legacy_gamma_accumulation_gain: u32,
}

/// Exact standalone material animation used by cutscenes that instantiate an
/// audited mesh effect outside `TutorialEffectRuntime`.
#[derive(Component, Clone, Debug)]
pub struct TutorialEffectMaterialAnimation {
    pub(super) age: f32,
    pub(super) plan: MaterialAnimationPlan,
    pub(super) paused: bool,
}

impl TutorialEffectMaterialAnimation {
    /// Keeps a standalone cutscene effect on its authored first material frame
    /// while the owning presentation barrier is still opaque.
    #[must_use]
    pub fn paused(mut self) -> Self {
        self.paused = true;
        self
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
}

pub(in super::super) fn compile_material_animation_component(
    effect_id: i32,
    effect: &ValidatedEffect,
) -> Result<TutorialEffectMaterialAnimation, TutorialNativeClosureBlockerReason> {
    let plan = compile_material_animation(effect_id, &effect.closure)?.ok_or_else(|| {
        invalid(format!(
            "effect {effect_id} has no exact standalone material animation"
        ))
    })?;
    Ok(TutorialEffectMaterialAnimation {
        age: 0.0,
        plan,
        paused: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParticleMaterialAnimationSample {
    pub(super) color_step: u8,
    pub(super) atlas_frame: u32,
}

#[derive(Component)]
pub(super) struct NativeAnimatedParticleMaterial {
    pub(super) sample: ParticleMaterialAnimationSample,
}

pub(super) fn mesh_effect_render_queue_sort_bias(source_queue: i32) -> f32 {
    crate::legacy_model_material::legacy_render_queue_sort_bias(source_queue)
}

pub(super) fn apply_material_animation(
    root_entity: Entity,
    animation: &MaterialAnimationPlan,
    animation_time: f32,
    children: &Query<&Children>,
    surfaces: &mut Query<(Entity, &Name, &mut MeshMaterial3d<LegacyModelMaterial>)>,
    materials: &mut Assets<LegacyModelMaterial>,
    changed: &mut BTreeSet<AssetId<LegacyModelMaterial>>,
) {
    for curve in &animation.curves {
        for entity in children.iter_descendants(root_entity) {
            let Ok((_, name, mut handle)) = surfaces.get_mut(entity) else {
                continue;
            };
            if !material_surface_matches_node(name.as_str(), &curve.node_name) {
                continue;
            }
            crate::legacy_model_material::make_legacy_material_unique(&mut handle.0, materials);
            let Some(material) = materials.get_mut_untracked(&handle.0) else {
                continue;
            };
            let before = material.uniform;
            apply_material_curve(
                &mut material.uniform,
                curve.property,
                curve.curve.evaluate(animation_time, 0.0),
            );
            if material.uniform != before {
                changed.insert(handle.0.id());
            }
        }
    }
}

pub(super) fn material_surface_matches_node(surface_name: &str, node_name: &str) -> bool {
    surface_name == node_name
        || surface_name
            .strip_prefix(node_name)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

pub(super) fn apply_material_curve(
    uniform: &mut crate::legacy_model_material::LegacyModelMaterialUniform,
    property: MaterialAnimatedProperty,
    value: f32,
) {
    match property {
        MaterialAnimatedProperty::BaseRed => uniform.base_color.red = value,
        MaterialAnimatedProperty::BaseGreen => uniform.base_color.green = value,
        MaterialAnimatedProperty::BaseBlue => uniform.base_color.blue = value,
        MaterialAnimatedProperty::BaseAlpha => uniform.base_color.alpha = value,
        MaterialAnimatedProperty::AmbientRed => uniform.ambient_color.red = value,
        MaterialAnimatedProperty::AmbientGreen => uniform.ambient_color.green = value,
        MaterialAnimatedProperty::AmbientBlue => uniform.ambient_color.blue = value,
        MaterialAnimatedProperty::AmbientAlpha => uniform.ambient_color.alpha = value,
        MaterialAnimatedProperty::EmissionRed => uniform.emission.red = value,
        MaterialAnimatedProperty::EmissionGreen => uniform.emission.green = value,
        MaterialAnimatedProperty::EmissionBlue => uniform.emission.blue = value,
        MaterialAnimatedProperty::EmissionAlpha => uniform.emission.alpha = value,
        MaterialAnimatedProperty::UvScaleX => uniform.uv_scale_offset.x = value,
        MaterialAnimatedProperty::UvScaleY => uniform.uv_scale_offset.y = value,
        MaterialAnimatedProperty::UvOffsetX => uniform.uv_scale_offset.z = value,
        MaterialAnimatedProperty::UvOffsetY => uniform.uv_scale_offset.w = value,
        MaterialAnimatedProperty::UvRotationDegrees => uniform.uv_pivot_rotation.z = value,
    }
}

pub(super) fn shared_trail_material(
    plan: &TrailPlan,
    images: &mut Assets<Image>,
    materials: &mut Assets<TutorialParticleMaterial>,
    visual_assets: &mut NativeVisualAssets,
) -> Handle<TutorialParticleMaterial> {
    let uniform = TutorialParticleUniform {
        tint: plan.material_tint,
        uv_scale_offset: Vec4::new(1.0, 1.0, 0.0, 0.0),
        legacy_gamma_accumulation_gain: 1.0,
    };
    let key = ParticleMaterialCacheKey {
        texture: plan.texture.key.clone(),
        blend_mode: plan.blend_mode as u8,
        tint: uniform.tint.to_array().map(f32::to_bits),
        uv_scale_offset: uniform.uv_scale_offset.to_array().map(f32::to_bits),
        legacy_gamma_accumulation_gain: uniform.legacy_gamma_accumulation_gain.to_bits(),
    };
    if let Some(material) = visual_assets.particle_materials.get(&key) {
        return material.clone();
    }
    let texture = texture_handle(&plan.texture, images, visual_assets);
    let material = materials.add(TutorialParticleMaterial {
        uniform,
        texture,
        blend_mode: plan.blend_mode,
    });
    visual_assets
        .particle_materials
        .insert(key, material.clone());
    material
}

pub(super) fn particle_material_is_animated(plan: &EmitterPlan) -> bool {
    let tiles = plan.uv_tiles.max(UVec2::ONE);
    plan.animate_color
        || (tiles.x.saturating_mul(tiles.y) > 1 && plan.uv_cycles.abs() > f32::EPSILON)
}

pub(super) fn shared_particle_material(
    plan: &EmitterPlan,
    normalized: f32,
    images: &mut Assets<Image>,
    materials: &mut Assets<TutorialParticleMaterial>,
    visual_assets: &mut NativeVisualAssets,
) -> Handle<TutorialParticleMaterial> {
    // A material per particle prevents Bevy from batching otherwise identical
    // quads. Large open-world effects (notably Firepits) can keep more than ten
    // thousand particles alive, turning that old path into ten thousand draw
    // items. Preserve exact atlas cells and quantize only the continuous legacy
    // colour ramp so particles can share a bounded set of render materials.
    let color_sample = if plan.animate_color {
        let last_step = PARTICLE_COLOR_ANIMATION_STEPS - 1.0;
        (normalized.clamp(0.0, 1.0) * last_step).round() / last_step
    } else {
        0.0
    };
    let uniform = TutorialParticleUniform {
        tint: plan.material_tint * particle_color(plan, color_sample),
        uv_scale_offset: particle_uv_scale_offset(plan, normalized),
        legacy_gamma_accumulation_gain: plan.legacy_gamma_accumulation_gain,
    };
    let key = ParticleMaterialCacheKey {
        texture: plan.texture.key.clone(),
        blend_mode: plan.blend_mode as u8,
        tint: uniform.tint.to_array().map(f32::to_bits),
        uv_scale_offset: uniform.uv_scale_offset.to_array().map(f32::to_bits),
        legacy_gamma_accumulation_gain: uniform.legacy_gamma_accumulation_gain.to_bits(),
    };
    if let Some(material) = visual_assets.particle_materials.get(&key) {
        return material.clone();
    }
    let texture = texture_handle(&plan.texture, images, visual_assets);
    let material = materials.add(TutorialParticleMaterial {
        uniform,
        texture,
        blend_mode: plan.blend_mode,
    });
    visual_assets
        .particle_materials
        .insert(key, material.clone());
    material
}

pub(super) fn particle_material_animation_sample(
    plan: &EmitterPlan,
    normalized: f32,
) -> ParticleMaterialAnimationSample {
    let normalized = normalized.clamp(0.0, 1.0);
    let color_step = if plan.animate_color {
        (normalized * (PARTICLE_COLOR_ANIMATION_STEPS - 1.0)).round() as u8
    } else {
        0
    };
    let tiles = plan.uv_tiles.max(UVec2::ONE);
    let frame_count = tiles.x.saturating_mul(tiles.y).max(1);
    let atlas_frame =
        ((normalized * plan.uv_cycles * frame_count as f32).floor() as u32) % frame_count;
    ParticleMaterialAnimationSample {
        color_step,
        atlas_frame,
    }
}
