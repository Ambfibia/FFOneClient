
/// Gravity used by `cnAvatarThirdPersonMove`.
pub const LEGACY_GRAVITY: f32 = 10.0;

/// The exact per-axis bias subtracted from every normal
/// `CharacterController.Move` request by `cnAvatarThirdPersonMove`.
///
/// In particular, the tiny downward component asks Unity for a fresh `Below`
/// contact every frame. It is not a ground-snap distance: once the capsule has
/// left a ledge, the following frame starts source gravity normally.
pub const LEGACY_CHARACTER_MOVE_BIAS: f32 = 0.000_001;

/// A normal unsupported avatar enters `Jump(fVelocityZ)` below this velocity.
pub const LEGACY_FALL_JUMP_VELOCITY: f32 = -LEGACY_GRAVITY / 2.0;

/// An unsupported avatar propelled upward enters `Jump(fVelocityZ)` above this velocity.
pub const LEGACY_RISE_JUMP_VELOCITY: f32 = LEGACY_GRAVITY / 8.0;

/// `GameCondition` cooldown type 3, checked by the normal jump path.
pub const LEGACY_NORMAL_JUMP_COOLDOWN_SECONDS: f32 = 0.5;

/// `cnAvatarThirdPersonMove.slideSpeedModifier` and
/// `UserMoveController.slideSpeedModifier` both initialize to this value.
pub const LEGACY_SURFACE_SLIDE_SPEED: f32 = 2.0;

/// Serialized `CharacterController.slopeLimit = 50` expressed as an up-dot.
pub(super) const LEGACY_SURFACE_SLIDE_MAX_UP_DOT: f32 = 0.642_787_64;

/// Baseline `m_iRunSpeed` from
/// `/tables/0/value/m_pAvatarTable/m_pAvatarData/1`.
pub const LEGACY_BASE_RUN_SPEED_SERVER_UNITS: i32 = 600;

/// Baseline `m_iJumpHeight` from
/// `/tables/0/value/m_pAvatarTable/m_pAvatarData/1`.
pub const LEGACY_BASE_JUMP_HEIGHT_SERVER_UNITS: i32 = 568;

pub(super) const LEGACY_DIRECTION_MATRIX: [[u8; 3]; 3] = [[6, 5, 4], [7, 0, 3], [8, 1, 2]];

pub(super) const LEGACY_JUMP_KEY_MIN: u32 = 1_073_741_823;

pub(super) const LEGACY_JUMP_KEY_MASK: u32 = 0x3fff_ffff;
