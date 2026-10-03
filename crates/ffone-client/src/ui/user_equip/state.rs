//! UserEquip mode, modal/lifecycle state, avatar preview presentation and close gates.

use super::asset_contract::UserEquipStaticAssetRole;
use super::catalog::{
    USER_EQUIP_EQUIPMENT_STRIP_ORDER, UserEquipEquipmentSlotKind, UserEquipSlotEndpoint,
};
use super::geometry::{
    USER_EQUIP_AVATAR_PREVIEW_RECT, USER_EQUIP_AVATAR_VEHICLE_Y_OFFSET,
    USER_EQUIP_INVENTORY_SCROLL_VELOCITY, USER_EQUIP_OPEN_SECONDS, UserEquipUiRect,
};
use super::layout::{
    UserEquipItemModeLayout, clamp_user_equip_scroll, user_equip_item_mode_layout,
    user_equip_nano_scroll_max,
};
use super::nano_station::UserEquipNanoStationAction;
use bevy::prelude::*;
use ffone_protocol::ItemBase0104;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum UserEquipMode {
    #[default]
    Item = 2,
    Nano = 3,
}

/// Entry routes shared by the NanoCom buttons and configurable shortcuts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipOpenSource {
    NanocomMyStuff,
    InventoryShortcut,
    NanoBookShortcut,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UserEquipLifecyclePhase {
    #[default]
    Hidden,
    Opening,
    Visible,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserEquipModalState {
    pub send_pending: bool,
    pub help_active: bool,
    pub system_popup_active: bool,
    pub inventory_popup_modal: bool,
    pub item_popup_active: bool,
    pub redeem_code_view: bool,
    /// Typed shell boundary for clean `(11, 13)` exit arbitration.
    pub external_exit_blocked: bool,
}

impl UserEquipModalState {
    #[must_use]
    pub const fn global_gui_enabled(self) -> bool {
        !self.send_pending && !self.help_active && !self.system_popup_active
    }

    #[must_use]
    pub const fn panel_gui_enabled(self) -> bool {
        self.global_gui_enabled()
            && !self.inventory_popup_modal
            && !self.item_popup_active
            && !self.redeem_code_view
    }

    #[must_use]
    pub const fn has_popup_to_dismiss(self) -> bool {
        self.inventory_popup_modal || self.item_popup_active
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserEquipPresentationContext {
    pub player_name: String,
    pub level: i32,
    pub gender: i32,
    /// Lossless authoritative mentor identity; display names are localized.
    pub guide: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub fusion_matter: i32,
    pub max_fusion_matter: i32,
    pub taros: i64,
    pub weapon_battery: i32,
    pub nano_battery: i32,
    pub guide_name: String,
    pub guide_name_key: Option<String>,
    pub guide_icon_path: Option<String>,
}

/// UI-owned rotation plus the authoritative mounted-state adapter needed to
/// place the independent render target. Neither field mutates the world avatar
/// or an inventory projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Resource)]
pub struct UserEquipAvatarPreviewPresentation {
    pub(super) yaw_degrees: f32,
    pub(super) vehicle_mounted: bool,
}

impl UserEquipAvatarPreviewPresentation {
    #[must_use]
    pub const fn yaw_degrees(&self) -> f32 {
        self.yaw_degrees
    }

    #[must_use]
    pub const fn vehicle_mounted(&self) -> bool {
        self.vehicle_mounted
    }

    pub fn set_vehicle_mounted(&mut self, mounted: bool) {
        self.vehicle_mounted = mounted;
    }

    /// One Bevy UI update is the native equivalent of one held IMGUI
    /// `RepeatButton` frame, so clean's literal `Rotate(..., +/-1, ...)` stays
    /// frame-based and intentionally does not use wall-clock normalization.
    pub fn rotate_one_gui_frame(&mut self, delta_degrees: f32) {
        if delta_degrees.is_finite() {
            self.yaw_degrees = (self.yaw_degrees + delta_degrees).rem_euclid(360.0);
        }
    }

    #[must_use]
    pub fn avatar_rect(&self) -> UserEquipUiRect {
        USER_EQUIP_AVATAR_PREVIEW_RECT.translated(
            0.0,
            if self.vehicle_mounted {
                USER_EQUIP_AVATAR_VEHICLE_Y_OFFSET
            } else {
                0.0
            },
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipAvatarTurnDirection {
    LeftPositioned,
    RightPositioned,
}

impl UserEquipAvatarTurnDirection {
    #[must_use]
    pub const fn yaw_delta_degrees(self) -> f32 {
        match self {
            Self::LeftPositioned => 1.0,
            Self::RightPositioned => -1.0,
        }
    }
}

/// Serialized style assignment is intentionally crossed by screen position:
/// `rectLeftTurn` owns `rightturn`, while `rectRightTurn` owns `leftturn`.
#[must_use]
pub const fn user_equip_avatar_turn_asset_role(
    direction: UserEquipAvatarTurnDirection,
    hovered: bool,
) -> UserEquipStaticAssetRole {
    match (direction, hovered) {
        (UserEquipAvatarTurnDirection::LeftPositioned, false) => {
            UserEquipStaticAssetRole::TurnRight
        }
        (UserEquipAvatarTurnDirection::LeftPositioned, true) => {
            UserEquipStaticAssetRole::TurnRightHover
        }
        (UserEquipAvatarTurnDirection::RightPositioned, false) => {
            UserEquipStaticAssetRole::TurnLeft
        }
        (UserEquipAvatarTurnDirection::RightPositioned, true) => {
            UserEquipStaticAssetRole::TurnLeftHover
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipCloseSource {
    CloseButton,
    Escape,
    ActiveTabHotkey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipCloseBlockedReason {
    NotVisible,
    OpeningAnimation,
    SendPending,
    HelpActive,
    SystemPopup,
    RedeemCodeView,
    ExternalExitGate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipCloseDisposition {
    ExitMode,
    /// Mirrors `(11, 13)` consuming the first close request to remove active
    /// InventoryManager/PopupControll windows.
    DismissPopups,
    Blocked(UserEquipCloseBlockedReason),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UserEquipInputCapabilities {
    pub draw: bool,
    pub panel_controls: bool,
    pub slot_pointer: bool,
    pub scroll: bool,
    pub close_button: bool,
    /// `true` also when a keyboard request would only dismiss a popup.
    pub keyboard_close_request: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct UserEquipUiState {
    pub(super) mode: UserEquipMode,
    pub(super) phase: UserEquipLifecyclePhase,
    pub(super) opening_elapsed_seconds: f32,
    pub(super) scroll_y: f32,
    pub(super) nano_station_npc: Option<i32>,
    pub(super) nano_station_pending: Option<(UserEquipNanoStationAction, f32)>,
}

impl Default for UserEquipUiState {
    fn default() -> Self {
        Self {
            mode: UserEquipMode::Item,
            phase: UserEquipLifecyclePhase::Hidden,
            opening_elapsed_seconds: 0.0,
            scroll_y: 0.0,
            nano_station_npc: None,
            nano_station_pending: None,
        }
    }
}

impl UserEquipUiState {
    #[must_use]
    pub const fn mode(&self) -> UserEquipMode {
        self.mode
    }

    #[must_use]
    pub const fn phase(&self) -> UserEquipLifecyclePhase {
        self.phase
    }

    #[must_use]
    pub const fn opening_elapsed_seconds(&self) -> f32 {
        self.opening_elapsed_seconds
    }

    #[must_use]
    pub const fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self.phase, UserEquipLifecyclePhase::Hidden)
    }

    pub fn open_item_mode(&mut self) {
        self.nano_station_npc = None;
        self.nano_station_pending = None;
        self.mode = UserEquipMode::Item;
        self.phase = UserEquipLifecyclePhase::Opening;
        self.opening_elapsed_seconds = 0.0;
        self.scroll_y = 0.0;
    }

    pub fn select_item_tab(&mut self) {
        if self.is_active() {
            self.mode = UserEquipMode::Item;
            self.scroll_y = 0.0;
        }
    }

    pub fn select_nano_tab(&mut self) {
        if self.is_active() {
            self.mode = UserEquipMode::Nano;
            self.scroll_y = 0.0;
        }
    }

    pub fn open_from(&mut self, source: UserEquipOpenSource) {
        match source {
            UserEquipOpenSource::NanocomMyStuff | UserEquipOpenSource::InventoryShortcut => {
                self.open_item_mode();
            }
            UserEquipOpenSource::NanoBookShortcut => {
                self.open_item_mode();
                self.select_nano_tab();
            }
        }
    }

    pub fn close(&mut self) {
        self.nano_station_npc = None;
        self.nano_station_pending = None;
        self.phase = UserEquipLifecyclePhase::Hidden;
        self.opening_elapsed_seconds = 0.0;
        self.scroll_y = 0.0;
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        if let Some((_, elapsed)) = &mut self.nano_station_pending {
            *elapsed += delta_seconds.max(0.0);
            if *elapsed >= 10.0 {
                self.nano_station_pending = None;
            }
        }
        if self.phase != UserEquipLifecyclePhase::Opening
            || delta_seconds.is_nan()
            || delta_seconds <= 0.0
        {
            return;
        }
        if !delta_seconds.is_finite() {
            self.opening_elapsed_seconds = USER_EQUIP_OPEN_SECONDS;
        } else {
            self.opening_elapsed_seconds =
                (self.opening_elapsed_seconds + delta_seconds).min(USER_EQUIP_OPEN_SECONDS);
        }
        if self.opening_elapsed_seconds >= USER_EQUIP_OPEN_SECONDS {
            self.phase = UserEquipLifecyclePhase::Visible;
        }
    }

    pub fn set_scroll_y(&mut self, scroll_y: f32) {
        self.scroll_y = if self.mode == UserEquipMode::Nano {
            scroll_y.clamp(0.0, user_equip_nano_scroll_max())
        } else {
            clamp_user_equip_scroll(scroll_y)
        };
    }

    /// Applies clean `CnEquip`: `axis.clamp(-1, 1) * 200`, followed by
    /// `Panel_PCStuff.Scroll`, which subtracts that value.
    pub fn apply_legacy_scroll_axis(&mut self, axis: f32) {
        let axis = if axis.is_finite() {
            axis.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        self.set_scroll_y(self.scroll_y - axis * USER_EQUIP_INVENTORY_SCROLL_VELOCITY);
    }

    #[must_use]
    pub fn layout(&self, viewport_width: u32, viewport_height: u32) -> UserEquipItemModeLayout {
        let mut layout = user_equip_item_mode_layout(
            viewport_width,
            viewport_height,
            self.opening_elapsed_seconds,
            self.scroll_y,
        );
        if self.mode == UserEquipMode::Nano {
            // The shared item-layout constructor owns the 690px item content
            // and therefore clamps at 190px. Nano content is 897px; retain the
            // mode-aware state clamp instead of truncating it a second time.
            layout.scroll_y = self.scroll_y.clamp(0.0, user_equip_nano_scroll_max());
            layout.inventory_content.top = layout.inventory_viewport.top - layout.scroll_y;
        }
        layout
    }

    #[must_use]
    pub fn close_disposition(
        &self,
        source: UserEquipCloseSource,
        modal: UserEquipModalState,
    ) -> UserEquipCloseDisposition {
        if self.phase == UserEquipLifecyclePhase::Hidden {
            return UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::NotVisible);
        }
        if modal.redeem_code_view {
            return UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::RedeemCodeView);
        }
        if modal.send_pending || self.nano_station_send_pending() {
            return UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::SendPending);
        }
        if modal.system_popup_active {
            return UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::SystemPopup);
        }
        if modal.help_active {
            return UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::HelpActive);
        }
        if modal.has_popup_to_dismiss() {
            return UserEquipCloseDisposition::DismissPopups;
        }
        if modal.external_exit_blocked {
            return UserEquipCloseDisposition::Blocked(
                UserEquipCloseBlockedReason::ExternalExitGate,
            );
        }
        if source == UserEquipCloseSource::CloseButton
            && self.phase == UserEquipLifecyclePhase::Opening
        {
            return UserEquipCloseDisposition::Blocked(
                UserEquipCloseBlockedReason::OpeningAnimation,
            );
        }
        UserEquipCloseDisposition::ExitMode
    }

    #[must_use]
    pub fn input_capabilities(&self, modal: UserEquipModalState) -> UserEquipInputCapabilities {
        let draw = self.phase != UserEquipLifecyclePhase::Hidden;
        let panel_controls = self.phase == UserEquipLifecyclePhase::Visible
            && modal.panel_gui_enabled()
            && !self.nano_station_send_pending();
        let hard_keyboard_block = self.nano_station_send_pending()
            || modal.send_pending
            || modal.help_active
            || modal.system_popup_active
            || modal.redeem_code_view;
        UserEquipInputCapabilities {
            draw,
            panel_controls,
            slot_pointer: panel_controls,
            scroll: panel_controls,
            close_button: panel_controls && !modal.external_exit_blocked,
            keyboard_close_request: draw && !hard_keyboard_block,
        }
    }
}

#[must_use]
pub fn user_equip_equipment_endpoint_for_item(
    item: ItemBase0104,
    secondary_weapon: bool,
) -> Option<UserEquipSlotEndpoint> {
    let visual_index = USER_EQUIP_EQUIPMENT_STRIP_ORDER.iter().position(|spec| {
        if item.item_type == 0 {
            matches!(
                spec.kind,
                UserEquipEquipmentSlotKind::SecondaryWeapon if secondary_weapon
            ) || matches!(
                spec.kind,
                UserEquipEquipmentSlotKind::PrimaryWeapon if !secondary_weapon
            )
        } else {
            spec.legacy_panel_item_type == item.item_type
        }
    })?;
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
    Some(UserEquipSlotEndpoint::Equipment {
        visual_index,
        wire_slot_index: spec.wire_slot_index,
    })
}
