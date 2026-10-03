use super::*;

/// Exact behavior of `FadeInNewAudio` at lines 425-502.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialAudioCrossfadeContract {
    pub source: TutorialSourceSpan,
    pub steps_per_fade: u8,
    pub seconds_per_step: f32,
    pub none_matches_by_substring: bool,
    pub skip_when_current_clip_name_matches: bool,
    pub load_before_fading_current_clip: bool,
    pub clear_clip_after_none_fade: bool,
}

pub const TUTORIAL_AUDIO_CROSSFADE: TutorialAudioCrossfadeContract =
    TutorialAudioCrossfadeContract {
        source: TutorialSourceSpan::new(425, 502),
        steps_per_fade: 10,
        seconds_per_step: 0.05,
        none_matches_by_substring: true,
        skip_when_current_clip_name_matches: true,
        load_before_fading_current_clip: true,
        clear_clip_after_none_fade: true,
    };
