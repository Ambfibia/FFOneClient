//! Clean-Retrobution `OptionMode` ownership slice.
//!
//! This module deliberately stops at the legacy UI boundary. It owns the
//! exact 1020x638 shell, all four reachable pages and the asymmetric Apply /
//! Save-and-exit / Cancel transaction. Renderer, mixer, input and persistence
//! changes leave this UI as typed adapter actions; the only eager adapter call
//! is the clean client's immediate sound preview/save behavior.

use std::collections::VecDeque;

use bevy::{
    asset::LoadState,
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::LineHeight,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use serde::{Deserialize, Serialize};

use crate::{
    localization::{Language, Localization, LocalizationSet, LocalizedText, VoiceLanguage},
    semantic_audio::NativeAudioCatalog,
};
use crate::ui_support::stretched_image;

#[cfg(test)]
mod tests;

mod layout;
mod assets;
mod frame;
mod interaction_input_settings;
mod interaction_handle_dropdown_interactions;
mod constants;
mod audio;
mod textures;
mod types_graphics_settings;
mod types_option_ui_plugin;
mod localization_option_localized_source_keys;
mod binding_controls_page;
mod binding_blocked_rows;
mod commands;
mod projection;
mod state;
mod models;
mod systems;
mod view_graphics_page;
mod view_controls_page;
mod view_social_page;
mod input_resolve_option_source;

pub use layout::{
    OPTION_REFERENCE_WIDTH, OPTION_REFERENCE_HEIGHT, OPTION_JEFFE_12_LINE_HEIGHT,
    OPTION_JEFFE_13_LINE_HEIGHT, OPTION_JEFFE_14_LINE_HEIGHT, OPTION_JEFFE_16_LINE_HEIGHT,
    OPTION_COMIC_LINE_HEIGHT, OPTION_CHALET_SMALL_LINE_HEIGHT, OPTION_CHALET_LINE_HEIGHT,
    OptionUiRect, OPTION_WINDOW_RECT, OPTION_BACKDROP_RECT, OPTION_PAGE_RECT,
    OPTION_CLOSE_RECT, OPTION_APPLY_RECT, OPTION_SAVE_RECT, OPTION_DEAD_RESET_RECT,
    OPTION_SOCIAL_REQUEST_HEADER_RECT, OPTION_SOCIAL_REQUEST_LEFT_RECT,
    OPTION_SOCIAL_REQUEST_CONNECTOR_RECT, OPTION_SOCIAL_REQUEST_RIGHT_RECT,
    OPTION_SOCIAL_BLOCKED_HEADER_RECT, OPTION_SOCIAL_BLOCKED_BODY_RECT,
    OPTION_SOCIAL_BLOCKED_LIST_RECT, OPTION_SOCIAL_UNIGNORE_RECT, OPTION_SOCIAL_DEFAULTS_RECT,
    OPTION_GRAPHICS_LEFT_HEADER_RECT, OPTION_GRAPHICS_RIGHT_HEADER_RECT,
    OPTION_GRAPHICS_LEFT_BODY_RECT, OPTION_GRAPHICS_LEFT_SIDE_RECT,
    OPTION_GRAPHICS_RIGHT_BODY_RECT, OPTION_GRAPHICS_RIGHT_SIDE_RECT,
    OPTION_DISPLAY_HEADER_RECT, OPTION_DISPLAY_BODY_RECT, OPTION_DISPLAY_SIDE_RECT,
    OPTION_CHAT_HEADER_RECT, OPTION_CHAT_BODY_RECT, OPTION_CHAT_SIDE_RECT,
    OPTION_CONTROLS_HEADER_RECT, OPTION_CONTROLS_INPUT_BODY_RECT,
    OPTION_CONTROLS_INPUT_SIDE_RECT, OPTION_KEYMAP_HEADER_RECT, OPTION_KEYMAP_VIEW_RECT,
    OPTION_KEYMAP_CONTENT_RECT, OPTION_CONTROL_CONTENT_HEIGHT, OPTION_BIG_LABEL_BORDER,
    OPTION_CLOSE_BORDER, OPTION_GRAPHICS_TAB_BORDER, OPTION_TOGGLE_BORDER,
    OPTION_DARK_BOX_BORDER, OPTION_PANEL_CONNECTOR_BORDER, OPTION_TEXT_FIELD_BORDER,
    OPTION_SCROLLBAR_BORDER, OPTION_SCROLL_THUMB_BORDER, OPTION_SCROLLBAR_VISUAL_WIDTH,
    OPTION_SCROLL_THUMB_VISUAL_WIDTH, OPTION_SLIDER_TRACK_WIDTH, OPTION_SLIDER_THUMB_WIDTH,
    OPTION_SLIDER_THUMB_HEIGHT, OptionUiLayout, clean_option_ui_scale, option_ui_layout,
    LegacyAxisDirection
};
use layout::update_option_layout;
pub use assets::{
    OPTION_UI_Z_INDEX, OPTION_NORMAL_TAB_Z_INDEX, OPTION_PAGE_Z_INDEX,
    OPTION_SELECTED_TAB_Z_INDEX, OPTION_CHROME_Z_INDEX, OPTION_DROPDOWN_PANEL_Z_INDEX,
    OPTION_JEFFE_FONT_PATH, OPTION_COMIC_FONT_PATH, OPTION_CHALET_FONT_PATH, OptionUiAssetGate
};
use assets::update_option_asset_gate;
pub use frame::{OPTION_FRAME_Z_INDEX, OPTION_FRAME_RECT, OPTION_FRAME_BORDER};
mod pad_controls;
use pad_controls::*;
pub use interaction_input_settings::{
    OPTION_DROPDOWN_BUTTON_Z_INDEX, OPTION_DROPDOWN_OPEN_BUTTON_Z_INDEX,
    OPTION_INPUT_ACTION_COUNT, OPTION_JEFFE_BUTTON_FONT_SIZE, OPTION_CONTROL_SCROLL_MAX,
    OPTION_SCROLL_THUMB_OVERFLOW, OptionInputMappingSlot, LegacyInputAxis, LegacyInputBinding,
    LegacyInputBindingSet, InputSettings, OptionTabButton, OptionChromeButton,
    OptionSocialFlagButton, OptionSocialDefaultsButton, OptionUnignoreButton,
    OptionBlockedScrollButton, OptionBlockedScrollThumb, OptionGraphicsDefaultsButton,
    OptionGraphicsToggleButton, OptionGraphicsSliderButton, OptionDropdownButton,
    OptionDisplayFlagButton, OptionDisplayDefaultsButton, OptionTextColorDefaultsButton,
    OptionTextColorButton, OptionInputDefaultsButton, OptionKeyMappingDefaultsButton,
    OptionInvertYButton, OptionSensitivityButton, OptionMappingButton,
    OptionControlsScrollButton, OptionControlsScrollContent, OptionControlsScrollThumb,
    OptionSystemPopupOkButton
};

use interaction_input_settings::{
    OptionBlockedScrollChrome, OptionPageButtonVisual, OptionButtonLabel,
    OptionKeyButtonLabel, option_button_node, tab_hover_role, capture_option_binding_input,
    handle_option_interactions, handle_graphics_interactions, handle_game_ui_interactions,
    handle_controls_interactions
};
use interaction_handle_dropdown_interactions::{
    handle_option_scroll_wheel, handle_dropdown_interactions,
    handle_system_popup_interactions, update_chrome_button_visuals,
    update_page_button_visuals, update_option_button_labels, update_key_button_labels,
    update_social_button_visuals
};
pub use constants::{
    OPTION_BLOCKED_SLOT_CAPACITY, OPTION_BLOCKED_VISIBLE_ROWS,
    OPTION_KEY_CAPTURE_TIMEOUT_SECONDS, OPTION_JEFFE_TAB_FONT_SIZE,
    OPTION_JEFFE_TITLE_FONT_SIZE, OPTION_CHALET_WHITE_LABEL_FONT_SIZE,
    OPTION_CHALET_WHITE_LABEL_TOP_OFFSET, OPTION_GRAPHICS_TAB_TEXT_INSET,
    OPTION_FONT_CONTRACTS, OPTION_CHAT_PALETTE_RGB, OPTION_RESOLUTION_CHOICES,
    OPTION_DROPDOWN_ITEM_OVERFLOW_LEFT, OPTION_DROPDOWN_ITEM_OVERFLOW_RIGHT,
    OPTION_MOVEMENT_ACTIONS, OPTION_INTERFACE_ACTIONS, OPTION_CAMERA_ACTIONS,
    OPTION_COMBAT_ACTIONS
};
pub use audio::{
    OPTION_OPEN_SOUND_PATH, OPTION_CLOSE_SOUND_PATH, OPTION_SUCCESS_SOUND_PATH,
    OPTION_BUTTON_SOUND_PATHS, OptionAudioContract, OPTION_AUDIO_CONTRACTS,
    SoundChannelSettings, SoundSettings, OptionSoundChannel, OptionOpenAudioRoute,
    OptionCloseAudioRoute, OptionUiAudioCue, OptionUiAudioRouting, OptionSoundDefaultsButton,
    OptionSoundToggleButton, OptionSoundSliderButton
};
use audio::{sound_channel_mut, sound_channel};
pub use textures::{
    OptionTextureRole, OptionTextureContract, OPTION_TEXTURE_CONTRACTS, TextureQuality
};
pub use types_graphics_settings::{
    OptionFontContract, OptionTab, GraphicsDetail, ShadowQuality, GraphicsSettings,
    DisplaySettings, TextColorSettings, OptionGraphicsToggle, OptionDisplayElement,
    OptionTextColorChannel, OptionControlGroup, LegacyPadProfile, LegacyPhysicalKey,
    OptionSettings, OptionBuddySlot, BlockedPlayerRow, OptionSystemPopup, OptionDropdownKind,
    OptionCloseTrigger, OptionUiOutbox, OptionUiRoot, OptionUiWindow, OptionUiBackdrop,
    OptionPage, OptionBlockedRow, OptionGraphicsSliderThumb, OptionDropdownPanel,
    OptionDropdownChoice, OptionSensitivityThumb, OptionSystemPopupRoot, OptionUiSet,
    OptionUiPlugin
};
use types_graphics_settings::{
    OptionUiAssets, OptionSkyHeader, OptionTabLabel, OptionBlockedRowLabel,
    OptionRadioIndicator, OptionRadioLabel, OptionDropdownValueLabel,
    OptionDropdownChoiceLabel, OptionDropdownChoiceBackground, OptionMappingLabel,
    OptionKeyCapturePrompt, OptionSystemPopupLabel
};
use types_option_ui_plugin::OptionTextAnchor;
pub use localization_option_localized_source_keys::{
    OPTION_TRANSLATION_RECT, OPTION_VOICE_LANGUAGE_RECT, OPTION_LANGUAGE_HEADER_RECT,
    OPTION_LANGUAGE_BODY_RECT
};
use localization_option_localized_source_keys::{
    OptionLanguageExtensionRoot, option_localized_text, dropdown_value_localized, locale_label
};
#[cfg(test)]
use localization_option_localized_source_keys::OPTION_LOCALIZED_SOURCE_KEYS;
pub use binding_controls_page::{
    legacy_raw_fifth_tab_reset_visible, legacy_binding_label, legacy_physical_key
};
use binding_controls_page::{
    binding_for_slot_mut, reset_pad_profile_bindings, option_ui_is_visible,
    option_passthrough_text, sliced_image, option_tab_image, option_tab_content_style,
    tab_normal_role, tab_label, bind_option_visibility, bind_option_pages, bind_option_radio_labels,
    bind_graphics_controls, bind_option_dropdowns, bind_game_ui_controls, bind_controls_page,
    bind_system_popup, pickable_for, dropdown_open, bind_social_flags
};
use binding_blocked_rows::bind_blocked_rows;
pub use commands::{
    SocialRequestSettings, LegacyOptionAction, OptionUiAction, OptionUiEvent,
    SocialRequestKind
};
pub use projection::project_blocked_players;
pub use state::{OptionModalState, KeyCaptureState};
use state::{
    OptionTextColorSelection, sliced_image_mode, option_tab_image_mode, tab_selected_role
};
pub use models::OptionUiModel;
use systems::{
    option_ui_visibility_needs_update, option_ui_bindings_need_update,
    option_ui_visuals_need_update, tick_option_key_capture, update_tab_visuals,
    update_dropdown_visuals
};
use view_graphics_page::{
    attach_option_localization, spawn_option_ui, spawn_dropdown_button, spawn_dropdown_panel
};
use view_controls_page::{spawn_game_ui_page, spawn_controls_page, spawn_language_extension};
use view_social_page::{
    spawn_system_popup, spawn_social_page, spawn_radio_button, spawn_option_anchored_text,
    spawn_text, spawn_footer_button, spawn_small_button, spawn_option_title, spawn_option_smallfont,
    spawn_option_smallcyan, spawn_option_bigfont14, spawn_option_default_label
};
use input_resolve_option_source::resolve_option_source;
