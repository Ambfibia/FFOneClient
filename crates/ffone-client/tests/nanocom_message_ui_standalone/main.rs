//! Direct-rustc NanoCom presentation harness. It keeps the clean primary
//! slice testable while unrelated production modules are changing.

#![allow(dead_code)]

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::localization;

pub mod semantic_audio {
    use std::path::Path;

    use bevy::prelude::Resource;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum NativeAudioCategory {
        Voice,
    }

    pub struct NativeAudioAsset {
        pub true_name: String,
        pub category: NativeAudioCategory,
        pub path: String,
    }

    #[derive(Default, Resource)]
    pub struct NativeAudioCatalog {
        assets: Vec<NativeAudioAsset>,
    }

    impl NativeAudioCatalog {
        pub fn open(_asset_root: impl AsRef<Path>, _verify_hashes: bool) -> Result<Self, String> {
            Ok(Self {
                assets: (1..=3)
                    .map(|take| NativeAudioAsset {
                        true_name: format!("Computress_CommOut0{take}"),
                        category: NativeAudioCategory::Voice,
                        path: format!("audio/voice/en/computress/computress_commout0{take}.ogg"),
                    })
                    .collect(),
            })
        }

        pub fn by_true_name(&self, true_name: &str) -> Vec<&NativeAudioAsset> {
            self.assets
                .iter()
                .filter(|asset| asset.true_name == true_name)
                .collect()
        }

        pub fn path_for_locale<'a>(
            &'a self,
            asset: &'a NativeAudioAsset,
            _locale: &str,
        ) -> Option<&'a str> {
            Some(asset.path.as_str())
        }
    }
}

#[path = "../../src/ui/startup/mod.rs"]
pub mod ui_startup;

#[path = "../../src/ui/nanocom_message/mod.rs"]
pub mod nanocom_message_ui;

#[path = "standalone_tests.rs"]
#[cfg(test)]
mod standalone_tests;
