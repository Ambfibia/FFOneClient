use super::*;

pub(super) fn write_json_report(path: &Path, report: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("cannot serialize runtime report: {error}"))?;
    fs::write(path, bytes)
        .map_err(|error| format!("cannot write runtime report {}: {error}", path.display()))
}

pub(super) fn save_preview_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    outline_sources: Query<&LegacyOutlineSource>,
    mut state: ResMut<RuntimeState>,
    shared: Res<SharedReport>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if state.terminal {
        return;
    }
    let result = event
        .image
        .clone()
        .try_into_dynamic()
        .map_err(|error| format!("cannot convert GPU screenshot: {error}"))
        .and_then(|image| {
            let image = image.to_rgb8();
            let outline_colors =
                if config.outline == PreviewOutlineMode::Source && config.only_material.is_none() {
                    outline_sources
                        .iter()
                        .map(|source| linear_rgba_to_srgb8(source.color))
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
            let (foreground_pixels, foreground_coverage) = screenshot_foreground(
                image.as_raw(),
                image.width(),
                image.height(),
                &outline_colors,
                config.only_material.as_ref().map(|_| 16),
            )?;
            let mut bytes = Vec::new();
            PngEncoder::new(&mut bytes)
                .write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    ColorType::Rgb8.into(),
                )
                .map_err(|error| format!("cannot encode GPU screenshot PNG: {error}"))?;
            Ok((
                CapturedPng {
                    bytes,
                    width: image.width(),
                    height: image.height(),
                    foreground_pixels,
                },
                foreground_coverage,
            ))
        });
    match result {
        Ok((captured, foreground_coverage)) => {
            let foreground_pixels = captured.foreground_pixels;
            state.captured_png = Some(captured);
            state.screenshot_saved_frame = Some(state.frames);
            shared.update(|report| {
                report.screenshot_saved = true;
                report.foreground_pixels = foreground_pixels;
                report.foreground_coverage = foreground_coverage;
            });
        }
        Err(error)
            if error.starts_with("GPU screenshot contains only the clear background:")
                || error.starts_with("GPU screenshot contains only the source outline pass:") =>
        {
            state.blank_capture_attempts = state.blank_capture_attempts.saturating_add(1);
            state.capture_issued = false;
            state.ready_since_frame = Some(state.frames);
            let retry_view = match config.blank_camera_retry {
                BlankCameraRetry::Same => None,
                BlankCameraRetry::Opposite if state.camera_retry_count == 0 => {
                    Some(state.camera_view.opposite())
                }
                BlankCameraRetry::Opposite => None,
                BlankCameraRetry::AllAxes if state.camera_retry_count < 7 => {
                    Some(state.camera_view.next_diagnostic())
                }
                BlankCameraRetry::AllAxes => None,
            };
            if let Some(retry_view) = retry_view {
                state.camera_retry_count = state.camera_retry_count.saturating_add(1);
                state.camera_view = retry_view;
                state.camera_framed = false;
            }
            shared.update(|report| {
                report.capture_issued = false;
                report.blank_capture_attempts = state.blank_capture_attempts;
                report.camera_view = state.camera_view;
            });
        }
        Err(error) => fail_runtime(&mut state, &shared, &mut app_exit, error),
    }
}

/// Atomically publishes one new regular file without replacing an existing
/// path. JSON is the transaction commit marker for the already-published PNG;
/// an interrupted process may leave an orphan PNG, which the independent audit
/// rejects. Ordinary errors roll back every final path created by this run.
pub(super) fn atomic_write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "immutable GPU evidence output already exists: {}",
            path.display()
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("GPU evidence path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "cannot create GPU evidence directory {}: {error}",
            parent.display()
        )
    })?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("GPU evidence filename is not UTF-8: {}", path.display()))?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            format!(
                "cannot create GPU evidence temporary file {}: {error}",
                temporary.display()
            )
        })?;
    let write_result = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| {
            format!(
                "cannot write GPU evidence temporary file {}: {error}",
                temporary.display()
            )
        });
    drop(file);
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if let Err(error) = fs::hard_link(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "cannot atomically publish new GPU evidence {}: {error}",
            path.display()
        ));
    }
    if let Err(error) = fs::remove_file(&temporary) {
        let rollback = fs::remove_file(path);
        return Err(match rollback {
            Ok(()) => format!(
                "cannot remove GPU evidence temporary link {}: {error}; final path rolled back",
                temporary.display()
            ),
            Err(rollback_error) => format!(
                "cannot remove GPU evidence temporary link {}: {error}; cannot roll back {}: {rollback_error}",
                temporary.display(),
                path.display()
            ),
        });
    }
    Ok(())
}
