use super::*;

pub(super) fn validate_materials_and_textures(model: &NativeModel) -> Result<()> {
    let mut used_textures = BTreeSet::new();
    for (material_index, material) in model.materials.iter().enumerate() {
        validate_true_asset_name("material", &material.name)?;
        valid_name("serialized legacy shader", &material.serialized_shader_name)?;
        valid_name("declared legacy shader", &material.declared_shader_name)?;
        valid_name("legacy shader", &material.legacy_shader_name)?;
        if material.legacy_shader_name != material.declared_shader_name {
            return invalid(format!(
                "material {material_index} runtime shader identity contradicts declaredShaderName"
            ));
        }
        if !material.standard_texture_refs_are_loader_hints {
            return invalid(format!(
                "material {material_index} does not mark standard texture references as Bevy loader hints"
            ));
        }
        if has_generated_identity(&material.legacy_shader_name) {
            return invalid(format!(
                "material {material_index} shader name is a hash/PathID fallback"
            ));
        }
        if has_generated_identity(&material.serialized_shader_name)
            || has_generated_identity(&material.declared_shader_name)
        {
            return invalid(format!(
                "material {material_index} exact shader identity is a hash/PathID fallback"
            ));
        }
        if !(0..=5_000).contains(&material.render_queue) {
            return invalid(format!(
                "material {material_index} has an unresolved/invalid render queue"
            ));
        }

        let mut colors = BTreeSet::new();
        for property in &material.colors {
            valid_name("material color property", &property.name)?;
            if !colors.insert(property.name.as_str()) {
                return invalid(format!(
                    "material {material_index} has duplicate color property {:?}",
                    property.name
                ));
            }
            finite("material color", &property.value)?;
        }
        let mut floats = BTreeSet::new();
        for property in &material.floats {
            valid_name("material float property", &property.name)?;
            if !floats.insert(property.name.as_str()) {
                return invalid(format!(
                    "material {material_index} has duplicate float property {:?}",
                    property.name
                ));
            }
            finite("material float", [&property.value])?;
        }
        let mut shader_texture_defaults = BTreeSet::new();
        for property in &material.shader_texture_defaults {
            valid_name("ShaderLab texture property", &property.slot)?;
            if !shader_texture_defaults.insert(property.slot.as_str()) {
                return invalid(format!(
                    "material {material_index} has duplicate ShaderLab texture default {:?}",
                    property.slot
                ));
            }
        }
        let mut slots = BTreeSet::new();
        for binding in &material.texture_bindings {
            if binding.unassigned_stale_null {
                // Two published shapes are unassigned and both bind no texture: a
                // nameless stale-null property, and a named property whose source
                // PPtr named no object in the bundle it was authored in. Neither
                // may carry a texture payload; a named one still owns its slot
                // name, so it must not collide with an assigned binding.
                if binding.ignored_stale_shader_binding
                    || binding.texture.is_some()
                    || binding.dynamic_texture.is_some()
                    || binding.source_name.is_some()
                    || binding.uri.is_some()
                    || binding.sampler.is_some()
                    || binding.mip_provenance.is_some()
                    || binding.mip_levels.is_some()
                {
                    return invalid(format!(
                        "material {material_index} unassigned texture property is contradictory"
                    ));
                }
                if !binding.slot.is_empty() {
                    valid_name("material texture slot", &binding.slot)?;
                    if !slots.insert(binding.slot.as_str()) {
                        return invalid(format!(
                            "material {material_index} has duplicate texture slot {:?}",
                            binding.slot
                        ));
                    }
                }
                finite(
                    "material stale-null texture transform",
                    binding
                        .scale
                        .iter()
                        .chain(&binding.offset)
                        .chain(binding.pivot.iter().flatten())
                        .chain(binding.rotation.iter()),
                )?;
                continue;
            }
            valid_name("material texture slot", &binding.slot)?;
            if !slots.insert(binding.slot.as_str()) {
                return invalid(format!(
                    "material {material_index} has duplicate texture slot {:?}",
                    binding.slot
                ));
            }
            if binding.texture.is_some() && binding.dynamic_texture.is_some() {
                return invalid(format!(
                    "material {material_index} texture slot {:?} references both Texture2D and dynamic media",
                    binding.slot
                ));
            }
            let has_assigned_source =
                binding.texture.is_some() || binding.dynamic_texture.is_some();
            if binding.ignored_stale_shader_binding {
                if shader_texture_defaults.contains(binding.slot.as_str()) || !has_assigned_source {
                    return invalid(format!(
                        "material {material_index} ignored stale texture binding is contradictory"
                    ));
                }
            } else if !shader_texture_defaults.contains(binding.slot.as_str())
                && has_assigned_source
            {
                return invalid(format!(
                    "material {material_index} assigned texture slot {:?} has no exact ShaderLab default",
                    binding.slot
                ));
            }
            finite(
                "material texture transform",
                binding
                    .scale
                    .iter()
                    .chain(&binding.offset)
                    .chain(binding.pivot.iter().flatten())
                    .chain(binding.rotation.iter()),
            )?;
            if let Some(texture) = binding.texture {
                match binding.slot.as_str() {
                    "_MainTex" if binding.color_space != crate::TextureColorSpace::Srgb => {
                        return invalid(format!(
                            "material {material_index} _MainTex must carry exact sRGB color space"
                        ));
                    }
                    "_BumpMap" | "_ShaderMap"
                        if binding.color_space != crate::TextureColorSpace::Linear =>
                    {
                        return invalid(format!(
                            "material {material_index} slot {:?} must carry exact linear color space",
                            binding.slot
                        ));
                    }
                    _ => {}
                }
                let Some(native_texture) = model.textures.get(texture as usize) else {
                    return invalid(format!(
                        "material {material_index} texture index is out of bounds"
                    ));
                };
                let Some(source_name) = binding.source_name.as_deref() else {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} has no exact sourceName",
                        binding.slot
                    ));
                };
                let Some(uri) = binding.uri.as_deref() else {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} has no exact URI",
                        binding.slot
                    ));
                };
                let Some(sampler) = binding.sampler.as_ref() else {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} has no exact sampler",
                        binding.slot
                    ));
                };
                let Some(mip_provenance) = binding.mip_provenance.as_ref() else {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} has no exact mip provenance",
                        binding.slot
                    ));
                };
                let Some(mip_levels) = binding.mip_levels.as_ref() else {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} has no exact mip levels",
                        binding.slot
                    ));
                };
                if source_name != native_texture.source_name || uri != native_texture.uri {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} sourceName/URI contradict texture {texture}",
                        binding.slot
                    ));
                }
                let Some(native_sampler) = model.samplers.get(native_texture.sampler as usize)
                else {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} references an invalid texture sampler",
                        binding.slot
                    ));
                };
                if sampler.index != native_texture.sampler || sampler.descriptor != *native_sampler
                {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} sampler contradicts texture {texture}",
                        binding.slot
                    ));
                }
                if mip_provenance != &native_texture.mip_provenance
                    || mip_levels != &native_texture.mip_levels
                {
                    return invalid(format!(
                        "material {material_index} assigned texture slot {:?} mip metadata contradicts texture {texture}",
                        binding.slot
                    ));
                }
                used_textures.insert(texture);
            } else if binding.source_name.is_some()
                || binding.uri.is_some()
                || binding.sampler.is_some()
                || binding.mip_provenance.is_some()
                || binding.mip_levels.is_some()
            {
                return invalid(format!(
                    "material {material_index} null texture slot {:?} carries sourceName/URI/sampler/mip metadata",
                    binding.slot
                ));
            }
            if let Some(dynamic) = &binding.dynamic_texture {
                if dynamic.source_id.trim().is_empty()
                    || dynamic.source_asset_index > usize::MAX as u64
                    || dynamic.source_path_id == 0
                    || dynamic.source_name.trim().is_empty()
                    || dynamic.object_type != "MovieTexture"
                    || dynamic.container != "ogg-theora"
                    || dynamic.mime_type != "video/ogg"
                    || dynamic.byte_length == 0
                    || !is_sha256(&dynamic.sha256)
                    || dynamic.audio_clip_pointer.is_null
                        != (dynamic.audio_clip_pointer.path_id == 0)
                {
                    return invalid(format!(
                        "material {material_index} dynamic texture slot {:?} has contradictory MovieTexture metadata",
                        binding.slot
                    ));
                }
                const PREFIX: &str = "data:video/ogg;base64,";
                let encoded = dynamic.data_url.strip_prefix(PREFIX).ok_or_else(|| {
                    crate::ModelError::Invalid(format!(
                        "material {material_index} dynamic texture is not a strict video/ogg data URL"
                    ))
                })?;
                if encoded.is_empty() || encoded.bytes().any(|byte| byte.is_ascii_whitespace()) {
                    return invalid(format!(
                        "material {material_index} dynamic texture base64 is empty or contains whitespace"
                    ));
                }
                let bytes = BASE64_STANDARD.decode(encoded).map_err(|error| {
                    crate::ModelError::Invalid(format!(
                        "material {material_index} dynamic texture base64 is invalid: {error}"
                    ))
                })?;
                if u64::try_from(bytes.len()).ok() != Some(dynamic.byte_length)
                    || format!("{:x}", Sha256::digest(&bytes)) != dynamic.sha256
                    || !bytes.starts_with(b"OggS")
                {
                    return invalid(format!(
                        "material {material_index} dynamic texture bytes contradict length/SHA/Ogg container"
                    ));
                }
            }
        }
        if material.passes.is_empty() {
            return invalid(format!(
                "material {material_index} has no resolved render passes"
            ));
        }
        for (pass_index, pass) in material.passes.iter().enumerate() {
            if let Some(name) = &pass.name {
                valid_name("material pass", name)?;
            }
            if pass.color_mask > 0x0f {
                return invalid(format!(
                    "material {material_index} pass {pass_index} color mask is invalid"
                ));
            }
            if !pass.blend.enabled
                && (pass.blend.source_color != MaterialBlendFactor::One
                    || pass.blend.destination_color != MaterialBlendFactor::Zero
                    || pass.blend.color_operation != MaterialBlendOperation::Add
                    || pass.blend.source_alpha != MaterialBlendFactor::One
                    || pass.blend.destination_alpha != MaterialBlendFactor::Zero
                    || pass.blend.alpha_operation != MaterialBlendOperation::Add)
            {
                return invalid(format!(
                    "material {material_index} pass {pass_index} disabled blend state is contradictory"
                ));
            }
            match &pass.alpha_test {
                MaterialAlphaTestState::Disabled => {}
                MaterialAlphaTestState::Enabled { compare, reference } => {
                    if *compare == MaterialCompareFunction::Disabled {
                        return invalid(format!(
                            "material {material_index} pass {pass_index} alpha test has no compare function"
                        ));
                    }
                    match reference {
                        MaterialAlphaReference::Literal { value } => {
                            finite("material alpha-test reference", [value])?;
                        }
                        MaterialAlphaReference::FloatProperty {
                            name,
                            resolved_value,
                        } => {
                            valid_name("material alpha-test property", name)?;
                            finite("material alpha-test reference", [resolved_value])?;
                            if let Some(property) = material
                                .floats
                                .iter()
                                .find(|property| property.name == *name)
                                && property.value != *resolved_value
                            {
                                return invalid(format!(
                                    "material {material_index} pass {pass_index} alpha-test resolved value contradicts material property"
                                ));
                            }
                        }
                    }
                }
            }
            match &pass.outline {
                MaterialOutlineState::Disabled => {}
                MaterialOutlineState::WorldSpace { width, color }
                | MaterialOutlineState::ScreenSpace { width, color } => {
                    finite("material outline", std::iter::once(width).chain(color))?;
                    if *width < 0.0 {
                        return invalid(format!(
                            "material {material_index} pass {pass_index} outline width is invalid"
                        ));
                    }
                }
            }
        }
    }

    let mut texture_names = BTreeSet::new();
    let mut texture_uris = BTreeSet::new();
    let mut used_samplers = BTreeSet::new();
    let mut preferred_name_counts = BTreeMap::<String, usize>::new();
    for texture in &model.textures {
        let preferred = minimal_windows_png_filename(&texture.source_name)?;
        *preferred_name_counts
            .entry(preferred.to_ascii_lowercase())
            .or_default() += 1;
    }
    for (texture_index, texture) in model.textures.iter().enumerate() {
        validate_true_asset_name("texture", &texture.source_name)?;
        if !texture_names.insert(texture.source_name.to_lowercase()) {
            return invalid("texture table contains case-insensitive duplicate source m_Names");
        }
        let preferred = minimal_windows_png_filename(&texture.source_name)?;
        let preserves_extension = preferred_name_counts
            .get(&preferred.to_ascii_lowercase())
            .copied()
            .unwrap_or_default()
            > 1;
        validate_texture_uri(&texture.source_name, &texture.uri, preserves_extension)?;
        if !texture_uris.insert(texture.uri.to_lowercase()) {
            return invalid("texture table contains case-insensitive duplicate PNG URIs");
        }
        if texture.width == 0 || texture.height == 0 {
            return invalid(format!("texture {texture_index} has zero dimensions"));
        }
        if texture.sampler as usize >= model.samplers.len() {
            return invalid(format!("texture {texture_index} sampler is out of bounds"));
        }
        used_samplers.insert(texture.sampler);
        let max_mips = 32 - texture.width.max(texture.height).leading_zeros();
        if texture.mip_provenance.source_mip_count == 0
            || texture.mip_provenance.source_mip_count > max_mips
        {
            return invalid(format!(
                "texture {texture_index} source mip count is impossible for its dimensions"
            ));
        }
        if !texture.mip_provenance.source_chain_complete {
            return invalid(format!(
                "texture {texture_index} source mip chain is incomplete"
            ));
        }
        let expected_format_name = texture_format_name(
            texture.mip_provenance.source_texture_format,
        )
        .ok_or_else(|| {
            crate::ModelError::Invalid(format!(
                "texture {texture_index} source TextureFormat is unsupported for exact mip publication"
            ))
        })?;
        if texture.mip_provenance.source_texture_format_name != expected_format_name {
            return invalid(format!(
                "texture {texture_index} source TextureFormat name contradicts its numeric value"
            ));
        }
        if texture.mip_provenance.source_chain_byte_length == 0
            || !is_sha256(&texture.mip_provenance.source_chain_sha256)
        {
            return invalid(format!(
                "texture {texture_index} source chain length/SHA-256 provenance is invalid"
            ));
        }
        if texture.mip_provenance.source_layout
            != crate::TextureSourceMipLayout::LargestToSmallestContiguous
        {
            return invalid(format!(
                "texture {texture_index} source mip layout is unsupported"
            ));
        }
        if texture.mip_levels.len() != texture.mip_provenance.source_mip_count as usize {
            return invalid(format!(
                "texture {texture_index} does not publish every source mip level"
            ));
        }
        let base_png = texture
            .uri
            .rsplit('/')
            .next()
            .expect("validated texture URI has a basename");
        let mip_stem = base_png
            .strip_suffix(".png")
            .expect("minimal PNG filename always ends in .png");
        let parent = texture.uri.rsplit_once('/').map(|(parent, _)| parent);
        let mut expected_width = texture.width;
        let mut expected_height = texture.height;
        let mut expected_source_offset = 0u64;
        for (level_index, level) in texture.mip_levels.iter().enumerate() {
            let level_number = u32::try_from(level_index)
                .map_err(|_| crate::ModelError::Invalid("mip level count overflow".into()))?;
            if level.level != level_number
                || level.width != expected_width
                || level.height != expected_height
                || level.source_byte_offset != expected_source_offset
            {
                return invalid(format!(
                    "texture {texture_index} mip {level_index} order/dimensions/source offset are not exact"
                ));
            }
            let expected_source_length = texture_mip_byte_length(
                texture.mip_provenance.source_texture_format,
                level.width,
                level.height,
            )
            .ok_or_else(|| {
                crate::ModelError::Invalid(format!(
                    "texture {texture_index} mip {level_index} format has no exact byte layout"
                ))
            })?;
            if level.source_byte_length != expected_source_length
                || !is_sha256(&level.source_byte_sha256)
            {
                return invalid(format!(
                    "texture {texture_index} mip {level_index} source length/SHA-256 is invalid"
                ));
            }
            let expected_rgba_length = u64::from(level.width)
                .checked_mul(u64::from(level.height))
                .and_then(|pixels| pixels.checked_mul(4))
                .ok_or_else(|| {
                    crate::ModelError::Invalid("decoded mip RGBA byte count overflow".into())
                })?;
            if level.decoded_rgba8_byte_length != expected_rgba_length
                || !is_sha256(&level.decoded_rgba8_sha256)
                || level.png_byte_length == 0
                || !is_sha256(&level.png_sha256)
            {
                return invalid(format!(
                    "texture {texture_index} mip {level_index} decoded PNG provenance is invalid"
                ));
            }
            let expected_uri = if level_index == 0 {
                texture.uri.clone()
            } else if let Some(parent) = parent {
                format!("{parent}/{mip_stem}.mips/mip-{level_index:02}.png")
            } else {
                format!("{mip_stem}.mips/mip-{level_index:02}.png")
            };
            if level.uri != expected_uri {
                return invalid(format!(
                    "texture {texture_index} mip {level_index} URI must be {expected_uri:?}"
                ));
            }
            if level_index > 0 && !texture_uris.insert(level.uri.to_ascii_lowercase()) {
                return invalid("texture table contains case-insensitive duplicate mip PNG URIs");
            }
            expected_source_offset = expected_source_offset
                .checked_add(level.source_byte_length)
                .ok_or_else(|| {
                    crate::ModelError::Invalid("source mip chain byte count overflow".into())
                })?;
            expected_width = (expected_width / 2).max(1);
            expected_height = (expected_height / 2).max(1);
        }
        if expected_source_offset != texture.mip_provenance.source_chain_byte_length {
            return invalid(format!(
                "texture {texture_index} mip slices do not cover the complete source chain"
            ));
        }
        let source_has_mips = texture.mip_provenance.source_mip_count > 1;
        match (source_has_mips, texture.mip_provenance.published_policy) {
            (true, PublishedMipPolicy::ExactSourceLevels)
            | (false, PublishedMipPolicy::BaseLevelOnly) => {}
            (true, PublishedMipPolicy::BaseLevelOnly) => {
                return invalid(format!(
                    "texture {texture_index} would discard source mip behavior"
                ));
            }
            (false, PublishedMipPolicy::ExactSourceLevels) => {
                return invalid(format!(
                    "texture {texture_index} requests a multi-level policy for a base-only source"
                ));
            }
        }
        validate_sampler_mapping(
            texture_index,
            &model.samplers[texture.sampler as usize],
            source_has_mips,
        )?;
    }
    if used_textures.len() != model.textures.len() {
        return invalid("texture table contains unreferenced extra textures");
    }

    for (sampler_index, sampler) in model.samplers.iter().enumerate() {
        validate_true_asset_name("sampler", &sampler.name)?;
        if !(0..=2).contains(&sampler.legacy_filter_mode) {
            return invalid(format!(
                "sampler {sampler_index} has unknown legacy filter mode"
            ));
        }
        if !(0..=1).contains(&sampler.legacy_wrap_mode) {
            return invalid(format!(
                "sampler {sampler_index} has unknown legacy wrap mode"
            ));
        }
        if sampler.anisotropy_level > 16 {
            return invalid(format!(
                "sampler {sampler_index} anisotropy level is invalid"
            ));
        }
        finite("sampler mip bias", [&sampler.mip_map_bias])?;
    }
    if used_samplers.len() != model.samplers.len() {
        return invalid("sampler table contains unreferenced extra samplers");
    }
    Ok(())
}

pub(super) fn validate_sampler_mapping(
    texture_index: usize,
    sampler: &crate::NativeSampler,
    has_mips: bool,
) -> Result<()> {
    let expected_mag = if sampler.legacy_filter_mode == 0 {
        SamplerMagFilter::Nearest
    } else {
        SamplerMagFilter::Linear
    };
    let expected_min = match (sampler.legacy_filter_mode, has_mips) {
        (0, false) => SamplerMinFilter::Nearest,
        (0, true) => SamplerMinFilter::NearestMipmapNearest,
        (1, false) | (2, false) => SamplerMinFilter::Linear,
        (1, true) => SamplerMinFilter::LinearMipmapNearest,
        (2, true) => SamplerMinFilter::LinearMipmapLinear,
        _ => {
            return invalid(format!(
                "texture {texture_index} sampler filter mode is unknown"
            ));
        }
    };
    let expected_wrap = match sampler.legacy_wrap_mode {
        0 => SamplerWrapMode::Repeat,
        1 => SamplerWrapMode::ClampToEdge,
        _ => {
            return invalid(format!(
                "texture {texture_index} sampler wrap mode is unknown"
            ));
        }
    };
    if sampler.mag_filter != expected_mag
        || sampler.min_filter != expected_min
        || sampler.wrap_s != expected_wrap
        || sampler.wrap_t != expected_wrap
    {
        return invalid(format!(
            "texture {texture_index} standard sampler state contradicts legacy provenance"
        ));
    }
    Ok(())
}
