use super::*;

pub const CHARACTER_SELECTION_PORTRAIT_HEAD_BONE: &str = "Bip01 Head";

pub const GAMEPLAY_PLAYER_PORTRAIT_NECK_BONE: &str = "Bip01 Neck";

#[derive(Component)]
pub(super) struct CharacterSelectionPortraitRig {
    pub(super) slot: usize,
    pub(super) generation: u64,
}
