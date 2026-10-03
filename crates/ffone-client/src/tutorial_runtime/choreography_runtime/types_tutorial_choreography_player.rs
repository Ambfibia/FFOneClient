use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoreographyCompletion {
    Natural,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TutorialChoreographyIssue {
    EffectUnavailable {
        source_line: u32,
        effect_id: Option<i32>,
        operation: &'static str,
    },
    ProjectileUnavailable {
        source_line: u32,
        projectile: ProjectileAction,
    },
    PanUnavailable {
        source_line: u32,
        action: PanAction,
    },
    RigAnimationUnavailable {
        source_line: u32,
        target: RigAnimationTarget,
        clip: &'static str,
    },
    UnresolvedReferenceAction {
        source_line: u32,
        detail: &'static str,
    },
    BlockingWaitUnavailable {
        source_line: u32,
        detail: &'static str,
    },
    ExpressionResolutionUnavailable {
        source_line: u32,
        error: ChoreographyFormulaError,
    },
    MissingEntityReference {
        source_line: u32,
        detail: &'static str,
    },
    EquipmentPresentationUnavailable {
        source_line: u32,
        action: EquipmentAction,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TutorialChoreographyIssueQueue {
    pub(super) pending: VecDeque<TutorialChoreographyIssue>,
}

impl TutorialChoreographyIssueQueue {
    pub fn push(&mut self, issue: TutorialChoreographyIssue) {
        if !self.pending.contains(&issue) {
            self.pending.push_back(issue);
        }
    }

    pub fn extend(&mut self, issues: impl IntoIterator<Item = TutorialChoreographyIssue>) {
        for issue in issues {
            self.push(issue);
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<TutorialChoreographyIssue> {
        self.pending.pop_front()
    }

    pub fn take_all(&mut self) -> VecDeque<TutorialChoreographyIssue> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialLoopOwner {
    pub scene: TutorialScene,
    pub persists_after_scene: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ActiveSequence {
    pub(super) source_line: u32,
    pub(super) start_seconds: f32,
    pub(super) sequence: FrameSequence,
    pub(super) next_frame: u16,
    pub(super) ordinal: u32,
}

#[derive(Debug)]
pub(super) struct ActivePlayback {
    pub(super) choreography: &'static TutorialSceneChoreography,
    pub(super) elapsed_seconds: f32,
    pub(super) cursor: usize,
    pub(super) blocking_wait_cursor: usize,
    pub(super) pending_blocking_wait: Option<BlockingWait>,
    pub(super) retry_attempt_counts: BTreeMap<&'static str, u8>,
    pub(super) pending_retry_delay_seconds: f32,
    pub(super) pending_retry_action_issued: bool,
    pub(super) next_sequence_ordinal: u32,
    pub(super) sequences: Vec<ActiveSequence>,
}

impl ActivePlayback {
    pub(super) fn new(choreography: &'static TutorialSceneChoreography) -> Self {
        Self {
            choreography,
            elapsed_seconds: 0.0,
            cursor: 0,
            blocking_wait_cursor: 0,
            pending_blocking_wait: None,
            retry_attempt_counts: BTreeMap::new(),
            pending_retry_delay_seconds: 0.0,
            pending_retry_action_issued: false,
            next_sequence_ordinal: 0,
            sequences: Vec::new(),
        }
    }
}

/// Scene cursor and loop ownership for the currently active finite tutorial
/// coroutine.
#[derive(Debug, Default, Resource)]
pub struct TutorialChoreographyPlayer {
    pub(super) active: Option<ActivePlayback>,
    pub(super) loop_owners: BTreeMap<&'static str, TutorialLoopOwner>,
    pub(super) issues: VecDeque<TutorialChoreographyIssue>,
}

impl TutorialChoreographyPlayer {
    #[must_use]
    pub fn active_scene(&self) -> Option<TutorialScene> {
        self.active
            .as_ref()
            .map(|playback| playback.choreography.scene)
    }

    #[must_use]
    pub fn elapsed_seconds(&self) -> f32 {
        self.active
            .as_ref()
            .map_or(0.0, |playback| playback.elapsed_seconds)
    }

    /// Logical clock for the separate audio/subtitle presenter.
    ///
    /// A wait is reached after its preceding source actions but before any
    /// presentation cue at the same timestamp. Holding just below that
    /// timestamp prevents those cues from running until the wait resolves.
    #[must_use]
    pub fn presentation_elapsed_seconds(&self) -> f32 {
        self.active.as_ref().map_or(0.0, |playback| {
            playback
                .pending_blocking_wait
                .map_or(playback.elapsed_seconds, |wait| {
                    (blocking_wait_start(wait) - TIME_EPSILON * 2.0).max(0.0)
                })
        })
    }

    #[must_use]
    pub fn is_loop_owned(&self, cue: &str) -> bool {
        self.loop_owners.contains_key(cue)
    }

    pub fn loop_owners(
        &self,
    ) -> impl ExactSizeIterator<Item = (&'static str, TutorialLoopOwner)> + '_ {
        self.loop_owners
            .iter()
            .map(|(&cue, &ownership)| (cue, ownership))
    }

    /// Starts `scene` and returns every source action due at time zero.
    ///
    /// Repeating the same start request is idempotent, which prevents the
    /// stage driver and a `StartScene` intent from spawning the same actors
    /// twice in one frame.
    pub fn start(&mut self, scene: TutorialScene) -> Vec<ChoreographyPlaybackEvent> {
        if self.active_scene() == Some(scene) {
            return Vec::new();
        }
        self.end_active_scene(false);
        let Some(choreography) = tutorial_scene_choreography(scene) else {
            self.active = None;
            return Vec::new();
        };
        let mut playback = ActivePlayback::new(choreography);
        let reached_wait =
            next_blocking_wait(&playback).filter(|wait| blocking_wait_start(*wait) <= TIME_EPSILON);
        let events = self.collect_until(&mut playback, 0.0, reached_wait);
        playback.pending_blocking_wait = reached_wait;
        self.active = Some(playback);
        events
    }

    /// Advances the active scene. A skip is handled before elapsed time is
    /// advanced and executes the complete reference skip contract atomically.
    pub fn tick(&mut self, delta_seconds: f32, skip: bool) -> Vec<ChoreographyPlaybackEvent> {
        self.tick_with_wait_resolution(delta_seconds, skip, |_| true)
    }

    /// Advances the active scene while resolving its exact blocking waits
    /// against live native state.
    ///
    /// Choreography time does not advance while an asset preload is pending.
    /// The two finale retries share the legacy `num` budget, so together they
    /// can add at most one second to the scene.
    pub fn tick_with_wait_resolution<F>(
        &mut self,
        delta_seconds: f32,
        skip: bool,
        mut wait_is_resolved: F,
    ) -> Vec<ChoreographyPlaybackEvent>
    where
        F: FnMut(BlockingWait) -> bool,
    {
        if skip {
            return self.skip();
        }
        let Some(mut playback) = self.active.take() else {
            return Vec::new();
        };
        let mut remaining_seconds = delta_seconds.max(0.0);
        let mut events = Vec::new();

        loop {
            if let Some(wait) = playback.pending_blocking_wait {
                match wait {
                    BlockingWait::AssetPreload { .. } => {
                        if wait_is_resolved(wait) {
                            playback.pending_blocking_wait = None;
                        } else {
                            self.active = Some(playback);
                            return events;
                        }
                    }
                    BlockingWait::EffectInstantiationRetry {
                        retry_source_line,
                        retry_action,
                        retry_interval_seconds,
                        maximum_attempts,
                        shared_attempt_counter,
                        ..
                    } => {
                        if playback.pending_retry_action_issued {
                            let available = (retry_interval_seconds.max(0.0)
                                - playback.pending_retry_delay_seconds)
                                .max(0.0);
                            let consumed = remaining_seconds.min(available);
                            playback.pending_retry_delay_seconds += consumed;
                            remaining_seconds -= consumed;
                            if playback.pending_retry_delay_seconds + TIME_EPSILON
                                < retry_interval_seconds
                            {
                                self.active = Some(playback);
                                return events;
                            }
                            playback.pending_retry_delay_seconds = 0.0;
                            playback.pending_retry_action_issued = false;
                        }

                        if wait_is_resolved(wait) {
                            playback.pending_blocking_wait = None;
                        } else {
                            let attempt_count = playback
                                .retry_attempt_counts
                                .entry(shared_attempt_counter)
                                .or_default();
                            if *attempt_count >= maximum_attempts {
                                playback.pending_blocking_wait = None;
                                continue;
                            }
                            *attempt_count += 1;
                            self.push_action_event(
                                &mut events,
                                playback.choreography.scene,
                                retry_source_line,
                                ChoreographyActionOrigin::Timeline,
                                retry_action,
                            );
                            playback.pending_retry_action_issued = true;
                            // The source yields immediately after every retry,
                            // including the retry which succeeds. Its 0.1 s
                            // timer begins after this frame applies the Add.
                            self.active = Some(playback);
                            return events;
                        }
                    }
                }
            }

            let requested_end = playback.elapsed_seconds + remaining_seconds;
            let reached_wait = next_blocking_wait(&playback)
                .filter(|wait| blocking_wait_start(*wait) <= requested_end + TIME_EPSILON);
            let effective_end = reached_wait.map_or(requested_end, |wait| {
                blocking_wait_start(wait).min(requested_end)
            });
            let timeline_advance = (effective_end - playback.elapsed_seconds).max(0.0);
            events.extend(self.collect_until(&mut playback, effective_end, reached_wait));
            playback.elapsed_seconds = effective_end;
            remaining_seconds = (remaining_seconds - timeline_advance).max(0.0);

            if let Some(wait) = reached_wait {
                playback.pending_blocking_wait = Some(wait);
                if matches!(wait, BlockingWait::EffectInstantiationRetry { .. }) {
                    // The initial Add events in this batch are applied after
                    // the player tick. Observe their real disposition on the
                    // next frame before deciding whether a retry is needed.
                    self.active = Some(playback);
                    return events;
                }
                continue;
            }
            break;
        }

        let duration = playback.choreography.deterministic_duration_seconds;
        if playback.elapsed_seconds + TIME_EPSILON >= duration {
            let scene = playback.choreography.scene;
            self.remove_non_persistent_loops(scene);
            events.push(ChoreographyPlaybackEvent::Finished {
                scene,
                completion: ChoreographyCompletion::Natural,
            });
        } else {
            self.active = Some(playback);
        }
        events
    }

    /// Executes scene-specific cleanup, common cleanup and the final fade in
    /// exactly that order.
    pub fn skip(&mut self) -> Vec<ChoreographyPlaybackEvent> {
        let Some(playback) = self.active.take() else {
            return Vec::new();
        };
        let scene = playback.choreography.scene;
        let contract = playback.choreography.skip;
        let mut events =
            Vec::with_capacity(contract.scene_specific.len() + contract.common_cleanup.len() + 2);
        for sourced in contract.scene_specific {
            self.push_action_event(
                &mut events,
                scene,
                sourced.source_line,
                ChoreographyActionOrigin::SkipSceneSpecific,
                sourced.action,
            );
        }
        for sourced in contract.common_cleanup {
            self.push_action_event(
                &mut events,
                scene,
                sourced.source_line,
                ChoreographyActionOrigin::SkipCommonCleanup,
                sourced.action,
            );
        }
        events.push(ChoreographyPlaybackEvent::FinalSkipFade {
            fade: contract.final_fade,
        });
        self.remove_non_persistent_loops(scene);
        events.push(ChoreographyPlaybackEvent::Finished {
            scene,
            completion: ChoreographyCompletion::Skipped,
        });
        events
    }

    pub fn stop(&mut self) {
        self.end_active_scene(false);
    }

    pub fn reset(&mut self) {
        self.active = None;
        self.loop_owners.clear();
        self.issues.clear();
    }

    pub fn stop_all_loops(&mut self) {
        self.loop_owners.clear();
    }

    pub fn report_issue(&mut self, issue: TutorialChoreographyIssue) {
        if !self.issues.contains(&issue) {
            self.issues.push_back(issue);
        }
    }

    pub fn take_issues(&mut self) -> VecDeque<TutorialChoreographyIssue> {
        std::mem::take(&mut self.issues)
    }

    pub(super) fn collect_until(
        &mut self,
        playback: &mut ActivePlayback,
        end_seconds: f32,
        reached_wait: Option<BlockingWait>,
    ) -> Vec<ChoreographyPlaybackEvent> {
        let mut scheduled = Vec::new();

        while let Some(action) = playback.choreography.actions.get(playback.cursor)
            && action.at_seconds <= end_seconds + TIME_EPSILON
        {
            if reached_wait.is_some_and(|wait| {
                (action.at_seconds - blocking_wait_start(wait)).abs() <= TIME_EPSILON
                    && action.source_line >= blocking_wait_continuation_source_line(wait)
            }) {
                break;
            }
            let action_index = playback.cursor;
            scheduled.push(ScheduledEvent {
                at_seconds: action.at_seconds,
                same_time_order: action_index as u64,
                event: ChoreographyPlaybackEvent::Action {
                    source_line: action.source_line,
                    origin: ChoreographyActionOrigin::Timeline,
                    action: action.action,
                },
            });
            if let ChoreographyAction::Sequence(sequence) = action.action {
                playback.sequences.push(ActiveSequence {
                    source_line: action.source_line,
                    start_seconds: action.at_seconds,
                    sequence,
                    next_frame: 0,
                    ordinal: playback.next_sequence_ordinal,
                });
                playback.next_sequence_ordinal = playback.next_sequence_ordinal.saturating_add(1);
            }
            playback.cursor += 1;
        }

        if let Some(wait) = reached_wait {
            let reached_at_seconds = blocking_wait_start(wait);
            scheduled.push(ScheduledEvent {
                at_seconds: reached_at_seconds,
                same_time_order: u64::MAX - 2,
                event: ChoreographyPlaybackEvent::BlockingWaitReached { wait },
            });
            playback.blocking_wait_cursor += 1;
        }

        for sequence in &mut playback.sequences {
            let frame_count = sequence_frame_count(sequence.sequence);
            while sequence.next_frame < frame_count {
                let frame = sequence.next_frame;
                let sample_seconds =
                    sequence.start_seconds + sequence_frame_delay(sequence.sequence, frame);
                if sample_seconds > end_seconds + TIME_EPSILON {
                    break;
                }
                scheduled.push(ScheduledEvent {
                    at_seconds: sample_seconds,
                    // Every source action at the same timestamp runs before
                    // the first body iteration of a recovered frame loop.
                    same_time_order: (1_u64 << 48)
                        + (u64::from(sequence.ordinal) << 16)
                        + u64::from(frame),
                    event: ChoreographyPlaybackEvent::SequenceSample {
                        source_line: sequence.source_line,
                        sample: sample_sequence(sequence.sequence, frame, sample_seconds),
                    },
                });
                sequence.next_frame += 1;
            }
        }
        playback
            .sequences
            .retain(|sequence| sequence.next_frame < sequence_frame_count(sequence.sequence));

        scheduled.sort_by(|left, right| {
            left.at_seconds
                .partial_cmp(&right.at_seconds)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.same_time_order.cmp(&right.same_time_order))
        });

        let mut events = Vec::with_capacity(scheduled.len());
        for scheduled in scheduled {
            self.observe_event(playback.choreography.scene, scheduled.event);
            events.push(scheduled.event);
        }
        events
    }

    pub(super) fn push_action_event(
        &mut self,
        events: &mut Vec<ChoreographyPlaybackEvent>,
        scene: TutorialScene,
        source_line: u32,
        origin: ChoreographyActionOrigin,
        action: ChoreographyAction,
    ) {
        let event = ChoreographyPlaybackEvent::Action {
            source_line,
            origin,
            action,
        };
        self.observe_event(scene, event);
        events.push(event);
        if let ChoreographyAction::Sequence(sequence) = action {
            let final_frame = sequence_frame_count(sequence).saturating_sub(1);
            events.push(ChoreographyPlaybackEvent::SequenceSample {
                source_line,
                sample: sample_sequence(sequence, final_frame, 0.0),
            });
        }
    }

    pub(super) fn observe_event(&mut self, scene: TutorialScene, event: ChoreographyPlaybackEvent) {
        match event {
            ChoreographyPlaybackEvent::Action {
                source_line,
                action,
                ..
            } => {
                self.observe_loop_action(scene, action);
                if let Some(issue) = unavailable_action_issue(source_line, action) {
                    self.report_issue(issue);
                }
            }
            ChoreographyPlaybackEvent::BlockingWaitReached { .. }
            | ChoreographyPlaybackEvent::SequenceSample { .. }
            | ChoreographyPlaybackEvent::FinalSkipFade { .. }
            | ChoreographyPlaybackEvent::Finished { .. } => {}
        }
    }

    pub(super) fn observe_loop_action(&mut self, scene: TutorialScene, action: ChoreographyAction) {
        let ChoreographyAction::Loop(loop_action) = action else {
            return;
        };
        match loop_action {
            LoopAction::Start(cue) | LoopAction::StartIfAbsent(cue) => {
                let persists_after_scene = tutorial_scene_choreography(scene)
                    .and_then(|choreography| {
                        choreography
                            .loops
                            .iter()
                            .find(|lifetime| lifetime.cue == cue)
                    })
                    .is_some_and(|lifetime| lifetime.persists_after_scene);
                self.loop_owners.entry(cue).or_insert(TutorialLoopOwner {
                    scene,
                    persists_after_scene,
                });
            }
            LoopAction::StopIfPresent => {
                self.loop_owners
                    .retain(|_, ownership| ownership.scene != scene);
            }
        }
    }

    pub(super) fn end_active_scene(&mut self, preserve_persistent: bool) {
        let Some(playback) = self.active.take() else {
            return;
        };
        if preserve_persistent {
            self.remove_non_persistent_loops(playback.choreography.scene);
        } else {
            let scene = playback.choreography.scene;
            self.loop_owners
                .retain(|_, ownership| ownership.scene != scene);
        }
    }

    pub(super) fn remove_non_persistent_loops(&mut self, scene: TutorialScene) {
        self.loop_owners
            .retain(|_, ownership| ownership.scene != scene || ownership.persists_after_scene);
    }
}

#[derive(Debug, Clone)]
pub struct TutorialCameraPresentation {
    pub mode: CameraMode,
    pub current_target: Option<PositionExpr>,
    pub target: Option<PositionExpr>,
    pub start: Option<PositionExpr>,
    pub stored_start: Option<PositionExpr>,
    pub target_rotation: Option<RotationExpr>,
    pub distance: f32,
    pub forward_interpolation: bool,
    pub look_at: Option<PositionExpr>,
    pub resolved_current_target: Option<Vec3>,
    pub resolved_target: Option<Vec3>,
    pub resolved_start: Option<Vec3>,
    pub resolved_stored_start: Option<Vec3>,
    pub resolved_target_rotation: Option<Quat>,
    pub resolved_look_at: Option<Vec3>,
    /// Monotonic serial for `kNewTargetPosition = kCTargetPosition`.
    pub freeze_target_revision: u64,
    /// Monotonic serial for `cnPlayerCamera.LookAtPosition`. The legacy call
    /// mutates the orbit yaw once; it is not a persistent cutscene look-at.
    pub look_at_revision: u64,
    pub shake_start_offset: Vec3,
    pub shake_target_offset: Vec3,
}

impl Default for TutorialCameraPresentation {
    fn default() -> Self {
        Self {
            mode: CameraMode::None,
            current_target: None,
            target: None,
            start: None,
            stored_start: None,
            target_rotation: None,
            distance: 5.0,
            forward_interpolation: true,
            look_at: None,
            resolved_current_target: None,
            resolved_target: None,
            resolved_start: None,
            resolved_stored_start: None,
            resolved_target_rotation: None,
            resolved_look_at: None,
            freeze_target_revision: 0,
            look_at_revision: 0,
            shake_start_offset: Vec3::ZERO,
            shake_target_offset: Vec3::ZERO,
        }
    }
}

/// Renderer-facing state produced by choreography events.
#[derive(Debug, Resource)]
pub struct TutorialChoreographyPresentation {
    pub scene: Option<TutorialScene>,
    pub event_scene: bool,
    pub movement_locked: bool,
    pub hud_hide_depth: u16,
    pub cinematic: bool,
    pub fade_enabled: bool,
    pub overlay_alpha: f32,
    pub cinematic_alpha: f32,
    pub subtitle_alpha: f32,
    pub subtitle_key: Option<&'static str>,
    pub subtitle_visible_characters: u16,
    pub spatial_audio_target: SpatialAudioTarget,
    pub player_hidden: bool,
    pub nano_voice_disabled: bool,
    pub temporary_nano_absent: bool,
    pub equipment_requested: bool,
    pub pan_loaded: bool,
    pub pan_active: bool,
    pub pan_elapsed_seconds: f32,
    pub camera: TutorialCameraPresentation,
}

impl Default for TutorialChoreographyPresentation {
    fn default() -> Self {
        Self {
            scene: None,
            event_scene: false,
            movement_locked: false,
            hud_hide_depth: 0,
            cinematic: false,
            fade_enabled: false,
            overlay_alpha: 0.0,
            cinematic_alpha: 0.0,
            subtitle_alpha: 0.0,
            subtitle_key: None,
            subtitle_visible_characters: 0,
            spatial_audio_target: SpatialAudioTarget::Player,
            player_hidden: false,
            nano_voice_disabled: false,
            temporary_nano_absent: false,
            equipment_requested: false,
            pan_loaded: false,
            pan_active: false,
            pan_elapsed_seconds: 0.0,
            camera: TutorialCameraPresentation::default(),
        }
    }
}
