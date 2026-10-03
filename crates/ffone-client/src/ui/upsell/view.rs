use super::*;

pub(super) fn spawn_upsell_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = UpsellUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            UpsellUiRoot,
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
            GlobalZIndex(UPSELL_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                UpsellUiBackdrop,
                UPSELL_BACKGROUND_RECT.node(),
                UiTransform::default(),
                stretch_image(assets.panelback.clone()),
                Pickable::IGNORE,
            ));
            root.spawn((
                UpsellUiDialog,
                UPSELL_NEWS_DIALOG_RECT.node(),
                UiTransform::default(),
                ImageNode::default(),
                Pickable::IGNORE,
            ))
            .with_children(|dialog| {
                for kind in [
                    UpsellUiButtonKind::Close,
                    UpsellUiButtonKind::Continue,
                    UpsellUiButtonKind::GetUpgrade,
                    UpsellUiButtonKind::NotRightNow,
                ] {
                    spawn_upsell_button(dialog, kind, &assets);
                }
                dialog.spawn((
                    UpsellUiAdvertis,
                    UPSELL_LEVEL_ONE_ADVERTIS_RECT.node(),
                    stretch_image(assets.advertis.clone()),
                    Pickable::IGNORE,
                ));
            });
        });
}

pub(super) fn spawn_upsell_button(
    parent: &mut ChildSpawnerCommands,
    kind: UpsellUiButtonKind,
    assets: &UpsellUiAssets,
) {
    parent
        .spawn((
            Button,
            UpsellUiButton { kind },
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..UPSELL_NEWS_CLOSE_RECT.node()
            },
            button_image(assets, UpsellUiButtonVisual::Close, Interaction::None),
            Pickable::IGNORE,
        ))
        .with_children(|button| {
            if matches!(
                kind,
                UpsellUiButtonKind::Close | UpsellUiButtonKind::GetUpgrade
            ) {
                return;
            }
            button.spawn((
                UpsellUiButtonLabel { kind },
                Text::new(""),
                (
                    TextFont {
                        font: (assets.jeffe.clone()).into(),
                        font_size: (UPSELL_JEFFE_12_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(UPSELL_JEFFE_12_LINE_HEIGHT),
                ),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                upsell_label_localized(match kind {
                    UpsellUiButtonKind::Continue => UPSELL_CONTINUE_LABEL,
                    UpsellUiButtonKind::NotRightNow => UPSELL_NOT_RIGHT_NOW_LABEL,
                    UpsellUiButtonKind::Close | UpsellUiButtonKind::GetUpgrade => unreachable!(),
                }),
                Pickable::IGNORE,
            ));
        });
}
