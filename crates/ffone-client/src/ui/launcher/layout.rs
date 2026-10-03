use super::*;

pub const LAUNCHER_UI_DEFAULT_LINE_HEIGHT: f32 = 13.710_000_038_146_973;

pub const LAUNCHER_UI_SMALL_LINE_HEIGHT: f32 = 8.225_999_832_153_32;

pub const LAUNCHER_UI_GAUGE_WIDTH: f32 = 169.0;

pub const LAUNCHER_UI_GAUGE_HEIGHT: f32 = 595.0;

pub const LAUNCHER_UI_GAUGE_AREA_WIDTH: f32 = 87.0;

pub const LAUNCHER_UI_GAUGE_AREA_HEIGHT: f32 = 500.0;

pub const LAUNCHER_UI_BAR_WIDTH: f32 = 87.0;

pub const LAUNCHER_UI_BAR_HEIGHT: f32 = 31.0;

pub const LAUNCHER_UI_BACKDROP_BORDER: BorderRect = BorderRect::all(2.0);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LauncherUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LauncherUiRect {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn translated(self, x: f32, y: f32) -> Self {
        Self::new(self.x + x, self.y + y, self.width, self.height)
    }

    pub(super) fn apply_to(self, node: &mut Node) {
        node.position_type = PositionType::Absolute;
        node.left = px(self.x);
        node.top = px(self.y);
        node.width = px(self.width);
        node.height = px(self.height);
    }
}

pub const LAUNCHER_UI_CROSS_SOURCE_RECT: LauncherUiRect =
    LauncherUiRect::new(0.0, 0.0, 660.0, 660.0);

pub const LAUNCHER_UI_GAUGE_SOURCE_RECT: LauncherUiRect =
    LauncherUiRect::new(0.0, 0.0, 169.0, 595.0);

pub const LAUNCHER_UI_GAUGE_AREA_SOURCE_RECT: LauncherUiRect =
    LauncherUiRect::new(42.0, 30.0, 87.0, 500.0);

pub const LAUNCHER_UI_BAR_SOURCE_RECT: LauncherUiRect = LauncherUiRect::new(0.0, 0.0, 87.0, 31.0);

pub const LAUNCHER_UI_POWER_SOURCE_RECT: LauncherUiRect =
    LauncherUiRect::new(0.0, 0.0, 169.0, 30.0);

pub const LAUNCHER_UI_TIP_SOURCE_RECT: LauncherUiRect =
    LauncherUiRect::new(10.0, 560.0, 150.0, 30.0);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LauncherUiLayout {
    pub viewport: Vec2,
    pub backdrop: [LauncherUiRect; 4],
    pub crosshair: LauncherUiRect,
    pub gauge: LauncherUiRect,
    pub power_label: LauncherUiRect,
    pub tip_label: LauncherUiRect,
    pub gauge_bar: LauncherUiRect,
}

impl LauncherUiLayout {
    #[must_use]
    pub fn from_viewport(viewport: Vec2, normalized_power: f32) -> Self {
        let viewport = Vec2::new(
            finite_or_zero(viewport.x).max(0.0),
            finite_or_zero(viewport.y).max(0.0),
        );
        // Unity's Screen dimensions are integers and C# casts truncate toward
        // zero. Preserve that behavior instead of introducing DPI/UI scaling.
        let top_height = ((viewport.y - LAUNCHER_UI_CROSS_SIZE) * 0.5).trunc();
        let side_width = ((viewport.x - LAUNCHER_UI_CROSS_SIZE) * 0.5).trunc();
        let crosshair = LauncherUiRect::new(
            side_width,
            top_height,
            LAUNCHER_UI_CROSS_SIZE,
            LAUNCHER_UI_CROSS_SIZE,
        );
        let backdrop = [
            LauncherUiRect::new(0.0, 0.0, viewport.x, top_height),
            LauncherUiRect::new(0.0, top_height, side_width, viewport.y - top_height),
            // `CnGuiLauncher.Repos` contains this oversized-height expression;
            // the top begins at the cross bottom, so clipping makes it harmless.
            LauncherUiRect::new(
                crosshair.x,
                crosshair.y + crosshair.height,
                viewport.x - crosshair.x,
                viewport.y - crosshair.y + crosshair.height,
            ),
            LauncherUiRect::new(
                crosshair.x + crosshair.width,
                top_height,
                viewport.x - crosshair.x - crosshair.width,
                crosshair.height,
            ),
        ];
        let gauge = LauncherUiRect::new(
            crosshair.x - (LAUNCHER_UI_GAUGE_WIDTH + 10.0),
            crosshair.y + 30.0,
            LAUNCHER_UI_GAUGE_WIDTH,
            LAUNCHER_UI_GAUGE_HEIGHT,
        );
        // The clean code subtracts rectPower.width, not rectPower.height.
        let power_label = LauncherUiRect::new(
            gauge.x,
            gauge.y - LAUNCHER_UI_POWER_SOURCE_RECT.width,
            LAUNCHER_UI_POWER_SOURCE_RECT.width,
            LAUNCHER_UI_POWER_SOURCE_RECT.height,
        );
        let tip_label = LAUNCHER_UI_TIP_SOURCE_RECT.translated(gauge.x, gauge.y);
        let normalized_power = finite_or_zero(normalized_power).clamp(0.0, 1.0);
        let bar_y =
            (LAUNCHER_UI_GAUGE_AREA_HEIGHT - LAUNCHER_UI_BAR_HEIGHT) * (1.0 - normalized_power);
        let gauge_bar = LauncherUiRect::new(
            gauge.x + LAUNCHER_UI_GAUGE_AREA_X,
            gauge.y + LAUNCHER_UI_GAUGE_AREA_Y + bar_y,
            LAUNCHER_UI_BAR_WIDTH,
            LAUNCHER_UI_BAR_HEIGHT,
        );
        Self {
            viewport,
            backdrop,
            crosshair,
            gauge,
            power_label,
            tip_label,
            gauge_bar,
        }
    }
}

pub(super) fn sync_launcher_layout(
    model: Res<LauncherUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut elements: Query<(&LauncherUiElement, &mut Node)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let layout = LauncherUiLayout::from_viewport(
        Vec2::new(window.width(), window.height()),
        model.normalized_power(),
    );
    for (element, mut node) in &mut elements {
        match *element {
            LauncherUiElement::Root => {
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
            LauncherUiElement::Backdrop(slot) => layout.backdrop[slot].apply_to(&mut node),
            LauncherUiElement::Crosshair => layout.crosshair.apply_to(&mut node),
            LauncherUiElement::Gauge => layout.gauge.apply_to(&mut node),
            LauncherUiElement::GaugeBar => layout.gauge_bar.apply_to(&mut node),
            LauncherUiElement::PowerLabel => layout.power_label.apply_to(&mut node),
            LauncherUiElement::TipLabel => layout.tip_label.apply_to(&mut node),
        }
    }
}
