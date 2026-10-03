use super::*;

pub const UPSELL_ACTION_SUCCESS_SOUND_PATH: &str = "audio/sfx/ui/action_sucess.ogg";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpsellUiAudioCue {
    /// Exact source call to randomized `SoundUtil.ButtonSound`.
    ButtonSound,
    /// Exact source call `SoundUtil.Playsound("Action_Sucess")`.
    ActionSuccess,
}

impl UpsellUiAudioCue {
    #[must_use]
    pub const fn exact_path(self) -> Option<&'static str> {
        match self {
            // The callsite proves the randomized family owner, not one
            // deterministic member; consumers must not substitute one clip.
            Self::ButtonSound => None,
            Self::ActionSuccess => Some(UPSELL_ACTION_SUCCESS_SOUND_PATH),
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct UpsellUiAudioOutbox {
    pub(super) cues: VecDeque<UpsellUiAudioCue>,
}

impl UpsellUiAudioOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<UpsellUiAudioCue> {
        self.cues.pop_front()
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }

    pub(super) fn push(&mut self, cue: UpsellUiAudioCue) {
        self.cues.push_back(cue);
    }
}
