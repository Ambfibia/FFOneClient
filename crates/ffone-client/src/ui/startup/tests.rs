use super::{
    NativeGameplayUiStartup, NativeUiStartupPhase, NativeUiStartupSet,
    init_native_ui_startup_phase,
};
use bevy::prelude::*;

#[test]
fn initialization_preserves_production_deferred_phase() {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(NativeUiStartupPhase::Deferred);
    init_native_ui_startup_phase(&mut app);
    init_native_ui_startup_phase(&mut app);

    assert_eq!(
        app.world().resource::<State<NativeUiStartupPhase>>().get(),
        &NativeUiStartupPhase::Deferred
    );
}

#[test]
fn focused_plugins_still_default_to_gameplay() {
    let mut app = App::new();
    init_native_ui_startup_phase(&mut app);
    init_native_ui_startup_phase(&mut app);
    app.update();

    assert_eq!(
        app.world().resource::<State<NativeUiStartupPhase>>().get(),
        &NativeUiStartupPhase::Gameplay
    );
}

#[derive(Default, Resource)]
struct RunCount(u32);

fn count_run(mut count: ResMut<RunCount>) {
    count.0 += 1;
}

#[test]
fn startup_commands_finish_before_updates_and_do_not_repeat_on_reentry() {
    let mut app = App::new();
    init_native_ui_startup_phase(&mut app);
    // Repeated plugin initialization must not register another dispatcher.
    init_native_ui_startup_phase(&mut app);
    app.add_systems(NativeGameplayUiStartup, |mut commands: Commands| {
        commands.insert_resource(RunCount::default());
    });
    app.add_systems(Update, count_run.in_set(NativeUiStartupSet));
    app.update();
    assert_eq!(app.world().resource::<RunCount>().0, 1);

    for phase in [
        NativeUiStartupPhase::CharacterSelection,
        NativeUiStartupPhase::Deferred,
        NativeUiStartupPhase::CharacterCreation,
    ] {
        app.world_mut()
            .resource_mut::<NextState<NativeUiStartupPhase>>()
            .set(phase);
        app.update();
        assert_eq!(app.world().resource::<RunCount>().0, 1);
    }
    app.world_mut()
        .resource_mut::<NextState<NativeUiStartupPhase>>()
        .set(NativeUiStartupPhase::Gameplay);
    app.update();
    assert_eq!(
        app.world().resource::<RunCount>().0,
        2,
        "the resident resource must survive while updates resume"
    );
}

#[test]
fn gameplay_system_set_waits_for_the_startup_transition() {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(NativeUiStartupPhase::Deferred);
    app.init_resource::<RunCount>();
    init_native_ui_startup_phase(&mut app);
    app.add_systems(Update, count_run.in_set(NativeUiStartupSet));

    app.update();
    assert_eq!(app.world().resource::<RunCount>().0, 0);

    app.world_mut()
        .resource_mut::<NextState<NativeUiStartupPhase>>()
        .set(NativeUiStartupPhase::Gameplay);
    app.update();
    assert_eq!(app.world().resource::<RunCount>().0, 1);
}
