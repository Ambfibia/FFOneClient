use super::*;

pub const USER_STORE_REFERENCE_WIDTH: i32 = 1_020;

pub const USER_STORE_REFERENCE_HEIGHT: i32 = 638;

pub const USER_STORE_SHELL_WIDTH: i32 = 1_036;

pub const USER_STORE_SHELL_HEIGHT: i32 = 653;

pub const USER_STORE_PANEL_WIDTH: f32 = 498.0;

pub const USER_STORE_PANEL_HEIGHT: f32 = 638.0;

pub const USER_STORE_ROW_HEIGHT: f32 = 80.0;

pub const USER_STORE_ROW_WIDTH: f32 = 433.0;

pub const USER_STORE_ROW_VISUAL_HEIGHT: f32 = 75.0;

pub const USER_STORE_INVENTORY_CONTENT_HEIGHT: f32 = 690.0;

pub const USER_STORE_INVENTORY_VIEWPORT_HEIGHT: f32 = 500.0;

pub const USER_STORE_JEFFE_12_LINE_HEIGHT: f32 = 13.560_000_42;

pub const USER_STORE_JEFFE_14_LINE_HEIGHT: f32 = 11.300_000_19;

pub const USER_STORE_JEFFE_16_LINE_HEIGHT: f32 = 13.560_000_42;

pub const USER_STORE_CHALET_SMALL_LINE_HEIGHT: f32 = 13.560_000_42;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserStoreUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl UserStoreUiRect {
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

pub const USER_STORE_DIALOG_RECT: UserStoreUiRect = UserStoreUiRect::new(11.0, 132.0, 486.0, 528.0);

pub const USER_STORE_LIST_BACK_RECT: UserStoreUiRect =
    UserStoreUiRect::new(0.0, 20.0, 486.0, 485.0);

pub const USER_STORE_TABLE_RECT: UserStoreUiRect = UserStoreUiRect::new(2.0, 32.0, 479.0, 410.0);

pub const USER_STORE_LIST_VIEWPORT_RECT: UserStoreUiRect =
    UserStoreUiRect::new(10.0, 10.0, 464.0, 400.0);

pub const USER_STORE_TABLE_SHADOW_RECT: UserStoreUiRect =
    UserStoreUiRect::new(2.0, 3.0, 450.0, 412.0);

pub const USER_STORE_INFO_RECT: UserStoreUiRect = UserStoreUiRect::new(3.0, 9.0, 481.0, 87.0);

pub const USER_STORE_TITLE_RECT: UserStoreUiRect = UserStoreUiRect::new(14.0, 10.0, 150.0, 20.0);

pub const USER_STORE_ITEM_TAB_RECT: UserStoreUiRect = UserStoreUiRect::new(1.0, 0.0, 95.0, 27.0);

pub const USER_STORE_ITEM_TAB_HIT_RECT: UserStoreUiRect =
    UserStoreUiRect::new(10.0, 5.0, 80.0, 15.0);

pub const USER_STORE_LIST_DIVIDER_RECT: UserStoreUiRect =
    UserStoreUiRect::new(0.0, 20.0, 486.0, 13.0);

pub const USER_STORE_PRIMARY_BUTTON_RECT: UserStoreUiRect =
    UserStoreUiRect::new(313.0, 595.0, 161.0, 25.0);

pub const USER_STORE_GO_TO_GAME_RECT: UserStoreUiRect =
    UserStoreUiRect::new(30.0, 595.0, 161.0, 25.0);

pub const USER_STORE_ROW_ITEM_RECT: UserStoreUiRect = UserStoreUiRect::new(4.0, 5.0, 66.0, 66.0);

pub const USER_STORE_ROW_NAME_RECT: UserStoreUiRect = UserStoreUiRect::new(82.0, 8.0, 300.0, 20.0);

pub const USER_STORE_ROW_LEVEL_RECT: UserStoreUiRect =
    UserStoreUiRect::new(82.0, 22.0, 300.0, 20.0);

pub const USER_STORE_ROW_PRICE_RECT: UserStoreUiRect =
    UserStoreUiRect::new(262.0, 20.0, 134.0, 20.0);

pub const USER_STORE_ROW_SELLER_NET_RECT: UserStoreUiRect =
    UserStoreUiRect::new(262.0, 30.0, 134.0, 20.0);

pub const USER_STORE_PC_STUFF_RECT: UserStoreUiRect = UserStoreUiRect::new(0.0, 0.0, 380.0, 632.0);

pub const USER_STORE_INVENTORY_VIEWPORT_RECT: UserStoreUiRect =
    UserStoreUiRect::new(6.0, 36.0, 366.0, 500.0);

pub const USER_STORE_CLOSE_RECT: UserStoreUiRect = UserStoreUiRect::new(400.0, 5.0, 30.0, 30.0);

pub const USER_STORE_TRASH_RECT: UserStoreUiRect = UserStoreUiRect::new(390.0, 555.0, 32.0, 32.0);

pub const USER_STORE_HELP_RECT: UserStoreUiRect = UserStoreUiRect::new(390.0, 590.0, 32.0, 32.0);

pub const USER_STORE_EQUIP_PANEL_RECT: UserStoreUiRect =
    UserStoreUiRect::new(0.0, 0.0, 66.0, 639.0);

pub const USER_STORE_EQUIP_CONTENT_RECT: UserStoreUiRect =
    UserStoreUiRect::new(0.0, 14.0, 66.0, 625.0);

pub const USER_STORE_POPUP_RECT: UserStoreUiRect = UserStoreUiRect::new(0.0, 0.0, 310.0, 355.0);

pub const USER_STORE_POPUP_IN_POSITION: Vec2 = Vec2::new(610.0, 90.0);

pub const USER_STORE_POPUP_OUT_POSITION: Vec2 = Vec2::new(200.0, 90.0);

pub const USER_STORE_POPUP_ICON_RECT: UserStoreUiRect =
    UserStoreUiRect::new(16.0, 16.0, 64.0, 64.0);

pub const USER_STORE_POPUP_NAME_RECT: UserStoreUiRect =
    UserStoreUiRect::new(82.0, 16.0, 170.0, 40.0);

pub const USER_STORE_POPUP_COST_RECT: UserStoreUiRect =
    UserStoreUiRect::new(82.0, 50.0, 170.0, 20.0);

pub const USER_STORE_POPUP_DESCRIPTION_RECT: UserStoreUiRect =
    UserStoreUiRect::new(12.0, 95.0, 280.0, 40.0);

pub const USER_STORE_POPUP_CLOSE_RECT: UserStoreUiRect =
    UserStoreUiRect::new(277.0, 0.0, 32.0, 32.0);

pub const USER_STORE_POPUP_VALUE_TYPE_RECT: UserStoreUiRect =
    UserStoreUiRect::new(87.0, 140.0, 137.0, 20.0);

pub const USER_STORE_POPUP_CALCULATOR_RECT: UserStoreUiRect =
    UserStoreUiRect::new(87.0, 162.0, 136.0, 109.0);

pub const USER_STORE_POPUP_VALUE_RECT: UserStoreUiRect =
    UserStoreUiRect::new(1.0, 1.0, 132.0, 20.0);

pub const USER_STORE_POPUP_REGISTER_ACTION_RECT: UserStoreUiRect =
    UserStoreUiRect::new(79.0, 301.0, 150.0, 25.0);

pub const USER_STORE_POPUP_LISTING_ACTION_RECT: UserStoreUiRect =
    UserStoreUiRect::new(59.0, 301.0, 190.0, 25.0);

pub const USER_STORE_GENERIC_CALCULATOR_RECT: UserStoreUiRect =
    UserStoreUiRect::new(-100.0, -200.0, 115.0, 200.0);

pub const USER_STORE_GENERIC_CALCULATOR_VALUE_RECT: UserStoreUiRect =
    UserStoreUiRect::new(10.0, 20.0, 150.0, 30.0);

pub const USER_STORE_POPUP_BUTTON_RECTS: [(u8, UserStoreUiRect); 10] = [
    (1, UserStoreUiRect::new(1.0, 29.0, 44.0, 19.0)),
    (2, UserStoreUiRect::new(46.0, 29.0, 44.0, 19.0)),
    (3, UserStoreUiRect::new(91.0, 29.0, 44.0, 19.0)),
    (4, UserStoreUiRect::new(1.0, 49.0, 44.0, 19.0)),
    (5, UserStoreUiRect::new(46.0, 49.0, 44.0, 19.0)),
    (6, UserStoreUiRect::new(91.0, 49.0, 44.0, 19.0)),
    (7, UserStoreUiRect::new(1.0, 69.0, 44.0, 19.0)),
    (8, UserStoreUiRect::new(46.0, 69.0, 44.0, 19.0)),
    (9, UserStoreUiRect::new(91.0, 69.0, 44.0, 19.0)),
    (0, UserStoreUiRect::new(46.0, 89.0, 44.0, 19.0)),
];

pub const USER_STORE_POPUP_CLEAR_RECT: UserStoreUiRect =
    UserStoreUiRect::new(1.0, 89.0, 44.0, 19.0);

pub const USER_STORE_POPUP_NO_OP_RECT: UserStoreUiRect =
    UserStoreUiRect::new(91.0, 89.0, 44.0, 19.0);

pub const USER_STORE_LIST_BACK_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 14.0),
    max_inset: Vec2::new(6.0, 55.0),
};

pub const USER_STORE_RIGHT_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(5.0, 0.0),
    max_inset: Vec2::new(5.0, 0.0),
};

pub const USER_STORE_INVENTORY_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(136.0, 33.0),
    max_inset: Vec2::new(10.0, 77.0),
};

pub const USER_STORE_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 8.0),
    max_inset: Vec2::new(8.0, 8.0),
};

pub const USER_STORE_POPUP_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

#[must_use]
pub fn user_store_popup_rect_0104(
    viewport_width: u32,
    viewport_height: u32,
    popup: UserStoreItemPopup0104,
) -> UserStoreUiRect {
    let width = viewport_width.min(i32::MAX as u32) as i32;
    let height = viewport_height.min(i32::MAX as u32) as i32;
    let offset_x = ((width - USER_STORE_REFERENCE_WIDTH) / 2).max(0) as f32;
    let offset_y = ((height - USER_STORE_REFERENCE_HEIGHT) / 2).max(0) as f32;
    let position = if popup.slot_type == 1 {
        USER_STORE_POPUP_IN_POSITION
    } else {
        USER_STORE_POPUP_OUT_POSITION
    };
    USER_STORE_POPUP_RECT.translated(position.x + offset_x, position.y + offset_y)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserStoreUiLayout0104 {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub opening_eased_fraction: f32,
    pub full_backdrop: UserStoreUiRect,
    pub store_backplate: UserStoreUiRect,
    pub right_backplate: UserStoreUiRect,
    pub store_panel: UserStoreUiRect,
    pub pc_stuff_panel: UserStoreUiRect,
    pub equipment_panel: UserStoreUiRect,
    pub list_viewport: UserStoreUiRect,
    pub inventory_viewport: UserStoreUiRect,
    pub inventory_content: UserStoreUiRect,
}

impl UserStoreUiLayout0104 {
    #[must_use]
    pub fn row_rect(self, visual_row: usize) -> UserStoreUiRect {
        UserStoreUiRect::new(
            self.list_viewport.left,
            self.list_viewport.top + visual_row as f32 * USER_STORE_ROW_HEIGHT,
            USER_STORE_ROW_WIDTH,
            USER_STORE_ROW_VISUAL_HEIGHT,
        )
    }

    #[must_use]
    pub fn inventory_slot_rect(self, slot: usize) -> Option<UserStoreUiRect> {
        if slot >= USER_STORE_INVENTORY_CAPACITY {
            return None;
        }
        let column = slot % USER_STORE_INVENTORY_COLUMNS;
        let row = slot / USER_STORE_INVENTORY_COLUMNS;
        Some(UserStoreUiRect::new(
            self.inventory_content.left + column as f32 * USER_STORE_INVENTORY_SLOT_STRIDE,
            self.inventory_content.top + row as f32 * USER_STORE_INVENTORY_SLOT_STRIDE,
            USER_STORE_INVENTORY_SLOT_SIZE,
            USER_STORE_INVENTORY_SLOT_SIZE,
        ))
    }
}

#[must_use]
pub fn user_store_layout_0104(
    viewport_width: u32,
    viewport_height: u32,
    opening_elapsed_seconds: f32,
    inventory_scroll_y: f32,
) -> UserStoreUiLayout0104 {
    let width = viewport_width.min(i32::MAX as u32) as i32;
    let height = viewport_height.min(i32::MAX as u32) as i32;
    let center_x = if width > USER_STORE_REFERENCE_WIDTH {
        (width - USER_STORE_REFERENCE_WIDTH) / 2
    } else {
        0
    };
    let top = ((height - USER_STORE_REFERENCE_HEIGHT) / 2).max(0);
    let eased = user_store_opening_eased_fraction(opening_elapsed_seconds);
    let remaining = 1.0 - eased;
    // Clean `Panel_UserStore.ScrollWindow`: final x is the 1020-wide center,
    // initial x is -498, and the truncating cast is part of the trajectory.
    let store_x = center_x - ((center_x + 498) as f32 * remaining) as i32;
    let pc_final = center_x + 585;
    let equipment_final = center_x + 504;
    let pc_x = pc_final + ((USER_STORE_REFERENCE_WIDTH - pc_final) as f32 * remaining) as i32;
    let equipment_x = equipment_final
        + ((USER_STORE_REFERENCE_WIDTH - equipment_final) as f32 * remaining) as i32;
    let shell_left = (width - USER_STORE_SHELL_WIDTH) / 2;
    let shell_top = (height - USER_STORE_SHELL_HEIGHT) / 2;
    let store_panel = UserStoreUiRect::new(
        store_x as f32,
        top as f32,
        USER_STORE_PANEL_WIDTH,
        USER_STORE_PANEL_HEIGHT,
    );
    let pc_stuff_panel = USER_STORE_PC_STUFF_RECT.translated(pc_x as f32, top as f32);
    let equipment_panel = USER_STORE_EQUIP_PANEL_RECT.translated(equipment_x as f32, top as f32);
    let list_viewport = USER_STORE_LIST_VIEWPORT_RECT.translated(
        store_panel.left + USER_STORE_DIALOG_RECT.left + USER_STORE_TABLE_RECT.left,
        store_panel.top + USER_STORE_DIALOG_RECT.top + USER_STORE_TABLE_RECT.top,
    );
    let inventory_viewport =
        USER_STORE_INVENTORY_VIEWPORT_RECT.translated(pc_stuff_panel.left, pc_stuff_panel.top);
    let scroll_y = clamp_user_store_inventory_scroll(inventory_scroll_y);
    let inventory_content = UserStoreUiRect::new(
        inventory_viewport.left,
        inventory_viewport.top - scroll_y,
        366.0,
        USER_STORE_INVENTORY_CONTENT_HEIGHT,
    );
    UserStoreUiLayout0104 {
        viewport_width,
        viewport_height,
        opening_eased_fraction: eased,
        full_backdrop: UserStoreUiRect::new(
            ((width - 1_920) / 2) as f32,
            ((height - 1_440) / 2) as f32,
            1_920.0,
            1_440.0,
        ),
        store_backplate: UserStoreUiRect::new(
            shell_left as f32,
            shell_top as f32,
            585.0,
            USER_STORE_SHELL_HEIGHT as f32,
        ),
        right_backplate: UserStoreUiRect::new(
            (shell_left + 585) as f32,
            shell_top as f32,
            451.0,
            USER_STORE_SHELL_HEIGHT as f32,
        ),
        store_panel,
        pc_stuff_panel,
        equipment_panel,
        list_viewport,
        inventory_viewport,
        inventory_content,
    }
}

pub(super) fn bind_user_store_rect(node: &mut Node, rect: UserStoreUiRect) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
