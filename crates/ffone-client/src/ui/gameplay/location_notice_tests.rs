use super::combat_frame::CombatModeNoticeText;
use super::*;

#[test]
fn location_notice_renders_localized_motion_expires_and_yields_to_combat() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (catalog, language) = crate::localization::Localization::open(&root, "ru").unwrap();
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<CombatModeNotice>()
        .insert_resource(GameplayUiModel {
            visible: true,
            ..default()
        })
        .insert_resource(catalog)
        .insert_resource(language)
        .add_systems(Update, update_combat_mode_notice);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let label = app
        .world_mut()
        .spawn((
            CombatModeNoticeText { shadow: false },
            Node::default(),
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
            TextColor(Color::WHITE),
            UiTransform::default(),
        ))
        .id();
    let mut random = LegacyNanoStandRandomStream::with_seed(7);
    for movement in 0..8 {
        {
            let mut notice = app.world_mut().resource_mut::<CombatModeNotice>();
            notice.show_location("Sector V", &mut random);
            notice.movement = movement;
            notice.elapsed = Some(1.0);
        }
        app.update();
        let entity = app.world().entity(label);
        assert_eq!(entity.get::<Node>().unwrap().display, Display::Flex);
        assert_eq!(
            entity.get::<LocalizedText>().unwrap().key,
            "content.location.world.sector_v"
        );
        let color = entity.get::<TextColor>().unwrap().0.to_srgba();
        assert!(color.alpha > 0.99);
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(2));
    app.update();
    assert_eq!(
        app.world().entity(label).get::<Node>().unwrap().display,
        Display::None
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::ZERO);
    app.world_mut()
        .resource_mut::<CombatModeNotice>()
        .show(false);
    app.update();
    assert_eq!(
        app.world()
            .entity(label)
            .get::<LocalizedText>()
            .unwrap()
            .key,
        "ui.gameplay.combat.disabled"
    );
    app.world_mut().resource_mut::<GameplayUiModel>().visible = false;
    app.update();
    assert_eq!(
        app.world().entity(label).get::<Node>().unwrap().display,
        Display::None
    );
}
