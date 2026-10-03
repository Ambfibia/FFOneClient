use super::*;

#[derive(Resource, Debug)]
pub struct CharacterSelectionPortraitsModel {
    pub visible: bool,
    pub slots: [CharacterSelectionPortraitSlot; CHARACTER_SELECTION_PORTRAIT_COUNT],
}

impl Default for CharacterSelectionPortraitsModel {
    fn default() -> Self {
        Self {
            visible: false,
            slots: std::array::from_fn(|_| CharacterSelectionPortraitSlot::default()),
        }
    }
}

impl CharacterSelectionPortraitsModel {
    pub fn set_look(&mut self, slot: usize, look: NativePlayerLook) -> Result<(), String> {
        let target = self
            .slots
            .get_mut(slot)
            .ok_or_else(|| format!("character portrait slot {slot} is outside 0..4"))?;
        look.validate()?;
        if target.look.as_ref() == Some(&look) {
            return Ok(());
        }
        target.look = Some(look);
        target.revision = target.revision.wrapping_add(1).max(1);
        target.status = CharacterSelectionPortraitStatus::Loading;
        Ok(())
    }

    pub fn clear_slot(&mut self, slot: usize) -> Result<(), String> {
        let target = self
            .slots
            .get_mut(slot)
            .ok_or_else(|| format!("character portrait slot {slot} is outside 0..4"))?;
        if target.look.is_none() && matches!(target.status, CharacterSelectionPortraitStatus::Empty)
        {
            return Ok(());
        }
        target.look = None;
        target.revision = target.revision.wrapping_add(1).max(1);
        target.status = CharacterSelectionPortraitStatus::Empty;
        Ok(())
    }

    pub fn block_slot(&mut self, slot: usize, error: impl Into<String>) -> Result<(), String> {
        let target = self
            .slots
            .get_mut(slot)
            .ok_or_else(|| format!("character portrait slot {slot} is outside 0..4"))?;
        let error = error.into();
        if target.look.is_none()
            && target.status == CharacterSelectionPortraitStatus::Blocked(error.clone())
        {
            return Ok(());
        }
        target.look = None;
        target.revision = target.revision.wrapping_add(1).max(1);
        target.status = CharacterSelectionPortraitStatus::Blocked(error);
        Ok(())
    }
}

/// Selects one independent character-selection idle instance as the
/// source for the upper-left gameplay HUD render texture. The moving local
/// avatar is never a camera target and never enters this portrait layer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct GameplayPlayerPortraitModel {
    pub visible: bool,
    pub slot: Option<usize>,
}
