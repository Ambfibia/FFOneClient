use super::*;

#[test]
fn user_equip_runtime_localization_is_key_complete_and_placeholder_safe() {
    let root = workspace_root();
    let runtime = root.join("assets/game/localization");
    let runtime_en = open_bundle(&runtime.join("en.json"), "en");
    let runtime_ru = open_bundle(&runtime.join("ru.json"), "ru");
    let station_source = include_str!("../../src/ui/user_equip/nano_station.rs");
    for (key, fallback) in [
        ("ui.nano_station.equip", "EQUIP"),
        ("ui.nano_station.unequip", "UNEQUIP"),
        ("ui.nano_station.slot", "NANO {slot}"),
    ] {
        assert!(station_source.contains(&format!("LocalizedText::new(\"{key}\", \"{fallback}\")")));
        assert_eq!(runtime_en.entries[key], fallback);
        assert!(!runtime_ru.entries[key].is_empty());
    }

    assert_eq!(
        runtime_en.entries.keys().collect::<BTreeSet<_>>(),
        runtime_ru.entries.keys().collect::<BTreeSet<_>>(),
        "EN/RU key parity drift"
    );

    for (key, en_value) in &runtime_en.entries {
        let ru_value = &runtime_ru.entries[key];
        assert_eq!(
            placeholders(en_value),
            placeholders(ru_value),
            "placeholder mismatch for {key}"
        );
    }

    let mut source_keys = inventory_keys_in_source(USER_EQUIP_SOURCE);
    source_keys.extend(inventory_keys_in_source(REDEEM_SOURCE));
    source_keys.extend(inventory_keys_in_source(MAIN_SOURCE));
    source_keys.extend(inventory_keys_in_source(APP_SOURCE));
    assert!(
        source_keys.contains("ui.inventory.popup.confirm_delete"),
        "main delete confirmation key is outside the audited set"
    );
    for required_prefix in [
        "ui.inventory.action.",
        "ui.inventory.help.",
        "ui.inventory.nano.",
        "ui.inventory.slot.",
        "ui.inventory.status.",
    ] {
        assert!(
            source_keys
                .iter()
                .any(|key| key.starts_with(required_prefix)),
            "production source has no {required_prefix} semantic keys"
        );
    }
    for key in source_keys {
        assert!(runtime_en.entries.contains_key(key), "EN is missing {key}");
        assert!(runtime_ru.entries.contains_key(key), "RU is missing {key}");
    }
}

#[test]
fn guide_status_uses_primary_icon_rect_and_localized_mentor_identity() {
    assert!(USER_EQUIP_SOURCE.contains("USER_EQUIP_STATUS_GUIDE_ICON_RECT"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiRect::new(440.0, 533.0, 36.0, 36.0)"));
    assert!(APP_SOURCE.contains("GUIDE_MENTOR_NAME_LOCALIZATION_KEYS"));
    assert!(APP_SOURCE.contains("USER_EQUIP_GUIDE_EDD_PATH"));
    assert!(APP_SOURCE.contains("USER_EQUIP_GUIDE_COMPUTRESS_PATH"));
    assert!(APP_SOURCE.contains("GuideRawMentor::ComputressFuture"));
    assert!(APP_SOURCE.contains("ui.inventory.status.guide_computress"));
}
