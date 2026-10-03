//! Ordinary-world Nano cooldown channels.
//!
//! Clean `GameCondition` owns one transient channel for each equipped Nano
//! slot. The accepted local request starts that channel immediately; server
//! success packets reconcile Nano/stamina state but do not start or restart
//! the UI timer.

use bevy::prelude::Resource;

pub const WORLD_NANO_EQUIPPED_SLOT_COUNT: usize = 3;
const LEGACY_COOLDOWN_TENTHS_TO_SECONDS: f32 = 0.1;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldNanoSlotCooldown {
    pub skill_id: i16,
    /// Raw XDT `m_iCoolType`, retained so the channel never loses its source
    /// classification even though clean HUD ownership is per equipped slot.
    pub cool_type: i32,
    pub total_seconds: f32,
    pub remaining_seconds: f32,
}

impl WorldNanoSlotCooldown {
    #[must_use]
    pub fn is_active_for(self, skill_id: i16) -> bool {
        self.skill_id == skill_id
            && self.total_seconds.is_finite()
            && self.total_seconds > 0.0
            && self.remaining_seconds.is_finite()
            && self.remaining_seconds > 0.0
    }

    #[must_use]
    pub fn remaining_fraction_for(self, skill_id: i16) -> Option<f32> {
        self.is_active_for(skill_id)
            .then(|| (self.remaining_seconds / self.total_seconds).clamp(0.0, 1.0))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct WorldNanoCooldownRuntime {
    slots: [WorldNanoSlotCooldown; WORLD_NANO_EQUIPPED_SLOT_COUNT],
}

impl WorldNanoCooldownRuntime {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub const fn slots(&self) -> &[WorldNanoSlotCooldown; WORLD_NANO_EQUIPPED_SLOT_COUNT] {
        &self.slots
    }

    #[must_use]
    pub fn is_active(&self, slot: usize, skill_id: i16) -> bool {
        self.slots
            .get(slot)
            .is_some_and(|cooldown| cooldown.is_active_for(skill_id))
    }

    #[must_use]
    pub fn remaining_fraction(&self, slot: usize, skill_id: i16) -> Option<f32> {
        self.slots
            .get(slot)
            .and_then(|cooldown| cooldown.remaining_fraction_for(skill_id))
    }

    /// Starts the exact equipped-slot channel from XDT tenths of a second.
    /// A zero-duration skill is valid and leaves no active overlay.
    pub fn start(
        &mut self,
        slot: usize,
        skill_id: i16,
        cool_type: i32,
        cooldown_tenths: i32,
    ) -> Result<f32, String> {
        let channel = self
            .slots
            .get_mut(slot)
            .ok_or_else(|| format!("invalid equipped Nano cooldown slot {slot}"))?;
        if skill_id <= 0 {
            return Err(format!("invalid Nano cooldown skill {skill_id}"));
        }
        if cooldown_tenths < 0 {
            return Err(format!(
                "Nano skill {skill_id} has negative XDT cooldown {cooldown_tenths}"
            ));
        }
        let total_seconds = cooldown_tenths as f32 * LEGACY_COOLDOWN_TENTHS_TO_SECONDS;
        *channel = WorldNanoSlotCooldown {
            skill_id,
            cool_type,
            total_seconds,
            remaining_seconds: total_seconds,
        };
        Ok(total_seconds)
    }

    /// Drops channels whose equipped skill identity changed through load,
    /// tuning, regeneration, or a PC tick. Active-Nano switching alone does
    /// not change these identities and therefore preserves every timer.
    pub fn retain_equipped(&mut self, equipped_skill_ids: [Option<i16>; 3]) {
        for (channel, equipped) in self.slots.iter_mut().zip(equipped_skill_ids) {
            if equipped != Some(channel.skill_id) {
                *channel = WorldNanoSlotCooldown::default();
            }
        }
    }

    pub fn advance(&mut self, delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        for channel in &mut self.slots {
            if channel.remaining_seconds > 0.0 {
                channel.remaining_seconds = (channel.remaining_seconds - delta_seconds).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests;
