use super::*;

pub(super) const BASIC_COMBAT_B_ACTIONS: &[TimedAction] = &[
    at(
        0.0,
        3146,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 3147, ChoreographyAction::Hud(HudAction::PushHide)),
    at(0.0, 3149, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        3150,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        0.0,
        3152,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::EventCameraControl)),
    ),
    at(
        0.0,
        3153,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(1004),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        3154,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(10.0, 90.0, 0.0),
        ))),
    ),
    at(
        0.0,
        3155,
        ChoreographyAction::Camera(CameraAction::Distance(5.0)),
    ),
    at(0.0, 3157, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        3158,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        1.0,
        3165,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "kick",
            once: false,
        }),
    ),
    at(
        1.8,
        3167,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 1004,
            clip: "death",
            once: true,
        }),
    ),
    at(
        2.3,
        3169,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        2.3,
        3170,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        3.3,
        3173,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "wipe",
            once: false,
        }),
    ),
    at(
        3.3,
        3174,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            566.0, -101.0, 665.0,
        )))),
    ),
    at(
        3.3,
        3178,
        ChoreographyAction::Player(PlayerAction::FaceAngle(50)),
    ),
    at(
        4.8,
        3180,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "stand2",
            once: false,
        }),
    ),
    at(
        5.8,
        3186,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 101,
            target: PositionExpr::Client(ClientVec3::new(567.0, -101.0, 664.0)),
            speed: 600,
        }),
    ),
    at(
        5.8,
        3191,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 100,
            target: PositionExpr::Client(ClientVec3::new(565.0, -101.0, 666.0)),
            speed: 600,
        }),
    ),
    at(
        5.8,
        3194,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "run",
            once: false,
        }),
    ),
    at(
        5.8,
        3195,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "run",
            once: false,
        }),
    ),
    at(
        5.8,
        3196,
        ChoreographyAction::Effect(EffectAction::Add(EffectSpawn {
            effect_id: 372,
            position: PositionExpr::Entity(EntityRef::Npc(1004)),
            scale: 2.0,
            tracked: true,
        })),
    ),
    at(
        5.8,
        3197,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 1004,
            command: NpcCommand::Dead,
        }),
    ),
    at(
        6.3,
        3199,
        ChoreographyAction::Projectile(ProjectileAction::NpcToPlayerPair {
            npc_id: 1004,
            types: [76, 77],
        }),
    ),
    at(
        6.8,
        3220,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 101,
            target: oriented(
                PositionAnchor::Client(ClientVec3::new(566.5, -101.0, 664.5)),
                ClientVec3::ZERO,
                OrientationBasis::Entity(EntityRef::Player),
                ClientVec3::new(0.0, 0.0, 1.0),
            ),
        }),
    ),
    at(
        6.8,
        3225,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 100,
            target: oriented(
                PositionAnchor::Client(ClientVec3::new(565.5, -101.0, 665.5)),
                ClientVec3::ZERO,
                OrientationBasis::Entity(EntityRef::Player),
                ClientVec3::new(0.0, 0.0, 1.0),
            ),
        }),
    ),
    at(
        6.8,
        3227,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        6.8,
        3228,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        6.8,
        3223,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 100,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        6.8,
        3224,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        6.8,
        3231,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        6.8,
        3232,
        ChoreographyAction::Camera(CameraAction::StoredStart(PositionExpr::Entity(
            EntityRef::Camera,
        ))),
    ),
    at(
        6.8,
        3234,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        6.8,
        3236,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 45.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -4.0),
        ))),
    ),
    at(
        8.8,
        3238,
        ChoreographyAction::Effect(EffectAction::ClearTracked),
    ),
    at(8.8, 3239, ChoreographyAction::Npc(NpcAction::Delete(1004))),
    at(
        8.8,
        3241,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        8.8,
        3236,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        14.9,
        3244,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "stand1",
            once: false,
        }),
    ),
    at(14.9, 3245, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        14.9,
        3246,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(15.9, 3252, ChoreographyAction::Cinematic(false)),
    at(
        15.9,
        3253,
        ChoreographyAction::Picture(TutorialPictureAction::ShowCursor {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::WidthMinus(120),
                TutorialScreenAxis::Pixels(50),
            ),
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            legacy_pivot: 0,
        }),
    ),
    at(
        29.1,
        3256,
        ChoreographyAction::Picture(TutorialPictureAction::HideAll),
    ),
    at(29.1, 3257, ChoreographyAction::Hud(HudAction::PushHide)),
    at(29.1, 3258, ChoreographyAction::Cinematic(true)),
    at(
        29.1,
        3259,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        32.6,
        3268,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: -10,
        }),
    ),
    at(
        32.6,
        3269,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: -10,
        }),
    ),
    at(
        32.6,
        3270,
        ChoreographyAction::Player(PlayerAction::FaceAngle(170)),
    ),
    at(
        32.6,
        3271,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        32.6,
        3272,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        33.1,
        3274,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            1005,
            2675,
            567.0,
            658.0,
            -97.0,
            Some(197),
            Some("stand2"),
            false,
        ))),
    ),
    at(
        33.1,
        3278,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            567.0, -100.0, 658.0,
        )))),
    ),
    at(
        33.6,
        3280,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(0.0, 210.0, 0.0),
        ))),
    ),
    at(
        33.6,
        3281,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(567.0, -100.0, 658.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 210.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        34.1,
        3284,
        ChoreographyAction::Sound(TutorialSoundAction::WorldOneShot {
            cue: "Cyberus_Landing",
            position: ClientVec3::new(567.0, -102.0, 658.0),
        }),
    ),
    at(
        34.1,
        3285,
        ChoreographyAction::Camera(CameraAction::StoredStart(PositionExpr::Entity(
            EntityRef::Camera,
        ))),
    ),
    at(
        34.1,
        3286,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Entity(EntityRef::Camera))),
    ),
    at(
        34.1,
        3287,
        ChoreographyAction::Camera(CameraAction::FreezeCurrentTarget),
    ),
    at(
        34.1,
        3290,
        ChoreographyAction::Camera(CameraAction::ForwardInterpolation(false)),
    ),
    at(
        34.1,
        3291,
        ChoreographyAction::Sequence(FrameSequence::CameraShake {
            frames: 100,
            seconds_per_frame: 0.01,
            amplitude: 0.1,
            start_wave: SHAKE_START_WAVE,
            target_wave: SHAKE_TARGET_WAVE,
        }),
    ),
    at(
        35.1,
        3308,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        35.1,
        3309,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(10.0, 180.0, 0.0),
        ))),
    ),
    at(
        35.1,
        3310,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -6.0),
        ))),
    ),
    at(
        35.1,
        3311,
        ChoreographyAction::Camera(CameraAction::ForwardInterpolation(true)),
    ),
    at(
        35.1,
        3312,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 1005,
            clip: "idle",
            once: false,
        }),
    ),
    at(
        36.1,
        3318,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 101,
            target: PositionExpr::Client(ClientVec3::new(569.3, -100.0, 659.1)),
            speed: 600,
        }),
    ),
    at(
        36.1,
        3323,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 100,
            target: PositionExpr::Client(ClientVec3::new(564.4, -101.0, 662.3)),
            speed: 600,
        }),
    ),
    at(
        36.1,
        3325,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "run",
            once: false,
        }),
    ),
    at(
        36.1,
        3326,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "run",
            once: false,
        }),
    ),
    at(
        37.1,
        3328,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 100,
            command: NpcCommand::ForceStop,
        }),
    ),
    at(
        37.1,
        3329,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "melee1",
            once: false,
        }),
    ),
    at(
        37.1,
        3330,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: -10,
        }),
    ),
    at(
        37.6,
        3332,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::ForceStop,
        }),
    ),
    at(
        37.6,
        3333,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 60,
        }),
    ),
    at(
        37.6,
        3334,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "melee1event",
            once: false,
        }),
    ),
    at(
        38.6,
        3337,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(
        39.6,
        3343,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(
        39.6,
        3344,
        ChoreographyAction::Camera(CameraAction::LookAt(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(39.6, 3345, ChoreographyAction::Cinematic(false)),
    at(39.6, 3346, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        39.6,
        3347,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(39.6, 3348, ChoreographyAction::EventScene(false)),
    at(
        39.6,
        3349,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
];
