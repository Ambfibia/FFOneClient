use super::*;

pub(super) fn pointer_app() -> App {
    let mut projection = CombiModeProjection0104::default();
    for index in [4_usize, 7] {
        projection.inventory[index].item = Some(item(0, index as i16, 0, 0));
    }
    let mut app = App::new();
    app.insert_resource(CombiUiState0104 {
        phase: CombiPhase0104::Ready,
        ..Default::default()
    })
    .insert_resource(projection)
    .init_resource::<CombiUiOutbox0104>()
    .init_resource::<CombiPointerState0104>()
    .init_resource::<ButtonInput<MouseButton>>()
    .add_systems(Update, collect_combi_ui_input);
    app
}

#[test]
fn wheel_scrolls_the_bag_through_the_shared_my_stuff_range() {
    let mut app = pointer_app();
    app.insert_resource(AccumulatedMouseScroll {
        unit: bevy::input::mouse::MouseScrollUnit::Line,
        delta: Vec2::new(0.0, -100.0),
    });
    app.update();
    let max = crate::user_equip_ui::user_equip_scroll_max();
    assert!(max > 0.0, "the 10-row bag must overflow its viewport");
    assert_eq!(
        app.world().resource::<CombiPointerState0104>().scroll_y(),
        max
    );

    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::new(0.0, 1.0);
    app.update();
    assert_eq!(
        app.world().resource::<CombiPointerState0104>().scroll_y(),
        max - 0.1 * USER_EQUIP_INVENTORY_SCROLL_VELOCITY
    );

    // Closing forgets the offset: the next visit starts at the top.
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::ZERO;
    app.world_mut().resource_mut::<CombiUiState0104>().phase = CombiPhase0104::Hidden;
    app.update();
    assert_eq!(
        app.world().resource::<CombiPointerState0104>().scroll_y(),
        0.0
    );
}
