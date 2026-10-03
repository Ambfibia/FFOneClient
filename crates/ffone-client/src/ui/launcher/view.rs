use super::*;

pub(super) fn spawn_launcher_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = LauncherUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());

    commands
        .spawn((
            LauncherUiElement::Root,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            GlobalZIndex(LAUNCHER_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            for slot in 0..4 {
                root.spawn((
                    LauncherUiElement::Backdrop(slot),
                    absolute_node(),
                    sliced_backdrop_image(assets.backdrop.clone()),
                    Pickable::IGNORE,
                ));
            }
            root.spawn((
                LauncherUiElement::Crosshair,
                absolute_node(),
                stretch_image(assets.crosshair.clone()),
                Pickable::IGNORE,
            ));
            root.spawn((
                LauncherUiElement::PowerLabel,
                centered_label_node(),
                Pickable::IGNORE,
            ))
            .with_child((
                LauncherUiTextRole::Power,
                Node {
                    width: percent(100),
                    ..default()
                },
                Text::new(LAUNCHER_UI_POWER_LABEL_KEY),
                LocalizedText::new(
                    LAUNCHER_UI_POWER_LOCALIZATION_KEY,
                    LAUNCHER_UI_POWER_LABEL_KEY,
                ),
                (
                    TextFont {
                        font: (assets.font.clone()).into(),
                        font_size: (LAUNCHER_UI_DEFAULT_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(LAUNCHER_UI_DEFAULT_LINE_HEIGHT),
                ),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            root.spawn((
                LauncherUiElement::Gauge,
                absolute_node(),
                stretch_image(assets.gauge.clone()),
                Pickable::IGNORE,
            ));
            root.spawn((
                LauncherUiElement::TipLabel,
                centered_label_node(),
                Pickable::IGNORE,
            ))
            .with_child((
                LauncherUiTextRole::Tip,
                Node {
                    width: percent(100),
                    ..default()
                },
                Text::new(LAUNCHER_UI_TIP_LABEL_KEY),
                LocalizedText::new(LAUNCHER_UI_TIP_LOCALIZATION_KEY, LAUNCHER_UI_TIP_LABEL_KEY),
                (
                    TextFont {
                        font: (assets.font.clone()).into(),
                        font_size: (LAUNCHER_UI_SMALL_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(LAUNCHER_UI_SMALL_LINE_HEIGHT),
                ),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            root.spawn((
                LauncherUiElement::GaugeBar,
                absolute_node(),
                stretch_image(assets.gauge_bar.clone()),
                Pickable::IGNORE,
            ));
        });
}
