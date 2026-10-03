use super::*;

pub(super) fn spawn_tutorial_choreography_overlay(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<TutorialPanAssets>,
) {
    // The clean AssetLoader fetched the Tutorial package before play. Keep the
    // eight PanScene frames resident from startup so the first activation only
    // toggles presentation state and never starts disk I/O mid-cutscene.
    assets.handles = Some(std::array::from_fn(|index| {
        asset_server.load(TUTORIAL_PAN_PATHS[index])
    }));
    commands.spawn((
        TutorialChoreographyOverlay::Fade,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        Visibility::Hidden,
        ZIndex(1_900),
    ));
    for (marker, top, bottom) in [
        (TutorialChoreographyOverlay::TopBar, Val::Px(0.0), Val::Auto),
        (
            TutorialChoreographyOverlay::BottomBar,
            Val::Auto,
            Val::Px(0.0),
        ),
    ] {
        commands.spawn((
            marker,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top,
                bottom,
                height: Val::Percent(TUTORIAL_CINEMATIC_BAR_HEIGHT_FRACTION * 100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
            ZIndex(1_901),
        ));
    }
    for index in 0..8 {
        commands.spawn((
            TutorialPanFrame(index),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                ..default()
            },
            ImageNode::default(),
            Visibility::Hidden,
            ZIndex(1_890),
        ));
    }
}
