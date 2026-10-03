use super::*;

#[test]
fn exact_serialized_geometry_and_type1_aspect_rect_are_preserved() {
    assert_eq!(
        WORLD_MAP_WINDOW_RECT,
        WorldMapUiRect::new(0.0, 0.0, 1036.0, 654.0)
    );
    assert_eq!(
        WORLD_MAP_CLICK_RECT,
        WorldMapUiRect::new(70.0, 60.0, 870.0, 540.0)
    );
    assert_eq!(
        WORLD_MAP_PICTURE_RECT,
        WorldMapUiRect::new(20.0, 14.0, 950.0, 624.0)
    );
    assert_eq!(
        WORLD_MAP_ZOOM_TICK_RECTS,
        [
            WorldMapUiRect::new(28.0, 110.0, 20.0, 7.0),
            WorldMapUiRect::new(28.0, 100.0, 20.0, 7.0),
            WorldMapUiRect::new(28.0, 90.0, 20.0, 7.0),
        ]
    );

    let mut model = WorldMapModel::default();
    model
        .try_open(WorldMapOpenContext::gameplay(player(4096.0, 0.0, 4096.0)))
        .unwrap();
    let rect = model.map_draw_rect();
    assert_close(rect.x, 128.666_67);
    assert_close(rect.y, 14.0);
    assert_close(rect.width, 732.666_7);
    assert_close(rect.height, 624.0);

    assert_eq!(WorldMapZoom::Type1.range(WorldMapZone::Other), (1.0, 1.0));
    assert_close(
        WorldMapZoom::Type4.range(WorldMapZone::Other).1,
        WORLD_MAP_TYPE4_RANGE * WORLD_MAP_SCREEN_RATIO,
    );
}

#[test]
fn presentation_texts_are_all_key_first_and_preserve_serialized_style_metrics() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let (localization, language) = Localization::open(&asset_root, "en").unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .insert_resource(localization)
        .insert_resource(language)
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins(LocalizationPlugin)
        .add_plugins(WorldMapPresentationPlugin);
    app.init_asset::<Image>().init_asset::<Font>();
    app.world_mut()
        .resource_mut::<WorldMapPresentation>()
        .current_location = "POKEY OAKS NORTH".to_owned();
    app.update();

    let mut all_text =
        app.world_mut()
            .query::<(&Text, Option<&LocalizedText>, Option<&WorldMapTextStyle>)>();
    let all_text = all_text
        .iter(app.world())
        .map(|(_, localized, style)| (localized.is_some(), style.is_some(), style.copied()))
        .collect::<Vec<_>>();
    assert_eq!(all_text.len(), 6);
    assert!(
        all_text
            .iter()
            .all(|(localized, style, _)| *localized && *style)
    );

    let mut controls = app.world_mut().query::<(
        &WorldMapPresentationControlLabel,
        &Text,
        &LocalizedText,
        (&TextFont, &LineHeight),
        &Node,
        &TextLayout,
        &UiTransform,
    )>();
    let controls = controls.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 3);
    for (control, text, localized, font, node, layout, transform) in controls {
        let (key, fallback) = match control.0 {
            WorldMapPresentationControl::LocalView => ("ui.world_map.my_view", "MY VIEW"),
            WorldMapPresentationControl::WorldView => ("ui.world_map.world_view", "WORLD VIEW"),
            WorldMapPresentationControl::ShowFilters => {
                ("ui.world_map.show_filters", "SHOW FILTERS")
            }
            _ => panic!("unexpected text-bearing control"),
        };
        assert_eq!(text.as_str(), fallback);
        assert_eq!(localized.key, key);
        assert_eq!(localized.fallback, fallback);
        assert!(localized.args.is_empty());
        assert_eq!(
            font.0.font_size.eval(Vec2::ZERO, 16.0),
            WORLD_MAP_VIEW_FONT_SIZE
        );
        assert_eq!((*font.1), LineHeight::Px(WORLD_MAP_VIEW_FONT_LINE_HEIGHT));
        assert_eq!(node.justify_content, JustifyContent::Center);
        assert_eq!(node.align_items, AlignItems::FlexStart);
        assert_eq!(node.padding, UiRect::all(px(WORLD_MAP_VIEW_TEXT_PADDING)));
        assert_eq!(
            transform.translation,
            Val2::px(WORLD_MAP_VIEW_TEXT_OFFSET_X, WORLD_MAP_VIEW_TEXT_OFFSET_Y,)
        );
        assert_eq!(node.overflow, Overflow::clip());
        assert_eq!(layout.justify, Justify::Center);
        assert_eq!(layout.linebreak, LineBreak::WordBoundary);
    }

    let mut current_label = app.world_mut().query_filtered::<(
        &Text,
        &LocalizedText,
        &LocalizedTextCase,
        (&TextFont, &LineHeight),
        &Node,
        &TextLayout,
        &UiTransform,
    ), With<WorldMapPresentationCurrentLabel>>();
    let (text, localized, text_case, font, node, layout, transform) =
        current_label.single(app.world()).unwrap();
    assert_eq!(text.as_str(), "CURRENT LOCATION:");
    assert_eq!(localized.key, "ui.world_map.current_location");
    assert_eq!(localized.fallback, "current location:");
    assert!(localized.args.is_empty());
    assert_eq!(*text_case, LocalizedTextCase::Uppercase);
    assert_eq!(
        font.0.font_size.eval(Vec2::ZERO, 16.0),
        WORLD_MAP_LOCATION_LABEL_FONT_SIZE
    );
    assert_eq!(
        (*font.1),
        LineHeight::Px(WORLD_MAP_LOCATION_LABEL_FONT_LINE_HEIGHT)
    );
    assert_eq!(node.justify_content, JustifyContent::Center);
    assert_eq!(node.align_items, AlignItems::Center);
    assert_eq!(
        node.padding,
        UiRect::all(px(WORLD_MAP_LOCATION_LABEL_PADDING))
    );
    assert_eq!(
        transform.translation,
        Val2::px(
            WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_X,
            WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_Y,
        )
    );
    assert_eq!(layout.justify, Justify::Center);
    assert_eq!(layout.linebreak, LineBreak::WordBoundary);

    let mut current_value = app.world_mut().query_filtered::<(
        &Text,
        &LocalizedText,
        (&TextFont, &LineHeight),
        &Node,
        &TextLayout,
        &UiTransform,
    ), With<WorldMapPresentationCurrentValue>>();
    let (text, localized, font, node, layout, transform) =
        current_value.single(app.world()).unwrap();
    assert_eq!(text.as_str(), "Pokey Oaks North");
    assert_eq!(localized.key, "content.location.world.pokey_oaks_north");
    assert_eq!(localized.fallback, "POKEY OAKS NORTH");
    assert!(localized.args.is_empty());
    assert_eq!(
        font.0.font_size.eval(Vec2::ZERO, 16.0),
        WORLD_MAP_SMALL_CHALET_FONT_SIZE
    );
    assert_eq!(
        (*font.1),
        LineHeight::Px(WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT)
    );
    assert_eq!(node.justify_content, JustifyContent::FlexStart);
    assert_eq!(node.align_items, AlignItems::FlexStart);
    assert_eq!(
        node.padding,
        UiRect::all(px(WORLD_MAP_LOCATION_VALUE_PADDING))
    );
    assert_eq!(
        transform.translation,
        Val2::px(
            WORLD_MAP_LOCATION_VALUE_TEXT_OFFSET_X,
            WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y,
        )
    );
    assert_eq!(layout.justify, Justify::Left);
    assert_eq!(layout.linebreak, LineBreak::WordBoundary);

    let mut tooltip = app.world_mut().query_filtered::<(
        &LocalizedText,
        (&TextFont, &LineHeight),
        &Node,
        &TextLayout,
        &UiTransform,
    ), With<WorldMapPresentationTooltipText>>();
    let (localized, font, node, layout, transform) = tooltip.single(app.world()).unwrap();
    assert_eq!(localized.key, "ui.content.passthrough");
    assert_eq!(localized.fallback, "{text}");
    assert_eq!(localized.args.get("text").map(String::as_str), Some(""));
    assert_eq!(
        font.0.font_size.eval(Vec2::ZERO, 16.0),
        WORLD_MAP_SMALL_CHALET_FONT_SIZE
    );
    assert_eq!(
        (*font.1),
        LineHeight::Px(WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT)
    );
    assert_eq!(node.justify_content, JustifyContent::Center);
    assert_eq!(node.align_items, AlignItems::FlexStart);
    assert_eq!(node.padding, UiRect::ZERO);
    assert_eq!(
        transform.translation,
        Val2::px(
            WORLD_MAP_TOOLTIP_TEXT_OFFSET_X,
            WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y,
        )
    );
    assert_eq!(layout.justify, Justify::Center);
    assert_eq!(layout.linebreak, LineBreak::WordBoundary);

    assert_eq!(WORLD_MAP_SKIN_PATH_ID, 1_373);
    assert_eq!(WORLD_MAP_VIEW_FONT_PATH_ID, 1_119);
    assert_eq!(WORLD_MAP_LOCATION_LABEL_FONT_PATH_ID, 1_066);
    assert_eq!(WORLD_MAP_SMALL_CHALET_FONT_PATH_ID, 1_018);
    assert_eq!(WORLD_MAP_TEXT_CONTENT_OFFSET_Y, 0.0);

    let expected_styles = [
        (WorldMapTextStyle::ViewButton, 3usize),
        (WorldMapTextStyle::CurrentLocationLabel, 1),
        (WorldMapTextStyle::CurrentLocationValue, 1),
        (WorldMapTextStyle::Tooltip, 1),
    ];
    for (style, expected_count) in expected_styles {
        assert_eq!(
            all_text
                .iter()
                .filter(|(_, _, actual)| actual.is_some_and(|actual| actual == style))
                .count(),
            expected_count
        );
        let spec = style.spec();
        assert_eq!(spec.source_skin_path_id, WORLD_MAP_SKIN_PATH_ID);
        assert_eq!(
            spec.y_offset,
            match style {
                WorldMapTextStyle::ViewButton => WORLD_MAP_VIEW_TEXT_OFFSET_Y,
                WorldMapTextStyle::CurrentLocationLabel => {
                    WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_Y
                }
                WorldMapTextStyle::CurrentLocationValue | WorldMapTextStyle::Tooltip => {
                    WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y
                }
            }
        );
        assert_eq!(spec.linebreak, LineBreak::WordBoundary);
    }

    let view = WorldMapTextStyle::ViewButton.spec();
    assert_eq!(view.source_style, "view_but/view_off");
    assert_eq!(view.source_font_path_id, 1_119);
    assert_eq!(view.anchor, WorldMapTextAnchor::UpperCenter);
    assert_eq!(view.padding, [2.0; 4]);

    let label = WorldMapTextStyle::CurrentLocationLabel.spec();
    assert_eq!(label.source_style, "sfont");
    assert_eq!(label.source_font_path_id, 1_066);
    assert_eq!(label.font_role, WorldMapFontRole::Jeffe);
    assert_eq!(label.anchor, WorldMapTextAnchor::MiddleCenter);

    let value = WorldMapTextStyle::CurrentLocationValue.spec();
    assert_eq!(value.source_style, "gray");
    assert_eq!(value.source_font_path_id, 1_018);
    assert_eq!(value.anchor, WorldMapTextAnchor::UpperLeft);
    assert_eq!(value.padding, [2.0; 4]);

    let tooltip = WorldMapTextStyle::Tooltip.spec();
    assert_eq!(tooltip.source_style, "window");
    assert_eq!(tooltip.source_font_path_id, 1_018);
    assert_eq!(tooltip.anchor, WorldMapTextAnchor::UpperCenter);
    assert_eq!(tooltip.padding, [0.0; 4]);

    let production = concat!(
        include_str!("../state.rs"),
        "\n",
        include_str!("../constants.rs"),
        "\n",
        include_str!("../assets.rs"),
        "\n",
        include_str!("../textures.rs"),
        "\n",
        include_str!("../types_world_map_presentation.rs"),
        "\n",
        include_str!("../types_world_map_presentation_plugin.rs"),
        "\n",
        include_str!("../codec.rs"),
        "\n",
        include_str!("../systems.rs"),
        "\n",
        include_str!("../commands.rs"),
        "\n",
        include_str!("../validation.rs"),
        "\n",
        include_str!("../models_world_map_model.rs"),
        "\n",
        include_str!("../models_world_map_model_world_map_model.rs"),
        "\n",
        include_str!("../operations.rs"),
        "\n",
        include_str!("../projects.rs"),
        "\n",
        include_str!("../animation.rs"),
        "\n",
        include_str!("../entities.rs"),
        "\n",
        include_str!("../mod.rs")
    );
    let binder = production
        .split("fn sync_world_map_text")
        .nth(1)
        .and_then(|tail| tail.split("fn sync_world_map_markers").next())
        .expect("WorldMap text binder source");
    assert!(!binder.contains("&mut Text"));
}
