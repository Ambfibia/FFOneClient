use super::*;

pub(super) fn validate_texture_uri(
    source_name: &str,
    uri: &str,
    preserves_legacy_extension: bool,
) -> Result<()> {
    if uri.is_empty()
        || uri.starts_with('/')
        || uri.contains(['\\', ':', '?', '#'])
        || uri.chars().any(char::is_control)
    {
        return invalid("texture URI is not a safe relative semantic PNG path");
    }
    let segments = uri.split('/').collect::<Vec<_>>();
    // Shared native chains use leading parent segments relative to the model.
    // The asset-root-aware loader/publisher must also prove root containment.
    let leading_parents = segments.iter().take_while(|segment| **segment == "..").count();
    let tail = &segments[leading_parents..];
    let shared = leading_parents > 0 && tail.contains(&"textures");
    if tail.is_empty() || tail.iter().any(|segment| {
        segment.is_empty() || matches!(*segment, "." | "..")
            || (!shared && has_generated_identity(segment))
    }) {
        return invalid("texture URI contains an unsafe/hash/PathID path segment");
    }
    if !uri.ends_with(".png") { return invalid("texture URI must name a PNG"); }
    let expected = if preserves_legacy_extension {
        windows_png_filename_preserving_legacy_extension(source_name)?
    } else {
        minimal_windows_png_filename(source_name)?
    };
    if !shared && segments.last().copied() != Some(expected.as_str()) {
        return invalid(format!(
            "texture PNG basename must be the minimally safe true-name filename {expected:?}"
        ));
    }
    Ok(())
}

pub(super) fn texture_format_name(format: i32) -> Option<&'static str> {
    match format {
        1 => Some("Alpha8"),
        2 => Some("ARGB4444"),
        3 => Some("RGB24"),
        4 => Some("RGBA32"),
        5 => Some("ARGB32"),
        7 => Some("RGB565"),
        10 => Some("DXT1"),
        11 => Some("DXT3"),
        12 => Some("DXT5"),
        13 => Some("RGBA4444"),
        14 => Some("BGRA32"),
        _ => None,
    }
}

pub(super) fn texture_mip_byte_length(format: i32, width: u32, height: u32) -> Option<u64> {
    let pixels = u64::from(width).checked_mul(u64::from(height))?;
    match format {
        1 => Some(pixels),
        3 => pixels.checked_mul(3),
        4 | 5 | 14 => pixels.checked_mul(4),
        2 | 7 | 13 => pixels.checked_mul(2),
        10 => u64::from(width.div_ceil(4))
            .checked_mul(u64::from(height.div_ceil(4)))?
            .checked_mul(8),
        11 | 12 => u64::from(width.div_ceil(4))
            .checked_mul(u64::from(height.div_ceil(4)))?
            .checked_mul(16),
        _ => None,
    }
}
