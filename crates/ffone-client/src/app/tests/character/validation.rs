use super::*;

#[test]
fn login_external_link_failure_retains_semantic_template_and_error_arg() {
    let localized = localized_login_external_link_error("shell rejected URL");
    assert_eq!(localized.key, LOGIN_EXTERNAL_LINK_ERROR_KEY);
    assert_eq!(localized.fallback, LOGIN_EXTERNAL_LINK_ERROR_FALLBACK);
    assert_eq!(localized.args.len(), 1);
    assert_eq!(
        localized.args.get("error").map(String::as_str),
        Some("shell rejected URL")
    );

    let request = SystemMessageRequest::new_localized(
        LOGIN_EXTERNAL_LINK_ERROR_REQUEST_ID,
        localized.clone(),
        SystemMessageButtonType::Ok,
    );
    assert_eq!(request.localized, localized);
    assert_eq!(
        request.text,
        "Unable to open the community page.\nshell rejected URL"
    );
    assert_eq!(request.button_type, SystemMessageButtonType::Ok);

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let asset_root = root.join("assets/game");
    for (locale, template, resolved) in [
        (
            "en",
            "Unable to open the community page.\n{error}",
            "Unable to open the community page.\nshell rejected URL",
        ),
        (
            "ru",
            "Не удалось открыть страницу сообщества.\n{error}",
            "Не удалось открыть страницу сообщества.\nshell rejected URL",
        ),
    ] {
        let runtime = asset_root.join(format!("localization/{locale}.json"));
        let bundle: serde_json::Value =
            serde_json::from_slice(&fs::read(&runtime).unwrap()).unwrap();
        assert_eq!(
            bundle["entries"][LOGIN_EXTERNAL_LINK_ERROR_KEY].as_str(),
            Some(template)
        );

        let (localization, language) = Localization::open(&asset_root, locale).unwrap();
        assert_eq!(localization.text(&language, &localized), resolved);
    }
}
