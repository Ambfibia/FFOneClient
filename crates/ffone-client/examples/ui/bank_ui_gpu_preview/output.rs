use super::*;

pub(super) fn write_item(bytes: &mut [u8], offset: usize, value: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}

pub(super) fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
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
            "rejecting BankMode capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let mut visible_pixels = 0;
    let mut checker_pixels = 0;
    for pixel in rgba.pixels() {
        let [red, green, blue, _alpha] = pixel.0;
        if u16::from(red) + u16::from(green) + u16::from(blue) > 35 {
            visible_pixels += 1;
        }
        if red > 180 && blue > 180 && green < 80 {
            checker_pixels += 1;
        }
    }
    let overlay = matches!(
        std::env::var("FFONE_BANK_POINTER").as_deref(),
        Ok("help"
            | "redeem"
            | "popup-general"
            | "popup-equip"
            | "popup-long-name"
            | "popup-expires"
            | "popup-rental"
            | "popup-combined"
            | "popup-vehicle"
            | "popup-chest"
            | "popup-inventory"
            | "popup-transfer"
            | "popup-delete"
            | "popup-delete-accept"
            | "popup-delete-cancel")
    );
    if visible_pixels < MIN_VISIBLE_PIXELS
        || (!overlay
            && env::var_os("FFONE_SERVICE_CONTROL").is_none()
            && checker_pixels < MIN_MISSING_CHECKER_PIXELS)
    {
        eprintln!(
            "rejecting incomplete BankMode capture: visible={visible_pixels} \
             (min {MIN_VISIBLE_PIXELS}), checker={checker_pixels} \
             (min {MIN_MISSING_CHECKER_PIXELS})"
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
    println!("{}", absolute_display(&output.0));
    state.capture_saved = true;
}
