use super::*;

pub(super) const MOVE_HINT_POSITION: TutorialScreenPoint = TutorialScreenPoint::new(
    TutorialScreenAxis::Pixels(80),
    TutorialScreenAxis::HeightMinus(330),
);

pub(super) const HALF_QUARTER_PIVOT: TutorialScreenPivot =
    TutorialScreenPivot::Point(TutorialScreenPoint::new(
        TutorialScreenAxis::WidthFractionMinus {
            numerator: 1,
            denominator: 2,
            subtract: 0,
        },
        TutorialScreenAxis::HeightFractionMinus {
            numerator: 1,
            denominator: 4,
            subtract: 0,
        },
    ));

pub(super) const MISSION_MENU_PIVOT: TutorialScreenPivot =
    TutorialScreenPivot::Point(TutorialScreenPoint::new(
        TutorialScreenAxis::WidthFractionMinus {
            numerator: 3,
            denominator: 4,
            subtract: 0,
        },
        TutorialScreenAxis::HeightFractionMinus {
            numerator: 1,
            denominator: 2,
            subtract: 0,
        },
    ));

pub(super) const TALK_CURSOR_POSITION: TutorialScreenPoint = TutorialScreenPoint::new(
    TutorialScreenAxis::WidthFractionMinus {
        numerator: 1,
        denominator: 2,
        subtract: 100,
    },
    TutorialScreenAxis::HeightFractionMinus {
        numerator: 1,
        denominator: 4,
        subtract: 0,
    },
);

pub(super) const MISSION_CURSOR_POSITION: TutorialScreenPoint = TutorialScreenPoint::new(
    TutorialScreenAxis::WidthFractionMinus {
        numerator: 3,
        denominator: 4,
        subtract: 246,
    },
    TutorialScreenAxis::HeightFractionMinus {
        numerator: 1,
        denominator: 2,
        subtract: 41,
    },
);

pub(super) const BASIC_ARROW_KEY_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut07",
            subtitle: Some(TutorialText::SceneText { group: 2, index: 5 }),
            legacy_duration_seconds: Some(7.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::ShowPicture {
            position: MOVE_HINT_POSITION,
            resource: "tut_move",
            pivot: TutorialScreenPivot::Legacy(3),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Primary,
            text: TutorialText::Localized("Press and hold the \"W\" key to move forward."),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        6.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut08",
            subtitle: Some(TutorialText::SceneText { group: 2, index: 6 }),
            legacy_duration_seconds: Some(5.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        6.0,
        TutorialAuxiliaryAction::ShowPicture {
            position: MOVE_HINT_POSITION,
            resource: "tut_move_w",
            pivot: TutorialScreenPivot::Legacy(3),
        },
    ),
];

pub(super) const MINIMAP_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::PreloadEffect { effect_id: 668 },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::SendMessageBox {
            npc_type: 2671,
            message: TutorialText::Literal(
                "I need some help with a sooper important mission! Report to me right away!",
            ),
            message_type: 9,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "NumTwo_Tut01",
            subtitle: None,
            legacy_duration_seconds: None,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::SpawnNpc {
            legacy_server_position: [651.0, 739.0, -82.0],
            npc_type: 2671,
            runtime_id: 1005,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        1.0,
        TutorialAuxiliaryAction::SpawnEffectRelativeToActor {
            runtime_id: 1005,
            offset: [0.0, -1.0, 0.0],
            effect_id: 668,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        1.0,
        TutorialAuxiliaryAction::SetNpcAngle {
            runtime_id: 1005,
            legacy_degrees: 68,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        1.0,
        TutorialAuxiliaryAction::SetNpcAnimation {
            runtime_id: 1005,
            animation: "think",
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        4.0,
        TutorialAuxiliaryAction::ShowCursor {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::WidthMinus(280),
                TutorialScreenAxis::Pixels(50),
            ),
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: TutorialScreenPivot::Legacy(0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        4.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut19",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 34,
            }),
            legacy_duration_seconds: Some(5.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        10.0,
        TutorialAuxiliaryAction::SetFlag {
            flag: TutorialBooleanFlag::MyPointEvent,
            value: true,
        },
    ),
    TimedTutorialAuxiliaryAction::new(10.0, TutorialAuxiliaryAction::HidePicture),
    TimedTutorialAuxiliaryAction::new(
        10.0,
        TutorialAuxiliaryAction::ShowCursor {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::WidthMinus(200),
                TutorialScreenAxis::Pixels(50),
            ),
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: TutorialScreenPivot::Legacy(0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        10.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut20",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 35,
            }),
            legacy_duration_seconds: Some(3.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        13.0,
        TutorialAuxiliaryAction::SetFlag {
            flag: TutorialBooleanFlag::MyPointEvent,
            value: false,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        13.0,
        TutorialAuxiliaryAction::SetFlag {
            flag: TutorialBooleanFlag::WayPointEvent,
            value: true,
        },
    ),
    TimedTutorialAuxiliaryAction::new(13.0, TutorialAuxiliaryAction::HidePicture),
    TimedTutorialAuxiliaryAction::new(
        13.0,
        TutorialAuxiliaryAction::ShowCursor {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::WidthMinus(140),
                TutorialScreenAxis::Pixels(0),
            ),
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: TutorialScreenPivot::Legacy(0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        13.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut21",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 36,
            }),
            legacy_duration_seconds: Some(3.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        13.0,
        TutorialAuxiliaryAction::SetWaypoint {
            target: TutorialWaypointTarget::ActorPosition { runtime_id: 1005 },
            enabled: true,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        16.0,
        TutorialAuxiliaryAction::SetFlag {
            flag: TutorialBooleanFlag::WayPointEvent,
            value: false,
        },
    ),
];

pub(super) const GO_TO_NUMBUH_TWO_REMINDER: &[TutorialAuxiliaryAction] = &[
    TutorialAuxiliaryAction::Voice {
        clip: "Computress_Tut22",
        subtitle: Some(TutorialText::SceneText {
            group: 2,
            index: 37,
        }),
        legacy_duration_seconds: Some(10.0),
    },
    TutorialAuxiliaryAction::HidePicture,
    TutorialAuxiliaryAction::Subtitle {
        channel: TutorialSubtitleChannel::Primary,
        text: TutorialText::Localized("Go to Numbuh Two."),
    },
];

pub(super) const TALK_NUM_TWO_1_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    // Chapter_05 clears the standalone MinimapEvent marker before starting
    // TalkNumTwo_1. Keep this at time zero so the task waypoint cannot overlap
    // the old "go to Numbuh Two" effect while the opening voice line plays.
    TimedTutorialAuxiliaryAction::new(0.0, TutorialAuxiliaryAction::ClearEffects),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut25",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 39,
            }),
            legacy_duration_seconds: Some(10.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::ShowCursor {
            position: TALK_CURSOR_POSITION,
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: HALF_QUARTER_PIVOT,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        4.5,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut26",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 40,
            }),
            legacy_duration_seconds: Some(10.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        4.5,
        TutorialAuxiliaryAction::SetWaypoint {
            target: TutorialWaypointTarget::DisabledAtZero,
            enabled: false,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        4.5,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Secondary,
            text: TutorialText::Localized("Select \"Transmitter Critters\" from mission menu."),
        },
    ),
    TimedTutorialAuxiliaryAction::new(4.5, TutorialAuxiliaryAction::HidePicture),
    TimedTutorialAuxiliaryAction::new(
        4.5,
        TutorialAuxiliaryAction::ShowCursor {
            position: MISSION_CURSOR_POSITION,
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: MISSION_MENU_PIVOT,
        },
    ),
];

pub(super) const TALK_NUM_TWO_2_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut38",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 41,
            }),
            legacy_duration_seconds: Some(10.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::ShowCursor {
            position: TALK_CURSOR_POSITION,
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: HALF_QUARTER_PIVOT,
        },
    ),
    TimedTutorialAuxiliaryAction::new(4.5, TutorialAuxiliaryAction::HidePicture),
    TimedTutorialAuxiliaryAction::new(
        4.5,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Secondary,
            text: TutorialText::Localized("Select \"Transmitter Critters\" from mission menu."),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        4.5,
        TutorialAuxiliaryAction::ShowCursor {
            position: MISSION_CURSOR_POSITION,
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: MISSION_MENU_PIVOT,
        },
    ),
];

pub(super) const TALK_BUTTERCUP_1_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Btrcup_Tut03",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 43,
            }),
            legacy_duration_seconds: Some(3.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        5.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut41",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 44,
            }),
            legacy_duration_seconds: Some(10.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        5.0,
        TutorialAuxiliaryAction::ShowCursor {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::WidthMinus(280),
                TutorialScreenAxis::Pixels(50),
            ),
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: TutorialScreenPivot::Legacy(0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(27.0, TutorialAuxiliaryAction::SetFusionMatter { value: 0 }),
    TimedTutorialAuxiliaryAction::new(27.0, TutorialAuxiliaryAction::HidePicture),
    TimedTutorialAuxiliaryAction::new(
        27.0,
        TutorialAuxiliaryAction::ShowCursor {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::WidthMinus(259),
                TutorialScreenAxis::Pixels(200),
            ),
            resource: "tut_right",
            direction: TutorialCursorDirection::Right,
            pivot: MISSION_MENU_PIVOT,
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        27.0,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Primary,
            text: TutorialText::Localized("Talk to Buttercup."),
        },
    ),
];

pub(super) const TALK_DEXTER_1_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Dexter_Tut01",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 69,
            }),
            legacy_duration_seconds: Some(5.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        5.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut49",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 70,
            }),
            legacy_duration_seconds: Some(10.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        5.0,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Primary,
            text: TutorialText::Localized("Go to the Fusion Portal."),
        },
    ),
];

pub(super) const TALK_DEXTER_2_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Dexter_Tut02",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 71,
            }),
            legacy_duration_seconds: Some(5.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        5.0,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Primary,
            text: TutorialText::Localized("Talk to Dexter."),
        },
    ),
];

pub(super) const TALK_DEXTER_3_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut56",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 72,
            }),
            legacy_duration_seconds: Some(3.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Primary,
            text: TutorialText::Literal("Use the warp gate to exit."),
        },
    ),
    TimedTutorialAuxiliaryAction::new(3.0, TutorialAuxiliaryAction::ClearSubtitle),
    TimedTutorialAuxiliaryAction::new(
        3.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut57",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 73,
            }),
            legacy_duration_seconds: Some(5.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        3.0,
        TutorialAuxiliaryAction::Subtitle {
            channel: TutorialSubtitleChannel::Primary,
            text: TutorialText::SceneText {
                group: 2,
                index: 74,
            },
        },
    ),
];

pub(super) const NANO_POWER_TALK_ACTIONS: &[TimedTutorialAuxiliaryAction] = &[
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::Voice {
            clip: "Computress_Tut60",
            subtitle: Some(TutorialText::SceneText {
                group: 2,
                index: 88,
            }),
            legacy_duration_seconds: Some(8.0),
        },
    ),
    TimedTutorialAuxiliaryAction::new(
        0.0,
        TutorialAuxiliaryAction::ShowPicture {
            position: TutorialScreenPoint::new(
                TutorialScreenAxis::Pixels(80),
                TutorialScreenAxis::HeightMinus(350),
            ),
            resource: "tut_rmouse",
            pivot: TutorialScreenPivot::Legacy(3),
        },
    ),
];

pub(super) const STUN_MONSTER_REMINDER: &[TutorialAuxiliaryAction] = &[
    TutorialAuxiliaryAction::Voice {
        clip: "Computress_Tut61",
        subtitle: Some(TutorialText::SceneText {
            group: 2,
            index: 89,
        }),
        legacy_duration_seconds: Some(10.0),
    },
    TutorialAuxiliaryAction::Subtitle {
        channel: TutorialSubtitleChannel::Primary,
        text: TutorialText::Localized("Stun the monster."),
    },
];

pub const TUTORIAL_AUXILIARY_DEFINITIONS: &[TutorialAuxiliaryDefinition] = &[
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::BasicArrowKey,
        source: TutorialSourceSpan::new(2769, 2778),
        actions: BASIC_ARROW_KEY_ACTIONS,
        completes_at_seconds: Some(6.0),
        repeating: None,
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::MinimapEvent,
        source: TutorialSourceSpan::new(3541, 3576),
        actions: MINIMAP_ACTIONS,
        completes_at_seconds: None,
        repeating: Some(TutorialRepeatBlock {
            starts_at_seconds: 16.0,
            every_seconds: 25.0,
            actions: GO_TO_NUMBUH_TWO_REMINDER,
        }),
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::TalkNumTwo1,
        source: TutorialSourceSpan::new(3608, 3626),
        actions: TALK_NUM_TWO_1_ACTIONS,
        completes_at_seconds: Some(4.5),
        repeating: None,
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::TalkNumTwo2,
        source: TutorialSourceSpan::new(3628, 3643),
        actions: TALK_NUM_TWO_2_ACTIONS,
        completes_at_seconds: Some(4.5),
        repeating: None,
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::TalkButtercup1,
        source: TutorialSourceSpan::new(3645, 3657),
        actions: TALK_BUTTERCUP_1_ACTIONS,
        completes_at_seconds: Some(27.0),
        repeating: None,
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::TalkDexter1,
        source: TutorialSourceSpan::new(4202, 4208),
        actions: TALK_DEXTER_1_ACTIONS,
        completes_at_seconds: Some(5.0),
        repeating: None,
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::TalkDexter2,
        source: TutorialSourceSpan::new(4210, 4215),
        actions: TALK_DEXTER_2_ACTIONS,
        completes_at_seconds: Some(5.0),
        repeating: None,
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::TalkDexter3,
        source: TutorialSourceSpan::new(4217, 4225),
        actions: TALK_DEXTER_3_ACTIONS,
        completes_at_seconds: Some(3.0),
        repeating: None,
    },
    TutorialAuxiliaryDefinition {
        sequence: TutorialAuxiliarySequence::NanoPowerTalk1,
        source: TutorialSourceSpan::new(4894, 4905),
        actions: NANO_POWER_TALK_ACTIONS,
        completes_at_seconds: None,
        repeating: Some(TutorialRepeatBlock {
            starts_at_seconds: 8.0,
            every_seconds: 25.0,
            actions: STUN_MONSTER_REMINDER,
        }),
    },
];

pub const TUTORIAL_INITIALIZATION: TutorialInitializationContract =
    TutorialInitializationContract {
        source: TutorialSourceSpan::new(845, 947),
        required_web_bundles: &["Tutorial.resourceFile", "NpcTexture.resourceFile"],
        force_view_menu: false,
        ambient_loop: "TutorialMain_wAmbient_Loop",
        initial_music: "none",
        new_nano_id: 1,
        new_nano_offscreen_position: [-1000.0, 0.0, -3000.0],
        dome_route: "Mob/etc_domeglass_04.kfm",
        dome_offscreen_position: [-1000.0, 0.0, -3000.0],
        initial_npc: TutorialInitialNpc {
            legacy_server_position: [973, 702, -76],
            npc_type: 2968,
            runtime_id: 2,
            legacy_angle_degrees: 88,
        },
        production_locks_all_input: true,
        production_fade_alpha: 1.0,
    };
