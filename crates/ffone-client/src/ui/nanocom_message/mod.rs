//! Reusable clean-Retrobution NanoCom message queue and presentation.
//!
//! This slice owns the legacy queue semantics shared by passive type-9 NPC
//! notices and the production-reached interactive type-13 buddy and type-14
//! group invitations. It deliberately stops at an outbox boundary:
//! protocol/network code can map request IDs to packets without making the UI
//! depend on the legacy client or a particular transport.

use std::collections::VecDeque;

use bevy::{
    audio::{PlaybackSettings, Volume},
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::{
    localization::{
        LocalizationSet, LocalizedText, LocalizedTextCase, LocalizedVoice, VoiceLanguage,
    },
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
};

use crate::ui_support::valid_ui_scale;

#[cfg(test)]
mod tests;

mod constants;
mod containers;
mod assets;
mod layout;
mod frame;
mod interaction;
mod localization_nanocom_content_localization_key;
mod commands;
mod types;
mod operations;
mod textures;
mod audio_resolve_nanocom_voice;
mod models;
mod view;
mod systems;

pub use constants::{
    NANOCOM_SOURCE_BUILD, NANOCOM_SOURCE_MAIN_ARCHIVE, NANOCOM_SOURCE_MAIN_ARCHIVE_BYTES,
    NANOCOM_SOURCE_MAIN_ARCHIVE_SHA256, NANOCOM_SOURCE_ASSEMBLY,
    NANOCOM_SOURCE_ASSEMBLY_BYTES, NANOCOM_SOURCE_ASSEMBLY_SHA256, NANOCOM_SOURCE_UI_CLASS,
    NANOCOM_SOURCE_LOGIN_METHOD, NANOCOM_SOURCE_MISSION_METHOD, NANOCOM_SOURCE_ICONS_ARCHIVE,
    NANOCOM_SOURCE_ICONS_ARCHIVE_BYTES, NANOCOM_SOURCE_ICONS_ARCHIVE_SHA256,
    NANOCOM_COMPONENT_DUMP_SHA256, NANOCOM_HUD_SKIN_DUMP_SHA256, NANOCOM_GUI_DEPTH,
    NANOCOM_TYPE_9_LIFETIME_SECONDS, NANOCOM_NANO_LIFETIME_SECONDS,
    NANOCOM_BUDDY_LIFETIME_SECONDS, NANOCOM_REVEAL_SECONDS, NANOCOM_REVEALED_RIGHT_MARGIN,
    NANOCOM_COMPACT_TITLE, NANOCOM_EXPANDED_TITLE, NANOCOM_GROUP_COMPACT_TITLE,
    NANOCOM_GROUP_EXPANDED_TITLE, NANOCOM_NANO_MISSION_TITLE, NANOCOM_ACCEPT_LABEL,
    NANOCOM_DECLINE_LABEL, NANOCOM_JEFFE_14_FONT_SIZE, NANOCOM_CHALET_SMALL_FONT_SIZE,
    NANOCOM_MESSAGE_TITLE_FONT_SIZE, NANOCOM_COMPACT_BODY_FALLBACK,
    NANOCOM_INVITATION_FALLBACK, NANOCOM_EXPIRATION_FALLBACK,
    NANOCOM_GROUP_COMPACT_BODY_FALLBACK, NANOCOM_GROUP_INVITATION_FALLBACK,
    NANOCOM_GROUP_EXPIRATION_FALLBACK, NANOCOM_GROUP_INVITATION_SUFFIX, NANOCOM_OVERLAY_ALPHA,
    NANOCOM_REACHED_TEXTURES
};
use constants::NANOCOM_CHAT_ECHO_LIMIT;
pub use containers::{
    NANOCOM_SOURCE_SERIALIZED_FILE, NANOCOM_GAME_OBJECT_DUMP_SHA256,
    NANOCOM_NETWORK_PREFAB_GUID
};
pub use assets::{
    NANOCOM_GAME_OBJECT_PATH_ID, NANOCOM_COMPONENT_PATH_ID, NANOCOM_SCRIPT_PATH_ID,
    NANOCOM_HUD_SKIN_PATH_ID, NANOCOM_BUDDY_ICON_PATH, NANOCOM_GROUP_ICON_PATH,
    NANOCOM_NUMBUH_TWO_ICON_PATH, NANOCOM_DIALOG_PATH, NANOCOM_MESSAGE_AREA_PATH,
    NANOCOM_JEFFE_FONT_PATH, NANOCOM_CHALET_FONT_PATH, NANOCOM_SLIDE_IN_PATH,
    NANOCOM_SLIDE_OUT_PATH, NANOCOM_NANO_CREATION_COMPLETE_PATH, NANOCOM_YES_PATH,
    NANOCOM_NO_PATH, NANOCOM_JEFFE_14_FONT_PATH_ID, NANOCOM_CHALET_SMALL_FONT_PATH_ID,
    NANOCOM_MESSAGE_TITLE_FONT_PATH_ID, NANOCOM_UI_Z_INDEX, NANOCOM_EXPANDED_Z_INDEX
};
pub use layout::{
    NANOCOM_MESSAGE_WIDTH, NANOCOM_MESSAGE_HEIGHT, NANOCOM_WINDOW_WIDTH,
    NANOCOM_SCALE_REFERENCE_HEIGHT, NANOCOM_SCALE_FACTOR, NANOCOM_JEFFE_14_LINE_HEIGHT,
    NANOCOM_CHALET_SMALL_LINE_HEIGHT, NANOCOM_MESSAGE_TITLE_LINE_HEIGHT, NanocomRect,
    NANOCOM_COMPACT_ICON_RECT, NANOCOM_NANO_ICON_RECT, NANOCOM_COMPACT_TITLE_RECT,
    NANOCOM_COMPACT_BODY_RECT, NANOCOM_COMPACT_BODY_CONTENT_RECT,
    NANOCOM_EXPANDED_DIALOG_RECT, NANOCOM_EXPANDED_MESSAGE_AREA_RECT,
    NANOCOM_EXPANDED_ICON_RECT, NANOCOM_EXPANDED_TITLE_RECT, NANOCOM_EXPANDED_TEXT_RECT,
    NANOCOM_EXPANDED_TEXT_CONTENT_RECT, NANOCOM_DECLINE_RECT, NANOCOM_ACCEPT_RECT,
    NANOCOM_DIALOG_BORDER, NANOCOM_MESSAGE_AREA_BORDER, NANOCOM_BUTTON_BORDER,
    clean_nanocom_ui_scale, NanocomCompactLayout, nanocom_compact_layout,
    NanocomExpandedLayout, nanocom_expanded_layout
};
pub use frame::{
    NANOCOM_BUDDY_FRAME_PATH, NANOCOM_TYPE_9_FRAME_PATH, NANOCOM_NANO_FRAME_PATH,
    NANOCOM_COMPACT_FRAME_RECT, NANOCOM_NANO_FRAME_RECT
};
pub use interaction::{
    NANOCOM_BLUE_BUTTON_PATH, NANOCOM_BLUE_BUTTON_OVER_PATH, NANOCOM_RED_BUTTON_PATH,
    NANOCOM_RED_BUTTON_OVER_PATH
};
use interaction::{
    NanocomMessageButton, NanocomMessageButtonLabel, nanocom_button_text_color,
    handle_nanocom_buttons, latch_nanocom_button_request_ids, update_nanocom_button_visuals
};
#[cfg(test)]
use interaction::nanocom_button_targets_presented_head;
pub use localization_nanocom_content_localization_key::{
    NANOCOM_CONTENT_LOCALIZATION_KEY, NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY,
    NANOCOM_COMPACT_BODY_LOCALIZATION_KEY, NANOCOM_INVITATION_LOCALIZATION_KEY,
    NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY, NANOCOM_EXPIRATION_LOCALIZATION_KEY,
    NANOCOM_ACCEPT_LOCALIZATION_KEY, NANOCOM_DECLINE_LOCALIZATION_KEY,
    NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY, NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY,
    NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY, NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY,
    NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY, NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY
};
use commands::NANOCOM_PASSIVE_REQUEST_ID_BASE;
pub use commands::{NanocomMessageRequest, NanocomMessageUiAction};
pub use types::{
    NanocomGuiInsets, NanocomGuiStyleRole, NanocomDrawRole, NanocomGuiStyleEvidence,
    NanocomMessageKind, QueuedNanocomMessage, NanocomMessageChoice, NanocomMessageResolution,
    NanocomMessageUiOutbox, NanocomChatEcho, NanocomMessageUiSet, NanocomMessageUiRoot,
    NanocomMessageCompactPanel, NanocomMessageExpandedRoot, NanocomMessageExpandedDialog,
    NanocomMessageUiPlugin
};
use types::{NanocomBindingKey, NanocomMessageElement, NanocomMessageUiAssets};
pub use operations::{
    nanocom_gui_style, compact_buddy_body, expanded_buddy_expiration,
    cleanup_nanocom_message_ui
};
use operations::{
    nanocom_comm_out_true_name, nanocom_passthrough, rounded_seconds, bind_nanocom_message_ui,
    play_nanocom_message_sounds
};
pub use textures::NanocomTextureEvidence;
pub use audio_resolve_nanocom_voice::NanocomMessageSound;
use audio_resolve_nanocom_voice::{NanocomMessageAudio, resolve_nanocom_voice};
pub use models::NanocomMessageUiModel;
use view::spawn_nanocom_message_ui;
use systems::tick_nanocom_messages;
