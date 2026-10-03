use super::*;

pub(super) const INFECTION_C_ACTIONS: &[TimedAction] = &[
    at(
        0.0,
        4014,
        ChoreographyAction::Effect(EffectAction::Preload(705)),
    ),
    at(
        0.0,
        4015,
        ChoreographyAction::Effect(EffectAction::Preload(10)),
    ),
    at(
        0.0,
        4016,
        ChoreographyAction::Effect(EffectAction::Preload(771)),
    ),
    at(
        0.0,
        4018,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 4019, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        4020,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(0.0, 4021, ChoreographyAction::Hud(HudAction::PushHide)),
    at(
        0.0,
        4023,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 4000,
            clip: "death",
            once: true,
        }),
    ),
    at(
        0.0,
        4028,
        ChoreographyAction::Camera(CameraAction::CaptureTransform(
            CameraCaptureSlot::InfectionEntry,
        )),
    ),
    at(
        0.0,
        4026,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        0.0,
        4031,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        4032,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(4000),
            offset: ClientVec3::new(0.0, 3.0, 0.0),
        })),
    ),
    at(
        0.0,
        4033,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(4000)),
            ClientVec3::new(0.0, 2.0, 0.0),
            OrientationBasis::CapturedCameraYaw(CameraCaptureSlot::InfectionEntry),
            ClientVec3::new(0.0, 0.0, -6.0),
        ))),
    ),
    at(0.0, 4035, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        4036,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        1.0,
        4042,
        ChoreographyAction::Projectile(ProjectileAction::PlayerToPositionPair { types: [76, 77] }),
    ),
    at(
        1.5,
        4061,
        ChoreographyAction::Effect(EffectAction::Add(EffectSpawn {
            effect_id: 705,
            position: PositionExpr::Entity(EntityRef::Npc(4000)),
            scale: 1.0,
            tracked: false,
        })),
    ),
    at(
        3.5,
        4064,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: -1,
            target: PositionExpr::EntityOffset {
                entity: EntityRef::Npc(4000),
                offset: ClientVec3::new(0.0, 3.0, 0.0),
            },
        }),
    ),
    at(
        3.5,
        4065,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: -1,
            rotation: RotationExpr::EntityYaw {
                entity: EntityRef::Camera,
                pitch: 0.0,
                yaw_offset: 180.0,
                roll: 0.0,
            },
        }),
    ),
    at(
        3.5,
        4068,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::NewNano,
            offset: ClientVec3::new(0.0, 0.25, 0.0),
        })),
    ),
    at(
        3.5,
        4069,
        ChoreographyAction::Nano(NanoAction::SetVoiceDisabled(false)),
    ),
    at(
        3.5,
        4070,
        ChoreographyAction::Nano(NanoAction::Emote("call2")),
    ),
    at(
        3.5,
        4071,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::NewNano),
            ClientVec3::new(0.0, 0.25, 0.0),
            OrientationBasis::Entity(EntityRef::NewNano),
            ClientVec3::new(0.0, 0.0, 1.5),
        ))),
    ),
    at(4.3, 4073, ChoreographyAction::Nano(NanoAction::Happy)),
    at(
        5.4,
        4075,
        ChoreographyAction::Nano(NanoAction::Emote("flex")),
    ),
    at(
        5.4,
        4076,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            3000, 2673, 563.0, 968.0, -133.0, None, None, false,
        ))),
    ),
    at(
        5.4,
        4079,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 3000,
            target: PositionExpr::Client(ClientVec3::new(563.0, -132.8, 968.0)),
        }),
    ),
    at(
        5.4,
        4083,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            563.0, -132.9, 969.0,
        )))),
    ),
    at(
        5.4,
        4084,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 3000,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        5.4,
        4085,
        ChoreographyAction::Player(PlayerAction::Face(EntityRef::Npc(3000))),
    ),
    at(
        7.4,
        4089,
        ChoreographyAction::Effect(EffectAction::Add(EffectSpawn {
            effect_id: 10,
            position: PositionExpr::EntityOffset {
                entity: EntityRef::Npc(4000),
                offset: ClientVec3::new(0.0, 3.0, 0.0),
            },
            scale: 1.0,
            tracked: false,
        })),
    ),
    at(
        7.4,
        4090,
        ChoreographyAction::Nano(NanoAction::Equip {
            nano_id: 1,
            skill_id: 1,
            slot: 0,
            stamina: 100,
        }),
    ),
    at(7.4, 4091, ChoreographyAction::Npc(NpcAction::Delete(4000))),
    at(
        7.4,
        4092,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        7.4,
        4094,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            563.0, -132.0, 968.5,
        )))),
    ),
    at(
        7.4,
        4095,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(563.0, -132.0, 968.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, -90.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -2.0),
        ))),
    ),
    at(
        7.4,
        4096,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            4001,
            2696,
            558.0,
            949.0,
            -133.0,
            Some(-132),
            None,
            false,
        ))),
    ),
    at(
        8.9,
        4100,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: -1,
            target: PositionExpr::Client(ClientVec3::new(562.0, -131.8, 968.5)),
        }),
    ),
    at(
        8.9,
        4101,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: -1,
            rotation: RotationExpr::Euler(ClientVec3::new(0.0, 90.0, 0.0)),
        }),
    ),
    at(
        8.9,
        4103,
        ChoreographyAction::Nano(NanoAction::SetVoiceDisabled(true)),
    ),
    at(8.9, 4104, ChoreographyAction::Nano(NanoAction::Call)),
    at(
        8.9,
        4114,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "nano",
            once: false,
        }),
    ),
    at(
        9.9,
        4116,
        ChoreographyAction::Nano(NanoAction::Emote("hello")),
    ),
    at(11.9, 4118, ChoreographyAction::Nano(NanoAction::Stand)),
    at(
        13.9,
        4120,
        ChoreographyAction::Nano(NanoAction::Emote("dance2")),
    ),
    at(
        15.9,
        4122,
        ChoreographyAction::Nano(NanoAction::Emote("happy")),
    ),
    at(
        17.9,
        4124,
        ChoreographyAction::Nano(NanoAction::Emote("dance5")),
    ),
    at(
        19.9,
        4126,
        ChoreographyAction::Nano(NanoAction::Emote("flex")),
    ),
    at(21.9, 4128, ChoreographyAction::Nano(NanoAction::Stand)),
    at(
        25.4,
        4130,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(26.4, 4136, ChoreographyAction::Cinematic(false)),
    at(26.4, 4137, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        26.4,
        4138,
        ChoreographyAction::Picture(TutorialPictureAction::ShowCursor {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::WidthMinus(240),
                TutorialScreenAxis::HeightMinus(180),
            ),
            resource: "tut_down",
            direction: TutorialCursorDirection::Down,
            legacy_pivot: 2,
        }),
    ),
    at(
        26.4,
        4139,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        26.4,
        4141,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            563.0, -131.8, 968.5,
        )))),
    ),
    at(
        26.4,
        4142,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(563.0, -131.8, 968.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, -90.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -2.0),
        ))),
    ),
    at(
        32.4,
        4144,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(563.0, -131.8, 968.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 200.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -2.0),
        ))),
    ),
    at(32.4, 4145, ChoreographyAction::Hud(HudAction::PushHide)),
    at(32.4, 4146, ChoreographyAction::Cinematic(true)),
    at(
        32.4,
        4147,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        33.4,
        4143,
        ChoreographyAction::Picture(TutorialPictureAction::HideAll),
    ),
    at(
        33.4,
        4154,
        ChoreographyAction::Effect(EffectAction::Add(effect(
            771, 563.0, -132.866, 969.0, 1.0, false,
        ))),
    ),
    at(
        33.4,
        4155,
        ChoreographyAction::Loop(LoopAction::Start("LairCollapse_Quake_LOOP")),
    ),
    at(
        33.4,
        4156,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "stun",
            once: false,
        }),
    ),
    at(
        33.4,
        4157,
        ChoreographyAction::Nano(NanoAction::Emote("shocked")),
    ),
    at(
        33.4,
        4164,
        ChoreographyAction::Sequence(FrameSequence::CameraShake {
            frames: 100,
            seconds_per_frame: 0.01,
            amplitude: 0.2,
            start_wave: SHAKE_START_WAVE,
            target_wave: SHAKE_TARGET_WAVE,
        }),
    ),
    at(
        34.4,
        4180,
        ChoreographyAction::Effect(EffectAction::Add(EffectSpawn {
            effect_id: 10,
            position: PositionExpr::Client(ClientVec3::new(562.0, -131.8, 968.5)),
            scale: 1.0,
            tracked: false,
        })),
    ),
    at(
        34.4,
        4181,
        ChoreographyAction::Nano(NanoAction::DestroyPresentationObject),
    ),
    at(
        34.4,
        4183,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        37.4,
        4185,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "point",
            once: false,
        }),
    ),
    at(
        39.4,
        4187,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        45.9,
        4189,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 3000,
            clip: "run",
            once: false,
        }),
    ),
    at(
        45.9,
        4194,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 3000,
            target: PositionExpr::Client(ClientVec3::new(587.0, -133.0, 984.0)),
            speed: 400,
        }),
    ),
    at(
        46.9,
        4196,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(47.9, 4202, ChoreographyAction::Npc(NpcAction::Delete(3000))),
    at(47.9, 4203, ChoreographyAction::Cinematic(false)),
    at(47.9, 4204, ChoreographyAction::Hud(HudAction::PopShow)),
    at(47.9, 4205, ChoreographyAction::EventScene(false)),
    at(
        47.9,
        4206,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        47.9,
        4207,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(
        47.9,
        4208,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
];

pub(super) const NANO_POWER_A_ACTIONS: &[TimedAction] = &[
    at(0.0, 4458, ChoreographyAction::Hud(HudAction::PushHide)),
    at(
        0.0,
        4460,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 4461, ChoreographyAction::EventScene(true)),
    at(0.0, 4462, ChoreographyAction::FadeEnabled(true)),
    at(
        0.0,
        4464,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        0.0,
        4465,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            2,
            2968,
            973.0,
            702.0,
            -76.0,
            Some(88),
            None,
            false,
        ))),
    ),
    at(
        0.0,
        4469,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 5100,
            helper_degrees: 270,
        }),
    ),
    at(
        0.0,
        4470,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "worry",
            once: false,
        }),
    ),
    at(
        0.0,
        4472,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5101,
            clip: "open",
            once: false,
        }),
    ),
    at(
        0.0,
        4473,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 5101,
            helper_degrees: 104,
        }),
    ),
    at(
        0.0,
        4474,
        ChoreographyAction::Loop(LoopAction::Start("RumbleQuake_LOOP")),
    ),
    at(
        0.0,
        4476,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        0.0,
        4475,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        4477,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(5100),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        4478,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5100)),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        0.0,
        4481,
        ChoreographyAction::Player(PlayerAction::Face(EntityRef::Npc(5100))),
    ),
    at(0.5, 4483, ChoreographyAction::Cinematic(true)),
    at(
        0.5,
        4484,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_OVERLAY_OUT)),
    ),
    at(
        0.5,
        4484,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(1.5, 4492, ChoreographyAction::FadeEnabled(false)),
    at(
        3.0,
        4494,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Client(ClientVec3::new(
            907.0, 18.0, 690.0,
        )))),
    ),
    at(
        3.0,
        4495,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            1024.0, -50.0, 720.0,
        )))),
    ),
    at(
        4.0,
        4497,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 5100,
            helper_degrees: 270,
        }),
    ),
    at(
        4.0,
        4498,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 2,
            clip: "fall",
            once: false,
        }),
    ),
    at(
        4.0,
        4506,
        ChoreographyAction::Sequence(FrameSequence::CameraShake {
            frames: 150,
            seconds_per_frame: 0.01,
            amplitude: 0.2,
            start_wave: SHAKE_START_WAVE,
            target_wave: SHAKE_TARGET_WAVE,
        }),
    ),
    at(
        5.5,
        4534,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(5100),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        5.5,
        4535,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5100)),
            ClientVec3::new(0.0, 9.0, 0.0),
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        7.5,
        4537,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5100)),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        8.5,
        4539,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 5100,
            helper_degrees: 180,
        }),
    ),
    at(
        8.5,
        4540,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "worry",
            once: false,
        }),
    ),
    at(
        9.5,
        4543,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "call",
            once: false,
        }),
    ),
    at(
        12.5,
        4545,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "worry",
            once: false,
        }),
    ),
    at(
        12.5,
        4546,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        12.5,
        4547,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -4.0),
        ))),
    ),
    at(
        13.5,
        4549,
        ChoreographyAction::Loop(LoopAction::StopIfPresent),
    ),
    at(
        13.5,
        4556,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(14.5, 4562, ChoreographyAction::Npc(NpcAction::Delete(2))),
    at(
        14.5,
        4563,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(
        14.5,
        4564,
        ChoreographyAction::Camera(CameraAction::LookAt(PositionExpr::Client(ClientVec3::new(
            907.0, 10.0, 699.0,
        )))),
    ),
    at(14.5, 4565, ChoreographyAction::Cinematic(false)),
    at(14.5, 4566, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        14.5,
        4567,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(14.5, 4568, ChoreographyAction::EventScene(false)),
    at(
        14.5,
        4569,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
];
