use super::*;


#[test]
fn fractional_shaped_line_height_does_not_shrink_a_fitting_label() {
    use parley::{FontContext, LayoutContext, StyleProperty};
    let bytes = fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/game/fonts/jeffe.otf"
    ))
    .unwrap();
    let mut fonts = FontContext::new();
    fonts.collection.register_fonts(
        bytes.into(),
        Some(parley::fontique::FontInfoOverride {
            family_name: Some("Fixture"),
            ..default()
        }),
    );
    for scale in [1.0, 1.5, 2.0] {
        let mut context = LayoutContext::<TextBrush>::new();
        let mut builder = context.ranged_builder(&mut fonts, "Create a Character", scale, true);
        builder.push_default(StyleProperty::FontFamily("Fixture".into()));
        builder.push_default(StyleProperty::FontSize(14.0));
        builder.push_default(StyleProperty::LineHeight(parley::LineHeight::Absolute(
            16.452,
        )));
        let mut buffer = builder.build("Create a Character");
        buffer.break_all_lines(None);
        let layout = TextLayoutInfo {
            scale_factor: scale,
            size: Vec2::new(buffer.full_width(), buffer.height()).ceil() / scale,
            ..default()
        };
        let measured = auto_fit_measurement(&buffer, &layout);
        assert_eq!(
            auto_fit_font_size(14.0, 6.0, Vec2::new(400.0, 16.452), measured),
            14.0,
            "physical pixel allocation must not shrink a fitting line at scale {scale}"
        );
        assert!(
            auto_fit_font_size(14.0, 6.0, Vec2::new(measured.x * 0.5, 16.452), measured) < 14.0,
            "real width overflow must still shrink"
        );
    }
}

#[test]
fn auto_fit_shrinks_only_overflowing_text_and_respects_its_floor() {
    assert_eq!(
        auto_fit_font_size(14.0, 9.0, Vec2::new(100.0, 20.0), Vec2::new(90.0, 18.0)),
        14.0
    );
    assert_eq!(
        auto_fit_font_size(14.0, 9.0, Vec2::new(100.0, 20.0), Vec2::new(200.0, 18.0)),
        9.0
    );
    assert_eq!(
        auto_fit_font_size(14.0, 9.0, Vec2::new(100.0, 20.0), Vec2::new(125.0, 18.0)),
        10.0
    );
}

#[test]
fn editable_fields_scroll_without_entering_label_auto_fit() {
    let mut app = App::new();
    app.add_systems(Update, admit_bounded_ui_text);
    let parent = app
        .world_mut()
        .spawn(Node {
            width: px(100.0),
            height: px(25.0),
            ..default()
        })
        .id();
    let field = app
        .world_mut()
        .spawn((
            Text::new("very long editable login"),
            LocalizedText::new("ui.content.passthrough", "{text}")
                .with_arg("text", "very long editable login"),
            crate::text_edit::EditVisual::default(),
            ChildOf(parent),
        ))
        .id();
    app.update();
    assert!(app.world().get::<UiTextAutoFit>(field).is_none());
    app.world_mut()
        .entity_mut(field)
        .remove::<crate::text_edit::EditVisual>();
    app.update();
    assert!(
        app.world().get::<UiTextAutoFit>(field).is_some(),
        "same fixed region must still admit an ordinary label"
    );
}

#[test]
fn hidden_login_labels_do_not_keep_a_minimum_width_font_size() {
    let mut app = App::new();
    app.add_systems(
        Update,
        (admit_bounded_ui_text, refresh_ui_text_fit_regions).chain(),
    );
    app.add_systems(PostUpdate, auto_fit_ui_text);
    let label = app
        .world_mut()
        .spawn((
            Text::new("ПАРОЛЬ :"),
            LocalizedText::new("ui.login.password", "Password :"),
            Node {
                width: px(200.0),
                height: px(20.0),
                padding: UiRect::vertical(px(3.0)),
                ..default()
            },
            TextFont {
                font_size: 14.0.into(),
                ..default()
            },
            LineHeight::Px(16.452),
            // The initial hidden frame wraps at minimum width into two lines.
            TextLayoutInfo {
                size: Vec2::new(73.0, 33.0),
                ..default()
            },
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        app.world().get::<TextFont>(label).unwrap().font_size,
        bevy::text::FontSize::Px(14.0)
    );
    app.world_mut().get_mut::<ComputedNode>(label).unwrap().size = Vec2::new(200.0, 20.0);
    app.world_mut()
        .get_mut::<TextLayoutInfo>(label)
        .unwrap()
        .size = Vec2::new(78.0, 16.452);
    app.update();
    assert_eq!(
        app.world().get::<TextFont>(label).unwrap().font_size,
        bevy::text::FontSize::Px(14.0)
    );
    // Real overflow after opening must still be fitted.
    app.world_mut()
        .get_mut::<TextLayoutInfo>(label)
        .unwrap()
        .size = Vec2::new(400.0, 16.452);
    app.update();
    assert!(
        app.world()
            .get::<TextFont>(label)
            .unwrap()
            .font_size
            .eval(Vec2::ZERO, 16.0)
            < 14.0
    );
}

#[test]
fn bounded_labels_fit_restore_and_keep_their_anchor() {
    let mut app = App::new();
    app.add_systems(
        Update,
        (admit_bounded_ui_text, refresh_ui_text_fit_regions).chain(),
    );
    app.add_systems(PostUpdate, auto_fit_ui_text);
    let parent = app
        .world_mut()
        .spawn(Node {
            width: px(120.0),
            height: px(30.0),
            padding: UiRect::horizontal(px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    let label = app
        .world_mut()
        .spawn((
            Text::new("Длинная русская надпись"),
            LocalizedText::new("test.label", "Label"),
            (
                TextFont {
                    font_size: (20.0).into(),
                    ..default()
                },
                LineHeight::Px(24.0),
            ),
            TextLayoutInfo {
                size: Vec2::new(240.0, 24.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    app.world_mut().get_mut::<ComputedNode>(label).unwrap().size = Vec2::new(240.0, 24.0);
    app.update();
    let fitted = (
        app.world().get::<TextFont>(label).unwrap(),
        app.world().get::<LineHeight>(label).unwrap(),
    );
    assert!(
        fitted.0.font_size.eval(Vec2::ZERO, 16.0) < 14.0,
        "must be allowed below the old 70% floor"
    );
    assert_eq!(
        (*fitted.1),
        LineHeight::Px(24.0 * fitted.0.font_size.eval(Vec2::ZERO, 16.0) / 20.0)
    );
    assert_eq!(
        app.world().get::<Node>(parent).unwrap().align_items,
        AlignItems::Center
    );
    // Mimic the next completed layout, then change language back to short copy.
    app.world_mut()
        .get_mut::<TextLayoutInfo>(label)
        .unwrap()
        .size = Vec2::new(90.0, 10.0);
    app.world_mut().get_mut::<Text>(label).unwrap().0 = "Label".into();
    app.update();
    assert_eq!(
        app.world().get::<TextFont>(label).unwrap().font_size,
        bevy::text::FontSize::Px(20.0)
    );
    // A resized source rectangle must reset the original metrics as well.
    app.world_mut().get_mut::<Node>(parent).unwrap().width = px(200.0);
    app.update();
    assert_eq!(
        app.world().get::<UiTextAutoFit>(label).unwrap().max_width,
        180.0
    );
    app.world_mut().clear_trackers();
    app.update();
    assert!(
        !app.world()
            .entity(label)
            .get_ref::<TextFont>()
            .unwrap()
            .is_changed()
    );
}

#[test]
fn fitting_does_not_admit_content_sized_or_shared_regions() {
    assert_eq!(fit_region_height(5.0, 12.0, LineHeight::Px(16.0)), 16.0);
    assert_eq!(fit_region_height(30.0, 12.0, LineHeight::Px(16.0)), 30.0);
    assert!(fixed_text_region(&Node::default()).is_none());
    let mut app = App::new();
    app.add_systems(Update, admit_bounded_ui_text);
    let parent = app
        .world_mut()
        .spawn(Node {
            width: px(100.0),
            height: px(30.0),
            ..default()
        })
        .id();
    for _ in 0..2 {
        app.world_mut().spawn((
            Text::new("Label"),
            LocalizedText::new("test.label", "Label"),
            ChildOf(parent),
        ));
    }
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&UiTextAutoFit>()
            .iter(app.world())
            .count(),
        0
    );
    assert_eq!(
        auto_fit_font_size(14.0, 6.0, Vec2::ONE, Vec2::splat(f32::NAN)),
        14.0
    );
}
