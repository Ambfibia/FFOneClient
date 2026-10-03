use super::*;

// One request can materialize a root plus several emitter/mesh children. Count
// those rendered nodes instead of treating a five-emitter controller like a
// one-entity request. Gameplay/projectile requests consume the shared budget
// first; streamed ambient work has a stricter sub-budget so tile admission
// cannot monopolize the frame.
pub(super) const NATIVE_SPAWN_WORK_PER_FRAME: usize = 48;

pub(super) const STREAMED_WORLD_NATIVE_SPAWN_WORK_PER_FRAME: usize = 24;

pub(super) const STREAMED_WORLD_PARTICLE_DESPAWNS_PER_FRAME: usize = 32;

pub(super) fn decode_exact_texture(texture_object: &TutorialUnityObjectProof) -> Result<ExactTexture, String> {
    let value = &texture_object.value;
    let texture_format = integer(value, "m_TextureFormat")?;
    let (format_name, block_bytes) = texture_format_layout(texture_format)?;
    if integer(value, "m_TextureDimension")? != 2 {
        return Err("particle texture is not two-dimensional".to_owned());
    }
    let settings = object(value, "m_TextureSettings")?;
    if integer(settings, "m_FilterMode")? != 1
        || integer(settings, "m_WrapMode")? != 0
        || integer(settings, "m_Aniso")? != 1
        || number(settings, "m_MipBias")? != 0.0
    {
        return Err("particle texture sampler is not Linear/Repeat/aniso1/bias0".to_owned());
    }
    let width = u32::try_from(integer(value, "m_Width")?)
        .map_err(|_| "texture width does not fit u32".to_owned())?;
    let height = u32::try_from(integer(value, "m_Height")?)
        .map_err(|_| "texture height does not fit u32".to_owned())?;
    rgba_level_size(width, height)?;
    let data = object(value, "image data")?;
    let encoded = data
        .get("base64")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "image data.base64 is absent".to_owned())?;
    let compressed = decode_base64(encoded)?;
    let declared = usize::try_from(integer(data, "bytes")?)
        .map_err(|_| "image-data bytes do not fit usize".to_owned())?;
    if compressed.len() != declared
        || declared
            != usize::try_from(integer(value, "m_CompleteImageSize")?)
                .map_err(|_| "complete image size does not fit usize".to_owned())?
    {
        return Err(format!(
            "{format_name} payload length differs from serialized sizes"
        ));
    }
    let mipmapped = boolean(value, "m_MipMap")?;
    let mut rgba = Vec::new();
    let mut offset = 0_usize;
    let (mut mip_width, mut mip_height) = (width, height);
    let mut mip_count = 0_u32;
    loop {
        let bytes = match block_bytes {
            Some(block_bytes) => compressed_level_size(block_bytes, mip_width, mip_height)?,
            None => argb4444_level_size(mip_width, mip_height)?,
        };
        let end = offset
            .checked_add(bytes)
            .ok_or_else(|| format!("{format_name} mip offset overflows"))?;
        let level = compressed
            .get(offset..end)
            .ok_or_else(|| format!("{format_name} ends before mip {mip_count}"))?;
        let mut decoded = match texture_format {
            2 => decode_argb4444_level(level, mip_width, mip_height)?,
            10 => decode_bc1_level(level, mip_width, mip_height)?,
            12 => decode_bc3_level(level, mip_width, mip_height)?,
            _ => unreachable!("texture format was validated above"),
        };
        flip_rgba_rows(&mut decoded, mip_width, mip_height);
        rgba.extend_from_slice(&decoded);
        offset = end;
        mip_count += 1;
        if !mipmapped || (mip_width == 1 && mip_height == 1) {
            break;
        }
        mip_width = (mip_width / 2).max(1);
        mip_height = (mip_height / 2).max(1);
    }
    if offset != compressed.len() {
        return Err(format!(
            "{format_name} payload has trailing bytes after exact mip chain"
        ));
    }
    Ok(ExactTexture {
        key: (
            texture_object.asset.clone(),
            texture_object.path_id,
            texture_object.canonical_blake3.clone(),
        ),
        width,
        height,
        mip_count,
        rgba_mips: rgba.into(),
    })
}

pub(super) fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    if input.len() % 4 != 0 {
        return Err("base64 length is not divisible by four".to_owned());
    }
    let mut output = Vec::with_capacity(input.len() / 4 * 3);
    for chunk in input.as_bytes().chunks_exact(4) {
        let mut values = [0_u8; 4];
        let mut padding = 0;
        for (index, byte) in chunk.iter().copied().enumerate() {
            values[index] = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' if index >= 2 => {
                    padding += 1;
                    0
                }
                _ => return Err(format!("invalid base64 byte {byte}")),
            };
        }
        if padding > 2 || (padding > 0 && chunk[3] != b'=') {
            return Err("invalid base64 padding".to_owned());
        }
        output.push((values[0] << 2) | (values[1] >> 4));
        if padding < 2 {
            output.push((values[1] << 4) | (values[2] >> 2));
        }
        if padding == 0 {
            output.push((values[2] << 6) | values[3]);
        }
    }
    Ok(output)
}

pub(super) fn decode_argb4444_level(source: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let expected = argb4444_level_size(width, height)?;
    if source.len() != expected {
        return Err("ARGB4444 level has an invalid pixel count".to_owned());
    }
    let mut output = Vec::with_capacity(rgba_level_size(width, height)?);
    for bytes in source.chunks_exact(2) {
        let pixel = u16::from_le_bytes([bytes[0], bytes[1]]);
        let expand = |value: u16| (value as u8) * 17;
        output.extend_from_slice(&[
            expand((pixel >> 8) & 0x0f),
            expand((pixel >> 4) & 0x0f),
            expand(pixel & 0x0f),
            expand((pixel >> 12) & 0x0f),
        ]);
    }
    Ok(output)
}

pub(super) fn decode_bc3_level(source: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let expected = compressed_level_size(16, width, height)?;
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);
    if source.len() != expected {
        return Err("DXT5 level has an invalid block count".to_owned());
    }
    let mut output = vec![0_u8; rgba_level_size(width, height)?];
    for block_y in 0..blocks_y {
        for block_x in 0..blocks_x {
            let start = ((block_y * blocks_x + block_x) * 16) as usize;
            let block = &source[start..start + 16];
            let alphas = bc3_alpha_palette(block[0], block[1]);
            let alpha_bits = block[2..8]
                .iter()
                .copied()
                .enumerate()
                .fold(0_u64, |bits, (shift, byte)| {
                    bits | u64::from(byte) << (shift * 8)
                });
            let colors = bc3_color_palette(
                u16::from_le_bytes([block[8], block[9]]),
                u16::from_le_bytes([block[10], block[11]]),
            );
            let color_bits = u32::from_le_bytes([block[12], block[13], block[14], block[15]]);
            for pixel in 0..16_u32 {
                let x = block_x * 4 + pixel % 4;
                let y = block_y * 4 + pixel / 4;
                if x >= width || y >= height {
                    continue;
                }
                let destination = ((y * width + x) * 4) as usize;
                output[destination..destination + 3]
                    .copy_from_slice(&colors[((color_bits >> (pixel * 2)) & 3) as usize]);
                output[destination + 3] = alphas[((alpha_bits >> (pixel * 3)) & 7) as usize];
            }
        }
    }
    Ok(output)
}

pub(super) fn decode_bc1_level(source: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let expected = compressed_level_size(8, width, height)?;
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);
    if source.len() != expected {
        return Err("DXT1 level has an invalid block count".to_owned());
    }
    let mut output = vec![0_u8; rgba_level_size(width, height)?];
    for block_y in 0..blocks_y {
        for block_x in 0..blocks_x {
            let start = ((block_y * blocks_x + block_x) * 8) as usize;
            let block = &source[start..start + 8];
            let colors = bc1_color_palette(
                u16::from_le_bytes([block[0], block[1]]),
                u16::from_le_bytes([block[2], block[3]]),
            );
            let color_bits = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
            for pixel in 0..16_u32 {
                let x = block_x * 4 + pixel % 4;
                let y = block_y * 4 + pixel / 4;
                if x >= width || y >= height {
                    continue;
                }
                let destination = ((y * width + x) * 4) as usize;
                output[destination..destination + 4]
                    .copy_from_slice(&colors[((color_bits >> (pixel * 2)) & 3) as usize]);
            }
        }
    }
    Ok(output)
}
