use super::*;

#[test]
fn cli_covers_all_clean_guide_states_both_locales_and_png_outputs() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            scene: PreviewScene::ChangeConfirmation,
            locale: "en".to_owned(),
            output: PathBuf::from("target/ui-parity/guide-confirm-change-en-1264x681.png"),
        }
    );
    assert_eq!(
        parse_cli([
            "confirm-initial".into(),
            "ru".into(),
            "target/guide-ru.png".into(),
        ])
        .unwrap()
        .output,
        PathBuf::from("target/guide-ru.png")
    );
    for scene in [
        "warp",
        "selection",
        "change",
        "confirm-initial",
        "confirm-change",
    ] {
        assert!(parse_cli([scene.into(), "en".into()]).is_ok());
    }
    assert!(parse_cli(["unknown".into(), "en".into()]).is_err());
    assert!(parse_cli(["warp".into(), "de".into()]).is_err());
    assert!(parse_cli(["warp".into(), "en".into(), "bad.jpg".into()]).is_err());
}

#[test]
fn every_preview_asset_exists_in_the_native_tree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let mut paths = GUIDE_PREVIEW_IMAGE_PATHS.to_vec();
    paths.extend([
        GUIDE_HELP_BUTTON_PATH,
        GUIDE_HELP_BUTTON_OVER_PATH,
        GUIDE_JEFFE_FONT_PATH,
        ffone_client::guide_ui::GUIDE_CHALET_FONT_PATH,
    ]);
    for mentor in GuideMentor::CLEAN_ORDER {
        paths.extend([
            mentor.portrait_path(),
            mentor.icon_path(),
            mentor.confirm_path(),
        ]);
    }
    for path in paths {
        assert!(
            root.join(path).is_file(),
            "missing Guide preview asset {path}"
        );
    }
}
