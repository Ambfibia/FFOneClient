//! Native clean-Retrobution system-message modal.
//!
//! The source component is `cnSystemMessageManager` on clean
//! `retrobution-20260613` `sharedassets0.assets` path ID 1468. Its
//! `MsgBoxList` is a LIFO stack: every entry paints another dialog frame at a
//! `(+10, +10)` offset, while only the newest entry receives content and
//! buttons. This module keeps that behavior behind a reusable resource and
//! outbox instead of retaining Unity `GameObject.SendMessage` callbacks.
//!
//! Legacy button types 4 (`eMsg_CurposRsrrt`) and 5
//! (`eMsg_YesNoCancel`) have no branch in the clean client's `DrawAll` switch.
//! Accepting either would create a modal that cannot be dismissed, so raw
//! ingestion rejects them. Failure types 10/11 are normalized to 1/2 exactly
//! as the clean `SendSystemMessageBox` and `ReceiveMessageBox` paths do.

use std::{collections::VecDeque, error::Error, fmt};

use crate::{
    gui_skin::{GuiColor, gui_style},
    localization::{Language, Localization, LocalizationSet, LocalizedText, UiTextAutoFit},
};
use bevy::{
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::ui_support::valid_ui_scale;

use crate::ui_support::sliced_image as sliced_button_image;

#[cfg(test)]
mod tests;

mod assets;
mod constants;
mod layout;
mod interaction;
mod types;
mod validation;
mod localization_system_message_button_localized;
mod commands;
mod audio;
mod models;
mod frame;
mod view;
use view::scroll_system_message_body;
mod operations;

pub use assets::{
    SYSTEM_MESSAGE_DIALOG_PATH, SYSTEM_MESSAGE_ITEM_BOX_PATH,
    SYSTEM_MESSAGE_WARNING_ICON_PATH, SYSTEM_MESSAGE_TRADE_ICON_PATH,
    SYSTEM_MESSAGE_BUDDY_ICON_PATH, SYSTEM_MESSAGE_GROUP_ICON_PATH,
    SYSTEM_MESSAGE_COMBI_ICON_PATH, SYSTEM_MESSAGE_COMBINED_BADGE_PATH,
    SYSTEM_MESSAGE_MANAGER_COMPONENT_PATH_ID, SYSTEM_MESSAGE_SKIN_PATH_ID,
    SYSTEM_MESSAGE_ITEM_BOX_PATH_ID, SYSTEM_MESSAGE_COMBINED_BADGE_PATH_ID,
    SYSTEM_MESSAGE_FONT_PATH, SYSTEM_MESSAGE_BODY_FONT_PATH, SYSTEM_MESSAGE_JEFFE_14_PATH_ID,
    SYSTEM_MESSAGE_JEFFE_12_PATH_ID, SYSTEM_MESSAGE_CHALET_SMALL_PATH_ID,
    SYSTEM_MESSAGE_UI_Z_INDEX, SystemMessageIconIndex, SystemMessageIconAssetContract,
    SYSTEM_MESSAGE_ICON_ASSET_CONTRACTS
};
pub use constants::{
    SYSTEM_MESSAGE_PRIMARY_MAIN_SIZE, SYSTEM_MESSAGE_PRIMARY_MAIN_SHA256,
    SYSTEM_MESSAGE_JEFFE_14_FONT_SIZE, SYSTEM_MESSAGE_JEFFE_12_FONT_SIZE,
    SYSTEM_MESSAGE_CHALET_SMALL_FONT_SIZE, SYSTEM_MESSAGE_OVERLAY_ALPHA,
    SYSTEM_MESSAGE_STACK_OFFSET, SYSTEM_MESSAGE_LABEL_COLOR, SYSTEM_MESSAGE_HEADER_COLOR,
    SYSTEM_MESSAGE_DELETE_SUBJECT_COLOR, SYSTEM_MESSAGE_IMAGE_WINDOW_COLOR
};
pub use layout::{
    SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT, SYSTEM_MESSAGE_JEFFE_12_LINE_HEIGHT,
    SYSTEM_MESSAGE_CHALET_SMALL_LINE_HEIGHT, SYSTEM_MESSAGE_SCALE_REFERENCE_HEIGHT,
    SYSTEM_MESSAGE_SCALE_FACTOR, SystemMessageUiRect, SYSTEM_MESSAGE_WINDOW_RECT,
    SYSTEM_MESSAGE_CONTENT_RECT, SYSTEM_MESSAGE_OK_RECT, SYSTEM_MESSAGE_CANCEL_RECT,
    SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT, SYSTEM_MESSAGE_ICON_RECT,
    SYSTEM_MESSAGE_ICON_QUANTITY_RECT, SYSTEM_MESSAGE_COMPARISON_ICON_RECTS,
    SYSTEM_MESSAGE_COMPARISON_BADGE_RECTS, SYSTEM_MESSAGE_DELETE_MISSION_CONTENT_RECT,
    SYSTEM_MESSAGE_COMBINATION_FAILURE_CONTENT_RECT, SYSTEM_MESSAGE_DIALOG_BORDER,
    SYSTEM_MESSAGE_BUTTON_BORDER, SystemMessageButtonLayout, system_message_button_layout,
    clean_system_message_ui_scale, SystemMessageLayerLayout, system_message_layer_layout,
    system_message_button_line_height
};
use layout::{layout_text_node, update_system_message_layout};
pub use interaction::{
    SYSTEM_MESSAGE_BLUE_BUTTON_PATH, SYSTEM_MESSAGE_BLUE_BUTTON_OVER_PATH,
    SYSTEM_MESSAGE_RED_BUTTON_PATH, SYSTEM_MESSAGE_RED_BUTTON_OVER_PATH,
    SYSTEM_MESSAGE_CANCEL_BUTTON_PATH, SystemMessageButtonType, SystemMessageButtonTypeError,
    SystemMessageButtonVisual, SystemMessageButtonSpec, system_message_button_font_size
};
use interaction::{
    button, SystemMessageButton, SystemMessageCursorLease, system_message_button_text_style,
    system_message_button_padding, handle_system_message_buttons,
    update_system_message_button_visuals, sync_system_message_cursor,
    system_message_button_text_color
};
pub use types::{
    SystemMessageFontRole, SystemMessageTextStyle, SystemMessageTextStyleSpec,
    SystemMessageChoice, SystemMessageUiOutbox, SystemMessageUiRoot, SystemMessageBodyLine,
    SystemMessageUiSet, SystemMessageUiPlugin
};
use types::{
    SystemMessageUiAssets, SystemMessageLayer, SystemMessageBody, SystemMessagePrimaryIcon,
    SystemMessageIconQuantity, SystemMessageComparisonIcon, SystemMessageCombinedBadge
};
pub use validation::SystemMessageIconIndexError;
pub use localization_system_message_button_localized::system_message_button_localized;
pub use commands::{SystemMessageRequest, SystemMessageUiAction};
pub use audio::{SystemMessageUiAudioCue, system_message_audio_cue, SystemMessageUiAudioOutbox};
pub use models::SystemMessageUiModel;
use frame::SystemMessageIconFrame;
use view::{spawn_system_message_ui, spawn_system_message_layer};
use operations::{
    rebuild_system_message_ui, style_padding, array_color,
    system_message_text_color, bevy_color
};
