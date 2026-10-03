use super::*;

#[derive(Debug, Default, Resource)]
pub(in super::super) struct CharacterCreationSession {
    pub(in super::super) pending_name_check: Option<CharacterNameCheckRequest0104>,
    pub(in super::super) saved_name: Option<CharacterNameSaveSuccess0104>,
    pub(in super::super) generated_name: bool,
    pub(in super::super) pending_created_character_entry_uid: Option<i64>,
}

impl CharacterCreationSession {
    pub(in super::super) fn reset(&mut self) {
        self.pending_name_check = None;
        self.saved_name = None;
        self.generated_name = false;
        self.pending_created_character_entry_uid = None;
    }

    pub(in super::super) fn await_created_character_roster(&mut self, pc_uid: i64) {
        self.reset();
        self.pending_created_character_entry_uid = Some(pc_uid);
    }

    pub(in super::super) fn take_created_character_from_roster(
        &mut self,
        characters: &[CharacterSummary],
    ) -> Result<Option<CharacterSummary>, i64> {
        let Some(pc_uid) = self.pending_created_character_entry_uid.take() else {
            return Ok(None);
        };
        characters
            .iter()
            .find(|character| character.pc_uid == pc_uid)
            .cloned()
            .map(Some)
            .ok_or(pc_uid)
    }
}
