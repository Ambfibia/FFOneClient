//! Measure glyph alpha, excluding transparent allocation margins and adjacent glyphs.
use bevy::{prelude::*, render::render_resource::TextureFormat, text::PositionedGlyph};

/// Invert Bevy TextPipeline's atlas placement to recover Parley's positioned pen.
/// Its Y equals the line baseline plus the shaping offset; marks can have offsets.
pub(super) fn glyph_pen_position(glyph: &PositionedGlyph) -> Vec2 {
    glyph.position - glyph.atlas_info.rect.size() * 0.5 - glyph.atlas_info.offset
}

pub(super) fn glyph_ink_bounds(
    glyph: &PositionedGlyph,
    images: &Assets<Image>,
) -> Result<Option<Rect>, String> {
    let image = images
        .get(glyph.atlas_info.texture)
        .ok_or("glyph atlas is not CPU resident")?;
    if !matches!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    ) {
        return Err("unsupported glyph atlas pixel format".into());
    }
    let rect = glyph.atlas_info.rect;
    if [rect.min.x, rect.min.y, rect.max.x, rect.max.y]
        .into_iter()
        .any(|x| !x.is_finite() || x < 0.0 || x.fract() != 0.0)
    {
        return Err("glyph atlas rectangle must have integer pixel bounds".into());
    }
    let crop = URect::from_corners(rect.min.as_uvec2(), rect.max.as_uvec2());
    let size = UVec2::new(image.width(), image.height());
    let bounds = alpha_bounds(
        image
            .data
            .as_deref()
            .ok_or("glyph atlas has no CPU pixels")?,
        size,
        crop,
    )?;
    let origin = glyph.position - rect.size() * 0.5;
    Ok(bounds.map(|b| {
        Rect::from_corners(
            origin + (b.min - crop.min).as_vec2(),
            origin + (b.max - crop.min).as_vec2(),
        )
    }))
}

fn alpha_bounds(data: &[u8], size: UVec2, crop: URect) -> Result<Option<URect>, String> {
    if crop.min.x > crop.max.x
        || crop.min.y > crop.max.y
        || crop.max.x > size.x
        || crop.max.y > size.y
        || data.len() != size.x as usize * size.y as usize * 4
    {
        return Err("invalid RGBA glyph atlas/crop dimensions".into());
    }
    let mut minimum = UVec2::splat(u32::MAX);
    let mut maximum = UVec2::ZERO;
    for y in crop.min.y..crop.max.y {
        for x in crop.min.x..crop.max.x {
            if data[((y as usize * size.x as usize + x as usize) * 4) + 3] > 0 {
                minimum = minimum.min(UVec2::new(x, y));
                maximum = maximum.max(UVec2::new(x + 1, y + 1));
            }
        }
    }
    Ok((minimum.x != u32::MAX).then(|| URect::from_corners(minimum, maximum)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alpha_measurement_excludes_transparent_margins_and_neighbouring_glyphs() {
        let mut pixels = vec![0; 8 * 8 * 4];
        pixels[(3 * 8 + 4) * 4 + 3] = 1;
        pixels[(4 * 8 + 5) * 4 + 3] = 255;
        pixels[(7 * 8 + 7) * 4 + 3] = 255;
        let crop = URect::from_corners(UVec2::new(2, 2), UVec2::new(6, 6));
        assert_eq!(
            alpha_bounds(&pixels, UVec2::splat(8), crop).unwrap(),
            Some(URect::from_corners(UVec2::new(4, 3), UVec2::new(6, 5)))
        );
        assert_eq!(
            alpha_bounds(&vec![0; pixels.len()], UVec2::splat(8), crop).unwrap(),
            None
        );
        assert!(alpha_bounds(&pixels[..4], UVec2::splat(8), crop).is_err());
    }
}
