use super::*;

/// Exact clean `cnSystemMessageManager.DrawAll` button audio routes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemMessageUiAudioCue {
    YesButton,
    NoButton,
    AbandonMission,
}

impl SystemMessageUiAudioCue {
    #[must_use]
    pub const fn true_name(self) -> &'static str {
        match self {
            Self::YesButton => "Yes_Button",
            Self::NoButton => "No_Button",
            Self::AbandonMission => "Abandon_Mission",
        }
    }
}

#[must_use]
pub const fn system_message_audio_cue(
    button_type: SystemMessageButtonType,
    choice: SystemMessageChoice,
) -> SystemMessageUiAudioCue {
    match (button_type, choice) {
        (SystemMessageButtonType::DeleteMission, SystemMessageChoice::Primary) => {
            SystemMessageUiAudioCue::AbandonMission
        }
        (_, SystemMessageChoice::Primary) => SystemMessageUiAudioCue::YesButton,
        (_, SystemMessageChoice::Secondary) => SystemMessageUiAudioCue::NoButton,
    }
}

#[derive(Default, Resource)]
pub struct SystemMessageUiAudioOutbox {
    pub(super) cues: VecDeque<SystemMessageUiAudioCue>,
}

impl SystemMessageUiAudioOutbox {
    pub fn push(&mut self, cue: SystemMessageUiAudioCue) {
        self.cues.push_back(cue);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = SystemMessageUiAudioCue> + '_ {
        self.cues.drain(..)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }
}
