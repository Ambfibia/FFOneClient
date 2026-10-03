use super::*;

#[test]
fn hidden_parent_and_changed_owner_reset_reused_ecs_bar() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .add_systems(Update, update_damage_bars);
    let parent = app.world_mut().spawn(Node::default()).id();
    let fill = app
        .world_mut()
        .spawn((
            Node {
                width: px(100.0),
                ..default()
            },
            DamageBarOwner(1),
            InheritedVisibility::VISIBLE,
            ZIndex(3),
            ChildOf(parent),
        ))
        .id();
    let back = app
        .world_mut()
        .spawn((
            DamageBar {
                fill,
                width: 100.0,
                owner: None,
                trail: default(),
            },
            Node::default(),
            ZIndex::default(),
        ))
        .id();
    app.update();
    app.world_mut().get_mut::<Node>(fill).unwrap().width = px(50.0);
    app.update();
    assert_eq!(app.world().get::<Node>(back).unwrap().width, px(100.0));
    assert_eq!(
        app.world().get::<Node>(back).unwrap().display,
        Display::Flex
    );
    assert_eq!(*app.world().get::<ZIndex>(back).unwrap(), ZIndex(3));
    app.world_mut().get_mut::<Node>(parent).unwrap().display = Display::None;
    app.update();
    assert_eq!(
        app.world().get::<Node>(back).unwrap().display,
        Display::None
    );
    app.world_mut().get_mut::<Node>(parent).unwrap().display = Display::Flex;
    app.update();
    assert_eq!(app.world().get::<Node>(back).unwrap().width, px(50.0));
    app.world_mut().get_mut::<Node>(fill).unwrap().width = px(20.0);
    app.world_mut().get_mut::<DamageBarOwner>(fill).unwrap().0 = 2;
    app.update();
    assert_eq!(
        app.world().get::<Node>(back).unwrap().display,
        Display::None
    );
    assert_eq!(app.world().get::<Node>(back).unwrap().width, px(20.0));
}

#[test]
fn damage_holds_then_lerps_and_consecutive_hits_do_not_restart_hold() {
    let mut trail = DamageTrail::default();
    assert_eq!(trail.advance(1.0, 0.0), 1.0);
    assert_eq!(trail.advance(0.8, 0.2), 1.0);
    assert_eq!(trail.advance(0.4, 0.2), 1.0);
    assert!((trail.advance(0.4, 0.1) - 0.7).abs() < 0.00001);
    for _ in 0..20 {
        trail.advance(0.4, 0.1);
    }
    assert_eq!(trail.advance(0.4, 0.1), 0.4);
}

#[test]
fn healing_and_rebinding_do_not_show_damage_from_another_owner() {
    let mut trail = DamageTrail::default();
    assert_eq!(trail.advance(0.5, 0.0), 0.5);
    assert_eq!(trail.advance(0.9, 0.1), 0.9);
    assert_eq!(trail.advance(0.2, 0.1), 0.9);
    trail.reset();
    assert_eq!(trail.advance(0.2, 0.1), 0.2);
    assert_eq!(trail.advance(0.1, 1.0), 0.2);
    assert_eq!(trail.advance(0.1, 1.0), 0.1);
}
