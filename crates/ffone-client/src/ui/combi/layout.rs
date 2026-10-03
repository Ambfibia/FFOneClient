use super::*;

pub const COMBI_REFERENCE_WIDTH: i32 = 1_036;

pub const COMBI_REFERENCE_HEIGHT: i32 = 653;

pub const COMBI_LEFT_GROUP_WIDTH: f32 = 585.0;

pub const COMBI_LEFT_GROUP_HEIGHT: f32 = 653.0;

pub const COMBI_PANEL_NATIVE_WIDTH: f32 = 584.0;

pub const COMBI_PANEL_NATIVE_HEIGHT: f32 = 650.0;

pub const COMBI_RIGHT_BACKPLATE_WIDTH: f32 = 451.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CombiUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl CombiUiRect {
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

impl From<UserEquipUiRect> for CombiUiRect {
    fn from(value: UserEquipUiRect) -> Self {
        Self::new(value.left, value.top, value.width, value.height)
    }
}

pub const COMBI_PRIMARY_NPC_PREVIEW_RECT: CombiUiRect = CombiUiRect::new(363.0, 2.0, 143.0, 128.0);

pub const COMBI_TITLE_RECT: CombiUiRect = CombiUiRect::new(28.0, 19.0, 299.0, 16.0);

pub const COMBI_INTRO_RECT: CombiUiRect = CombiUiRect::new(28.0, 36.0, 327.0, 49.0);

pub const COMBI_STYLE_TITLE_RECT: CombiUiRect = CombiUiRect::new(52.0, 154.0, 55.0, 11.0);

pub const COMBI_STATS_TITLE_RECT: CombiUiRect = CombiUiRect::new(52.0, 340.0, 55.0, 11.0);

pub const COMBI_COST_RECT: CombiUiRect = CombiUiRect::new(48.0, 518.0, 55.0, 11.0);

pub const COMBI_TAROS_LABEL_RECT: CombiUiRect = CombiUiRect::new(48.0, 537.0, 55.0, 11.0);

pub const COMBI_TAROS_ICON_RECT: CombiUiRect = CombiUiRect::new(106.0, 532.0, 23.0, 24.0);

pub const COMBI_CHANCE_TITLE_RECT: CombiUiRect = CombiUiRect::new(81.0, 581.0, 69.0, 30.0);

pub const COMBI_CHANCE_RECT: CombiUiRect = CombiUiRect::new(75.0, 603.0, 80.0, 28.0);

pub const COMBI_CHANCE_LEVEL_RECT: CombiUiRect = CombiUiRect::new(39.0, 595.0, 25.0, 25.0);

pub const COMBI_NEW_ITEM_TITLE_RECT: CombiUiRect = CombiUiRect::new(259.0, 151.0, 120.0, 10.0);

pub const COMBI_LOOK_DROP_RECT: CombiUiRect = CombiUiRect::new(168.0, 167.0, 305.0, 133.0);

pub const COMBI_LOOK_ERROR_RECT: CombiUiRect = CombiUiRect::new(168.0, 169.0, 303.0, 131.0);

pub const COMBI_LOOK_SELECTED_SLOT_RECT: CombiUiRect = CombiUiRect::new(45.0, 176.0, 64.0, 64.0);

pub const COMBI_LOOK_ICON_RECT: CombiUiRect = CombiUiRect::new(191.0, 180.0, 60.0, 60.0);

pub const COMBI_LOOK_BADGE_RECT: CombiUiRect = CombiUiRect::new(225.0, 215.0, 25.0, 25.0);

pub const COMBI_LOOK_NAME_RECT: CombiUiRect = CombiUiRect::new(258.0, 179.0, 201.0, 12.0);

pub const COMBI_LOOK_LEVEL_RECT: CombiUiRect = CombiUiRect::new(258.0, 214.0, 81.0, 14.0);

pub const COMBI_LOOK_DESCRIPTION_RECT: CombiUiRect = CombiUiRect::new(180.0, 250.0, 285.0, 36.0);

pub const COMBI_LOOK_EMPTY_TEXT_RECT: CombiUiRect = CombiUiRect::new(233.0, 203.0, 176.0, 50.0);

pub const COMBI_STAT_DROP_RECT: CombiUiRect = CombiUiRect::new(168.0, 321.0, 305.0, 230.0);

pub const COMBI_STAT_ERROR_RECT: CombiUiRect = CombiUiRect::new(168.0, 323.0, 304.0, 228.0);

pub const COMBI_STAT_SELECTED_SLOT_RECT: CombiUiRect = CombiUiRect::new(44.0, 363.0, 64.0, 64.0);

pub const COMBI_STAT_LEVEL_RECT: CombiUiRect = CombiUiRect::new(280.0, 328.0, 81.0, 14.0);

pub const COMBI_STAT_SECTION_RECT: CombiUiRect = CombiUiRect::new(175.0, 352.0, 51.0, 13.0);

pub const COMBI_STAT_SINGLE_RECT: CombiUiRect = CombiUiRect::new(200.0, 430.0, 59.0, 14.0);

pub const COMBI_STAT_MULTI_RECT: CombiUiRect = CombiUiRect::new(291.0, 430.0, 59.0, 14.0);

pub const COMBI_STAT_DEFENSE_RECT: CombiUiRect = CombiUiRect::new(383.0, 430.0, 59.0, 14.0);

pub const COMBI_INFO_SECTION_RECT: CombiUiRect = CombiUiRect::new(175.0, 453.0, 42.0, 13.0);

pub const COMBI_STAT_EMPTY_TEXT_RECT: CombiUiRect = CombiUiRect::new(233.0, 396.0, 163.0, 45.0);

pub const COMBI_CLEAR_ALL_RECT: CombiUiRect = CombiUiRect::new(180.0, 579.0, 104.0, 20.0);

pub const COMBI_COMBINE_RECT: CombiUiRect = CombiUiRect::new(327.0, 576.0, 131.0, 24.0);

pub const COMBI_SUCCESS_GROUP_RECT: CombiUiRect = CombiUiRect::new(0.0, 0.0, 352.0, 550.0);

pub const COMBI_WAITING_GROUP_RECT: CombiUiRect = CombiUiRect::new(0.0, 0.0, 357.0, 384.0);

pub const COMBI_WAITING_NPC_PREVIEW_RECT: CombiUiRect = CombiUiRect::new(0.0, 0.0, 357.0, 384.0);

pub const COMBI_SUCCESS_NPC_ICON_RECT: CombiUiRect = CombiUiRect::new(10.0, 10.0, 64.0, 64.0);

pub const COMBI_SUCCESS_HOORAY_RECT: CombiUiRect = CombiUiRect::new(80.0, 7.0, 98.0, 18.0);

pub const COMBI_SUCCESS_MESSAGE_RECT: CombiUiRect = CombiUiRect::new(80.0, 28.0, 250.0, 36.0);

pub const COMBI_SUCCESS_ICON_RECT: CombiUiRect = CombiUiRect::new(38.0, 102.0, 60.0, 60.0);

pub const COMBI_SUCCESS_BADGE_RECT: CombiUiRect = CombiUiRect::new(70.0, 136.0, 25.0, 25.0);

pub const COMBI_SUCCESS_NAME_RECT: CombiUiRect = CombiUiRect::new(106.0, 101.0, 201.0, 25.0);

pub const COMBI_SUCCESS_LEVEL_RECT: CombiUiRect = CombiUiRect::new(106.0, 138.0, 81.0, 14.0);

pub const COMBI_SUCCESS_DESCRIPTION_RECT: CombiUiRect = CombiUiRect::new(32.0, 180.0, 285.0, 40.0);

pub const COMBI_SUCCESS_COMBINE_MORE_RECT: CombiUiRect = CombiUiRect::new(70.0, 478.0, 210.0, 25.0);

pub const COMBI_SUCCESS_GO_TO_STUFF_RECT: CombiUiRect = CombiUiRect::new(70.0, 513.0, 210.0, 25.0);

pub const COMBI_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CombiModeLayout0104 {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub full_backdrop: CombiUiRect,
    pub panel: CombiUiRect,
    pub right_backplate: CombiUiRect,
    pub main_group: CombiUiRect,
    pub pc_stuff_panel: CombiUiRect,
    pub equipment_panel: CombiUiRect,
    pub shade: CombiUiRect,
    pub success_group: CombiUiRect,
    pub waiting_group: CombiUiRect,
}

#[must_use]
pub fn combi_mode_layout_0104(viewport_width: u32, viewport_height: u32) -> CombiModeLayout0104 {
    let viewport_width = viewport_width.min(i32::MAX as u32) as i32;
    let viewport_height = viewport_height.min(i32::MAX as u32) as i32;
    let shell_left = (viewport_width - COMBI_REFERENCE_WIDTH) / 2;
    let shell_top = (viewport_height - COMBI_REFERENCE_HEIGHT) / 2;
    let full_backdrop = CombiUiRect::new(
        ((viewport_width - USER_EQUIP_BACKDROP_WIDTH as i32) / 2) as f32,
        ((viewport_height - USER_EQUIP_BACKDROP_HEIGHT as i32) / 2) as f32,
        USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_BACKDROP_HEIGHT,
    );
    let panel = CombiUiRect::new(
        shell_left as f32,
        shell_top as f32,
        COMBI_LEFT_GROUP_WIDTH,
        COMBI_LEFT_GROUP_HEIGHT,
    );
    let right_backplate = CombiUiRect::new(
        (shell_left as f32) + COMBI_LEFT_GROUP_WIDTH,
        shell_top as f32,
        COMBI_RIGHT_BACKPLATE_WIDTH,
        COMBI_LEFT_GROUP_HEIGHT,
    );
    let item_mode = user_equip_item_mode_layout(
        viewport_width.max(0) as u32,
        viewport_height.max(0) as u32,
        1.0,
        0.0,
    );
    let shade = CombiUiRect::new(
        0.0,
        0.0,
        viewport_width.max(0) as f32,
        viewport_height.max(0) as f32,
    );
    let success_group = CombiUiRect::new(
        ((viewport_width - COMBI_SUCCESS_GROUP_RECT.width as i32) as f32) * 0.5,
        ((viewport_height - COMBI_SUCCESS_GROUP_RECT.height as i32) as f32) * 0.5,
        COMBI_SUCCESS_GROUP_RECT.width,
        COMBI_SUCCESS_GROUP_RECT.height,
    );
    let waiting_group = CombiUiRect::new(
        ((viewport_width - COMBI_WAITING_GROUP_RECT.width as i32) as f32) * 0.5,
        ((viewport_height - COMBI_WAITING_GROUP_RECT.height as i32) as f32) * 0.5,
        COMBI_WAITING_GROUP_RECT.width,
        COMBI_WAITING_GROUP_RECT.height,
    );
    CombiModeLayout0104 {
        viewport_width,
        viewport_height,
        full_backdrop,
        panel,
        right_backplate,
        main_group: panel,
        pc_stuff_panel: item_mode.pc_stuff_panel.into(),
        equipment_panel: item_mode.equipment_panel.into(),
        shade,
        success_group,
        waiting_group,
    }
}

pub(super) fn bind_combi_rect(node: &mut Node, rect: CombiUiRect) {
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
