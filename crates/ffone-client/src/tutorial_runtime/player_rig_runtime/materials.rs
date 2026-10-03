use super::*;

#[derive(Component, Clone, Debug)]
pub struct TutorialPlayerAppearanceMaterialBound {
    pub rig_root: Entity,
    pub exact_route: String,
    pub binding: NativePlayerMaterialBinding,
}

#[derive(Component)]
pub(super) struct TutorialPlayerWeaponMaterialBound {
    // Retain the asynchronous replacement texture through its consumer.
    pub(super) _binding: Option<NativePlayerMaterialBinding>,
}
