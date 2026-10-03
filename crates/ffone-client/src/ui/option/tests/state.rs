use super::*;

#[test]
fn idle_key_capture_does_not_dirty_option_state() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .init_resource::<OptionUiModel>()
        .add_systems(Update, tick_option_key_capture);

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    assert!(!app.world().resource_ref::<OptionUiModel>().is_changed());
}
