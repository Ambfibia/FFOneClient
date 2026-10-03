//! Image, slot-frame and missing-icon checker helpers.

use super::asset_contract::UserEquipStaticAssetRole;
use super::components::UserEquipUiAssets;
use super::geometry::{
    USER_EQUIP_MISSING_CHECKER_QUADRANT, USER_EQUIP_MISSING_CHECKER_SIZE, UserEquipUiRect,
};
use super::view_model::{UserEquipPresentationIcon, UserEquipSlotFrameVisual};
use bevy::{
    asset::{LoadState, RenderAssetUsages},
    image::{ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub(super) fn bind_rect(node: &mut Node, rect: UserEquipUiRect) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}

pub(super) fn slot_frame_image(visual: UserEquipSlotFrameVisual, assets: &UserEquipUiAssets) -> Handle<Image> {
    assets.image(match visual {
        UserEquipSlotFrameVisual::Empty => UserEquipStaticAssetRole::SlotEmpty,
        UserEquipSlotFrameVisual::Occupied => UserEquipStaticAssetRole::SlotOccupied,
    })
}

pub(super) fn presentation_icon_handle(
    icon: &UserEquipPresentationIcon,
    asset_server: &AssetServer,
    assets: &UserEquipUiAssets,
) -> (Handle<Image>, bool) {
    match icon {
        UserEquipPresentationIcon::Empty => (Handle::default(), false),
        UserEquipPresentationIcon::MissingChecker => (assets.missing_checker.clone(), true),
        UserEquipPresentationIcon::Resolved(path) => {
            let handle = if path.starts_with("icons/entities/nanos/") {
                asset_server
                    .load_builder()
                    .with_settings::<ImageLoaderSettings>(configure_user_equip_nano_icon)
                    .load::<Image>(path.clone())
            } else {
                asset_server.load::<Image>(path.clone())
            };
            if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
                (assets.missing_checker.clone(), true)
            } else {
                // The TableData catalog admitted this path only after proving
                // that the installed PNG exists. Keep its stable handle while
                // Bevy loads it; treating `NotLoaded`/`Loading` as a catalog
                // miss made every valid icon flash as the magenta checker.
                (handle, true)
            }
        }
    }
}

pub(super) fn configure_user_equip_nano_icon(settings: &mut ImageLoaderSettings) {
    // These 64px DXT-origin silhouettes keep non-zero RGB in transparent edge
    // texels. Linear resampling to the 67px IMGUI cell interpolates that hidden
    // color into a pale fringe; point sampling preserves the authored alpha
    // boundary and the source's pixel-art presentation.
    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });
}

pub(super) use crate::ui_support::stretched_image;

pub(super) use crate::ui_support::sliced_image;

#[must_use]
pub fn user_equip_missing_checker_rgba() -> Vec<u8> {
    let mut rgba =
        vec![0; (USER_EQUIP_MISSING_CHECKER_SIZE * USER_EQUIP_MISSING_CHECKER_SIZE * 4) as usize];
    for y in 0..USER_EQUIP_MISSING_CHECKER_SIZE {
        for x in 0..USER_EQUIP_MISSING_CHECKER_SIZE {
            let magenta = (x / USER_EQUIP_MISSING_CHECKER_QUADRANT
                + y / USER_EQUIP_MISSING_CHECKER_QUADRANT)
                % 2
                == 0;
            let offset = ((y * USER_EQUIP_MISSING_CHECKER_SIZE + x) * 4) as usize;
            rgba[offset..offset + 4].copy_from_slice(if magenta {
                &[255, 0, 255, 255]
            } else {
                &[0, 0, 0, 255]
            });
        }
    }
    rgba
}

pub(super) fn user_equip_missing_checker_image() -> Image {
    Image::new(
        Extent3d {
            width: USER_EQUIP_MISSING_CHECKER_SIZE,
            height: USER_EQUIP_MISSING_CHECKER_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        user_equip_missing_checker_rgba(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
