use super::*;
use user_store_ui::{UserStoreUiRect, user_store_layout_0104};

#[test]
fn cli_supports_every_mode_and_rejects_bad_outputs() {
    for mode in UserStorePreviewMode0104::ALL {
        let parsed = parse_cli([mode.as_str().into()]).expect("known mode");
        assert_eq!(parsed.mode, mode);
        assert_eq!(parsed.language, "en");
        assert_eq!(parsed.output, default_output(mode, "en"));
        let russian = parse_cli([mode.as_str().into(), "ru".into()]).expect("RU mode");
        assert_eq!(russian.language, "ru");
        assert_eq!(russian.output, default_output(mode, "ru"));
    }
    assert!(parse_cli(["unknown".into()]).is_err());
    assert!(parse_cli(["user-list".into(), "bad.jpg".into()]).is_err());
    assert!(parse_cli(["popup-price".into(), "de".into(), "x.png".into()]).is_err());
}

#[test]
fn preview_uses_exact_centered_geometry() {
    let layout = user_store_layout_0104(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT, 1.0, 0.0);
    assert_eq!(
        layout.store_panel,
        UserStoreUiRect::new(122.0, 21.0, 498.0, 638.0)
    );
    assert_eq!(
        layout.store_backplate,
        UserStoreUiRect::new(114.0, 14.0, 585.0, 653.0)
    );
}

#[test]
fn every_reused_preview_asset_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in USER_STORE_IMAGE_ASSET_PATHS
        .into_iter()
        .chain([USER_STORE_FONT_PATH, USER_STORE_BODY_FONT_PATH])
    {
        assert!(
            root.join(relative).is_file(),
            "missing {}",
            root.join(relative).display()
        );
    }
}
