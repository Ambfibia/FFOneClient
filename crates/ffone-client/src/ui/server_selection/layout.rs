use super::*;

pub const SERVER_SELECTION_INITIAL_SCROLL_HEIGHT: f32 = 402.0;

pub const SERVER_SELECTION_COLLAPSED_SCROLL_HEIGHT: f32 = 30.0;

pub const SERVER_SELECTION_EXPANDED_SCROLL_HEIGHT: f32 = 492.0;

pub const SERVER_SELECTION_REFERENCE_BACKGROUND_WIDTH: f32 = 1_020.0;

pub const SERVER_SELECTION_REFERENCE_BACKGROUND_HEIGHT: f32 = 638.0;

pub const SERVER_SELECTION_JEFFE_16_LINE_HEIGHT: f32 = 16.451_999_66;

pub const SERVER_SELECTION_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ServerSelectionUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl ServerSelectionUiRect {
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

    pub(super) fn apply(self, node: &mut Node) {
        node.position_type = PositionType::Absolute;
        node.left = px(self.left);
        node.top = px(self.top);
        node.width = px(self.width);
        node.height = px(self.height);
    }

    pub(super) fn node(self) -> Node {
        let mut node = Node::default();
        self.apply(&mut node);
        node
    }
}

pub const SERVER_SELECTION_PANEL_LOCAL_RECT: ServerSelectionUiRect =
    ServerSelectionUiRect::new(0.0, 0.0, 375.0, 330.0);

pub const SERVER_SELECTION_INNER_RECT: ServerSelectionUiRect =
    ServerSelectionUiRect::new(37.0, 46.0, 295.0, 235.0);

pub const SERVER_SELECTION_VIEWPORT_RECT: ServerSelectionUiRect =
    ServerSelectionUiRect::new(37.0, 50.0, 293.0, 227.0);

pub const SERVER_SELECTION_CONNECT_RECT: ServerSelectionUiRect =
    ServerSelectionUiRect::new(113.0, 290.0, 135.0, 28.0);

pub const SERVER_SELECTION_SERVER_ROW_RECT: ServerSelectionUiRect =
    ServerSelectionUiRect::new(0.0, 7.0, 272.0, 17.0);

pub const SERVER_SELECTION_CHANNEL_STATUS_RECT: ServerSelectionUiRect =
    ServerSelectionUiRect::new(168.0, 0.0, 45.0, 16.0);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ServerSelectionUiLayout {
    pub viewport: Vec2,
    pub background: ServerSelectionUiRect,
    pub panel: ServerSelectionUiRect,
    pub account: ServerSelectionUiRect,
    pub homepage: ServerSelectionUiRect,
    pub quit: ServerSelectionUiRect,
    pub scroll_track: ServerSelectionUiRect,
    pub scroll_up: ServerSelectionUiRect,
    pub scroll_down: ServerSelectionUiRect,
    pub scroll_thumb: ServerSelectionUiRect,
}

impl ServerSelectionUiLayout {
    #[must_use]
    pub fn from_viewport(viewport: Vec2, scroll_y: f32, expanded: bool) -> Self {
        Self::from_content_height(
            viewport,
            scroll_y,
            if expanded {
                SERVER_SELECTION_EXPANDED_SCROLL_HEIGHT
            } else {
                SERVER_SELECTION_COLLAPSED_SCROLL_HEIGHT
            },
        )
    }

    #[must_use]
    pub fn from_content_height(viewport: Vec2, scroll_y: f32, content_height: f32) -> Self {
        let width = finite_nonnegative(viewport.x).trunc();
        let height = finite_nonnegative(viewport.y).trunc();
        let half_width = ((width as i32) / 2) as f32;
        let half_height = ((height as i32) / 2) as f32;
        let scale = (width / SERVER_SELECTION_REFERENCE_BACKGROUND_WIDTH)
            .min(height / SERVER_SELECTION_REFERENCE_BACKGROUND_HEIGHT)
            .max(0.0);
        let background_width = SERVER_SELECTION_REFERENCE_BACKGROUND_WIDTH * scale;
        let background_height = SERVER_SELECTION_REFERENCE_BACKGROUND_HEIGHT * scale;
        let background = ServerSelectionUiRect::new(
            (width - background_width) * 0.5,
            (height - background_height) * 0.5,
            background_width,
            background_height,
        );
        let panel = ServerSelectionUiRect::new(
            half_width + 73.0,
            half_height - 160.0,
            SERVER_SELECTION_PANEL_LOCAL_RECT.width,
            SERVER_SELECTION_PANEL_LOCAL_RECT.height,
        );
        let account =
            ServerSelectionUiRect::new(half_width + 160.0, half_height + 180.0, 200.0, 27.0);
        let homepage =
            ServerSelectionUiRect::new(half_width + 160.0, half_height + 212.0, 200.0, 27.0);
        let quit = ServerSelectionUiRect::new(half_width + 198.0, half_height + 244.0, 125.0, 27.0);

        let chrome_left = SERVER_SELECTION_VIEWPORT_RECT.left + 277.0;
        let scroll_up = ServerSelectionUiRect::new(chrome_left, 50.0, 17.0, 12.0);
        let scroll_down = ServerSelectionUiRect::new(chrome_left, 265.0, 17.0, 12.0);
        let scroll_track = ServerSelectionUiRect::new(chrome_left, 62.0, 18.0, 203.0);
        let content_height = finite_nonnegative(content_height);
        let maximum_scroll = (content_height - SERVER_SELECTION_VIEWPORT_RECT.height).max(0.0);
        let clamped_scroll = finite_nonnegative(scroll_y).min(maximum_scroll);
        let thumb_height = if maximum_scroll > 0.0 {
            (scroll_track.height * SERVER_SELECTION_VIEWPORT_RECT.height / content_height)
                .max(15.0)
                .min(scroll_track.height)
        } else {
            scroll_track.height
        };
        let thumb_travel = (scroll_track.height - thumb_height).max(0.0);
        let scroll_thumb = ServerSelectionUiRect::new(
            chrome_left + 2.0,
            scroll_track.top
                + if maximum_scroll > 0.0 {
                    thumb_travel * clamped_scroll / maximum_scroll
                } else {
                    0.0
                },
            13.0,
            thumb_height,
        );

        Self {
            viewport: Vec2::new(width, height),
            background,
            panel,
            account,
            homepage,
            quit,
            scroll_track,
            scroll_up,
            scroll_down,
            scroll_thumb,
        }
    }
}

pub(super) fn bind_server_selection_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut model: ResMut<ServerSelectionUiModel>,
    mut elements: Query<(&ServerSelectionUiElement, &mut Node)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let layout = ServerSelectionUiLayout::from_content_height(
        Vec2::new(window.width(), window.height()),
        model.scroll_y,
        model.scroll_view_height,
    );
    let scrollbar_visible = model.scroll_view_height > SERVER_SELECTION_VIEWPORT_RECT.height;
    for (element, mut node) in &mut elements {
        match *element {
            ServerSelectionUiElement::Root => {
                node.left = px(0);
                node.top = px(0);
                node.width = px(layout.viewport.x);
                node.height = px(layout.viewport.y);
                node.display = if model.visible() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ServerSelectionUiElement::Background => layout.background.apply(&mut node),
            ServerSelectionUiElement::Panel => layout.panel.apply(&mut node),
            ServerSelectionUiElement::ScrollContent => {
                node.left = px(0);
                node.top = px(-model.scroll_y);
                node.width = px(272);
                node.height = px(model.scroll_view_height);
            }
            ServerSelectionUiElement::ChannelRow(_) => {
                node.display = if model.server_expanded {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ServerSelectionUiElement::ScrollTrack => {
                layout.scroll_track.apply(&mut node);
                node.display = if scrollbar_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ServerSelectionUiElement::ScrollThumb => {
                layout.scroll_thumb.apply(&mut node);
                node.display = if scrollbar_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ServerSelectionUiElement::ScrollUp => {
                layout.scroll_up.apply(&mut node);
                node.display = if scrollbar_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ServerSelectionUiElement::ScrollDown => {
                layout.scroll_down.apply(&mut node);
                node.display = if scrollbar_visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ServerSelectionUiElement::AccountButton => layout.account.apply(&mut node),
            ServerSelectionUiElement::HomepageButton => layout.homepage.apply(&mut node),
            ServerSelectionUiElement::QuitButton => layout.quit.apply(&mut node),
        }
    }
    if model.visible() {
        model.complete_gui_pass();
    }
}
