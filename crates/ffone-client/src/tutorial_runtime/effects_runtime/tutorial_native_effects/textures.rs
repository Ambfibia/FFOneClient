use super::*;

pub(super) const RETROBUTION_SWORD_TRAIL_TEXTURE: &str = "map/shared/effects/textures/swordtail.png";

#[derive(Clone, Debug)]
pub(super) struct ExactTexture {
    pub(super) key: (String, i64, String),
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) mip_count: u32,
    pub(super) rgba_mips: Arc<[u8]>,
}

pub(super) fn material_and_texture(
    closure: &TutorialEffectClosureFile,
    renderer: &TutorialUnityObjectProof,
) -> Result<(ParticleBlendMode, Vec4, ExactTexture), TutorialNativeClosureBlockerReason> {
    let materials = array(&renderer.value, "m_Materials").map_err(invalid)?;
    if materials.len() != 1 {
        return Err(invalid(
            "particle renderer does not have exactly one material",
        ));
    }
    let material_path = materials[0]
        .get("pathId")
        .and_then(JsonValue::as_i64)
        .ok_or_else(|| invalid("material reference has no pathId"))?;
    let material = unique_path_type(closure, material_path, "Material").map_err(invalid)?;
    let shader_path = path_id(&material.value, "m_Shader").map_err(invalid)?;
    let shader = unique_path_type(closure, shader_path, "Shader").map_err(invalid)?;
    let shader_name = exact_shader_name(shader).map_err(invalid)?;
    let Some(blend_mode) = particle_blend_mode(&shader_name) else {
        return Err(TutorialNativeClosureBlockerReason::UnsupportedParticleShader { shader_name });
    };
    let properties = object(&material.value, "m_SavedProperties").map_err(invalid)?;
    let tint = named_color(properties, "_TintColor").map_err(invalid)?;
    let texture_path = named_texture_path(properties, "_MainTex").map_err(invalid)?;
    let texture = unique_path_type(closure, texture_path, "Texture2D").map_err(invalid)?;
    Ok((
        blend_mode,
        tint,
        decode_exact_texture(texture).map_err(invalid)?,
    ))
}

pub(super) fn named_texture_path(properties: &JsonValue, name: &str) -> Result<i64, String> {
    for pair in array(properties, "m_TexEnvs")? {
        let pair = pair
            .as_array()
            .ok_or_else(|| "m_TexEnvs entry is not a pair".to_owned())?;
        if pair.len() == 2 && pair[0].get("name").and_then(JsonValue::as_str) == Some(name) {
            if vector2(&pair[1], "m_Scale")? != Vec2::ONE
                || vector2(&pair[1], "m_Offset")? != Vec2::ZERO
                || number(&pair[1], "m_Rotation")? != 0.0
            {
                return Err("particle MainTex transform is not identity".to_owned());
            }
            return path_id(&pair[1], "m_Texture");
        }
    }
    Err(format!("m_TexEnvs has no exact {name} entry"))
}

pub(super) fn texture_format_layout(texture_format: i64) -> Result<(&'static str, Option<usize>), String> {
    match texture_format {
        2 => Ok(("ARGB4444", None)),
        10 => Ok(("DXT1", Some(8))),
        12 => Ok(("DXT5", Some(16))),
        _ => Err(format!(
            "particle texture uses unsupported TextureFormat {texture_format}"
        )),
    }
}

pub(super) fn rgba_level_size(width: u32, height: u32) -> Result<usize, String> {
    if width == 0 || height == 0 {
        return Err("texture level has a zero dimension".to_owned());
    }
    let width =
        usize::try_from(width).map_err(|_| "texture width does not fit usize".to_owned())?;
    let height =
        usize::try_from(height).map_err(|_| "texture height does not fit usize".to_owned())?;
    width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "decoded texture level byte count overflows".to_owned())
}

pub(super) fn bc1_color_palette(c0: u16, c1: u16) -> [[u8; 4]; 4] {
    let expand = |value: u16| {
        [
            ((((value >> 11) & 31) * 255) / 31) as u8,
            ((((value >> 5) & 63) * 255) / 63) as u8,
            (((value & 31) * 255) / 31) as u8,
            255,
        ]
    };
    let (a, b) = (expand(c0), expand(c1));
    if c0 > c1 {
        let mix = |left: [u8; 4], right: [u8; 4], lw: u16, rw: u16| {
            std::array::from_fn(|index| {
                ((u16::from(left[index]) * lw + u16::from(right[index]) * rw) / 3) as u8
            })
        };
        [a, b, mix(a, b, 2, 1), mix(a, b, 1, 2)]
    } else {
        let average =
            std::array::from_fn(|index| ((u16::from(a[index]) + u16::from(b[index])) / 2) as u8);
        [a, b, average, [0, 0, 0, 0]]
    }
}

pub(super) fn bc3_alpha_palette(a0: u8, a1: u8) -> [u8; 8] {
    let mut result = [a0, a1, 0, 0, 0, 0, 0, 0];
    if a0 > a1 {
        for index in 1..=6_u16 {
            result[index as usize + 1] =
                (((7 - index) * u16::from(a0) + index * u16::from(a1)) / 7) as u8;
        }
    } else {
        for index in 1..=4_u16 {
            result[index as usize + 1] =
                (((5 - index) * u16::from(a0) + index * u16::from(a1)) / 5) as u8;
        }
        result[6] = 0;
        result[7] = 255;
    }
    result
}

pub(super) fn bc3_color_palette(c0: u16, c1: u16) -> [[u8; 3]; 4] {
    let expand = |value: u16| {
        [
            ((((value >> 11) & 31) * 255) / 31) as u8,
            ((((value >> 5) & 63) * 255) / 63) as u8,
            (((value & 31) * 255) / 31) as u8,
        ]
    };
    let (a, b) = (expand(c0), expand(c1));
    let mix = |left: [u8; 3], right: [u8; 3], lw: u16, rw: u16| {
        std::array::from_fn(|index| {
            ((u16::from(left[index]) * lw + u16::from(right[index]) * rw) / 3) as u8
        })
    };
    [a, b, mix(a, b, 2, 1), mix(a, b, 1, 2)]
}

pub(super) fn flip_rgba_rows(bytes: &mut [u8], width: u32, height: u32) {
    let row = width as usize * 4;
    for y in 0..height as usize / 2 {
        let opposite = height as usize - 1 - y;
        for x in 0..row {
            bytes.swap(y * row + x, opposite * row + x);
        }
    }
}

pub(super) fn configure_native_sword_trail_texture(
    mut images: ResMut<Assets<Image>>,
    mut visual_assets: ResMut<NativeVisualAssets>,
) {
    if visual_assets.sword_trail_sampler_configured {
        return;
    }
    let Some(texture) = visual_assets.sword_trail_texture.as_ref() else {
        return;
    };
    let Some(mut image) = images.get_mut(texture) else {
        return;
    };
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    visual_assets.sword_trail_sampler_configured = true;
}

pub(super) fn texture_handle(
    texture: &ExactTexture,
    images: &mut Assets<Image>,
    visual_assets: &mut NativeVisualAssets,
) -> Handle<Image> {
    if let Some(handle) = visual_assets.textures.get(&texture.key) {
        return handle.clone();
    }
    // `Image::new` validates against its initial one-mip descriptor before we
    // can set `mip_level_count`, so a complete authored mip chain must be
    // installed through the uninitialized constructor.
    let mut image = Image::new_uninit(
        Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.mip_level_count = texture.mip_count;
    image.data = Some(texture.rgba_mips.to_vec());
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });
    let handle = images.add(image);
    visual_assets
        .textures
        .insert(texture.key.clone(), handle.clone());
    handle
}
