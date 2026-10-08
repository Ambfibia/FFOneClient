use super::*;

/// Runtime permissions applied before legacy movement and camera input is
/// exposed to gameplay systems.
///
/// The normal client leaves every permission enabled. Tutorial stages can
/// replace individual fields without changing the legacy key mappings or
/// inserting another input system between the ordered movement sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct LegacyInputGate {
    pub allow_forward: bool,
    pub allow_backward: bool,
    pub allow_strafe: bool,
    pub allow_keyboard_turning: bool,
    pub allow_jump: bool,
    pub allow_mouse_camera: bool,
}

impl Default for LegacyInputGate {
    fn default() -> Self {
        Self {
            allow_forward: true,
            allow_backward: true,
            allow_strafe: true,
            allow_keyboard_turning: true,
            allow_jump: true,
            allow_mouse_camera: true,
        }
    }
}

/// Per-player state and server-provided movement attributes.
#[derive(Debug, Clone, Component)]
pub struct LegacyPlayerController {
    /// Raw `iRunSpeed` from the server/table data.
    pub run_speed_server_units: i32,
    /// Raw `iJumpHeight` from the server/table data.
    pub jump_height_server_units: i32,
    /// Legacy Unity heading of visual/model +Z. Never feed this value directly
    /// to a Bevy transform; use `LegacyUnityHeadingDegrees` so the global X
    /// reflection and Bevy -Z root-forward convention are both applied.
    pub yaw_degrees: f32,
    pub velocity: Vec3,
    pub grounded: bool,
    pub jumping: bool,
    pub movement_enabled: bool,
    /// Authoritative stun/sleep gate; gravity and collision remain live.
    pub incapacitated: bool,
    pub(super) nano_dash_remaining: f32,
    pub(super) nano_dash_speed: f32,
    pub(super) nano_dash_airborne: bool,
    pub collision: LegacyCollisionMode,
    pub(super) flight_enabled: bool,
    pub(super) vehicle_speed: Option<f32>,
    pub(super) vehicle_momentum: Vec3,
    pub(super) vehicle_stop_pending: bool,
    pub(super) auto_run: bool,
    pub(super) scripted_horizontal_velocity: Option<Vec2>,
    pub(super) vertical_velocity: f32,
    pub(super) last_move_displacement: Vec3,
    pub(super) jump_packet_velocity: f32,
    pub(super) normal_jump_cooldown_remaining: f32,
    pub(super) previous_collision_flags: u8,
    pub(super) contact_point_normal: Vec3,
    pub(super) non_priority_collision: bool,
    pub(super) surface_sliding: bool,
    pub(super) terminal_fall_previous_y: f32,
    pub(super) jump_key: u32,
    pub(super) jump_rng_state: u32,
    pub(super) packet_elapsed: f32,
    pub(super) current_direction_key: u8,
    pub(super) last_direction_key: u8,
    pub(super) last_packet_yaw: f32,
    pub(super) last_packet_position: Vec3,
    pub(super) packet_position_sampled_this_frame: bool,
    pub(super) movement_intent_emitted_this_frame: bool,
    pub(super) external_transport_position_pending: bool,
}

impl LegacyPlayerController {
    /// Personal transport starts only after the authoritative mount reply.
    pub fn set_vehicle_speed(&mut self, speed_server_units: Option<i32>) {
        let speed = speed_server_units.map(|speed| speed.max(0) as f32 * SERVER_TO_CLIENT_SCALE);
        if self.vehicle_speed != speed {
            self.vehicle_momentum = Vec3::ZERO;
            self.vehicle_stop_pending = false;
            self.packet_elapsed = f32::INFINITY;
            self.vehicle_speed = speed;
        }
    }

    pub(super) fn step_vehicle_horizontal(&mut self, axis: Vec2, yaw: f32, dt: f32) -> Vec3 {
        let Some(speed) = self.vehicle_speed else { return Vec3::ZERO; };
        if axis == Vec2::ZERO {
            if self.vehicle_momentum.length() > 0.1 {
                self.vehicle_momentum *= if self.grounded { 0.98 } else { 0.997 };
            } else {
                self.vehicle_stop_pending |= self.vehicle_momentum != Vec3::ZERO;
                self.vehicle_momentum = Vec3::ZERO;
            }
            return self.vehicle_momentum;
        }
        let ground = self.grounded;
        let (acceleration, multiplier) = if axis.y < 0.0 {
            (if ground { 0.75 } else { 0.375 }, 0.54)
        } else if axis.x.abs() > 0.9 && axis.y > 0.9 {
            (if ground { 0.8 } else { 0.45 }, 1.215)
        } else if axis.x.abs() > 0.9 && axis.y < 0.1 {
            (if self.vehicle_momentum.length() > 10.0 {
                if ground { 2.5 } else { 1.25 }
            } else if ground { 0.9 } else { 0.45 }, 0.9)
        } else {
            (if ground { 0.85 } else { 0.5 }, 1.35)
        };
        let direction = LegacyUnityHeadingDegrees::new(yaw).native_root_rotation()
            * Vec3::new(axis.x, 0.0, -axis.y).normalize_or_zero();
        let target = direction * speed * multiplier;
        self.vehicle_momentum = self.vehicle_momentum.lerp(target, (acceleration * dt).clamp(0.0, 1.0));
        let aligned = direction.dot(self.vehicle_momentum.normalize_or_zero()) >= 0.8;
        self.vehicle_momentum = self.vehicle_momentum.lerp(target, if aligned { (3.0 * dt).min(1.0) } else { 0.0001 });
        self.vehicle_momentum = self.vehicle_momentum.clamp_length_max(speed * multiplier);
        self.vehicle_momentum
    }

    /// Creates the unmodified avatar attributes selected by
    /// `cnAvatarStatus.InitAvatarAttrib`. Equipment, Nano and buff overrides
    /// must update the raw fields after spawning.
    #[must_use]
    pub fn from_baseline_table() -> Self {
        Self::from_server_attributes(
            LEGACY_BASE_RUN_SPEED_SERVER_UNITS,
            LEGACY_BASE_JUMP_HEIGHT_SERVER_UNITS,
        )
    }

    #[must_use]
    pub fn from_server_attributes(run_speed: i32, jump_height: i32) -> Self {
        Self {
            run_speed_server_units: run_speed,
            jump_height_server_units: jump_height,
            yaw_degrees: 0.0,
            velocity: Vec3::ZERO,
            grounded: true,
            jumping: false,
            movement_enabled: true,
            incapacitated: false,
            nano_dash_remaining: 0.0,
            nano_dash_speed: 0.0,
            nano_dash_airborne: false,
            collision: LegacyCollisionMode::External,
            flight_enabled: false,
            vehicle_speed: None,
            vehicle_momentum: Vec3::ZERO,
            vehicle_stop_pending: false,
            auto_run: false,
            scripted_horizontal_velocity: None,
            vertical_velocity: 0.0,
            last_move_displacement: Vec3::ZERO,
            jump_packet_velocity: 0.0,
            normal_jump_cooldown_remaining: 0.0,
            previous_collision_flags: 0,
            contact_point_normal: Vec3::ZERO,
            non_priority_collision: false,
            surface_sliding: false,
            terminal_fall_previous_y: 0.0,
            jump_key: LEGACY_JUMP_KEY_MIN,
            jump_rng_state: 0x6d2b_79f5,
            packet_elapsed: 0.0,
            current_direction_key: 0,
            last_direction_key: 0,
            last_packet_yaw: 0.0,
            // This intentionally matches the zero-initialized C# field rather
            // than silently treating the spawn position as already sent.
            last_packet_position: Vec3::ZERO,
            packet_position_sampled_this_frame: false,
            movement_intent_emitted_this_frame: false,
            external_transport_position_pending: false,
        }
    }

    #[must_use]
    pub fn with_yaw(mut self, yaw_degrees: f32) -> Self {
        self.yaw_degrees = yaw_degrees;
        self
    }

    #[must_use]
    pub fn with_placeholder_ground_plane(mut self, height: f32) -> Self {
        self.collision = LegacyCollisionMode::PlaceholderGroundPlane { height };
        self
    }

    /// Hook for the native collision system that will replace the placeholder.
    pub fn set_grounded(&mut self, grounded: bool) {
        self.grounded = grounded;
        if grounded && !self.jumping {
            self.vertical_velocity = 0.0;
            self.velocity.y = 0.0;
        }
    }

    /// Exact displacement most recently submitted by the normal legacy
    /// movement pass. The authored collision owner uses this instead of
    /// reconstructing a sweep from velocity after contacts may have clipped
    /// that velocity.
    #[must_use]
    pub(crate) fn last_move_displacement(&self) -> Vec3 {
        self.last_move_displacement
    }

    /// Stores the result of the authored-world equivalent of the most recent
    /// `CharacterController.Move`. `EnvironmentCollision.HandleSurfaceSliding`
    /// reads these values on the following frame, before issuing the next
    /// Move, so the contact must not be consumed immediately by collision.
    pub(crate) fn set_external_collision_result(
        &mut self,
        collision_flags: u8,
        contact_normal: Option<Vec3>,
    ) {
        self.previous_collision_flags = collision_flags;
        if let Some(normal) = contact_normal.filter(|normal| normal.is_finite()) {
            self.contact_point_normal = normal.normalize_or_zero();
            // Static native terrain and ordinary untagged MeshColliders both
            // take ResetCollisionVariables' non-priority branch in the clean
            // client. Tagged moving-platform ownership is handled separately
            // by NativeWorldGroundSupport and never calls this method as a
            // substitute for that platform state.
            self.non_priority_collision = true;
        }
    }

    #[must_use]
    pub(crate) fn surface_sliding(&self) -> bool {
        self.surface_sliding
    }

    /// Exact late `ForceUpdate` escape from a blocked terminal fall.
    ///
    /// Retrobution samples `fOldY` only while `bJumpFlag` is active at the
    /// capped `-fGravity` velocity. If two consecutive post-Move positions
    /// have exactly the same Y, the CCT has wedged or come to rest even though
    /// `EnvironmentCollision.sliding` suppressed its ordinary Below landing;
    /// the source then clears both the jump and sliding states. Small branch
    /// concavities rely on this transition before Space can launch again.
    pub(crate) fn settle_terminal_blocked_fall(&mut self, position_y: f32) -> bool {
        if !position_y.is_finite() || !self.jumping || self.vertical_velocity > -LEGACY_GRAVITY {
            return false;
        }
        let stationary = self.terminal_fall_previous_y == position_y;
        self.terminal_fall_previous_y = position_y;
        if stationary {
            self.land_on_external_collider();
            self.surface_sliding = false;
        }
        stationary
    }

    /// Walkable contact retained from the preceding CharacterController move.
    /// The native collision owner uses this to reproduce the CCT's internal
    /// slope constraint without changing the source-authored packet velocity.
    #[must_use]
    pub(crate) fn walkable_support_normal(&self) -> Option<Vec3> {
        if !self.grounded
            || self.surface_sliding
            || self.vertical_velocity > 0.0
            || self.previous_collision_flags & LEGACY_COLLISION_BELOW == 0
        {
            return None;
        }
        let normal = self.contact_point_normal.normalize_or_zero();
        (normal.y >= LEGACY_SURFACE_SLIDE_MAX_UP_DOT).then_some(normal)
    }

    /// Retain platform carry until the next packet interval, including when
    /// the platform stops between intervals and the rider has no input.
    pub(crate) fn record_external_transport_motion(&mut self) {
        self.external_transport_position_pending = true;
    }

    /// Completes the source `CharacterController.Move` -> `MovePacket`
    /// ordering when authored collision is owned by the later world system.
    /// The movement pass constructs packet fields first, but no packet may
    /// leave with the unconstrained pre-sweep position.
    pub(crate) fn reconcile_external_collision_position(
        &mut self,
        owner: Entity,
        position: Vec3,
        intents: Option<&mut MovementIntentQueue>,
    ) {
        if !self.packet_position_sampled_this_frame {
            return;
        }
        self.last_packet_position = position;
        if let Some(intents) = intents {
            if self.movement_intent_emitted_this_frame {
                intents.reconcile_last_position(owner, position);
            } else if self.external_transport_position_pending {
                // Idle riders still change server chunks. Without a position
                // packet the server eventually removes their moving support
                // from interest, destroying both its visual and collider.
                intents.push(owner, MovementIntent::Stop(make_pc_stop_request(position)));
                self.movement_intent_emitted_this_frame = true;
            }
            self.external_transport_position_pending = false;
        }
    }

    /// Completes the legacy landing transition after an external authored
    /// collider has resolved the player's feet onto a surface.
    ///
    /// Collision runs after [`LegacyMovementSet::Simulate`], matching the old
    /// `Below` flag ordering: the current frame's movement packet has already
    /// been constructed, then landing clears the jump state. When the old
    /// `bJumpFlag` was active, it also makes the next movement update
    /// immediately packet-eligible.
    pub fn land_on_external_collider(&mut self) {
        let was_jumping = self.jumping;
        self.grounded = true;
        self.jumping = false;
        self.scripted_horizontal_velocity = None;
        self.vertical_velocity = 0.0;
        self.velocity.y = 0.0;
        // `ForceUpdate` resets fLastSendTime only when bJumpFlag was active.
        // A short step/fall that lands before the -gravity/2 transition does
        // not force an otherwise unnecessary movement packet.
        if was_jumping {
            self.packet_elapsed = f32::INFINITY;
        }
    }

    /// Resets locally integrated movement after an authoritative server
    /// discontinuity such as `P_FE2CL_REP_PC_REGEN_SUCC`.
    ///
    /// The server already owns the destination, so the next normal movement
    /// frame starts from that position instead of replaying pre-warp velocity,
    /// auto-run or a stale jump.
    pub fn apply_authoritative_teleport(&mut self, position: Vec3) {
        self.nano_dash_remaining = 0.0;
        self.velocity = Vec3::ZERO;
        self.vehicle_momentum = Vec3::ZERO;
        self.grounded = false;
        self.jumping = false;
        self.auto_run = false;
        self.scripted_horizontal_velocity = None;
        self.vertical_velocity = 0.0;
        self.last_move_displacement = Vec3::ZERO;
        self.jump_packet_velocity = 0.0;
        self.previous_collision_flags = 0;
        self.contact_point_normal = Vec3::ZERO;
        self.non_priority_collision = false;
        self.surface_sliding = false;
        self.terminal_fall_previous_y = 0.0;
        self.packet_elapsed = 0.0;
        self.current_direction_key = 0;
        self.last_direction_key = 0;
        self.last_packet_position = position;
        self.packet_position_sampled_this_frame = false;
        self.movement_intent_emitted_this_frame = false;
        self.external_transport_position_pending = false;
    }

    /// Native equivalent of `cnAvatarThirdPersonMove.SetAutoRun`.
    pub fn set_auto_run(&mut self, enabled: bool) {
        self.auto_run = enabled;
    }

    #[must_use]
    pub fn is_auto_running(&self) -> bool {
        self.auto_run
    }

    /// Enables or disables the native free-flight extension used by `/fly`.
    /// Disabling it in mid-air hands vertical movement back to normal gravity.
    pub fn set_flight_enabled(&mut self, enabled: bool) {
        if self.flight_enabled == enabled {
            return;
        }
        self.flight_enabled = enabled;
        self.auto_run = false;
        self.scripted_horizontal_velocity = None;
        self.grounded = false;
        self.jumping = false;
        self.vertical_velocity = 0.0;
        self.last_move_displacement = Vec3::ZERO;
        self.jump_packet_velocity = 0.0;
        self.previous_collision_flags = 0;
        self.contact_point_normal = Vec3::ZERO;
        self.non_priority_collision = false;
        self.surface_sliding = false;
        self.terminal_fall_previous_y = 0.0;
        self.velocity.y = 0.0;
        self.packet_elapsed = f32::INFINITY;
        self.packet_position_sampled_this_frame = false;
        self.movement_intent_emitted_this_frame = false;
    }

    #[must_use]
    pub fn flight_enabled(&self) -> bool {
        self.flight_enabled
    }

    /// Camera-directed Nano Dash with the native speed and deceleration.
    pub fn nano_dash_active(&self) -> bool { self.nano_dash_remaining > 0.0 }

    pub fn nano_dash_started_airborne(&self) -> bool { self.nano_dash_airborne }

    pub fn launch_nano_dash(&mut self) {
        self.launch_nano_dash_from(!self.grounded);
    }

    pub fn launch_nano_dash_from(&mut self, airborne: bool) {
        self.nano_dash_airborne = airborne;
        self.nano_dash_remaining = 0.5555555;
        self.nano_dash_speed = 35.0;
    }

    /// Exact `AvatarUtil.GetDirNum` value resolved for the current frame.
    /// `0` is idle and `4..=6` selects the legacy `runback` motion.
    #[must_use]
    pub fn current_direction_key(&self) -> u8 {
        self.current_direction_key
    }

    /// Updates the exact legacy eight-way direction used by animation and
    /// scripted/native movement bridges. Device input normally owns this
    /// value; callers outside that path must still obey the 0..=8 protocol
    /// table.
    pub fn set_current_direction_key(&mut self, direction_key: u8) -> bool {
        if direction_key > 8 {
            return false;
        }
        self.current_direction_key = direction_key;
        true
    }

    /// Exact scripted `iCurMoveDir = 0` operation used by
    /// `AvatarStandForce`. This intentionally leaves the last packet direction
    /// untouched; the next movement simulation owns that transition.
    pub fn reset_current_move_direction(&mut self) {
        self.current_direction_key = 0;
    }

    /// Applies the clean scripted vertical launch used by `EpJumppadTrigger`.
    /// Horizontal input remains live while the authored upward power replaces
    /// the normal jump-height impulse.
    pub fn launch_from_jumppad(&mut self, power: f32) -> bool {
        if !power.is_finite() || power <= 0.0 {
            return false;
        }
        self.grounded = false;
        self.last_move_displacement = Vec3::ZERO;
        self.velocity.y = power;
        self.start_normal_jump(power);
        self.packet_elapsed = 0.0;
        true
    }

    /// A Nano rocket can replace an in-flight jump. Keep the current ground
    /// result for the first Move, just like Jump(force); collision and normal
    /// movement packets own the subsequent ascent and landing.
    pub fn launch_nano_rocket(&mut self, power: f32) -> bool {
        if !power.is_finite() || power <= 0.0 {
            return false;
        }
        self.scripted_horizontal_velocity = None;
        self.start_normal_jump(power);
        self.velocity.y = power;
        self.packet_elapsed = f32::INFINITY;
        true
    }

    /// Applies the source launcher shot while leaving gravity and authored
    /// collision in the ordinary movement/collision pipeline.
    pub fn launch_scripted_ballistic(&mut self, velocity: Vec3) -> bool {
        if !velocity.is_finite() || velocity.length_squared() <= f32::EPSILON {
            return false;
        }
        self.movement_enabled = true;
        self.grounded = false;
        self.last_move_displacement = Vec3::ZERO;
        self.scripted_horizontal_velocity = Some(Vec2::new(velocity.x, velocity.z));
        self.velocity = velocity;
        self.start_normal_jump(velocity.y);
        self.packet_elapsed = 0.0;
        self.previous_collision_flags = 0;
        true
    }

    /// Cannon flight owns heading, movement packets and full-body presentation.
    pub fn launcher_active(&self) -> bool {
        self.scripted_horizontal_velocity.is_some()
    }

    pub(crate) fn launcher_hit_surface(&self) -> bool {
        self.previous_collision_flags & 0b101 != 0
    }

    /// Submit EpUpdate's CharacterController.Move before the world's collision
    /// pass. Zipline applies its hanging offset only after that Move resolves.
    pub(crate) fn submit_scripted_move(&mut self, displacement: Vec3, delta_seconds: f32) {
        self.last_move_displacement = displacement;
        self.velocity = if delta_seconds > 0.0 { displacement / delta_seconds } else { Vec3::ZERO };
    }

    /// EpUpdate exits the cable/cannon with Jump(0), including the jump flag
    /// and reset of surface-sliding state. Space cannot launch a second jump
    /// during this fall before the next Below contact.
    pub(crate) fn finish_scripted_traversal_with_jump(&mut self) {
        self.finish_scripted_traversal(Vec3::ZERO);
        self.start_normal_jump(0.0);
        self.surface_sliding = false;
        self.non_priority_collision = true;
        self.previous_collision_flags = 0;
    }

    /// Suspends ordinary input integration while an authored rope/zipline
    /// driver owns the avatar transform.
    pub fn begin_scripted_traversal(&mut self) {
        self.movement_enabled = false;
        self.grounded = false;
        self.jumping = false;
        self.vertical_velocity = 0.0;
        self.velocity = Vec3::ZERO;
        self.last_move_displacement = Vec3::ZERO;
        self.scripted_horizontal_velocity = None;
        self.current_direction_key = 0;
    }

    /// Returns transform ownership to normal movement at the end of a
    /// scripted traversal, preserving the exit velocity for the first fall.
    pub fn finish_scripted_traversal(&mut self, exit_velocity: Vec3) {
        self.movement_enabled = true;
        self.grounded = false;
        self.jumping = false;
        self.velocity = exit_velocity;
        self.scripted_horizontal_velocity = None;
        self.vertical_velocity = exit_velocity.y;
        self.last_move_displacement = Vec3::ZERO;
        self.packet_elapsed = f32::INFINITY;
    }

    pub(super) fn resolve_local_axis(&mut self, manual_axis: Vec2) -> Vec2 {
        if manual_axis != Vec2::ZERO {
            // Any manual movement cancels an already active auto-run before
            // the Home key's end-of-frame toggle is applied.
            self.auto_run = false;
            manual_axis
        } else if self.auto_run {
            Vec2::Y
        } else {
            Vec2::ZERO
        }
    }

    pub(super) fn finish_input_frame(&mut self, auto_run_just_pressed: bool) {
        if auto_run_just_pressed {
            self.auto_run = !self.auto_run;
        }
    }

    pub(super) fn advance_jump_key(&mut self) {
        // Unity's integer RandomRange is only used as an opaque echoed value by
        // OpenFusion. Xorshift keeps the native client dependency-free while
        // preserving the exact legacy numeric range [1073741823, i32::MAX).
        let value = advance_native_xorshift32(&mut self.jump_rng_state);
        self.jump_key = LEGACY_JUMP_KEY_MIN + (value & LEGACY_JUMP_KEY_MASK);
    }

    /// Native equivalent of normal `cnAvatarThirdPersonMove.Jump`.
    ///
    /// The legacy method accepts negative velocity when a fall becomes a
    /// networked jump state, but refreshes `jumpVelocity` only for positive
    /// launches. That distinction is observable in the upper half of
    /// `sP_CL2FE_REQ_PC_JUMP.iCliTime`.
    pub(super) fn start_normal_jump(&mut self, force: f32) {
        self.vertical_velocity = force;
        if force > 0.0 {
            self.jump_packet_velocity = force;
        }
        self.jumping = true;
    }

    pub(super) fn advance_normal_jump_cooldown(&mut self, delta_seconds: f32) {
        self.normal_jump_cooldown_remaining =
            (self.normal_jump_cooldown_remaining - delta_seconds).max(0.0);
    }

    /// Exact ordinary-surface branch of
    /// `EnvironmentCollision.HandleSurfaceSliding`.
    ///
    /// This deliberately changes only the submitted `kMovement`. The source
    /// keeps `fVelocityZ` unchanged, so gravity continues from the undoubled
    /// vertical velocity on the next frame while the current Move and packet
    /// observe `kMovement.y * 2`.
    pub(super) fn apply_surface_sliding(&mut self, movement: Vec3) -> Vec3 {
        if !self.non_priority_collision || self.previous_collision_flags == 0 {
            return movement;
        }
        let normal = self.contact_point_normal.normalize_or_zero();
        if normal.dot(Vec3::Y) < LEGACY_SURFACE_SLIDE_MAX_UP_DOT {
            let horizontal_normal = Vec3::new(normal.x, 0.0, normal.z).normalize_or_zero();
            self.surface_sliding = true;
            return Vec3::new(
                horizontal_normal.x * LEGACY_SURFACE_SLIDE_SPEED,
                movement.y * 2.0,
                horizontal_normal.z * LEGACY_SURFACE_SLIDE_SPEED,
            );
        }
        self.surface_sliding = false;
        movement
    }

    /// Reproduces the normal vertical portion of
    /// `cnAvatarThirdPersonMove.ForceUpdate` before `CharacterController.Move`.
    pub(super) fn step_normal_vertical(&mut self, jump_requested: bool, delta_seconds: f32) {
        self.advance_normal_jump_cooldown(delta_seconds);
        // Retrobution gates a normal jump on !bJumpFlag, not bGround. This
        // preserves its short edge/coyote window after walking off a surface.
        // GameCondition cooldown type 3 separately rejects a new launch for
        // 0.5 seconds; the original has no queued/pre-landing jump buffer.
        if jump_requested && !self.jumping && self.normal_jump_cooldown_remaining <= 0.0 {
            let force = self.jump_height_server_units as f32 * SERVER_TO_CLIENT_SCALE;
            self.normal_jump_cooldown_remaining = LEGACY_NORMAL_JUMP_COOLDOWN_SECONDS;
            self.start_normal_jump(force);
        }

        // Jump() deliberately does not clear bGround. A launch from a surface
        // therefore receives its full authored velocity for one Move; the
        // post-Move Below result clears ground for the following frame.
        if !self.grounded {
            self.vertical_velocity -= LEGACY_GRAVITY * delta_seconds;
        }
        self.vertical_velocity = self.vertical_velocity.max(-LEGACY_GRAVITY);

        // Walking off an edge is initially an ordinary unsupported Move. The
        // old client changes it to the jump animation/packet state only after
        // crossing these strict velocity thresholds.
        if !self.jumping
            && !self.grounded
            && (self.vertical_velocity < LEGACY_FALL_JUMP_VELOCITY
                || self.vertical_velocity > LEGACY_RISE_JUMP_VELOCITY)
        {
            self.start_normal_jump(self.vertical_velocity);
        }
    }
}

/// Orbit state copied from the serialized Retrobution `MainCamera`.
///
/// `cnPlayerCamera` has different C# field initializers, but Unity applies the
/// values serialized on `mainData` object 45 after construction. Gameplay must
/// use those scene overrides rather than the constructor fallbacks.
#[derive(Debug, Clone, Component)]
pub struct LegacyOrbitCamera {
    pub target: Entity,
    /// Fixed SubTargetCamera path: no orbit smoothing or inward correction.
    pub sub_target_forward: Option<Vec3>,
    pub height: f32,
    pub minimum_distance: f32,
    pub maximum_distance: f32,
    pub distance: f32,
    pub default_distance: f32,
    pub pitch_degrees: f32,
    pub default_pitch_degrees: f32,
    pub minimum_pitch_degrees: f32,
    pub maximum_pitch_degrees: f32,
    /// Legacy Unity camera heading around +Y. The pose system reflects its
    /// resulting forward vector through the shared native coordinate contract.
    pub yaw_degrees: f32,
    pub mouse_sensitivity: f32,
    pub configurable_sensitivity: f32,
    /// Exact serialized `mainData` InputManager sensitivity for both the
    /// legacy `Mouse X` and `Mouse Y` movement axes.
    pub mouse_axis_per_pixel: f32,
    /// Exact serialized `mainData` InputManager sensitivity for one normalized
    /// wheel line on the legacy `Mouse ScrollWheel` axis.
    pub scroll_axis_per_line: f32,
    pub(super) force_player_angle_this_frame: bool,
}

impl LegacyOrbitCamera {
    #[must_use]
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            sub_target_forward: None,
            height: 1.4,
            minimum_distance: 4.0,
            maximum_distance: 12.0,
            distance: 5.0,
            default_distance: 7.0,
            pitch_degrees: 0.0,
            default_pitch_degrees: 11.5,
            minimum_pitch_degrees: -30.0,
            maximum_pitch_degrees: 70.0,
            yaw_degrees: 0.0,
            mouse_sensitivity: 6.0,
            configurable_sensitivity: 5.0,
            // primary main.unity3d -> mainData InputManager object 2.
            mouse_axis_per_pixel: 0.03,
            scroll_axis_per_line: 0.1,
            force_player_angle_this_frame: false,
        }
    }

    #[must_use]
    pub fn with_yaw(mut self, yaw_degrees: f32) -> Self {
        self.yaw_degrees = yaw_degrees;
        self
    }
}

/// Public ordering points so gameplay/UI systems can run before or after the
/// compatible input, movement and camera stages without duplicating them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum LegacyMovementSet {
    ReadInput,
    CameraInput,
    Simulate,
    CameraPose,
}

/// Installs native input, player simulation, typed packet production and orbit
/// camera behavior. It does not spawn entities and does not open a socket.
pub struct LegacyMovementPlugin;

impl Plugin for LegacyMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LegacyInputState>()
            .init_resource::<LegacyInputGate>()
            .init_resource::<MovementIntentQueue>()
            .configure_sets(
                Update,
                (
                    LegacyMovementSet::ReadInput,
                    LegacyMovementSet::CameraInput,
                    LegacyMovementSet::Simulate,
                    LegacyMovementSet::CameraPose,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                read_legacy_input.in_set(LegacyMovementSet::ReadInput),
            )
            .add_systems(
                Update,
                update_legacy_camera_input.in_set(LegacyMovementSet::CameraInput),
            )
            .add_systems(
                Update,
                simulate_legacy_players.in_set(LegacyMovementSet::Simulate),
            )
            .add_systems(
                Update,
                update_legacy_camera_pose.in_set(LegacyMovementSet::CameraPose),
            );
    }
}
