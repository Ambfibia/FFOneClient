use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct LauncherUiModel {
    pub phase: LauncherUiPhase,
    pub current_rotation_degrees: Vec3,
    pub start_rotation_degrees: Vec3,
    pub maximum_rotation_degrees: Vec3,
    pub current_power: f32,
    pub charging: bool,
    pub power_rising: bool,
    pub dismissal: Option<LauncherUiDismissalSource>,
    pub(super) trigger: Option<LauncherTriggerSpec>,
    pub(super) old_avatar_position: Vec3,
    pub(super) camera_position: Vec3,
}

impl Default for LauncherUiModel {
    fn default() -> Self {
        Self {
            phase: LauncherUiPhase::Hidden,
            current_rotation_degrees: Vec3::ZERO,
            start_rotation_degrees: Vec3::ZERO,
            maximum_rotation_degrees: Vec3::ZERO,
            current_power: 0.0,
            charging: false,
            power_rising: true,
            dismissal: None,
            trigger: None,
            old_avatar_position: Vec3::ZERO,
            camera_position: Vec3::ZERO,
        }
    }
}

impl LauncherUiModel {
    #[must_use]
    pub const fn visible(&self) -> bool {
        !matches!(self.phase, LauncherUiPhase::Hidden)
    }

    #[must_use]
    pub fn normalized_power(&self) -> f32 {
        let Some(trigger) = self.trigger else {
            return 0.0;
        };
        ((self.current_power - trigger.min_power) / (trigger.max_power - trigger.min_power))
            .clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn input_boundary(&self, system_popup_active: bool) -> LauncherUiInputBoundary {
        let visible = self.visible();
        LauncherUiInputBoundary {
            blocks_lower_ui: visible,
            blocks_gameplay_input: visible,
            requires_pointer: false,
            fire_enabled: visible && !system_popup_active,
            escape_enabled: matches!(self.phase, LauncherUiPhase::Aiming) && !system_popup_active,
        }
    }

    pub fn open(
        &mut self,
        trigger: LauncherTriggerSpec,
        old_avatar_position: Vec3,
        camera_height: f32,
        outbox: &mut LauncherUiOutbox,
    ) -> Result<(), LauncherUiOpenError> {
        let trigger = trigger.validate()?;
        if !vec3_is_finite(old_avatar_position) || !camera_height.is_finite() {
            return Err(LauncherUiOpenError::NonFiniteAvatarOrCamera);
        }
        let mut start_rotation = trigger.trigger_euler_degrees + trigger.initial_rotation_degrees;
        while start_rotation.y > 360.0 {
            start_rotation.y -= 360.0;
        }
        while start_rotation.y < 0.0 {
            start_rotation.y += 360.0;
        }
        let camera_position = trigger.trigger_position + Vec3::Y * camera_height;

        self.phase = LauncherUiPhase::Aiming;
        self.current_rotation_degrees = Vec3::ZERO;
        self.start_rotation_degrees = start_rotation;
        self.maximum_rotation_degrees = trigger.maximum_rotation_degrees;
        self.current_power = trigger.min_power;
        self.charging = false;
        self.power_rising = true;
        self.dismissal = None;
        self.trigger = Some(trigger);
        self.old_avatar_position = old_avatar_position;
        self.camera_position = camera_position;

        outbox.push(LauncherUiEffect::Audio(LauncherUiAudioCue::ClickOn));
        outbox.push(LauncherUiEffect::SetNameVisible(false));
        outbox.push(LauncherUiEffect::SetCameraPosition(camera_position));
        outbox.push(LauncherUiEffect::SetCameraCustomControl(true));
        outbox.push(LauncherUiEffect::SetTriggerRenderersVisible(false));
        outbox.push(LauncherUiEffect::SetAvatarRenderersVisible(false));
        Ok(())
    }

    /// Exact `cnLauncher.FixedUpdate` aim step. It is intentionally not
    /// blocked by a system popup because the clean popup guard is in Update.
    pub fn fixed_update_aim(&mut self, vertical_axis: f32, horizontal_axis: f32) {
        if !self.visible() || !vertical_axis.is_finite() || !horizontal_axis.is_finite() {
            return;
        }
        self.current_rotation_degrees.x -=
            vertical_axis * LAUNCHER_UI_ROTATE_SPEED_PER_FIXED_UPDATE;
        self.current_rotation_degrees.x = self
            .current_rotation_degrees
            .x
            .clamp(-self.maximum_rotation_degrees.x, 0.0);
        self.current_rotation_degrees.y +=
            horizontal_axis * LAUNCHER_UI_ROTATE_SPEED_PER_FIXED_UPDATE;
        if self.current_rotation_degrees.y.abs() > self.maximum_rotation_degrees.y {
            self.current_rotation_degrees.y =
                self.current_rotation_degrees.y.signum() * self.maximum_rotation_degrees.y;
        }
    }

    pub fn press_fire(&mut self, system_popup_active: bool, outbox: &mut LauncherUiOutbox) -> bool {
        if !matches!(self.phase, LauncherUiPhase::Aiming) || self.charging || system_popup_active {
            return false;
        }
        outbox.push(LauncherUiEffect::Audio(LauncherUiAudioCue::StartPower));
        outbox.push(LauncherUiEffect::Audio(LauncherUiAudioCue::PowerPulse));
        self.charging = true;
        true
    }

    pub fn advance_power(
        &mut self,
        delta_seconds: f32,
        system_popup_active: bool,
        outbox: &mut LauncherUiOutbox,
    ) {
        if !self.charging
            || !matches!(self.phase, LauncherUiPhase::Aiming)
            || system_popup_active
            || !delta_seconds.is_finite()
        {
            return;
        }
        let Some(trigger) = self.trigger else {
            return;
        };
        let delta = (trigger.max_power - trigger.min_power) * delta_seconds.max(0.0);
        if self.power_rising {
            self.current_power += delta;
            if self.current_power > trigger.max_power {
                self.current_power = trigger.max_power;
                self.power_rising = false;
                outbox.push(LauncherUiEffect::Audio(LauncherUiAudioCue::PowerPulse));
            }
        } else {
            self.current_power -= delta;
            if self.current_power < trigger.min_power {
                self.current_power = trigger.min_power;
                self.power_rising = true;
                outbox.push(LauncherUiEffect::Audio(LauncherUiAudioCue::PowerPulse));
            }
        }
    }

    pub fn release_fire(
        &mut self,
        system_popup_active: bool,
        outbox: &mut LauncherUiOutbox,
    ) -> Option<LauncherShot> {
        if !matches!(self.phase, LauncherUiPhase::Aiming) || !self.charging || system_popup_active {
            return None;
        }
        let facing_yaw = self.start_rotation_degrees.y + self.current_rotation_degrees.y + 180.0;
        let forward = launcher_forward(self.current_rotation_degrees.x, facing_yaw);
        let shot = LauncherShot {
            position: self.camera_position + forward * LAUNCHER_UI_CAMERA_LAUNCH_OFFSET,
            velocity: forward * self.current_power,
            forward,
            facing_yaw_degrees: facing_yaw,
            power: self.current_power,
            request_packet_id: LAUNCHER_REQUEST_PACKET_ID,
            request_packet_size: LAUNCHER_REQUEST_PACKET_SIZE,
        };

        outbox.push(LauncherUiEffect::SetTriggerRenderersVisible(true));
        outbox.push(LauncherUiEffect::SetAvatarRenderersVisible(true));
        outbox.push(LauncherUiEffect::Audio(LauncherUiAudioCue::StopPower));
        outbox.push(LauncherUiEffect::SetCameraCustomControl(false));
        outbox.push(LauncherUiEffect::StartLauncher(shot));
        outbox.push(LauncherUiEffect::SetCameraRotationY(facing_yaw));
        outbox.push(LauncherUiEffect::Audio(LauncherUiAudioCue::Firing));
        self.exit(LauncherUiDismissalSource::Fired, outbox, false);
        Some(shot)
    }

    pub fn request_escape_close(
        &mut self,
        system_popup_active: bool,
        outbox: &mut LauncherUiOutbox,
    ) -> bool {
        if !matches!(self.phase, LauncherUiPhase::Aiming) || system_popup_active {
            return false;
        }
        self.phase = LauncherUiPhase::AwaitingEscapeGate;
        outbox.push(LauncherUiEffect::RequestEscapeCloseGate {
            event_group: 2,
            event_function: 24,
        });
        true
    }

    pub fn resolve_escape_close_gate(
        &mut self,
        accepted: bool,
        outbox: &mut LauncherUiOutbox,
    ) -> bool {
        if !matches!(self.phase, LauncherUiPhase::AwaitingEscapeGate) {
            return false;
        }
        if accepted {
            outbox.push(LauncherUiEffect::SetTriggerRenderersVisible(true));
            outbox.push(LauncherUiEffect::SetCameraCustomControl(false));
            outbox.push(LauncherUiEffect::SetAvatarRenderersVisible(true));
            self.exit(LauncherUiDismissalSource::EscapeGate, outbox, false);
        } else {
            self.phase = LauncherUiPhase::Aiming;
        }
        true
    }

    pub fn cancel_for_death(
        &mut self,
        system_popup_active: bool,
        outbox: &mut LauncherUiOutbox,
    ) -> bool {
        if !self.visible() || system_popup_active {
            return false;
        }
        outbox.push(LauncherUiEffect::SetTriggerRenderersVisible(true));
        outbox.push(LauncherUiEffect::SetCameraCustomControl(false));
        outbox.push(LauncherUiEffect::SetAvatarRenderersVisible(true));
        self.exit(LauncherUiDismissalSource::Death, outbox, true);
        true
    }

    pub fn update(&mut self, input: LauncherUiFrameInput, outbox: &mut LauncherUiOutbox) {
        if !self.visible() {
            return;
        }
        // This occurs before the clean system-popup early return.
        outbox.push(LauncherUiEffect::SetCombatIcon(-1));
        if input.system_popup_active {
            return;
        }
        if input.fire_pressed {
            self.press_fire(false, outbox);
        }
        self.advance_power(input.delta_seconds, false, outbox);
        if input.fire_released {
            self.release_fire(false, outbox);
        }
        if !self.visible() {
            return;
        }
        if input.current_hp <= 0 {
            self.cancel_for_death(false, outbox);
        } else if input.escape_pressed {
            self.request_escape_close(false, outbox);
        }
    }

    pub(super) fn exit(
        &mut self,
        source: LauncherUiDismissalSource,
        outbox: &mut LauncherUiOutbox,
        restore_old_position: bool,
    ) {
        outbox.push(LauncherUiEffect::ExitMode {
            source,
            event_group: 2,
            event_function: 1,
        });
        if restore_old_position {
            outbox.push(LauncherUiEffect::RestoreAvatarPosition(
                self.old_avatar_position,
            ));
        }
        outbox.push(LauncherUiEffect::SetNameVisible(true));
        self.phase = LauncherUiPhase::Hidden;
        self.charging = false;
        self.dismissal = Some(source);
    }
}
