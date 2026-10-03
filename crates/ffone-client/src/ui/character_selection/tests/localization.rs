use super::*;

pub(super) fn insert_test_localization(app: &mut App, asset_root: &Path) {
    let (localization, language) =
        Localization::open(asset_root, "en").expect("test localization catalog must load");
    app.insert_resource(localization).insert_resource(language);
}

#[test]
fn slot_locations_follow_the_selected_text_language() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, en) = Localization::open(&root, "en").unwrap();
    let (_, ru) = Localization::open(&root, "ru").unwrap();
    let slot = |district: &str, zone: &str| OccupiedCharacterSlotUi {
        pc_uid: 1,
        display_name: "Ordinary".to_owned(),
        level: 1,
        district: district.to_owned(),
        zone: zone.to_owned(),
        background: CharacterLocationBackground::Future,
    };
    let label = |slot: &OccupiedCharacterSlotUi, language: &Language, selected: bool| {
        let text = slot.location_localized_in(selected, Some(&localization), Some(language));
        localization.text(language, &text)
    };
    let ordinary = slot("GENIUS GROVE", "THE FUTURE");
    assert_eq!(label(&ordinary, &en, true), "GENIUS GROVE - THE FUTURE");
    assert_eq!(label(&ordinary, &ru, true), "РОЩА ГЕНИЕВ - БУДУЩЕЕ");
    assert_eq!(label(&ordinary, &ru, false), "РОЩА ГЕНИЕВ - БУДУЩЕЕ");
    assert_eq!(
        label(&slot("CHARACTER CREATION", ""), &ru, false),
        "СОЗДАНИЕ ПЕРСОНАЖА"
    );
    assert_eq!(
        label(&slot("UNKNOWN", "UNKNOWN"), &ru, true),
        "НЕИЗВЕСТНО - НЕИЗВЕСТНО"
    );
    for area in
        crate::character_selection_ui::character_selection_world_names::LEGACY_WORLD_NAME_AREAS
    {
        for name in [area.district, area.zone] {
            let text = localized_world_location_text(name);
            assert_ne!(
                localization.text(&ru, &text),
                localization.text(&en, &text),
                "{name} needs a Russian worldname"
            );
        }
    }
}
