use super::*;

#[test]
fn cli_selects_scene_language_and_png_output() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            scene: PreviewScene::Player,
            language: "en".to_owned(),
            output: PathBuf::from("target/ui-parity/email-player-en-1264x681.png"),
        }
    );
    assert_eq!(
        parse_cli(["target/custom-email.png".into()]).unwrap(),
        PreviewCli {
            scene: PreviewScene::Player,
            language: "en".to_owned(),
            output: PathBuf::from("target/custom-email.png"),
        }
    );
    assert_eq!(
        parse_cli([
            "calculator".into(),
            "ru".into(),
            "target/calculator-ru.png".into(),
        ])
        .unwrap(),
        PreviewCli {
            scene: PreviewScene::Calculator,
            language: "ru".to_owned(),
            output: PathBuf::from("target/calculator-ru.png"),
        }
    );
    assert!(parse_cli(["target/email.jpg".into()]).is_err());
    assert!(parse_cli(["compose".into(), "de".into()]).is_err());
    assert!(parse_cli(["unknown".into()]).is_err());
}

#[test]
fn preview_frame_uses_exact_clean_integer_geometry_and_player_authority() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(PreviewScene::Player.hover(), EmailUiButtonKind::SendMail);
    assert_eq!(preview_summaries("en").len(), 5);
    assert_eq!(preview_summaries("ru").len(), 5);
    let layout = email_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        clean_email_ui_scale(CLIENT_AREA_HEIGHT as f32),
        1.0,
    );
    assert_eq!(layout.scale, Vec2::ONE);
    assert_eq!(layout.list_window.left, 122.0);
    assert_eq!(layout.list_window.top, 21.0);
    assert_eq!(layout.right_window.left, 707.0);
    assert_eq!(layout.right_window.top, 21.0);
}

#[test]
fn every_preview_image_and_font_asset_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in EMAIL_UI_IMAGE_PATHS
        .into_iter()
        .chain(PREVIEW_ICON_PATHS)
        .chain([EMAIL_UI_FONT_PATH, EMAIL_UI_BODY_FONT_PATH])
    {
        assert!(
            root.join(relative).is_file(),
            "missing preview asset {}",
            root.join(relative).display()
        );
    }
}
