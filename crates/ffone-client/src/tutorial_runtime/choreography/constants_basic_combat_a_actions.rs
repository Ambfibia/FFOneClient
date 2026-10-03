use super::*;

pub(super) const COMBAT_A_SPAWNS: &[NpcSpawn] = &[
    spawn(1001, 2674, 573.0, 677.0, -101.0, None, None, false),
    spawn(1002, 2674, 573.0, 676.0, -101.0, None, None, false),
    spawn(1003, 2674, 574.0, 677.0, -101.0, None, None, false),
    spawn(1004, 2897, 574.0, 677.0, -101.0, None, None, false),
];

pub(super) const BASIC_COMBAT_A_ACTIONS: &[TimedAction] = &[
    at(
        0.0,
        2997,
        ChoreographyAction::Effect(EffectAction::Preload(742)),
    ),
    at(0.0, 2998, ChoreographyAction::Hud(HudAction::PushHide)),
    at(
        0.0,
        3001,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::None),
    ),
    at(0.0, 3002, ChoreographyAction::EventScene(true)),
    at(
        0.0,
        3003,
        ChoreographyAction::Player(PlayerAction::StandForce),
    ),
    at(
        0.0,
        3004,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::EventCameraControl)),
    ),
    at(
        0.0,
        3005,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            573.0, -100.0, 677.0,
        )))),
    ),
    at(
        0.0,
        3006,
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(10.0, 60.0, 0.0),
        ))),
    ),
    at(
        0.0,
        3007,
        ChoreographyAction::Camera(CameraAction::Distance(12.0)),
    ),
    at(
        0.0,
        3009,
        ChoreographyAction::Npc(NpcAction::SpawnBatch(COMBAT_A_SPAWNS)),
    ),
    at(
        0.0,
        3015,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 1001,
            target: PositionExpr::Client(ClientVec3::new(569.0, -101.0, 675.0)),
            speed: 300,
        }),
    ),
    at(
        0.0,
        3020,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 1002,
            target: PositionExpr::Client(ClientVec3::new(570.0, -101.0, 672.0)),
            speed: 300,
        }),
    ),
    at(
        0.0,
        3025,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 1003,
            target: PositionExpr::Client(ClientVec3::new(571.0, -101.0, 672.0)),
            speed: 300,
        }),
    ),
    at(
        0.0,
        3030,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 1004,
            target: PositionExpr::Client(ClientVec3::new(564.0, -101.0, 680.0)),
            speed: 300,
        }),
    ),
    at(
        0.0,
        3033,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: 230,
        }),
    ),
    at(
        0.0,
        3034,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 230,
        }),
    ),
    at(
        0.0,
        3036,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::SetAngleTo(EntityRef::Npc(1004)),
        }),
    ),
    at(0.0, 3037, ChoreographyAction::Cinematic(true)),
    at(
        0.0,
        3038,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_IN)),
    ),
    at(
        2.0,
        3046,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            566.0, -100.0, 665.0,
        )))),
    ),
    at(
        2.0,
        3050,
        ChoreographyAction::Player(PlayerAction::FaceAngle(50)),
    ),
    at(
        3.0,
        3052,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "point",
            once: false,
        }),
    ),
    at(
        3.0,
        3053,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        3.0,
        3054,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::FromToCamera)),
    ),
    at(
        3.0,
        3055,
        ChoreographyAction::Camera(CameraAction::StoredStart(PositionExpr::Entity(
            EntityRef::Camera,
        ))),
    ),
    at(
        3.0,
        3056,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::EntityOffset {
            entity: EntityRef::Player,
            offset: ClientVec3::UP,
        })),
    ),
    at(
        3.0,
        3057,
        ChoreographyAction::Camera(CameraAction::Start(oriented(
            PositionAnchor::Entity(EntityRef::Player),
            ClientVec3::UP,
            OrientationBasis::Euler(ClientVec3::new(10.0, 30.0, 0.0)),
            ClientVec3::new(0.0, 0.0, -5.0),
        ))),
    ),
    at(
        6.0,
        3060,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "ready",
            once: false,
        }),
    ),
    at(
        6.0,
        3062,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 100,
            command: NpcCommand::SetAngleTo(EntityRef::Player),
        }),
    ),
    at(
        6.0,
        3063,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 1001,
            helper_degrees: 20,
        }),
    ),
    at(
        6.0,
        3064,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 1002,
            helper_degrees: 20,
        }),
    ),
    at(
        6.0,
        3065,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 1003,
            helper_degrees: 20,
        }),
    ),
    at(
        6.0,
        3066,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 1004,
            helper_degrees: 20,
        }),
    ),
    // The reference calls `PreloadEffect(742)` a second time after all four
    // angles, then blocks on that returned AssetBundleRequest before the
    // hand-off animation and particle at line 3070.
    at(
        6.0,
        3062,
        ChoreographyAction::Effect(EffectAction::Preload(742)),
    ),
    at(
        6.0,
        3070,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "pass",
            once: false,
        }),
    ),
    at(
        6.0,
        3071,
        ChoreographyAction::Effect(EffectAction::AttachBone(EffectBoneAttachment {
            effect_id: 742,
            actor_id: 100,
            exact_node_name: "Bip01 L Hand",
            spawn_world_rotation: RotationExpr::Euler(ClientVec3::new(90.0, 0.0, 0.0)),
            local_rotation_after_parenting: RotationExpr::Euler(ClientVec3::new(90.0, 0.0, 0.0)),
            scale: 1.0,
            tracked: false,
            name: "Numbuh Five handoff particle 742",
            locale_gate: EffectLocaleGate::LegacyLocalEquals(0),
            destroy_after_seconds: 1.3,
        })),
    ),
    at(
        7.3,
        3080,
        ChoreographyAction::Equipment(EquipmentAction::TutorialWeaponByLocaleAndClass),
    ),
    at(
        8.0,
        3100,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: 230,
        }),
    ),
    at(
        8.0,
        3102,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 250,
        }),
    ),
    at(
        8.0,
        3107,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 101,
            target: PositionExpr::Client(ClientVec3::new(565.6, -101.0, 679.8)),
            speed: 600,
        }),
    ),
    at(
        8.0,
        3112,
        ChoreographyAction::Npc(NpcAction::Move {
            id: 100,
            target: PositionExpr::Client(ClientVec3::new(563.8, -101.0, 674.9)),
            speed: 600,
        }),
    ),
    at(
        8.0,
        3114,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "run",
            once: false,
        }),
    ),
    at(
        8.0,
        3114,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "run",
            once: false,
        }),
    ),
    at(
        9.5,
        3117,
        ChoreographyAction::Camera(CameraAction::Target(PositionExpr::Client(ClientVec3::new(
            567.3, -98.5, 676.9,
        )))),
    ),
    at(
        10.0,
        3120,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 100,
            command: NpcCommand::ForceStop,
        }),
    ),
    at(
        10.0,
        3121,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "melee1",
            once: false,
        }),
    ),
    at(
        10.0,
        3122,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: 180,
        }),
    ),
    // Numbuh Five's ES734 tag01 shot is emitted by the native animation-event
    // bridge at melee1 t=0.233333 (primary pathId 1117).
    at(
        11.0,
        3124,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 101,
            command: NpcCommand::ForceStop,
        }),
    ),
    at(
        11.0,
        3125,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "melee1event",
            once: false,
        }),
    ),
    at(
        11.0,
        3126,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 90,
        }),
    ),
    at(
        11.0,
        3127,
        ChoreographyAction::Sequence(FrameSequence::Fade(FADE_SCREEN_OUT)),
    ),
    at(
        12.0,
        3133,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    at(
        12.0,
        3134,
        ChoreographyAction::Camera(CameraAction::LookAt(PositionExpr::Client(ClientVec3::new(
            567.0, -98.0, 678.0,
        )))),
    ),
    at(12.0, 3135, ChoreographyAction::Cinematic(false)),
    at(12.0, 3137, ChoreographyAction::Hud(HudAction::PopShow)),
    at(
        12.0,
        3138,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    at(12.0, 3139, ChoreographyAction::EventScene(false)),
];

pub(super) const BASIC_COMBAT_A_WAITS: &[BlockingWait] = &[BlockingWait::AssetPreload {
    reached_at_seconds: 6.0,
    source_line: 3065,
    continuation_source_line: 3070,
    effect_id: 742,
}];
