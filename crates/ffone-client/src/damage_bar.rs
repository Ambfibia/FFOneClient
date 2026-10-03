//! Delayed damage feedback shared by HUD health and stamina bars.
use bevy::prelude::*;

const HOLD_SECONDS: f32 = 0.4;
const LERP_SPEED: f32 = 5.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct DamageTrail {
    value: Option<f32>,
    hold: f32,
}

impl DamageTrail {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn advance(&mut self, value: f32, delta: f32) -> f32 {
        let value = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let delta = if delta.is_finite() {
            delta.max(0.0)
        } else {
            0.0
        };
        let mut previous = self.value.unwrap_or(value);
        if value < previous {
            if self.hold > 0.0 {
                self.hold -= delta;
            } else {
                previous += (value - previous) * (delta * LERP_SPEED).min(1.0);
                if previous - value < 0.001 {
                    previous = value;
                }
            }
        } else {
            previous = value;
            self.hold = HOLD_SECONDS;
        }
        self.value = Some(previous);
        previous
    }
}

/// Updated by the presentation owner when a reused bar changes character/Nano.
#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
pub struct DamageBarOwner(pub u64);

#[derive(Component)]
struct DamageBar {
    fill: Entity,
    width: f32,
    owner: Option<u64>,
    trail: DamageTrail,
}

pub struct DamageBarPlugin;

impl Plugin for DamageBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_damage_bars.before(bevy::ui::UiSystems::Layout),
        );
    }
}

/// Paint the red texture before the current fill, retaining its full UV range.
pub fn spawn_damage_bar(
    parent: &mut ChildSpawnerCommands,
    node: Node,
    image: ImageNode,
    bundle: impl Bundle,
) {
    let Val::Px(width) = node.width else {
        panic!("damage bars require a fixed maximum width")
    };
    let mut back_node = node.clone();
    back_node.display = Display::None;
    let mut back_image = image.clone();
    back_image.color = Color::srgb(1.0, 0.0, 0.0);
    let back = parent
        .spawn((back_node, back_image, Pickable::IGNORE, ZIndex::default()))
        .id();
    let fill = parent
        .spawn((node, image, bundle, DamageBarOwner::default()))
        .id();
    parent.commands().entity(back).insert(DamageBar {
        fill,
        width,
        owner: None,
        trail: DamageTrail::default(),
    });
}

fn update_damage_bars(
    time: Res<Time>,
    fills: Query<
        (
            &Node,
            &DamageBarOwner,
            &InheritedVisibility,
            Option<&ZIndex>,
        ),
        Without<DamageBar>,
    >,
    ancestors: Query<(&Node, Option<&ChildOf>), Without<DamageBar>>,
    mut bars: Query<(&mut DamageBar, &mut Node, &mut ZIndex)>,
) {
    for (mut bar, mut node, mut layer) in &mut bars {
        let Ok((fill, owner, visibility, fill_layer)) = fills.get(bar.fill) else {
            continue;
        };
        let Val::Px(width) = fill.width else { continue };
        let mut parent = Some(bar.fill);
        let mut layout_visible = true;
        while let Some(entity) = parent {
            let Ok((ancestor, relationship)) = ancestors.get(entity) else {
                break;
            };
            if ancestor.display == Display::None {
                layout_visible = false;
                break;
            }
            parent = relationship.map(ChildOf::parent);
        }
        if !layout_visible || !visibility.get() {
            bar.trail.reset();
            bar.owner = None;
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        }
        if bar.owner != Some(owner.0) {
            bar.trail.reset();
            bar.owner = Some(owner.0);
        }
        let next_layer = fill_layer.copied().unwrap_or_default();
        if *layer != next_layer {
            *layer = next_layer;
        }
        let fraction = width / bar.width;
        let trail = bar.trail.advance(fraction, time.delta_secs());
        let next_width = px(bar.width * trail);
        let display = if trail > fraction {
            Display::Flex
        } else {
            Display::None
        };
        if node.width != next_width {
            node.width = next_width;
        }
        if node.display != display {
            node.display = display;
        }
    }
}

#[cfg(test)]
mod tests;
