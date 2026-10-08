//! Direct-rustc harness for login text ownership while unrelated client
//! modules remain in progress.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

pub use ffone_client::{localization, user_settings};

mod world_audio {
    #[derive(bevy::prelude::Resource, Default)]
    pub struct RetrobutionLoadingAudioState {
        pub active: bool,
    }
}

mod option_ui {
    use bevy::prelude::*;

    #[derive(Clone, Debug, Default)]
    pub struct DisplaySettings {
        pub scale_ui: bool,
    }

    #[derive(Clone, Debug, Default)]
    pub struct OptionSettings {
        pub display: DisplaySettings,
    }

    #[derive(Debug, Default, Resource)]
    pub struct OptionUiModel {
        pub visible: bool,
        pub persisted_options: OptionSettings,
    }
}

#[path = "../../src/ui/text_edit/mod.rs"]
mod text_edit;

#[path = "../../src/ui/login/mod.rs"]
mod login_ui;

#[path = "standalone_tests.rs"]
#[cfg(test)]
mod standalone_tests;
