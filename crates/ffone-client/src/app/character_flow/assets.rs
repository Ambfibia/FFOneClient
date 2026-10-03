use super::*;

#[derive(Debug, Default, Resource)]
pub(in super::super) struct CharacterCreationAssetLease {
    pub(in super::super) pending: VecDeque<CharacterAppearance>,
    pub(in super::super) total: usize,
    pub(in super::super) completed: usize,
    pub(in super::super) blocker: Option<String>,
}

impl CharacterCreationAssetLease {
    pub(in super::super) fn progress(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.completed as f32 / self.total as f32
        }
    }

    pub(in super::super) fn is_ready(&self) -> bool {
        self.total > 0 && self.completed == self.total && self.blocker.is_none()
    }

    pub(in super::super) fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in super::super) enum CharacterEntryRoute {
    ResumeAppearance,
    TutorialSequence,
    ShardWorld,
}

pub(in super::super) fn character_entry_route(
    character: &CharacterSummary,
) -> Result<CharacterEntryRoute, i8> {
    if character.style.appearance_flag == 0 {
        return Ok(CharacterEntryRoute::ResumeAppearance);
    }
    match character.style.tutorial_flag {
        0 => Ok(CharacterEntryRoute::TutorialSequence),
        1 => Ok(CharacterEntryRoute::ShardWorld),
        value => Err(value),
    }
}
