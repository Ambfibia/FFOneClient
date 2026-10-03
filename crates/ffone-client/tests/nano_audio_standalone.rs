//! Focused production-module tests, also runnable directly with rustc against
//! the development dependencies while another Cargo job owns the build lock.
#![allow(dead_code, unused_imports)]

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::*;

pub mod player_emote {
    use crate::tutorial_player_presentation::TutorialPlayerClip;
    pub use ffone_client::player_emote::*;
    use ffone_runtime_contracts::PlayerRigGender;
    pub(crate) struct PlayerEmoteEvents {
        pub end: f32,
        pub sounds: &'static [(f32, &'static str)],
    }
    mod events {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/player_emote/events.rs"
        ));
    }
    pub(crate) const fn emote_events(
        gender: PlayerRigGender,
        clip: TutorialPlayerClip,
    ) -> Option<PlayerEmoteEvents> {
        events::events(gender, clip)
    }
}

#[path = "../src/gameplay/gameplay_audio/mod.rs"]
pub mod gameplay_audio;
#[path = "../src/rendering/legacy_npc_nano_animation/mod.rs"]
pub mod legacy_npc_nano_animation;
#[path = "../src/tutorial_runtime/nano_gameplay/mod.rs"]
pub mod tutorial_nano_gameplay;

#[test]
fn production_character_consumers_open_table_routes_without_inventory_files() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = assets::AssetLocator::open(&root).unwrap();
    assert!(!root.join("_runtime/characters.json").exists());
    let portraits = gameplay_nano_portraits::GameplayNanoPortraitCatalog::open(&locator).unwrap();
    // Every installed Nano model has a gameplay identity; IDs 52 and 66
    // deliberately retain the same Van Kleiss model.
    assert_eq!(portraits.len(), 67);
    assert!(portraits.model_path(1).unwrap().contains("nano_buttercup"));
    let npcs = network_world_runtime::NetworkNpcVisualCatalog0104::open(&locator).unwrap();
    assert!(npcs.len() > 1000);
}
