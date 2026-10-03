//! Central Retrobution animation-state contracts for NPCs, mobs, and Nanos.
//!
//! The legacy client has one `NpcAnimation` owner for friendly NPCs and mobs,
//! and one `NanoAnimation` state machine for every Nano model.  Presentation
//! code may still use different scene roots, but it must consume the contracts
//! in this module instead of inventing its own repeat/fade/completion rules.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::prelude::Resource;

use crate::movement::advance_native_xorshift32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyAnimationBlend {
    Immediate,
    CrossFade100Ms,
    CrossFade200Ms,
    CrossFade300Ms,
}

impl LegacyAnimationBlend {
    #[must_use]
    pub const fn duration(self) -> Duration {
        match self {
            Self::Immediate => Duration::ZERO,
            Self::CrossFade100Ms => Duration::from_millis(100),
            Self::CrossFade200Ms => Duration::from_millis(200),
            Self::CrossFade300Ms => Duration::from_millis(300),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyAnimationRepeat {
    Forever,
    Once,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyNpcAnimationRole {
    Stand,
    Locomotion,
    Ready,
    Forced,
    Melee,
    Wound,
    Death,
}

impl LegacyNpcAnimationRole {
    #[must_use]
    pub const fn is_additive(self) -> bool {
        matches!(self, Self::Melee | Self::Wound)
    }

    #[must_use]
    pub const fn repeat(self, forced_once: bool) -> LegacyAnimationRepeat {
        match self {
            Self::Stand | Self::Locomotion | Self::Ready => LegacyAnimationRepeat::Forever,
            Self::Forced if !forced_once => LegacyAnimationRepeat::Forever,
            Self::Forced | Self::Melee | Self::Wound | Self::Death => LegacyAnimationRepeat::Once,
        }
    }

    /// `NpcAnimation.EndAnimation` replays a clamped `ForceAnimationOnce`
    /// clip instead of leaving its last frame on screen. Death is the only
    /// forced clip which remains clamped.
    #[must_use]
    pub const fn restarts_after_completion(self, clip: &str, forced_once: bool) -> bool {
        matches!(self, Self::Forced) && forced_once && !is_death_clip(clip)
    }
}

#[must_use]
pub const fn npc_role_for_forced_clip(clip: &str) -> LegacyNpcAnimationRole {
    if is_death_clip(clip) {
        LegacyNpcAnimationRole::Death
    } else {
        LegacyNpcAnimationRole::Forced
    }
}

#[must_use]
pub const fn is_death_clip(clip: &str) -> bool {
    // Unity checks `IndexOf("death", 0) >= 0` when configuring forced clips.
    let bytes = clip.as_bytes();
    let needle = b"death";
    if bytes.len() < needle.len() {
        return false;
    }
    let mut index = 0;
    while index + needle.len() <= bytes.len() {
        let mut offset = 0;
        while offset < needle.len() && bytes[index + offset] == needle[offset] {
            offset += 1;
        }
        if offset == needle.len() {
            return true;
        }
        index += 1;
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyNanoAnimationMode {
    None,
    Stand,
    Call,
    Discharge,
    Casting,
    Happy,
    Sad,
    Win,
    Lose,
    Tie,
    Withdraw,
    Skill1,
    Skill2,
    Skill3,
    Emote,
    SkillSpecial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyNanoCompletion {
    Continue,
    Despawn,
    SkillSpecialFinished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyNanoAnimationMachine {
    mode: LegacyNanoAnimationMode,
    clip: Option<String>,
    blend: LegacyAnimationBlend,
    request_serial: u64,
    passive_skill: Option<LegacyNanoAnimationMode>,
    low_stamina: bool,
    force_stand_sound: bool,
}

impl Default for LegacyNanoAnimationMachine {
    fn default() -> Self {
        Self {
            mode: LegacyNanoAnimationMode::None,
            clip: None,
            blend: LegacyAnimationBlend::Immediate,
            request_serial: 0,
            passive_skill: None,
            low_stamina: false,
            force_stand_sound: false,
        }
    }
}

impl LegacyNanoAnimationMachine {
    #[must_use]
    pub const fn mode(&self) -> LegacyNanoAnimationMode {
        self.mode
    }

    #[must_use]
    pub fn clip(&self) -> Option<&str> {
        self.clip.as_deref()
    }

    #[must_use]
    pub const fn blend(&self) -> LegacyAnimationBlend {
        self.blend
    }

    #[must_use]
    pub const fn request_serial(&self) -> u64 {
        self.request_serial
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn request(
        &mut self,
        mode: LegacyNanoAnimationMode,
        clip: impl Into<String>,
        blend: LegacyAnimationBlend,
    ) {
        self.mode = mode;
        self.clip = Some(clip.into());
        self.blend = blend;
        self.request_serial = self.request_serial.wrapping_add(1).max(1);
    }

    pub fn request_stand(&mut self, random: &mut LegacyNanoStandRandomStream) {
        self.force_stand_sound = self.low_stamina && self.clip() != Some("discharge");
        let clip = if self.low_stamina {
            "discharge"
        } else {
            random.next_stand_clip()
        };
        self.request(
            LegacyNanoAnimationMode::Stand,
            clip,
            LegacyAnimationBlend::CrossFade100Ms,
        );
    }

    pub fn set_low_stamina(&mut self, low_stamina: bool) {
        self.low_stamina = low_stamina;
    }

    pub const fn force_stand_sound(&self) -> bool {
        self.force_stand_sound
    }

    pub fn allows_sound(&self, payload: &str, random: &mut LegacyNanoStandRandomStream) -> bool {
        self.mode != LegacyNanoAnimationMode::Stand
            || self.force_stand_sound
            || random.next_index(20) == 1
            || payload.contains("_SFX_Dance")
    }

    pub fn set_passive_skill(&mut self, mode: Option<LegacyNanoAnimationMode>) {
        self.passive_skill = mode.filter(|mode| {
            matches!(
                mode,
                LegacyNanoAnimationMode::Skill1
                    | LegacyNanoAnimationMode::Skill2
                    | LegacyNanoAnimationMode::Skill3
            )
        });
    }

    /// Applies the exact `NanoAnimation.EndAnimation` continuation for the
    /// currently accepted request. Every clip is played as a finite pass so
    /// this callback can choose the next state, just as Unity's embedded end
    /// event did even though most source `AnimationState`s used Loop wrap mode.
    pub fn complete(&mut self, random: &mut LegacyNanoStandRandomStream) -> LegacyNanoCompletion {
        match self.mode {
            LegacyNanoAnimationMode::Stand => self.request_stand(random),
            LegacyNanoAnimationMode::Call => {
                if let Some(skill) = self.passive_skill.take() {
                    self.request(
                        skill,
                        nano_skill_clip(skill),
                        LegacyAnimationBlend::CrossFade100Ms,
                    );
                } else {
                    self.request_stand(random);
                }
            }
            LegacyNanoAnimationMode::Discharge => self.request(
                LegacyNanoAnimationMode::Withdraw,
                "withdraw",
                LegacyAnimationBlend::CrossFade300Ms,
            ),
            LegacyNanoAnimationMode::Withdraw => return LegacyNanoCompletion::Despawn,
            LegacyNanoAnimationMode::Casting | LegacyNanoAnimationMode::Sad => {
                self.replay_current(LegacyAnimationBlend::CrossFade300Ms);
            }
            LegacyNanoAnimationMode::Emote if self.clip().is_some_and(is_dance_clip) => {
                self.replay_current(LegacyAnimationBlend::Immediate);
            }
            LegacyNanoAnimationMode::SkillSpecial => {
                self.request_stand(random);
                return LegacyNanoCompletion::SkillSpecialFinished;
            }
            LegacyNanoAnimationMode::None => {}
            LegacyNanoAnimationMode::Happy
            | LegacyNanoAnimationMode::Win
            | LegacyNanoAnimationMode::Lose
            | LegacyNanoAnimationMode::Tie
            | LegacyNanoAnimationMode::Skill1
            | LegacyNanoAnimationMode::Skill2
            | LegacyNanoAnimationMode::Skill3
            | LegacyNanoAnimationMode::Emote => self.request_stand(random),
        }
        LegacyNanoCompletion::Continue
    }

    fn replay_current(&mut self, blend: LegacyAnimationBlend) {
        self.blend = blend;
        self.request_serial = self.request_serial.wrapping_add(1).max(1);
    }
}

#[must_use]
pub fn event_nano_mode_for_action(action: &str) -> LegacyNanoAnimationMode {
    match action {
        "stand" | "stand1" | "stand2" | "stand3" => LegacyNanoAnimationMode::Stand,
        "call" => LegacyNanoAnimationMode::Call,
        "discharge" => LegacyNanoAnimationMode::Discharge,
        "happy" => LegacyNanoAnimationMode::Happy,
        "sad" => LegacyNanoAnimationMode::Sad,
        "win" => LegacyNanoAnimationMode::Win,
        "lose" => LegacyNanoAnimationMode::Lose,
        "tie" => LegacyNanoAnimationMode::Tie,
        "withdraw" => LegacyNanoAnimationMode::Withdraw,
        "skill1" => LegacyNanoAnimationMode::Skill1,
        "skill2" => LegacyNanoAnimationMode::Skill2,
        "skill3" => LegacyNanoAnimationMode::Skill3,
        _ => LegacyNanoAnimationMode::Emote,
    }
}

#[must_use]
pub fn event_nano_blend_for_action(action: &str) -> LegacyAnimationBlend {
    match action {
        "stand" | "stand1" | "stand2" | "stand3" | "call" | "discharge" | "happy" | "sad"
        | "win" | "lose" | "tie" | "withdraw" | "skill1" | "skill2" | "skill3" => {
            LegacyAnimationBlend::CrossFade100Ms
        }
        // EventNanoAnimation.SetEmote(string) uses the longer 0.3 second fade.
        _ => LegacyAnimationBlend::CrossFade300Ms,
    }
}

#[must_use]
pub fn is_dance_clip(clip: &str) -> bool {
    matches!(clip, "dance1" | "dance2" | "dance3" | "dance4" | "dance5")
}

const fn nano_skill_clip(mode: LegacyNanoAnimationMode) -> &'static str {
    match mode {
        LegacyNanoAnimationMode::Skill1 => "skill1",
        LegacyNanoAnimationMode::Skill2 => "skill2",
        LegacyNanoAnimationMode::Skill3 => "skill3",
        _ => "skill1",
    }
}

/// One app-global legacy Unity stream shared by cutscene, gameplay, portrait
/// Nano, NPC Barker, movement-pattern and animation-event owners. Unity used
/// the same process-global `Random.Range` source for all of them.
#[derive(Debug, Resource)]
pub struct LegacyNanoStandRandomStream {
    state: u32,
    draw_count: u64,
}

impl Default for LegacyNanoStandRandomStream {
    fn default() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let folded = nanos ^ (nanos >> 32) ^ (nanos >> 64) ^ (nanos >> 96);
        Self::with_seed(folded as u32)
    }
}

impl LegacyNanoStandRandomStream {
    const FALLBACK_SEED: u32 = 0x4e41_4e4f;

    #[must_use]
    pub const fn with_seed(seed: u32) -> Self {
        Self {
            state: if seed == 0 { Self::FALLBACK_SEED } else { seed },
            draw_count: 0,
        }
    }

    /// One draw from the app-global legacy Nano random stream.
    ///
    /// Unity's Nano animation, movement-pattern, and animation-event code all
    /// consumed the same `UnityEngine.Random` source. Keeping the primitive
    /// draw here prevents those owners from silently becoming independent.
    pub fn next_index(&mut self, upper_exclusive: usize) -> usize {
        assert!(
            upper_exclusive > 0,
            "legacy Unity random range must be non-empty"
        );
        self.draw_count = self.draw_count.wrapping_add(1);
        advance_native_xorshift32(&mut self.state) as usize % upper_exclusive
    }

    /// Native stand-in for Unity's floating `Random.Range(0f, 1f)` draw.
    pub fn next_unit_f32(&mut self) -> f32 {
        self.draw_count = self.draw_count.wrapping_add(1);
        advance_native_xorshift32(&mut self.state) as f32 / u32::MAX as f32
    }

    pub fn next_stand_clip(&mut self) -> &'static str {
        match self.next_index(3) {
            0 => "stand1",
            1 => "stand2",
            _ => "stand3",
        }
    }

    #[must_use]
    pub const fn draw_count(&self) -> u64 {
        self.draw_count
    }
}

#[cfg(test)]
mod tests;
