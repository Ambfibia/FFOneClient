use super::*;

pub(super) fn user_store_missing_checker_image() -> Image {
    Image::new(
        Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        user_store_missing_checker_rgba_0104(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
