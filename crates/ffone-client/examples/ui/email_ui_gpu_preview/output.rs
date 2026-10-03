use super::*;

pub(super) fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewCli>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    if rgba.dimensions() != (CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT) {
        eprintln!(
            "rejecting Email capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let layout = email_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        clean_email_ui_scale(CLIENT_AREA_HEIGHT as f32),
        1.0,
    );
    let x_start = layout.left_backplate.left.max(0.0) as u32;
    let x_end = (layout.right_backplate.left + layout.right_backplate.width)
        .min(CLIENT_AREA_WIDTH as f32) as u32;
    let y_start = layout.left_backplate.top.max(0.0) as u32;
    let y_end = (layout.left_backplate.top + layout.left_backplate.height)
        .min(CLIENT_AREA_HEIGHT as f32) as u32;
    let mut visible_pixels = 0;
    let mut cyan_pixels = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
            if alpha > 0 && u16::from(red) + u16::from(green) + u16::from(blue) > 90 {
                visible_pixels += 1;
            }
            if alpha > 0 && green > 105 && blue > 120 && blue > red.saturating_add(35) {
                cyan_pixels += 1;
            }
        }
    }
    if visible_pixels < MIN_WINDOW_VISIBLE_PIXELS || cyan_pixels < MIN_CYAN_PIXELS {
        eprintln!(
            "rejecting incomplete Email capture: visible={visible_pixels} \
             (min {MIN_WINDOW_VISIBLE_PIXELS}), cyan={cyan_pixels} \
             (min {MIN_CYAN_PIXELS})"
        );
        state.capture_failed = true;
        return;
    }

    if let Some(parent) = output
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&output.output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.output.display()));
    println!(
        "{}",
        output
            .output
            .canonicalize()
            .unwrap_or_else(|_| output.output.clone())
            .display()
    );
    state.capture_saved = true;
}
