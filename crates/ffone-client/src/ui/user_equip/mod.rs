//! Clean-Retrobution `UserEquip` Item-mode presentation and interaction shell.
//!
//! This module captures the deterministic parts of `mode.user_equip` without
//! taking ownership of protocol encoding or inventory mutation:
//! - `Panel_PCStuffScript`'s active `localized.newInv == Use` 5x10 inventory;
//! - `Panel_Equip`'s active `localized.quickSlot == None` fixed equip strip;
//! - the one-second `sin(pi / 2 * t)` panel entrance;
//! - `AvatarUtil.GetEquipIconElement`'s TableData routing and combined-look ID;
//! - `CnEquip`/`InventoryManagerScript` modal and close-request gates.
//!
//! `UserEquipItemModeProjection` is rebuilt only from
//! [`InventoryRuntime0104`]. Pointer gestures emit typed requests through
//! [`UserEquipUiOutbox`]; clean `UserSlot` updates only after authoritative
//! server replies, and the native runtime remains the sole 9+50 authority.
use bevy::asset::AssetLoader;
use bevy::asset::LoadContext;
use bevy::asset::io::Reader;
use ffone_ui_layout::UiLayoutDocument;
use ffone_ui_layout::UiLayoutError;

mod nano_station;

mod redeem;

pub use nano_station::{UserEquipNanoStationAction, nano_station_action_allowed};

use crate::localization::{LocalizationSet, LocalizedText};

use bevy::{prelude::*, sprite::BorderRect, text::LineHeight};

mod actions;
mod asset_contract;
mod asset_paths;
mod binding;
mod catalog;
mod components;
mod geometry;
mod images;
mod interaction;
mod item_popup;
mod item_projection;
mod layout;
mod nano_projection;
mod nano_viewer;
mod popup_state;
mod source_evidence;
mod spawn;
mod spawn_slots;
mod state;
mod view_model;
pub use actions::{
    UserEquipUiAction, UserEquipUiActionOutcome, UserEquipUiAudioCue, UserEquipUiAudioOutbox,
    UserEquipUiOutbox, apply_user_equip_ui_action,
};
pub use asset_contract::{
    USER_EQUIP_DEFAULT_ASSET_PATHS, UserEquipAssetContractError, UserEquipStaticAssetRole,
    UserEquipUiAssetContract, is_user_equip_semantic_icon_path,
};
pub use asset_paths::{
    USER_EQUIP_BACKDROP_PATH, USER_EQUIP_BOOST_ICON_PATH, USER_EQUIP_BUTTON_HOVER_PATH,
    USER_EQUIP_BUTTON_NORMAL_PATH, USER_EQUIP_CALCULATOR_BACK_PATH, USER_EQUIP_CLOSE_HOVER_PATH,
    USER_EQUIP_CLOSE_PATH, USER_EQUIP_CLOTHES_PANEL_PATH, USER_EQUIP_COMBINED_PATH,
    USER_EQUIP_DEXLABS_PATH, USER_EQUIP_EQUIP_INFO_PATH, USER_EQUIP_EQUIP_POPUP_PATH,
    USER_EQUIP_EQUIP_TITLE_PATH, USER_EQUIP_FONT_PATH, USER_EQUIP_FONT_SHA256,
    USER_EQUIP_FUSION_MATTER_BAR_PATH, USER_EQUIP_GENERAL_DIALOG_PATH,
    USER_EQUIP_GUIDE_BEN_TENNYSON_PATH, USER_EQUIP_GUIDE_BOX_PATH,
    USER_EQUIP_GUIDE_COMPUTRESS_PATH, USER_EQUIP_GUIDE_DEXTER_PATH, USER_EQUIP_GUIDE_EDD_PATH,
    USER_EQUIP_GUIDE_ICON_EVIDENCE, USER_EQUIP_GUIDE_MOJO_JOJO_PATH, USER_EQUIP_HELP_HOVER_PATH,
    USER_EQUIP_HELP_PATH, USER_EQUIP_HP_BACK_PATH, USER_EQUIP_HP_BAR_PATH,
    USER_EQUIP_INVENTORY_PANEL_PATH, USER_EQUIP_INVENTORY_SHADOW_PATH,
    USER_EQUIP_ITEM_TAB_HOVER_PATH, USER_EQUIP_ITEM_TAB_PATH, USER_EQUIP_NANO_BACK_PATH,
    USER_EQUIP_NANO_BLUE_PATH, USER_EQUIP_NANO_DIALOG_PATH, USER_EQUIP_NANO_FM_BAR_PATH,
    USER_EQUIP_NANO_ITEM_BAR_PATH, USER_EQUIP_NANO_POPUP_EQUIPPED_PATH,
    USER_EQUIP_NANO_POPUP_NEXT_PATH, USER_EQUIP_NANO_POPUP_PATH, USER_EQUIP_NANO_RED_PATH,
    USER_EQUIP_NANO_TAB_HOVER_PATH, USER_EQUIP_NANO_TAB_PATH, USER_EQUIP_NANO_YELLOW_PATH,
    USER_EQUIP_POTION_ICON_PATH, USER_EQUIP_RED_BUTTON_HOVER_PATH, USER_EQUIP_RED_BUTTON_PATH,
    USER_EQUIP_RIGHT_PANEL_PATH, USER_EQUIP_SCROLL_DOWN_PATH, USER_EQUIP_SCROLL_THUMB_PATH,
    USER_EQUIP_SCROLL_TRACK_PATH, USER_EQUIP_SCROLL_UP_PATH, USER_EQUIP_SLOT_EMPTY_PATH,
    USER_EQUIP_SLOT_OCCUPIED_PATH, USER_EQUIP_TAROS_PATH, USER_EQUIP_TRASH_HOVER_PATH,
    USER_EQUIP_TRASH_PATH, USER_EQUIP_TURN_LEFT_HOVER_PATH, USER_EQUIP_TURN_LEFT_PATH,
    USER_EQUIP_TURN_RIGHT_HOVER_PATH, USER_EQUIP_TURN_RIGHT_PATH, USER_EQUIP_UNEQUIP_POPUP_PATH,
    USER_EQUIP_USE_DIALOG_PATH, USER_EQUIP_USER_STATUS_PANEL_PATH,
};
use binding::{bind_item_card_text_case, bind_user_equip_ui};
pub use catalog::{
    USER_EQUIP_EQUIPMENT_STRIP_ORDER, UserEquipCatalogKind, UserEquipCatalogQuery,
    UserEquipCatalogQueryError, UserEquipEquipmentSlotKind, UserEquipEquipmentSlotSpec,
    UserEquipIconRef, UserEquipIconRefError, UserEquipItemCatalog, UserEquipItemIds,
    UserEquipMissingIconReason, UserEquipProjectedIcon, UserEquipProjectionSource,
    UserEquipSlotEndpoint,
};
use components::{UserEquipEditableLayoutHandle, UserEquipUiAssets, UserEquipUiRuntimeAssets};
pub use components::{UserEquipUiElement, UserEquipUiRoot, UserEquipUiSet};
use geometry::user_equip_nano_popup_content_rect;
pub use geometry::{
    USER_EQUIP_AVATAR_PREVIEW_RECT, USER_EQUIP_AVATAR_VEHICLE_Y_OFFSET, USER_EQUIP_BACKDROP_HEIGHT,
    USER_EQUIP_BACKDROP_WIDTH, USER_EQUIP_BACKPLATE_REFERENCE_HEIGHT,
    USER_EQUIP_BACKPLATE_REFERENCE_WIDTH, USER_EQUIP_BOOST_ICON_RECT, USER_EQUIP_BOOST_LABEL_RECT,
    USER_EQUIP_BOOST_RECT, USER_EQUIP_BOOST_VALUE_RECT, USER_EQUIP_CLOSE_RECT,
    USER_EQUIP_COMBINED_BADGE_LEFT, USER_EQUIP_COMBINED_BADGE_SIZE, USER_EQUIP_COMBINED_BADGE_TOP,
    USER_EQUIP_COUNT_FONT_SIZE, USER_EQUIP_COUNT_LABEL_LEFT, USER_EQUIP_COUNT_LABEL_TOP,
    USER_EQUIP_DEXLABS_RECT, USER_EQUIP_EQUIP_STRIP_FINAL_X, USER_EQUIP_EQUIP_STRIP_RECT,
    USER_EQUIP_EQUIPMENT_CONTENT_RECT, USER_EQUIP_EQUIPMENT_SLOT_SIZE,
    USER_EQUIP_EQUIPMENT_SLOT_STRIDE, USER_EQUIP_EQUIPMENT_STRIP_COUNT,
    USER_EQUIP_EQUIPMENT_TITLE_RECT, USER_EQUIP_GUM_NANO_BUTTON_RECTS,
    USER_EQUIP_GUM_NANO_FRAME_RECTS, USER_EQUIP_HELP_PANEL_RECT, USER_EQUIP_HELP_RECT,
    USER_EQUIP_INVENTORY_COLUMNS, USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
    USER_EQUIP_INVENTORY_CONTENT_WIDTH, USER_EQUIP_INVENTORY_PANEL_BORDER,
    USER_EQUIP_INVENTORY_ROWS, USER_EQUIP_INVENTORY_SCROLL_VELOCITY,
    USER_EQUIP_INVENTORY_SHADOW_A_RECT, USER_EQUIP_INVENTORY_SHADOW_B_RECT,
    USER_EQUIP_INVENTORY_SHADOW_BORDER, USER_EQUIP_INVENTORY_SLOT_GAP,
    USER_EQUIP_INVENTORY_SLOT_SIZE, USER_EQUIP_INVENTORY_SLOT_STRIDE,
    USER_EQUIP_INVENTORY_VIEWPORT_RECT, USER_EQUIP_ITEM_TAB_HIT_RECT,
    USER_EQUIP_ITEM_TAB_TEXTURE_RECT, USER_EQUIP_LEFT_PANEL_START_X,
    USER_EQUIP_MISSING_CHECKER_QUADRANT, USER_EQUIP_MISSING_CHECKER_SIZE,
    USER_EQUIP_MOUSE_SCROLL_AXIS_SENSITIVITY, USER_EQUIP_NANO_ATTRIBUTE_RECT,
    USER_EQUIP_NANO_COLUMNS, USER_EQUIP_NANO_CONTENT_HEIGHT, USER_EQUIP_NANO_GALLERY_COUNT,
    USER_EQUIP_NANO_NAME_RECT, USER_EQUIP_NANO_PANEL_BORDER, USER_EQUIP_NANO_PREVIEW_RECTS,
    USER_EQUIP_NANO_SKILL_RECT, USER_EQUIP_NANO_SLOT_LABEL_RECT, USER_EQUIP_NANO_SLOT_SIZE,
    USER_EQUIP_NANO_SLOT_STRIDE, USER_EQUIP_NANO_STAMINA_RECT, USER_EQUIP_NANO_STATUS_RECTS,
    USER_EQUIP_NANO_TAB_HIT_RECT, USER_EQUIP_NANO_TAB_TEXTURE_RECT, USER_EQUIP_NANO_TYPE_RECTS,
    USER_EQUIP_OPEN_SECONDS, USER_EQUIP_PC_STUFF_FINAL_X, USER_EQUIP_PC_STUFF_RECT,
    USER_EQUIP_POPUP_FONT_SIZE, USER_EQUIP_POPUP_RECT, USER_EQUIP_POTION_ICON_RECT,
    USER_EQUIP_POTION_LABEL_RECT, USER_EQUIP_POTION_RECT, USER_EQUIP_POTION_VALUE_RECT,
    USER_EQUIP_REFERENCE_HEIGHT, USER_EQUIP_REFERENCE_WIDTH, USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
    USER_EQUIP_RIGHT_PANEL_BORDER, USER_EQUIP_SCROLL_DOWN_RECT, USER_EQUIP_SCROLL_TRACK_RECT,
    USER_EQUIP_SCROLL_UP_RECT, USER_EQUIP_SMALL_FONT_LINE_HEIGHT, USER_EQUIP_SMALL_FONT_SIZE,
    USER_EQUIP_STATUS_FONT_SIZE, USER_EQUIP_STATUS_FUSION_BACK_RECT,
    USER_EQUIP_STATUS_FUSION_BAR_RECT, USER_EQUIP_STATUS_FUSION_RECT,
    USER_EQUIP_STATUS_GUIDE_ICON_RECT, USER_EQUIP_STATUS_GUIDE_LABEL_RECT,
    USER_EQUIP_STATUS_GUIDE_NAME_RECT, USER_EQUIP_STATUS_GUIDE_RECT,
    USER_EQUIP_STATUS_HP_BACK_RECT, USER_EQUIP_STATUS_HP_BAR_RECT, USER_EQUIP_STATUS_HP_RECT,
    USER_EQUIP_STATUS_LEVEL_RECT, USER_EQUIP_STATUS_NAME_RECT, USER_EQUIP_STATUS_PANEL_BORDER,
    USER_EQUIP_STATUS_PANEL_RECT, USER_EQUIP_TAB_FONT_SIZE, USER_EQUIP_TAROS_BACK_RECT,
    USER_EQUIP_TRASH_RECT, USER_EQUIP_TURN_LEFT_POSITIONED_RECT,
    USER_EQUIP_TURN_RIGHT_POSITIONED_RECT, USER_EQUIP_UI_Z_INDEX, USER_EQUIP_USER_CLOTHES_RECT,
    UserEquipUiRect, user_equip_taros_digit_rect, user_equip_taros_digits,
};
pub use images::user_equip_missing_checker_rgba;
use images::{bind_rect, sliced_image};
use interaction::{
    advance_user_equip_lifecycle, clear_closed_user_equip_selection,
    collect_user_equip_avatar_rotation, collect_user_equip_scrollbar_pointer,
    collect_user_equip_ui_actions,
};
use item_popup::bind_user_equip_popup_button_text_color;
pub use item_projection::{
    USER_EQUIP_RATING_BETTER_TINT, USER_EQUIP_RATING_WORSE_TINT, UserEquipEquipmentSlotProjection,
    UserEquipInventorySlotProjection, UserEquipItemModeProjection, UserEquipItemProjection,
    user_equip_destination_accepts_item, user_equip_item_rating_colors, user_equip_rating_colors,
};
pub use layout::{
    UserEquipItemModeLayout, UserEquipScrollbarMetrics, clamp_user_equip_scroll,
    user_equip_bevy_wheel_to_legacy_axis, user_equip_item_mode_layout, user_equip_nano_scroll_max,
    user_equip_opening_eased_fraction, user_equip_popup_rects, user_equip_scroll_max,
};
pub use nano_projection::{
    UserEquipNanoEquippedAuthority, UserEquipNanoGalleryEntryProjection,
    UserEquipNanoModeProjection, UserEquipNanoSkillProjection, UserEquipNanoStatusProjection,
};
pub use popup_state::{
    UserEquipDragState, UserEquipItemPopupState, UserEquipNanoViewerState, UserEquipPopupCommand,
    user_equip_drag_move, user_equip_gum_attribute_accepts_style, user_equip_gum_target_enabled,
    user_equip_trash_drop,
};
pub use source_evidence::{
    USER_EQUIP_BACKDROP_PATH_ID, USER_EQUIP_BACKDROP_SHA256, USER_EQUIP_CALCULATOR_BACK_PATH_ID,
    USER_EQUIP_CALCULATOR_BACK_SHA256, USER_EQUIP_CLOSE_PATH_ID, USER_EQUIP_CLOSE_SHA256,
    USER_EQUIP_CLOTHES_PANEL_PATH_ID, USER_EQUIP_CLOTHES_PANEL_SHA256,
    USER_EQUIP_CN_EQUIP_COMPONENT_PATH_ID, USER_EQUIP_COMBINED_PATH_ID, USER_EQUIP_COMBINED_SHA256,
    USER_EQUIP_EDITABLE_LAYOUT_PATH, USER_EQUIP_EQUIP_PANEL_COMPONENT_PATH_ID,
    USER_EQUIP_EQUIP_TITLE_PATH_ID, USER_EQUIP_EQUIP_TITLE_SHA256, USER_EQUIP_GAME_OBJECT_PATH_ID,
    USER_EQUIP_GENERAL_DIALOG_PATH_ID, USER_EQUIP_GENERAL_DIALOG_SHA256, USER_EQUIP_HELP_PATH_ID,
    USER_EQUIP_HELP_SHA256, USER_EQUIP_INVENTORY_MANAGER_COMPONENT_PATH_ID,
    USER_EQUIP_INVENTORY_PANEL_PATH_ID, USER_EQUIP_INVENTORY_PANEL_SHA256,
    USER_EQUIP_INVENTORY_SKIN_PATH_ID, USER_EQUIP_NANO_BACK_PATH_ID, USER_EQUIP_NANO_BACK_SHA256,
    USER_EQUIP_NANO_BLUE_PATH_ID, USER_EQUIP_NANO_BLUE_SHA256, USER_EQUIP_NANO_DIALOG_PATH_ID,
    USER_EQUIP_NANO_DIALOG_SHA256, USER_EQUIP_NANO_RED_PATH_ID, USER_EQUIP_NANO_RED_SHA256,
    USER_EQUIP_NANO_TAB_PATH_ID, USER_EQUIP_NANO_TAB_SHA256, USER_EQUIP_NANO_YELLOW_PATH_ID,
    USER_EQUIP_NANO_YELLOW_SHA256, USER_EQUIP_PC_STUFF_COMPONENT_PATH_ID,
    USER_EQUIP_POPUP_TEXTURE_EVIDENCE, USER_EQUIP_RED_BUTTON_HOVER_PATH_ID,
    USER_EQUIP_RED_BUTTON_HOVER_SHA256, USER_EQUIP_RED_BUTTON_PATH_ID,
    USER_EQUIP_RED_BUTTON_SHA256, USER_EQUIP_REGULAR_FONT_PATH_ID, USER_EQUIP_RIGHT_PANEL_PATH_ID,
    USER_EQUIP_RIGHT_PANEL_SHA256, USER_EQUIP_SLOT_EMPTY_PATH_ID, USER_EQUIP_SLOT_EMPTY_SHA256,
    USER_EQUIP_SLOT_OCCUPIED_PATH_ID, USER_EQUIP_SLOT_OCCUPIED_SHA256,
    USER_EQUIP_SMALL_FONT_PATH_ID, USER_EQUIP_SOURCE_BACKDROP_PATH, USER_EQUIP_SOURCE_BUILD,
    USER_EQUIP_SOURCE_CLOSE_PATH, USER_EQUIP_SOURCE_CLOTHES_PANEL_PATH,
    USER_EQUIP_SOURCE_COMBINED_PATH, USER_EQUIP_SOURCE_EQUIP_TITLE_PATH,
    USER_EQUIP_SOURCE_HELP_PATH, USER_EQUIP_SOURCE_INVENTORY_PANEL_PATH,
    USER_EQUIP_SOURCE_MAIN_ARCHIVE, USER_EQUIP_SOURCE_MAIN_ARCHIVE_SHA256,
    USER_EQUIP_SOURCE_MAIN_SERIALIZED_FILE, USER_EQUIP_SOURCE_NANO_TAB_PATH,
    USER_EQUIP_SOURCE_RIGHT_PANEL_PATH, USER_EQUIP_SOURCE_SLOT_EMPTY_PATH,
    USER_EQUIP_SOURCE_SLOT_OCCUPIED_PATH, USER_EQUIP_SOURCE_TEXTURES, USER_EQUIP_SOURCE_TRASH_PATH,
    USER_EQUIP_SOURCE_TUTORIAL_ARCHIVE, USER_EQUIP_SOURCE_TUTORIAL_ARCHIVE_SHA256,
    USER_EQUIP_TRASH_PATH_ID, USER_EQUIP_TRASH_SHA256, USER_EQUIP_TURN_LEFT_HOVER_PATH_ID,
    USER_EQUIP_TURN_LEFT_HOVER_SHA256, USER_EQUIP_TURN_LEFT_PATH_ID, USER_EQUIP_TURN_LEFT_SHA256,
    USER_EQUIP_TURN_RIGHT_HOVER_PATH_ID, USER_EQUIP_TURN_RIGHT_HOVER_SHA256,
    USER_EQUIP_TURN_RIGHT_PATH_ID, USER_EQUIP_TURN_RIGHT_SHA256,
    USER_EQUIP_USER_CLOTHES_COMPONENT_PATH_ID, UserEquipSourceArchive,
    UserEquipSourceTextureEvidence, UserEquipTextureRole,
};
use spawn::spawn_user_equip_ui;
pub use state::{
    UserEquipAvatarPreviewPresentation, UserEquipAvatarTurnDirection, UserEquipCloseBlockedReason,
    UserEquipCloseDisposition, UserEquipCloseSource, UserEquipInputCapabilities,
    UserEquipLifecyclePhase, UserEquipModalState, UserEquipMode, UserEquipOpenSource,
    UserEquipPresentationContext, UserEquipUiState, user_equip_avatar_turn_asset_role,
    user_equip_equipment_endpoint_for_item,
};
pub use view_model::{
    UserEquipEquipmentSlotView, UserEquipInventorySlotView, UserEquipItemModeView,
    UserEquipNanoGallerySlotView, UserEquipPresentationIcon, UserEquipSlotFrameVisual,
    user_equip_item_mode_view, user_equip_nano_gallery_view,
};
pub(crate) use view_model::{
    user_equip_display_text_id, user_equip_item_type_localized, user_equip_range_localized,
    user_equip_rarity_localized, user_equip_weapon_type_localized,
};

#[derive(Asset, TypePath, Debug, Clone)]
#[type_path = "ffone_client::user_equip_ui"]
struct UserEquipEditableLayout(UiLayoutDocument);

#[derive(Default, TypePath)]
#[type_path = "ffone_client::user_equip_ui"]
struct UserEquipEditableLayoutLoader;

impl AssetLoader for UserEquipEditableLayoutLoader {
    type Asset = UserEquipEditableLayout;
    type Settings = ();
    type Error = UiLayoutError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(UserEquipEditableLayout(UiLayoutDocument::from_json(
            &bytes,
        )?))
    }

    fn extensions(&self) -> &[&str] {
        &["ffui.json"]
    }
}

#[derive(Default)]
pub struct UserEquipUiPlugin;

impl Plugin for UserEquipUiPlugin {
    fn build(&self, app: &mut App) {
        crate::item_card::install(app);
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_asset::<UserEquipEditableLayout>()
            .init_asset_loader::<UserEquipEditableLayoutLoader>()
            .init_resource::<UserEquipEditableLayoutHandle>()
            .init_resource::<UserEquipUiState>()
            .init_resource::<UserEquipModalState>()
            .init_resource::<UserEquipAvatarPreviewPresentation>()
            .init_resource::<UserEquipItemModeProjection>()
            .init_resource::<UserEquipNanoModeProjection>()
            .init_resource::<UserEquipItemPopupState>()
            .init_resource::<UserEquipDragState>()
            .init_resource::<UserEquipNanoViewerState>()
            .init_resource::<UserEquipPresentationContext>()
            .init_resource::<UserEquipUiOutbox>()
            .init_resource::<UserEquipUiAudioOutbox>()
            .init_resource::<UserEquipUiAssetContract>()
            .add_systems(PostUpdate, clear_closed_user_equip_selection)
            .configure_sets(
                Update,
                (
                    UserEquipUiSet::Lifecycle,
                    UserEquipUiSet::Interaction,
                    UserEquipUiSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_user_equip_ui,
            )
            .add_systems(
                Update,
                (advance_user_equip_lifecycle.in_set(UserEquipUiSet::Lifecycle))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    nano_station::collect_actions,
                    redeem::collect_actions,
                    interaction::collect_user_equip_controller_actions,
                    collect_user_equip_ui_actions,
                    collect_user_equip_scrollbar_pointer,
                    collect_user_equip_avatar_rotation,
                )
                    .chain()
                    .in_set(UserEquipUiSet::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    bind_user_equip_ui,
                    bind_user_equip_popup_button_text_color,
                    bind_item_card_text_case,
                    nano_station::bind_controls,
                    redeem::bind_controls,
                )
                    .chain()
                    .in_set(UserEquipUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)] mod closed_selection_tests;
