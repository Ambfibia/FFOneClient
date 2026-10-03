use std::{fs, path::Path};

use ffone_client_foundation::semantic_audio::{NativeAudioCatalog, NativeAudioCategory};

#[test]
fn player_and_mandroid_performances_use_localized_voice_routes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();

    let player_voice = catalog
        .assets()
        .iter()
        .filter(|audio| {
            let name = audio.true_name.to_ascii_lowercase();
            name.starts_with("f_avatar_") || name.starts_with("m_avatar_")
        })
        .collect::<Vec<_>>();
    assert_eq!(player_voice.len(), 162);
    assert!(
        player_voice
            .iter()
            .all(|audio| audio.category == NativeAudioCategory::Voice)
    );
    for audio in player_voice {
        let path = catalog.path_for_locale(audio, "ru").unwrap();
        assert!(
            path.starts_with(if audio.locale_variants.contains_key("ru") {
                "audio/voice/ru/"
            } else {
                "audio/voice/en/"
            }),
            "player voice must use the exact RU take or its EN fallback: {path}"
        );
        assert!(fs::read(root.join(path)).unwrap().starts_with(b"OggS"));
    }

    let mandroid_voice = catalog
        .assets()
        .iter()
        .filter(|audio| {
            matches!(
                audio.owner.as_str(),
                "m_mandrd1" | "m_mandrd2" | "m_mandrd3"
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(mandroid_voice.len(), 53);
    assert!(
        mandroid_voice
            .iter()
            .all(|audio| audio.category == NativeAudioCategory::Voice)
    );
    for audio in mandroid_voice {
        let path = catalog.path_for_locale(audio, "ru").unwrap();
        let is_untranslated_hurt = matches!(
            audio.true_name.as_str(),
            "M_Mandroid1_Hurt_1" | "M_Mandroid1_Hurt_2"
        );
        let expected_locale = if is_untranslated_hurt { "en" } else { "ru" };
        assert!(
            path.starts_with(&format!("audio/voice/{expected_locale}/")),
            "unexpected Mandroid locale route for {}: {path}",
            audio.true_name
        );
        assert!(fs::read(root.join(path)).unwrap().starts_with(b"OggS"));
    }

    for non_vocal_effect in ["Avatar_Flying", "Avatar_SwimBack", "Avatar_SwimIdle"] {
        let routes = catalog.by_true_name(non_vocal_effect);
        let [audio] = routes.as_slice() else {
            panic!("{non_vocal_effect} must retain one native route")
        };
        assert_eq!(audio.category, NativeAudioCategory::Sfx);
    }
}
