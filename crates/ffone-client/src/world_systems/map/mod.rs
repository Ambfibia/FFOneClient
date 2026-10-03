//! Pure state and projection boundary for Retrobution's mode-15 world map.
//!
//! This module deliberately owns no Bevy entities, rendering, packet parsing,
//! or legacy container access. It preserves the audited legacy state machine
//! and exposes typed outbox/catalog boundaries for those runtime adapters.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bevy::{
    asset::LoadState,
    math::{Rect as BevyRect, Rot2},
    prelude::*,
    sprite::BorderRect,
    text::{LineBreak, LineHeight},
    window::PrimaryWindow,
};

use crate::localization::{
    LocalizationSet, LocalizedText, LocalizedTextCase, localized_tabledata_npc_name,
    localized_world_location_text,
};

use crate::ui_support::valid_ui_scale as valid_world_map_ui_scale;

use crate::ui_support::stretched_image as stretch_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod state;
mod constants;
mod assets;
mod textures;
mod types_world_map_presentation;
mod types_world_map_presentation_plugin;
mod codec;
mod systems;
mod commands;
mod validation;
mod models_world_map_model;
mod models_world_map_model_world_map_model;
mod operations;
mod projects;
mod animation;
mod entities;

pub use state::WORLD_MAP_MODE_ID;
use state::world_map_control_selected;
pub use constants::{
    WORLD_MAP_ESCAPE_KEY_ID, WORLD_MAP_TOGGLE_KEY_ID, WORLD_MAP_HELP_PAGE_ID,
    WORLD_MAP_NO_MAP_MESSAGE_ID, WORLD_MAP_EXTENT, WORLD_MAP_MARKER_SIZE,
    WORLD_MAP_SCREEN_RATIO, WORLD_MAP_UI_SCALE_REFERENCE_HEIGHT, WORLD_MAP_UI_SCALE_NUDGE,
    WORLD_MAP_BACKDROP_RECT, WORLD_MAP_WINDOW_RECT, WORLD_MAP_CLICK_RECT,
    WORLD_MAP_PICTURE_RECT, WORLD_MAP_CLOSE_RECT, WORLD_MAP_HELP_RECT,
    WORLD_MAP_MISSION_FINDER_RECT, WORLD_MAP_UP_RECT, WORLD_MAP_DOWN_RECT,
    WORLD_MAP_LEFT_RECT, WORLD_MAP_RIGHT_RECT, WORLD_MAP_LOCAL_VIEW_RECT,
    WORLD_MAP_WORLD_VIEW_RECT, WORLD_MAP_ZOOM_IN_RECT, WORLD_MAP_ZOOM_BAR_RECT,
    WORLD_MAP_ZOOM_OUT_RECT, WORLD_MAP_CURRENT_LOCATION_LABEL_RECT,
    WORLD_MAP_CURRENT_LOCATION_VALUE_RECT, WORLD_MAP_VIEW_FONT_SIZE,
    WORLD_MAP_VIEW_FONT_LINE_HEIGHT, WORLD_MAP_VIEW_TEXT_OFFSET_X,
    WORLD_MAP_VIEW_TEXT_OFFSET_Y, WORLD_MAP_LOCATION_LABEL_FONT_SIZE,
    WORLD_MAP_LOCATION_LABEL_FONT_LINE_HEIGHT, WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_X,
    WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_Y, WORLD_MAP_SMALL_CHALET_FONT_SIZE,
    WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT, WORLD_MAP_LOCATION_VALUE_TEXT_OFFSET_X,
    WORLD_MAP_TOOLTIP_TEXT_OFFSET_X, WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y,
    WORLD_MAP_TEXT_CONTENT_OFFSET_Y, WORLD_MAP_VIEW_TEXT_PADDING,
    WORLD_MAP_LOCATION_LABEL_PADDING, WORLD_MAP_LOCATION_VALUE_PADDING,
    WORLD_MAP_PAYZONE_PATHS, WORLD_MAP_FREEZONE_PATHS, WORLD_MAP_DARKLAND_PATHS,
    WORLD_MAP_MARKER_PATHS, WORLD_MAP_MARKER_SOURCE_DIMENSIONS,
    RETROBUTION_WORLD_MAP_MAIN_ARCHIVE_SHA256, RETROBUTION_WORLD_MAP_TUTORIAL_ARCHIVE_SHA256,
    RETROBUTION_WORLD_MAP_MAIN_ARCHIVE_BYTES, RETROBUTION_WORLD_MAP_TUTORIAL_ARCHIVE_BYTES,
    WORLD_MAP_CHALET_FONT_SHA256, WORLD_MAP_JEFFE_FONT_SHA256, WORLD_MAP_FILTERS
};
use constants::{
    WORLD_MAP_TYPE4_RANGE, WORLD_MAP_PAINT_TARGET_WEIGHT, WORLD_MAP_PAINT_DISPLAY_WEIGHT,
    WORLD_MAP_ARROW_UNITS_PER_SECOND, WORLD_MAP_LEGACY_DEGREES_DIVISOR,
    WORLD_MAP_VIEW_EDGE_EPSILON, FUTURE_ZONE_RECT, DARKLANDS_ZONE_RECT, TUTORIAL_ZONE_RECT,
    FUSION_LAIR_RECT
};
pub use assets::{
    WORLD_MAP_UI_Z_INDEX, WORLD_MAP_ASSET_ROOT, WORLD_MAP_BACKDROP_PATH,
    WORLD_MAP_NO_MAP_PATH, WORLD_MAP_MAP_BACK_PATH, WORLD_MAP_BLACK_PATH,
    WORLD_MAP_LOCATION_BACK_PATH, WORLD_MAP_TOOLTIP_BACK_PATH, WORLD_MAP_CLOSE_PATH,
    WORLD_MAP_CLOSE_HOVER_PATH, WORLD_MAP_HELP_PATH, WORLD_MAP_HELP_HOVER_PATH,
    WORLD_MAP_UP_PATH, WORLD_MAP_UP_HOVER_PATH, WORLD_MAP_DOWN_PATH,
    WORLD_MAP_DOWN_HOVER_PATH, WORLD_MAP_LEFT_PATH, WORLD_MAP_LEFT_HOVER_PATH,
    WORLD_MAP_RIGHT_PATH, WORLD_MAP_RIGHT_HOVER_PATH, WORLD_MAP_ZOOM_IN_PATH,
    WORLD_MAP_ZOOM_IN_HOVER_PATH, WORLD_MAP_ZOOM_OUT_PATH, WORLD_MAP_ZOOM_OUT_HOVER_PATH,
    WORLD_MAP_ZOOM_TICK_PATH, WORLD_MAP_ZOOM_TICK_HOVER_PATH,
    WORLD_MAP_ZOOM_TICK_SELECTED_PATH, WORLD_MAP_ZOOM_BAR_PATH, WORLD_MAP_VIEW_SELECTED_PATH,
    WORLD_MAP_VIEW_NORMAL_PATH, WORLD_MAP_LINE_PATH, WORLD_MAP_LINE_EFFECT_LARGE_PATH,
    WORLD_MAP_LINE_EFFECT_SMALL_PATH, WORLD_MAP_CHALET_FONT_PATH, WORLD_MAP_JEFFE_FONT_PATH,
    WORLD_MAP_SKIN_PATH_ID, WORLD_MAP_VIEW_FONT_PATH_ID,
    WORLD_MAP_LOCATION_LABEL_FONT_PATH_ID, WORLD_MAP_SMALL_CHALET_FONT_PATH_ID,
    WORLD_MAP_MARKER_SOURCE_PATH_IDS, WORLD_MAP_SEMANTIC_ASSET_FILES,
    WORLD_MAP_SEMANTIC_ASSET_BYTES, WORLD_MAP_SEMANTIC_ASSET_SET_SHA256, WorldMapAssetSource,
    world_map_presentation_asset_paths, WorldMapNpcCatalogEntry, WorldMapCatalogLookup,
    WorldMapCatalog, world_map_map_asset_path, WorldMapPresentationAssetStatus,
    world_map_route_segments
};
use assets::{update_world_map_asset_status, world_map_control_asset_path};
use textures::{
    WORLD_MAP_TYPE1_TEXTURE_WIDTH, WORLD_MAP_TYPE1_TEXTURE_HEIGHT, view_is_inside_texture
};
pub use textures::{
    WorldMapTextureProof, WORLD_MAP_TEXTURE_PROOFS, world_map_texture_dimensions
};
pub use types_world_map_presentation::{
    WorldMapUiPoint, WorldMapUiRect, WorldMapFontRole, WorldMapTextAnchor, WorldMapTextStyle,
    WorldMapTextStyleSpec, WorldMapPoint, WorldMapViewRect, WorldMapZone, WorldMapZoom,
    WorldMapPhase, WorldMapPlayer, WorldMapOpenContext, WorldMapOpenDisposition,
    WorldMapCloseInput, WorldMapInputGates, WorldMapZoomDirection, WorldMapPanDirection,
    WorldMapInputResult, WorldMapMissionAvailability, WorldMapNpcSource, WorldMapMarkerKind,
    WorldMapMarker, WorldMapPresentationControl, WorldMapPresentationHover,
    WorldMapPresentation, WorldMapTransportNode, WorldMapScaledGroup,
    WorldMapPresentationLayout, WorldMapPresentationRoot, WorldMapPresentationWindow
};
use types_world_map_presentation::{
    WorldMapNormalizedPoint, WorldMapZoneRect, WorldMapPresentationAssets
};
pub use types_world_map_presentation_plugin::{
    WorldMapPresentationMap, WorldMapPresentationMarker, WorldMapPresentationControlNode,
    WorldMapPresentationSet, WorldMapPresentationPlugin
};
use types_world_map_presentation_plugin::{
    WorldMapPresentationBackdrop, WorldMapPresentationNormalLayer, WorldMapPresentationNoMap,
    WorldMapPresentationBlack, WorldMapPresentationLine, WorldMapPresentationLineEffect,
    WorldMapPresentationMarkerLayer, WorldMapDecoration, WorldMapSpin, WorldMapPulse,
    WorldMapFilterVisual, WorldMapPresentationControlLabel, WorldMapPresentationZoomBar,
    WorldMapPresentationCurrentLabel, WorldMapPresentationCurrentValue,
    WorldMapPresentationTooltip, WorldMapPresentationTooltipText
};
pub use codec::{WORLD_MAP_FRAME_RECT, WORLD_MAP_FRAME_PATH};
use codec::WorldMapPresentationFrame;
pub use systems::WORLD_MAP_ZOOM_TICK_RECTS;
use systems::{
    sync_world_map_filter_visuals, advance_world_map_scan_phases, sync_world_map_presentation,
    sync_world_map_controls, sync_world_map_text, sync_world_map_markers
};
pub use commands::WorldMapOutboxEvent;
pub use validation::{WorldMapError, WorldMapPresentationError};
use validation::validate_motion;
pub use models_world_map_model::WorldMapModel;
use operations::{
    centered_and_clamped_view, clamp_after_legacy_pan, clamp_after_legacy_repeat,
    legacy_aspect_adjusted_rect, is_npc_outside_current_special_zone, animate_world_map_decorations,
    world_map_control_text, world_map_passthrough_text, presentation_node, presentation_text_node,
    valid_world_map_scan_phase, world_map_scan_effect_rect, world_map_source_rect,
    world_map_control_enabled
};
pub use operations::{
    world_map_filter_rect, world_map_control_rect, world_map_control_at, world_map_marker_at,
    clean_world_map_ui_scale, world_map_presentation_layout,
    world_map_presentation_layout_with_scale
};
use projects::project_marker_rect;
pub use animation::WorldMapScanAnimation;
use animation::advance_world_map_scan_animation;
use entities::spawn_world_map_presentation;
