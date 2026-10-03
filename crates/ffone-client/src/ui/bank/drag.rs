use super::*;

#[derive(Default, Resource)]
pub(super) struct DragVisual {
    pub(super) active: bool,
    origin: Vec2,
    cursor: Vec2,
    elapsed: f32,
    pub(super) dropping: bool,
    icon: UserEquipPresentationIcon,
}
#[derive(Component)]
pub(super) struct Ghost;
impl DragVisual {
    pub(super) fn start(
        &mut self,
        source: BankSlotRef0104,
        projection: &BankModeProjection0104,
        origin: Vec2,
    ) {
        let icon = match source.location {
            BankSlotLocation0104::Bank => &projection.bank[source.index].icon,
            BankSlotLocation0104::Inventory => {
                &projection.item_mode.inventory[source.index].item.icon
            }
        };
        *self = Self {
            active: true,
            origin,
            cursor: origin,
            icon: UserEquipPresentationIcon::from_projection(icon),
            ..default()
        };
    }
    pub(super) fn follow(&mut self, cursor: Vec2) {
        if !self.dropping {
            self.cursor = cursor;
        }
    }
    pub(super) fn drop_at(&mut self, destination: Option<Vec2>) {
        if self.active {
            self.cursor = destination.unwrap_or(self.origin);
            self.dropping = true;
            self.elapsed = 0.;
        }
    }
    pub(super) fn cancel(&mut self) {
        if self.active {
            *self = default();
        }
    }
}
pub(super) fn spawn(mut commands: Commands) {
    commands.spawn((
        Name::new("Bank dragged item"),
        Ghost,
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            ..default()
        },
        ImageNode::default(),
        GlobalZIndex(65),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
}
fn berp(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    ((t * std::f32::consts::PI * (0.2 + 2.5 * t * t * t)).sin() * (1. - t).powf(2.2) + t)
        * (1. + 1.2 * (1. - t))
}
pub(super) fn bind(
    time: Res<Time>,
    state: Res<BankUiState>,
    modal: Res<BankModalState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut visual: ResMut<DragVisual>,
    asset_server: Res<AssetServer>,
    assets: Res<BankUiRuntimeAssets>,
    mut ghosts: Query<(&mut Node, &mut ImageNode), With<Ghost>>,
) {
    // A sent transfer may lock further input while its short visual tail finishes.
    if state.phase == BankLifecyclePhase::Hidden
        || modal.any()
        || windows.iter().any(|w| !w.focused)
    {
        visual.cancel();
    }
    if visual.active {
        visual.elapsed += time.delta_secs();
    }
    if visual.dropping && visual.elapsed >= 0.3 {
        visual.cancel();
    }
    for (mut node, mut image) in &mut ghosts {
        if !visual.active {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        }
        let t = (visual.elapsed / 0.3).clamp(0., 1.);
        let (size, alpha, center) = if visual.dropping {
            (64. * (1.25 - 0.25 * berp(t)), 0.4 * (1. - t), visual.cursor)
        } else {
            (
                64. * (1. + 0.25 * berp(t)),
                0.1 + 0.3 * t,
                visual.origin.lerp(visual.cursor, t),
            )
        };
        bind_presentation_icon(
            &mut node,
            Some(image.reborrow()),
            &visual.icon,
            &asset_server,
            &assets.0,
        );
        node.left = px(center.x - size / 2.);
        node.top = px(center.y - size / 2.);
        node.width = px(size);
        node.height = px(size);
        image.color = Color::srgba(1., 1., 1., alpha);
    }
}
#[cfg(test)]
mod tests;
