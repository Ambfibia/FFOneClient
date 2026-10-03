use super::*;

pub(super) fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
    mode: Res<PreviewMode>,
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
            "rejecting ResurrectMode capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let layout = resurrect_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        1.0,
    );
    let dialog = layout.visual_rect;
    // The previous panel-only audit accepted a frame with Grim clipped off.
    // His light face/scythe pixels must also be rendered above the panel.
    let mut grim_pixels = 0;
    for y in (dialog.y + RESURRECT_GRIM_RECT.y).max(0.0) as u32..dialog.y as u32 {
        for x in (dialog.x + RESURRECT_GRIM_RECT.x) as u32
            ..(dialog.x + RESURRECT_GRIM_RECT.x + RESURRECT_GRIM_RECT.width) as u32
        {
            let [red, green, blue, _] = rgba.get_pixel(x, y).0;
            if red > 150 && green > 150 && blue > 150 {
                grim_pixels += 1;
            }
        }
    }
    if grim_pixels < 500 {
        eprintln!("rejecting clipped Grim above resurrection panel: {grim_pixels} light pixels");
        state.capture_failed = true;
        return;
    }
    let mut visible_pixels = 0;
    let mut cyan_pixels = 0;
    let mut red_pixels = 0;
    for y in
        dialog.y.max(0.0) as u32..(dialog.y + dialog.height).min(CLIENT_AREA_HEIGHT as f32) as u32
    {
        for x in
            dialog.x.max(0.0) as u32..(dialog.x + dialog.width).min(CLIENT_AREA_WIDTH as f32) as u32
        {
            let [red, green, blue, _alpha] = rgba.get_pixel(x, y).0;
            if u16::from(red) + u16::from(green) + u16::from(blue) > 75 {
                visible_pixels += 1;
            }
            if blue > 100 && green > 75 && blue > red.saturating_add(20) {
                cyan_pixels += 1;
            }
            if red > 125 && red > green.saturating_add(45) && red > blue.saturating_add(45) {
                red_pixels += 1;
            }
        }
    }
    if visible_pixels < MIN_DIALOG_VISIBLE_PIXELS
        || cyan_pixels < MIN_CYAN_PIXELS
        || red_pixels < MIN_RED_COUNTDOWN_PIXELS
    {
        eprintln!(
            "rejecting incomplete ResurrectMode {} capture: visible={visible_pixels} \
             (min {MIN_DIALOG_VISIBLE_PIXELS}), cyan={cyan_pixels} \
             (min {MIN_CYAN_PIXELS}), red={red_pixels} \
             (min {MIN_RED_COUNTDOWN_PIXELS})",
            mode.cli_name()
        );
        state.capture_failed = true;
        return;
    }

    if let Some(parent) = output
        .0
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&output.0)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.0.display()));
    println!(
        "{}",
        output
            .0
            .canonicalize()
            .unwrap_or_else(|_| output.0.clone())
            .display()
    );
    state.capture_saved = true;
}
