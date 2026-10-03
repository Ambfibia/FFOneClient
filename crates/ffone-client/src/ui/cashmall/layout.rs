use super::*;

pub const CASHMALL_REFERENCE_WIDTH: i32 = 1_020;

pub const CASHMALL_REFERENCE_HEIGHT: i32 = 638;

pub const CASHMALL_BACKPLATE_REFERENCE_WIDTH: i32 = 1_036;

pub const CASHMALL_BACKPLATE_REFERENCE_HEIGHT: i32 = 653;

pub const CASHMALL_PANEL_WIDTH: f32 = 498.0;

pub const CASHMALL_PANEL_HEIGHT: f32 = 638.0;

pub const CASHMALL_ROW_HEIGHT: f32 = 80.0;

pub const CASHMALL_ROW_WIDTH: f32 = 433.0;

pub const CASHMALL_ROW_VISUAL_HEIGHT: f32 = 75.0;

pub const CASHMALL_LEGACY_SCROLL_CONTENT_HEIGHT: f32 =
    CASHMALL_CACHED_ITEM_COUNT_0104 as f32 * CASHMALL_ROW_HEIGHT;

pub const CASHMALL_VENDOR_POPUP_RECT: [i32; 4] = [-1, -1, 0, 0];

pub const CASHMALL_LABEL_FONT_LINE_HEIGHT: f32 = 13.560_000_42;

pub const CASHMALL_BUTTON_FONT_LINE_HEIGHT: f32 = 11.300_000_19;

pub const CASHMALL_SMALL_FONT_LINE_HEIGHT: f32 = 6.780_000_21;

pub const CASHMALL_FIRST_TAB_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 0.0),
    max_inset: Vec2::new(32.0, 0.0),
};

pub const CASHMALL_SECOND_TAB_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(36.0, 0.0),
    max_inset: Vec2::new(45.0, 0.0),
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CashmallUiRect0104 {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl CashmallUiRect0104 {
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

impl From<crate::user_equip_ui::UserEquipUiRect> for CashmallUiRect0104 {
    fn from(value: crate::user_equip_ui::UserEquipUiRect) -> Self {
        Self::new(value.left, value.top, value.width, value.height)
    }
}

pub const CASHMALL_DIALOG_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(11.0, 132.0, 486.0, 528.0);

pub const CASHMALL_LIST_BACK_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(0.0, 20.0, 486.0, 485.0);

pub const CASHMALL_TABLE_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(2.0, 32.0, 479.0, 410.0);

pub const CASHMALL_LIST_VIEWPORT_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(10.0, 10.0, 464.0, 400.0);

pub const CASHMALL_TABLE_SHADOW_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(2.0, 3.0, 450.0, 412.0);

pub const CASHMALL_INFO_RECT: CashmallUiRect0104 = CashmallUiRect0104::new(3.0, 9.0, 481.0, 87.0);

pub const CASHMALL_TITLE_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(14.0, 10.0, 150.0, 20.0);

pub const CASHMALL_NPC_NAME_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(14.0, 30.0, 300.0, 20.0);

pub const CASHMALL_CASH_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(300.0, 50.0, 147.0, 31.0);

pub const CASHMALL_LIST_DIVIDER_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(0.0, 20.0, 486.0, 13.0);

pub const CASHMALL_GO_TO_STUFF_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(313.0, 595.0, 161.0, 25.0);

pub const CASHMALL_ROW_ITEM_BOX_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(4.0, 5.0, 66.0, 66.0);

pub const CASHMALL_ROW_NAME_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(82.0, 8.0, 300.0, 20.0);

pub const CASHMALL_ROW_LEVEL_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(82.0, 22.0, 300.0, 20.0);

pub const CASHMALL_ROW_PRICE_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(262.0, 50.0, 134.0, 20.0);

pub const CASHMALL_SCROLL_UP_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(456.0, 42.0, 17.0, 12.0);

pub const CASHMALL_SCROLL_DOWN_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(456.0, 418.0, 17.0, 12.0);

pub const CASHMALL_SCROLL_TRACK_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(456.0, 54.0, 18.0, 364.0);

pub const CASHMALL_TAB_RECTS: [CashmallUiRect0104; CASHMALL_TAB_COUNT] = [
    CashmallUiRect0104::new(1.0, 2.0, 80.0, 27.0),
    CashmallUiRect0104::new(40.0, 2.0, 130.0, 27.0),
    CashmallUiRect0104::new(120.0, 2.0, 130.0, 27.0),
    CashmallUiRect0104::new(200.0, 2.0, 150.0, 27.0),
    CashmallUiRect0104::new(300.0, 2.0, 130.0, 27.0),
];

pub const CASHMALL_TAB_HIT_RECTS: [CashmallUiRect0104; CASHMALL_TAB_COUNT] = [
    CashmallUiRect0104::new(10.0, 5.0, 30.0, 15.0),
    CashmallUiRect0104::new(76.0, 5.0, 100.0, 15.0),
    CashmallUiRect0104::new(156.0, 5.0, 100.0, 15.0),
    CashmallUiRect0104::new(236.0, 5.0, 100.0, 15.0),
    CashmallUiRect0104::new(336.0, 5.0, 100.0, 15.0),
];

pub const CASHMALL_LIST_BACK_BORDER: BorderRect = VENDOR_LIST_BACK_BORDER;

pub const CASHMALL_SCROLL_SHADOW_BORDER: BorderRect = VENDOR_SCROLL_SHADOW_BORDER;

pub const CASHMALL_BUTTON_BORDER: BorderRect = VENDOR_BUTTON_BORDER;

pub const CASHMALL_PC_STUFF_INVENTORY_PANEL_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(0.0, 0.0, 380.0, 632.0);

pub const CASHMALL_PC_STUFF_NANO_TAB_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(96.0, 0.0, 129.0, 29.0);

pub const CASHMALL_PC_STUFF_NANO_TAB_HIT_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(135.0, 5.0, 60.0, 15.0);

pub const CASHMALL_PC_STUFF_DEXLABS_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(170.0, 561.0, 203.0, 67.0);

pub const CASHMALL_PC_STUFF_TAROS_COUNTER_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(20.0, 560.0, 149.0, 32.0);

pub const CASHMALL_PC_STUFF_REDEEM_CODE_RECT: CashmallUiRect0104 =
    CashmallUiRect0104::new(15.0, 598.0, 149.0, 25.0);

pub const CASHMALL_PC_STUFF_TAROS_DIGIT_RECTS: [CashmallUiRect0104; 9] = [
    CashmallUiRect0104::new(22.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(34.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(46.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(58.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(70.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(82.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(94.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(106.0, 564.0, 12.0, 20.0),
    CashmallUiRect0104::new(118.0, 564.0, 12.0, 20.0),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CashmallModeLayout0104 {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub opening_eased_fraction: f32,
    pub full_backdrop: CashmallUiRect0104,
    pub cashmall_backplate: CashmallUiRect0104,
    pub right_backplate: CashmallUiRect0104,
    pub cashmall_panel: CashmallUiRect0104,
    pub info: CashmallUiRect0104,
    pub title: CashmallUiRect0104,
    pub npc_name: CashmallUiRect0104,
    pub cash: CashmallUiRect0104,
    pub dialog: CashmallUiRect0104,
    pub list_back: CashmallUiRect0104,
    pub list_divider: CashmallUiRect0104,
    pub table: CashmallUiRect0104,
    pub list_viewport: CashmallUiRect0104,
    pub list_content: CashmallUiRect0104,
    pub table_shadow: CashmallUiRect0104,
    pub scroll_track: CashmallUiRect0104,
    pub scroll_thumb: CashmallUiRect0104,
    pub scroll_up: CashmallUiRect0104,
    pub scroll_down: CashmallUiRect0104,
    pub go_to_stuff: CashmallUiRect0104,
    pub item_mode: UserEquipItemModeLayout,
}

impl CashmallModeLayout0104 {
    #[must_use]
    pub fn row_rect(self, row_index: usize) -> Option<CashmallUiRect0104> {
        if row_index >= CASHMALL_SLOT_SCAN_COUNT {
            return None;
        }
        Some(CashmallUiRect0104::new(
            self.list_content.left,
            self.list_content.top + row_index as f32 * CASHMALL_ROW_HEIGHT,
            CASHMALL_ROW_WIDTH,
            CASHMALL_ROW_VISUAL_HEIGHT,
        ))
    }

    #[must_use]
    pub fn scroll_thumb_in_dialog(self) -> CashmallUiRect0104 {
        self.scroll_thumb
            .translated(-self.dialog.left, -self.dialog.top)
    }
}

#[must_use]
pub fn cashmall_mode_layout_0104(
    viewport_width: u32,
    viewport_height: u32,
    opening_elapsed_seconds: f32,
    inventory_scroll_y: f32,
) -> CashmallModeLayout0104 {
    let viewport_width = viewport_width.min(i32::MAX as u32) as i32;
    let viewport_height = viewport_height.min(i32::MAX as u32) as i32;
    let center_x = if viewport_width > CASHMALL_REFERENCE_WIDTH {
        (viewport_width - CASHMALL_REFERENCE_WIDTH) / 2
    } else {
        0
    };
    let top = ((viewport_height - CASHMALL_REFERENCE_HEIGHT) / 2).max(0);
    let eased = cashmall_opening_eased_fraction_0104(opening_elapsed_seconds);
    let remaining = 1.0 - eased;
    let panel_x = center_x - ((center_x + CASHMALL_PANEL_WIDTH as i32) as f32 * remaining) as i32;
    let backplate_left = (viewport_width - CASHMALL_BACKPLATE_REFERENCE_WIDTH) / 2;
    let backplate_top = (viewport_height - CASHMALL_BACKPLATE_REFERENCE_HEIGHT) / 2;

    // Direct `GUI.DrawTexture(..., ScaleMode.ScaleAndCrop)` covers the full
    // screen without distorting the 1920x1440 source texture.
    let backdrop_scale = (viewport_width as f32 / USER_EQUIP_BACKDROP_WIDTH)
        .max(viewport_height as f32 / USER_EQUIP_BACKDROP_HEIGHT);
    let backdrop_width = USER_EQUIP_BACKDROP_WIDTH * backdrop_scale;
    let backdrop_height = USER_EQUIP_BACKDROP_HEIGHT * backdrop_scale;
    let full_backdrop = CashmallUiRect0104::new(
        (viewport_width as f32 - backdrop_width) * 0.5,
        (viewport_height as f32 - backdrop_height) * 0.5,
        backdrop_width,
        backdrop_height,
    );
    let cashmall_backplate =
        CashmallUiRect0104::new(backplate_left as f32, backplate_top as f32, 585.0, 653.0);
    let right_backplate = CashmallUiRect0104::new(
        (backplate_left + 585) as f32,
        backplate_top as f32,
        451.0,
        653.0,
    );
    let cashmall_panel = CashmallUiRect0104::new(
        panel_x as f32,
        top as f32,
        CASHMALL_PANEL_WIDTH,
        CASHMALL_PANEL_HEIGHT,
    );
    let info = CASHMALL_INFO_RECT.translated(cashmall_panel.left, cashmall_panel.top);
    let title = CASHMALL_TITLE_RECT.translated(info.left, info.top);
    let npc_name = CASHMALL_NPC_NAME_RECT.translated(info.left, info.top);
    let cash = CASHMALL_CASH_RECT.translated(info.left, info.top);
    let dialog = CASHMALL_DIALOG_RECT.translated(cashmall_panel.left, cashmall_panel.top);
    let list_back = CASHMALL_LIST_BACK_RECT.translated(dialog.left, dialog.top);
    let list_divider = CASHMALL_LIST_DIVIDER_RECT.translated(dialog.left, dialog.top);
    let table = CASHMALL_TABLE_RECT.translated(dialog.left, dialog.top);
    let list_viewport = CASHMALL_LIST_VIEWPORT_RECT.translated(table.left, table.top);
    let list_content = CashmallUiRect0104::new(
        list_viewport.left,
        list_viewport.top,
        CASHMALL_ROW_WIDTH,
        CASHMALL_LEGACY_SCROLL_CONTENT_HEIGHT,
    );
    let table_shadow = CASHMALL_TABLE_SHADOW_RECT.translated(table.left, table.top);
    let scroll_track = CASHMALL_SCROLL_TRACK_RECT.translated(dialog.left, dialog.top);
    let scroll_thumb = CashmallUiRect0104::new(
        dialog.left + CASHMALL_SCROLL_TRACK_RECT.left + 2.0,
        dialog.top + CASHMALL_SCROLL_TRACK_RECT.top,
        13.0,
        15.0,
    );
    let scroll_up = CASHMALL_SCROLL_UP_RECT.translated(dialog.left, dialog.top);
    let scroll_down = CASHMALL_SCROLL_DOWN_RECT.translated(dialog.left, dialog.top);
    let go_to_stuff = CASHMALL_GO_TO_STUFF_RECT.translated(cashmall_panel.left, cashmall_panel.top);
    let item_mode = user_equip_item_mode_layout(
        viewport_width as u32,
        viewport_height as u32,
        opening_elapsed_seconds,
        inventory_scroll_y,
    );

    CashmallModeLayout0104 {
        viewport_width,
        viewport_height,
        opening_eased_fraction: eased,
        full_backdrop,
        cashmall_backplate,
        right_backplate,
        cashmall_panel,
        info,
        title,
        npc_name,
        cash,
        dialog,
        list_back,
        list_divider,
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

pub(super) fn bind_cashmall_rect_0104(node: &mut Node, rect: CashmallUiRect0104) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}

#[must_use]
pub const fn cashmall_tab_border_0104(tab: CashmallTab0104) -> BorderRect {
    if matches!(tab, CashmallTab0104::New) {
        CASHMALL_FIRST_TAB_BORDER
    } else {
        CASHMALL_SECOND_TAB_BORDER
    }
}
