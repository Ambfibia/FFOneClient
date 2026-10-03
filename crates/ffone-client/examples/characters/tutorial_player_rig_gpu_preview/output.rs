use super::*;

pub(super) fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    if let Some(parent) = config
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&config.output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", config.output.display()));
    println!("{}", absolute_display(&config.output));
    state.capture_saved = true;
}

pub(super) fn save_dismount_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
) {
    let output = config.output.with_file_name(format!(
        "{}-{}.png",
        config.output.file_stem().unwrap().to_string_lossy(),
        if config.case == PreviewCase::WeaponSwap {
            "unarmed"
        } else {
            "dismounted"
        }
    ));
    event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image")
        .save(&output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.display()));
    println!("{}", absolute_display(&output));
    state.dismount_capture_saved = true;
}
