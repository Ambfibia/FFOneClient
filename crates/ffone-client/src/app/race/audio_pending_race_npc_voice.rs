
pub(in super::super) const RACE_START_AUDIO_PATH: &str = "audio/sfx/ui/racestart.ogg";

pub(in super::super) const RACE_FINISH_AUDIO_PATH: &str = "audio/sfx/ui/racefinish.ogg";

pub(in super::super) const RACE_RING_AUDIO_PATH: &str = "audio/sfx/ui/ring.ogg";

pub(in super::super) const RACE_BUTTON_SOUND_GAIN: f32 = 0.7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in super::super) enum PendingRaceNpcVoice {
    RaceFinished,
}
