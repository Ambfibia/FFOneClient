use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub first_line: u32,
    pub last_line: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ClientVec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const UP: Self = Self::new(0.0, 1.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Raw argument order accepted by `AddNpc(x, y, z, ...)` and
/// `WarpNpc(x, y, z, ...)`: server X, server Y (client horizontal Z), server Z
/// (client vertical Y).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyServerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl LegacyServerPosition {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn client(self) -> ClientVec3 {
        ClientVec3::new(self.x, self.z, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityRef {
    Player,
    Camera,
    StartPosition,
    Npc(i32),
    NewNano,
}

/// A camera transform explicitly captured into a coroutine local before the
/// authored camera controller mutates its live transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraCaptureSlot {
    /// `position2 = PlayerCamera.transform.position` in `Infection_Event_B`.
    InfectionReturnStart,
    /// `rotation = PlayerCamera.transform.rotation` in `Infection_Event_C`.
    InfectionEntry,
}

/// Non-recursive origin for an authored position expression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionAnchor {
    Client(ClientVec3),
    Entity(EntityRef),
    Midpoint { left: EntityRef, right: EntityRef },
    CapturedCamera(CameraCaptureSlot),
}

/// Rotation which turns a local Unity-space vector into a world-space offset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrientationBasis {
    /// Exact `Quaternion.Euler(x, y, z)` authored by the coroutine.
    Euler(ClientVec3),
    /// The live `Transform.rotation` of the referenced object.
    Entity(EntityRef),
    /// The yaw-only quaternion reconstructed from a captured camera rotation.
    CapturedCameraYaw(CameraCaptureSlot),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionExpr {
    Client(ClientVec3),
    Entity(EntityRef),
    EntityOffset {
        entity: EntityRef,
        offset: ClientVec3,
    },
    Midpoint {
        left: EntityRef,
        right: EntityRef,
        offset: ClientVec3,
    },
    /// `origin + world_offset + orientation * local_offset`.
    ///
    /// `world_offset` and `local_offset` retain the exact legacy Unity axes.
    /// An ECS adapter resolves the referenced transforms at the action time.
    OrientedOffset {
        origin: PositionAnchor,
        world_offset: ClientVec3,
        orientation: OrientationBasis,
        local_offset: ClientVec3,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RotationExpr {
    Euler(ClientVec3),
    FaceEntity(EntityRef),
    FacePosition(PositionExpr),
    EntityYaw {
        entity: EntityRef,
        pitch: f32,
        yaw_offset: f32,
        roll: f32,
    },
    /// Preserve only the yaw produced by `Transform.LookAt`, then apply the
    /// explicitly authored pitch/yaw-offset/roll.
    LookAtYaw {
        target: ClientVec3,
        pitch: f32,
        yaw_offset: f32,
        roll: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeChannel {
    Overlay,
    CinematicBars,
    Subtitle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeSequence {
    pub channel: FadeChannel,
    pub from: f32,
    pub to: f32,
    pub steps: u16,
    pub divisor: f32,
    pub seconds_per_step: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NpcSpawn {
    pub id: i32,
    pub npc_type: i32,
    pub position: LegacyServerPosition,
    /// Argument passed to `AngleNpc`; the helper itself adds 180 degrees.
    pub helper_angle_degrees: Option<i16>,
    pub initial_animation: Option<&'static str>,
    pub force_update: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectSpawn {
    pub effect_id: i32,
    pub position: PositionExpr,
    pub scale: f32,
    /// `true` for `AddEffect`, whose object is owned by `ClearEffect`.
    pub tracked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraShakeWave {
    pub x_sin_multiplier: f32,
    pub y_cos_multiplier: f32,
    pub z_sin_multiplier: f32,
    pub z_cos_multiplier: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialCursorDirection {
    Right,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialScreenAxis {
    Pixels(i32),
    WidthMinus(i32),
    HeightMinus(i32),
}

impl TutorialScreenAxis {
    #[must_use]
    pub const fn evaluate(self, width: i32, height: i32) -> i32 {
        match self {
            Self::Pixels(value) => value,
            Self::WidthMinus(value) => width - value,
            Self::HeightMinus(value) => height - value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialScreenPoint {
    pub x: TutorialScreenAxis,
    pub y: TutorialScreenAxis,
}

impl TutorialScreenPoint {
    pub const fn new(x: TutorialScreenAxis, y: TutorialScreenAxis) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn evaluate(self, width: i32, height: i32) -> [i32; 2] {
        [
            self.x.evaluate(width, height),
            self.y.evaluate(width, height),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockingWait {
    AssetPreload {
        reached_at_seconds: f32,
        source_line: u32,
        continuation_source_line: u32,
        effect_id: i32,
    },
    EffectInstantiationRetry {
        reached_at_seconds: f32,
        source_line: u32,
        continuation_source_line: u32,
        effect_id: i32,
        instance_name: &'static str,
        retry_source_line: u32,
        retry_action: ChoreographyAction,
        retry_interval_seconds: f32,
        maximum_attempts: u8,
        maximum_wait_seconds: f32,
        shared_attempt_counter: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipFinalFade {
    GameEventFadeIn,
    OpaqueOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkipContract {
    pub scene_specific: &'static [SourcedAction],
    pub common_cleanup: &'static [SourcedAction],
    pub final_fade: SkipFinalFade,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoopLifetime {
    pub cue: &'static str,
    pub starts_at_seconds: f32,
    pub natural_stop_at_seconds: Option<f32>,
    pub persists_after_scene: bool,
    pub skip_action: LoopAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialSceneChoreography {
    pub scene: TutorialScene,
    pub coroutine: &'static str,
    pub source: SourceSpan,
    /// Sum of deterministic `WaitForSeconds` and fixed loop waits.
    pub deterministic_duration_seconds: f32,
    /// Upper bound when the coroutine contains a bounded retry wait.
    pub maximum_duration_seconds: Option<f32>,
    /// Audio is scheduled by `tutorial_presenter` for this same scene.
    pub presenter_audio_scene: TutorialScene,
    pub actions: &'static [TimedAction],
    pub blocking_waits: &'static [BlockingWait],
    pub loops: &'static [LoopLifetime],
    pub skip: SkipContract,
}
