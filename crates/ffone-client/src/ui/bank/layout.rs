use super::*;

pub const BANK_REFERENCE_WIDTH: i32 = 1_020;

pub const BANK_REFERENCE_HEIGHT: i32 = 638;

pub const BANK_BACKPLATE_REFERENCE_WIDTH: i32 = 1_036;

pub const BANK_BACKPLATE_REFERENCE_HEIGHT: i32 = 653;

pub const BANK_PANEL_WIDTH: f32 = 495.0;

pub const BANK_PANEL_HEIGHT: f32 = 638.0;

pub const BANK_CONTENT_WIDTH: f32 = 402.0;

pub const BANK_CONTENT_HEIGHT: f32 = 2_278.0;

pub const BANK_VIEWPORT_WIDTH: f32 = 450.0;

pub const BANK_VIEWPORT_HEIGHT: f32 = 445.0;

pub const BANK_LABEL_LINE_HEIGHT: f32 = USER_EQUIP_REGULAR_FONT_LINE_HEIGHT;

pub const BANK_BUTTON_LINE_HEIGHT: f32 = 11.300_000_19;

pub const BANK_EQUIP_LINE_HEIGHT: f32 = USER_EQUIP_SMALL_FONT_LINE_HEIGHT;

pub const BANK_INFO_RECT: BankUiRect = BankUiRect::new(3.0, 9.0, 481.0, 87.0);

pub const BANK_TITLE_RECT: BankUiRect = BankUiRect::new(14.0, 10.0, 400.0, 20.0);

pub const BANK_DIALOG_RECT: BankUiRect = BankUiRect::new(11.0, 105.0, 482.0, 531.0);

pub const BANK_TAB_RECT: BankUiRect = BankUiRect::new(15.0, 0.0, 130.0, 29.0);

pub const BANK_VIEWPORT_RECT: BankUiRect = BankUiRect::new(29.0, 34.0, 450.0, 445.0);

pub const BANK_SHADOW_RECT: BankUiRect = BankUiRect::new(6.0, 32.0, 447.0, 449.0);

pub const BANK_SCROLL_UP_RECT: BankUiRect = BankUiRect::new(462.0, 34.0, 17.0, 12.0);

pub const BANK_SCROLL_DOWN_RECT: BankUiRect = BankUiRect::new(462.0, 467.0, 17.0, 12.0);

pub const BANK_SCROLL_TRACK_RECT: BankUiRect = BankUiRect::new(462.0, 46.0, 17.0, 421.0);

pub const BANK_PC_STUFF_DEXLABS_RECT: BankUiRect = BankUiRect::new(170.0, 561.0, 203.0, 67.0);

pub const BANK_PC_STUFF_TAROS_COUNTER_RECT: BankUiRect = BankUiRect::new(20.0, 560.0, 149.0, 32.0);

pub const BANK_PC_STUFF_REDEEM_CODE_RECT: BankUiRect = BankUiRect::new(15.0, 598.0, 149.0, 25.0);

pub const BANK_PC_STUFF_TAROS_DIGIT_RECTS: [BankUiRect; 9] = [
    BankUiRect::new(22.0, 564.0, 12.0, 20.0),
    BankUiRect::new(34.0, 564.0, 12.0, 20.0),
    BankUiRect::new(46.0, 564.0, 12.0, 20.0),
    BankUiRect::new(58.0, 564.0, 12.0, 20.0),
    BankUiRect::new(70.0, 564.0, 12.0, 20.0),
    BankUiRect::new(82.0, 564.0, 12.0, 20.0),
    BankUiRect::new(94.0, 564.0, 12.0, 20.0),
    BankUiRect::new(106.0, 564.0, 12.0, 20.0),
    BankUiRect::new(118.0, 564.0, 12.0, 20.0),
];

pub const BANK_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(136.0, 33.0),
    max_inset: Vec2::new(10.0, 51.0),
};

pub const BANK_SHADOW_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(0.0, 32.0),
    max_inset: Vec2::new(0.0, 32.0),
};

pub const BANK_SLOT_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BankUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl BankUiRect {
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

impl From<UserEquipUiRect> for BankUiRect {
    fn from(value: UserEquipUiRect) -> Self {
        Self::new(value.left, value.top, value.width, value.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BankModeLayout {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub opening_eased_fraction: f32,
    pub bank_scroll_y: f32,
    pub inventory_scroll_y: f32,
    pub full_backdrop: BankUiRect,
    pub bank_backplate: BankUiRect,
    pub right_backplate: BankUiRect,
    pub bank_panel: BankUiRect,
    pub bank_info: BankUiRect,
    pub bank_title: BankUiRect,
    pub bank_dialog: BankUiRect,
    pub bank_tab: BankUiRect,
    pub bank_viewport: BankUiRect,
    pub bank_content: BankUiRect,
    pub bank_shadow: BankUiRect,
    pub bank_scroll_track: BankUiRect,
    pub bank_scroll_up: BankUiRect,
    pub bank_scroll_down: BankUiRect,
    pub bank_scroll_thumb: BankUiRect,
    pub item_mode: UserEquipItemModeLayout,
}

impl BankModeLayout {
    #[must_use]
    pub fn bank_slot_rect(self, slot_index: usize) -> Option<BankUiRect> {
        if slot_index >= BANK_SLOT_COUNT_0104 {
            return None;
        }
        Some(BankUiRect::new(
            self.bank_content.left + (slot_index % BANK_GRID_COLUMNS) as f32 * BANK_SLOT_STRIDE,
            self.bank_content.top + (slot_index / BANK_GRID_COLUMNS) as f32 * BANK_SLOT_STRIDE,
            BANK_SLOT_SIZE,
            BANK_SLOT_SIZE,
        ))
    }

    #[must_use]
    pub fn scroll_thumb_in_dialog(self) -> BankUiRect {
        self.bank_scroll_thumb
            .translated(-self.bank_dialog.left, -self.bank_dialog.top)
    }
}

#[must_use]
pub fn bank_mode_layout(
    viewport_width: u32,
    viewport_height: u32,
    opening_elapsed_seconds: f32,
    bank_scroll_y: f32,
    inventory_scroll_y: f32,
) -> BankModeLayout {
    let viewport_width = viewport_width.min(i32::MAX as u32) as i32;
    let viewport_height = viewport_height.min(i32::MAX as u32) as i32;
    let center_x = if viewport_width > BANK_REFERENCE_WIDTH {
        (viewport_width - BANK_REFERENCE_WIDTH) / 2
    } else {
        0
    };
    let top = ((viewport_height - BANK_REFERENCE_HEIGHT) / 2).max(0);
    let eased = bank_opening_eased_fraction(opening_elapsed_seconds);
    let remaining = 1.0 - eased;
    let bank_x = center_x - ((center_x - BANK_PANEL_START_X) as f32 * remaining) as i32;

    let backplate_left = (viewport_width - BANK_BACKPLATE_REFERENCE_WIDTH) / 2;
    let backplate_top = (viewport_height - BANK_BACKPLATE_REFERENCE_HEIGHT) / 2;
    let full_backdrop = BankUiRect::new(
        ((viewport_width - USER_EQUIP_BACKDROP_WIDTH as i32) / 2) as f32,
        ((viewport_height - USER_EQUIP_BACKDROP_HEIGHT as i32) / 2) as f32,
        USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_BACKDROP_HEIGHT,
    );
    let bank_backplate = BankUiRect::new(backplate_left as f32, backplate_top as f32, 585.0, 653.0);
    let right_backplate = BankUiRect::new(
        (backplate_left + 585) as f32,
        backplate_top as f32,
        451.0,
        653.0,
    );
    let bank_panel = BankUiRect::new(
        bank_x as f32,
        top as f32,
        BANK_PANEL_WIDTH,
        BANK_PANEL_HEIGHT,
    );
    let bank_info = BANK_INFO_RECT.translated(bank_panel.left, bank_panel.top);
    let bank_title = BANK_TITLE_RECT.translated(bank_info.left, bank_info.top);
    let bank_dialog = BANK_DIALOG_RECT.translated(bank_panel.left, bank_panel.top);
    let bank_tab = BANK_TAB_RECT.translated(bank_dialog.left, bank_dialog.top);
    let bank_viewport = BANK_VIEWPORT_RECT.translated(bank_dialog.left, bank_dialog.top);
    let bank_scroll_y = clamp_bank_scroll(bank_scroll_y);
    let bank_content = BankUiRect::new(
        bank_viewport.left,
        bank_viewport.top - bank_scroll_y,
        BANK_CONTENT_WIDTH,
        BANK_CONTENT_HEIGHT,
    );
    let bank_shadow = BANK_SHADOW_RECT.translated(bank_dialog.left, bank_dialog.top);
    let bank_scroll_track = BANK_SCROLL_TRACK_RECT.translated(bank_dialog.left, bank_dialog.top);
    let bank_scroll_up = BANK_SCROLL_UP_RECT.translated(bank_dialog.left, bank_dialog.top);
    let bank_scroll_down = BANK_SCROLL_DOWN_RECT.translated(bank_dialog.left, bank_dialog.top);
    let extent = crate::service_scroll::thumb_extent(
        BANK_SCROLL_TRACK_RECT.height,
        BANK_VIEWPORT_HEIGHT,
        bank_scroll_max(),
    );
    let track_travel = (BANK_SCROLL_TRACK_RECT.height - extent).max(0.0);
    let thumb_fraction = if bank_scroll_max() > 0.0 {
        bank_scroll_y / bank_scroll_max()
    } else {
        0.0
    };
    let bank_scroll_thumb = BankUiRect::new(
        bank_dialog.left + BANK_SCROLL_TRACK_RECT.left,
        bank_dialog.top + BANK_SCROLL_TRACK_RECT.top + track_travel * thumb_fraction,
        15.0,
        extent,
    );
    let item_mode = user_equip_item_mode_layout(
        viewport_width as u32,
        viewport_height as u32,
        opening_elapsed_seconds,
        inventory_scroll_y,
    );

    BankModeLayout {
        viewport_width,
        viewport_height,
        opening_eased_fraction: eased,
        bank_scroll_y,
        inventory_scroll_y: item_mode.scroll_y,
        full_backdrop,
        bank_backplate,
        right_backplate,
        bank_panel,
        bank_info,
        bank_title,
        bank_dialog,
        bank_tab,
        bank_viewport,
        bank_content,
        bank_shadow,
        bank_scroll_track,
        bank_scroll_up,
        bank_scroll_down,
        bank_scroll_thumb,
        item_mode,
    }
}

pub(super) fn bind_rect(node: &mut Node, rect: BankUiRect) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
