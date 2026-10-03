use super::*;

pub(super) const BASIC_COMBAT_C_ACTIONS: &[TimedAction] = &[
    at(
        0.0,
        3342,
        ChoreographyAction::Pan(PanAction::LoadEightTextures),
    ),
    at(0.0, 3346, ChoreographyAction::Hud(HudAction::PushHide)),
    at(0.0, 3348, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        3349,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        0.0,
        3353,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        0.0,
        3358,
        ChoreographyAction::Camera(CameraAction::StoredStart(PositionExpr::Entity(
            EntityRef::Camera,
        ))),
    ),
    at(
        0.0,
        3354,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Midpoint {
            left: EntityRef::Npc(1005),
            right: EntityRef::Npc(101),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        3359,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Midpoint {
            left: EntityRef::Npc(1005),
            right: EntityRef::Npc(101),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        3355,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(0.0, 180.0, 0.0),
        ))),
    ),
    at(
        0.0,
        3356,
        ChoreographyAction::Camera(CameraAction::Distance(5.0)),
    ),
    at(
        0.0,
        3360,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Midpoint {
                left: EntityRef::Npc(1005),
                right: EntityRef::Npc(101),
            },
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        0.0,
        3362,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 100,
            target: PositionExpr::Client(ClientVec3::new(566.0, -101.0, 665.0)),
            speed: 300,
        }),
    ),
    at(
        0.0,
        3363,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "run",
            once: false,
        }),
    ),
    at(0.0, 3364, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        3365,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        1.0,
        3371,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            566.0, -101.0, 665.0,
        )))),
    ),
    at(
        1.0,
        3372,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "cheer",
            once: false,
        }),
    ),
    at(
        1.0,
        3374,
        ChoreographyAction::Player(PlayerAction::FacePosition(PositionExpr::Client(
            ClientVec3::new(566.0, -101.0, 664.0),
        ))),
    ),
    at(
        3.0,
        3380,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 101,
            target: PositionExpr::Client(ClientVec3::new(566.0, -101.0, 665.0)),
            speed: 300,
        }),
    ),
    at(
        3.0,
        3381,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "run",
            once: false,
        }),
    ),
    at(
        4.0,
        3387,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 101,
            target: oriented(
                PositionAnchor::Client(ClientVec3::new(566.0, -101.0, 665.0)),
                ClientVec3::new(0.5, -1.0, -0.5),
                OrientationBasis::Entity(EntityRef::Player),
                ClientVec3::new(0.0, 0.0, 1.0),
            ),
        }),
    ),
    at(
        4.0,
        3392,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 100,
            target: oriented(
                PositionAnchor::Client(ClientVec3::new(566.0, -101.0, 665.0)),
                ClientVec3::new(-0.5, -1.0, 0.5),
                OrientationBasis::Entity(EntityRef::Player),
                ClientVec3::new(0.0, 0.0, 1.0),
            ),
        }),
    ),
    at(
        4.0,
        3392,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        4.0,
        3393,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 100,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        4.0,
        3395,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        4.0,
        3396,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        5.0,
        3398,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        5.0,
        3401,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::EntityYaw {
            entity: EntityRef::Player,
            pitch: 0.0,
            yaw_offset: 0.0,
            roll: 0.0,
        })),
    ),
    at(
        5.0,
        3402,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        5.0,
        3403,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        5.0,
        3403,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        5.0,
        3404,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(5.0, 3407, ChoreographyAction::Npc(NpcAction::Delete(1005))),
    at(
        5.0,
        3409,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        8.0,
        3411,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "event1",
            once: false,
        }),
    ),
    at(
        11.0,
        3413,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        14.5,
        3415,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "point",
            once: false,
        }),
    ),
    at(
        15.5,
        3422,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::new(4.5, 10.5, 15.0),
        })),
    ),
    at(17.5, 3424, ChoreographyAction::Pan(PanAction::Start)),
    at(
        25.5,
        3427,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        25.5,
        3428,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        25.5,
        3429,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "stand1",
            once: false,
        }),
    ),
    at(39.0, 3432, ChoreographyAction::Pan(PanAction::Stop)),
    at(
        39.0,
        3433,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "event2",
            once: false,
        }),
    ),
    at(
        39.0,
        3434,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        43.0,
        3437,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 100,
            target: oriented(
                PositionAnchor::Client(ClientVec3::new(566.0, -101.0, 665.0)),
                ClientVec3::ZERO,
                OrientationBasis::Entity(EntityRef::Player),
                ClientVec3::new(-40.0, 0.0, 10.0),
            ),
            speed: 500,
        }),
    ),
    at(
        43.0,
        3438,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 101,
            target: oriented(
                PositionAnchor::Client(ClientVec3::new(566.0, -101.0, 665.0)),
                ClientVec3::ZERO,
                OrientationBasis::Entity(EntityRef::Player),
                ClientVec3::new(-40.0, 0.0, 10.0),
            ),
            speed: 500,
        }),
    ),
    at(
        43.0,
        3439,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "run",
            once: false,
        }),
    ),
    at(
        43.0,
        3440,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "run",
            once: false,
        }),
    ),
    at(
        44.0,
        3442,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(45.0, 3448, ChoreographyAction::Npc(NpcAction::Delete(100))),
    at(45.0, 3449, ChoreographyAction::Npc(NpcAction::Delete(101))),
    at(
        45.0,
        3450,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(45.0, 3451, ChoreographyAction::Cinematic(false)),
    at(45.0, 3452, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        45.0,
        3453,
        ChoreographyAction::Progress(ProgressAction::InitChapter {
            chapter: 2,
            step: 0,
        }),
    ),
    at(45.0, 3454, ChoreographyAction::EventScene(false)),
    at(
        45.0,
        3455,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        45.0,
        3456,
        ChoreographyAction::Pan(PanAction::ReleaseEightTextures),
    ),
];

pub(super) const INFECTION_A_ACTIONS: &[TimedAction] = &[
    at(
        0.0,
        3883,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 3884, ChoreographyAction::Hud(HudAction::PushHide)),
    at(0.0, 3886, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        3887,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            1012, 2695, 564.31, 724.16, -91.0, None, None, false,
        ))),
    ),
    at(
        0.0,
        3888,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        0.0,
        3889,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Client(
            ClientVec3::new(584.0, -95.0, 728.0),
        ))),
    ),
    at(
        0.0,
        3890,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            584.0, -95.0, 728.0,
        )))),
    ),
    at(
        0.0,
        3891,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Client(ClientVec3::new(
            577.0, -90.0, 747.0,
        )))),
    ),
    at(
        0.0,
        3891,
        ChoreographyAction::Camera(CameraAction::StoredStart(PositionExpr::Client(
            ClientVec3::new(577.0, -90.0, 747.0),
        ))),
    ),
    at(0.0, 3894, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        3895,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        3.0,
        3903,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            564.0, -92.0, 745.0,
        )))),
    ),
    at(
        6.0,
        3905,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            557.0, -90.0, 746.0,
        )))),
    ),
    at(
        6.0,
        3906,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Client(ClientVec3::new(
            572.0, -80.0, 747.0,
        )))),
    ),
    at(
        9.0,
        3908,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            568.0, -86.0, 725.0,
        )))),
    ),
    at(
        9.0,
        3909,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Client(ClientVec3::new(
            577.0, -92.0, 749.0,
        )))),
    ),
    at(
        14.0,
        3911,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(
        15.0,
        3917,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(15.0, 3918, ChoreographyAction::Cinematic(false)),
    at(15.0, 3919, ChoreographyAction::Hud(HudAction::PopShow)),
    at(15.0, 3920, ChoreographyAction::Npc(NpcAction::Delete(1012))),
    at(
        15.0,
        3921,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(15.0, 3922, ChoreographyAction::EventScene(false)),
];
