use super::*;

#[test]
fn cli_defaults_to_clean_news_and_accepts_explicit_upgrade() {
    assert_eq!(
        parse_preview_args(Vec::<OsString>::new()).unwrap(),
        (
            PreviewMode::News,
            PathBuf::from(PreviewMode::News.default_output())
        )
    );
    assert_eq!(
        parse_preview_args([OsString::from("capture.png")]).unwrap(),
        (PreviewMode::News, PathBuf::from("capture.png"))
    );
    assert_eq!(
        parse_preview_args([OsString::from("upgrade"), OsString::from("upgrade.png")]).unwrap(),
        (PreviewMode::Upgrade, PathBuf::from("upgrade.png"))
    );
    assert!(parse_preview_args([OsString::from("bad.jpg")]).is_err());
}

#[test]
fn authority_frame_has_exact_news_and_level_four_geometry() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(clean_upsell_ui_scale(CLIENT_AREA_HEIGHT as f32), 1.0);

    let viewport = Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32);
    let news = upsell_ui_layout(viewport, 1.0, UpsellUiMode::NewsFreeZone);
    assert_eq!(
        news.dialog.source,
        UpsellUiRect::new(116.0, 15.0, 1_032.0, 650.0)
    );
    assert_eq!(
        expected_button_rect(PreviewMode::News, UpsellUiButtonKind::Close),
        Some(UpsellUiRect::new(987.0, 10.0, 31.0, 31.0))
    );
    assert_eq!(
        expected_button_rect(PreviewMode::News, UpsellUiButtonKind::Continue),
        Some(UpsellUiRect::new(10.0, 610.0, 132.0, 27.0))
    );
    assert_eq!(
        expected_button_rect(PreviewMode::News, UpsellUiButtonKind::GetUpgrade),
        None
    );

    let upgrade = upsell_ui_layout(viewport, 1.0, UpsellUiMode::Upgrade);
    assert_eq!(
        upgrade.dialog.source,
        UpsellUiRect::new(76.0, 20.0, 1_112.0, 641.0)
    );
    let geometry = upsell_upgrade_geometry(PREVIEW_LEVEL as u8).unwrap();
    assert_eq!(
        geometry.continue_playing,
        UpsellUiRect::new(55.0, 555.0, 288.0, 85.0)
    );
    assert_eq!(
        geometry.get_upgrade,
        UpsellUiRect::new(544.0, 520.0, 334.0, 105.0)
    );
    assert_eq!(
        geometry.not_right_now,
        UpsellUiRect::new(883.0, 597.0, 153.0, 26.0)
    );
    assert_eq!(EXPECTED_INPUT_BOUNDARY.blocks_lower_ui, true);
    assert_eq!(EXPECTED_INPUT_BOUNDARY.blocks_gameplay_input, true);
    assert_eq!(EXPECTED_INPUT_BOUNDARY.requires_pointer, true);
    assert_eq!(EXPECTED_INPUT_BOUNDARY.mouse_controls_enabled, true);
    assert_eq!(EXPECTED_INPUT_BOUNDARY.escape_dismiss_enabled, false);
}

#[test]
fn every_owned_png_font_and_action_success_asset_exists() {
    assert_eq!(PREVIEW_IMAGE_PATHS.len(), 16);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in PREVIEW_IMAGE_PATHS
        .into_iter()
        .chain([UPSELL_FONT_PATH, UPSELL_ACTION_SUCCESS_SOUND_PATH])
    {
        assert!(
            root.join(relative).is_file(),
            "missing Upsell preview asset {}",
            root.join(relative).display()
        );
    }
}
