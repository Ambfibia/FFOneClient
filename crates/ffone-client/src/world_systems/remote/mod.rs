//! Native remote-player movement for the OpenFusion 0104 protocol.
//!
//! The compatibility constants and update order are anchored in the decompiled
//! client at `UserMoveController.cs:473-492`, `650-683`, `832-851`, and
//! `1067-1142`. Protocol axis order comes from `CoordUtil.cs:5-9`; the shared
//! runtime contract additionally applies the exporter reflection
//! `H = diag(-1, 1, 1)`.

use std::{collections::BTreeMap, fmt};

use bevy::prelude::*;
use ffone_protocol::{
    DecodedFrame, PcJump0104, PcMove0104, PcStop0104, WirePayload,
    packet::{P_FE2CL_PC_JUMP, P_FE2CL_PC_MOVE, P_FE2CL_PC_STOP},
};

use crate::{
    coordinates::{
        ProtocolMoveVelocity, ProtocolPosition, ProtocolScaledVelocity, ProtocolYawDegrees,
        protocol_distance_to_native,
    },
    tutorial_player_presentation::TutorialPlayerClip,
};

/// The old client calls `LagMoveStop` after this much time without a movement
/// broadcast while a non-zero movement key is active.
pub const LEGACY_MOVE_PACKET_TIMEOUT_SECONDS: f32 = 0.7;

/// `Quaternion.Slerp(..., Time.deltaTime * 8f)` in the old remote controller.
pub const LEGACY_REMOTE_ROTATION_LERP_SPEED: f32 = 8.0;

/// Adds native protocol decoding and interpolation for other player characters.
#[derive(Debug, Default)]
pub struct RemotePlayerPlugin;

impl Plugin for RemotePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RemotePacketQueue>()
            .init_resource::<RemotePacketDiagnostics>()
            .init_resource::<RemotePlayerRegistry>()
            .add_systems(
                Update,
                (consume_remote_packet_queue, interpolate_remote_players).chain(),
            );
    }
}

/// Lossless ingress queue. The network worker can push every `DecodedFrame`
/// here; packets outside this module's MOVE/STOP/JUMP scope are retained in
/// [`RemotePacketDiagnostics`] instead of being discarded.
#[derive(Debug, Default, Resource)]
pub struct RemotePacketQueue {
    frames: Vec<DecodedFrame>,
}

impl RemotePacketQueue {
    pub fn push(&mut self, frame: DecodedFrame) {
        self.frames.push(frame);
    }

    pub fn extend(&mut self, frames: impl IntoIterator<Item = DecodedFrame>) {
        self.frames.extend(frames);
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    fn take(&mut self) -> Vec<DecodedFrame> {
        std::mem::take(&mut self.frames)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedRemoteFrame {
    pub frame: DecodedFrame,
    pub error: RemoteFrameDecodeError,
}

/// Unknown and malformed frames remain available to later protocol slices and
/// diagnostics. This intentionally has no lossy ring-buffer limit.
#[derive(Debug, Default, Resource)]
pub struct RemotePacketDiagnostics {
    pub received: u64,
    pub decoded: u64,
    pub unknown: Vec<DecodedFrame>,
    pub malformed: Vec<MalformedRemoteFrame>,
}

impl RemotePacketDiagnostics {
    pub fn clear_preserved_frames(&mut self) {
        self.unknown.clear();
        self.malformed.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFrameDecodeError {
    pub packet_type: u32,
    pub reason: String,
}

impl RemoteFrameDecodeError {
    fn new(packet_type: u32, reason: impl Into<String>) -> Self {
        Self {
            packet_type,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for RemoteFrameDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "remote packet 0x{:08x}: {}",
            self.packet_type, self.reason
        )
    }
}

impl std::error::Error for RemoteFrameDecodeError {}

/// The three protocol-0104 broadcasts handled by this movement slice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecodedRemotePacket {
    Move(PcMove0104),
    Stop(PcStop0104),
    Jump(PcJump0104),
}

impl DecodedRemotePacket {
    pub const fn pc_id(self) -> i32 {
        match self {
            Self::Move(packet) => packet.pc_id,
            Self::Stop(packet) => packet.pc_id,
            Self::Jump(packet) => packet.pc_id,
        }
    }

    pub const fn server_time(self) -> u64 {
        match self {
            Self::Move(packet) => packet.server_time,
            Self::Stop(packet) => packet.server_time,
            Self::Jump(packet) => packet.server_time,
        }
    }
}

/// Decodes a frame if it belongs to the remote movement slice. `Ok(None)` is
/// an unknown packet, not an error, so its caller can preserve and route it.
pub fn decode_remote_frame(
    frame: &DecodedFrame,
) -> Result<Option<DecodedRemotePacket>, RemoteFrameDecodeError> {
    let packet = match frame.packet_type {
        P_FE2CL_PC_MOVE => {
            let packet = PcMove0104::decode(&frame.payload).map_err(|error| {
                RemoteFrameDecodeError::new(frame.packet_type, error.to_string())
            })?;
            if packet
                .movement
                .velocity
                .iter()
                .any(|value| !value.is_finite())
            {
                return Err(RemoteFrameDecodeError::new(
                    frame.packet_type,
                    "non-finite velocity",
                ));
            }
            DecodedRemotePacket::Move(packet)
        }
        P_FE2CL_PC_STOP => {
            DecodedRemotePacket::Stop(PcStop0104::decode(&frame.payload).map_err(|error| {
                RemoteFrameDecodeError::new(frame.packet_type, error.to_string())
            })?)
        }
        P_FE2CL_PC_JUMP => {
            DecodedRemotePacket::Jump(PcJump0104::decode(&frame.payload).map_err(|error| {
                RemoteFrameDecodeError::new(frame.packet_type, error.to_string())
            })?)
        }
        _ => return Ok(None),
    };

    Ok(Some(packet))
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct RemotePlayer {
    pub pc_id: i32,
}

impl RemotePlayer {
    pub const fn new(pc_id: i32) -> Self {
        Self { pc_id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteAnimationState {
    Idle,
    /// Server broadcast confirms an attack by this remote player.
    Attacking,
    Moving {
        direction_key: u8,
    },
    Jumping {
        direction_key: u8,
        /// The old wire encoding adds 50 per double-jump variant.
        double_jump: Option<u8>,
    },
    /// Server-confirmed `AvatarEmote(int)` clamp clip. Any subsequent
    /// movement/stop/jump packet replaces this state, matching the clean
    /// remote `UserMoveController` interruption path.
    Emoting {
        clip: TutorialPlayerClip,
    },
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct RemoteAnimation {
    pub state: RemoteAnimationState,
}

impl Default for RemoteAnimation {
    fn default() -> Self {
        Self {
            state: RemoteAnimationState::Idle,
        }
    }
}

/// Interpolation state corresponding to `kTargetPosition`, `kTargetAngle`,
/// `kMoveDir`, `fMoveSpeed`, and `fMoveTime` in the original controller.
#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct RemoteMotion {
    pub target_position: Vec3,
    /// Client-axis packet velocity before horizontal normalization.
    pub velocity: Vec3,
    pub speed: f32,
    pub target_rotation: Quat,
    pub seconds_without_movement_packet: f32,
    pub movement_key: u8,
    pub last_server_time: u64,
    lag_stopped: bool,
}

impl Default for RemoteMotion {
    fn default() -> Self {
        Self {
            target_position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            speed: 0.0,
            target_rotation: Quat::IDENTITY,
            seconds_without_movement_packet: 0.0,
            movement_key: 0,
            last_server_time: 0,
            lag_stopped: false,
        }
    }
}

impl RemoteMotion {
    pub fn from_packet(packet: DecodedRemotePacket) -> (Self, RemoteAnimation) {
        let mut motion = Self::default();
        let mut animation = RemoteAnimation::default();
        motion.apply_packet(packet, &mut animation);
        (motion, animation)
    }

    pub fn apply_packet(&mut self, packet: DecodedRemotePacket, animation: &mut RemoteAnimation) {
        self.last_server_time = packet.server_time();

        match packet {
            DecodedRemotePacket::Move(packet) => {
                let movement = packet.movement;
                self.target_position = ProtocolPosition::new(movement.position).to_native();
                self.velocity = ProtocolMoveVelocity::new(movement.velocity).to_native();
                self.speed = protocol_distance_to_native(movement.speed);
                self.seconds_without_movement_packet = 0.0;
                self.movement_key = movement.key_value;
                self.lag_stopped = false;

                // The legacy client keeps the previous yaw for zero velocity.
                if self.velocity.length_squared() > 0.0 {
                    self.target_rotation =
                        ProtocolYawDegrees::new(movement.angle).native_root_rotation();
                }
                animation.state = RemoteAnimationState::Moving {
                    direction_key: movement.key_value,
                };
            }
            DecodedRemotePacket::Stop(packet) => {
                self.target_position = ProtocolPosition::new(packet.movement.position).to_native();
                self.velocity = Vec3::ZERO;
                self.speed = 0.0;
                self.movement_key = 0;
                self.lag_stopped = false;
                animation.state = RemoteAnimationState::Idle;
            }
            DecodedRemotePacket::Jump(packet) => {
                let movement = packet.movement;
                let encoded_key = movement.key_value;
                let double_jump = (encoded_key >= 50).then_some(encoded_key / 50);
                let direction_key = if double_jump.is_some() {
                    encoded_key % 50
                } else {
                    encoded_key
                };

                self.target_position = ProtocolPosition::new(movement.position).to_native();
                self.velocity = ProtocolScaledVelocity::new(movement.velocity).to_native();
                self.speed = protocol_distance_to_native(movement.speed);
                self.target_rotation =
                    ProtocolYawDegrees::new(movement.angle).native_root_rotation();
                self.seconds_without_movement_packet = 0.0;
                self.movement_key = direction_key;
                self.lag_stopped = false;
                animation.state = RemoteAnimationState::Jumping {
                    direction_key,
                    double_jump,
                };
            }
        }
    }

    fn extrapolated_velocity(self) -> Vec3 {
        if !vec3_is_finite(self.velocity) || !self.speed.is_finite() {
            return Vec3::ZERO;
        }

        let horizontal = Vec3::new(self.velocity.x, 0.0, self.velocity.z);
        let horizontal = if horizontal.length_squared() > 0.0 {
            horizontal.normalize() * self.speed
        } else {
            Vec3::ZERO
        };
        Vec3::new(horizontal.x, self.velocity.y, horizontal.z)
    }
}

/// Public registry so model-spawning code can associate an already-created
/// visual root with a `pc_id`. Registered entities must carry `Transform`,
/// [`RemoteMotion`], and [`RemoteAnimation`].
#[derive(Debug, Default, Resource)]
pub struct RemotePlayerRegistry {
    entities: BTreeMap<i32, Entity>,
}

impl RemotePlayerRegistry {
    pub fn get(&self, pc_id: i32) -> Option<Entity> {
        self.entities.get(&pc_id).copied()
    }

    pub fn register(&mut self, pc_id: i32, entity: Entity) -> Option<Entity> {
        self.entities.insert(pc_id, entity)
    }

    pub fn remove(&mut self, pc_id: i32) -> Option<Entity> {
        self.entities.remove(&pc_id)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// Pure movement update used by both Bevy and unit tests. The displacement is
/// applied to both current and target positions before correction, matching the
/// old controller's `controller.Move` followed by `kTargetPosition += delta`.
pub fn step_remote_motion(
    motion: &mut RemoteMotion,
    animation: &mut RemoteAnimation,
    transform: &mut Transform,
    delta_seconds: f32,
) {
    if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
        return;
    }

    // Capture this before the lag transition: the legacy ForceUpdate takes its
    // local kMovement copy before it calls LagMoveStop.
    let extrapolated_velocity = motion.extrapolated_velocity();

    if motion.movement_key != 0 {
        motion.seconds_without_movement_packet += delta_seconds;
    }
    if motion.seconds_without_movement_packet > LEGACY_MOVE_PACKET_TIMEOUT_SECONDS
        && !motion.lag_stopped
    {
        motion.lag_stopped = true;
        motion.movement_key = 0;
        // LagMoveStop clears horizontal movement. Vertical motion remains a
        // physics concern, just as fVelocityZ is separate in the old client.
        motion.velocity.x = 0.0;
        motion.velocity.z = 0.0;
        animation.state = RemoteAnimationState::Idle;
    } else if motion.seconds_without_movement_packet < LEGACY_MOVE_PACKET_TIMEOUT_SECONDS {
        motion.lag_stopped = false;
    }

    let displacement = extrapolated_velocity * delta_seconds;
    if vec3_is_finite(displacement) {
        transform.translation += displacement;
        motion.target_position += displacement;
    }

    if vec3_is_finite(motion.target_position) {
        let position_lerp = (delta_seconds * motion.speed).clamp(0.0, 1.0);
        transform.translation = transform
            .translation
            .lerp(motion.target_position, position_lerp);
    }

    let rotation_lerp = (delta_seconds * LEGACY_REMOTE_ROTATION_LERP_SPEED).clamp(0.0, 1.0);
    if quat_is_finite_and_nonzero(motion.target_rotation) {
        transform.rotation = transform
            .rotation
            .slerp(motion.target_rotation, rotation_lerp);
    }
}

pub fn consume_remote_packet_queue(
    mut commands: Commands,
    mut queue: ResMut<RemotePacketQueue>,
    mut diagnostics: ResMut<RemotePacketDiagnostics>,
    mut registry: ResMut<RemotePlayerRegistry>,
    mut remotes: Query<(&mut RemoteMotion, &mut RemoteAnimation, &mut Transform)>,
) {
    let mut pending_spawns = BTreeMap::<i32, DecodedRemotePacket>::new();

    for frame in queue.take() {
        diagnostics.received = diagnostics.received.saturating_add(1);
        match decode_remote_frame(&frame) {
            Ok(Some(packet)) => {
                diagnostics.decoded = diagnostics.decoded.saturating_add(1);
                let pc_id = packet.pc_id();

                // Deferred spawns are not query-visible until the schedule's
                // command application. Keep only the last arrival for a new PC.
                if let Some(pending) = pending_spawns.get_mut(&pc_id) {
                    *pending = packet;
                    continue;
                }

                let Some(entity) = registry.get(pc_id) else {
                    pending_spawns.insert(pc_id, packet);
                    continue;
                };

                match remotes.get_mut(entity) {
                    Ok((mut motion, mut animation, _)) => {
                        motion.apply_packet(packet, &mut animation);
                    }
                    Err(_) => {
                        registry.remove(pc_id);
                        pending_spawns.insert(pc_id, packet);
                    }
                }
            }
            Ok(None) => diagnostics.unknown.push(frame),
            Err(error) => diagnostics
                .malformed
                .push(MalformedRemoteFrame { frame, error }),
        }
    }

    for (pc_id, packet) in pending_spawns {
        let (motion, animation) = RemoteMotion::from_packet(packet);
        let transform = Transform {
            translation: motion.target_position,
            rotation: motion.target_rotation,
            ..default()
        };
        let entity = commands
            .spawn((RemotePlayer::new(pc_id), motion, animation, transform))
            .id();
        registry.register(pc_id, entity);
    }
}

pub fn interpolate_remote_players(
    time: Res<Time>,
    mut remotes: Query<(&mut RemoteMotion, &mut RemoteAnimation, &mut Transform)>,
) {
    let delta_seconds = time.delta_secs();
    for (mut motion, mut animation, mut transform) in &mut remotes {
        let previous = *animation;
        step_remote_motion(&mut motion, animation.bypass_change_detection(), &mut transform, delta_seconds);
        if *animation != previous {
            animation.set_changed();
        }
    }
}

fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

fn quat_is_finite_and_nonzero(value: Quat) -> bool {
    let length_squared = value.length_squared();
    value.x.is_finite()
        && value.y.is_finite()
        && value.z.is_finite()
        && value.w.is_finite()
        && length_squared > f32::EPSILON
}

#[cfg(test)]
mod tests;
