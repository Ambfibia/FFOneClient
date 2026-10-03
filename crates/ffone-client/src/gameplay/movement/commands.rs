use super::*;

/// Builds the exact normal MOVE request fields written by the old client.
#[must_use]
pub fn make_pc_move_request(
    position: Vec3,
    velocity: Vec3,
    yaw_degrees: f32,
    direction_key: u8,
    speed_server_units: i32,
) -> PcMoveRequest0104 {
    PcMoveRequest0104 {
        client_time: 0,
        position: ProtocolPosition::from_native(position).raw(),
        velocity: ProtocolMoveVelocity::from_native(velocity).raw(),
        angle: LegacyUnityHeadingDegrees::new(yaw_degrees)
            .to_protocol()
            .degrees(),
        key_value: direction_key,
        speed: speed_server_units,
    }
}

/// Builds the exact normal STOP request fields written by the old client.
#[must_use]
pub fn make_pc_stop_request(position: Vec3) -> PcStopRequest0104 {
    PcStopRequest0104 {
        client_time: 0,
        position: ProtocolPosition::from_native(position).raw(),
    }
}

/// Builds the exact normal JUMP request fields written by the old client.
#[must_use]
pub fn make_pc_jump_request(
    position: Vec3,
    velocity: Vec3,
    yaw_degrees: f32,
    direction_key: u8,
    jump_height_server_units: i32,
    jump_launch_velocity: f32,
    jump_key: u32,
) -> PcJumpRequest0104 {
    PcJumpRequest0104 {
        client_time: pack_legacy_jump_client_time(jump_launch_velocity, jump_key),
        position: ProtocolPosition::from_native(position).raw(),
        velocity: ProtocolScaledVelocity::from_native(velocity).raw(),
        angle: LegacyUnityHeadingDegrees::new(yaw_degrees)
            .to_protocol()
            .degrees(),
        key_value: direction_key,
        speed: jump_height_server_units,
    }
}

/// A typed request waiting for the networking worker. This module never sends
/// sockets itself; the owner of the OpenFusion session drains this queue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementIntent {
    Move(PcMoveRequest0104),
    Stop(PcStopRequest0104),
    Jump(PcJumpRequest0104),
}

/// Lossless FIFO between Bevy movement simulation and the network worker.
#[derive(Debug, Default, Resource)]
pub struct MovementIntentQueue {
    pub(super) pending: VecDeque<QueuedMovementIntent>,
}

#[derive(Debug)]
pub(super) struct QueuedMovementIntent {
    pub(super) owner: Entity,
    pub(super) intent: MovementIntent,
}

impl MovementIntentQueue {
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<MovementIntent> {
        self.pending.pop_front().map(|queued| queued.intent)
    }

    /// Moves all pending intents out without cloning packets.
    pub fn take_all(&mut self) -> VecDeque<MovementIntent> {
        std::mem::take(&mut self.pending)
            .into_iter()
            .map(|queued| queued.intent)
            .collect()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub(super) fn push(&mut self, owner: Entity, intent: MovementIntent) {
        self.pending
            .push_back(QueuedMovementIntent { owner, intent });
    }

    pub(super) fn reconcile_last_position(&mut self, owner: Entity, position: Vec3) {
        let position = ProtocolPosition::from_native(position).raw();
        let Some(queued) = self
            .pending
            .iter_mut()
            .rev()
            .find(|queued| queued.owner == owner)
        else {
            return;
        };
        match &mut queued.intent {
            MovementIntent::Move(request) => request.position = position,
            MovementIntent::Stop(request) => request.position = position,
            MovementIntent::Jump(request) => request.position = position,
        }
    }
}
