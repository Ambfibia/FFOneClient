use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TutorialAuxiliaryAction {
    PreloadEffect {
        effect_id: i32,
    },
    SendMessageBox {
        npc_type: i32,
        message: TutorialText,
        message_type: i32,
    },
    Voice {
        clip: &'static str,
        subtitle: Option<TutorialText>,
        legacy_duration_seconds: Option<f32>,
    },
    ShowPicture {
        position: TutorialScreenPoint,
        resource: &'static str,
        pivot: TutorialScreenPivot,
    },
    ShowCursor {
        position: TutorialScreenPoint,
        resource: &'static str,
        direction: TutorialCursorDirection,
        pivot: TutorialScreenPivot,
    },
    HidePicture,
    Subtitle {
        channel: TutorialSubtitleChannel,
        text: TutorialText,
    },
    ClearSubtitle,
    ClearEffects,
    SpawnNpc {
        legacy_server_position: [f32; 3],
        npc_type: i32,
        runtime_id: i32,
    },
    SpawnEffectRelativeToActor {
        runtime_id: i32,
        offset: [f32; 3],
        effect_id: i32,
    },
    SetNpcAngle {
        runtime_id: i32,
        legacy_degrees: i32,
    },
    SetNpcAnimation {
        runtime_id: i32,
        animation: &'static str,
    },
    SetFlag {
        flag: TutorialBooleanFlag,
        value: bool,
    },
    SetWaypoint {
        target: TutorialWaypointTarget,
        enabled: bool,
    },
    SetFusionMatter {
        value: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedTutorialAuxiliaryAction {
    pub at_seconds: f32,
    pub action: TutorialAuxiliaryAction,
}

impl TimedTutorialAuxiliaryAction {
    pub const fn new(at_seconds: f32, action: TutorialAuxiliaryAction) -> Self {
        Self { at_seconds, action }
    }
}
