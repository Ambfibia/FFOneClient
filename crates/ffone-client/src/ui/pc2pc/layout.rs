use super::*;

pub const PC2PC_REFERENCE_WIDTH: i32 = 1_020;

pub const PC2PC_REFERENCE_HEIGHT: i32 = 638;

pub const PC2PC_BACKPLATE_REFERENCE_WIDTH: i32 = 1_036;

pub const PC2PC_BACKPLATE_REFERENCE_HEIGHT: i32 = 653;

pub const PC2PC_PANEL_WIDTH: f32 = 550.0;

pub const PC2PC_PANEL_HEIGHT: f32 = 700.0;

pub const PC2PC_CHAT_LINE_HEIGHT: f32 = 15.0;

pub const PC2PC_JEFFE_12_LINE_HEIGHT: f32 = 13.560_000_42;

pub const PC2PC_JEFFE_14_LINE_HEIGHT: f32 = 11.300_000_19;

pub const PC2PC_JEFFE_16_LINE_HEIGHT: f32 = 13.560_000_42;

pub const PC2PC_CHALET_SMALL_LINE_HEIGHT: f32 = 13.560_000_42;

pub const PC2PC_TRADE_AREA_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(4.0, 4.0),
    max_inset: Vec2::new(4.0, 4.0),
};

pub const PC2PC_MONEY_BACK_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(5.0, 5.0),
    max_inset: Vec2::new(5.0, 5.0),
};

pub const PC2PC_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pc2pcUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Pc2pcUiRect {
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
    pub const fn translated(self, left: f32, top: f32) -> Self {
        Self::new(self.left + left, self.top + top, self.width, self.height)
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

impl From<UserEquipUiRect> for Pc2pcUiRect {
    fn from(value: UserEquipUiRect) -> Self {
        Self::new(value.left, value.top, value.width, value.height)
    }
}

pub const PC2PC_TRADE_AREA_RECT: Pc2pcUiRect = Pc2pcUiRect::new(12.0, 29.0, 480.0, 448.0);

pub const PC2PC_LOCAL_OFFER_RECT: Pc2pcUiRect = Pc2pcUiRect::new(25.0, 35.0, 480.0, 157.0);

pub const PC2PC_REMOTE_OFFER_RECT: Pc2pcUiRect = Pc2pcUiRect::new(0.0, 200.0, 480.0, 172.0);

pub const PC2PC_LOCAL_MONEY_RECT: Pc2pcUiRect = Pc2pcUiRect::new(49.0, 103.0, 191.0, 32.0);

pub const PC2PC_REMOTE_MONEY_RECT: Pc2pcUiRect = Pc2pcUiRect::new(115.0, 105.0, 327.0, 32.0);

pub const PC2PC_ADD_TAROS_RECT: Pc2pcUiRect = Pc2pcUiRect::new(247.0, 107.0, 132.0, 25.0);

pub const PC2PC_LOCAL_PORTRAIT_RECT: Pc2pcUiRect = Pc2pcUiRect::new(400.0, 20.0, 70.0, 70.0);

pub const PC2PC_REMOTE_PORTRAIT_RECT: Pc2pcUiRect = Pc2pcUiRect::new(35.0, 12.0, 70.0, 70.0);

pub const PC2PC_LOCAL_TITLE_RECT: Pc2pcUiRect = Pc2pcUiRect::new(50.0, 20.0, 320.0, 20.0);

pub const PC2PC_REMOTE_TITLE_RECT: Pc2pcUiRect = Pc2pcUiRect::new(110.0, 15.0, 330.0, 20.0);

pub const PC2PC_MAIN_BUTTON_RECT: Pc2pcUiRect = Pc2pcUiRect::new(340.0, 432.0, 133.0, 30.0);

pub const PC2PC_READY_NAME_RECT: Pc2pcUiRect = Pc2pcUiRect::new(32.0, 380.0, 440.0, 20.0);

pub const PC2PC_READY_SUBJECT_RECT: Pc2pcUiRect = Pc2pcUiRect::new(32.0, 395.0, 440.0, 20.0);

pub const PC2PC_CHAT_BOX_RECT: Pc2pcUiRect = Pc2pcUiRect::new(10.0, 490.0, 483.0, 141.0);

pub const PC2PC_CHAT_LIST_VIEW_RECT: Pc2pcUiRect = Pc2pcUiRect::new(12.0, 49.0, 458.0, 49.0);

pub const PC2PC_CHAT_INPUT_RECT: Pc2pcUiRect = Pc2pcUiRect::new(18.0, 104.0, 400.0, 25.0);

pub const PC2PC_CHAT_SEND_RECT: Pc2pcUiRect = Pc2pcUiRect::new(415.0, 105.0, 57.0, 24.0);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pc2pcOfferDirection0104 {
    Outgoing,
    Incoming,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pc2pcModeLayout {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub opening_eased_fraction: f32,
    pub full_backdrop: Pc2pcUiRect,
    pub trade_backplate: Pc2pcUiRect,
    pub right_backplate: Pc2pcUiRect,
    pub trade_panel: Pc2pcUiRect,
    pub trade_area: Pc2pcUiRect,
    pub local_offer: Pc2pcUiRect,
    pub remote_offer: Pc2pcUiRect,
    pub local_money: Pc2pcUiRect,
    pub remote_money: Pc2pcUiRect,
    pub add_taros: Pc2pcUiRect,
    pub local_portrait: Pc2pcUiRect,
    pub remote_portrait: Pc2pcUiRect,
    pub local_title: Pc2pcUiRect,
    pub remote_title: Pc2pcUiRect,
    pub main_button: Pc2pcUiRect,
    pub ready_name: Pc2pcUiRect,
    pub ready_subject: Pc2pcUiRect,
    pub chat_box: Pc2pcUiRect,
    pub chat_list: Pc2pcUiRect,
    pub chat_input: Pc2pcUiRect,
    pub chat_send: Pc2pcUiRect,
    pub item_mode: UserEquipItemModeLayout,
}

impl Pc2pcModeLayout {
    #[must_use]
    pub fn local_offer_slot(self, slot: usize) -> Option<Pc2pcUiRect> {
        (slot < PC2PC_OFFER_SLOT_COUNT).then(|| {
            Pc2pcUiRect::new(
                self.local_offer.left
                    + PC2PC_LOCAL_SLOT_ORIGIN.x
                    + slot as f32 * PC2PC_OFFER_SLOT_STRIDE,
                self.local_offer.top + PC2PC_LOCAL_SLOT_ORIGIN.y,
                PC2PC_OFFER_SLOT_SIZE,
                PC2PC_OFFER_SLOT_SIZE,
            )
        })
    }

    #[must_use]
    pub fn remote_offer_slot(self, slot: usize) -> Option<Pc2pcUiRect> {
        (slot < PC2PC_OFFER_SLOT_COUNT).then(|| {
            Pc2pcUiRect::new(
                self.remote_offer.left
                    + PC2PC_REMOTE_SLOT_ORIGIN.x
                    + slot as f32 * PC2PC_OFFER_SLOT_STRIDE,
                self.remote_offer.top + PC2PC_REMOTE_SLOT_ORIGIN.y,
                PC2PC_OFFER_SLOT_SIZE,
                PC2PC_OFFER_SLOT_SIZE,
            )
        })
    }
}

#[must_use]
pub fn pc2pc_mode_layout(
    viewport_width: u32,
    viewport_height: u32,
    opening_elapsed: f32,
    inventory_scroll_y: f32,
) -> Pc2pcModeLayout {
    let viewport_width = viewport_width as i32;
    let viewport_height = viewport_height as i32;
    let fraction = (opening_elapsed / PC2PC_OPEN_SECONDS).clamp(0.0, 1.0);
    let eased = (std::f32::consts::FRAC_PI_2 * fraction).sin();
    let final_left = if viewport_width > PC2PC_REFERENCE_WIDTH {
        (viewport_width - PC2PC_REFERENCE_WIDTH) / 2
    } else {
        0
    };
    let top = ((viewport_height - PC2PC_REFERENCE_HEIGHT) / 2).max(0);
    let panel_left = PC2PC_PANEL_START_X as f32 + (final_left - PC2PC_PANEL_START_X) as f32 * eased;
    let backplate_left = (viewport_width - PC2PC_BACKPLATE_REFERENCE_WIDTH) / 2;
    let backplate_top = (viewport_height - PC2PC_BACKPLATE_REFERENCE_HEIGHT) / 2;
    let panel = Pc2pcUiRect::new(
        panel_left,
        top as f32,
        PC2PC_PANEL_WIDTH,
        PC2PC_PANEL_HEIGHT,
    );
    let item_mode = user_equip_item_mode_layout(
        viewport_width as u32,
        viewport_height as u32,
        opening_elapsed,
        inventory_scroll_y,
    );

    Pc2pcModeLayout {
        viewport_width,
        viewport_height,
        opening_eased_fraction: eased,
        full_backdrop: Pc2pcUiRect::new(
            (viewport_width as f32 - USER_EQUIP_BACKDROP_WIDTH) * 0.5,
            (viewport_height as f32 - USER_EQUIP_BACKDROP_HEIGHT) * 0.5,
            USER_EQUIP_BACKDROP_WIDTH,
            USER_EQUIP_BACKDROP_HEIGHT,
        ),
        trade_backplate: Pc2pcUiRect::new(
            backplate_left as f32,
            backplate_top as f32,
            585.0,
            653.0,
        ),
        right_backplate: Pc2pcUiRect::new(
            (backplate_left + 585) as f32,
            backplate_top as f32,
            451.0,
            653.0,
        ),
        trade_panel: panel,
        trade_area: PC2PC_TRADE_AREA_RECT.translated(panel.left, panel.top),
        local_offer: PC2PC_LOCAL_OFFER_RECT.translated(panel.left, panel.top),
        remote_offer: PC2PC_REMOTE_OFFER_RECT.translated(panel.left, panel.top),
        local_money: PC2PC_LOCAL_MONEY_RECT.translated(
            panel.left + PC2PC_LOCAL_OFFER_RECT.left,
            panel.top + PC2PC_LOCAL_OFFER_RECT.top,
        ),
        remote_money: PC2PC_REMOTE_MONEY_RECT.translated(
            panel.left + PC2PC_REMOTE_OFFER_RECT.left,
            panel.top + PC2PC_REMOTE_OFFER_RECT.top,
        ),
        add_taros: PC2PC_ADD_TAROS_RECT.translated(
            panel.left + PC2PC_LOCAL_OFFER_RECT.left,
            panel.top + PC2PC_LOCAL_OFFER_RECT.top,
        ),
        local_portrait: PC2PC_LOCAL_PORTRAIT_RECT.translated(
            panel.left + PC2PC_LOCAL_OFFER_RECT.left,
            panel.top + PC2PC_LOCAL_OFFER_RECT.top,
        ),
        remote_portrait: PC2PC_REMOTE_PORTRAIT_RECT.translated(
            panel.left + PC2PC_REMOTE_OFFER_RECT.left,
            panel.top + PC2PC_REMOTE_OFFER_RECT.top,
        ),
        local_title: PC2PC_LOCAL_TITLE_RECT.translated(
            panel.left + PC2PC_LOCAL_OFFER_RECT.left,
            panel.top + PC2PC_LOCAL_OFFER_RECT.top,
        ),
        remote_title: PC2PC_REMOTE_TITLE_RECT.translated(
            panel.left + PC2PC_REMOTE_OFFER_RECT.left,
            panel.top + PC2PC_REMOTE_OFFER_RECT.top,
        ),
        main_button: PC2PC_MAIN_BUTTON_RECT.translated(panel.left, panel.top),
        ready_name: PC2PC_READY_NAME_RECT.translated(panel.left, panel.top),
        ready_subject: PC2PC_READY_SUBJECT_RECT.translated(panel.left, panel.top),
        chat_box: PC2PC_CHAT_BOX_RECT.translated(panel.left, panel.top),
        chat_list: PC2PC_CHAT_LIST_VIEW_RECT.translated(
            panel.left + PC2PC_CHAT_BOX_RECT.left,
            panel.top + PC2PC_CHAT_BOX_RECT.top,
        ),
        chat_input: PC2PC_CHAT_INPUT_RECT.translated(
            panel.left + PC2PC_CHAT_BOX_RECT.left,
            panel.top + PC2PC_CHAT_BOX_RECT.top,
        ),
        chat_send: PC2PC_CHAT_SEND_RECT.translated(
            panel.left + PC2PC_CHAT_BOX_RECT.left,
            panel.top + PC2PC_CHAT_BOX_RECT.top,
        ),
        item_mode,
    }
}

pub(super) fn bind_rect(node: &mut Node, rect: Pc2pcUiRect) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
