use super::*;

#[test]
fn reached_text_styles_pin_the_exact_serialized_guiskin_metrics() {
    use CharacterSelectionFontRole::{Chalet, Jeffe};
    use CharacterSelectionTextAnchor::{MiddleCenter, MiddleLeft};

    let expected = [
        (
            CharacterSelectionTextStyle::Transparent2,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 903,
                font_role: Jeffe,
                font_size: 12.0,
                line_height: 13.710_000_04,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleCenter,
                normal_color: [0.0, 0.0, 0.405_109_5, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::Transparent3,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 903,
                font_role: Jeffe,
                font_size: 12.0,
                line_height: 13.710_000_04,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleCenter,
                normal_color: [0.8, 1.0, 1.0, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::CharNameUp,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 1_018,
                font_role: Chalet,
                font_size: 12.0,
                line_height: 12.071_999_55,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleLeft,
                normal_color: [0.898_039_2, 0.898_039_2, 0.898_039_2, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::CharNameDown,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 1_018,
                font_role: Chalet,
                font_size: 12.0,
                line_height: 12.071_999_55,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleLeft,
                normal_color: [0.0, 0.178_832_11, 0.558_394_13, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::CharLevelUp,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 1_018,
                font_role: Chalet,
                font_size: 12.0,
                line_height: 12.071_999_55,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleLeft,
                normal_color: [0.0, 0.8, 1.0, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::DeleteText,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 1_018,
                font_role: Chalet,
                font_size: 12.0,
                line_height: 12.071_999_55,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleCenter,
                normal_color: [0.8, 1.0, 1.0, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::AvatarName,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 1_115,
                font_role: Chalet,
                font_size: 14.0,
                line_height: 14.083_999_63,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleCenter,
                normal_color: [0.8, 1.0, 1.0, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::QuitButton,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 903,
                font_role: Jeffe,
                font_size: 12.0,
                line_height: 13.710_000_04,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleCenter,
                normal_color: [1.0; 4],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::CreateButton,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 903,
                font_role: Jeffe,
                font_size: 12.0,
                line_height: 13.710_000_04,
                padding: [10.0, 6.0, 4.0, 6.0],
                anchor: MiddleCenter,
                normal_color: [0.898_039_2, 0.898_039_2, 0.898_039_2, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::EnterGame,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 1_012,
                font_role: Jeffe,
                font_size: 14.0,
                line_height: 16.451_999_66,
                padding: [0.0; 4],
                anchor: MiddleCenter,
                normal_color: [0.8, 1.0, 1.0, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
        (
            CharacterSelectionTextStyle::Cancel,
            CharacterSelectionTextStyleSpec {
                source_font_path_id: 953,
                font_role: Jeffe,
                font_size: 10.0,
                line_height: 12.338_999_75,
                padding: [0.0, 0.0, 4.0, 7.0],
                anchor: MiddleCenter,
                normal_color: [0.8, 1.0, 1.0, 1.0],
                word_wrap: false,
                y_offset: 0.0,
            },
        ),
    ];
    for (style, spec) in expected {
        assert_eq!(style.spec(), spec, "{style:?}");
    }
}

#[test]
fn every_character_selection_text_uses_its_serialized_style_and_replacement_font() {
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
        .add_plugins(NativeCharacterSelectionUiPlugin);
    app.update();

    let assets = app.world().resource::<CharacterSelectionAssets>().clone();
    let world = app.world_mut();
    let total_texts = world.query::<&Text>().iter(world).count();
    let mut styled = world.query::<(
        Entity,
        &LocalizedText,
        &CharacterSelectionTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &ChildOf,
    )>();
    let rows = styled
        .iter(world)
        .map(|(entity, localized, style, font, layout, parent)| {
            (
                entity,
                localized.clone(),
                *style,
                font.clone(),
                layout.clone(),
                parent.parent(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        total_texts,
        "a character-selection Text bypassed the exact style contract"
    );
    assert!(!rows.is_empty());

    let mut seen = [false; 11];
    for (entity, localized, style, font, layout, parent) in rows {
        assert!(!localized.key.is_empty());
        let spec = style.spec();
        let expected_font = match spec.font_role {
            CharacterSelectionFontRole::Jeffe => &assets.jeffe_font,
            CharacterSelectionFontRole::Chalet => &assets.chalet_font,
        };
        assert_eq!(
            &font.0.font,
            &bevy::text::FontSource::Handle(expected_font.clone()),
            "font for {style:?}"
        );
        assert_eq!(
            font.0.font_size.eval(Vec2::ZERO, 16.0),
            spec.font_size,
            "font size for {style:?}"
        );
        assert_eq!(
            (*font.1),
            LineHeight::Px(spec.line_height),
            "line height for {style:?}"
        );
        let expected_justify = match spec.anchor {
            CharacterSelectionTextAnchor::MiddleLeft => Justify::Left,
            CharacterSelectionTextAnchor::MiddleCenter => Justify::Center,
        };
        assert_eq!(layout.justify, expected_justify, "justify for {style:?}");
        assert_eq!(layout.linebreak, LineBreak::NoWrap, "wrap for {style:?}");

        let own_node = world.get::<Node>(entity).unwrap();
        let styled_node = if own_node.position_type == PositionType::Absolute {
            own_node
        } else {
            world.get::<Node>(parent).unwrap()
        };
        let [left, right, top, bottom] = spec.padding;
        assert_eq!(
            styled_node.padding,
            UiRect::new(px(left), px(right), px(top), px(bottom)),
            "padding for {style:?}"
        );
        assert_eq!(styled_node.align_items, AlignItems::Center);
        assert_eq!(
            styled_node.justify_content,
            match spec.anchor {
                CharacterSelectionTextAnchor::MiddleLeft => JustifyContent::Start,
                CharacterSelectionTextAnchor::MiddleCenter => JustifyContent::Center,
            }
        );
        assert_eq!(spec.y_offset, 0.0, "source contentOffset.y for {style:?}");
        assert!(!spec.word_wrap, "source wordWrap for {style:?}");

        seen[match style {
            CharacterSelectionTextStyle::Transparent2 => 0,
            CharacterSelectionTextStyle::Transparent3 => 1,
            CharacterSelectionTextStyle::CharNameUp => 2,
            CharacterSelectionTextStyle::CharNameDown => 3,
            CharacterSelectionTextStyle::CharLevelUp => 4,
            CharacterSelectionTextStyle::DeleteText => 5,
            CharacterSelectionTextStyle::AvatarName => 6,
            CharacterSelectionTextStyle::QuitButton => 7,
            CharacterSelectionTextStyle::CreateButton => 8,
            CharacterSelectionTextStyle::EnterGame => 9,
            CharacterSelectionTextStyle::Cancel => 10,
        }] = true;
    }
    // CharNameDown is a runtime hover/selection state; the other ten
    // styles must all be present in the initially spawned clean tree.
    assert!(
        seen.into_iter()
            .enumerate()
            .all(|(index, value)| index == 3 || value)
    );
}
