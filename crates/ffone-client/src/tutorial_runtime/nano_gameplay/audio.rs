use super::*;

pub const TUTORIAL_BUTTERCUP_SUMMON_SFX_TRUE_NAME: &str = "Nano Ability 06";

pub const TUTORIAL_BUTTERCUP_SUMMON_VOICE_TRUE_NAMES: [&str; 3] = [
    "Btrcup_NanSummon01_01",
    "Btrcup_NanSummon01_02",
    "Btrcup_NanSummon01_03",
];

pub const TUTORIAL_BUTTERCUP_SKILL_SFX_TRUE_NAME: &str = "Stun_Root_GENERIC";

pub const TUTORIAL_BUTTERCUP_SKILL_VOICE_TRUE_NAMES: [&str; 3] = [
    "Btrcup_NanPwrMF01_01",
    "Btrcup_NanPwrMF01_02",
    "Btrcup_NanPwrMF01_03",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialNanoGameplayAudioCategory {
    Sfx,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Component)]
pub(super) struct WorldNanoAnimationSoundCursor {
    pub(super) generation: u64,
    pub(super) request_serial: u64,
    pub(super) clip: &'static str,
    pub(super) node: AnimationNodeIndex,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}
