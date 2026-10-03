//! Standalone clean-Retrobution Player User Store acceptance boundary.
//!
//! Including the production source directly keeps its ABI, transaction,
//! geometry, interaction, key-first ECS and eleven-state renderer tests
//! independently executable when unrelated crate targets are under construction.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

pub use ffone_client::localization;

#[path = "../src/ui/user_store/mod.rs"]
mod user_store_ui;

#[test]
fn standalone_module_exposes_clean_mode_28_contract() {
    assert_eq!(user_store_ui::USER_STORE_GAME_MODE_SLOT, 28);
    assert_eq!(
        user_store_ui::UserStorePreviewMode0104::ALL.map(|mode| mode.as_str()),
        [
            "my-ready",
            "my-items",
            "my-open",
            "user-list",
            "user-sold",
            "popup-quantity",
            "popup-price",
            "popup-unregister",
            "popup-buy",
            "busy",
            "error",
        ]
    );
    assert_eq!(
        user_store_ui::UserStoreMode0104::ReturnStore.listing_slot_type(),
        None
    );
}
