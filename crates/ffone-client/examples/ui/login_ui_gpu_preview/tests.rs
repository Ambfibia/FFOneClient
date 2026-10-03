use super::*;

#[test]
fn cli_defaults_to_english_and_accepts_only_png_locale_pairs() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            output: PathBuf::from("target/ui-parity/login-manual-en-1264x681.png"),
            language: PreviewLanguage::En,
            stage: PreviewStage::Manual,
        }
    );
    assert_eq!(
        parse_cli(["warp".into(), "ru".into(), "login-warp-ru.png".into()]).unwrap(),
        PreviewCli {
            output: PathBuf::from("login-warp-ru.png"),
            language: PreviewLanguage::Ru,
            stage: PreviewStage::WarpShard,
        }
    );
    assert!(parse_cli(["missing-stage".into()]).is_err());
    assert!(parse_cli(["manual".into(), "de".into()]).is_err());
    assert!(parse_cli(["manual".into(), "en".into(), "bad.jpg".into()]).is_err());
    assert!(parse_cli(["auto".into(), "en".into(), "one.png".into(), "extra".into()]).is_err());
}

#[test]
fn preview_localization_has_exact_clean_english_and_russian_values() {
    for (key, en, ru) in [
        ("ui.login.username", "Username :", "ЛОГИН :"),
        ("ui.login.password", "Password :", "ПАРОЛЬ :"),
        ("ui.login.submit", "Log In", "ВОЙТИ"),
        (
            "ui.login.discord",
            "Discord Community",
            "СООБЩЕСТВО DISCORD",
        ),
        (
            "ui.login.register",
            "How Do I Register?",
            "КАК ЗАРЕГИСТРИРОВАТЬСЯ?",
        ),
    ] {
        let localized = LocalizedText::new(key, en);
        for (locale, expected) in [("en", en), ("ru", ru)] {
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
            let (catalog, language) = Localization::open(&root, locale).unwrap();
            assert_eq!(catalog.text(&language, &localized), expected);
        }
    }
}

#[test]
fn every_preview_asset_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in [
        LOGIN_BACKGROUND_PATH,
        LOGIN_FALLBACK_BACKGROUND_PATH,
        LOGIN_PANEL_PATH,
        LOGIN_BUTTON_PATH,
        LOGIN_BUTTON_OVER_PATH,
        LOGIN_BUTTON_ACTIVE_PATH,
        LOGIN_TEXT_FIELD_PATH,
        LOGIN_FONT_PATH,
        LOGIN_TEXT_FIELD_FONT_PATH,
        LOGIN_MUSIC_PATH,
    ] {
        assert!(
            root.join(relative).is_file(),
            "missing {}",
            root.join(relative).display()
        );
    }
}

#[test]
fn preview_viewport_matches_the_clean_acceptance_crop() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(LOGIN_PANEL_SIZE, Vec2::new(370.0, 275.0));
}
