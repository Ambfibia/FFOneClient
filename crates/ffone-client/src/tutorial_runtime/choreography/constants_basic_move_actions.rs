use super::*;

pub(super) const BASIC_MOVE_ACTIONS: &[TimedAction] = &[
    at(
        0.0,
        2451,
        ChoreographyAction::Effect(EffectAction::Preload(767)),
    ),
    at(
        0.0,
        2452,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 2453, ChoreographyAction::Hud(HudAction::PushHide)),
    at(0.0, 2454, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        2455,
        ChoreographyAction::Player(PlayerAction::Animation("staying")),
    ),
    at(
        0.0,
        2457,
        ChoreographyAction::Npc(NpcAction::SpawnBatch(BASIC_MOVE_SPAWNS)),
    ),
    at(0.0, 2499, ChoreographyAction::Cinematic(true)),
    at(0.0, 2500, ChoreographyAction::FadeEnabled(true)),
    at(
        0.0,
        2502,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_OVERLAY_OUT)),
    ),
    at(
        0.0,
        2502,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(1.0, 2509, ChoreographyAction::FadeEnabled(false)),
    at(
        1.0,
        2510,
        ChoreographyAction::Player(PlayerAction::Animation("standup")),
    ),
    at(
        2.0,
        2515,
        ChoreographyAction::Sequence(FrameSequence::SubtitleTypewriter {
            localization_key: "tutorial.cinematic.tech_square_future",
            canonical_english_chars: 24,
            first_character: 1,
            seconds_per_character: 0.05,
        }),
    ),
    at(
        4.15,
        2523,
        ChoreographyAction::Sequence(FrameSequence::Fade(FadeSequence {
            channel: FadeChannel::Subtitle,
            from: 1.0,
            to: 0.0,
            steps: 10,
            divisor: 9.0,
            seconds_per_step: 0.1,
        })),
    ),
    at(5.15, 2528, ChoreographyAction::SubtitleClear),
    at(
        5.15,
        2530,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        5.15,
        2531,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Client(
            ClientVec3::new(856.0, -40.0, 719.0),
        ))),
    ),
    at(
        5.15,
        2532,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            856.0, -40.0, 719.0,
        )))),
    ),
    at(
        5.15,
        2535,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Client(ClientVec3::new(
            826.0, -40.0, 719.0,
        )))),
    ),
    at(
        5.15,
        2540,
        ChoreographyAction::Camera(CameraAction::StoredStart(PositionExpr::Client(
            ClientVec3::new(756.0, -40.0, 719.0),
        ))),
    ),
    at(
        6.15,
        2548,
        ChoreographyAction::Effect(EffectAction::AddBatch(BASIC_MOVE_EXPLOSIONS)),
    ),
    at(
        6.15,
        2565,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            856.0, -20.0, 719.0,
        )))),
    ),
    // The flyby freezes the currently rendered camera before placing
    // Buttercup two units in front of it.  The legacy assignments use the
    // live `TempPosition` and `kCTargetPosition`; restoring the earlier
    // explosion-pan constants here would move the render camera away from the
    // transform used to place her and leave the flyby outside the frame.
    at(
        8.15,
        2574,
        ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Entity(EntityRef::Camera))),
    ),
    at(
        8.15,
        2575,
        ChoreographyAction::Camera(CameraAction::FreezeCurrentTarget),
    ),
    at(
        8.15,
        2578,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 202,
            target: oriented(
                PositionAnchor::Entity(EntityRef::Camera),
                ClientVec3::new(0.0, -1.0, 2.0),
                OrientationBasis::Entity(EntityRef::Camera),
                ClientVec3::new(0.0, 0.0, 2.0),
            ),
        }),
    ),
    at(
        8.15,
        2580,
        ChoreographyAction::Sequence(FrameSequence::NpcDelta {
            id: 202,
            frames: 50,
            seconds_per_frame: 0.01,
            translation_per_frame: ClientVec3::new(0.0, 0.075, -0.15),
            rotation_each_frame: None,
        }),
    ),
    at(
        8.65,
        2588,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 204,
            target: PositionExpr::Client(ClientVec3::new(903.0, 11.0, 723.5)),
        }),
    ),
    at(
        8.65,
        2590,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 205,
            target: PositionExpr::Client(ClientVec3::new(903.0, 11.0, 723.0)),
        }),
    ),
    at(
        8.65,
        2594,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 202,
            target: PositionExpr::Client(ClientVec3::new(913.0, -13.5, 735.0)),
        }),
    ),
    at(
        8.65,
        2596,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: 202,
            rotation: RotationExpr::Euler(ClientVec3::ZERO),
        }),
    ),
    at(
        8.65,
        2598,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 200,
            target: PositionExpr::Client(ClientVec3::new(909.0, -14.0, 746.0)),
        }),
    ),
    at(
        8.65,
        2600,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: 200,
            rotation: RotationExpr::Euler(ClientVec3::new(0.0, 180.0, 0.0)),
        }),
    ),
    at(
        8.65,
        2602,
        ChoreographyAction::Npc(NpcAction::SetPosition {
            id: 201,
            target: PositionExpr::Client(ClientVec3::new(911.0, -14.0, 741.5)),
        }),
    ),
    at(
        8.65,
        2604,
        ChoreographyAction::Npc(NpcAction::SetRotation {
            id: 201,
            rotation: RotationExpr::Euler(ClientVec3::new(25.0, -45.0, 0.0)),
        }),
    ),
    at(
        8.65,
        2605,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Client(
            ClientVec3::new(910.0, -13.0, 744.5),
        ))),
    ),
    at(
        8.65,
        2609,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            910.0, -13.0, 744.5,
        )))),
    ),
    at(
        8.65,
        2606,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(0.0, 90.0, 0.0),
        ))),
    ),
    at(
        8.65,
        2607,
        ChoreographyAction::Camera(CameraAction::Distance(5.0)),
    ),
    at(
        8.65,
        2608,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Client(ClientVec3::new(910.0, -13.0, 744.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 90.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        8.65,
        2610,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(910.0, -13.0, 744.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(0.0, 90.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        8.65,
        2611,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 200,
            clip: "melee1",
            once: false,
        }),
    ),
    at(
        8.65,
        2612,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 201,
            clip: "melee1",
            once: false,
        }),
    ),
    at(
        8.65,
        2613,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 202,
            clip: "flying",
            once: false,
        }),
    ),
    at(
        9.15,
        2615,
        ChoreographyAction::Effect(EffectAction::ClearTracked),
    ),
    at(
        9.15,
        2616,
        ChoreographyAction::Sequence(FrameSequence::NpcDelta {
            id: 202,
            frames: 200,
            seconds_per_frame: 0.01,
            translation_per_frame: ClientVec3::new(0.0, 0.0, 0.22500001),
            rotation_each_frame: Some(RotationExpr::Euler(ClientVec3::new(0.0, 0.0, -90.0))),
        }),
    ),
    // Bubbles' ES736 is an AnimationClip event at melee1 t=0.9. The native
    // animation-event bridge owns it so asynchronous model loading cannot
    // move the effect away from the authored pose.
    at(
        11.65,
        2628,
        ChoreographyAction::Npc(NpcAction::DeleteInclusive {
            first: 200,
            last: 202,
        }),
    ),
    at(
        11.65,
        2631,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Client(
            ClientVec3::new(903.0, 11.5, 723.5),
        ))),
    ),
    at(
        11.65,
        2634,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            903.0, 11.5, 723.5,
        )))),
    ),
    at(
        11.65,
        2633,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(10.0, 45.0, 0.0),
        ))),
    ),
    at(
        11.65,
        2634,
        ChoreographyAction::Camera(CameraAction::Distance(3.0)),
    ),
    at(
        11.65,
        2637,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Client(ClientVec3::new(903.0, 11.5, 723.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(10.0, 45.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        11.65,
        2638,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(903.0, 11.5, 723.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(10.0, 45.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        11.65,
        2643,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 205,
            clip: "melee1",
            once: false,
        }),
    ),
    // Numbuh One's and Dexter's ES734 shots are emitted from their actual
    // repeating melee1 playback clocks (pathIds 892 and 1018 respectively).
    at(
        12.65,
        2642,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(10.0, 135.0, 0.0),
        ))),
    ),
    at(
        12.65,
        2646,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(903.0, 11.5, 723.5)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(10.0, 135.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        14.65,
        2645,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::Client(
            ClientVec3::new(883.0, -32.0, 693.0),
        ))),
    ),
    at(
        14.65,
        2649,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            883.0, -32.0, 693.0,
        )))),
    ),
    at(
        14.65,
        2649,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(-10.0, 194.0, 0.0),
        ))),
    ),
    at(
        14.65,
        2650,
        ChoreographyAction::Camera(CameraAction::Distance(4.0)),
    ),
    at(
        14.65,
        2652,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Client(ClientVec3::new(883.0, -32.0, 693.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(10.0, 194.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -4.0),
        ))),
    ),
    at(
        14.65,
        2653,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(883.0, -32.0, 693.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(10.0, 194.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -4.0),
        ))),
    ),
    at(
        14.65,
        2651,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 203,
            clip: "melee1event",
            once: false,
        }),
    ),
    // Samurai Jack's ES751 root slash and paired ES38 hand strikes are also
    // AnimationClip events. The bridge preserves their exact clip times and
    // the root-versus-hand rotation rule from AnimationEventHandler.cs.
    at(
        15.65,
        2657,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Client(ClientVec3::new(883.0, -32.0, 693.0)),
            ClientVec3::ZERO,
            OrientationBasis::Euler(ClientVec3::new(-10.0, 194.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -4.0),
        ))),
    ),
    at(
        17.65,
        2657,
        ChoreographyAction::Npc(NpcAction::AnimationInclusive {
            first: 314,
            last: 320,
            clip: "death",
            once: true,
        }),
    ),
    at(
        17.65,
        2661,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 203,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        18.65,
        2664,
        ChoreographyAction::Effect(EffectAction::AddAtNpcInclusive {
            effect_id: 372,
            first: 314,
            last: 320,
            scale: 2.0,
            tracked: true,
        }),
    ),
    at(
        18.65,
        2664,
        ChoreographyAction::Npc(NpcAction::DeleteInclusive {
            first: 314,
            last: 320,
        }),
    ),
    at(
        18.65,
        2674,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            330,
            2676,
            882.0,
            690.0,
            -34.0,
            Some(216),
            Some("standup"),
            false,
        ))),
    ),
    at(
        20.65,
        2678,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 203,
            clip: "turn",
            once: false,
        }),
    ),
    at(
        21.15,
        2680,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 203,
            command: NpcCommand::Pause,
        }),
    ),
    at(
        21.15,
        2681,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 330,
            command: NpcCommand::Pause,
        }),
    ),
    at(
        22.15,
        2683,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::StartPosition,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        22.15,
        2686,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::StartPosition,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        22.15,
        2687,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::StartPosition),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 270.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -7.0),
        ))),
    ),
    at(
        22.15,
        2688,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Entity(EntityRef::StartPosition),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 270.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -40.0),
        ))),
    ),
    at(
        22.15,
        2694,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 100,
            target: PositionExpr::EntityOffset {
                entity: EntityRef::StartPosition,
                offset: ClientVec3::new(1.0, 0.0, 1.0),
            },
            speed: 700,
        }),
    ),
    at(
        22.15,
        2699,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 101,
            target: PositionExpr::EntityOffset {
                entity: EntityRef::StartPosition,
                offset: ClientVec3::new(1.0, 0.0, -1.0),
            },
            speed: 700,
        }),
    ),
    at(
        23.15,
        2704,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 100,
            command: NpcCommand::ForceStop,
        }),
    ),
    at(
        23.15,
        2705,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::ForceStop,
        }),
    ),
    at(
        23.15,
        2702,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 100,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        23.15,
        2703,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        23.15,
        2708,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        23.15,
        2709,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "event1",
            once: false,
        }),
    ),
    at(
        23.15,
        2710,
        ChoreographyAction::Camera(CameraAction::CurrentTarget(PositionExpr::EntityOffset {
            entity: EntityRef::StartPosition,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        23.15,
        2711,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::StartPosition,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        23.15,
        2713,
        ChoreographyAction::Camera(CameraAction::StoredStart(oriented(
            PositionAnchor::Entity(EntityRef::StartPosition),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 90.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        23.15,
        2714,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::StartPosition),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 90.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -3.0),
        ))),
    ),
    at(
        23.15,
        2711,
        ChoreographyAction::Player(PlayerAction::Face(EntityRef::Npc(100))),
    ),
    at(
        25.15,
        2721,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "stand1",
            once: false,
        }),
    ),
    at(
        25.15,
        2722,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "talk",
            once: false,
        }),
    ),
    at(
        25.15,
        2724,
        ChoreographyAction::Player(PlayerAction::Face(EntityRef::Npc(101))),
    ),
    at(
        26.15,
        2726,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 270,
        }),
    ),
    at(
        26.15,
        2727,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "point",
            once: false,
        }),
    ),
    at(
        28.15,
        2731,
        ChoreographyAction::Player(PlayerAction::FacePosition(PositionExpr::EntityOffset {
            entity: EntityRef::StartPosition,
            offset: ClientVec3::new(10.0, 0.0, 0.0),
        })),
    ),
    at(
        28.15,
        2735,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 100,
            target: PositionExpr::EntityOffset {
                entity: EntityRef::StartPosition,
                offset: ClientVec3::new(30.0, 0.0, 1.0),
            },
            speed: 500,
        }),
    ),
    at(
        28.15,
        2736,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "run",
            once: false,
        }),
    ),
    at(
        28.15,
        2741,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 101,
            target: PositionExpr::EntityOffset {
                entity: EntityRef::StartPosition,
                offset: ClientVec3::new(30.0, 0.0, -1.0),
            },
            speed: 500,
        }),
    ),
    at(
        28.15,
        2742,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "run",
            once: false,
        }),
    ),
    at(
        29.15,
        2745,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "jump",
            once: false,
        }),
    ),
    at(
        30.15,
        2747,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "jump",
            once: false,
        }),
    ),
    at(30.15, 2748, ChoreographyAction::Cinematic(false)),
    at(30.15, 2749, ChoreographyAction::FadeEnabled(true)),
    at(
        30.15,
        2751,
        ChoreographyAction::Npc(NpcAction::WarpServer {
            id: 100,
            target: LegacyServerPosition::new(565.0, 666.0, -101.0),
        }),
    ),
    at(
        30.15,
        2752,
        ChoreographyAction::Npc(NpcAction::WarpServer {
            id: 101,
            target: LegacyServerPosition::new(567.0, 664.0, -101.0),
        }),
    ),
    at(
        30.15,
        2753,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: 230,
        }),
    ),
    at(
        30.15,
        2754,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 230,
        }),
    ),
    at(
        30.15,
        2755,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        30.15,
        2757,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_OVERLAY_OUT)),
    ),
    at(
        30.15,
        2756,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        31.15,
        2763,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(31.15, 2764, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        31.15,
        2765,
        ChoreographyAction::Npc(NpcAction::DeleteInclusive {
            first: 304,
            last: 330,
        }),
    ),
    at(
        31.15,
        2769,
        ChoreographyAction::Npc(NpcAction::DeleteInclusive {
            first: 200,
            last: 205,
        }),
    ),
    at(
        31.15,
        2773,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(31.15, 2774, ChoreographyAction::EventScene(false)),
];
