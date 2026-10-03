use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EnchantUiRect0104 {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl EnchantUiRect0104 {
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

    #[must_use]
    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.left
            && point.y >= self.top
            && point.x < self.left + self.width
            && point.y < self.top + self.height
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

pub const ENCHANT_REFERENCE_WIDTH: i32 = 1_036;

pub const ENCHANT_REFERENCE_HEIGHT: i32 = 653;

pub const ENCHANT_MAIN_GROUP_WIDTH: f32 = 585.0;

pub const ENCHANT_MAIN_GROUP_HEIGHT: f32 = 653.0;

pub const ENCHANT_RIGHT_BACKPLATE_WIDTH: f32 = 451.0;

pub const ENCHANT_PANEL_RECT: EnchantUiRect0104 = EnchantUiRect0104::new(15.0, 19.0, 481.0, 600.0);

pub const ENCHANT_TITLE_RECT: EnchantUiRect0104 = EnchantUiRect0104::new(28.0, 28.0, 299.0, 16.0);

pub const ENCHANT_INTRO_RECT: EnchantUiRect0104 = EnchantUiRect0104::new(28.0, 38.0, 327.0, 49.0);

pub const ENCHANT_PRIMARY_NPC_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(355.0, 2.0, 148.0, 147.0);

pub const ENCHANT_TARGET_TITLE_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(28.0, 155.0, 150.0, 20.0);

pub const ENCHANT_NEEDED_TITLE_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(28.0, 268.0, 110.0, 20.0);

pub const ENCHANT_HELP_TITLE_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(28.0, 390.0, 110.0, 20.0);

pub const ENCHANT_TARGET_SLOT_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(28.0, 189.0, 64.0, 64.0);

pub const ENCHANT_WEAPON_SLOT_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(28.0, 302.0, 64.0, 64.0);

pub const ENCHANT_ARMOR_SLOT_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(273.0, 302.0, 64.0, 64.0);

pub const ENCHANT_HELP_1_SLOT_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(28.0, 425.0, 64.0, 64.0);

pub const ENCHANT_HELP_2_SLOT_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(273.0, 425.0, 64.0, 64.0);

pub const ENCHANT_CHANCE_TITLE_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 519.0, 143.0, 25.0);

pub const ENCHANT_CHANCE_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 545.0, 143.0, 20.0);

pub const ENCHANT_TAROS_TITLE_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(365.0, 519.0, 112.0, 25.0);

pub const ENCHANT_TAROS_RECT: EnchantUiRect0104 = EnchantUiRect0104::new(365.0, 545.0, 112.0, 20.0);

pub const ENCHANT_PREVIEW_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(24.0, 585.0, 142.0, 24.0);

pub const ENCHANT_CLEAR_RECT: EnchantUiRect0104 = EnchantUiRect0104::new(185.0, 585.0, 142.0, 24.0);

pub const ENCHANT_ACTION_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(345.0, 585.0, 142.0, 24.0);

pub const ENCHANT_RAW_COVER_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(16.0, 266.0, 479.0, 237.0);

pub const ENCHANT_EMPTY_PROMPT_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(115.0, 384.0, 280.0, 20.0);

pub const ENCHANT_WAITING_GROUP_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(0.0, 0.0, 434.0, 435.0);

pub const ENCHANT_WAITING_LABEL_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(25.0, 20.0, 170.0, 22.0);

pub const ENCHANT_WAITING_NPC_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(18.0, 69.0, 403.0, 288.0);

pub const ENCHANT_WAIT_PROGRESS_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(12.0, 386.0, 414.0, 13.0);

pub const ENCHANT_SUCCESS_GROUP_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(0.0, 0.0, 352.0, 550.0);

pub const ENCHANT_SUCCESS_MORE_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(70.0, 478.0, 210.0, 25.0);

pub const ENCHANT_SUCCESS_STUFF_RECT: EnchantUiRect0104 =
    EnchantUiRect0104::new(70.0, 513.0, 210.0, 25.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnchantModeLayout0104 {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub inventory_opening_eased_fraction: f32,
    pub inventory_scroll_y: f32,
    pub full_backdrop: EnchantUiRect0104,
    pub main_group: EnchantUiRect0104,
    pub right_backplate: EnchantUiRect0104,
    pub pc_stuff_panel: EnchantUiRect0104,
    pub equipment_panel: EnchantUiRect0104,
    pub shade: EnchantUiRect0104,
    pub success_group: EnchantUiRect0104,
    pub waiting_group: EnchantUiRect0104,
}

#[must_use]
pub fn enchant_mode_layout_0104(
    viewport_width: u32,
    viewport_height: u32,
) -> EnchantModeLayout0104 {
    enchant_mode_animated_layout_0104(
        viewport_width,
        viewport_height,
        ENCHANT_INVENTORY_OPEN_SECONDS_0104,
        0.0,
    )
}

#[must_use]
pub fn enchant_mode_animated_layout_0104(
    viewport_width: u32,
    viewport_height: u32,
    opening_elapsed_seconds: f32,
    inventory_scroll_y: f32,
) -> EnchantModeLayout0104 {
    let width = viewport_width.min(i32::MAX as u32) as i32;
    let height = viewport_height.min(i32::MAX as u32) as i32;
    let shell_left = (width - ENCHANT_REFERENCE_WIDTH) / 2;
    let shell_top = (height - ENCHANT_REFERENCE_HEIGHT) / 2;
    let inventory_center = if width > 1_020 {
        (width - 1_020) / 2
    } else {
        0
    };
    let inventory_top = ((height - 638) / 2).max(0);
    let eased = enchant_inventory_opening_eased_fraction_0104(opening_elapsed_seconds);
    let remaining = 1.0 - eased;
    let pc_stuff_final_x = 585 + inventory_center;
    let pc_stuff_x = pc_stuff_final_x + ((1_020 - pc_stuff_final_x) as f32 * remaining) as i32;
    let equipment_final_x = 504 + inventory_center;
    let equipment_x = equipment_final_x + ((1_020 - equipment_final_x) as f32 * remaining) as i32;
    let inventory_scroll_y = clamp_enchant_inventory_scroll_0104(inventory_scroll_y);
    EnchantModeLayout0104 {
        viewport_width: width,
        viewport_height: height,
        inventory_opening_eased_fraction: eased,
        inventory_scroll_y,
        full_backdrop: EnchantUiRect0104::new(
            ((width - 1_920) / 2) as f32,
            ((height - 1_440) / 2) as f32,
            1_920.0,
            1_440.0,
        ),
        main_group: EnchantUiRect0104::new(
            shell_left as f32,
            shell_top as f32,
            ENCHANT_MAIN_GROUP_WIDTH,
            ENCHANT_MAIN_GROUP_HEIGHT,
        ),
        right_backplate: EnchantUiRect0104::new(
            shell_left as f32 + ENCHANT_MAIN_GROUP_WIDTH,
            shell_top as f32,
            ENCHANT_RIGHT_BACKPLATE_WIDTH,
            ENCHANT_MAIN_GROUP_HEIGHT,
        ),
        pc_stuff_panel: EnchantUiRect0104::new(
            pc_stuff_x as f32,
            inventory_top as f32,
            380.0,
            632.0,
        ),
        equipment_panel: EnchantUiRect0104::new(
            equipment_x as f32,
            inventory_top as f32,
            66.0,
            639.0,
        ),
        shade: EnchantUiRect0104::new(0.0, 0.0, width.max(0) as f32, height.max(0) as f32),
        success_group: EnchantUiRect0104::new(
            (width - 352) as f32 * 0.5,
            (height - 550) as f32 * 0.5,
            352.0,
            550.0,
        ),
        waiting_group: EnchantUiRect0104::new(
            (width - 434) as f32 * 0.5,
            (height - 435) as f32 * 0.5,
            434.0,
            435.0,
        ),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantReplyDisposition0104 {
    SuccessOverlay,
    FailureMessage,
    IgnoredLegacyFlag,
}

pub const ENCHANT_INVENTORY_PANEL_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(0.0, 0.0, 380.0, 632.0);

pub const ENCHANT_INVENTORY_VIEWPORT_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(6.0, 36.0, 366.0, 500.0);

pub const ENCHANT_EQUIPMENT_LOCAL_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(0.0, 0.0, 66.0, 639.0);

pub const ENCHANT_EQUIPMENT_TITLE_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(0.0, 3.0, 64.0, 13.0);

pub const ENCHANT_CLOSE_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(400.0, 5.0, 30.0, 30.0);

pub const ENCHANT_TRASH_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(390.0, 555.0, 32.0, 32.0);

pub const ENCHANT_HELP_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(390.0, 590.0, 32.0, 32.0);

pub const ENCHANT_DEXLABS_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(170.0, 561.0, 203.0, 67.0);

pub const ENCHANT_TAROS_COUNTER_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(20.0, 560.0, 149.0, 32.0);

pub const ENCHANT_REDEEM_CODE_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(15.0, 598.0, 149.0, 25.0);

pub const ENCHANT_TAROS_DIGIT_RECTS_0104: [EnchantUiRect0104; 9] = [
    EnchantUiRect0104::new(22.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(34.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(46.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(58.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(70.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(82.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(94.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(106.0, 564.0, 12.0, 20.0),
    EnchantUiRect0104::new(118.0, 564.0, 12.0, 20.0),
];

pub const ENCHANT_BATTERY_SLOT_RECTS_0104: [EnchantUiRect0104; 2] = [
    EnchantUiRect0104::new(0.0, 572.0, 64.0, 30.0),
    EnchantUiRect0104::new(0.0, 602.0, 64.0, 30.0),
];

pub const ENCHANT_BATTERY_ICON_RECTS_0104: [EnchantUiRect0104; 2] = [
    EnchantUiRect0104::new(32.0, 577.0, 30.0, 28.0),
    EnchantUiRect0104::new(32.0, 606.0, 30.0, 28.0),
];

pub const ENCHANT_BATTERY_LABEL_RECTS_0104: [EnchantUiRect0104; 2] = [
    EnchantUiRect0104::new(0.0, 570.0, 60.0, 19.0),
    EnchantUiRect0104::new(0.0, 599.0, 60.0, 19.0),
];

pub const ENCHANT_BATTERY_COUNT_RECTS_0104: [EnchantUiRect0104; 2] = [
    EnchantUiRect0104::new(4.0, 580.0, 64.0, 30.0),
    EnchantUiRect0104::new(4.0, 610.0, 64.0, 30.0),
];

pub const ENCHANT_TARGET_NAME_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 192.0, 100.0, 60.0);

pub const ENCHANT_TARGET_DESCRIPTION_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 219.0, 380.0, 40.0);

pub const ENCHANT_TARGET_COMBINED_BADGE_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(64.0, 225.0, 26.0, 26.0);

pub const ENCHANT_TARGET_LEVEL_BADGE_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(28.0, 241.0, 25.0, 12.0);

pub const ENCHANT_WEAPON_NAME_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 304.0, 100.0, 60.0);

pub const ENCHANT_WEAPON_DESCRIPTION_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 331.0, 130.0, 40.0);

pub const ENCHANT_ARMOR_NAME_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(345.0, 304.0, 100.0, 60.0);

pub const ENCHANT_ARMOR_DESCRIPTION_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(345.0, 331.0, 130.0, 40.0);

pub const ENCHANT_HELP_1_NAME_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 427.0, 100.0, 60.0);

pub const ENCHANT_HELP_1_DESCRIPTION_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 451.0, 130.0, 40.0);

pub const ENCHANT_HELP_2_NAME_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(345.0, 427.0, 100.0, 60.0);

pub const ENCHANT_HELP_2_DESCRIPTION_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(345.0, 451.0, 130.0, 40.0);

pub const ENCHANT_WEAPON_QUANTITY_COVER_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(95.0, 300.0, 147.0, 68.0);

pub const ENCHANT_ARMOR_QUANTITY_COVER_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(340.0, 300.0, 147.0, 68.0);

pub const ENCHANT_WEAPON_QUANTITY_WARNING_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(115.0, 326.0, 120.0, 37.0);

pub const ENCHANT_ARMOR_QUANTITY_WARNING_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(365.0, 326.0, 120.0, 37.0);

pub const ENCHANT_DEAD_ITEM_COVER_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(100.0, 190.0, 385.0, 68.0);

pub const ENCHANT_DEAD_WARNING_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(142.0, 200.0, 325.0, 46.0);

pub const ENCHANT_SUCCESS_NPC_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(10.0, 10.0, 64.0, 64.0);

pub const ENCHANT_SUCCESS_HOORAY_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(80.0, 7.0, 98.0, 18.0);

pub const ENCHANT_SUCCESS_MESSAGE_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(80.0, 28.0, 250.0, 36.0);

pub const ENCHANT_SUCCESS_ICON_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(38.0, 102.0, 60.0, 60.0);

pub const ENCHANT_SUCCESS_LEVEL_BADGE_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(38.0, 151.0, 25.0, 12.0);

pub const ENCHANT_SUCCESS_NAME_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(106.0, 101.0, 201.0, 25.0);

pub const ENCHANT_SUCCESS_LEVEL_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(106.0, 138.0, 81.0, 14.0);

pub const ENCHANT_SUCCESS_DESCRIPTION_RECT_0104: EnchantUiRect0104 =
    EnchantUiRect0104::new(32.0, 180.0, 285.0, 40.0);

pub const ENCHANT_BUTTON_BORDER_0104: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

pub const ENCHANT_INVENTORY_PANEL_BORDER_0104: BorderRect = BorderRect {
    min_inset: Vec2::new(136.0, 33.0),
    max_inset: Vec2::new(10.0, 77.0),
};

pub const ENCHANT_RIGHT_PANEL_BORDER_0104: BorderRect = BorderRect {
    min_inset: Vec2::new(5.0, 0.0),
    max_inset: Vec2::new(5.0, 0.0),
};

pub(super) fn enchant_bind_rect_0104(node: &mut Node, rect: EnchantUiRect0104) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
