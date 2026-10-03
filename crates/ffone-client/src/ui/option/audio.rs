use super::*;

pub const OPTION_OPEN_SOUND_PATH: &str = "audio/sfx/ui/open_screen.ogg";

pub const OPTION_CLOSE_SOUND_PATH: &str = "audio/sfx/ui/close_screen.ogg";

pub const OPTION_SUCCESS_SOUND_PATH: &str = "audio/sfx/ui/action_sucess.ogg";

pub const OPTION_BUTTON_SOUND_PATHS: [&str; 5] = [
    "audio/sfx/ui/mouse_click01.ogg",
    "audio/sfx/ui/mouse_click02.ogg",
    "audio/sfx/ui/mouse_click03.ogg",
    "audio/sfx/ui/mouse_click04.ogg",
    "audio/sfx/ui/mouse_click05.ogg",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptionAudioContract {
    pub runtime_path: &'static str,
    pub bytes: u64,
    pub sha256: &'static str,
}

pub const OPTION_AUDIO_CONTRACTS: [OptionAudioContract; 8] = [
    OptionAudioContract {
        runtime_path: OPTION_OPEN_SOUND_PATH,
        bytes: 6_989,
        sha256: "0807111730f94bd08f21ecc8a3c38bdd1da5aed4ebf20da24155f492cd18721d",
    },
    OptionAudioContract {
        runtime_path: OPTION_CLOSE_SOUND_PATH,
        bytes: 5_516,
        sha256: "ededc59d6edb9d7b71df4adb0f3bee46f8057d9e470fe21e770a34868c7f5d1f",
    },
    OptionAudioContract {
        runtime_path: OPTION_SUCCESS_SOUND_PATH,
        bytes: 7_790,
        sha256: "b450c96d5ef80e4c6a55251d6d2ff68e7aef6e580295c5d0dd02a2af0f267f7a",
    },
    OptionAudioContract {
        runtime_path: OPTION_BUTTON_SOUND_PATHS[0],
        bytes: 3_997,
        sha256: "e2b000255b7b31c8e87756ccab6003d0f77631cfe292bbb466b79a68bf67588e",
    },
    OptionAudioContract {
        runtime_path: OPTION_BUTTON_SOUND_PATHS[1],
        bytes: 4_150,
        sha256: "d4fc4e38c9153907a9e49d8115d35c032a11834883e46aa8ef97bca4c3220ca2",
    },
    OptionAudioContract {
        runtime_path: OPTION_BUTTON_SOUND_PATHS[2],
        bytes: 4_175,
        sha256: "1df348d55d5ac6628e8a6658d3539d51fb6ab8508cfaf61cc945f830fde364c2",
    },
    OptionAudioContract {
        runtime_path: OPTION_BUTTON_SOUND_PATHS[3],
        bytes: 4_041,
        sha256: "fdd50f98423b81454e5c23125cf836f494f51ad7101773c385129a3d611e9db3",
    },
    OptionAudioContract {
        runtime_path: OPTION_BUTTON_SOUND_PATHS[4],
        bytes: 4_508,
        sha256: "bf3c9f15c5ac3540eb10a84e10182a04e2c478f32a0ecdc85c0a0cc24f5d80cb",
    },
];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct SoundChannelSettings {
    pub enabled: bool,
    pub volume: f32,
}

impl Default for SoundChannelSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.5,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SoundSettings {
    /// Clean `cnSoundOption.bMasterVolume` / `fMasterVolume`.
    pub master: SoundChannelSettings,
    pub music: SoundChannelSettings,
    pub effects: SoundChannelSettings,
    pub ambient: SoundChannelSettings,
    pub voice: SoundChannelSettings,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OptionSoundChannel {
    Master,
    Effects,
    Voice,
    Ambient,
    Music,
}

impl OptionSoundChannel {
    pub const ALL: [Self; 5] = [
        Self::Master,
        Self::Effects,
        Self::Voice,
        Self::Ambient,
        Self::Music,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Master => "MASTER VOLUME",
            Self::Effects => "SOUND EFFECT VOLUME",
            Self::Voice => "VOICE VOLUME",
            Self::Ambient => "AMBIENT VOLUME",
            Self::Music => "MUSIC VOLUME",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OptionOpenAudioRoute {
    pub main_game_transition: bool,
    pub inventory_transition: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OptionCloseAudioRoute {
    pub main_game_transition: bool,
    pub inventory_transition: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionUiAudioCue {
    OpenScreen,
    CloseScreen,
    ActionSuccess,
    ButtonClick { clip_index: u8, gain: f32 },
}

impl OptionUiAudioCue {
    #[must_use]
    pub fn path(self) -> &'static str {
        match self {
            Self::OpenScreen => OPTION_OPEN_SOUND_PATH,
            Self::CloseScreen => OPTION_CLOSE_SOUND_PATH,
            Self::ActionSuccess => OPTION_SUCCESS_SOUND_PATH,
            Self::ButtonClick { clip_index, .. } => {
                OPTION_BUTTON_SOUND_PATHS[usize::from(clip_index).min(4)]
            }
        }
    }

    #[must_use]
    pub const fn gain(self) -> f32 {
        match self {
            Self::ButtonClick { gain, .. } => gain,
            Self::OpenScreen | Self::CloseScreen | Self::ActionSuccess => 0.7,
        }
    }
}

pub(super) fn sound_channel_mut(
    sound: &mut SoundSettings,
    channel: OptionSoundChannel,
) -> &mut SoundChannelSettings {
    match channel {
        OptionSoundChannel::Master => &mut sound.master,
        OptionSoundChannel::Effects => &mut sound.effects,
        OptionSoundChannel::Voice => &mut sound.voice,
        OptionSoundChannel::Ambient => &mut sound.ambient,
        OptionSoundChannel::Music => &mut sound.music,
    }
}

pub(super) fn sound_channel(sound: &SoundSettings, channel: OptionSoundChannel) -> SoundChannelSettings {
    match channel {
        OptionSoundChannel::Master => sound.master,
        OptionSoundChannel::Effects => sound.effects,
        OptionSoundChannel::Voice => sound.voice,
        OptionSoundChannel::Ambient => sound.ambient,
        OptionSoundChannel::Music => sound.music,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct OptionUiAudioRouting {
    pub close: OptionCloseAudioRoute,
}

impl Default for OptionUiAudioRouting {
    fn default() -> Self {
        Self {
            close: OptionCloseAudioRoute {
                main_game_transition: true,
                inventory_transition: false,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSoundDefaultsButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSoundToggleButton {
    pub channel: OptionSoundChannel,
    pub value: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSoundSliderButton {
    pub channel: OptionSoundChannel,
    pub step: u8,
}
