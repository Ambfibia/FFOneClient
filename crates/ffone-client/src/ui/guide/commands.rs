use super::*;

pub const GUIDE_HELP_EVENT_RECEIVER: i32 = 44;

pub const GUIDE_HELP_EVENT_GROUP: i32 = 2;

pub const GUIDE_HELP_EVENT_FUNCTION: i32 = 21;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideUiCommand {
    AcceptWarpWarning,
    SelectMentor(GuideMentor),
    OpenConfirmation,
    ConfirmMentor,
    CancelConfirmation,
    Dismiss(GuideUiDismissalSource),
    RequestHelp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideUiAction {
    WarpWarningAccepted,
    Dismissed {
        source: GuideUiDismissalSource,
    },
    HelpRequested {
        receiver: i32,
        event_group: i32,
        event_function: i32,
    },
    CurrentMentorRejected {
        mentor: GuideMentor,
        system_message_id: i32,
    },
    ChangeMentorRequested {
        mentor: GuideMentor,
    },
    FirstMentorChangeSucceeded {
        mentor: GuideMentor,
        mentor_count: i16,
        fusion_matter: i32,
        next_mode: i32,
        warp_npc_table_id: i32,
    },
    MentorChangeSucceeded {
        mentor: GuideMentor,
        mentor_count: i16,
        fusion_matter: i32,
        refresh_guide_missions: bool,
    },
    MentorChangeFailed {
        mentor: GuideMentor,
        error_code: i32,
        system_message_id: i32,
    },
}

pub fn apply_guide_ui_command(
    model: &mut GuideUiModel,
    actions: &mut GuideUiOutbox,
    audio: &mut GuideUiAudioOutbox,
    command: GuideUiCommand,
) -> bool {
    match command {
        GuideUiCommand::AcceptWarpWarning => {
            if !model.controls_enabled() || model.phase != GuideUiPhase::WarpWarning {
                return false;
            }
            model.phase = GuideUiPhase::MentorSelection;
            audio.push(GuideUiAudioCue::YesButton);
            actions.push(GuideUiAction::WarpWarningAccepted);
            true
        }
        GuideUiCommand::SelectMentor(mentor) => {
            if !model.controls_enabled()
                || model.phase != GuideUiPhase::MentorSelection
                || model.confirmation_open
            {
                return false;
            }
            if model.selected == Some(mentor) {
                return false;
            }
            model.selected = Some(mentor);
            audio.push(GuideUiAudioCue::ButtonSound);
            true
        }
        GuideUiCommand::OpenConfirmation => {
            if model.primary_blocker().is_some() || model.phase != GuideUiPhase::MentorSelection {
                return false;
            }
            model.confirmation_open = true;
            audio.push(GuideUiAudioCue::YesButton);
            true
        }
        GuideUiCommand::CancelConfirmation => {
            if !model.controls_enabled() || !model.confirmation_open {
                return false;
            }
            model.confirmation_open = false;
            audio.push(GuideUiAudioCue::NoButton);
            true
        }
        GuideUiCommand::ConfirmMentor => {
            if !model.controls_enabled() || !model.confirmation_open {
                return false;
            }
            let Some(mentor) = model.selected else {
                return false;
            };
            model.confirmation_open = false;
            audio.push(GuideUiAudioCue::YesButton);
            if model.current == Some(mentor) {
                actions.push(GuideUiAction::CurrentMentorRejected {
                    mentor,
                    system_message_id: GUIDE_ALREADY_CURRENT_MESSAGE_ID,
                });
                return true;
            }
            model.awaiting_server = true;
            model.pending_mentor = Some(mentor);
            actions.push(GuideUiAction::ChangeMentorRequested { mentor });
            true
        }
        GuideUiCommand::Dismiss(source) => {
            let escape_allowed = source != GuideUiDismissalSource::EscapeKey
                || model.input_boundary().escape_dismiss_enabled;
            let source_matches_phase = match source {
                GuideUiDismissalSource::WarpCancelButton => {
                    model.phase == GuideUiPhase::WarpWarning
                }
                GuideUiDismissalSource::CancelButton | GuideUiDismissalSource::CloseButton => {
                    model.phase == GuideUiPhase::MentorSelection && !model.confirmation_open
                }
                GuideUiDismissalSource::EscapeKey => true,
            };
            if !model.visible || model.awaiting_server || !escape_allowed || !source_matches_phase {
                return false;
            }
            if source != GuideUiDismissalSource::EscapeKey && !model.controls_enabled() {
                return false;
            }
            match source {
                GuideUiDismissalSource::CloseButton => {
                    audio.push(GuideUiAudioCue::ButtonSound);
                }
                GuideUiDismissalSource::WarpCancelButton | GuideUiDismissalSource::CancelButton => {
                    audio.push(GuideUiAudioCue::NoButton);
                }
                GuideUiDismissalSource::EscapeKey => {}
            }
            actions.push(GuideUiAction::Dismissed { source });
            model.close();
            true
        }
        GuideUiCommand::RequestHelp => {
            if !model.controls_enabled()
                || model.phase != GuideUiPhase::MentorSelection
                || model.confirmation_open
            {
                return false;
            }
            audio.push(GuideUiAudioCue::ButtonSound);
            actions.push(GuideUiAction::HelpRequested {
                receiver: GUIDE_HELP_EVENT_RECEIVER,
                event_group: GUIDE_HELP_EVENT_GROUP,
                event_function: GUIDE_HELP_EVENT_FUNCTION,
            });
            true
        }
    }
}
