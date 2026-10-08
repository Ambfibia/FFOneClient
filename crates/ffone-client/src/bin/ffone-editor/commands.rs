use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NpcInspectorTab { Details, Animations, Edit }

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditorAction {
    Tab(CatalogKind),
    Strings,
    Xdt,
    Missions,
    World2d,
    World3d,
    NewNpcTemplate,
    FocusSearch,
    ClearSearch,
    CatalogSlot(usize),
    DefaultPose,
    TPose,
    AnimationSlot(usize),
    AnimationPreviousPage,
    AnimationNextPage,
    PreviousClip,
    NextClip,
    TogglePlayback,
    ToggleLoop,
    SpeedDown,
    SpeedUp,
    ResetCamera,
    ToggleTurntable,
    ToggleLanguage,
    ToggleDetails,
    NpcInspector(NpcInspectorTab),
    EquipmentGender(bool),
    EquipmentCategory(Option<ffone_runtime_contracts::AvatarItemCategory>),
    ResetOutfit,
    IconGenerator,
}
