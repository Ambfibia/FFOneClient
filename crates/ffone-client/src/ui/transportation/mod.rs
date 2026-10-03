//! Clean-Retrobution parity boundary for the legacy `TransportMode`.
//!
//! The original enum member is misspelled `eGameMode.Transpotation`; that
//! spelling is preserved as an explicit contract below.  This module projects
//! the clean TableData transportation rows into a native catalog, reproduces
//! the three `cnTrans` branches, and exposes typed effects/travel intents.
//! It deliberately never subtracts Taros and never changes player position:
//! those remain authoritative server/runtime responsibilities.

use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
};

use bevy::{
    asset::LoadState,
    input::mouse::MouseWheel,
    math::Rect as BevyRect,
    prelude::*,
    sprite::BorderRect,
    text::LineHeight,
    window::PrimaryWindow,
};
use ffone_protocol::{RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::{
    assets::{AssetLocator, TABLE_SET_PATH},
    localization::{Language, Localization, LocalizationSet, LocalizedText},
    world_map::{
        WORLD_MAP_BACKDROP_PATH, WORLD_MAP_CLOSE_HOVER_PATH, WORLD_MAP_CLOSE_PATH,
        WORLD_MAP_DARKLAND_PATHS, WORLD_MAP_FREEZONE_PATHS, WORLD_MAP_JEFFE_FONT_PATH,
        WORLD_MAP_LINE_EFFECT_LARGE_PATH, WORLD_MAP_LINE_EFFECT_SMALL_PATH, WORLD_MAP_LINE_PATH,
        WORLD_MAP_PAYZONE_PATHS,
    },
};

use crate::ui_support::stretched_image as transportation_stretch_image;

use crate::ui_support::sliced_image as transportation_sliced_image;

#[cfg(test)]
mod tests;

mod state;
mod constants;
mod codec;
mod assets_transportation_catalog;
mod layout;
mod interaction;
mod types;
mod textures;
mod operations;
mod validation;
mod containers;
mod audio;
mod commands;
mod models;
mod output;
mod animation;
mod view;
mod systems;

pub use state::{
    TRANSPOTATION_GAME_MODE_ID, TRANSPORTATION_MODE_NAME,
    TRANSPORTATION_NO_SELECTION_MESSAGE_ID
};
pub use constants::{
    TRANSPORTATION_ESCAPE_KEY_ID, TRANSPORTATION_WARP_DELAY_SECONDS,
    TRANSPORTATION_GAME_CONDITION_COOLDOWN, TRANSPORTATION_WARP_EFFECT_ID,
    TRANSPORTATION_ICON_CONTENT_PADDING, TRANSPORTATION_UNREGISTERED_MESSAGE_ID,
    TRANSPORTATION_INSUFFICIENT_TAROS_MESSAGE_ID, TRANSPORTATION_FAILURE_MESSAGE_ID_8,
    TRANSPORTATION_GENERIC_FAILURE_MESSAGE_ID, TRANSPORTATION_HAS_ZONE_TABS,
    TRANSPORTATION_TYPE1_CAMERA_BINDING_OWNED_EXTERNALLY, TRANSPORTATION_TRANSPORT_ICON_PATHS,
    RETROBUTION_TRANSPORTATION_MAIN_ARCHIVE_BYTES,
    RETROBUTION_TRANSPORTATION_MAIN_ARCHIVE_SHA256,
    RETROBUTION_TRANSPORTATION_TABLE_ARCHIVE_BYTES,
    RETROBUTION_TRANSPORTATION_TABLE_ARCHIVE_SHA256, TRANSPORTATION_JEFFE_14_FONT_SIZE,
    TRANSPORTATION_JEFFE_16_FONT_SIZE, TRANSPORTATION_BIGFONT14_REPLACEMENT_Y_OFFSET,
    TRANSPORTATION_BIGFONT16_REPLACEMENT_Y_OFFSET,
    TRANSPORTATION_RIGHT_TEXT_REPLACEMENT_Y_OFFSET, RETROBUTION_TRANSPORT_JEFFE_FONT_BLAKE3,
    RETROBUTION_TRANSPORT_CHALET_FONT_BLAKE3, RETROBUTION_TRANSPORT_TABLE_CANONICAL_SHA256,
    RETROBUTION_NATIVE_TABLE_SET_BYTES, RETROBUTION_NATIVE_TABLE_SET_SHA256,
    RETROBUTION_TRANSPORTATION_WARP_LOCATION_COUNT,
    RETROBUTION_TRANSPORTATION_BROOM_LOCATION_COUNT, RETROBUTION_TRANSPORTATION_ICON_COUNT,
    RETROBUTION_WORLD_NAME_REGION_COUNT, RETROBUTION_WORLD_NAME_REGIONS
};
use constants::{TABLE_SET_SCHEMA, CONSOLIDATED_TABLE};
pub use codec::{
    TRANSPORTATION_REQUEST_PACKET_ID, TRANSPORTATION_REGISTRATION_REQUEST_PACKET_ID,
    TRANSPORTATION_REGISTRATION_FAILURE_PACKET_ID,
    TRANSPORTATION_REGISTRATION_SUCCESS_PACKET_ID, TRANSPORTATION_SUCCESS_PACKET_ID,
    TRANSPORTATION_FAILURE_PACKET_ID, decode_transportation_registration_reply_0104,
    TransportationReplyCodecError0104, decode_transportation_warp_reply_0104
};
pub use assets_transportation_catalog::{
    TRANSPORTATION_UI_Z_INDEX, TRANSPORTATION_ROUTE_STRIDE,
    TRANSPORTATION_ROUTE_CONTENT_PADDING, TRANSPORTATION_ASSET_ROOT,
    TRANSPORTATION_LEFT_BACK_PATH, TRANSPORTATION_RIGHT_BACK_PATH,
    TRANSPORTATION_LEFT_BOX_PATH, TRANSPORTATION_ROUTE_ROW_PATH,
    TRANSPORTATION_ROUTE_SELECTED_PATH, TRANSPORTATION_ICON_BOX_PATH,
    TRANSPORTATION_MONKEY_PATH, TRANSPORTATION_BUBBLE_PATH, TRANSPORTATION_TAROS_PATH,
    TRANSPORTATION_REGISTERED_WARP_PATH, TRANSPORTATION_UNREGISTERED_WARP_PATH,
    TRANSPORTATION_SELECTED_WARP_PATH, TRANSPORTATION_REGISTERED_WYVERN_PATH,
    TRANSPORTATION_UNREGISTERED_WYVERN_PATH, TRANSPORTATION_SELECTED_WYVERN_PATH,
    TRANSPORTATION_START_LABEL_PATH, TRANSPORTATION_FALLBACK_ROUTE_PATH,
    RETROBUTION_TRANSPORTATION_TABLE_OBJECT_PATH_ID, RETROBUTION_WORLD_NAME_OBJECT_PATH_ID,
    RETROBUTION_TRANSPORT_MODE_GAME_OBJECT_PATH_ID, RETROBUTION_CN_TRANS_COMPONENT_PATH_ID,
    RETROBUTION_TRANSPORT_SKIN_PATH_ID, RETROBUTION_TRANSPORT_SKIN_DEFAULT_FONT_PATH_ID,
    RETROBUTION_TRANSPORT_BIGFONT14_PATH_ID, RETROBUTION_TRANSPORT_BIGFONT16_PATH_ID,
    RETROBUTION_TRANSPORTATION_ROUTE_COUNT, TRANSPORTATION_SEMANTIC_ASSET_FILES,
    TRANSPORTATION_SEMANTIC_ASSET_BYTES, TRANSPORTATION_SEMANTIC_ASSET_SET_SHA256,
    TransportationCatalogProvenance, TransportationRouteDefinition, TransportationCatalog,
    TransportationRoute, TransportationPresentationAssetStatus,
    TransportationPresentationRouteRow
};
use assets_transportation_catalog::{
    transportation_presentation_asset_paths, TransportationPresentationRouteLayer,
    TransportationPresentationDynamicRoute, transportation_route_name_text,
    transportation_route_region_text, update_transportation_asset_status
};
pub use layout::{
    TRANSPORTATION_WORLD_EXTENT, TransportationUiRect, TRANSPORTATION_BACKDROP_RECT,
    TRANSPORTATION_WINDOW_RECT, TRANSPORTATION_RIGHT_BACK_RECT, TRANSPORTATION_LEFT_BACK_RECT,
    TRANSPORTATION_LEFT_BOX_RECT, TRANSPORTATION_SELECT_RECT, TRANSPORTATION_MAP_RECT,
    TRANSPORTATION_CLOSE_RECT, TRANSPORTATION_GO_RECT, TRANSPORTATION_TURBO_TOGGLE_RECT,
    TRANSPORTATION_TURBO_BACKGROUND_RECT, TRANSPORTATION_TURBO_INTERACT_RECT,
    TRANSPORTATION_TURBO_LABEL_RECT, TRANSPORTATION_TITLE_RECT, TRANSPORTATION_SUBTITLE_RECT,
    TRANSPORTATION_CAMERA_RECT, TRANSPORTATION_BUBBLE_RECT, TRANSPORTATION_WHERE_TO_RECT,
    TRANSPORTATION_SCROLL_CONTENT_WIDTH, TRANSPORTATION_SCROLLBAR_RECT,
    TRANSPORTATION_SCROLL_UP_RECT, TRANSPORTATION_SCROLL_DOWN_RECT,
    TRANSPORTATION_SCROLL_THUMB_RECT, TRANSPORTATION_FUTURE_ZONE_RECT,
    TRANSPORTATION_DARK_ZONE_RECT, TRANSPORTATION_TUTORIAL_RECT,
    TRANSPORTATION_JEFFE_14_LINE_HEIGHT, TRANSPORTATION_JEFFE_16_LINE_HEIGHT
};
use layout::{transportation_marker_rect, apply_transportation_rect, transportation_source_rect};
pub use interaction::{
    TRANSPORTATION_SCROLL_VELOCITY_SOURCE, TRANSPORTATION_BLUE_BUTTON_PATH,
    TRANSPORTATION_BLUE_BUTTON_HOVER_PATH, TRANSPORTATION_SCROLL_UP_PATH,
    TRANSPORTATION_SCROLL_BAR_PATH, TRANSPORTATION_SCROLL_THUMB_PATH,
    TRANSPORTATION_SCROLL_DOWN_PATH, TRANSPORTATION_BUTTON_REPLACEMENT_Y_OFFSET,
    TransportationInputGates, TransportationInputResult, TransportationPresentationInput
};
use interaction::{
    TransportationPresentationScrollThumb, TransportationPresentationScrollbar,
    queue_transportation_scroll
};
pub use types::{
    TransportationUiPoint, TransportationUiTextStyle, TransportationRegionProof,
    TransportationLocation, TransportationWorldPoint, TransportationUnlocks,
    TransportationPlayerSnapshot, TransportationTarget, TransportationOpenContext,
    TransportationService, TransportationMap, TransportationPhase, TransportationFade,
    TransportationWarpReply0104, TransportationRegistrationReply0104,
    TransportationMarkerKind, TransportationMarkerProjection, TransportationPresentationRoot,
    TransportationPresentationWindow, TransportationPresentationMap,
    TransportationPresentationNpcCameraSlot, TransportationPresentationMarker,
    TransportationPresentationControlNode, TransportationPresentationControl,
    TransportationPresentationSet, TransportationUiPlugin
};
use types::{
    TransportationProjection, TransportationPresentationAssets,
    TransportationPresentationBackdrop, TransportationPresentationMonkey,
    TransportationPresentationTurboLayer, TransportationPresentationTitle,
    TransportationPresentationSubtitle, TransportationPresentationWhereTo,
    TransportationPresentationGoLabel, TransportationPresentationTurboLabel,
    TransportationPresentationMarkerLayer, TransportationPresentationDynamicMarker,
    TransportationPresentationLineEffect
};
pub use textures::{TransportationTextureProof, TRANSPORTATION_TEXTURE_PROOFS};
pub use operations::transportation_region_for_point;
use operations::{
    extract_locations, indexed_location, invalid, required_array, required_string,
    required_i32, transportation_target_view, transportation_node, transportation_text_node,
    transportation_empty_subtitle_text, transportation_subtitle_text,
    transportation_cost_text, queue_transportation_controls, queue_transportation_escape
};
pub use validation::TransportationCatalogError;
use validation::TransportationPresentationTurboCheck;
use containers::{value_object, required_object};
pub use audio::TransportationSound;
pub use commands::{
    TransportationCloseReason, TransportationOutboxEvent, TransportationRegistrationIntent,
    TransportationTravelIntent,
    TransportationUiCommand, TransportationUiCommandOutbox
};
pub use models::{TransportationModelError, TransportationModel};
pub use output::TransportationUiCopy;
use animation::{
    TransportationPresentationAnimation, advance_transportation_animation,
    sync_transportation_animation
};
use view::{spawn_transportation_presentation, spawn_transportation_route};
use systems::{
    sync_transportation_shell, sync_transportation_controls, sync_transportation_text,
    sync_transportation_routes, sync_transportation_markers
};
