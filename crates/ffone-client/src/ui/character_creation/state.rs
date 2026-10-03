use super::*;

/// The clean default is `localized.weapon = eNone`; the class-selection branch
/// in `CnGuiCharCreation` is consequently unreachable in primary.
pub const CHARACTER_CREATION_PRIMARY_CLASS_SELECTION_REACHABLE: bool = false;

pub(super) const CC_COLOR_SELECTED: &str = "ui/en/character/creation/colors/CCSelectedColor.png";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterNameMode {
    #[default]
    Generated,
    Custom,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterCreationPreviewStatus {
    #[default]
    PlayerAssemblyPending,
    Ready,
}

#[derive(Component)]
pub(super) struct ModeGeneratedRoot;

#[derive(Component)]
pub(super) struct ModeCustomRoot;

#[derive(Component)]
pub(super) struct SelectedColor(pub(super) CharacterCreationColorKind, pub(super) u8);
