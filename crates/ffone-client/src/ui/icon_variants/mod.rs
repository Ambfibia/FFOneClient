//! Finite, reusable variants of resident UI icons. No per-frame pixel conversion.
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum IconVariant {
    Hue(u16),
    Grayscale,
    /// One-pixel square dilation, with a transparent interior for independent pulsing.
    Outline,
}

#[derive(Resource, Default)]
pub struct UiIconVariants(HashMap<(AssetId<Image>, IconVariant), Handle<Image>>);

impl UiIconVariants {
    pub fn image(
        &mut self,
        source: &Handle<Image>,
        variant: IconVariant,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        let key = (source.id(), variant);
        if let Some(image) = self.0.get(&key) {
            return Some(image.clone());
        }
        let source = images.get(source)?;
        let converted = source.convert(TextureFormat::Rgba8UnormSrgb)?;
        let data = converted.data.as_deref()?;
        let width = converted.width();
        let height = converted.height();
        let (width, height, pixels) = transform_icon(data, width, height, variant);
        let mut image = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        );
        image.sampler = source.sampler.clone();
        let handle = images.add(image);
        self.0.insert(key, handle.clone());
        Some(handle)
    }
}

fn transform_icon(
    data: &[u8],
    width: u32,
    height: u32,
    variant: IconVariant,
) -> (u32, u32, Vec<u8>) {
    if variant == IconVariant::Outline {
        let mut output = vec![0; ((width + 2) * (height + 2) * 4) as usize];
        for y in 0..height + 2 {
            for x in 0..width + 2 {
                let sx = x as i32 - 1;
                let sy = y as i32 - 1;
                let opaque = |x: i32, y: i32| {
                    x >= 0
                        && y >= 0
                        && x < width as i32
                        && y < height as i32
                        && data[((y as u32 * width + x as u32) * 4 + 3) as usize] > 0
                };
                if !opaque(sx, sy) && (-1..=1).any(|dy| (-1..=1).any(|dx| opaque(sx + dx, sy + dy)))
                {
                    let index = ((y * (width + 2) + x) * 4) as usize;
                    output[index..index + 4].copy_from_slice(&[255; 4]);
                }
            }
        }
        return (width + 2, height + 2, output);
    }
    let mut output = data.to_vec();
    for pixel in output.chunks_exact_mut(4) {
        let r = f32::from(pixel[0]) / 255.0;
        let g = f32::from(pixel[1]) / 255.0;
        let b = f32::from(pixel[2]) / 255.0;
        let color = match variant {
            IconVariant::Grayscale => [r * 0.299 + g * 0.587 + b * 0.114; 3],
            IconVariant::Hue(hue) => {
                let high = r.max(g).max(b);
                let low = r.min(g).min(b);
                let c = high - low;
                let h = f32::from(hue) / 60.0;
                let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
                let rgb = match h as u32 {
                    0 => [c, x, 0.0],
                    1 => [x, c, 0.0],
                    2 => [0.0, c, x],
                    3 => [0.0, x, c],
                    4 => [x, 0.0, c],
                    _ => [c, 0.0, x],
                };
                rgb.map(|value| value + low)
            }
            IconVariant::Outline => unreachable!(),
        };
        for index in 0..3 {
            pixel[index] = (color[index] * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    (width, height, output)
}

#[cfg(test)]
mod tests;
