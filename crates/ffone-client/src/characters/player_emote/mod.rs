//! Player-owned emote events and continuation requests.
use crate::tutorial_player_presentation::TutorialPlayerClip;
use bevy::prelude::*;
use ffone_runtime_contracts::PlayerRigGender;

mod events;

pub(crate) struct PlayerEmoteEvents {
    pub end: f32,
    pub sounds: &'static [(f32, &'static str)],
}

pub(crate) const fn emote_events(
    gender: PlayerRigGender,
    clip: TutorialPlayerClip,
) -> Option<PlayerEmoteEvents> {
    events::events(gender, clip)
}

#[derive(Default)]
pub(crate) struct PlayerEmoteCursor {
    elapsed: f32,
    pub waiting_for_echo: bool,
}

impl PlayerEmoteCursor {
    pub fn sounds(
        &mut self,
        events: &PlayerEmoteEvents,
        elapsed: f32,
    ) -> impl Iterator<Item = &'static str> + use<> {
        let previous = self.elapsed;
        self.elapsed = elapsed;
        events
            .sounds
            .iter()
            .filter_map(move |&(time, sound)| (time > previous && time <= elapsed).then_some(sound))
    }
}

/// Drained by the network owner; animation starts only on the server echo.
#[derive(Resource)]
pub struct PlayerEmoteContinuation {
    pub pending: Vec<(Entity, i32)>,
    random: u64,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerEmoteAdvance;

impl Default for PlayerEmoteContinuation {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            random: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
                | 1,
        }
    }
}

impl PlayerEmoteContinuation {
    pub fn next_code(&mut self, clip: TutorialPlayerClip) -> Option<i32> {
        match clip {
            TutorialPlayerClip::Beach1 => Some(22),
            TutorialPlayerClip::Beach2 => Some(23),
            TutorialPlayerClip::Beach3 => Some(24),
            TutorialPlayerClip::Dance1
            | TutorialPlayerClip::Dance2
            | TutorialPlayerClip::Dance3
            | TutorialPlayerClip::Dance4
            | TutorialPlayerClip::Dance5 => {
                self.random ^= self.random << 13;
                self.random ^= self.random >> 7;
                self.random ^= self.random << 17;
                Some([6, 17, 18, 19, 20][self.random as usize % 5])
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
