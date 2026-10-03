//! Shared UI primitives; screen-specific layout remains in its owning module.
pub mod controller;
use bevy::{
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    ui::widget::NodeImageMode,
};

/// Draw a control background across its full rect; padding belongs to its content.
pub fn sliced_image(image: Handle<Image>, border: BorderRect) -> ImageNode {
    ImageNode {
        image,
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border,
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

pub fn stretched_image(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

pub const fn display_if(visible: bool) -> Display {
    if visible {
        Display::Flex
    } else {
        Display::None
    }
}

/// Minimum frame height is independent of the intrinsic height of a list row.
/// `content_bottom` is the measured end of the window's content, in frame
/// coordinates; `footer_height` reserves the controls below it.
pub fn window_height(minimum_frame_height: f32, content_bottom: f32, footer_height: f32) -> f32 {
    minimum_frame_height.max(content_bottom + footer_height)
}

#[cfg(test)]
mod window_height_tests {
    use super::window_height;

    #[test]
    fn short_content_keeps_frame_and_wrapped_content_grows_it() {
        assert_eq!(window_height(183.0, 50.0, 76.0), 183.0);
        assert_eq!(window_height(183.0, 187.0, 76.0), 263.0);
    }
}

pub const fn valid_ui_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

pub fn legacy_screen_extent(value: f32) -> i32 {
    if value.is_finite() && value > 0.0 {
        value.floor().min(i32::MAX as f32) as i32
    } else {
        0
    }
}
