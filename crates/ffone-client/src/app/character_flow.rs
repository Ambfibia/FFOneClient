//! Character selection, creation, preview, and shard-entry flow.

use crate::app::*;

mod constants;
mod projects;
mod assets;
mod types;
mod operations;
mod state;
mod commands;
mod systems;
mod input;

#[cfg(test)]
mod equipment_refresh_tests;

pub(super) use constants::CHARACTER_CREATION_PREWARM_BATCH_SIZE;
pub(super) use projects::CharacterCreationSession;
pub(super) use assets::{
    CharacterCreationAssetLease, CharacterEntryRoute, character_entry_route
};
pub(super) use types::{BufferedCharacterEntry, CharacterFlowPlugin};
use types::VendorTryOnOwners;
use operations::{
    character_summary_ui_slot, handle_character_creation_ui_actions, select_character_from_keyboard
};
pub(super) use operations::{
    resume_incomplete_character, accept_reserved_character_name,
    authoritative_user_equip_preview_character
};
pub(super) use state::{sync_character_selection_ui, handle_character_selection_ui_actions};
#[cfg(test)]
pub(super) use state::character_slots_from_runtime;
pub(super) use commands::request_character_entry;
use commands::submit_character_name_request;
use systems::{sync_character_creation_ui, sync_native_player_preview};
pub(super) use input::resolve_current_creator;
