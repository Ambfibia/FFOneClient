use super::*;
#[test]
fn animation_has_source_endpoints_and_cancel_releases_icon() {
    assert_eq!(berp(0.), 0.);
    assert_eq!(berp(1.), 1.);
    let mut visual = DragVisual {
        active: true,
        icon: UserEquipPresentationIcon::Resolved("icons/test.png".into()),
        ..default()
    };
    visual.drop_at(Some(Vec2::new(10., 20.)));
    assert_eq!(visual.cursor, Vec2::new(10., 20.));
    visual.cancel();
    assert!(!visual.active);
    assert_eq!(visual.icon, UserEquipPresentationIcon::Empty);
}
