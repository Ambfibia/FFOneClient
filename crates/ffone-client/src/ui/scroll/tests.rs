use super::*;
#[test]
fn held_arrow_repeats_then_modal_cancels_until_a_new_press() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(BankUiState {
            phase: BankLifecyclePhase::Visible,
            ..default()
        })
        .init_resource::<BankModalState>()
        .init_resource::<BankModeProjection0104>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_systems(Update, input);
    let mut window = Window::default();
    window.focused = true;
    window.set_physical_cursor_position(Some(bevy::math::DVec2::ZERO));
    app.world_mut().spawn((window, PrimaryWindow));
    for part in [Part::Track, Part::Down] {
        app.world_mut().spawn((
            Owner::Bank,
            part,
            if part == Part::Down {
                Interaction::Pressed
            } else {
                Interaction::None
            },
            ComputedNode::default(),
            UiGlobalTransform::default(),
        ));
    }
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(app.world().resource::<BankUiState>().bank_scroll_y, 10.);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(200));
    app.update();
    assert_eq!(app.world().resource::<BankUiState>().bank_scroll_y, 10.);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(60));
    app.update();
    assert_eq!(app.world().resource::<BankUiState>().bank_scroll_y, 20.);
    app.world_mut()
        .resource_mut::<BankModalState>()
        .system_popup = true;
    app.update();
    app.world_mut()
        .resource_mut::<BankModalState>()
        .system_popup = false;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(1));
    app.update();
    assert_eq!(app.world().resource::<BankUiState>().bank_scroll_y, 20.);
}
#[test]
fn scrollbar_extent_and_pointer_travel_share_the_same_endpoints() {
    for (track, page, maximum) in [
        (421.0, 445.0, 1833.0),
        (376.0, 400.0, 80.0),
        (476.0, 500.0, user_equip_scroll_max()),
    ] {
        let extent = thumb_extent(track, page, maximum);
        let travel = track - extent;
        assert!(extent > 15.0 && extent < track);
        assert_eq!(drag_value(0.0, travel, maximum, travel), maximum);
        assert_eq!(drag_value(maximum, -travel, maximum, travel), 0.0);
        assert_eq!(page_direction(extent * 0.5, 0.0, extent), 0.0);
        assert_eq!(page_direction(extent + 1.0, 0.0, extent), 1.0);
    }
    assert_eq!(thumb_extent(421.0, 445.0, 0.0), 421.0);
    assert!(thumb_extent(421.0, 445.0, 67.0) > thumb_extent(421.0, 445.0, 1833.0));
}
#[test]
fn thumb_preserves_grab_offset_and_clamps_both_ends() {
    assert_eq!(drag_value(100., 0., 500., 400.), 100.);
    assert_eq!(drag_value(100., 80., 500., 400.), 200.);
    assert_eq!(drag_value(100., 900., 500., 400.), 500.);
    assert_eq!(drag_value(100., -900., 500., 400.), 0.);
}
