use super::*;

#[test]
fn named_appearance_uses_gender_category_and_selector_keys() {
    for (gender, category, selector, source, expected) in [
        (
            CharacterGender::Boy,
            "hair",
            2,
            "RAZOR CUT",
            "content.appearance.male.hair.2.name",
        ),
        (
            CharacterGender::Girl,
            "face",
            3,
            "CAT EYE",
            "content.appearance.female.face.3.name",
        ),
    ] {
        let text = indexed_appearance_label(
            source,
            "HAIR ",
            "ui.character_create.hair_variant",
            "HAIR {index}",
            gender,
            selector,
            category,
        );
        assert_eq!(text.key, expected);
        assert_eq!(text.fallback, source);
        assert!(text.args.is_empty());
    }
}

pub(super) fn names() -> CharacterNameLists {
    CharacterNameLists {
        first: ["", "Ace", "Betty", "Cal"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        middle: ["", " ", "Dark", "Mega "]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        last: ["", "bolt", "Rider", "storm"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }
}

#[test]
fn every_character_creation_text_entity_has_semantic_ownership() {
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

    let mut texts = app.world_mut().query::<(&Text, Option<&LocalizedText>)>();
    let rows = texts.iter(app.world()).collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|(_, localized)| localized.is_some()));
}

#[test]
fn every_character_creation_text_uses_its_exact_style_and_replacement_font() {
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
    let total_texts = world.query::<&Text>().iter(world).count();
    let mut styled = world.query::<(
        Entity,
        &LocalizedText,
        &CharacterCreationTextStyle,
        (&TextFont, &LineHeight),
        &TextColor,
        &TextLayout,
        &ChildOf,
    )>();
    let rows = styled
        .iter(world)
        .map(|(entity, localized, style, font, color, layout, parent)| {
            (
                entity,
                localized.clone(),
                *style,
                font.clone(),
                *color,
                layout.clone(),
                parent.parent(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        total_texts,
        "a character-creation Text bypassed the exact style contract"
    );

    let mut seen = [false; 16];
    for (_entity, localized, style, font, color, layout, parent) in rows {
        assert!(!localized.key.is_empty());
        let spec = style.spec();
        let expected_font = match spec.font_role {
            CharacterCreationFontRole::Jeffe => &assets.display_font,
            CharacterCreationFontRole::Chalet => &assets.font,
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
            CharacterCreationTextAnchor::MiddleLeft => Justify::Left,
            CharacterCreationTextAnchor::MiddleCenter => Justify::Center,
        };
        assert_eq!(layout.justify, expected_justify, "justify for {style:?}");
        assert_eq!(
            layout.linebreak,
            if spec.word_wrap {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            },
            "wrap for {style:?}"
        );
        assert_eq!(
            color,
            TextColor(Color::srgba(
                spec.normal_color[0],
                spec.normal_color[1],
                spec.normal_color[2],
                spec.normal_color[3],
            )),
            "normal text color for {style:?}"
        );

        let styled_node = world.get::<Node>(parent).unwrap();
        if style == CharacterCreationTextStyle::Toggle {
            assert_eq!(styled_node.width, px(102.0));
            assert_eq!(styled_node.height, px(22.0));
            assert_eq!(styled_node.padding, UiRect::all(px(0.0)));
        } else {
            let [left, right, top, bottom] = spec.padding;
            assert_eq!(
                styled_node.padding,
                UiRect::new(px(left), px(right), px(top), px(bottom)),
                "padding for {style:?}"
            );
        }
        assert_eq!(styled_node.align_items, AlignItems::Center);
        assert_eq!(
            styled_node.justify_content,
            match spec.anchor {
                CharacterCreationTextAnchor::MiddleLeft => JustifyContent::Start,
                CharacterCreationTextAnchor::MiddleCenter => JustifyContent::Center,
            }
        );
        assert_eq!(spec.content_offset, [0.0, 0.0]);
        assert_eq!(spec.y_offset, 0.0);

        seen[match style {
            CharacterCreationTextStyle::Label => 0,
            CharacterCreationTextStyle::Transparent => 1,
            CharacterCreationTextStyle::SectionLabel => 2,
            CharacterCreationTextStyle::Toggle => 3,
            CharacterCreationTextStyle::BodyText => 4,
            CharacterCreationTextStyle::Button => 5,
            CharacterCreationTextStyle::ButtonTabFont => 6,
            CharacterCreationTextStyle::ExitButton => 7,
            CharacterCreationTextStyle::TabButton => 8,
            CharacterCreationTextStyle::TabText => 9,
            CharacterCreationTextStyle::OrText => 10,
            CharacterCreationTextStyle::NameDisplay => 11,
            CharacterCreationTextStyle::Transparent4 => 12,
            CharacterCreationTextStyle::Transparent5 => 13,
            CharacterCreationTextStyle::CustomQuestion => 14,
            CharacterCreationTextStyle::TextField => 15,
        }] = true;
    }
    assert!(seen.into_iter().all(|value| value));
}

#[test]
fn clean_primary_ownership_depths_and_unreachable_branches_are_explicit() {
    assert_eq!(CHARACTER_CREATION_PRIMARY_MAIN_UNITY3D_BYTES, 7_000_415);
    assert_eq!(
        CHARACTER_CREATION_PRIMARY_MAIN_UNITY3D_SHA256,
        "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
    );
    assert_eq!(CHARACTER_CREATION_PRIMARY_RESOURCE_FILE_BYTES, 8_974_798);
    assert_eq!(
        CHARACTER_CREATION_PRIMARY_RESOURCE_FILE_SHA256,
        "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E"
    );
    assert_eq!(CHARACTER_CREATION_SKIN_PATH_ID, 1_382);
    assert_eq!(CHARACTER_CREATION_BSD_PATH_ID, 1_378);
    assert_eq!(CHARACTER_CREATION_APPEARANCE_GAME_OBJECT_PATH_ID, 1_281);
    assert_eq!(CHARACTER_CREATION_APPEARANCE_GUI_COMPONENT_PATH_ID, 1_538);
    assert_eq!(CHARACTER_CREATION_APPEARANCE_MODE_COMPONENT_PATH_ID, 1_539);
    assert_eq!(CHARACTER_CREATION_NAME_GAME_OBJECT_PATH_ID, 1_358);
    assert_eq!(CHARACTER_CREATION_NAME_GUI_COMPONENT_PATH_ID, 1_570);
    assert_eq!(CHARACTER_CREATION_NAME_MODE_COMPONENT_PATH_ID, 1_571);
    assert_eq!(CHARACTER_CREATION_NAME_SUBMIT_COMPONENT_PATH_ID, 1_572);
    assert_eq!(CHARACTER_CREATION_RENDER_CAMERA_GAME_OBJECT_PATH_ID, 1_280);
    assert_eq!(CHARACTER_CREATION_RENDER_CAMERA_COMPONENT_PATH_ID, 1_540);
    assert_eq!(CHARACTER_CREATION_SIMPLE_CAMERA_COMPONENT_PATH_ID, 1_541);
    assert_eq!(CHARACTER_CREATION_NAME_GUI_DEPTH, 8);
    assert_eq!(CHARACTER_CREATION_APPEARANCE_GUI_DEPTH, 10);
    assert!(!CHARACTER_CREATION_PRIMARY_KOREAN_CHECK_REACHABLE);
    assert!(!CHARACTER_CREATION_PRIMARY_CLASS_SELECTION_REACHABLE);
}

#[test]
fn appearance_continue_does_not_wait_for_the_gpu_preview() {
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
    let expected = {
        let mut model = app.world_mut().resource_mut::<CharacterCreationUiModel>();
        model.visible = true;
        model.screen = CharacterCreationScreen::Appearance;
        model.save_appearance = CharacterCreationCapability::Enabled;
        assert_ne!(model.preview, CharacterCreationPreviewStatus::Ready);
        model.appearance.clone()
    };
    let continue_button = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &CharacterCreationButton)>();
        query
            .iter(world)
            .find_map(|(entity, button)| {
                (button.0 == CharacterCreationControl::ContinueAppearance).then_some(entity)
            })
            .unwrap()
    };
    app.world_mut()
        .entity_mut(continue_button)
        .insert(Interaction::Pressed);
    app.update();

    assert_eq!(
        app.world_mut()
            .resource_mut::<CharacterCreationUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![CharacterCreationUiAction::ConfirmAppearance(expected)]
    );
}

#[test]
fn source_option_counts_and_wrapping_match_cn_gui_char_creation() {
    let counts = CharacterCreationOptionCounts::default();
    assert_eq!(
        counts.count(CharacterGender::Boy, AppearanceField::Hair),
        23
    );
    assert_eq!(
        counts.count(CharacterGender::Girl, AppearanceField::Shoes),
        27
    );
    let mut appearance = CharacterAppearance::default();
    appearance.height = 0;
    appearance.step(AppearanceField::Height, -1, counts);
    assert_eq!(appearance.height, 4);
    appearance.body = 2;
    appearance.step(AppearanceField::Body, 1, counts);
    assert_eq!(appearance.body, 0);
    appearance.hair = 2;
    appearance.step(AppearanceField::Hair, -1, counts);
    assert_eq!(appearance.hair, 24);
    appearance.hair = 24;
    appearance.set_gender(CharacterGender::Girl, counts);
    assert_eq!(appearance.hair, 22);
}

#[test]
fn protocol_color_indices_are_one_based() {
    let mut model = CharacterCreationUiModel::default();
    model.set_skin_color(11);
    model.set_hair_color(17);
    model.set_eye_color(4);
    assert_eq!(model.appearance.skin_color, 12);
    assert_eq!(model.appearance.hair_color, 18);
    assert_eq!(model.appearance.eye_color, 5);
    model.set_skin_color(12);
    assert_eq!(model.appearance.skin_color, 12);
}

#[test]
fn all_source_proven_character_creation_pngs_exist_with_exact_dimensions() {
    assert_eq!(CHARACTER_CREATION_IMAGE_SPECS.len(), 53);
    assert_eq!(CHARACTER_CREATION_SHARED_IMAGE_SPECS.len(), 8);
    assert_eq!(CHARACTER_CREATION_ENGINE_IMAGE_SPECS.len(), 1);
    for spec in CHARACTER_CREATION_IMAGE_SPECS
        .into_iter()
        .chain(CHARACTER_CREATION_SHARED_IMAGE_SPECS)
        .chain(CHARACTER_CREATION_ENGINE_IMAGE_SPECS)
    {
        let path = project_asset(spec.path);
        assert!(
            path.is_file(),
            "{} is missing from the native semantic tree",
            path.display()
        );
        assert_eq!(
            image::image_dimensions(&path).unwrap(),
            (spec.width, spec.height),
            "{} ({}) changed source dimensions",
            spec.true_name,
            path.display()
        );
    }
    let text_field_bytes = std::fs::read(project_asset(CHARACTER_CREATION_TEXT_FIELD_PATH))
        .expect("published textField background");
    assert_eq!(
        format!("{:X}", Sha256::digest(text_field_bytes)),
        CHARACTER_CREATION_TEXT_FIELD_PNG_SHA256
    );
    assert_eq!(CHARACTER_CREATION_TEXT_FIELD_SOURCE_FILE_ID, 1);
    assert_eq!(CHARACTER_CREATION_TEXT_FIELD_SOURCE_PATH_ID, 11_024);
}

#[test]
fn production_defers_creation_assets_until_post_login_phase() {
    let asset_root = project_asset("");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .insert_state(crate::ui_startup::NativeUiStartupPhase::Deferred)
        .add_plugins(NativeCharacterCreationUiPlugin);

    app.update();
    assert!(!app.world().contains_resource::<CharacterCreationAssets>());

    app.world_mut()
        .resource_mut::<NextState<crate::ui_startup::NativeUiStartupPhase>>()
        .set(crate::ui_startup::NativeUiStartupPhase::CharacterSelection);
    app.update();
    assert!(app.world().contains_resource::<CharacterCreationAssets>());
}
