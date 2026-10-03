use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerifiedRgbaImage {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) pixels: Arc<[u8]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct VerifiedRgbaImageCacheKey {
    pub(super) path: PathBuf,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) png_blake3: String,
    pub(super) rgba_blake3: String,
}

#[derive(Debug, Default)]
pub(super) struct VerifiedRgbaImageCache {
    // Weak pixels make the cache share overlapping resident tiles without
    // retaining every terrain texture ever visited. NativeTerrain owns the
    // strong Arcs for exactly the resident/LRU terrain set.
    pub(super) entries: HashMap<VerifiedRgbaImageCacheKey, Weak<[u8]>>,
    // A descriptor often publishes byte-identical base and mip-0 PNGs at two
    // provenance paths. After each PNG path has passed its own encoded hash,
    // canonical RGBA identity safely avoids decoding/copying that payload a
    // second time.
    pub(super) payloads: HashMap<VerifiedRgbaPayloadCacheKey, Weak<[u8]>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct VerifiedSourceMipKey {
    pub(super) path: PathBuf,
    pub(super) byte_length: usize,
    pub(super) blake3: String,
}

pub(super) static VERIFIED_RGBA_IMAGE_CACHE: OnceLock<Mutex<VerifiedRgbaImageCache>> = OnceLock::new();

pub(super) static VERIFIED_SOURCE_MIP_CACHE: OnceLock<Mutex<HashSet<VerifiedSourceMipKey>>> = OnceLock::new();

pub(super) fn validate_texture_contract_metadata(
    sampler: Option<&NativeTerrainTextureSampler>,
    mips: &[NativeTerrainTextureMip],
    width: u32,
    height: u32,
    usage_color_space: &str,
    context: &str,
) -> Result<(), NativeTerrainError> {
    let Some(sampler) = sampler else {
        if !mips.is_empty() {
            return Err(NativeTerrainError::new(format!(
                "{context} has mip files without a source sampler contract"
            )));
        }
        return Ok(());
    };
    let expected_filter = match sampler.filter_mode.serialized_value {
        0 => "Point",
        1 => "Bilinear",
        2 => "Trilinear",
        _ => {
            return Err(NativeTerrainError::new(format!(
                "{context} has unsupported source filter mode"
            )));
        }
    };
    let expected_wrap = match sampler.wrap_mode.serialized_value {
        0 => "Repeat",
        1 => "Clamp",
        _ => {
            return Err(NativeTerrainError::new(format!(
                "{context} has unsupported source wrap mode"
            )));
        }
    };
    let maximum_dimension = width.max(height);
    if maximum_dimension == 0 {
        return Err(NativeTerrainError::new(format!(
            "{context} has zero-sized source dimensions"
        )));
    }
    let expected_mip_count = if sampler.mip_map {
        (u32::BITS - maximum_dimension.leading_zeros()) as usize
    } else {
        1
    };
    if sampler.source_schema != "Unity 2.5.5 Texture2D serialized type tree"
        || sampler.filter_mode.unity_name != expected_filter
        || sampler.wrap_mode.unity_name != expected_wrap
        || sampler.wrap_mode.u != expected_wrap
        || sampler.wrap_mode.v != expected_wrap
        || sampler.wrap_mode.axis_source != "legacy single m_WrapMode applies to both axes"
        || sampler.aniso_level < 0
        || !sampler.mip_bias.is_finite()
        || sampler.usage_color_space != usage_color_space
        || sampler.mip_count != mips.len()
        || sampler.mip_count != expected_mip_count
        || sampler.mip_count_source
            != "all serialized levels decoded from the contiguous Texture2D image payload"
        || sampler.color_space_flag.serialized_status != "notPresentInUnity2.5.5Texture2DTypeTree"
        || sampler.color_space_flag.runtime_interpretation_source != "terrain texture role"
        || sampler.alpha_is_transparency_flag.serialized_status
            != "notPresentInUnity2.5.5Texture2DTypeTree"
        || sampler.alpha_is_transparency_flag.runtime_contract
            != "preserve decoded RGBA alpha exactly"
        || mips.is_empty()
    {
        return Err(NativeTerrainError::new(format!(
            "{context} source sampler/mip contract is incomplete or contradictory"
        )));
    }
    let mut expected_width = width;
    let mut expected_height = height;
    let mut source_byte_length = Some(0_usize);
    for (level, mip) in mips.iter().enumerate() {
        validate_relative_path(&mip.path, "png")?;
        if mip.level != level
            || mip.width != expected_width
            || mip.height != expected_height
            || mip.png_color_type != "RGBA8"
            || mip.canonical_transform != "flipY"
        {
            return Err(NativeTerrainError::new(format!(
                "{context} mip {level} dimensions/order differ from the exact chain"
            )));
        }
        validate_prefixed_blake3(&mip.canonical_rgba_blake3, "texture mip RGBA")?;
        validate_prefixed_blake3(&mip.png_blake3, "texture mip PNG")?;
        if let Some(source) = &mip.source_encoded {
            validate_relative_path(&source.path, "bin")?;
            if source.byte_length == 0 {
                return Err(NativeTerrainError::new(format!(
                    "{context} mip {level} has an empty serialized source"
                )));
            }
            validate_prefixed_blake3(&source.blake3, "texture source mip")?;
            source_byte_length =
                source_byte_length.and_then(|total| total.checked_add(source.byte_length));
        } else {
            source_byte_length = None;
        }
        expected_width = (expected_width / 2).max(1);
        expected_height = (expected_height / 2).max(1);
    }
    if sampler.mip_map
        && mips
            .last()
            .is_none_or(|mip| mip.width != 1 || mip.height != 1)
    {
        return Err(NativeTerrainError::new(format!(
            "{context} mipmapped source chain does not end at 1x1"
        )));
    }
    if let (Some(complete_image_size), Some(source_byte_length)) = (
        sampler.serialized_auxiliary.complete_image_size,
        source_byte_length,
    ) {
        let complete_image_size = usize::try_from(complete_image_size).map_err(|_| {
            NativeTerrainError::new(format!(
                "{context} has a negative or unrepresentable complete image size"
            ))
        })?;
        if complete_image_size != source_byte_length {
            return Err(NativeTerrainError::new(format!(
                "{context} serialized mip byte total differs from m_CompleteImageSize"
            )));
        }
    }
    Ok(())
}

pub(super) fn load_rgba_image(
    terrain_root: &Path,
    relative: &str,
    expected_width: u32,
    expected_height: u32,
    expected_png_blake3: &str,
    expected_rgba_blake3: &str,
    context: &str,
) -> Result<VerifiedRgbaImage, NativeTerrainError> {
    let path = join_relative(terrain_root, relative);
    load_verified_rgba_image(
        path,
        expected_width,
        expected_height,
        expected_png_blake3,
        expected_rgba_blake3,
        context,
    )
}

pub(super) fn load_verified_rgba_image(
    path: PathBuf,
    expected_width: u32,
    expected_height: u32,
    expected_png_blake3: &str,
    expected_rgba_blake3: &str,
    context: &str,
) -> Result<VerifiedRgbaImage, NativeTerrainError> {
    let key = VerifiedRgbaImageCacheKey {
        path: path.clone(),
        width: expected_width,
        height: expected_height,
        png_blake3: expected_png_blake3.to_owned(),
        rgba_blake3: expected_rgba_blake3.to_owned(),
    };
    let cache = VERIFIED_RGBA_IMAGE_CACHE.get_or_init(Default::default);
    if let Some(pixels) = cache
        .lock()
        .map_err(|_| NativeTerrainError::new("verified terrain image cache mutex was poisoned"))?
        .entries
        .get(&key)
        .and_then(Weak::upgrade)
    {
        return Ok(VerifiedRgbaImage {
            width: expected_width,
            height: expected_height,
            pixels,
        });
    }

    let png = read_file(&path, context)?;
    verify_prefixed_hash(&png, expected_png_blake3, &path)?;
    expected_rgba_blake3
        .strip_prefix("blake3:")
        .unwrap_or(expected_rgba_blake3)
        .parse::<blake3::Hash>()
        .map_err(|error| {
            NativeTerrainError::new(format!(
                "invalid canonical RGBA BLAKE3 for {}: {error}",
                path.display()
            ))
        })?;
    let payload_key = VerifiedRgbaPayloadCacheKey {
        width: expected_width,
        height: expected_height,
        rgba_blake3: expected_rgba_blake3.to_owned(),
    };
    let cached_payload = {
        let cache = cache.lock().map_err(|_| {
            NativeTerrainError::new("verified terrain image cache mutex was poisoned")
        })?;
        cache.payloads.get(&payload_key).and_then(Weak::upgrade)
    };
    if let Some(pixels) = cached_payload {
        let image = VerifiedRgbaImage {
            width: expected_width,
            height: expected_height,
            pixels,
        };
        cache
            .lock()
            .map_err(|_| {
                NativeTerrainError::new("verified terrain image cache mutex was poisoned")
            })?
            .entries
            .insert(key, Arc::downgrade(&image.pixels));
        return Ok(image);
    }
    let decoded = decode_png(&png, &path)?;
    if decoded.color() != ColorType::Rgba8
        || decoded.width() != expected_width
        || decoded.height() != expected_height
    {
        return Err(NativeTerrainError::new(format!(
            "{} must be exact {expected_width}x{expected_height} RGBA8, got {}x{} {:?}",
            path.display(),
            decoded.width(),
            decoded.height(),
            decoded.color()
        )));
    }
    let pixels = decoded.into_rgba8().into_raw();
    verify_prefixed_hash(&pixels, expected_rgba_blake3, &path)?;
    let image = VerifiedRgbaImage {
        width: expected_width,
        height: expected_height,
        pixels: pixels.into(),
    };
    let mut cache = cache
        .lock()
        .map_err(|_| NativeTerrainError::new("verified terrain image cache mutex was poisoned"))?;
    // Remove expired weak entries while this uncommon disk/decode path is
    // already paying for a map mutation. This keeps a long play session from
    // accumulating path metadata for every visited dong.
    cache.entries.retain(|_, pixels| pixels.strong_count() > 0);
    cache.payloads.retain(|_, pixels| pixels.strong_count() > 0);
    cache.entries.insert(key, Arc::downgrade(&image.pixels));
    cache
        .payloads
        .insert(payload_key, Arc::downgrade(&image.pixels));
    Ok(image)
}

#[cfg(test)]
pub(super) fn verify_texture_mip_files(
    terrain_root: &Path,
    mips: &[NativeTerrainTextureMip],
    context: &str,
) -> Result<(), NativeTerrainError> {
    load_texture_mip_files(terrain_root, mips, context).map(|_| ())
}

pub(super) fn load_texture_mip_files(
    terrain_root: &Path,
    mips: &[NativeTerrainTextureMip],
    context: &str,
) -> Result<Vec<VerifiedRgbaImage>, NativeTerrainError> {
    let mut images = Vec::with_capacity(mips.len());
    for mip in mips {
        let path = join_relative(terrain_root, &mip.path);
        images.push(load_verified_rgba_image(
            path,
            mip.width,
            mip.height,
            &mip.png_blake3,
            &mip.canonical_rgba_blake3,
            context,
        )?);
        if let Some(source_encoded) = &mip.source_encoded {
            let source_path = join_relative(terrain_root, &source_encoded.path);
            let key = VerifiedSourceMipKey {
                path: source_path.clone(),
                byte_length: source_encoded.byte_length,
                blake3: source_encoded.blake3.clone(),
            };
            let cache = VERIFIED_SOURCE_MIP_CACHE.get_or_init(Default::default);
            let already_verified = cache
                .lock()
                .map_err(|_| {
                    NativeTerrainError::new("verified source-mip cache mutex was poisoned")
                })?
                .contains(&key);
            if !already_verified {
                let source = read_file(&source_path, "serialized texture mip bytes")?;
                if source.len() != source_encoded.byte_length {
                    return Err(NativeTerrainError::new(format!(
                        "{} byte length differs from terrain.json",
                        source_path.display()
                    )));
                }
                verify_prefixed_hash(&source, &source_encoded.blake3, &source_path)?;
                cache
                    .lock()
                    .map_err(|_| {
                        NativeTerrainError::new("verified source-mip cache mutex was poisoned")
                    })?
                    .insert(key);
            }
        }
    }
    Ok(images)
}
