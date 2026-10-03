//! Direct-rustc Character Selection parity harness. It keeps this UI slice
//! testable while unrelated client modules are temporarily incomplete.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup, world_audio};

pub use ffone_client::localization;

pub mod gameplay_ui {
    pub const GAMEPLAY_UI_CAMERA_ORDER: isize = 100;
}

pub mod player_preview {
    pub const NATIVE_PLAYER_PREVIEW_CAMERA_ORDER: isize = 101;
}

pub mod character_selection_portraits {
    pub const CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE: isize = 102;
}

#[path = "../../src/ui/character_selection/mod.rs"]
pub mod character_selection_ui;

#[path = "standalone_tests.rs"]
#[cfg(test)]
mod standalone_tests;
