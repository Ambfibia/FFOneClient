use super::*;

pub(super) fn arguments() -> (PathBuf, PathBuf) {
    let mut layout = PathBuf::from(DEFAULT_LAYOUT);
    let mut background = PathBuf::from(DEFAULT_BACKGROUND);
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--layout" => {
                if let Some(value) = args.next() {
                    layout = PathBuf::from(value);
                }
            }
            "--background" => {
                if let Some(value) = args.next() {
                    background = PathBuf::from(value);
                }
            }
            _ if !arg.starts_with('-') => layout = PathBuf::from(arg),
            _ => {}
        }
    }
    (layout, background)
}

pub(super) fn draw_element_visual(
    painter: &egui::Painter,
    target: Rect,
    visual: &UiLayoutVisual,
    textures: &BTreeMap<String, egui::TextureHandle>,
    fonts: &BTreeMap<String, FontFamily>,
    zoom: f32,
) {
    if let Some(fill) = visual.fill {
        painter.rect_filled(target, 0.0, rgba(fill));
    }
    if let Some(image) = &visual.image {
        if let Some(texture) = textures.get(&image.path) {
            draw_image(painter, target, texture, image, zoom);
        } else {
            painter.rect_stroke(
                target,
                0.0,
                Stroke::new(1.5_f32, Color32::RED),
                StrokeKind::Inside,
            );
            if target.width() > 45.0 && target.height() > 14.0 {
                painter.text(
                    target.center(),
                    egui::Align2::CENTER_CENTER,
                    "! asset",
                    FontId::monospace(9.0),
                    Color32::RED,
                );
            }
        }
    }
    if visual.dynamic_placeholder {
        draw_checker(
            painter,
            if visual.image.is_some() {
                target.shrink(3.0 * zoom)
            } else {
                target
            },
            zoom,
        );
    }
    if let Some(text) = &visual.text {
        draw_visual_text(painter, target, text, fonts, zoom);
    }
}

pub(super) fn draw_image(
    painter: &egui::Painter,
    target: Rect,
    texture: &egui::TextureHandle,
    image: &UiLayoutImage,
    zoom: f32,
) {
    let texture_size = texture.size_vec2();
    let source =
        image
            .source_rect
            .unwrap_or(UiLayoutRect::new(0.0, 0.0, texture_size.x, texture_size.y));
    let tint = rgba(image.tint);
    match image.mode {
        UiLayoutImageMode::Stretch => {
            painter.image(texture.id(), target, source_uv(source, texture_size), tint);
        }
        UiLayoutImageMode::Contain => {
            let source_aspect = source.width / source.height;
            let target_aspect = target.width() / target.height();
            let size = if target_aspect > source_aspect {
                Vec2::new(target.height() * source_aspect, target.height())
            } else {
                Vec2::new(target.width(), target.width() / source_aspect)
            };
            painter.image(
                texture.id(),
                Rect::from_center_size(target.center(), size),
                source_uv(source, texture_size),
                tint,
            );
        }
        UiLayoutImageMode::NineSlice => {
            draw_nine_slice(
                painter,
                target,
                texture,
                source,
                image.border,
                texture_size,
                tint,
                zoom,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_nine_slice(
    painter: &egui::Painter,
    target: Rect,
    texture: &egui::TextureHandle,
    source: UiLayoutRect,
    border: [f32; 4],
    texture_size: Vec2,
    tint: Color32,
    zoom: f32,
) {
    let [left, right, top, bottom] = border;
    let source_x = [
        source.x,
        source.x + left.min(source.width),
        source.x + (source.width - right).max(left),
        source.x + source.width,
    ];
    let source_y = [
        source.y,
        source.y + top.min(source.height),
        source.y + (source.height - bottom).max(top),
        source.y + source.height,
    ];
    let mut target_left = left * zoom;
    let mut target_right = right * zoom;
    if target_left + target_right > target.width() && target_left + target_right > 0.0 {
        let scale = target.width() / (target_left + target_right);
        target_left *= scale;
        target_right *= scale;
    }
    let mut target_top = top * zoom;
    let mut target_bottom = bottom * zoom;
    if target_top + target_bottom > target.height() && target_top + target_bottom > 0.0 {
        let scale = target.height() / (target_top + target_bottom);
        target_top *= scale;
        target_bottom *= scale;
    }
    let target_x = [
        target.left(),
        target.left() + target_left,
        target.right() - target_right,
        target.right(),
    ];
    let target_y = [
        target.top(),
        target.top() + target_top,
        target.bottom() - target_bottom,
        target.bottom(),
    ];
    for row in 0..3 {
        for column in 0..3 {
            if source_x[column + 1] <= source_x[column]
                || source_y[row + 1] <= source_y[row]
                || target_x[column + 1] <= target_x[column]
                || target_y[row + 1] <= target_y[row]
            {
                continue;
            }
            painter.image(
                texture.id(),
                Rect::from_min_max(
                    Pos2::new(target_x[column], target_y[row]),
                    Pos2::new(target_x[column + 1], target_y[row + 1]),
                ),
                Rect::from_min_max(
                    Pos2::new(
                        source_x[column] / texture_size.x,
                        source_y[row] / texture_size.y,
                    ),
                    Pos2::new(
                        source_x[column + 1] / texture_size.x,
                        source_y[row + 1] / texture_size.y,
                    ),
                ),
                tint,
            );
        }
    }
}

pub(super) fn draw_visual_text(
    painter: &egui::Painter,
    target: Rect,
    text: &ffone_ui_layout::UiLayoutText,
    fonts: &BTreeMap<String, FontFamily>,
    zoom: f32,
) {
    let family = text
        .font
        .as_ref()
        .and_then(|path| fonts.get(path))
        .cloned()
        .unwrap_or(FontFamily::Proportional);
    let color = rgba(text.color);
    let font = FontId::new((text.font_size * zoom).max(1.0), family);
    let wrap_width = if text.wrap {
        target.width().max(1.0)
    } else {
        f32::INFINITY
    };
    let galley = painter.layout(text.value.clone(), font, color, wrap_width);
    let x = match text.horizontal {
        UiLayoutHorizontalAlign::Left => target.left(),
        UiLayoutHorizontalAlign::Center => target.center().x - galley.size().x * 0.5,
        UiLayoutHorizontalAlign::Right => target.right() - galley.size().x,
    };
    let y = match text.vertical {
        UiLayoutVerticalAlign::Top => target.top(),
        UiLayoutVerticalAlign::Center => target.center().y - galley.size().y * 0.5,
        UiLayoutVerticalAlign::Bottom => target.bottom() - galley.size().y,
    };
    painter
        .with_clip_rect(target)
        .galley(Pos2::new(x, y), galley, color);
}

pub(super) fn draw_checker(painter: &egui::Painter, target: Rect, zoom: f32) {
    let cell = (8.0 * zoom).clamp(4.0, 16.0);
    let mut row = 0usize;
    let mut y = target.top();
    while y < target.bottom() {
        let mut column = 0usize;
        let mut x = target.left();
        while x < target.right() {
            let color = if (row + column).is_multiple_of(2) {
                Color32::from_rgba_unmultiplied(235, 0, 235, 105)
            } else {
                Color32::from_rgba_unmultiplied(20, 20, 20, 105)
            };
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, y),
                    Pos2::new(
                        (x + cell).min(target.right()),
                        (y + cell).min(target.bottom()),
                    ),
                ),
                0.0,
                color,
            );
            x += cell;
            column += 1;
        }
        y += cell;
        row += 1;
    }
    painter.rect_stroke(
        target,
        0.0,
        Stroke::new(1.0_f32, Color32::from_white_alpha(90)),
        StrokeKind::Inside,
    );
}

pub(super) fn source_uv(source: UiLayoutRect, texture_size: Vec2) -> Rect {
    Rect::from_min_max(
        Pos2::new(source.x / texture_size.x, source.y / texture_size.y),
        Pos2::new(
            (source.x + source.width) / texture_size.x,
            (source.y + source.height) / texture_size.y,
        ),
    )
}

pub(super) fn draw_grid(painter: &egui::Painter, canvas: Rect, zoom: f32, grid: f32) {
    let step = grid * zoom;
    if step < 4.0 {
        return;
    }
    let major = (grid * 10.0 * zoom).max(step);
    let mut x = canvas.left();
    while x <= canvas.right() {
        let is_major = ((x - canvas.left()) % major).abs() < 0.5;
        painter.line_segment(
            [Pos2::new(x, canvas.top()), Pos2::new(x, canvas.bottom())],
            Stroke::new(
                1.0_f32,
                if is_major {
                    Color32::from_white_alpha(35)
                } else {
                    Color32::from_white_alpha(12)
                },
            ),
        );
        x += step;
    }
    let mut y = canvas.top();
    while y <= canvas.bottom() {
        let is_major = ((y - canvas.top()) % major).abs() < 0.5;
        painter.line_segment(
            [Pos2::new(canvas.left(), y), Pos2::new(canvas.right(), y)],
            Stroke::new(
                1.0_f32,
                if is_major {
                    Color32::from_white_alpha(35)
                } else {
                    Color32::from_white_alpha(12)
                },
            ),
        );
        y += step;
    }
}

pub(super) fn group_color(group: &str) -> Color32 {
    let mut hash = 2_166_136_261_u32;
    for byte in group.bytes() {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16_777_619);
    }
    Color32::from_rgb(
        80 + (hash & 127) as u8,
        100 + ((hash >> 8) & 127) as u8,
        120 + ((hash >> 16) & 127) as u8,
    )
}
