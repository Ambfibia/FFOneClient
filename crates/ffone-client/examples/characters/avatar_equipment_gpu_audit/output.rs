use super::*;

pub(super) fn save_item_capture(
    event: On<ScreenshotCaptured>,
    captures: Query<&PendingItemCapture>,
    mut state: ResMut<AuditState>,
) {
    let Ok(capture) = captures.get(event.event().entity) else {
        return;
    };
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("equipment audit screenshot converts to an image");
    let rgba = image.to_rgba8();
    let total_pixels = u64::from(rgba.width()) * u64::from(rgba.height());
    let visible_pixels = rgba
        .pixels()
        .filter(|pixel| pixel[3] != 0 && pixel[0].max(pixel[1]).max(pixel[2]) > 8)
        .count() as u64;
    let save_error = image.save(&capture.path).err().map(|error| {
        eprintln!(
            "cannot save equipment screenshot {}: {error}",
            capture.path.display()
        );
        error.to_string()
    });
    state.capture_complete = Some(CaptureOutcome {
        path: capture.path.clone(),
        visible_pixels,
        total_pixels,
        save_error,
    });
}

pub(super) fn write_report(state: &AuditState) -> Result<(), String> {
    let passed = state.results.iter().filter(|result| result.passed).count();
    let rendered = state
        .results
        .iter()
        .filter(|result| result.rendered)
        .count();
    let report = serde_json::json!({
        "schema": REPORT_SCHEMA,
        "assetRoot": state.cli.asset_root,
        "outputRoot": state.cli.output_root,
        "filters": {
            "category": state.cli.category.map(category_label),
            "gender": match state.cli.gender {
                GenderFilter::All => "all",
                GenderFilter::Male => "male",
                GenderFilter::Female => "female",
            },
            "start": state.cli.start,
            "limit": state.cli.limit,
            "screenshots": match state.cli.screenshots {
                ScreenshotMode::All => "all",
                ScreenshotMode::Failures => "failures",
            },
        },
        "counts": {
            "queued": state.jobs.len(),
            "completed": state.results.len(),
            "rendered": rendered,
            "passed": passed,
            "failed": state.results.len() - passed,
        },
        "elapsedSeconds": state.started.elapsed().as_secs_f64(),
        "complete": state.results.len() == state.jobs.len(),
        "results": state.results,
    });
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot serialize report: {error}"))?;
    fs::write(&state.cli.report, bytes)
        .map_err(|error| format!("cannot write {}: {error}", state.cli.report.display()))
}
