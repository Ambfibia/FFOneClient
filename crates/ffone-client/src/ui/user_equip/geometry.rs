//! Reference-space dimensions and rectangles of the UserEquip window.

use crate::inventory_runtime::{EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104};
use bevy::{prelude::*, sprite::BorderRect};

pub const USER_EQUIP_REFERENCE_WIDTH: i32 = 1_020;
pub const USER_EQUIP_REFERENCE_HEIGHT: i32 = 638;
pub const USER_EQUIP_BACKDROP_WIDTH: f32 = 1_920.0;
pub const USER_EQUIP_BACKDROP_HEIGHT: f32 = 1_440.0;
pub const USER_EQUIP_BACKPLATE_REFERENCE_WIDTH: i32 = 1_036;
pub const USER_EQUIP_BACKPLATE_REFERENCE_HEIGHT: i32 = 653;

pub const USER_EQUIP_OPEN_SECONDS: f32 = 1.0;
pub const USER_EQUIP_LEFT_PANEL_START_X: i32 = -475;
pub const USER_EQUIP_PC_STUFF_FINAL_X: i32 = 585;
pub const USER_EQUIP_EQUIP_STRIP_FINAL_X: i32 = 504;

pub const USER_EQUIP_INVENTORY_COLUMNS: usize = 5;
pub const USER_EQUIP_INVENTORY_ROWS: usize = 10;
pub const USER_EQUIP_INVENTORY_SLOT_SIZE: f32 = 67.0;
pub const USER_EQUIP_INVENTORY_SLOT_GAP: f32 = 2.0;
pub const USER_EQUIP_INVENTORY_SLOT_STRIDE: f32 =
    USER_EQUIP_INVENTORY_SLOT_SIZE + USER_EQUIP_INVENTORY_SLOT_GAP;
pub const USER_EQUIP_INVENTORY_CONTENT_WIDTH: f32 =
    USER_EQUIP_INVENTORY_COLUMNS as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE;
pub const USER_EQUIP_INVENTORY_CONTENT_HEIGHT: f32 =
    USER_EQUIP_INVENTORY_ROWS as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE;
pub const USER_EQUIP_INVENTORY_SCROLL_VELOCITY: f32 = 200.0;
/// `mainData` serializes `Mouse ScrollWheel.sensitivity == 0.1`. Bevy's
/// accumulated line delta has already skipped that Unity input-axis stage.
pub const USER_EQUIP_MOUSE_SCROLL_AXIS_SENSITIVITY: f32 = 0.1;
/// Native gameplay identities, including the restored event Nano and the
/// formerly preview-only models. Ownership always comes from the server bank.
pub const USER_EQUIP_NANO_GALLERY_COUNT: usize = 66;
pub const USER_EQUIP_NANO_COLUMNS: usize = 5;
pub const USER_EQUIP_NANO_SLOT_SIZE: f32 = 67.0;
pub const USER_EQUIP_NANO_SLOT_STRIDE: f32 = 69.0;
/// Complete fourteen-row extent needed to expose every published Nano through
/// the clean five-column scroll view.
pub const USER_EQUIP_NANO_CONTENT_HEIGHT: f32 = USER_EQUIP_NANO_GALLERY_COUNT
    .div_ceil(USER_EQUIP_NANO_COLUMNS) as f32
    * USER_EQUIP_NANO_SLOT_STRIDE;

pub(super) const USER_EQUIP_MERGED_NANO_ORDER: [i16; USER_EQUIP_NANO_GALLERY_COUNT] = [
    45, 46, 41, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 42, 19, 20, 21, 22,
    23, 24, 25, 26, 27, 28, 43, 29, 30, 31, 32, 33, 34, 35, 36, 44, 68, 48, 49, 50, 38, 51, 53, 37,
    54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 69, 70,
];

pub const USER_EQUIP_EQUIPMENT_STRIP_COUNT: usize = 9;
pub const USER_EQUIP_EQUIPMENT_SLOT_SIZE: f32 = 64.0;
pub const USER_EQUIP_EQUIPMENT_SLOT_STRIDE: f32 = 62.0;
pub const USER_EQUIP_UI_Z_INDEX: i32 = 19;
pub const USER_EQUIP_COMBINED_BADGE_LEFT: f32 = 36.0;
pub const USER_EQUIP_COMBINED_BADGE_TOP: f32 = 36.0;
pub const USER_EQUIP_COMBINED_BADGE_SIZE: f32 = 26.0;
pub const USER_EQUIP_COUNT_LABEL_LEFT: f32 = 5.0;
pub const USER_EQUIP_COUNT_LABEL_TOP: f32 = 5.0;
pub const USER_EQUIP_TAB_FONT_SIZE: f32 = 12.0;
pub const USER_EQUIP_COUNT_FONT_SIZE: f32 = 12.0;
/// Clean `JEFFE___12` path ID 977 serialized line spacing.
pub const USER_EQUIP_REGULAR_FONT_LINE_HEIGHT: f32 = 13.560_000_42;
pub const USER_EQUIP_SMALL_FONT_SIZE: f32 = 8.0;
/// Clean `JEFFE___08` path ID 1127 serialized line spacing.
pub const USER_EQUIP_SMALL_FONT_LINE_HEIGHT: f32 = 9.039_999_96;
pub const USER_EQUIP_STATUS_FONT_SIZE: f32 = 12.0;
pub const USER_EQUIP_POPUP_FONT_SIZE: f32 = 12.0;
pub const USER_EQUIP_MISSING_CHECKER_SIZE: u32 = 64;
pub const USER_EQUIP_MISSING_CHECKER_QUADRANT: u32 = 32;
pub const USER_EQUIP_INVENTORY_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(136.0, 33.0),
    max_inset: Vec2::new(10.0, 77.0),
};
pub const USER_EQUIP_NANO_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(219.0, 34.0),
    max_inset: Vec2::new(10.0, 77.0),
};
pub const USER_EQUIP_RIGHT_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(5.0, 0.0),
    max_inset: Vec2::new(5.0, 0.0),
};
pub const USER_EQUIP_STATUS_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 8.0),
    max_inset: Vec2::new(8.0, 8.0),
};
pub const USER_EQUIP_INVENTORY_SHADOW_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(0.0, 32.0),
    max_inset: Vec2::new(0.0, 32.0),
};

const _: [(); INVENTORY_SLOT_COUNT_0104] =
    [(); USER_EQUIP_INVENTORY_COLUMNS * USER_EQUIP_INVENTORY_ROWS];
const _: [(); EQUIPMENT_SLOT_COUNT_0104] = [(); USER_EQUIP_EQUIPMENT_STRIP_COUNT];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserEquipUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl UserEquipUiRect {
    #[must_use]
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn translated(self, x: f32, y: f32) -> Self {
        Self::new(self.left + x, self.top + y, self.width, self.height)
    }

    pub(super) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.left),
            top: px(self.top),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

pub(super) fn user_equip_nano_popup_content_rect(rect: UserEquipUiRect, equipped: bool) -> UserEquipUiRect {
    if equipped {
        UserEquipUiRect::new(rect.left + 2.5, rect.top + 18.0, rect.width, rect.height)
    } else {
        rect
    }
}

pub const USER_EQUIP_USER_CLOTHES_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 0.0, 512.0, 649.0);
/// Runtime `Panel_UserClothes.ownuser`; the parent group clips its negative
/// top without changing the source camera's 500:564 aspect.
pub const USER_EQUIP_AVATAR_PREVIEW_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, -50.0, 500.0, 564.0);
pub const USER_EQUIP_AVATAR_VEHICLE_Y_OFFSET: f32 = 100.0;
/// `rectLeftTurn` is position-based naming: clean assigns the visually right
/// `rightturn` GUIStyle here and rotates the independent avatar by +1 degree.
pub const USER_EQUIP_TURN_LEFT_POSITIONED_RECT: UserEquipUiRect =
    UserEquipUiRect::new(120.0, 440.0, 43.0, 78.0);
/// Clean assigns `leftturn` here and rotates by -1 degree.
pub const USER_EQUIP_TURN_RIGHT_POSITIONED_RECT: UserEquipUiRect =
    UserEquipUiRect::new(340.0, 440.0, 43.0, 78.0);
pub const USER_EQUIP_PC_STUFF_RECT: UserEquipUiRect = UserEquipUiRect::new(0.0, 0.0, 380.0, 632.0);
pub const USER_EQUIP_EQUIP_STRIP_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 0.0, 66.0, 639.0);
pub const USER_EQUIP_INVENTORY_VIEWPORT_RECT: UserEquipUiRect =
    UserEquipUiRect::new(6.0, 36.0, 366.0, 500.0);
pub const USER_EQUIP_SCROLL_UP_RECT: UserEquipUiRect =
    UserEquipUiRect::new(355.0, 36.0, 17.0, 12.0);
pub const USER_EQUIP_SCROLL_TRACK_RECT: UserEquipUiRect =
    UserEquipUiRect::new(355.0, 48.0, 17.0, 476.0);
pub const USER_EQUIP_SCROLL_DOWN_RECT: UserEquipUiRect =
    UserEquipUiRect::new(355.0, 524.0, 17.0, 12.0);
pub const USER_EQUIP_EQUIPMENT_CONTENT_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 14.0, 66.0, 625.0);
pub const USER_EQUIP_EQUIPMENT_TITLE_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 3.0, 64.0, 13.0);
pub const USER_EQUIP_ITEM_TAB_TEXTURE_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 0.0, 142.0, 29.0);
pub const USER_EQUIP_NANO_TAB_TEXTURE_RECT: UserEquipUiRect =
    UserEquipUiRect::new(96.0, 0.0, 129.0, 29.0);
pub const USER_EQUIP_ITEM_TAB_HIT_RECT: UserEquipUiRect =
    UserEquipUiRect::new(10.0, 5.0, 105.0, 15.0);
pub const USER_EQUIP_NANO_TAB_HIT_RECT: UserEquipUiRect =
    UserEquipUiRect::new(135.0, 5.0, 60.0, 15.0);
pub const USER_EQUIP_CLOSE_RECT: UserEquipUiRect = UserEquipUiRect::new(400.0, 5.0, 30.0, 30.0);
pub const USER_EQUIP_TRASH_RECT: UserEquipUiRect = UserEquipUiRect::new(390.0, 555.0, 32.0, 32.0);
pub const USER_EQUIP_HELP_RECT: UserEquipUiRect = UserEquipUiRect::new(390.0, 590.0, 32.0, 32.0);
/// Clean `Panel_Equip.DoEquipPanel` splits the last 60 px into two half slots.
pub const USER_EQUIP_BOOST_RECT: UserEquipUiRect = UserEquipUiRect::new(0.0, 558.0, 64.0, 30.0);
pub const USER_EQUIP_POTION_RECT: UserEquipUiRect = UserEquipUiRect::new(0.0, 588.0, 64.0, 30.0);
pub const USER_EQUIP_BOOST_ICON_RECT: UserEquipUiRect =
    UserEquipUiRect::new(32.0, 563.0, 30.0, 28.0);
pub const USER_EQUIP_POTION_ICON_RECT: UserEquipUiRect =
    UserEquipUiRect::new(32.0, 592.0, 30.0, 28.0);
pub const USER_EQUIP_BOOST_LABEL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 556.0, 60.0, 19.0);
pub const USER_EQUIP_POTION_LABEL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 585.0, 60.0, 19.0);
pub const USER_EQUIP_BOOST_VALUE_RECT: UserEquipUiRect =
    UserEquipUiRect::new(4.0, 566.0, 64.0, 30.0);
pub const USER_EQUIP_POTION_VALUE_RECT: UserEquipUiRect =
    UserEquipUiRect::new(4.0, 596.0, 64.0, 30.0);
pub const USER_EQUIP_TAROS_BACK_RECT: UserEquipUiRect =
    UserEquipUiRect::new(20.0, 560.0, 149.0, 32.0);
#[must_use]
pub const fn user_equip_taros_digit_rect(index: usize) -> UserEquipUiRect {
    UserEquipUiRect::new(22.0 + index as f32 * 12.0, 564.0, 12.0, 20.0)
}

#[must_use]
pub fn user_equip_taros_digits(value: i64) -> String {
    format!("{:09}", value.clamp(0, 999_999_999))
}
pub const USER_EQUIP_DEXLABS_RECT: UserEquipUiRect =
    UserEquipUiRect::new(170.0, 561.0, 203.0, 67.0);
pub const USER_EQUIP_STATUS_NAME_RECT: UserEquipUiRect =
    UserEquipUiRect::new(13.0, 529.0, 308.0, 40.0);
pub const USER_EQUIP_STATUS_LEVEL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(18.0, 560.0, 180.0, 20.0);
pub const USER_EQUIP_STATUS_HP_RECT: UserEquipUiRect =
    UserEquipUiRect::new(17.0, 583.0, 450.0, 20.0);
pub const USER_EQUIP_STATUS_FUSION_RECT: UserEquipUiRect =
    UserEquipUiRect::new(17.0, 605.0, 450.0, 20.0);
pub const USER_EQUIP_POPUP_RECT: UserEquipUiRect = UserEquipUiRect::new(35.0, 90.0, 310.0, 435.0);
/// Exact `GumPopup.rectNanoIcon` values from the primary managed assembly.
pub const USER_EQUIP_GUM_NANO_FRAME_RECTS: [UserEquipUiRect; 3] = [
    UserEquipUiRect::new(33.0, 160.0, 62.0, 62.0),
    UserEquipUiRect::new(125.0, 160.0, 62.0, 62.0),
    UserEquipUiRect::new(215.0, 160.0, 62.0, 62.0),
];
/// Exact `GumPopup.rectNanoGumBut` values from the primary managed assembly.
pub const USER_EQUIP_GUM_NANO_BUTTON_RECTS: [UserEquipUiRect; 3] = [
    UserEquipUiRect::new(31.0, 232.0, 66.0, 25.0),
    UserEquipUiRect::new(125.0, 232.0, 66.0, 25.0),
    UserEquipUiRect::new(215.0, 232.0, 66.0, 25.0),
];
pub const USER_EQUIP_HELP_PANEL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(35.0, 125.0, 310.0, 310.0);
pub const USER_EQUIP_STATUS_PANEL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(0.0, 517.0, 498.0, 115.0);
pub const USER_EQUIP_STATUS_HP_BACK_RECT: UserEquipUiRect =
    UserEquipUiRect::new(13.0, 585.0, 472.0, 17.0);
pub const USER_EQUIP_STATUS_HP_BAR_RECT: UserEquipUiRect =
    UserEquipUiRect::new(14.0, 586.0, 470.0, 15.0);
pub const USER_EQUIP_STATUS_FUSION_BACK_RECT: UserEquipUiRect =
    UserEquipUiRect::new(13.0, 607.0, 472.0, 17.0);
pub const USER_EQUIP_STATUS_FUSION_BAR_RECT: UserEquipUiRect =
    UserEquipUiRect::new(14.0, 608.0, 470.0, 15.0);
pub const USER_EQUIP_STATUS_GUIDE_RECT: UserEquipUiRect =
    UserEquipUiRect::new(350.0, 524.0, 139.0, 57.0);
pub const USER_EQUIP_STATUS_GUIDE_LABEL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(360.0, 533.0, 100.0, 20.0);
pub const USER_EQUIP_STATUS_GUIDE_NAME_RECT: UserEquipUiRect =
    UserEquipUiRect::new(360.0, 553.0, 100.0, 20.0);
pub const USER_EQUIP_STATUS_GUIDE_ICON_RECT: UserEquipUiRect =
    UserEquipUiRect::new(440.0, 533.0, 36.0, 36.0);
pub const USER_EQUIP_INVENTORY_SHADOW_A_RECT: UserEquipUiRect =
    UserEquipUiRect::new(3.0, 34.0, 350.0, 503.0);
pub const USER_EQUIP_INVENTORY_SHADOW_B_RECT: UserEquipUiRect =
    UserEquipUiRect::new(3.0, 34.0, 350.0, 503.0);
pub const USER_EQUIP_NANO_STATUS_RECTS: [UserEquipUiRect; 3] = [
    UserEquipUiRect::new(24.0, 215.0, 126.0, 69.0),
    UserEquipUiRect::new(335.0, 175.0, 126.0, 69.0),
    UserEquipUiRect::new(348.0, 350.0, 126.0, 69.0),
];
pub const USER_EQUIP_NANO_PREVIEW_RECTS: [UserEquipUiRect; 3] = [
    UserEquipUiRect::new(45.0, 110.0, 128.0, 128.0),
    UserEquipUiRect::new(355.0, 69.0, 128.0, 128.0),
    UserEquipUiRect::new(368.0, 244.0, 128.0, 128.0),
];
pub const USER_EQUIP_NANO_TYPE_RECTS: [UserEquipUiRect; 3] = [
    UserEquipUiRect::new(79.0, 180.0, 59.0, 61.0),
    UserEquipUiRect::new(389.0, 139.0, 59.0, 61.0),
    UserEquipUiRect::new(402.0, 314.0, 59.0, 61.0),
];
pub const USER_EQUIP_NANO_SLOT_LABEL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(5.0, 18.0, 48.0, 9.0);
pub const USER_EQUIP_NANO_NAME_RECT: UserEquipUiRect = UserEquipUiRect::new(5.0, 33.0, 118.0, 14.0);
pub const USER_EQUIP_NANO_ATTRIBUTE_RECT: UserEquipUiRect =
    UserEquipUiRect::new(5.0, 45.0, 92.0, 14.0);
pub const USER_EQUIP_NANO_SKILL_RECT: UserEquipUiRect =
    UserEquipUiRect::new(101.0, 44.0, 20.0, 20.0);
pub const USER_EQUIP_NANO_STAMINA_RECT: UserEquipUiRect =
    UserEquipUiRect::new(7.0, 59.0, 90.0, 4.0);
