use super::*;

#[test]
fn exact_center_pivot_layout_matches_legacy_rectangles() {
    let viewport = Vec2::new(1264.0, 681.0);
    let layout = CharacterCreationLayout::for_viewport(viewport, 1.0);
    assert_eq!(
        layout.background,
        LegacyCreationRect::new(-328.0, -379.5, 1920.0, 1440.0)
    );
    assert_eq!(
        layout.appearance,
        LegacyCreationRect::new(122.0, 21.5, 1020.0, 638.0)
    );
    assert_eq!(
        layout.name,
        LegacyCreationRect::new(296.0, 158.0, 672.0, 365.0)
    );
    assert_eq!(
        layout.fullscreen,
        LegacyCreationRect::new(1219.0, 646.0, 35.0, 31.0)
    );
}

#[test]
fn ui_scale_uses_center_and_bottom_right_source_pivots() {
    let viewport = Vec2::new(1280.0, 720.0);
    let layout = CharacterCreationLayout::for_viewport(viewport, 1.25);
    assert_eq!(layout.appearance.center(), viewport * 0.5);
    assert_eq!(
        layout.appearance.width,
        CHARACTER_CREATION_APPEARANCE_WIDTH * 1.25
    );
    assert_eq!(
        layout.fullscreen.x + layout.fullscreen.width,
        1280.0 - 10.0 * 1.25
    );
    assert_eq!(
        layout.fullscreen.y + layout.fullscreen.height,
        720.0 - 4.0 * 1.25
    );
}

#[test]
fn every_reached_primary_button_keeps_its_clean_on_gui_rect() {
    fn pixels(value: Val) -> f32 {
        let Val::Px(value) = value else {
            panic!("creation controls must use source pixel Rects");
        };
        value
    }
    fn rect(node: &Node) -> LegacyCreationRect {
        LegacyCreationRect::new(
            pixels(node.left),
            pixels(node.top),
            pixels(node.width),
            pixels(node.height),
        )
    }

    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins(NativeCharacterCreationUiPlugin);
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<(&CharacterCreationButton, &Node)>();
    let controls = query
        .iter(world)
        .map(|(control, node)| (control.0, rect(node)))
        .collect::<Vec<_>>();
    for expected in [
        (
            CharacterCreationControl::Gender(CharacterGender::Boy),
            LegacyCreationRect::new(520.0, 44.0, 72.0, 22.0),
        ),
        (
            CharacterCreationControl::Gender(CharacterGender::Girl),
            LegacyCreationRect::new(603.0, 44.0, 72.0, 22.0),
        ),
        (
            CharacterCreationControl::Step(AppearanceField::Height, -1),
            LegacyCreationRect::new(521.0, 75.0, 22.0, 22.0),
        ),
        (
            CharacterCreationControl::Step(AppearanceField::Height, 1),
            LegacyCreationRect::new(655.0, 75.0, 22.0, 22.0),
        ),
        (
            CharacterCreationControl::RandomAppearance,
            LegacyCreationRect::new(287.0, 49.0, 90.0, 20.0),
        ),
        (
            CharacterCreationControl::ContinueAppearance,
            LegacyCreationRect::new(84.0, 550.0, 295.0, 40.0),
        ),
        (
            CharacterCreationControl::Camera(CharacterCreationCameraAction::RotateLeft),
            LegacyCreationRect::new(87.0, 431.0, 60.0, 100.0),
        ),
        (
            CharacterCreationControl::Camera(CharacterCreationCameraAction::ZoomIn),
            LegacyCreationRect::new(195.0, 498.0, 38.0, 38.0),
        ),
        (
            CharacterCreationControl::NameMode(CharacterNameMode::Generated),
            LegacyCreationRect::new(90.0, 33.0, 231.0, 26.0),
        ),
        (
            CharacterCreationControl::NameMode(CharacterNameMode::Custom),
            LegacyCreationRect::new(354.0, 31.0, 231.0, 26.0),
        ),
        (
            CharacterCreationControl::NameScroll(CharacterNamePart::First, -1),
            LegacyCreationRect::new(120.0, 112.0, 147.0, 16.0),
        ),
        (
            CharacterCreationControl::NameScroll(CharacterNamePart::First, 1),
            LegacyCreationRect::new(120.0, 276.0, 147.0, 16.0),
        ),
        (
            CharacterCreationControl::RandomName,
            LegacyCreationRect::new(567.0, 192.0, 92.0, 20.0),
        ),
        (
            CharacterCreationControl::ContinueName,
            LegacyCreationRect::new(525.0, 322.0, 135.0, 29.0),
        ),
        (
            CharacterCreationControl::Exit,
            LegacyCreationRect::new(22.0, 600.0, 68.0, 28.0),
        ),
        (
            CharacterCreationControl::Exit,
            LegacyCreationRect::new(9.0, 323.0, 68.0, 29.0),
        ),
    ] {
        assert!(
            controls.contains(&expected),
            "missing clean Rect {expected:?}"
        );
    }
}

#[test]
fn repeat_camera_deltas_match_source_direction_and_rates() {
    let half_second = Duration::from_millis(500);
    assert_eq!(
        CharacterCreationCameraAction::RotateLeft.rotation_delta_degrees(half_second),
        50.0
    );
    assert_eq!(
        CharacterCreationCameraAction::RotateRight.rotation_delta_degrees(half_second),
        -50.0
    );
    assert_eq!(
        CharacterCreationCameraAction::ZoomIn.distance_delta(half_second),
        -0.2
    );
    assert_eq!(
        CharacterCreationCameraAction::ZoomOut.distance_delta(half_second),
        0.2
    );
}

#[test]
fn generated_name_composition_matches_legacy_middle_rules() {
    let names = names();
    let mut model = CharacterCreationUiModel::default();
    model.name_indices = [1, 1, 1];
    assert_eq!(
        model.generated_name(&names).unwrap(),
        GeneratedCharacterName {
            first: "Ace".to_owned(),
            last: "Bolt".to_owned(),
            first_index: 1,
            middle_index: 1,
            last_index: 1,
        }
    );
    model.name_indices = [1, 2, 2];
    assert_eq!(model.generated_name(&names).unwrap().last, "Darkrider");
    model.name_indices = [1, 3, 3];
    assert_eq!(model.generated_name(&names).unwrap().last, "Mega Storm");
}

#[test]
fn source_guiskin_borders_and_active_states_are_preserved() {
    assert_eq!(
        source_style_border(CHARACTER_CREATION_BACKGROUND_PATH),
        None
    );
    assert_eq!(source_style_border(CC_FULLSCREEN), None);
    assert_eq!(
        source_style_border(CCBG),
        Some(BorderRect {
            min_inset: Vec2::new(10.0, 0.0),
            max_inset: Vec2::new(10.0, 0.0)
        })
    );
    assert_eq!(source_style_border(CC_RIGHT_BG), Some(BorderRect::all(5.0)));
    assert_eq!(source_style_border(CC_IN_3_BG), Some(BorderRect::all(5.0)));
    assert_eq!(
        source_style_border(CC_IN_12_BG),
        Some(BorderRect {
            min_inset: Vec2::new(5.0, 5.0),
            max_inset: Vec2::new(40.0, 40.0)
        })
    );
    assert_eq!(
        source_style_border(CC_NAME_BUTTON),
        Some(BorderRect {
            min_inset: Vec2::new(35.0, 20.0),
            max_inset: Vec2::new(35.0, 0.0)
        })
    );
    assert_eq!(
        source_style_border(CHARACTER_CREATION_TEXT_FIELD_PATH),
        Some(BorderRect::all(4.0))
    );
    assert_eq!(
        source_button_states(CharacterCreationControl::ContinueAppearance),
        Some((CC_BLUE_BUTTON, CC_BLUE_BUTTON_OVER, CC_BLUE_BUTTON))
    );
    assert_eq!(
        source_button_states(CharacterCreationControl::Exit),
        Some((CC_RED_BUTTON, CC_RED_BUTTON_OVER, CC_RED_BUTTON))
    );
}

#[test]
fn toggle_hit_rect_textfield_and_color_draw_order_match_clean_on_gui() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins(NativeCharacterCreationUiPlugin);
    app.update();

    let assets = app.world().resource::<CharacterCreationAssets>().clone();
    let world = app.world_mut();
    let mut toggles = world.query::<(&CharacterCreationButton, &Node, &ImageNode, &Children)>();
    let toggle_rows = toggles
        .iter(world)
        .filter(|(button, _, _, _)| matches!(button.0, CharacterCreationControl::Gender(_)))
        .map(|(_, node, image, children)| (node.clone(), image.clone(), children[0]))
        .collect::<Vec<_>>();
    assert_eq!(toggle_rows.len(), 2);
    for (node, image, content) in toggle_rows {
        assert_eq!(node.width, px(72.0));
        assert_eq!(node.height, px(22.0));
        let NodeImageMode::Sliced(slicer) = image.image_mode else {
            panic!("clean Toggle must preserve its serialized border");
        };
        assert_eq!(
            slicer.border,
            BorderRect {
                min_inset: Vec2::new(30.0, 5.0),
                max_inset: Vec2::new(0.0, 5.0)
            }
        );
        assert_eq!(world.get::<Node>(content).unwrap().width, px(102.0));
        let text = world.get::<Children>(content).unwrap()[0];
        assert_eq!(
            world.get::<CharacterCreationTextStyle>(text),
            Some(&CharacterCreationTextStyle::Toggle)
        );
        assert!(world.get::<UiTextAutoFit>(text).is_some());
    }

    let mut controls = world.query::<(&CharacterCreationButton, &ImageNode)>();
    let field_image = controls
        .iter(world)
        .find_map(|(button, image)| {
            (button.0 == CharacterCreationControl::FocusCustomName).then_some(image.clone())
        })
        .expect("custom textField control");
    assert_eq!(
        field_image.image,
        assets.image(CHARACTER_CREATION_TEXT_FIELD_PATH)
    );
    let NodeImageMode::Sliced(field_slicer) = field_image.image_mode else {
        panic!("clean textField must use its 4px GUIStyle border");
    };
    assert_eq!(field_slicer.border, BorderRect::all(4.0));

    let appearance = world
        .query_filtered::<&Children, With<AppearanceRoot>>()
        .single(world)
        .unwrap()
        .to_vec();
    let mut selected_count = 0;
    for (position, entity) in appearance.iter().copied().enumerate() {
        if world.get::<SelectedColor>(entity).is_none() {
            continue;
        }
        selected_count += 1;
        let outline = appearance[position - 1];
        let inside = appearance[position + 1];
        assert_eq!(
            world.get::<ImageNode>(outline).unwrap().image,
            assets.image(CC_COLOR_OUTLINE)
        );
        assert_eq!(
            world.get::<ImageNode>(inside).unwrap().image,
            assets.image(CC_COLOR_INSIDE)
        );
        assert!(world.get::<Button>(inside).is_some());
    }
    assert_eq!(selected_count, 35);
}
