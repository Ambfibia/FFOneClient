use super::*;

#[derive(Resource)]
pub(super) struct PreviewConfig {
    pub(super) output: PathBuf,
    pub(super) rig_root: Entity,
    pub(super) controller_root: Entity,
    pub(super) gender: PlayerRigGender,
    pub(super) expected_clip: TutorialPlayerClip,
    pub(super) case: PreviewCase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PreviewCase {
    Locomotion,
    Standup,
    StartupLanding,
    AttackRun,
    AttackFall,
    Skyway,
    Zipline,
    VehicleBoard,
    VehicleScooter,
    WeaponSwap,
    Dance,
    Beach,
}

impl PreviewCase {
    pub(super) const fn captures_attack(self) -> bool {
        matches!(self, Self::AttackRun | Self::AttackFall)
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct PreviewTravelResources<'w> {
    pub(super) skyway: ResMut<'w, TutorialSkywayPresentation>,
    pub(super) vehicle:
        ResMut<'w, ffone_client::tutorial_player_rig_runtime::PersonalVehiclePresentation>,
}
