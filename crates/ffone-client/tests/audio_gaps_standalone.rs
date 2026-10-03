//! Production audio routing checks without an audio device.
#![allow(dead_code, unused_imports)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::*;

#[path = "../src/world_systems/audio/mod.rs"]
pub mod world_audio;

#[test]
fn dialogue_resurrection_and_outgoing_chat_cues_resolve_to_existing_sfx() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = semantic_audio::NativeAudioCatalog::open(&root, false).unwrap();
    for cue in ["Open_Screen", "Resurrectm_Warp", "Outgoing_chat"] {
        let matches = catalog.by_true_name(cue);
        assert_eq!(matches.len(), 1, "{cue}");
        assert_eq!(matches[0].category, semantic_audio::NativeAudioCategory::Sfx);
        for locale in ["en", "ru"] {
            let path = catalog.path_for_locale(matches[0], locale).unwrap();
            assert!(root.join(path).is_file(), "{cue}: {path}");
        }
    }
}

#[test]
fn added_npc_voice_families_resolve_in_both_languages() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = semantic_audio::NativeAudioCatalog::open(&root, false).unwrap();
    for owner in ["fatima", "noonja", "m_barber", "warpfrog", "dustin"] {
        for (cue, count) in [("greeting", 2), ("farewell", 3)] {
            for take in 1..=count {
                let name = format!("{owner}_{cue}{take:02}");
                let matches = catalog.by_true_name(&name);
                assert_eq!(matches.len(), 1, "{name}");
                for locale in ["en", "ru"] {
                    let path = catalog.path_for_locale(matches[0], locale).expect(&name);
                    assert!(root.join(path).is_file(), "{locale}: {name}");
                    assert_eq!(matches[0].owner, owner);
                }
            }
        }
    }
}
