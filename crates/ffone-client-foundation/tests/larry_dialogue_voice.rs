use std::{fs, path::Path};

use bevy::audio::{AudioSource, Decodable};
use ffone_client_foundation::semantic_audio::{NativeAudioCatalog, NativeAudioCategory};

#[test]
fn larry_dialogue_routes_resolve_recordings_in_both_locales() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let groups: &[&[&str]] = &[
        &["greeting01", "greeting02", "qgreeting"],
        &["farewell01", "farewell02"],
        &["farewell03"],
        &["goodluck01"],
        &["goodluck02", "goodluck03"],
        &["nicejob01"],
        &["nicejob02", "nicejob03"],
    ];
    for locale in ["en", "ru"] {
        for group in groups {
            let mut shared_recording = None;
            for cue in *group {
                let name = format!("larry_{cue}");
                let candidates = catalog.by_true_name(&name);
                assert_eq!(candidates.len(), 1, "{name}");
                let audio = candidates[0];
                assert_eq!(audio.category, NativeAudioCategory::Voice);
                assert_eq!(audio.owner, "larry");
                let path = catalog.path_for_locale(audio, locale).unwrap();
                assert!(path.starts_with(&format!("audio/voice/{locale}/larry/")));
                let bytes = fs::read(root.join(path)).unwrap();
                assert!(bytes.starts_with(b"OggS"));
                let source = AudioSource {
                    bytes: bytes.clone().into(),
                };
                assert!(
                    source
                        .decoder()
                        .fold(false, |audible, sample| { audible | (sample != 0.0) }),
                    "silent {locale} {name}"
                );
                if let Some(expected) = &shared_recording {
                    assert_eq!(&bytes, expected, "shared {locale} recording for {name}");
                } else {
                    shared_recording = Some(bytes);
                }
            }
        }
    }
}
