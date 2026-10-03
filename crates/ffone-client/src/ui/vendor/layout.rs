use super::*;

pub const VENDOR_LABEL_LINE_HEIGHT: f32 = USER_EQUIP_REGULAR_FONT_LINE_HEIGHT;

pub const VENDOR_BUTTON_LINE_HEIGHT: f32 = 11.300_000_19;

pub const VENDOR_EQUIP_LINE_HEIGHT: f32 = USER_EQUIP_SMALL_FONT_LINE_HEIGHT;

pub const VENDOR_SERVICE_LINE_HEIGHT: f32 = USER_EQUIP_REGULAR_FONT_LINE_HEIGHT;

pub const VENDOR_REFERENCE_WIDTH: i32 = 1_020;

pub const VENDOR_REFERENCE_HEIGHT: i32 = 638;

pub const VENDOR_BACKPLATE_REFERENCE_WIDTH: i32 = 1_036;

pub const VENDOR_BACKPLATE_REFERENCE_HEIGHT: i32 = 653;

pub const VENDOR_PANEL_WIDTH: f32 = 498.0;

pub const VENDOR_PANEL_HEIGHT: f32 = 638.0;

pub const VENDOR_ROW_HEIGHT: f32 = 80.0;

pub const VENDOR_ROW_WIDTH: f32 = 433.0;

pub const VENDOR_ROW_VISUAL_HEIGHT: f32 = 75.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VendorUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl VendorUiRect {
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

impl From<UserEquipUiRect> for VendorUiRect {
    fn from(value: UserEquipUiRect) -> Self {
        Self::new(value.left, value.top, value.width, value.height)
    }
}

pub const VENDOR_NPC_PREVIEW_RECT: VendorUiRect = VendorUiRect::new(310.0, 0.0, 200.0, 150.0);

pub const VENDOR_DIALOG_RECT: VendorUiRect = VendorUiRect::new(11.0, 132.0, 486.0, 528.0);

pub const VENDOR_LIST_BACK_RECT: VendorUiRect = VendorUiRect::new(0.0, 20.0, 486.0, 485.0);

pub const VENDOR_TABLE_RECT: VendorUiRect = VendorUiRect::new(2.0, 32.0, 479.0, 410.0);

pub const VENDOR_LIST_VIEWPORT_RECT: VendorUiRect = VendorUiRect::new(10.0, 10.0, 464.0, 400.0);

pub const VENDOR_TABLE_SHADOW_RECT: VendorUiRect = VendorUiRect::new(2.0, 3.0, 450.0, 412.0);

pub const VENDOR_INFO_RECT: VendorUiRect = VendorUiRect::new(3.0, 9.0, 481.0, 87.0);

pub const VENDOR_TITLE_RECT: VendorUiRect = VendorUiRect::new(14.0, 10.0, 300.0, 20.0);

pub const VENDOR_SERVICE_RECT: VendorUiRect = VendorUiRect::new(14.0, 30.0, 350.0, 80.0);

pub const VENDOR_BUY_TAB_RECT: VendorUiRect = VendorUiRect::new(1.0, 0.0, 95.0, 27.0);

pub const VENDOR_BUY_TAB_HIT_RECT: VendorUiRect = VendorUiRect::new(10.0, 5.0, 80.0, 15.0);

pub const VENDOR_BUYBACK_TAB_RECT: VendorUiRect = VendorUiRect::new(54.0, 0.0, 169.0, 27.0);

pub const VENDOR_BUYBACK_TAB_HIT_RECT: VendorUiRect = VendorUiRect::new(90.0, 5.0, 90.0, 15.0);

pub const VENDOR_LIST_DIVIDER_RECT: VendorUiRect = VendorUiRect::new(0.0, 20.0, 486.0, 13.0);

pub const VENDOR_GO_TO_STUFF_RECT: VendorUiRect = VendorUiRect::new(313.0, 595.0, 161.0, 25.0);

pub const VENDOR_ROW_ITEM_BOX_RECT: VendorUiRect = VendorUiRect::new(4.0, 5.0, 66.0, 66.0);

pub const VENDOR_ROW_NAME_RECT: VendorUiRect = VendorUiRect::new(82.0, 8.0, 300.0, 20.0);

pub const VENDOR_ROW_LEVEL_RECT: VendorUiRect = VendorUiRect::new(82.0, 22.0, 300.0, 20.0);

pub const VENDOR_ROW_VEHICLE_SPEED_RECT: VendorUiRect = VendorUiRect::new(82.0, 37.0, 300.0, 20.0);

pub const VENDOR_ROW_PRICE_RECT: VendorUiRect = VendorUiRect::new(262.0, 45.0, 134.0, 20.0);

pub const VENDOR_ROW_PRICE_ICON_RECT: VendorUiRect = VendorUiRect::new(406.0, 40.0, 23.0, 24.0);

pub const VENDOR_SCROLL_UP_RECT: VendorUiRect = VendorUiRect::new(459.0, 42.0, 17.0, 12.0);

pub const VENDOR_SCROLL_DOWN_RECT: VendorUiRect = VendorUiRect::new(459.0, 430.0, 17.0, 12.0);

pub const VENDOR_SCROLL_TRACK_RECT: VendorUiRect = VendorUiRect::new(459.0, 54.0, 17.0, 376.0);

pub const VENDOR_PC_STUFF_INVENTORY_SHADOW_RECT: VendorUiRect =
    VendorUiRect::new(3.0, 33.0, 350.0, 503.0);

pub const VENDOR_PC_STUFF_DEXLABS_RECT: VendorUiRect = VendorUiRect::new(170.0, 561.0, 203.0, 67.0);

pub const VENDOR_PC_STUFF_TAROS_COUNTER_RECT: VendorUiRect =
    VendorUiRect::new(20.0, 560.0, 149.0, 32.0);

pub const VENDOR_PC_STUFF_REDEEM_CODE_RECT: VendorUiRect =
    VendorUiRect::new(15.0, 598.0, 149.0, 25.0);

pub const VENDOR_PC_STUFF_TAROS_DIGIT_RECTS: [VendorUiRect; 9] = [
    VendorUiRect::new(22.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(34.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(46.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(58.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(70.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(82.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(94.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(106.0, 564.0, 12.0, 20.0),
    VendorUiRect::new(118.0, 564.0, 12.0, 20.0),
];

pub const VENDOR_BATTERY_ICON_RECTS: [VendorUiRect; 2] = [
    VendorUiRect::new(32.0, 577.0, 30.0, 28.0),
    VendorUiRect::new(32.0, 606.0, 30.0, 28.0),
];

pub const VENDOR_BATTERY_LABEL_RECTS: [VendorUiRect; 2] = [
    VendorUiRect::new(0.0, 570.0, 60.0, 19.0),
    VendorUiRect::new(0.0, 599.0, 60.0, 19.0),
];

pub const VENDOR_BATTERY_COUNT_RECTS: [VendorUiRect; 2] = [
    VendorUiRect::new(4.0, 580.0, 64.0, 30.0),
    VendorUiRect::new(4.0, 610.0, 64.0, 30.0),
];

pub const VENDOR_LIST_BACK_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 14.0),
    max_inset: Vec2::new(6.0, 55.0),
};

pub const VENDOR_SCROLL_TRACK_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(2.0, 4.0),
    max_inset: Vec2::new(2.0, 4.0),
};

pub const VENDOR_SCROLL_SHADOW_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(0.0, 32.0),
    max_inset: Vec2::new(0.0, 32.0),
};

pub const VENDOR_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VendorModeLayout {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub opening_eased_fraction: f32,
    pub vendor_scroll_y: f32,
    pub inventory_scroll_y: f32,
    pub full_backdrop: VendorUiRect,
    pub vendor_backplate: VendorUiRect,
    pub right_backplate: VendorUiRect,
    pub vendor_panel: VendorUiRect,
    pub npc_preview_boundary: VendorUiRect,
    pub vendor_info: VendorUiRect,
    pub vendor_title: VendorUiRect,
    pub vendor_service: VendorUiRect,
    pub vendor_dialog: VendorUiRect,
    pub list_back: VendorUiRect,
    pub list_divider: VendorUiRect,
    pub buy_tab: VendorUiRect,
    pub buy_tab_hit: VendorUiRect,
    pub buyback_tab: VendorUiRect,
    pub buyback_tab_hit: VendorUiRect,
    pub table: VendorUiRect,
    pub list_viewport: VendorUiRect,
    pub list_content: VendorUiRect,
    pub table_shadow: VendorUiRect,
    pub scroll_track: VendorUiRect,
    pub scroll_thumb: VendorUiRect,
    pub scroll_up: VendorUiRect,
    pub scroll_down: VendorUiRect,
    pub go_to_stuff: VendorUiRect,
    pub item_mode: UserEquipItemModeLayout,
}

impl VendorModeLayout {
    #[must_use]
    pub fn row_rect(self, row_index: usize) -> Option<VendorUiRect> {
        if row_index >= VENDOR_CATALOG_CAPACITY {
            return None;
        }
        Some(VendorUiRect::new(
            self.list_content.left,
            self.list_content.top + row_index as f32 * VENDOR_ROW_HEIGHT,
            VENDOR_ROW_WIDTH,
            VENDOR_ROW_VISUAL_HEIGHT,
        ))
    }

    #[must_use]
    pub fn scroll_thumb_in_dialog(self) -> VendorUiRect {
        self.scroll_thumb
            .translated(-self.vendor_dialog.left, -self.vendor_dialog.top)
    }
}

#[must_use]
pub fn vendor_mode_layout(
    viewport_width: u32,
    viewport_height: u32,
    opening_elapsed_seconds: f32,
    vendor_scroll_y: f32,
    inventory_scroll_y: f32,
    vendor_row_count: usize,
) -> VendorModeLayout {
    let viewport_width = viewport_width.min(i32::MAX as u32) as i32;
    let viewport_height = viewport_height.min(i32::MAX as u32) as i32;
    let center_x = if viewport_width > VENDOR_REFERENCE_WIDTH {
        (viewport_width - VENDOR_REFERENCE_WIDTH) / 2
    } else {
        0
    };
    let top = ((viewport_height - VENDOR_REFERENCE_HEIGHT) / 2).max(0);
    let eased = vendor_opening_eased_fraction(opening_elapsed_seconds);
    let remaining = 1.0 - eased;
    let vendor_x = center_x - ((center_x - VENDOR_PANEL_START_X) as f32 * remaining) as i32;

    let backplate_left = (viewport_width - VENDOR_BACKPLATE_REFERENCE_WIDTH) / 2;
    let backplate_top = (viewport_height - VENDOR_BACKPLATE_REFERENCE_HEIGHT) / 2;
    let full_backdrop = VendorUiRect::new(
        ((viewport_width - USER_EQUIP_BACKDROP_WIDTH as i32) / 2) as f32,
        ((viewport_height - USER_EQUIP_BACKDROP_HEIGHT as i32) / 2) as f32,
        USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_BACKDROP_HEIGHT,
    );
    let vendor_backplate =
        VendorUiRect::new(backplate_left as f32, backplate_top as f32, 585.0, 653.0);
    let right_backplate = VendorUiRect::new(
        (backplate_left + 585) as f32,
        backplate_top as f32,
        451.0,
        653.0,
    );
    let vendor_panel = VendorUiRect::new(
        vendor_x as f32,
        top as f32,
        VENDOR_PANEL_WIDTH,
        VENDOR_PANEL_HEIGHT,
    );
    let npc_preview_boundary =
        VENDOR_NPC_PREVIEW_RECT.translated(vendor_panel.left, vendor_panel.top);
    let vendor_info = VENDOR_INFO_RECT.translated(vendor_panel.left, vendor_panel.top);
    let vendor_title = VENDOR_TITLE_RECT.translated(vendor_panel.left, vendor_panel.top);
    let vendor_service = VENDOR_SERVICE_RECT.translated(vendor_panel.left, vendor_panel.top);
    let vendor_dialog = VENDOR_DIALOG_RECT.translated(vendor_panel.left, vendor_panel.top);
    let list_back = VENDOR_LIST_BACK_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let list_divider = VENDOR_LIST_DIVIDER_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let buy_tab = VENDOR_BUY_TAB_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let buy_tab_hit = VENDOR_BUY_TAB_HIT_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let buyback_tab = VENDOR_BUYBACK_TAB_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let buyback_tab_hit =
        VENDOR_BUYBACK_TAB_HIT_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let table = VENDOR_TABLE_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let list_viewport = VENDOR_LIST_VIEWPORT_RECT.translated(table.left, table.top);
    let vendor_scroll_y = clamp_vendor_scroll(vendor_scroll_y, vendor_row_count);
    let list_content = VendorUiRect::new(
        list_viewport.left,
        list_viewport.top - vendor_scroll_y,
        VENDOR_ROW_WIDTH,
        vendor_row_count as f32 * VENDOR_ROW_HEIGHT,
    );
    let table_shadow = VENDOR_TABLE_SHADOW_RECT.translated(table.left, table.top);
    let scroll_track = VENDOR_SCROLL_TRACK_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let scroll_up = VENDOR_SCROLL_UP_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let scroll_down = VENDOR_SCROLL_DOWN_RECT.translated(vendor_dialog.left, vendor_dialog.top);
    let extent = crate::service_scroll::thumb_extent(
        VENDOR_SCROLL_TRACK_RECT.height,
        VENDOR_LIST_VIEWPORT_RECT.height,
        vendor_scroll_max(vendor_row_count),
    );
    let track_travel = (VENDOR_SCROLL_TRACK_RECT.height - extent).max(0.0);
    let max_scroll = vendor_scroll_max(vendor_row_count);
    let thumb_fraction = if max_scroll > 0.0 {
        vendor_scroll_y / max_scroll
    } else {
        0.0
    };
    let scroll_thumb = VendorUiRect::new(
        vendor_dialog.left + VENDOR_SCROLL_TRACK_RECT.left,
        vendor_dialog.top + VENDOR_SCROLL_TRACK_RECT.top + track_travel * thumb_fraction,
        15.0,
        extent,
    );
    let go_to_stuff = VENDOR_GO_TO_STUFF_RECT.translated(vendor_panel.left, vendor_panel.top);
    let item_mode = user_equip_item_mode_layout(
        viewport_width as u32,
        viewport_height as u32,
        opening_elapsed_seconds,
        inventory_scroll_y,
    );

    VendorModeLayout {
        viewport_width,
        viewport_height,
        opening_eased_fraction: eased,
        vendor_scroll_y,
        inventory_scroll_y: item_mode.scroll_y,
        full_backdrop,
        vendor_backplate,
        right_backplate,
        vendor_panel,
        npc_preview_boundary,
        vendor_info,
        vendor_title,
        vendor_service,
        vendor_dialog,
        list_back,
        list_divider,
        buy_tab,
        buy_tab_hit,
        buyback_tab,
        buyback_tab_hit,
        table,
        list_viewport,
        list_content,
        table_shadow,
        scroll_track,
        scroll_thumb,
        scroll_up,
        scroll_down,
        go_to_stuff,
        item_mode,
    }
}

pub(super) fn bind_vendor_rect(node: &mut Node, rect: VendorUiRect) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
