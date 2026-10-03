use bevy::audio::{AudioSource, Decodable};
use ffone_client_foundation::semantic_audio::{NativeAudioCatalog, NativeAudioCategory};
use std::{fs, path::Path};

const REPAIRED_DIALOGUE: &[&str] = &[
    "belladonna_greeting01",
    "belladonna_greeting02",
    "belladonna_qgreeting",
    "belladonna_farewell01",
    "belladonna_farewell02",
    "belladonna_farewell03",
    "belladonna_goodluck01",
    "belladonna_goodluck02",
    "belladonna_goodluck03",
    "belladonna_nicejob01",
    "belladonna_nicejob02",
    "belladonna_nicejob03",
    "mvehicle_greeting01",
    "mvehicle_greeting02",
    "mvehicle_farewell01",
    "mvehicle_farewell02",
    "mvehicle_farewell03",
    "noodies_greeting01",
    "noodies_greeting02",
    "albedo_qgreeting",
    "albedo_goodluck01",
    "albedo_goodluck02",
    "albedo_goodluck03",
    "albedo_nicejob01",
    "albedo_nicejob02",
    "albedo_nicejob03",
    "ampfibian_greeting01",
    "ampfibian_greeting02",
    "ampfibian_qgreeting",
    "ampfibian_farewell01",
    "ampfibian_farewell02",
    "ampfibian_farewell03",
    "ampfibian_goodluck01",
    "ampfibian_goodluck02",
    "ampfibian_goodluck03",
    "ampfibian_nicejob01",
    "ampfibian_nicejob02",
    "ampfibian_nicejob03",
    "spidermonkey_qgreeting",
    "spidermonkey_goodluck02",
    "spidermonkey_goodluck03",
    "spidermonkey_nicejob01",
    "spidermonkey_nicejob02",
    "spidermonkey_nicejob03",
    "fvehicle_greeting01",
    "fvehicle_greeting02",
    "fvehicle_farewell01",
    "fvehicle_farewell02",
    "fvehicle_farewell03",
    "woosh_greeting01",
    "woosh_greeting02",
    "woosh_farewell01",
    "woosh_farewell02",
    "woosh_farewell03",
    "throm_greeting01",
    "throm_greeting02",
    "throm_qgreeting",
    "throm_farewell01",
    "throm_farewell02",
    "throm_farewell03",
    "throm_goodluck01",
    "throm_goodluck02",
    "throm_goodluck03",
    "throm_nicejob01",
    "throm_nicejob02",
    "throm_nicejob03",
    "f_banker1_greeting01",
    "f_banker1_greeting02",
    "f_banker1_farewell01",
    "f_banker1_farewell02",
    "f_banker1_farewell03",
    "kumari_greeting01",
    "kumari_greeting02",
    "kumari_farewell01",
    "kumari_farewell02",
    "kumari_farewell03",
    "accountant_greeting01",
    "accountant_greeting02",
    "accountant_farewell01",
    "accountant_farewell02",
    "accountant_farewell03",
    "carl_greeting01",
    "carl_greeting02",
    "carl_qgreeting",
    "carl_farewell01",
    "carl_farewell02",
    "carl_farewell03",
    "carl_goodluck01",
    "carl_goodluck02",
    "carl_goodluck03",
    "carl_nicejob01",
    "carl_nicejob02",
    "carl_nicejob03",
    "dracula_qgreeting",
    "puckerberry_greeting01",
    "puckerberry_greeting02",
    "puckerberry_qgreeting",
    "puckerberry_farewell01",
    "puckerberry_farewell02",
    "puckerberry_farewell03",
    "puckerberry_nicejob01",
    "puckerberry_nicejob02",
    "puckerberry_nicejob03",
    "jeff_greeting01",
    "jeff_greeting02",
    "jeff_qgreeting",
    "jeff_farewell01",
    "jeff_farewell02",
    "jeff_farewell03",
    "jeff_goodluck01",
    "jeff_goodluck02",
    "jeff_goodluck03",
    "jeff_nicejob01",
    "estrangelove_qgreeting",
    "estrangelove_goodluck01",
    "estrangelove_goodluck02",
    "estrangelove_goodluck03",
    "estrangelove_nicejob01",
    "estrangelove_nicejob02",
    "estrangelove_nicejob03",
    "fred_greeting01",
    "fred_greeting02",
    "fred_farewell01",
    "fred_farewell02",
    "fred_farewell03",
    "fred_nicejob01",
    "fred_nicejob02",
    "fred_nicejob03",
    "sourron_greeting01",
    "sourron_greeting02",
    "sourron_farewell01",
    "sourron_farewell02",
    "sourron_farewell03",
];

#[test]
fn published_npc_dialogue_resolves_and_decodes_in_both_languages() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    for name in REPAIRED_DIALOGUE {
        let candidates = catalog.by_true_name(name);
        assert_eq!(candidates.len(), 1, "{name}");
        let asset = candidates[0];
        assert_eq!(asset.category, NativeAudioCategory::Voice);
        let has_recorded_translation = name.starts_with("albedo_")
            || matches!(
                *name,
                "spidermonkey_qgreeting" | "spidermonkey_goodluck02" | "spidermonkey_goodluck03"
            );
        if has_recorded_translation {
            assert!(
                asset.locale_variants.contains_key("ru"),
                "missing Russian {name}"
            );
        }
        for locale in ["en", "ru"] {
            let path = catalog.path_for_locale(asset, locale).unwrap();
            let expected_locale = if asset.locale_variants.contains_key(locale) {
                locale
            } else {
                "en"
            };
            assert!(
                path.starts_with(&format!("audio/voice/{expected_locale}/")),
                "{name}: {path}"
            );
            let bytes = fs::read(root.join(path)).unwrap();
            let source = AudioSource {
                bytes: bytes.into(),
            };
            assert!(
                source
                    .decoder()
                    .fold(false, |audible, sample| audible | (sample != 0.0)),
                "silent {name} ({locale})"
            );
        }
    }
}
