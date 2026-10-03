use bevy::audio::{AudioSource, Decodable};
use ffone_client_foundation::semantic_audio::NativeAudioCatalog;
use std::{fs, path::Path};

#[test]
fn male_scamper_pilot_dialogue_uses_audible_native_files_in_both_locales() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    for pilot in [1, 2] {
        for (cue, count) in [("greeting", 2), ("farewell", 3), ("clickmove", 3)] {
            for take in 1..=count {
                let name = format!("m_kndtransport{pilot}_{cue}{take:02}");
                let matches = catalog.by_true_name(&name);
                assert_eq!(matches.len(), 1, "{name}");
                for locale in ["en", "ru"] {
                    // The approved departure translation supplies takes 02/03 only.
                    // A missing RU take must keep its number and fall back to EN.
                    let expected_locale = if cue == "clickmove" && take == 1 {
                        assert!(!matches[0].locale_variants.contains_key("ru"));
                        "en"
                    } else {
                        locale
                    };
                    let expected = format!(
                        "audio/voice/{expected_locale}/m_kndtransport{pilot}/kndtransport{pilot}_{cue}{take:02}.ogg"
                    );
                    assert_eq!(
                        catalog.path_for_locale(matches[0], locale),
                        Some(expected.as_str()),
                        "{name}: unexpected {locale} path or take substitution"
                    );
                    let source = AudioSource {
                        bytes: fs::read(root.join(expected)).unwrap().into(),
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
    }
}
