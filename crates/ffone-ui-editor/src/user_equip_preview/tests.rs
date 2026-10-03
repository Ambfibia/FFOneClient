use super::*;
use std::path::Path;

#[test]
fn user_equip_catalog_exposes_real_assets_and_complete_repeated_content() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let layout = root.join("assets/game/ui/en/user-equip/user-equip.ffui.json");
    let mut document = UiLayoutDocument::load(&layout).expect("production layout");
    enrich(&mut document);
    document.validate().expect("enriched editor layout");

    for form in [
        "inventory",
        "inventory_content",
        "item_popup",
        "item_popup_unequip",
        "item_popup_use",
        "item_popup_chest",
        "character_status",
        "equipment_strip",
        "character_panel",
        "nano_gallery",
        "nano_viewer",
        "help",
        "item_mode_overview",
    ] {
        assert!(document.form(form).is_some(), "missing form {form}");
        assert!(
            document.elements.iter().any(|element| element.form == form),
            "empty form {form}"
        );
    }
    assert_eq!(
        document
            .elements
            .iter()
            .filter(|element| element.id.starts_with("inventory_slot_"))
            .count(),
        50
    );
    assert_eq!(
        document
            .elements
            .iter()
            .filter(|element| element.id.starts_with("nano_slot_"))
            .count(),
        64
    );
    assert!(
        document
            .elements
            .iter()
            .all(|element| element.visual.is_some()),
        "every editor element needs an image, text, fill, or runtime placeholder"
    );
    for image in document.elements.iter().filter_map(|element| {
        element
            .visual
            .as_ref()
            .and_then(|visual| visual.image.as_ref())
    }) {
        assert!(
            !image.path.contains("ui-parity"),
            "a screenshot is not an element visual: {}",
            image.path
        );
        assert!(root.join(&image.path).is_file(), "missing {}", image.path);
    }
}
