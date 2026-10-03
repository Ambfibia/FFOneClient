use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AuditGender {
    Male,
    Female,
}

impl AuditGender {
    pub(super) const fn rig(self) -> PlayerRigGender {
        match self {
            Self::Male => PlayerRigGender::Male,
            Self::Female => PlayerRigGender::Female,
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
        }
    }
}

#[derive(Resource)]
pub(super) struct AuditData {
    pub(super) data: Arc<CharacterCreationData>,
    pub(super) male: ResolvedCreatorSelection,
    pub(super) female: ResolvedCreatorSelection,
}

#[derive(Resource)]
pub(super) struct AuditState {
    pub(super) cli: Cli,
    pub(super) jobs: Vec<Job>,
    pub(super) cursor: usize,
    pub(super) phase: Phase,
    pub(super) results: Vec<ItemResult>,
    pub(super) capture_complete: Option<CaptureOutcome>,
    pub(super) transient_retries: u8,
    pub(super) started: Instant,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_audit(
    mut commands: Commands,
    data: Res<AuditData>,
    mut state: ResMut<AuditState>,
    mut preview: ResMut<NativePlayerPreviewModel>,
    audit: Res<NativePlayerPreviewAudit>,
    mut cache: ResMut<NativePlayerRigAssetCache>,
    mut exit: MessageWriter<AppExit>,
) {
    let phase = std::mem::replace(&mut state.phase, Phase::Finished);
    match phase {
        Phase::Start => {
            if state.cursor >= state.jobs.len() {
                state.phase = Phase::Finished;
                if let Err(error) = write_report(&state) {
                    eprintln!("cannot write final audit report: {error}");
                    exit.write(AppExit::error());
                } else {
                    let passed = state.results.iter().filter(|result| result.passed).count();
                    let failed = state.results.len().saturating_sub(passed);
                    println!(
                        "avatar equipment GPU audit complete: total={}, passed={passed}, failed={failed}, report={}",
                        state.results.len(),
                        state.cli.report.display()
                    );
                    exit.write(if failed == 0 {
                        AppExit::Success
                    } else {
                        AppExit::error()
                    });
                }
                return;
            }
            let job = state.jobs[state.cursor].clone();
            match resolve_job_look(&data, &job) {
                Ok((mut look, target_kind)) => {
                    let identity = job.identity();
                    look.identity = identity.clone();
                    preview.visible = true;
                    preview.stage = NativePlayerPreviewStage::Selection;
                    preview.yaw_degrees = 20.0;
                    if let Err(error) = preview.set_look(look.clone()) {
                        finish_result(&mut state, resolution_failure(&job, error), &mut cache);
                    } else {
                        state.phase = Phase::Loading {
                            started: Instant::now(),
                            frames: 0,
                            identity,
                            look,
                            target_kind,
                        };
                    }
                }
                Err(error) => {
                    finish_result(&mut state, resolution_failure(&job, error), &mut cache)
                }
            }
        }
        Phase::Loading {
            started,
            frames,
            identity,
            look,
            target_kind,
        } => {
            let frames = frames.saturating_add(1);
            match &preview.status {
                NativePlayerPreviewStatus::ReadyAnimated { .. } => {
                    state.phase = Phase::ReadyWarmup {
                        started,
                        frames,
                        ready_frames: 0,
                        identity,
                        look,
                        target_kind,
                    };
                }
                NativePlayerPreviewStatus::Blocked(error) => {
                    let job = state.jobs[state.cursor].clone();
                    finish_result(
                        &mut state,
                        blocked_result(&job, &look, target_kind, error.clone(), frames, started),
                        &mut cache,
                    );
                }
                _ if started.elapsed() >= ITEM_TIMEOUT => {
                    let job = state.jobs[state.cursor].clone();
                    if state.transient_retries < MAX_TRANSIENT_RETRIES {
                        state.transient_retries += 1;
                        cache.release_cached_handles();
                        preview.clear_look();
                        let mut retry_look = look;
                        let identity =
                            format!("{}-retry{}", job.identity(), state.transient_retries);
                        retry_look.identity = identity.clone();
                        if let Err(error) = preview.set_look(retry_look.clone()) {
                            finish_result(&mut state, resolution_failure(&job, error), &mut cache);
                        } else {
                            state.phase = Phase::Loading {
                                started: Instant::now(),
                                frames: 0,
                                identity,
                                look: retry_look,
                                target_kind,
                            };
                        }
                    } else {
                        let retries = state.transient_retries;
                        finish_result(
                            &mut state,
                            blocked_result(
                                &job,
                                &look,
                                target_kind,
                                format!(
                                    "preview timed out after {} retry: {}",
                                    retries,
                                    preview.loading_detail.as_deref().unwrap_or("no detail")
                                ),
                                frames,
                                started,
                            ),
                            &mut cache,
                        );
                    }
                }
                _ => {
                    state.phase = Phase::Loading {
                        started,
                        frames,
                        identity,
                        look,
                        target_kind,
                    };
                }
            }
        }
        Phase::ReadyWarmup {
            started,
            frames,
            ready_frames,
            identity,
            look,
            target_kind,
        } => {
            let frames = frames.saturating_add(1);
            let ready_frames = ready_frames.saturating_add(1);
            if !matches!(
                preview.status,
                NativePlayerPreviewStatus::ReadyAnimated { .. }
            ) {
                state.phase = Phase::Loading {
                    started,
                    frames,
                    identity,
                    look,
                    target_kind,
                };
            } else if ready_frames
                < if state.cursor == 0 {
                    FIRST_CAPTURE_RENDER_FRAMES
                } else {
                    READY_RENDER_FRAMES
                }
            {
                state.phase = Phase::ReadyWarmup {
                    started,
                    frames,
                    ready_frames,
                    identity,
                    look,
                    target_kind,
                };
            } else {
                let job = state.jobs[state.cursor].clone();
                let result = evaluate_ready_job(
                    &job,
                    &look,
                    target_kind,
                    &identity,
                    &audit,
                    frames,
                    started,
                );
                let capture = state.cli.screenshots == ScreenshotMode::All || !result.passed;
                if capture {
                    let path = job.screenshot_path(&state.cli.output_root);
                    if let Some(parent) = path.parent()
                        && let Err(error) = fs::create_dir_all(parent)
                    {
                        let mut result = result;
                        result
                            .failure_reasons
                            .push(format!("cannot create screenshot directory: {error}"));
                        result.passed = false;
                        finish_result(&mut state, result, &mut cache);
                        return;
                    }
                    state.capture_complete = None;
                    commands
                        .spawn((
                            Screenshot::primary_window(),
                            PendingItemCapture { path: path.clone() },
                        ))
                        .observe(save_item_capture);
                    state.phase = Phase::Capturing { result, path };
                } else {
                    finish_result(&mut state, result, &mut cache);
                }
            }
        }
        Phase::Capturing { mut result, path } => {
            if state
                .capture_complete
                .as_ref()
                .is_some_and(|capture| capture.path == path)
            {
                let capture = state
                    .capture_complete
                    .take()
                    .expect("matching capture outcome exists");
                result.screenshot = Some(path.to_string_lossy().into_owned());
                result.capture_visible_pixels = Some(capture.visible_pixels);
                result.capture_total_pixels = Some(capture.total_pixels);
                if let Some(error) = capture.save_error {
                    result
                        .failure_reasons
                        .push(format!("cannot save screenshot: {error}"));
                }
                if capture.visible_pixels < MIN_CAPTURE_VISIBLE_PIXELS {
                    result.rendered = false;
                    result.failure_reasons.push(format!(
                        "captured frame is visually blank: only {} of {} pixels differ from the black background",
                        capture.visible_pixels, capture.total_pixels
                    ));
                }
                result.failure_reasons.sort();
                result.failure_reasons.dedup();
                result.passed = result.failure_reasons.is_empty();
                finish_result(&mut state, result, &mut cache);
            } else {
                state.phase = Phase::Capturing { result, path };
            }
        }
        Phase::Finished => {
            state.phase = Phase::Finished;
        }
    }
}
