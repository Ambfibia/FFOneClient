#[cfg(test)]
use super::*;

/// Tutorial `NanoTableElement` / `SkillTableElement` values recovered from the
/// Retrobution client data.
pub const TUTORIAL_BUTTERCUP_NANO_ID: i16 = 1;

pub const TUTORIAL_BUTTERCUP_SKILL_ID: i16 = 1;

pub const TUTORIAL_BUTTERCUP_INITIAL_STAMINA: i32 = 100;

pub const TUTORIAL_BUTTERCUP_SKILL_DRAIN: i32 = 20;

pub const TUTORIAL_BUTTERCUP_SKILL_STAMINA_FLOOR: i32 = 50;

/// `SkillTableElement.m_iCoolTime == 80`; the clean client stores tenths of a
/// second and assigns `m_iCoolTime * 0.1` to the equipped-slot channel.
pub const TUTORIAL_BUTTERCUP_SKILL_COOLDOWN_SECONDS: f32 = 8.0;

pub const TUTORIAL_BUTTERCUP_SKILL_CONDITION: i32 = 512;

/// `SkillBuffElement[10].m_iBuffEffect`, projected by clean
/// `Status.UpdateSkillBuff` while the stun condition bit is set.
pub const TUTORIAL_BUTTERCUP_STUN_EFFECT_ID: i32 = 366;

pub const TUTORIAL_BUTTERCUP_STUN_EFFECT_NODE: &str = "state";

pub const TUTORIAL_BUTTERCUP_SKILL_RANGE: f32 = 10.0;

pub const TUTORIAL_BUTTERCUP_SKILL_HALF_ANGLE_DEGREES: f32 = 90.0;

pub const TUTORIAL_BUTTERCUP_SKILL_TARGET_CAPACITY: usize = 2;

pub const TUTORIAL_BUTTERCUP_SUMMON_EFFECT_ID: i32 = 528;

pub const TUTORIAL_NANO_DISMISS_EFFECT_ID: i32 = 10;

pub const TUTORIAL_NANO_CALL_CROSS_FADE_SECONDS: f32 = 0.1;

pub const TUTORIAL_NANO_FOLLOW_RATE: f32 = 3.0;

pub const TUTORIAL_NANO_PATTERN_SECONDS: f32 = 10.0;

pub const TUTORIAL_BUTTERCUP_SKILL_EFFECT_ID: i32 = 60;

pub const TUTORIAL_BUTTERCUP_SKILL_EFFECT_NODE: &str = "Bip01 Footsteps";

pub(super) const CHEESE_NANO_ID: i16 = 51;

#[cfg(test)]
pub(super) const GAMEPLAY_REQUIRED_CLIPS: [&str; 5] = [CALL_CLIP, "stand1", "stand2", "stand3", SKILL_CLIP];
