//! Native Bevy implementation of the first FusionFall player/camera slice.
//!
//! The normal movement, jump integration and packet field rules in this module
//! come from `cnAvatarThirdPersonMove`, `AvatarUtil`, `CoordUtil`, and
//! `ConfigurableInput` in the clean `retrobution-20260613` primary build.
//! Its `main.unity3d` SHA-256 is
//! `59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F`
//! (7,000,415 bytes);
//! the bundled `Assembly - CSharp.dll` SHA-256 is
//! `33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB`
//! (1,517,568 bytes). It was recovered offline with
//! `ffbuildtool 5.1.0 extract-bundle` and decompiled with
//! `ilspycmd 10.1.1.8388 -p`; neither tool nor legacy input is a runtime
//! dependency.
//! Collision is deliberately not presented as complete: callers can drive
//! `grounded` from the native authored-world collision system or explicitly
//! opt into the flat-plane placeholder in [`LegacyCollisionMode`].

use std::collections::VecDeque;

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
use ffone_protocol::{PcJumpRequest0104, PcMoveRequest0104, PcStopRequest0104};

use crate::coordinates::{
    LegacyUnityHeadingDegrees, ProtocolMoveVelocity, ProtocolPosition, ProtocolScaledVelocity,
    unity_to_native_vector,
};

/// Unity/client units per one integer server coordinate unit.
pub use crate::coordinates::PROTOCOL_TO_NATIVE_SCALE as SERVER_TO_CLIENT_SCALE;

#[cfg(test)]
mod tests;

mod constants;
mod collision;
mod codec;
mod systems;
mod operations;
mod commands;
mod state;
mod types;
mod input_read_legacy_input;
mod containers;
mod animation;

pub use constants::{
    LEGACY_GRAVITY, LEGACY_CHARACTER_MOVE_BIAS, LEGACY_FALL_JUMP_VELOCITY,
    LEGACY_RISE_JUMP_VELOCITY, LEGACY_NORMAL_JUMP_COOLDOWN_SECONDS,
    LEGACY_SURFACE_SLIDE_SPEED, LEGACY_BASE_RUN_SPEED_SERVER_UNITS,
    LEGACY_BASE_JUMP_HEIGHT_SERVER_UNITS
};
use constants::{
    LEGACY_SURFACE_SLIDE_MAX_UP_DOT, LEGACY_DIRECTION_MATRIX, LEGACY_JUMP_KEY_MIN,
    LEGACY_JUMP_KEY_MASK
};
pub(crate) use collision::{
    LEGACY_COLLISION_SIDES, LEGACY_COLLISION_ABOVE, LEGACY_COLLISION_BELOW
};
pub use collision::{LegacyWorldColliderPending, LegacyCollisionMode};
pub use codec::{
    LEGACY_MAX_FRAME_DELTA, LEGACY_PACKET_SEND_INTERVAL, LEGACY_PACKET_MIN_INTERVAL,
    LEGACY_PACKET_ANGLE_THRESHOLD
};
use codec::packet_is_due;
pub use systems::{advance_native_xorshift32, update_legacy_camera_input};
use systems::should_apply_camera_yaw;
pub use operations::{
    legacy_direction_key, pack_legacy_jump_client_time, simulate_legacy_players
};
use operations::{
    movement_axes_from_keys, free_camera_from_keys, camera_turn_keys, arrow_camera_recenter,
    clamp_legacy_angle, legacy_mouse_camera_axes, legacy_camera_rotation_to_native
};
#[cfg(test)]
use operations::legacy_camera_forward_to_native;
pub use commands::{
    make_pc_move_request, make_pc_stop_request, make_pc_jump_request, MovementIntent,
    MovementIntentQueue
};
pub use state::{LegacyCameraKeyInput, LegacyInputState};
pub use types::{
    LegacyInputGate, LegacyPlayerController, LegacyOrbitCamera, LegacyMovementSet,
    LegacyMovementPlugin
};
pub use input_read_legacy_input::read_legacy_input;
use containers::bevy_mouse_delta_to_unity;
pub use animation::update_legacy_camera_pose;
