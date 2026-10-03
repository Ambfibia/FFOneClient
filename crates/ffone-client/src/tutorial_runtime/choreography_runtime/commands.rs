use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoreographyActionOrigin {
    Timeline,
    SkipSceneSpecific,
    SkipCommonCleanup,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChoreographyPlaybackEvent {
    Action {
        source_line: u32,
        origin: ChoreographyActionOrigin,
        action: ChoreographyAction,
    },
    SequenceSample {
        source_line: u32,
        sample: FrameSequenceSample,
    },
    BlockingWaitReached {
        wait: BlockingWait,
    },
    FinalSkipFade {
        fade: SkipFinalFade,
    },
    Finished {
        scene: TutorialScene,
        completion: ChoreographyCompletion,
    },
}

#[derive(Debug)]
pub(super) struct ScheduledEvent {
    pub(super) at_seconds: f32,
    pub(super) same_time_order: u64,
    pub(super) event: ChoreographyPlaybackEvent,
}

pub(super) fn unavailable_action_issue(
    source_line: u32,
    action: ChoreographyAction,
) -> Option<TutorialChoreographyIssue> {
    match action {
        ChoreographyAction::Unresolved(action) => {
            Some(TutorialChoreographyIssue::UnresolvedReferenceAction {
                source_line,
                detail: action.detail,
            })
        }
        // Every typed action now has a live adapter. Adapter-specific failures
        // are reported where the command is actually resolved, so speculative
        // timeline warnings would only mask the actionable issue.
        _ => None,
    }
}
