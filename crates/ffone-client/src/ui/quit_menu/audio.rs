use super::*;

pub const QUIT_MENU_OPEN_SOUND_PATH: &str = "audio/sfx/ui/open_screen.ogg";

pub const QUIT_MENU_CLOSE_SOUND_PATH: &str = "audio/sfx/ui/close_screen.ogg";

pub const QUIT_MENU_BUTTON_SOUND_PATHS: [&str; 5] = [
    "audio/sfx/ui/mouse_click01.ogg",
    "audio/sfx/ui/mouse_click02.ogg",
    "audio/sfx/ui/mouse_click03.ogg",
    "audio/sfx/ui/mouse_click04.ogg",
    "audio/sfx/ui/mouse_click05.ogg",
];

pub const QUIT_MENU_BUTTON_SOUND_GAIN: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QuitMenuAudioCue {
    OpenScreen,
    CloseScreen,
    ButtonClick { clip_index: u8, gain: f32 },
}

impl QuitMenuAudioCue {
    #[must_use]
    pub fn path(self) -> &'static str {
        match self {
            Self::OpenScreen => QUIT_MENU_OPEN_SOUND_PATH,
            Self::CloseScreen => QUIT_MENU_CLOSE_SOUND_PATH,
            Self::ButtonClick { clip_index, .. } => {
                QUIT_MENU_BUTTON_SOUND_PATHS[usize::from(clip_index).min(4)]
            }
        }
    }

    #[must_use]
    pub const fn gain(self) -> f32 {
        match self {
            Self::OpenScreen | Self::CloseScreen => 0.7,
            Self::ButtonClick { gain, .. } => gain,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct QuitMenuAudioOutbox {
    pub(super) cues: VecDeque<QuitMenuAudioCue>,
}

impl QuitMenuAudioOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<QuitMenuAudioCue> {
        self.cues.pop_front()
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }

    pub(super) fn push(&mut self, cue: QuitMenuAudioCue) {
        self.cues.push_back(cue);
    }
}

/// Deterministic native replacement for Unity's `Random.Range(0, 5)`.
///
/// A fixed default seed keeps reducer and GPU tests stable. Production callers
/// may replace the seed without changing the exact five-clip/gain contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct QuitMenuClickSoundSequence {
    pub(super) state: u32,
}

impl Default for QuitMenuClickSoundSequence {
    fn default() -> Self {
        Self::seeded(0x51F0_0E11)
    }
}

impl QuitMenuClickSoundSequence {
    #[must_use]
    pub const fn seeded(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0xA341_316C } else { seed },
        }
    }

    pub fn next_clip_index(&mut self) -> u8 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        self.state = value;
        (value % QUIT_MENU_BUTTON_SOUND_PATHS.len() as u32) as u8
    }
}
