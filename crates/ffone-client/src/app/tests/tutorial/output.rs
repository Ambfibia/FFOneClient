use super::*;

#[test]
fn dexter_ship_runtime_copy_is_keyed_and_argument_only() {
    let localized = dexter_ship_runtime_copy("Dexter: Welcome aboard.");
    assert_eq!(localized.key, "ui.content.passthrough");
    assert_eq!(localized.fallback, "{text}");
    assert_eq!(
        localized.args.get("text").map(String::as_str),
        Some("Dexter: Welcome aboard.")
    );
}
