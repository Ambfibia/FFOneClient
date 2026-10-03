use super::*;

#[test]
fn production_user_store_tree_attaches_key_first_localization_to_every_text() {
    let asset_root = tempfile::tempdir().expect("temporary asset root");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::asset::AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(UserStoreUiPlugin0104);
    app.update();

    let expected_fonts = {
        let assets = app.world().resource::<UserStoreUiAssets0104>();
        [assets.font.clone(), assets.body_font.clone()]
    };
    let world = app.world_mut();
    let total_text = world.query::<&Text>().iter(world).count();
    let mut localized_text = world.query::<(&Text, &LocalizedText, (&TextFont, &LineHeight))>();
    let rows = localized_text.iter(world).collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert_eq!(
        rows.len(),
        total_text,
        "a User Store Text entity bypassed LocalizedText"
    );
    for (text, localized, font) in rows {
        assert!(
            !localized.key.is_empty(),
            "empty localization key for {:?}",
            text.0
        );
        assert!(
            expected_fonts
                .iter()
                .any(|handle| font.0.font == bevy::text::FontSource::Handle(handle.clone())),
            "Text bypassed a validated replacement font"
        );
    }

    let mut popup_styles = world.query::<(
        &Text,
        &UserStorePopupTextStyle0104,
        (&TextFont, &LineHeight),
        &UiTextAutoFit,
    )>();
    let popup_rows = popup_styles.iter(world).collect::<Vec<_>>();
    assert_eq!(popup_rows.len(), 17);
    for (_, style, font, _) in popup_rows {
        let (expected_handle, expected_size, expected_line_height) = match style {
            UserStorePopupTextStyle0104::LabelUpperLeft
            | UserStorePopupTextStyle0104::LabelMiddleRight => (
                &expected_fonts[0],
                USER_STORE_JEFFE_12_FONT_SIZE,
                USER_STORE_JEFFE_12_LINE_HEIGHT,
            ),
            UserStorePopupTextStyle0104::CenterLabel => (
                &expected_fonts[1],
                USER_STORE_CHALET_SMALL_FONT_SIZE,
                USER_STORE_CHALET_SMALL_LINE_HEIGHT,
            ),
            UserStorePopupTextStyle0104::CalculatorButton => (
                &expected_fonts[0],
                USER_STORE_JEFFE_16_FONT_SIZE,
                USER_STORE_JEFFE_16_LINE_HEIGHT,
            ),
            UserStorePopupTextStyle0104::Button => (
                &expected_fonts[0],
                USER_STORE_JEFFE_14_FONT_SIZE,
                USER_STORE_JEFFE_14_LINE_HEIGHT,
            ),
        };
        assert_eq!(
            &font.0.font,
            &bevy::text::FontSource::Handle(expected_handle.clone())
        );
        assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), expected_size);
        assert_eq!((*font.1), LineHeight::Px(expected_line_height));
    }
}
