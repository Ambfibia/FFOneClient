use super::*;

pub const RULE_UI_BUTTON_SOUND_PATHS: [&str; 5] = [
    "audio/sfx/ui/mouse_click01.ogg",
    "audio/sfx/ui/mouse_click02.ogg",
    "audio/sfx/ui/mouse_click03.ogg",
    "audio/sfx/ui/mouse_click04.ogg",
    "audio/sfx/ui/mouse_click05.ogg",
];

pub const RULE_UI_BUTTON_SOUND_GAIN: f32 = 0.7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleUiAudioCue {
    /// Exact `SoundUtil.ButtonSound()` request. The source utility selects a
    /// shared click clip; the passive surface does not invent that RNG state.
    ButtonSound,
}

impl RuleUiAudioCue {
    #[must_use]
    pub const fn candidate_paths(self) -> &'static [&'static str; 5] {
        match self {
            Self::ButtonSound => &RULE_UI_BUTTON_SOUND_PATHS,
        }
    }

    #[must_use]
    pub const fn gain(self) -> f32 {
        match self {
            Self::ButtonSound => RULE_UI_BUTTON_SOUND_GAIN,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct RuleUiAudioOutbox {
    pub(super) cues: VecDeque<RuleUiAudioCue>,
}

impl RuleUiAudioOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<RuleUiAudioCue> {
        self.cues.pop_front()
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }

    pub(super) fn push(&mut self, cue: RuleUiAudioCue) {
        self.cues.push_back(cue);
    }
}
