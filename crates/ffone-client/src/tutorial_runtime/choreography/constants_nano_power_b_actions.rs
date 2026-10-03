use super::*;

pub(super) const NANO_POWER_A2_ACTIONS: &[TimedAction] = &[
    at(0.0, 4576, ChoreographyAction::Hud(HudAction::PushHide)),
    at(
        0.0,
        4578,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 4579, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        4582,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            907.0, 11.0, 713.0,
        )))),
    ),
    at(0.0, 4585, ChoreographyAction::Player(PlayerAction::Hide)),
    at(
        0.0,
        4587,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        0.0,
        4588,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Client(
            ClientVec3::new(907.0, 12.0, 710.0),
        ))),
    ),
    at(
        0.0,
        4590,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            907.0, 12.0, 710.0,
        )))),
    ),
    at(
        0.0,
        4589,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(907.0, 12.0, 710.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -2.5),
        ))),
    ),
    at(
        0.0,
        4591,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Client(ClientVec3::new(907.0, 12.0, 710.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -2.5),
        ))),
    ),
    at(
        0.0,
        4591,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            5000,
            2677,
            907.0,
            710.0,
            17.0,
            Some(183),
            None,
            false,
        ))),
    ),
    at(0.0, 4594, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        4595,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        1.0,
        4601,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            907.0, 14.0, 710.0,
        )))),
    ),
    at(
        1.0,
        4602,
        ChoreographyAction::Player(PlayerAction::Face(EntityRef::Npc(5000))),
    ),
    at(
        3.0,
        4604,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5000,
            clip: "skill0",
            once: true,
        }),
    ),
    at(
        4.0,
        4606,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5000,
            clip: "idle",
            once: true,
        }),
    ),
    at(
        4.0,
        4607,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(
        5.0,
        4613,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(
        5.0,
        4614,
        ChoreographyAction::Camera(CameraAction::LookAt(PositionExpr::Client(ClientVec3::new(
            907.0, 10.0, 699.0,
        )))),
    ),
    at(5.0, 4615, ChoreographyAction::Cinematic(false)),
    at(5.0, 4616, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        5.0,
        4617,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(5.0, 4618, ChoreographyAction::EventScene(false)),
    at(5.0, 4619, ChoreographyAction::Player(PlayerAction::Show)),
];

pub(super) const NANO_POWER_B_SPAWNS: &[NpcSpawn] = &[
    spawn(6000, 2665, 990.0, 548.0, 111.0, Some(53), None, false),
    spawn(6001, 2669, 986.0, 544.0, 111.0, Some(231), None, false),
    spawn(6002, 2666, 990.0, 544.0, 111.0, Some(184), None, false),
    spawn(6003, 2903, 986.0, 548.0, 111.0, Some(331), None, false),
];

pub(super) const NANO_POWER_B_EXPLOSIONS: &[EffectSpawn] = &[
    effect(767, 907.0, 11.0, 699.0, 1.0, true),
    effect(767, 904.0, 10.0, 722.0, 1.0, true),
    effect(767, 904.0, -8.0, 699.0, 1.0, true),
    effect(767, 884.0, -33.0, 701.0, 1.0, true),
];

pub(super) const NANO_POWER_B_ACTIONS: &[TimedAction] = &[
    at(
        0.0,
        4626,
        ChoreographyAction::Effect(EffectAction::Preload(767)),
    ),
    at(
        0.0,
        4627,
        ChoreographyAction::Sound(TutorialSoundAction::Ambient { cue: "none" }),
    ),
    at(0.0, 4629, ChoreographyAction::GameFadeIn),
    at(
        0.0,
        4630,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 4631, ChoreographyAction::Hud(HudAction::PushHide)),
    at(0.0, 4633, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        4634,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        0.0,
        4635,
        ChoreographyAction::Effect(EffectAction::Preload(739)),
    ),
    at(
        0.0,
        4636,
        ChoreographyAction::Effect(EffectAction::Preload(741)),
    ),
    at(
        0.0,
        4638,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        0.0,
        4639,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(5100),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        4642,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(5100),
            offset: ClientVec3::UP,
        })),
    ),
    at(
        0.0,
        4640,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5100)),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        0.0,
        4641,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5100)),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        0.0,
        4647,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            907.0, 11.119, 687.0,
        )))),
    ),
    at(
        0.0,
        4648,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 5100,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        0.0,
        4649,
        ChoreographyAction::Player(PlayerAction::Face(EntityRef::Npc(5100))),
    ),
    at(0.0, 4650, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        4651,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        1.0,
        4658,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 5101,
            command: NpcCommand::ForceUpdate,
        }),
    ),
    at(
        1.0,
        4659,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5100)),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 120.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        2.0,
        4661,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "admiration",
            once: false,
        }),
    ),
    at(
        6.0,
        4664,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        13.0,
        4666,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5100)),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        13.0,
        4667,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 5100,
            target: PositionExpr::Entity(EntityRef::Npc(5101)),
            speed: 200,
        }),
    ),
    at(
        13.0,
        4668,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "run",
            once: false,
        }),
    ),
    at(
        13.0,
        4669,
        ChoreographyAction::Player(PlayerAction::Animation("run")),
    ),
    at(13.0, 4670, ChoreographyAction::FadeEnabled(true)),
    at(
        13.0,
        4671,
        ChoreographyAction::Sequence(FrameSequence::PlayerPathAndFade {
            frames: 25,
            seconds_per_frame: 0.01,
            start: ClientVec3::new(907.0, 11.119, 686.0),
            translation_per_frame: ClientVec3::new(0.0, 0.0, -0.02),
            fade_from: 0.0,
            fade_numerator_per_frame: 1.0,
            fade_divisor: 26.0,
        }),
    ),
    at(
        13.25,
        4679,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Entity(
            EntityRef::StartPosition,
        ))),
    ),
    at(
        13.25,
        4680,
        ChoreographyAction::Npc(NpcAction::Delete(5100)),
    ),
    at(
        13.25,
        4681,
        ChoreographyAction::Npc(NpcAction::Delete(5000)),
    ),
    at(
        13.25,
        4684,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 5101,
            target: PositionExpr::Client(ClientVec3::new(928.0, 40.0, 682.0)),
        }),
    ),
    at(
        13.25,
        4685,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: 5101,
            rotation: RotationExpr::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
        }),
    ),
    at(
        13.25,
        4686,
        ChoreographyAction::Npc(NpcAction::SpawnBatch(NANO_POWER_B_SPAWNS)),
    ),
    at(
        13.25,
        4698,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        13.25,
        4697,
        ChoreographyAction::Effect(EffectAction::Add(effect(
            739, 987.1, 111.3, 546.8, 1.0, false,
        ))),
    ),
    at(
        13.25,
        4701,
        ChoreographyAction::Effect(EffectAction::Add(effect(
            741, 987.1, 111.3, 546.8, 1.0, false,
        ))),
    ),
    at(
        13.25,
        4717,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: 5101,
            rotation: RotationExpr::LookAtYaw {
                target: ClientVec3::new(994.0, 100.0, 554.0),
                pitch: -30.0,
                yaw_offset: 0.0,
                roll: 0.0,
            },
        }),
    ),
    at(
        13.25,
        4718,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Entity(
            EntityRef::Npc(5101),
        ))),
    ),
    at(
        13.25,
        4722,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Entity(EntityRef::Npc(
            5101,
        )))),
    ),
    at(
        13.25,
        4723,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5101)),
            ClientVec3::ZERO,
            OrientationBasis::Entity(EntityRef::Npc(5101)),
            ClientVec3::new(0.0, 0.0, 50.0),
        ))),
    ),
    at(
        13.25,
        4724,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Entity(EntityRef::Npc(5101)),
            ClientVec3::ZERO,
            OrientationBasis::Entity(EntityRef::Npc(5101)),
            ClientVec3::new(0.0, 0.0, 50.0),
        ))),
    ),
    at(
        13.25,
        4735,
        ChoreographyAction::Player(PlayerAction::SetTemporaryNanoAbsent),
    ),
    at(
        13.25,
        4737,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5101,
            clip: "fly",
            once: false,
        }),
    ),
    at(
        13.25,
        4738,
        ChoreographyAction::Effect(EffectAction::AddBatch(NANO_POWER_B_EXPLOSIONS)),
    ),
    at(13.45, 4757, ChoreographyAction::FadeEnabled(false)),
    at(
        13.45,
        4759,
        ChoreographyAction::Sequence(FrameSequence::NpcLerp {
            id: 5101,
            frames: 100,
            seconds_per_frame: 0.01,
            from: ClientVec3::new(928.0, 40.0, 682.0),
            to: ClientVec3::new(994.0, 100.0, 554.0),
            numerator_offset: 0,
            denominator: 500.0,
        }),
    ),
    at(14.45, 4777, ChoreographyAction::FadeEnabled(false)),
    at(
        14.45,
        4777,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Client(
            ClientVec3::new(980.8, 88.0, 579.6),
        ))),
    ),
    at(
        14.45,
        4778,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            980.8, 88.0, 579.6,
        )))),
    ),
    at(
        14.45,
        4779,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(980.8, 88.0, 579.6)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 120.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -40.0),
        ))),
    ),
    at(
        14.45,
        4780,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Client(ClientVec3::new(980.8, 88.0, 579.6)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 120.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -40.0),
        ))),
    ),
    at(
        14.45,
        4782,
        ChoreographyAction::Sequence(FrameSequence::NpcLerp {
            id: 5101,
            frames: 100,
            seconds_per_frame: 0.01,
            from: ClientVec3::new(928.0, 40.0, 682.0),
            to: ClientVec3::new(994.0, 100.0, 554.0),
            numerator_offset: 300,
            denominator: 500.0,
        }),
    ),
    at(15.45, 4788, ChoreographyAction::FadeEnabled(true)),
    at(
        15.45,
        4792,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 5101,
            target: PositionExpr::Client(ClientVec3::new(993.2, 111.4, 548.4)),
        }),
    ),
    at(
        15.45,
        4793,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: 5101,
            rotation: RotationExpr::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
        }),
    ),
    at(
        15.45,
        4794,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 5101,
            command: NpcCommand::ForceAnimation("open"),
        }),
    ),
    at(
        15.45,
        4797,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            987.9, 111.2, 547.3,
        )))),
    ),
    at(
        15.45,
        4801,
        ChoreographyAction::Player(PlayerAction::FacePosition(PositionExpr::Client(
            ClientVec3::new(987.1, 111.3, 546.8),
        ))),
    ),
    at(
        15.45,
        4802,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        15.45,
        4800,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        15.45,
        4803,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        15.45,
        4806,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 122.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -8.0),
        ))),
    ),
    at(
        15.45,
        4807,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 122.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        15.45,
        4812,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 6001,
            target: PositionExpr::Client(ClientVec3::new(987.5, 111.2, 545.5)),
        }),
    ),
    at(
        15.45,
        4817,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 6000,
            target: PositionExpr::Client(ClientVec3::new(988.0, 111.2, 546.0)),
        }),
    ),
    at(
        15.45,
        4822,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 6002,
            target: PositionExpr::Client(ClientVec3::new(988.2, 111.2, 546.6)),
        }),
    ),
    at(
        15.45,
        4827,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 6003,
            target: PositionExpr::Client(ClientVec3::new(987.5, 111.2, 547.9)),
        }),
    ),
    at(15.65, 4829, ChoreographyAction::FadeEnabled(false)),
    at(
        15.65,
        4830,
        ChoreographyAction::Npc(NpcAction::AnimationInclusive {
            first: 6000,
            last: 6003,
            clip: "worry",
            once: false,
        }),
    ),
    at(
        15.65,
        4834,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(6001),
            offset: ClientVec3::new(0.0, 1.2, 0.0),
        })),
    ),
    at(
        15.65,
        4835,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(6001)),
            ClientVec3::new(0.0, 1.2, 0.0),
            OrientationBasis::Euler(ClientVec3::new(0.0, 310.0, 0.0)),
            ClientVec3::new(0.0, 0.0, 3.0),
        ))),
    ),
    at(
        16.65,
        4844,
        ChoreographyAction::Loop(LoopAction::Start("DexterHologram_LOOP")),
    ),
    at(
        19.65,
        4855,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(6000),
            offset: ClientVec3::new(0.0, 1.2, 0.0),
        })),
    ),
    at(
        19.65,
        4856,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(6000)),
            ClientVec3::new(0.0, 1.2, 0.0),
            OrientationBasis::Entity(EntityRef::Npc(6000)),
            ClientVec3::new(0.0, 0.0, 3.0),
        ))),
    ),
    at(
        22.65,
        4858,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Npc(6002),
            offset: ClientVec3::new(0.0, 1.2, 0.0),
        })),
    ),
    at(
        22.65,
        4859,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Npc(6002)),
            ClientVec3::new(0.0, 1.2, 0.0),
            OrientationBasis::Entity(EntityRef::Npc(6002)),
            ClientVec3::new(0.0, 0.0, 3.0),
        ))),
    ),
    at(
        25.65,
        4861,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::new(0.0, 1.2, 0.0),
        })),
    ),
    at(
        25.65,
        4862,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 1.2, 0.0),
            OrientationBasis::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 0.0, 3.0),
        ))),
    ),
    at(
        27.65,
        4864,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 1.2, 0.0),
            OrientationBasis::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 0.0, 2.0),
        ))),
    ),
    at(
        40.65,
        4867,
        ChoreographyAction::Effect(EffectAction::DestroyNamed("Dexter hologram effect 739")),
    ),
    at(
        40.65,
        4868,
        ChoreographyAction::Loop(LoopAction::StopIfPresent),
    ),
    at(
        41.15,
        4873,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 6003,
            clip: "admiration",
            once: false,
        }),
    ),
    at(
        41.15,
        4875,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 6003,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        42.15,
        4877,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 1.2, 0.0),
            OrientationBasis::Entity(EntityRef::Player),
            ClientVec3::new(0.0, 0.0, 2.0),
        ))),
    ),
    at(45.15, 4879, ChoreographyAction::FadeEnabled(true)),
    at(
        45.15,
        4880,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_OVERLAY_IN)),
    ),
    at(
        46.15,
        4886,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(46.15, 4887, ChoreographyAction::Cinematic(false)),
    at(46.15, 4888, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        46.15,
        4889,
        ChoreographyAction::Effect(EffectAction::ClearTracked),
    ),
    at(
        46.15,
        4890,
        ChoreographyAction::Effect(EffectAction::DestroyNamed("portal effect 741")),
    ),
    at(
        46.15,
        4891,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(46.15, 4892, ChoreographyAction::EventScene(false)),
];
