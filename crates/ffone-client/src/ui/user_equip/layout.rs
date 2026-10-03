//! Opening animation, scroll metrics and computed item-mode/popup layout.

use super::catalog::UserEquipSlotEndpoint;
use super::geometry::{
    USER_EQUIP_BACKDROP_HEIGHT, USER_EQUIP_BACKDROP_WIDTH, USER_EQUIP_BACKPLATE_REFERENCE_HEIGHT,
    USER_EQUIP_BACKPLATE_REFERENCE_WIDTH, USER_EQUIP_CLOSE_RECT, USER_EQUIP_EQUIP_STRIP_FINAL_X,
    USER_EQUIP_EQUIP_STRIP_RECT, USER_EQUIP_EQUIPMENT_CONTENT_RECT, USER_EQUIP_EQUIPMENT_SLOT_SIZE,
    USER_EQUIP_EQUIPMENT_SLOT_STRIDE, USER_EQUIP_EQUIPMENT_STRIP_COUNT,
    USER_EQUIP_EQUIPMENT_TITLE_RECT, USER_EQUIP_HELP_RECT, USER_EQUIP_INVENTORY_COLUMNS,
    USER_EQUIP_INVENTORY_CONTENT_HEIGHT, USER_EQUIP_INVENTORY_CONTENT_WIDTH,
    USER_EQUIP_INVENTORY_SLOT_SIZE, USER_EQUIP_INVENTORY_SLOT_STRIDE,
    USER_EQUIP_INVENTORY_VIEWPORT_RECT, USER_EQUIP_LEFT_PANEL_START_X,
    USER_EQUIP_MOUSE_SCROLL_AXIS_SENSITIVITY, USER_EQUIP_NANO_CONTENT_HEIGHT,
    USER_EQUIP_OPEN_SECONDS, USER_EQUIP_PC_STUFF_FINAL_X, USER_EQUIP_PC_STUFF_RECT,
    USER_EQUIP_REFERENCE_HEIGHT, USER_EQUIP_REFERENCE_WIDTH, USER_EQUIP_SCROLL_TRACK_RECT,
    USER_EQUIP_TRASH_RECT, USER_EQUIP_USER_CLOTHES_RECT, UserEquipUiRect,
};
use super::state::UserEquipMode;
use crate::inventory_runtime::INVENTORY_SLOT_COUNT_0104;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserEquipItemModeLayout {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub opening_eased_fraction: f32,
    pub scroll_y: f32,
    pub full_backdrop: UserEquipUiRect,
    pub clothes_backplate: UserEquipUiRect,
    pub pc_stuff_backplate: UserEquipUiRect,
    pub user_clothes_panel: UserEquipUiRect,
    pub pc_stuff_panel: UserEquipUiRect,
    pub equipment_panel: UserEquipUiRect,
    pub inventory_viewport: UserEquipUiRect,
    pub inventory_content: UserEquipUiRect,
    pub equipment_content: UserEquipUiRect,
    pub equipment_title: UserEquipUiRect,
    pub close_button: UserEquipUiRect,
    pub trash_button: UserEquipUiRect,
    pub help_button: UserEquipUiRect,
}

impl UserEquipItemModeLayout {
    #[must_use]
    pub fn inventory_slot_rect(self, slot_index: usize) -> Option<UserEquipUiRect> {
        if slot_index >= INVENTORY_SLOT_COUNT_0104 {
            return None;
        }
        let column = slot_index % USER_EQUIP_INVENTORY_COLUMNS;
        let row = slot_index / USER_EQUIP_INVENTORY_COLUMNS;
        Some(UserEquipUiRect::new(
            self.inventory_content.left + column as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
            self.inventory_content.top + row as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
            USER_EQUIP_INVENTORY_SLOT_SIZE,
            USER_EQUIP_INVENTORY_SLOT_SIZE,
        ))
    }

    #[must_use]
    pub fn equipment_slot_rect(self, visual_index: usize) -> Option<UserEquipUiRect> {
        if visual_index >= USER_EQUIP_EQUIPMENT_STRIP_COUNT {
            return None;
        }
        Some(UserEquipUiRect::new(
            self.equipment_content.left,
            self.equipment_content.top + visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE,
            USER_EQUIP_EQUIPMENT_SLOT_SIZE,
            USER_EQUIP_EQUIPMENT_SLOT_SIZE,
        ))
    }

    #[must_use]
    pub fn equipment_label_rect(self, visual_index: usize) -> Option<UserEquipUiRect> {
        if visual_index >= USER_EQUIP_EQUIPMENT_STRIP_COUNT {
            return None;
        }
        Some(UserEquipUiRect::new(
            self.equipment_content.left,
            self.equipment_content.top - 2.0
                + visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE,
            60.0,
            19.0,
        ))
    }
}

#[must_use]
pub fn user_equip_opening_eased_fraction(elapsed_seconds: f32) -> f32 {
    if elapsed_seconds.is_nan() || elapsed_seconds <= 0.0 {
        return 0.0;
    }
    if !elapsed_seconds.is_finite() || elapsed_seconds >= USER_EQUIP_OPEN_SECONDS {
        return 1.0;
    }
    (elapsed_seconds / USER_EQUIP_OPEN_SECONDS * std::f32::consts::FRAC_PI_2).sin()
}

#[must_use]
pub fn user_equip_scroll_max() -> f32 {
    USER_EQUIP_INVENTORY_CONTENT_HEIGHT - USER_EQUIP_INVENTORY_VIEWPORT_RECT.height
}

#[must_use]
pub fn user_equip_nano_scroll_max() -> f32 {
    (USER_EQUIP_NANO_CONTENT_HEIGHT - USER_EQUIP_INVENTORY_VIEWPORT_RECT.height).max(0.0)
}

/// Vertical-scrollbar geometry of the `Panel_PCStuff` scroll view shared by
/// the Item and Nano pages. Drawing and pointer dragging both use it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserEquipScrollbarMetrics {
    pub thumb_height: f32,
    pub travel: f32,
    pub scroll_max: f32,
}

impl UserEquipScrollbarMetrics {
    /// The thumb spans `visible / content` of the track, never below 15 px.
    #[must_use]
    pub fn for_mode(mode: UserEquipMode) -> Self {
        let content_height = match mode {
            UserEquipMode::Item => USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
            UserEquipMode::Nano => USER_EQUIP_NANO_CONTENT_HEIGHT,
        };
        let viewport_height = USER_EQUIP_INVENTORY_VIEWPORT_RECT.height;
        let track_height = USER_EQUIP_SCROLL_TRACK_RECT.height;
        let thumb_height =
            (track_height * viewport_height / content_height).clamp(15.0, track_height);
        Self {
            thumb_height,
            travel: track_height - thumb_height,
            scroll_max: (content_height - viewport_height).max(0.0),
        }
    }

    #[must_use]
    pub fn thumb_offset(self, scroll_y: f32) -> f32 {
        if self.scroll_max > 0.0 {
            self.travel * scroll_y / self.scroll_max
        } else {
            0.0
        }
    }

    /// Unity `SliderHandler` drags keep the grab offset, so the thumb follows
    /// the pointer one-to-one and clamps at both ends of the track.
    #[must_use]
    pub fn drag_value(self, start: f32, pointer_delta: f32) -> f32 {
        if self.travel <= 0.0 || !pointer_delta.is_finite() {
            return start.clamp(0.0, self.scroll_max);
        }
        (start + pointer_delta / self.travel * self.scroll_max).clamp(0.0, self.scroll_max)
    }
}

#[must_use]
pub fn clamp_user_equip_scroll(scroll_y: f32) -> f32 {
    if scroll_y.is_nan() || scroll_y <= 0.0 {
        0.0
    } else if !scroll_y.is_finite() {
        user_equip_scroll_max()
    } else {
        scroll_y.min(user_equip_scroll_max())
    }
}

#[must_use]
pub fn user_equip_bevy_wheel_to_legacy_axis(delta_y: f32) -> f32 {
    if delta_y.is_finite() {
        delta_y * USER_EQUIP_MOUSE_SCROLL_AXIS_SENSITIVITY
    } else {
        0.0
    }
}

#[must_use]
pub fn user_equip_item_mode_layout(
    viewport_width: u32,
    viewport_height: u32,
    opening_elapsed_seconds: f32,
    scroll_y: f32,
) -> UserEquipItemModeLayout {
    let viewport_width = viewport_width.min(i32::MAX as u32) as i32;
    let viewport_height = viewport_height.min(i32::MAX as u32) as i32;
    let center_x = if viewport_width > USER_EQUIP_REFERENCE_WIDTH {
        (viewport_width - USER_EQUIP_REFERENCE_WIDTH) / 2
    } else {
        0
    };
    let top = ((viewport_height - USER_EQUIP_REFERENCE_HEIGHT) / 2).max(0);
    let eased = user_equip_opening_eased_fraction(opening_elapsed_seconds);
    let remaining = 1.0 - eased;

    let user_clothes_x =
        center_x - ((center_x - USER_EQUIP_LEFT_PANEL_START_X) as f32 * remaining) as i32;
    let pc_stuff_final_x = USER_EQUIP_PC_STUFF_FINAL_X + center_x;
    let pc_stuff_x = pc_stuff_final_x
        + ((USER_EQUIP_REFERENCE_WIDTH - pc_stuff_final_x) as f32 * remaining) as i32;
    let equipment_final_x = USER_EQUIP_EQUIP_STRIP_FINAL_X + center_x;
    let equipment_x = equipment_final_x
        + ((USER_EQUIP_REFERENCE_WIDTH - equipment_final_x) as f32 * remaining) as i32;

    let full_backdrop = UserEquipUiRect::new(
        ((viewport_width - USER_EQUIP_BACKDROP_WIDTH as i32) / 2) as f32,
        ((viewport_height - USER_EQUIP_BACKDROP_HEIGHT as i32) / 2) as f32,
        USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_BACKDROP_HEIGHT,
    );
    let backplate_left = (viewport_width - USER_EQUIP_BACKPLATE_REFERENCE_WIDTH) / 2;
    let backplate_top = (viewport_height - USER_EQUIP_BACKPLATE_REFERENCE_HEIGHT) / 2;
    let clothes_backplate =
        UserEquipUiRect::new(backplate_left as f32, backplate_top as f32, 585.0, 653.0);
    let pc_stuff_backplate = UserEquipUiRect::new(
        (backplate_left + 585) as f32,
        backplate_top as f32,
        451.0,
        653.0,
    );

    let user_clothes_panel =
        USER_EQUIP_USER_CLOTHES_RECT.translated(user_clothes_x as f32, top as f32);
    let pc_stuff_panel = USER_EQUIP_PC_STUFF_RECT.translated(pc_stuff_x as f32, top as f32);
    let equipment_panel = USER_EQUIP_EQUIP_STRIP_RECT.translated(equipment_x as f32, top as f32);
    let inventory_viewport =
        USER_EQUIP_INVENTORY_VIEWPORT_RECT.translated(pc_stuff_x as f32, top as f32);
    let scroll_y = clamp_user_equip_scroll(scroll_y);
    let inventory_content = UserEquipUiRect::new(
        inventory_viewport.left,
        inventory_viewport.top - scroll_y,
        USER_EQUIP_INVENTORY_CONTENT_WIDTH,
        USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
    );
    let equipment_content =
        USER_EQUIP_EQUIPMENT_CONTENT_RECT.translated(equipment_x as f32, top as f32);
    let equipment_title =
        USER_EQUIP_EQUIPMENT_TITLE_RECT.translated(equipment_x as f32, top as f32);

    UserEquipItemModeLayout {
        viewport_width,
        viewport_height,
        opening_eased_fraction: eased,
        scroll_y,
        full_backdrop,
        clothes_backplate,
        pc_stuff_backplate,
        user_clothes_panel,
        pc_stuff_panel,
        equipment_panel,
        inventory_viewport,
        inventory_content,
        equipment_content,
        equipment_title,
        close_button: USER_EQUIP_CLOSE_RECT.translated(pc_stuff_x as f32, top as f32),
        trash_button: USER_EQUIP_TRASH_RECT.translated(pc_stuff_x as f32, top as f32),
        help_button: USER_EQUIP_HELP_RECT.translated(pc_stuff_x as f32, top as f32),
    }
}

/// Primary `PopupControll` geometry at the clean 1020x638 reference frame,
/// translated with the corresponding Item-mode panel.
#[must_use]
pub fn user_equip_popup_rects(
    layout: UserEquipItemModeLayout,
    endpoint: UserEquipSlotEndpoint,
) -> (UserEquipUiRect, UserEquipUiRect) {
    match endpoint {
        UserEquipSlotEndpoint::Inventory { .. } => (
            UserEquipUiRect::new(
                layout.pc_stuff_panel.left + 25.0,
                layout.pc_stuff_panel.top + 60.0,
                310.0,
                449.0,
            ),
            UserEquipUiRect::new(0.0, 14.0, 310.0, 435.0),
        ),
        UserEquipSlotEndpoint::Equipment { .. } => (
            UserEquipUiRect::new(
                layout.user_clothes_panel.left + 195.0,
                layout.user_clothes_panel.top + 75.0,
                310.0,
                448.0,
            ),
            UserEquipUiRect::new(0.0, 0.0, 310.0, 448.0),
        ),
    }
}
