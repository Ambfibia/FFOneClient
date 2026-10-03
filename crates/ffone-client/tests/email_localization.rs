use ffone_client::localization::{Localization, LocalizedText, LocalizedTextLimit};
use std::path::Path;

#[test]
fn email_subject_limit_preserves_localization_and_utf16_boundaries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let subject = LocalizedText::new(
        "content.tabledata.mission.mission_string.1450.str_name_string",
        "Don't Be a Drip",
    );
    for locale in ["en", "ru"] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        let full = localization.text(&language, &subject);
        let shortened = LocalizedTextLimit(24).apply(&full);
        assert!(shortened.encode_utf16().count() <= 24);
        assert!(!shortened.is_empty());
        if locale == "ru" {
            assert_ne!(full, subject.fallback);
        }
        assert_eq!(localization.text(&language, &subject), full);
    }
    assert_eq!(LocalizedTextLimit(5).apply("😀😀😀"), "😀...");
    assert_eq!(LocalizedTextLimit(4).apply("😀😀😀"), "...");
    assert_eq!(LocalizedTextLimit(0).apply("abc"), "");
    assert_eq!(LocalizedTextLimit(2).apply("abc"), "..");
    assert_eq!(LocalizedTextLimit(4).apply("😀😀"), "😀😀");
}
