use super::*;

pub(super) fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub(super) fn legacy_text_font(font: &Handle<Font>, size: LegacyTextSize) -> (TextFont, LineHeight) {
    let (font_size, line_height) = match size {
        // Vector calibration for clean fixed raster JEFFE___16 path ID 1012.
        LegacyTextSize::Large => (
            SERVER_SELECTION_JEFFE_16_FONT_SIZE,
            SERVER_SELECTION_JEFFE_16_LINE_HEIGHT,
        ),
        // Vector calibration for clean fixed raster JEFFE___14 path ID 903.
        LegacyTextSize::Small => (
            SERVER_SELECTION_JEFFE_14_FONT_SIZE,
            SERVER_SELECTION_JEFFE_14_LINE_HEIGHT,
        ),
    };
    (
        TextFont {
            font: (font.clone()).into(),
            font_size: (font_size).into(),
            ..default()
        },
        LineHeight::Px(line_height),
    )
}

pub(super) fn absolute_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        ..default()
    }
}

pub(super) fn transparent_row_image(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        image_mode: NodeImageMode::Stretch,
        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        ..default()
    }
}
