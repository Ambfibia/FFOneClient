//! Currency counters and the persistent, authoritative inventory-full badge.
use super::*;

pub const STATUS_TEXTURES: [(&str, usize, &str); 3] = [
    (
        "ui/en/gameplay/rewards/fusion-matter.png",
        10944,
        "97b874456be11c070fc626b12e410629d530414807f3171595c8913fba6fad70",
    ),
    (
        "ui/en/gameplay/rewards/taros.png",
        10704,
        "7bf96c14cf70627b180dcb3242e34f978e43b884dd80ada3a82ec406659a0709",
    ),
    (
        "ui/en/gameplay/rewards/inventory-full.png",
        2243,
        "403f1b547fd461278d30a6e5edb980080ac7338568ab09f0840c91e051252291",
    ),
];

#[derive(Clone, Debug, Default)]
struct Counter {
    current: i32,
    target: i32,
    remaining: f32,
}

impl Counter {
    fn visible(&self) -> bool {
        self.current != self.target || self.remaining > 0.0
    }

    fn receive(&mut self, previous: i32, target: i32, initialized: bool) {
        self.current = if initialized {
            self.current.min(previous)
        } else {
            previous
        };
        self.target = target;
        if target > previous {
            self.remaining = 2.0;
        }
    }

    fn step(&mut self, delta: f32) -> bool {
        let difference = i64::from(self.target) - i64::from(self.current);
        if difference != 0 {
            self.current += (difference.signum() * (difference.abs() / 5).clamp(1, 10)) as i32;
            true
        } else {
            self.remaining = (self.remaining - delta).max(0.0);
            false
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct RewardStatus {
    counters: [Counter; 2], // FM, then Taros, preserving paint order.
    initialized: bool,
    since_step: f32,
    inventory_full: Option<bool>,
    full: bool,
    full_remaining: f32,
    since_inventory_check: f32,
}

impl RewardStatus {
    pub(super) fn needs_tick(&self) -> bool {
        self.inventory_full.is_some() || self.full || self.counters.iter().any(Counter::visible)
    }

    pub(super) fn advance(&mut self, delta: f32, crate_active: bool) {
        self.since_step += delta;
        if self.since_step > 0.1 {
            // The source shares one step clock; hold/fade timers run only on
            // eligible frames and the clock resets only when a value moves.
            let fm_changed = self.counters[0].step(delta);
            let taros_changed = self.counters[1].step(delta);
            if fm_changed || taros_changed {
                self.since_step = 0.0;
            }
        }
        self.full_remaining = (self.full_remaining - delta).max(0.0);
        self.since_inventory_check += delta;
        if !crate_active && self.since_inventory_check > 1.5 {
            self.since_inventory_check = 0.0;
            let full = self.inventory_full == Some(true);
            if full && !self.full {
                self.full_remaining = 2.0;
            }
            self.full = full;
        }
    }
}

impl RewardNotices {
    /// Feed only the validated reward packet, before replacing runtime totals.
    /// Merely loading a wallet or buying/selling an item must not create rewards.
    pub fn receive_currencies(
        &mut self,
        previous_taros: i32,
        previous_fm: i32,
        taros: i32,
        fm: i32,
    ) {
        self.status.counters[0].receive(previous_fm, fm, self.status.initialized);
        self.status.counters[1].receive(previous_taros, taros, self.status.initialized);
        self.status.initialized = true;
    }

    /// None means no authoritative world inventory (including session exit).
    /// Actual visibility follows the original 1.5-second check and crate gate.
    pub fn set_inventory_full(&mut self, full: Option<bool>) {
        if self.status.inventory_full != full {
            self.status.inventory_full = full;
        }
        if full.is_none() {
            self.status.full = false;
            self.status.full_remaining = 0.0;
            self.status.since_inventory_check = 0.0;
        }
    }
}

#[derive(Component)]
pub(super) enum StatusImage {
    Currency(usize),
    FullInventory,
}

#[derive(Component)]
pub(super) struct CurrencyDigit {
    lane: usize,
    index: usize,
}

fn digit_text(value: i32) -> LocalizedText {
    LocalizedText::new("ui.gameplay.reward.counter_digit", "{digit}")
        .with_arg("digit", value.to_string())
}

pub(super) fn spawn(
    hud: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
    server: &AssetServer,
) {
    for lane in 0..2 {
        let size = if lane == 0 {
            Vec2::new(172.0, 63.0)
        } else {
            Vec2::new(170.0, 64.0)
        };
        hud.spawn((
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: px(size.x),
                height: px(size.y),
                ..default()
            },
            ImageNode {
                image: load_nano_wheel_image(server, STATUS_TEXTURES[lane].0),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            StatusImage::Currency(lane),
            Pickable::IGNORE,
            ZIndex(941),
        ))
        .with_children(|panel| {
            for index in 0..9 {
                panel.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(6 + 12 * index),
                        top: px(31),
                        width: px(10),
                        height: px(20),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Text::new("0"),
                    digit_text(0),
                    chat_jeffe_14_font(&assets.jeffe_font),
                    UiTransform::from_scale(Vec2::new(1.0, CHAT_JEFFE_14_VERTICAL_SCALE)),
                    TextLayout::default().with_justify(Justify::Center),
                    TextColor(Color::WHITE),
                    CurrencyDigit { lane, index },
                    Pickable::IGNORE,
                ));
            }
        });
    }
    hud.spawn((
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(178),
            height: px(142),
            ..default()
        },
        ImageNode {
            image: load_nano_wheel_image(server, REWARD_TEXTURES[0].0),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        StatusImage::FullInventory,
        Pickable::IGNORE,
        ZIndex(940),
    ))
    .with_children(|panel| {
        panel.spawn((
            // Source centers by integer texture half-sizes (41 / 2 == 20).
            Node {
                position_type: PositionType::Absolute,
                left: px(64),
                top: px(66),
                width: px(40),
                height: px(41),
                ..default()
            },
            ImageNode {
                image: load_nano_wheel_image(server, STATUS_TEXTURES[2].0),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            Pickable::IGNORE,
        ));
    });
}

fn currency_alpha(status: &RewardStatus, lane: usize) -> f32 {
    // Preserve the original Taros branch's FM-timer gate, including its
    // asymmetry when the two counters finish at different times.
    if status.counters[0].remaining < 1.0 {
        status.counters[lane].remaining.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

pub(super) fn update(
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<GameplayUiModel>,
    notices: Res<RewardNotices>,
    quick_slot: Option<Res<QuickSlotUiConfig>>,
    transition: Option<Res<GameplayMenuTransition>>,
    mut images: Query<
        (&StatusImage, &mut Node, &mut ImageNode, &mut UiTransform),
        Without<CurrencyDigit>,
    >,
    mut digits: Query<(&CurrencyDigit, &mut LocalizedText, &mut TextColor)>,
) {
    let Ok(window) = windows.single() else { return };
    let chat_editing = model.chat.active && transition.as_ref().is_none_or(|t| t.remaining <= 0.0);
    let visible = model.visible && !chat_editing;
    let quick = quick_slot
        .as_ref()
        .is_some_and(|c| c.localized_mode.draws_component());
    let viewport = Vec2::new(window.width(), window.height());
    let scale = model.ui_scale;
    let status = &notices.status;
    for (role, mut node, mut image, mut transform) in &mut images {
        let (show, size, position, alpha) = match *role {
            StatusImage::Currency(lane) => {
                let size = if lane == 0 {
                    Vec2::new(172.0, 63.0)
                } else {
                    Vec2::new(170.0, 64.0)
                };
                let position = Vec2::new(
                    viewport.x * 0.5 - if lane == 0 { 172.0 * scale } else { 0.0 },
                    viewport.y - (size.y + if quick { 40.0 } else { 0.0 }) * scale,
                );
                (
                    status.counters[lane].visible(),
                    size,
                    position,
                    currency_alpha(status, lane),
                )
            }
            StatusImage::FullInventory => {
                let (position, _) = icon_position(status.full_remaining, 0, viewport, scale, quick);
                (
                    status.full && notices.icons[0].is_empty(),
                    Vec2::new(178.0, 142.0),
                    position,
                    1.0,
                )
            }
        };
        let display = if visible && show {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if display == Display::None {
            continue;
        }
        let position = position + size * (scale - 1.0) * 0.5;
        if node.left != px(position.x) {
            node.left = px(position.x);
        }
        if node.top != px(position.y) {
            node.top = px(position.y);
        }
        transform.set_if_neq(UiTransform::from_scale(Vec2::splat(scale)));
        let tint = Color::srgba(1.0, 1.0, 1.0, alpha);
        if image.color != tint {
            image.color = tint;
        }
    }
    for (digit, mut text, mut color) in &mut digits {
        let value = status.counters[digit.lane].current.max(0);
        let divisor = 10_i32.pow(8 - digit.index as u32);
        let value = if digit.index == 0 {
            value / divisor
        } else {
            value / divisor % 10
        };
        text.set_if_neq(digit_text(value));
        color.set_if_neq(TextColor(Color::srgba(
            1.0,
            1.0,
            1.0,
            currency_alpha(status, digit.lane),
        )));
    }
}

#[cfg(test)]
mod tests;
