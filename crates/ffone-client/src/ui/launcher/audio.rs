
pub const LAUNCHER_UI_CLICK_ON_AUDIO_PATH: &str = "audio/sfx/ui/launcher_clickon.ogg";

pub const LAUNCHER_UI_START_POWER_AUDIO_PATH: &str = "audio/sfx/ui/launcher_startpower.ogg";

pub const LAUNCHER_UI_POWER_PULSE_AUDIO_PATH: &str = "audio/sfx/ui/launcher_powerpulse.ogg";

pub const LAUNCHER_UI_STOP_POWER_AUDIO_PATH: &str = "audio/sfx/ui/launcher_stoppower.ogg";

pub const LAUNCHER_UI_FIRING_AUDIO_PATH: &str = "audio/sfx/ui/launcher_firing.ogg";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LauncherUiAudioCue {
    ClickOn,
    StartPower,
    PowerPulse,
    StopPower,
    Firing,
}

impl LauncherUiAudioCue {
    #[must_use]
    pub const fn legacy_name(self) -> &'static str {
        match self {
            Self::ClickOn => "Launcher_ClickOn",
            Self::StartPower => "Launcher_StartPower",
            Self::PowerPulse => "Launcher_PowerPulse",
            Self::StopPower => "Launcher_StopPower",
            Self::Firing => "Launcher_Firing",
        }
    }

    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::ClickOn => LAUNCHER_UI_CLICK_ON_AUDIO_PATH,
            Self::StartPower => LAUNCHER_UI_START_POWER_AUDIO_PATH,
            Self::PowerPulse => LAUNCHER_UI_POWER_PULSE_AUDIO_PATH,
            Self::StopPower => LAUNCHER_UI_STOP_POWER_AUDIO_PATH,
            Self::Firing => LAUNCHER_UI_FIRING_AUDIO_PATH,
        }
    }
}
