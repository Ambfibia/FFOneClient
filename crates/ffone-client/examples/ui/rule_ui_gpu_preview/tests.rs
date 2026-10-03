use super::*;

#[test]
fn cli_routes_both_clean_pages_and_png_outputs() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            page: PreviewPage::Vehicle,
            output: PathBuf::from("target/ui-parity/rule-vehicle-1264x681.png"),
        }
    );
    assert_eq!(
        parse_cli(["combining".into(), "target/custom-rule.png".into()]).unwrap(),
        PreviewCli {
            page: PreviewPage::Combining,
            output: PathBuf::from("target/custom-rule.png"),
        }
    );
    assert_eq!(
        parse_cli(["target/vehicle.png".into()]).unwrap(),
        PreviewCli {
            page: PreviewPage::Vehicle,
            output: PathBuf::from("target/vehicle.png"),
        }
    );
    assert!(parse_cli(["unknown".into()]).is_err());
    assert!(parse_cli(["vehicle".into(), "bad.jpg".into()]).is_err());
}

#[test]
fn preview_frame_uses_exact_authority_geometry_page_and_hover_target() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(PREVIEW_HOVER, RuleUiButtonKind::Back);
    let layout = rule_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        clean_rule_ui_scale(CLIENT_AREA_HEIGHT as f32),
    );
    assert_eq!(layout.ui_scale, 1.0);
    assert_eq!(layout.fit_scale, Vec2::ONE);
    assert_eq!(layout.window.source.x, 122.0);
    assert_eq!(layout.window.source.y, 21.5);
    assert_eq!(layout.window.painted.width, 1_020.0);
    assert_eq!(layout.window.painted.height, 638.0);
    assert_eq!(PreviewPage::Vehicle.rule_page(), RulePageId::Vehicle);
    assert_eq!(PreviewPage::Combining.rule_page(), RulePageId::Combining);
}

#[test]
fn every_preview_image_and_font_asset_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in RULE_UI_IMAGE_PATHS
        .into_iter()
        .chain([RULE_UI_JEFFE_FONT_PATH, RULE_UI_CHALET_FONT_PATH])
    {
        assert!(
            root.join(relative).is_file(),
            "missing preview asset {}",
            root.join(relative).display()
        );
    }
}
