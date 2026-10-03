use super::*;

pub(super) fn spawn_world_backdrop(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.46, 0.68)),
            GlobalZIndex(-1_000),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            for (left, top, width, height, color) in [
                (70.0, 70.0, 260.0, 160.0, Color::srgb(0.85, 0.42, 0.16)),
                (910.0, 90.0, 250.0, 190.0, Color::srgb(0.28, 0.78, 0.36)),
                (90.0, 465.0, 300.0, 135.0, Color::srgb(0.68, 0.24, 0.72)),
                (870.0, 450.0, 310.0, 145.0, Color::srgb(0.88, 0.75, 0.18)),
            ] {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(left),
                        top: px(top),
                        width: px(width),
                        height: px(height),
                        border: UiRect::all(px(3)),
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderColor::all(Color::WHITE),
                    Pickable::IGNORE,
                ));
            }
        });
}
