use super::*;

pub const NANO_FREE_TUNING_PANEL_WIDTH: f32 = 356.0;

pub const NANO_FREE_TUNING_PANEL_HEIGHT: f32 = 460.0;

pub const NANO_FREE_TUNING_PANEL_SOURCE_WIDTH: u32 = 357;

pub const NANO_FREE_TUNING_PANEL_SOURCE_HEIGHT: u32 = 460;

pub const NANO_FREE_TUNING_JEFFE_16_LINE_HEIGHT: f32 = 16.451_999_66;

pub const NANO_FREE_TUNING_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const NANO_FREE_TUNING_JEFFE_08_LINE_HEIGHT: f32 = 10.968_000_41;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanoFreeTuningRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl NanoFreeTuningRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(super) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.x),
            top: px(self.y),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanoFreeTuningPowerLayout {
    pub icon: NanoFreeTuningRect,
    pub name: NanoFreeTuningRect,
    pub power_type: NanoFreeTuningRect,
    pub description: NanoFreeTuningRect,
    pub select_button: NanoFreeTuningRect,
}

pub const NANO_FREE_TUNING_TITLE_RECT: NanoFreeTuningRect =
    NanoFreeTuningRect::new(100.0, 100.0, 300.0, 15.0);

pub const NANO_FREE_TUNING_POWER_LAYOUTS: [NanoFreeTuningPowerLayout; 3] = [
    NanoFreeTuningPowerLayout {
        icon: NanoFreeTuningRect::new(18.0, 139.0, 35.0, 35.0),
        name: NanoFreeTuningRect::new(50.0, 143.0, 200.0, 15.0),
        power_type: NanoFreeTuningRect::new(50.0, 158.0, 150.0, 12.0),
        description: NanoFreeTuningRect::new(20.0, 180.0, 317.0, 25.0),
        select_button: NanoFreeTuningRect::new(237.0, 145.0, 97.0, 20.0),
    },
    NanoFreeTuningPowerLayout {
        icon: NanoFreeTuningRect::new(18.0, 221.0, 35.0, 35.0),
        name: NanoFreeTuningRect::new(51.0, 226.0, 200.0, 15.0),
        power_type: NanoFreeTuningRect::new(51.0, 241.0, 150.0, 12.0),
        description: NanoFreeTuningRect::new(20.0, 261.0, 315.0, 25.0),
        select_button: NanoFreeTuningRect::new(237.0, 232.0, 97.0, 20.0),
    },
    NanoFreeTuningPowerLayout {
        icon: NanoFreeTuningRect::new(18.0, 304.0, 35.0, 35.0),
        name: NanoFreeTuningRect::new(51.0, 309.0, 200.0, 15.0),
        power_type: NanoFreeTuningRect::new(51.0, 324.0, 150.0, 12.0),
        description: NanoFreeTuningRect::new(20.0, 345.0, 315.0, 25.0),
        select_button: NanoFreeTuningRect::new(237.0, 317.0, 97.0, 20.0),
    },
];

#[must_use]
pub fn nano_free_tuning_black_bar_rects(
    viewport_width: f32,
    viewport_height: f32,
) -> [NanoFreeTuningRect; 2] {
    let height = viewport_height * NANO_FREE_TUNING_BLACK_BAR_RATIO;
    [
        NanoFreeTuningRect::new(0.0, 0.0, viewport_width, height),
        NanoFreeTuningRect::new(0.0, viewport_height - height, viewport_width, height),
    ]
}

/// The rectangle actually passed to `GUI.BeginGroup` in clean code.
#[must_use]
pub fn nano_free_tuning_panel_rect(
    viewport_width: f32,
    viewport_height: f32,
) -> NanoFreeTuningRect {
    NanoFreeTuningRect::new(
        viewport_width / 2.0 + 100.0,
        viewport_height / 2.0 - 230.0,
        NANO_FREE_TUNING_PANEL_WIDTH,
        NANO_FREE_TUNING_PANEL_HEIGHT,
    )
}

/// `CnGuiNanoFreeTuning` computes this `val2` but never renders with it.
#[must_use]
pub fn nano_free_tuning_dormant_panel_rect(
    viewport_width: f32,
    viewport_height: f32,
) -> NanoFreeTuningRect {
    NanoFreeTuningRect::new(
        viewport_width * 3.0 / 4.0 - 178.0,
        viewport_height / 2.0 - 230.0,
        NANO_FREE_TUNING_PANEL_WIDTH,
        NANO_FREE_TUNING_PANEL_HEIGHT,
    )
}

pub(super) fn sync_nano_free_tuning_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<NanoFreeTuningModel>,
    mut roots: Query<
        &mut Node,
        (
            With<NanoFreeTuningPresentationRoot>,
            Without<NanoFreeTuningPresentationPanel>,
            Without<NanoFreeTuningPresentationBar>,
            Without<NanoFreeTuningAnnouncement>,
        ),
    >,
    mut panels: Query<
        &mut Node,
        (
            With<NanoFreeTuningPresentationPanel>,
            Without<NanoFreeTuningPresentationRoot>,
            Without<NanoFreeTuningPresentationBar>,
            Without<NanoFreeTuningAnnouncement>,
        ),
    >,
    mut bars: Query<
        (&NanoFreeTuningPresentationBar, &mut Node),
        (
            Without<NanoFreeTuningPresentationRoot>,
            Without<NanoFreeTuningPresentationPanel>,
            Without<NanoFreeTuningAnnouncement>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport_width = window.resolution.width();
    let viewport_height = window.resolution.height();
    for mut root in &mut roots {
        root.display = if model.panel_visible() {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut panel in &mut panels {
        let rect = nano_free_tuning_panel_rect(viewport_width, viewport_height);
        panel.left = px(rect.x);
        panel.top = px(rect.y);
        panel.width = px(rect.width);
        panel.height = px(rect.height);
    }
    let bar_rects = nano_free_tuning_black_bar_rects(viewport_width, viewport_height);
    for (bar, mut node) in &mut bars {
        let rect = bar_rects[usize::from(bar.bottom)];
        node.left = px(rect.x);
        node.top = px(rect.y);
        node.width = px(rect.width);
        node.height = px(rect.height);
    }
}
