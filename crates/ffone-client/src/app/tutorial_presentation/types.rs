use super::*;

pub(in super::super) struct TutorialPresentationPlugin;

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in super::super) enum TutorialPresentationSet {
    GameplayHud,
}

impl Plugin for TutorialPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            sync_gameplay_hud
                .after(toggle_player_interaction)
                // Chat/Nanocom input must project the current HUD model before
                // LegacyMovement reads this frame's input. Ordering this after
                // the post-movement tutorial driver would close that schedule
                // loop; tutorial-owned HUD changes are projected next frame.
                .before(GameplayUiSet::BindNanoWheel)
                .in_set(TutorialPresentationSet::GameplayHud),
        )
        .add_systems(
            Update,
            sync_world_location_notice
                .after(sync_gameplay_hud)
                .after(sync_gameplay_loading_screen)
                .before(GameplayUiSet::MapNotice),
        )
        .add_systems(
            Update,
            sync_tutorial_overlay
                .after(drive_local_tutorial)
                .after(apply_pending_tutorial_exit),
        );
    }
}

#[derive(SystemParam)]
pub(super) struct GameplayHudOptionInputs<'w, 's> {
    pub(super) runtime: Res<'w, OptionProductionRuntime>,
    pub(super) windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}

#[derive(SystemParam)]
pub(super) struct GameplayHudMissionInputs<'w, 's> {
    pub(super) skill_buffs: Res<'w, SkillBuffUiModel>,
    pub(super) shinies: Query<'w, 's, &'static GlobalTransform, With<ffone_client::entity_lifecycle::NetworkShiny0104>>,
    pub(super) tutorial: Res<'w, TutorialMissionRuntime>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) world: Res<'w, WorldMissionRuntime>,
    pub(super) world_map: Res<'w, WorldMapPresentation>,
    pub(super) world_map_production: Res<'w, WorldMapProductionRuntime>,
    pub(super) waypoint: Res<'w, WorldMissionWaypointRuntime>,
    pub(super) inventory: Res<'w, LocalInventoryRuntime>,
    pub(super) guide: Res<'w, GuideRuntime>,
    pub(super) nano_bank: Res<'w, NanoFreeTuningBank0104>,
    pub(super) localization: Res<'w, Localization>,
    pub(super) language: Res<'w, Language>,
}
