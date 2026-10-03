use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcCommand {
    ForceStop,
    Pause,
    Dead,
    ForceUpdate,
    SetAngleTo(EntityRef),
    ForceAnimation(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NpcAction {
    Spawn(NpcSpawn),
    SpawnBatch(&'static [NpcSpawn]),
    Delete(i32),
    DeleteInclusive {
        first: i32,
        last: i32,
    },
    Angle {
        id: i32,
        helper_degrees: i16,
    },
    Move {
        id: i32,
        target: PositionExpr,
        speed: i32,
    },
    WarpServer {
        id: i32,
        target: LegacyServerPosition,
    },
    WarpClient {
        id: i32,
        target: PositionExpr,
    },
    SetPosition {
        id: i32,
        target: PositionExpr,
    },
    SetRotation {
        id: i32,
        rotation: RotationExpr,
    },
    Animation {
        id: i32,
        clip: &'static str,
        once: bool,
    },
    AnimationInclusive {
        first: i32,
        last: i32,
        clip: &'static str,
        once: bool,
    },
    Command {
        id: i32,
        command: NpcCommand,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerAction {
    Animation(&'static str),
    StandForce,
    Warp(PositionExpr),
    Face(EntityRef),
    FacePosition(PositionExpr),
    FaceAngle(i32),
    Hide,
    Show,
    SetTemporaryNanoAbsent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraAction {
    Mode(CameraMode),
    /// The legacy `kCTargetPosition`: the focus point used by the current
    /// rendered frame and the start of a `FromToCamera` interpolation.
    CurrentTarget(PositionExpr),
    /// The legacy `kNewTargetPosition`: the destination focus point.
    Target(PositionExpr),
    /// `kNewTargetPosition = kCTargetPosition`: freeze the currently
    /// interpolated focus without guessing it from an authored destination.
    FreezeCurrentTarget,
    Start(PositionExpr),
    StoredStart(PositionExpr),
    TargetRotation(RotationExpr),
    Distance(f32),
    ForwardInterpolation(bool),
    LookAt(PositionExpr),
    CaptureTransform(CameraCaptureSlot),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectAction {
    Preload(i32),
    Add(EffectSpawn),
    AddBatch(&'static [EffectSpawn]),
    AddAtNpcInclusive {
        effect_id: i32,
        first: i32,
        last: i32,
        scale: f32,
        tracked: bool,
    },
    AttachBone(EffectBoneAttachment),
    ClearTracked,
    DestroyNamed(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudAction {
    PushHide,
    PopShow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopAction {
    Start(&'static str),
    StartIfAbsent(&'static str),
    StopIfPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NanoAction {
    Equip {
        nano_id: i16,
        skill_id: i16,
        slot: i16,
        stamina: i16,
    },
    Call,
    Happy,
    Stand,
    Emote(&'static str),
    SetVoiceDisabled(bool),
    DestroyPresentationObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentAction {
    TutorialWeaponByLocaleAndClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressAction {
    InitChapter { chapter: u8, step: i16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileAction {
    NpcToPlayerPair { npc_id: i32, types: [i32; 2] },
    PlayerToPositionPair { types: [i32; 2] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanAction {
    LoadEightTextures,
    Start,
    Stop,
    ReleaseEightTextures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialPictureAction {
    ShowCursor {
        position: TutorialScreenPoint,
        resource: &'static str,
        direction: TutorialCursorDirection,
        /// Numeric value of the original `ScreenPivot` enum.
        legacy_pivot: u8,
    },
    HideAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnresolvedAction {
    pub source_line: u32,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChoreographyAction {
    Hud(HudAction),
    SpatialAudio(SpatialAudioTarget),
    EventScene(bool),
    Cinematic(bool),
    FadeEnabled(bool),
    VoiceOff,
    StopBgm,
    GameFadeIn,
    SubtitleClear,
    Npc(NpcAction),
    Player(PlayerAction),
    Camera(CameraAction),
    Effect(EffectAction),
    Sequence(FrameSequence),
    Loop(LoopAction),
    Nano(NanoAction),
    Equipment(EquipmentAction),
    Progress(ProgressAction),
    Projectile(ProjectileAction),
    Pan(PanAction),
    Picture(TutorialPictureAction),
    Sound(TutorialSoundAction),
    Unresolved(UnresolvedAction),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedAction {
    pub at_seconds: f32,
    pub source_line: u32,
    pub action: ChoreographyAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourcedAction {
    pub source_line: u32,
    pub action: ChoreographyAction,
}
