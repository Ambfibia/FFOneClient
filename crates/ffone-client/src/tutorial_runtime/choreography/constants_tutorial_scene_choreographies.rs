use super::*;

pub(super) const NANO_POWER_B_WAITS: &[BlockingWait] = &[
    BlockingWait::EffectInstantiationRetry {
        reached_at_seconds: 13.25,
        source_line: 4703,
        continuation_source_line: 4714,
        effect_id: 741,
        instance_name: "portal effect 741",
        retry_source_line: 4711,
        retry_action: ChoreographyAction::Effect(EffectAction::Add(effect(
            741, 987.1, 111.3, 546.8, 1.0, false,
        ))),
        retry_interval_seconds: 0.1,
        maximum_attempts: 10,
        maximum_wait_seconds: 1.0,
        shared_attempt_counter: "num",
    },
    BlockingWait::EffectInstantiationRetry {
        // 0.2 + 100*0.01 + 100*0.01 + 0.2 + 1.0 seconds elapse
        // after the ES741 loop before the ES739 loop is reached.
        reached_at_seconds: 16.65,
        source_line: 4834,
        continuation_source_line: 4844,
        effect_id: 739,
        instance_name: "Dexter hologram effect 739",
        retry_source_line: 4841,
        retry_action: ChoreographyAction::Effect(EffectAction::Add(effect(
            739, 987.1, 111.3, 546.8, 1.0, false,
        ))),
        retry_interval_seconds: 0.1,
        maximum_attempts: 10,
        maximum_wait_seconds: 1.0,
        shared_attempt_counter: "num",
    },
];

pub(super) const COMMON_SKIP_CLEANUP: &[SourcedAction] = &[
    skip(5232, ChoreographyAction::Effect(EffectAction::ClearTracked)),
    skip(5233, ChoreographyAction::VoiceOff),
    skip(5234, ChoreographyAction::Hud(HudAction::PopShow)),
    skip(5235, ChoreographyAction::StopBgm),
    skip(
        5236,
        ChoreographyAction::SpatialAudio(SpatialAudioTarget::Player),
    ),
    skip(5237, ChoreographyAction::EventScene(false)),
    skip(5238, ChoreographyAction::Player(PlayerAction::StandForce)),
    skip(
        5239,
        ChoreographyAction::Camera(CameraAction::Mode(CameraMode::None)),
    ),
    skip(5240, ChoreographyAction::FadeEnabled(false)),
    skip(5241, ChoreographyAction::Cinematic(false)),
];

pub(super) const BASIC_MOVE_SKIP_SPAWNS: &[NpcSpawn] = &[
    spawn(100, 2669, 541.0, 657.0, -104.0, None, None, false),
    spawn(101, 2670, 541.0, 653.0, -104.0, None, None, false),
];

pub(super) const BASIC_MOVE_SKIP: &[SourcedAction] = &[
    skip(
        5058,
        ChoreographyAction::Npc(NpcAction::SpawnBatch(BASIC_MOVE_SKIP_SPAWNS)),
    ),
    skip(
        5060,
        ChoreographyAction::Npc(NpcAction::WarpServer {
            id: 100,
            target: LegacyServerPosition::new(565.0, 666.0, -101.0),
        }),
    ),
    skip(
        5061,
        ChoreographyAction::Npc(NpcAction::WarpServer {
            id: 101,
            target: LegacyServerPosition::new(567.0, 664.0, -101.0),
        }),
    ),
    skip(
        5062,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: 230,
        }),
    ),
    skip(
        5063,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 230,
        }),
    ),
    skip(
        5064,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "ready",
            once: false,
        }),
    ),
    skip(
        5065,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "ready",
            once: false,
        }),
    ),
    skip(
        5066,
        ChoreographyAction::Npc(NpcAction::DeleteInclusive {
            first: 314,
            last: 320,
        }),
    ),
    skip(
        5070,
        ChoreographyAction::Npc(NpcAction::DeleteInclusive {
            first: 304,
            last: 330,
        }),
    ),
    skip(
        5074,
        ChoreographyAction::Npc(NpcAction::DeleteInclusive {
            first: 200,
            last: 205,
        }),
    ),
    skip(5078, ChoreographyAction::SubtitleClear),
];

pub(super) const COMBAT_A_SKIP_SPAWNS: &[NpcSpawn] = &[
    spawn(1001, 2674, 569.0, 675.0, -101.0, Some(20), None, false),
    spawn(1002, 2674, 570.0, 672.0, -101.0, Some(20), None, false),
    spawn(1003, 2674, 571.0, 672.0, -101.0, Some(20), None, false),
    spawn(1004, 2897, 564.0, 680.0, -101.0, Some(20), None, false),
];

pub(super) const BASIC_COMBAT_A_SKIP: &[SourcedAction] = &[
    skip(
        5084,
        ChoreographyAction::Equipment(EquipmentAction::TutorialWeaponByLocaleAndClass),
    ),
    skip(
        5101,
        ChoreographyAction::Npc(NpcAction::SpawnBatch(COMBAT_A_SKIP_SPAWNS)),
    ),
    skip(
        5114,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 101,
            target: PositionExpr::Client(ClientVec3::new(565.6, -101.0, 679.8)),
        }),
    ),
    skip(
        5118,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 100,
            target: PositionExpr::Client(ClientVec3::new(563.8, -101.0, 674.9)),
        }),
    ),
    skip(
        5119,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "melee1",
            once: false,
        }),
    ),
    skip(
        5120,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "melee1event",
            once: false,
        }),
    ),
    skip(
        5121,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: 180,
        }),
    ),
    skip(
        5122,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 90,
        }),
    ),
    skip(
        5123,
        ChoreographyAction::Camera(CameraAction::LookAt(PositionExpr::Client(ClientVec3::new(
            575.6, -98.0, 668.2,
        )))),
    ),
];

pub(super) const BASIC_COMBAT_B_SKIP: &[SourcedAction] = &[
    skip(5131, ChoreographyAction::Npc(NpcAction::Delete(1004))),
    skip(
        5134,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 101,
            target: PositionExpr::Client(ClientVec3::new(569.3, -100.0, 659.1)),
        }),
    ),
    skip(
        5138,
        ChoreographyAction::Npc(NpcAction::WarpClient {
            id: 100,
            target: PositionExpr::Client(ClientVec3::new(564.4, -101.0, 662.3)),
        }),
    ),
    skip(
        5139,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 100,
            clip: "melee1",
            once: false,
        }),
    ),
    skip(
        5140,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 100,
            helper_degrees: -25,
        }),
    ),
    skip(
        5141,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 101,
            helper_degrees: 60,
        }),
    ),
    skip(
        5142,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 101,
            clip: "melee1event",
            once: false,
        }),
    ),
    skip(
        5143,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            1005,
            2675,
            567.0,
            658.0,
            -101.0,
            Some(197),
            None,
            false,
        ))),
    ),
];

pub(super) const BASIC_COMBAT_C_SKIP: &[SourcedAction] = &[
    skip(5149, ChoreographyAction::Pan(PanAction::Stop)),
    skip(5150, ChoreographyAction::Npc(NpcAction::Delete(1005))),
    skip(5151, ChoreographyAction::Npc(NpcAction::Delete(100))),
    skip(5152, ChoreographyAction::Npc(NpcAction::Delete(101))),
    skip(
        5155,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            566.0, -101.0, 665.0,
        )))),
    ),
    skip(
        5157,
        ChoreographyAction::Progress(ProgressAction::InitChapter {
            chapter: 2,
            step: 0,
        }),
    ),
    skip(
        5159,
        ChoreographyAction::Pan(PanAction::ReleaseEightTextures),
    ),
];

pub(super) const INFECTION_A_SKIP: &[SourcedAction] =
    &[skip(5166, ChoreographyAction::Npc(NpcAction::Delete(1012)))];

pub(super) const INFECTION_B_SKIP: &[SourcedAction] = &[
    skip(5171, ChoreographyAction::Npc(NpcAction::Delete(3000))),
    skip(
        5172,
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
    skip(
        5176,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            3000,
            2902,
            562.0,
            967.0,
            -133.0,
            None,
            Some("melee1event"),
            false,
        ))),
    ),
    skip(
        5177,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 3000,
            command: NpcCommand::SetAngleTo(EntityRef::Npc(4000)),
        }),
    ),
    skip(
        5179,
        ChoreographyAction::Npc(NpcAction::Command {
            id: 4000,
            command: NpcCommand::SetAngleTo(EntityRef::Npc(3000)),
        }),
    ),
    skip(
        5180,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 4000,
            clip: "melee1",
            once: false,
        }),
    ),
    skip(
        5183,
        ChoreographyAction::Player(PlayerAction::Warp(PositionExpr::Client(ClientVec3::new(
            588.4, -133.5, 983.8,
        )))),
    ),
];

pub(super) const INFECTION_C_SKIP: &[SourcedAction] = &[
    skip(
        5187,
        ChoreographyAction::Nano(NanoAction::Equip {
            nano_id: 1,
            skill_id: 1,
            slot: 0,
            stamina: 100,
        }),
    ),
    skip(
        5188,
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
    skip(5191, ChoreographyAction::Npc(NpcAction::Delete(3000))),
    skip(5192, ChoreographyAction::Npc(NpcAction::Delete(4000))),
    skip(
        5195,
        ChoreographyAction::Nano(NanoAction::DestroyPresentationObject),
    ),
    skip(
        5198,
        ChoreographyAction::Loop(LoopAction::StartIfAbsent("LairCollapse_Quake_LOOP")),
    ),
];

pub(super) const NANO_POWER_A_SKIP: &[SourcedAction] = &[
    skip(
        5203,
        ChoreographyAction::Npc(NpcAction::Angle {
            id: 5100,
            helper_degrees: 180,
        }),
    ),
    skip(
        5204,
        ChoreographyAction::Npc(NpcAction::Animation {
            id: 5100,
            clip: "worry",
            once: false,
        }),
    ),
    skip(5205, ChoreographyAction::Npc(NpcAction::Delete(2))),
    skip(5208, ChoreographyAction::Loop(LoopAction::StopIfPresent)),
];

pub(super) const NANO_POWER_A2_SKIP: &[SourcedAction] = &[
    skip(
        5213,
        ChoreographyAction::Npc(NpcAction::Spawn(spawn(
            5000,
            2677,
            907.0,
            710.0,
            12.0,
            Some(183),
            None,
            false,
        ))),
    ),
    skip(5216, ChoreographyAction::Player(PlayerAction::Show)),
];

pub(super) const NANO_POWER_B_SKIP: &[SourcedAction] = &[
    skip(5221, ChoreographyAction::Loop(LoopAction::StopIfPresent)),
    skip(5223, ChoreographyAction::FadeEnabled(true)),
    skip(
        5224,
        ChoreographyAction::Sequence(FrameSequence::Fade(FadeSequence {
            channel: FadeChannel::Overlay,
            from: 1.0,
            to: 1.0,
            steps: 1,
            divisor: 1.0,
            seconds_per_step: 0.0,
        })),
    ),
    skip(5225, ChoreographyAction::Npc(NpcAction::Delete(5100))),
    skip(5226, ChoreographyAction::Npc(NpcAction::Delete(5101))),
];

pub(super) const INFECTION_C_LOOPS: &[LoopLifetime] = &[LoopLifetime {
    cue: "LairCollapse_Quake_LOOP",
    starts_at_seconds: 33.4,
    natural_stop_at_seconds: None,
    persists_after_scene: true,
    skip_action: LoopAction::StartIfAbsent("LairCollapse_Quake_LOOP"),
}];

pub(super) const NANO_POWER_A_LOOPS: &[LoopLifetime] = &[LoopLifetime {
    cue: "RumbleQuake_LOOP",
    starts_at_seconds: 0.0,
    natural_stop_at_seconds: Some(13.5),
    persists_after_scene: false,
    skip_action: LoopAction::StopIfPresent,
}];

pub(super) const NANO_POWER_B_LOOPS: &[LoopLifetime] = &[LoopLifetime {
    cue: "DexterHologram_LOOP",
    starts_at_seconds: 16.65,
    natural_stop_at_seconds: Some(40.65),
    persists_after_scene: false,
    skip_action: LoopAction::StopIfPresent,
}];

pub(super) const NO_WAITS: &[BlockingWait] = &[];

pub(super) const NO_LOOPS: &[LoopLifetime] = &[];

pub const TUTORIAL_SCENE_CHOREOGRAPHIES: &[TutorialSceneChoreography] = &[
    TutorialSceneChoreography {
        scene: TutorialScene::BasicMove,
        coroutine: "BasicMoveEvent",
        source: SourceSpan {
            first_line: 2449,
            last_line: 2775,
        },
        deterministic_duration_seconds: 31.15,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::BasicMove,
        actions: BASIC_MOVE_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: NO_LOOPS,
        skip: skip_contract(BASIC_MOVE_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::BasicCombatA,
        coroutine: "BasicCombatEvent_A",
        source: SourceSpan {
            first_line: 2995,
            last_line: 3142,
        },
        deterministic_duration_seconds: 12.0,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::BasicCombatA,
        actions: BASIC_COMBAT_A_ACTIONS,
        blocking_waits: BASIC_COMBAT_A_WAITS,
        loops: NO_LOOPS,
        skip: skip_contract(BASIC_COMBAT_A_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::BasicCombatB,
        coroutine: "BasicCombatEvent_B",
        source: SourceSpan {
            first_line: 3144,
            last_line: 3338,
        },
        deterministic_duration_seconds: 39.6,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::BasicCombatB,
        actions: BASIC_COMBAT_B_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: NO_LOOPS,
        skip: skip_contract(BASIC_COMBAT_B_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::BasicCombatC,
        coroutine: "BasicCombatEvent_C",
        source: SourceSpan {
            first_line: 3340,
            last_line: 3458,
        },
        deterministic_duration_seconds: 45.0,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::BasicCombatC,
        actions: BASIC_COMBAT_C_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: NO_LOOPS,
        skip: skip_contract(BASIC_COMBAT_C_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::InfectionA,
        coroutine: "Infection_Event_A",
        source: SourceSpan {
            first_line: 3881,
            last_line: 3922,
        },
        deterministic_duration_seconds: 15.0,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::InfectionA,
        actions: INFECTION_A_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: NO_LOOPS,
        skip: skip_contract(INFECTION_A_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::InfectionB,
        coroutine: "Infection_Event_B",
        source: SourceSpan {
            first_line: 3924,
            last_line: 4010,
        },
        deterministic_duration_seconds: 33.1,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::InfectionB,
        actions: INFECTION_B_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: NO_LOOPS,
        skip: skip_contract(INFECTION_B_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::InfectionC,
        coroutine: "Infection_Event_C",
        source: SourceSpan {
            first_line: 4012,
            last_line: 4200,
        },
        deterministic_duration_seconds: 47.9,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::InfectionC,
        actions: INFECTION_C_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: INFECTION_C_LOOPS,
        skip: skip_contract(INFECTION_C_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::NanoPowerA,
        coroutine: "NanoPower_Event_A",
        source: SourceSpan {
            first_line: 4456,
            last_line: 4572,
        },
        deterministic_duration_seconds: 14.5,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::NanoPowerA,
        actions: NANO_POWER_A_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: NANO_POWER_A_LOOPS,
        skip: skip_contract(NANO_POWER_A_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::NanoPowerA2,
        coroutine: "NanoPower_Event_A2",
        source: SourceSpan {
            first_line: 4574,
            last_line: 4622,
        },
        deterministic_duration_seconds: 5.0,
        maximum_duration_seconds: None,
        presenter_audio_scene: TutorialScene::NanoPowerA2,
        actions: NANO_POWER_A2_ACTIONS,
        blocking_waits: NO_WAITS,
        loops: NO_LOOPS,
        skip: skip_contract(NANO_POWER_A2_SKIP, SkipFinalFade::GameEventFadeIn),
    },
    TutorialSceneChoreography {
        scene: TutorialScene::NanoPowerB,
        coroutine: "NanoPower_Event_B",
        source: SourceSpan {
            first_line: 4624,
            last_line: 4892,
        },
        deterministic_duration_seconds: 46.15,
        maximum_duration_seconds: Some(47.15),
        presenter_audio_scene: TutorialScene::NanoPowerB,
        actions: NANO_POWER_B_ACTIONS,
        blocking_waits: NANO_POWER_B_WAITS,
        loops: NANO_POWER_B_LOOPS,
        skip: skip_contract(NANO_POWER_B_SKIP, SkipFinalFade::OpaqueOverlay),
    },
];
