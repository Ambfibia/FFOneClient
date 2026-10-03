use super::*;

impl TutorialChoreographyPresentation {
    pub fn begin_scene(&mut self, scene: TutorialScene) {
        self.scene = Some(scene);
        self.event_scene = true;
        self.movement_locked = true;
        // `LookAtPosition` belongs to the terminal edge of the preceding
        // coroutine. Do not let that one-shot target leak into a later scene.
        self.camera.look_at = None;
        self.camera.resolved_look_at = None;
        self.camera.shake_start_offset = Vec3::ZERO;
        self.camera.shake_target_offset = Vec3::ZERO;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn apply(&mut self, event: ChoreographyPlaybackEvent) {
        match event {
            ChoreographyPlaybackEvent::Action { action, .. } => self.apply_action(action),
            ChoreographyPlaybackEvent::SequenceSample { sample, .. } => {
                self.apply_sequence_sample(sample);
            }
            ChoreographyPlaybackEvent::FinalSkipFade { fade } => match fade {
                SkipFinalFade::GameEventFadeIn => {
                    self.overlay_alpha = 0.0;
                    self.fade_enabled = false;
                }
                SkipFinalFade::OpaqueOverlay => {
                    self.overlay_alpha = 1.0;
                    self.fade_enabled = true;
                }
            },
            ChoreographyPlaybackEvent::Finished { .. } => {
                self.scene = None;
                self.event_scene = false;
                self.movement_locked = false;
                self.hud_hide_depth = 0;
                self.cinematic = false;
                self.cinematic_alpha = 0.0;
                // Event-camera interpolation is scene-scoped, but the final
                // `cnPlayerCamera.LookAtPosition` must survive this event long
                // enough for the camera adapter to copy its horizontal angle
                // into the normal orbit controller.
                let look_at = self.camera.look_at;
                let resolved_look_at = self.camera.resolved_look_at;
                let freeze_target_revision = self.camera.freeze_target_revision;
                let look_at_revision = self.camera.look_at_revision;
                self.camera = TutorialCameraPresentation::default();
                self.camera.look_at = look_at;
                self.camera.resolved_look_at = resolved_look_at;
                self.camera.freeze_target_revision = freeze_target_revision;
                self.camera.look_at_revision = look_at_revision;
            }
            ChoreographyPlaybackEvent::BlockingWaitReached { .. } => {}
        }
    }

    pub(super) fn apply_action(&mut self, action: ChoreographyAction) {
        match action {
            ChoreographyAction::Hud(HudAction::PushHide) => {
                self.hud_hide_depth = self.hud_hide_depth.saturating_add(1);
            }
            ChoreographyAction::Hud(HudAction::PopShow) => {
                self.hud_hide_depth = self.hud_hide_depth.saturating_sub(1);
            }
            ChoreographyAction::SpatialAudio(target) => {
                self.spatial_audio_target = target;
            }
            ChoreographyAction::EventScene(value) => {
                self.event_scene = value;
                self.movement_locked = value;
            }
            ChoreographyAction::Cinematic(value) => {
                self.cinematic = value;
                if !value {
                    self.cinematic_alpha = 0.0;
                }
            }
            ChoreographyAction::FadeEnabled(value) => {
                self.fade_enabled = value;
                if !value {
                    self.overlay_alpha = 0.0;
                }
            }
            ChoreographyAction::GameFadeIn => {
                self.fade_enabled = false;
                self.overlay_alpha = 0.0;
            }
            ChoreographyAction::SubtitleClear => {
                self.subtitle_key = None;
                self.subtitle_visible_characters = 0;
                self.subtitle_alpha = 0.0;
            }
            ChoreographyAction::Player(PlayerAction::Hide) => {
                self.player_hidden = true;
            }
            ChoreographyAction::Player(PlayerAction::Show) => {
                self.player_hidden = false;
            }
            ChoreographyAction::Player(PlayerAction::SetTemporaryNanoAbsent) => {
                self.temporary_nano_absent = true;
            }
            ChoreographyAction::Nano(NanoAction::SetVoiceDisabled(value)) => {
                self.nano_voice_disabled = value;
            }
            ChoreographyAction::Nano(NanoAction::DestroyPresentationObject) => {
                self.temporary_nano_absent = true;
            }
            ChoreographyAction::Equipment(_) => {
                self.equipment_requested = true;
            }
            ChoreographyAction::Pan(action) => match action {
                PanAction::LoadEightTextures => self.pan_loaded = true,
                PanAction::Start => {
                    self.pan_active = true;
                    self.pan_elapsed_seconds = 0.0;
                }
                PanAction::Stop => self.pan_active = false,
                PanAction::ReleaseEightTextures => {
                    self.pan_active = false;
                    self.pan_loaded = false;
                }
            },
            ChoreographyAction::Camera(action) => self.apply_camera_action(action),
            ChoreographyAction::VoiceOff
            | ChoreographyAction::StopBgm
            | ChoreographyAction::Npc(_)
            | ChoreographyAction::Player(_)
            | ChoreographyAction::Effect(_)
            | ChoreographyAction::Sequence(_)
            | ChoreographyAction::Loop(_)
            | ChoreographyAction::Nano(_)
            | ChoreographyAction::Progress(_)
            | ChoreographyAction::Projectile(_)
            | ChoreographyAction::Picture(_)
            | ChoreographyAction::Sound(_)
            | ChoreographyAction::Unresolved(_) => {}
        }
    }

    pub(super) fn apply_camera_action(&mut self, action: CameraAction) {
        match action {
            CameraAction::Mode(mode) => self.camera.mode = mode,
            CameraAction::CurrentTarget(target) => {
                self.camera.current_target = Some(target);
                self.camera.resolved_current_target = None;
            }
            CameraAction::Target(target) => {
                self.camera.target = Some(target);
                self.camera.resolved_target = None;
                // Assigning kNewTargetPosition replaces the last shake sample.
                self.camera.shake_target_offset = Vec3::ZERO;
            }
            CameraAction::FreezeCurrentTarget => {
                self.camera.freeze_target_revision =
                    self.camera.freeze_target_revision.wrapping_add(1);
                self.camera.shake_target_offset = Vec3::ZERO;
            }
            CameraAction::Start(start) => {
                self.camera.start = Some(start);
                self.camera.resolved_start = None;
                // Assigning kNewStartPosition replaces the last shake sample.
                self.camera.shake_start_offset = Vec3::ZERO;
            }
            CameraAction::StoredStart(start) => {
                self.camera.stored_start = Some(start);
                self.camera.resolved_stored_start = None;
            }
            CameraAction::TargetRotation(rotation) => {
                self.camera.target_rotation = Some(rotation);
                self.camera.resolved_target_rotation = None;
            }
            CameraAction::Distance(distance) => self.camera.distance = distance,
            CameraAction::ForwardInterpolation(value) => {
                self.camera.forward_interpolation = value;
            }
            CameraAction::LookAt(target) => {
                self.camera.look_at = Some(target);
                self.camera.resolved_look_at = None;
                self.camera.look_at_revision = self.camera.look_at_revision.wrapping_add(1);
            }
            // Capturing the live ECS camera transform belongs to the adapter
            // which consumes the same playback event.
            CameraAction::CaptureTransform(_) => {}
        }
    }

    pub(super) fn apply_sequence_sample(&mut self, sample: FrameSequenceSample) {
        match sample {
            FrameSequenceSample::Fade { channel, value, .. } => match channel {
                FadeChannel::Overlay => self.overlay_alpha = value.clamp(0.0, 1.0),
                FadeChannel::CinematicBars => {
                    self.cinematic_alpha = value.clamp(0.0, 1.0);
                }
                FadeChannel::Subtitle => self.subtitle_alpha = value.clamp(0.0, 1.0),
            },
            FrameSequenceSample::SubtitleTypewriter {
                localization_key,
                visible_characters,
                ..
            } => {
                self.subtitle_key = Some(localization_key);
                self.subtitle_visible_characters = visible_characters;
                self.subtitle_alpha = 1.0;
            }
            FrameSequenceSample::CameraShake {
                start_offset,
                target_offset,
                ..
            } => {
                self.camera.shake_start_offset = client_vec3(start_offset);
                self.camera.shake_target_offset = client_vec3(target_offset);
            }
            FrameSequenceSample::PlayerPathAndFade { fade_alpha, .. } => {
                self.overlay_alpha = fade_alpha.clamp(0.0, 1.0);
            }
            FrameSequenceSample::NpcDelta { .. } | FrameSequenceSample::NpcLerp { .. } => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum TutorialChoreographyOverlay {
    Fade,
    TopBar,
    BottomBar,
}

#[derive(Debug, Default, Resource)]
pub(super) struct TutorialPanAssets {
    pub(super) handles: Option<[Handle<Image>; 8]>,
}
