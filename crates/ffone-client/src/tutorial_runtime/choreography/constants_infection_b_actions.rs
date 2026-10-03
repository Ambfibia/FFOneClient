use super::*;

pub(super) const INFECTION_B_ACTIONS: &[TimedAction] = &[
    at(0.0, 3926, ChoreographyAction::GameFadeIn),
    at(
        0.0,
        3927,
        ChoreographyAction::Effect(EffectAction::Preload(740)),
    ),
    at(0.0, 3929, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        3930,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(0.0, 3931, ChoreographyAction::Hud(HudAction::PushHide)),
    at(
        0.0,
        3932,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            4000,
            2678,
            563.0,
            967.0,
            -133.0,
            Some(266),
            None,
            false,
        ))),
    ),
    at(
        0.0,
        3936,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            588.4, -133.5, 983.8,
        )))),
    ),
    at(
        0.0,
        3940,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        0.0,
        3941,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(3000),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        3943,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Midpoint {
            left: EntityRef::Npc(3000),
            right: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        3944,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Midpoint {
                left: EntityRef::Npc(3000),
                right: EntityRef::Player,
            },
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 320.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        0.0,
        3944,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Midpoint {
                left: EntityRef::Npc(3000),
                right: EntityRef::Player,
            },
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 320.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        0.0,
        3948,
        ChoreographyAction::Player(PlayerAction::Face(EntityRef::Npc(3000))),
    ),
    at(
        0.0,
        3949,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 3000,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(0.0, 3950, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        3951,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        1.0,
        3958,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "event1",
            once: false,
        }),
    ),
    at(
        4.0,
        3960,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "event2",
            once: false,
        }),
    ),
    at(
        7.0,
        3962,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        12.0,
        3964,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "point",
            once: false,
        }),
    ),
    at(
        13.6,
        3967,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        17.1,
        3969,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        17.1,
        3967,
        ChoreographyAction::Camera(CameraAction::CaptureTransform(
            CameraCaptureSlot::InfectionReturnStart,
        )),
    ),
    at(
        17.1,
        3969,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(oriented(
            PositionAnchor::Client(ClientVec3::new(563.0, -131.0, 967.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(15.0, 270.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        17.1,
        3971,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            563.0, -131.0, 967.0,
        )))),
    ),
    at(
        17.1,
        3973,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(563.0, -131.0, 967.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(15.0, 270.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        17.1,
        3974,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Client(ClientVec3::new(563.0, -131.0, 967.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(15.0, 270.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -15.0),
        ))),
    ),
    at(
        18.1,
        3977,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        18.1,
        3979,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(
        18.1,
        3980,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 3000,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        21.1,
        3982,
        ChoreographyAction::Sound(TutorialSoundAction::UiOneShot {
            cue: "FusionButtercup_Stand1",
        }),
    ),
    at(
        24.1,
        3984,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Midpoint {
            left: EntityRef::Npc(3000),
            right: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        24.1,
        3985,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::OrientedOffset {
            origin: PositionAnchor::CapturedCamera(CameraCaptureSlot::InfectionReturnStart),
            world_offset: ClientVec3::ZERO,
            orientation: OrientationBasis::Euler(ClientVec3::ZERO),
            local_offset: ClientVec3::ZERO,
        })),
    ),
    at(
        24.1,
        3986,
        ChoreographyAction::Camera(CameraAction::ForwardInterpolation(false)),
    ),
    at(
        24.6,
        3988,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(
        30.1,
        3994,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 3000,
            target: PositionExpr::Client(ClientVec3::new(565.0, -133.0, 979.0)),
            speed: 300,
        }),
    ),
    at(
        30.1,
        3995,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "run",
            once: false,
        }),
    ),
    at(32.1, 3997, ChoreographyAction::Npc(NpcAction::Delete(3000))),
    at(
        32.1,
        3999,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            3000, 2902, 562.0, 967.0, -133.0, None, None, false,
        ))),
    ),
    at(
        32.1,
        4000,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 3000,
            command: NpcCommand::SetAngleTo(EntityRef::Npc(4000)),
        }),
    ),
    at(
        32.1,
        4001,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "melee1event",
            once: false,
        }),
    ),
    at(
        32.1,
        4002,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 4000,
            command: NpcCommand::SetAngleTo(EntityRef::Npc(3000)),
        }),
    ),
    at(
        32.1,
        4003,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 4000,
            clip: "melee1",
            once: false,
        }),
    ),
    at(
        32.1,
        4004,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(
        33.1,
        4010,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(33.1, 4011, ChoreographyAction::Cinematic(false)),
    at(33.1, 4012, ChoreographyAction::Hud(HudAction::PopShow)),
    at(33.1, 4013, ChoreographyAction::EventScene(false)),
    at(
        33.1,
        4014,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
];
