use super::*;

pub(super) fn spawn_world_backdrop(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.04, 0.15)),
        GlobalZIndex(-1_000),
        Pickable::IGNORE,
    ));
}
