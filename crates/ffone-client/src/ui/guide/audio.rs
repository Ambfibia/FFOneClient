use super::*;

pub const GUIDE_YES_SOUND_PATH: &str = "audio/sfx/ui/yes_button.ogg";

pub const GUIDE_NO_SOUND_PATH: &str = "audio/sfx/ui/no_button.ogg";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideUiAudioCue {
    YesButton,
    NoButton,
    ButtonSound,
}

impl GuideUiAudioCue {
    #[must_use]
    pub const fn exact_path(self) -> Option<&'static str> {
        match self {
            Self::YesButton => Some(GUIDE_YES_SOUND_PATH),
            Self::NoButton => Some(GUIDE_NO_SOUND_PATH),
            // `SoundUtil.ButtonSound` chooses among the shared click family.
            // The owning mixer must preserve that selection policy.
            Self::ButtonSound => None,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct GuideUiAudioOutbox {
    pub(super) cues: VecDeque<GuideUiAudioCue>,
}

impl GuideUiAudioOutbox {
    pub fn pop_front(&mut self) -> Option<GuideUiAudioCue> {
        self.cues.pop_front()
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }

    pub(super) fn push(&mut self, cue: GuideUiAudioCue) {
        self.cues.push_back(cue);
    }
}
