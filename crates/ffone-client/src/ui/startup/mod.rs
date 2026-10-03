//! Production startup phase for UI trees that are not needed by login.
//!
//! Preview harnesses and focused UI tests keep `Gameplay` as the default, so
//! their first `App::update` still materializes the requested presentation.
//! The real client starts in `Deferred`. Phases change on every character
//! entry, but resident gameplay trees are constructed only on the first entry.

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

/// One-time construction of resident gameplay windows, cameras and assets.
/// Register constructors here, not on the recurring `OnEnter(Gameplay)`.
/// The schedule finishes (including deferred spawns) before gameplay updates.
#[derive(ScheduleLabel, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NativeGameplayUiStartup;

#[derive(Resource)]
struct NativeGameplayUiInitialized;

fn initialize_gameplay_ui(world: &mut World) {
    if world.contains_resource::<NativeGameplayUiInitialized>() {
        return;
    }
    world.run_schedule(NativeGameplayUiStartup);
    world.insert_resource(NativeGameplayUiInitialized);
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum NativeUiStartupPhase {
    Deferred,
    CharacterSelection,
    CharacterCreation,
    #[default]
    Gameplay,
}

/// Umbrella set for systems whose entities and presentation resources are
/// created by the one-time [`NativeGameplayUiStartup`] pass.
///
/// The set is configured in every frame schedule used by those plugins. This
/// keeps their update systems dormant while the production client is still at
/// login/loading, rather than requiring every resource access to be optional.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub struct NativeUiStartupSet;

#[derive(Resource)]
struct NativeUiStartupSchedulesConfigured;

/// Initializes the shared native-UI startup state without resetting a phase
/// already selected by the production client.
///
/// Focused previews install individual UI plugins and therefore still need
/// the default `Gameplay` state. Production inserts `Deferred` before adding
/// those plugins; checking the state resource avoids both overwriting that
/// choice and asking Bevy to initialize the same state for every plugin.
pub fn init_native_ui_startup_phase(app: &mut App) {
    if !app.is_plugin_added::<bevy::state::app::StatesPlugin>() {
        app.add_plugins(bevy::state::app::StatesPlugin);
    }
    if !app
        .world()
        .contains_resource::<State<NativeUiStartupPhase>>()
    {
        app.init_state::<NativeUiStartupPhase>();
    }

    if !app
        .world()
        .contains_resource::<NativeUiStartupSchedulesConfigured>()
    {
        app.init_schedule(NativeGameplayUiStartup)
            .add_systems(
                OnEnter(NativeUiStartupPhase::Gameplay),
                initialize_gameplay_ui,
            )
            .configure_sets(
                PreUpdate,
                NativeUiStartupSet.run_if(in_state(NativeUiStartupPhase::Gameplay)),
            )
            .configure_sets(
                Update,
                NativeUiStartupSet.run_if(in_state(NativeUiStartupPhase::Gameplay)),
            )
            .configure_sets(
                FixedUpdate,
                NativeUiStartupSet.run_if(in_state(NativeUiStartupPhase::Gameplay)),
            )
            .insert_resource(NativeUiStartupSchedulesConfigured);
    }
}

#[cfg(test)]
mod tests;
