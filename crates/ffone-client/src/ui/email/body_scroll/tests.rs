use super::*;
#[test]
fn wheel_and_drag_scroll_long_mail_and_new_selection_returns_to_top() {
    let mut app = App::new();
    app.insert_resource(EmailUiModel {
        visible: true,
        ..default()
    })
    .init_resource::<AccumulatedMouseScroll>()
    .init_resource::<ButtonInput<MouseButton>>()
    .add_systems(Update, scroll);
    let view = app
        .world_mut()
        .spawn((
            Viewport,
            ComputedNode {
                size: Vec2::new(400., 100.),
                content_size: Vec2::new(400., 400.),
                inverse_scale_factor: 1.,
                ..default()
            },
            Interaction::Hovered,
            ScrollPosition::default(),
        ))
        .id();
    let track = app
        .world_mut()
        .spawn((
            Track,
            Node::default(),
            Interaction::None,
            RelativeCursorPosition::default(),
        ))
        .id();
    app.world_mut().spawn((Thumb, Node::default()));
    app.world_mut().spawn((
        EmailUiTextElement {
            role: EmailUiTextRole::DetailBody,
        },
        Text::new("body"),
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "body"),
    ));
    app.update();
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta
        .y = -2.;
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(view).unwrap().y, 60.);
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::ZERO;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    *app.world_mut().get_mut::<Interaction>(track).unwrap() = Interaction::Pressed;
    app.world_mut()
        .get_mut::<RelativeCursorPosition>(track)
        .unwrap()
        .normalized = Some(Vec2::new(0., 0.5));
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(view).unwrap().y, 300.);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    *app.world_mut().get_mut::<Interaction>(track).unwrap() = Interaction::None;
    app.world_mut().resource_mut::<EmailUiModel>().selected_row = Some(1);
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(view).unwrap().y, 0.);
}
