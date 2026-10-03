use super::*;

pub(super) fn spawn_enchant_overlays_0104(parent: &mut ChildSpawnerCommands, assets: &EnchantUiAssets0104) {
    parent.spawn((
        EnchantUiElement0104::Shade,
        Node {
            display: Display::None,
            ..EnchantUiRect0104::default().node()
        },
        ImageNode {
            color: Color::srgba(1.0, 1.0, 1.0, 0.75),
            ..enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::BlackShade))
        },
        GlobalZIndex(ENCHANT_UI_Z_INDEX_0104 + 1),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    parent
        .spawn((
            EnchantUiElement0104::WaitingGroup,
            Node {
                display: Display::None,
                ..EnchantUiRect0104::default().node()
            },
            enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Waiting)),
            GlobalZIndex(ENCHANT_UI_Z_INDEX_0104 + 2),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|group| {
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::WaitingLabel,
                ENCHANT_WAITING_LABEL_RECT,
                "Enchanting....",
                LocalizedText::new("ui.enchant.waiting", "Enchanting...."),
                EnchantUiTextStyle0104::Cha12YellowMiddleLeft,
                Color::BLACK,
                assets,
            );
            group.spawn((
                EnchantUiElement0104::WaitingNpcBoundary,
                crate::service_portrait::ServicePortraitSlot::EnchantWaiting,
                ImageNode::default(),
                ENCHANT_WAITING_NPC_RECT.node(),
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            group.spawn((
                EnchantUiElement0104::WaitingProgress,
                ENCHANT_WAIT_PROGRESS_RECT.node(),
                enchant_stretched_image_0104(
                    assets.image(EnchantStaticAssetRole0104::WaitProgress),
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
    parent
        .spawn((
            EnchantUiElement0104::SuccessGroup,
            Node {
                display: Display::None,
                ..EnchantUiRect0104::default().node()
            },
            enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Success)),
            GlobalZIndex(ENCHANT_UI_Z_INDEX_0104 + 2),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|group| {
            group.spawn((
                EnchantUiElement0104::SuccessNpc,
                ENCHANT_SUCCESS_NPC_RECT_0104.node(),
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::NpcIcon)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::SuccessHooray,
                ENCHANT_SUCCESS_HOORAY_RECT_0104,
                "HOORAY!",
                LocalizedText::new("ui.enchant.success.hooray", "HOORAY!"),
                EnchantUiTextStyle0104::Jeff12SkyBlueMiddleLeft,
                Color::srgb(0.0, 1.0, 1.0),
                assets,
            );
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::SuccessMessage,
                ENCHANT_SUCCESS_MESSAGE_RECT_0104,
                "Enchant was a success! Your new item is ready and piping hot.",
                LocalizedText::new(
                    "ui.enchant.success.message",
                    "Enchant was a success! Your new item is ready and piping hot.",
                ),
                EnchantUiTextStyle0104::Cha10LightBlueMiddleLeft,
                Color::srgb(0.8, 1.0, 1.0),
                assets,
            );
            group.spawn((
                EnchantUiElement0104::SuccessIcon,
                Node {
                    display: Display::None,
                    ..ENCHANT_SUCCESS_ICON_RECT_0104.node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            group
                .spawn((
                    EnchantUiElement0104::SuccessLevelBadge,
                    ENCHANT_SUCCESS_LEVEL_BADGE_RECT_0104.node(),
                    enchant_stretched_image_0104(
                        assets.image(EnchantStaticAssetRole0104::LevelBadge),
                    ),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|badge| {
                    spawn_enchant_text_0104(
                        badge,
                        EnchantUiElement0104::SuccessLevelText,
                        EnchantUiRect0104::new(0.0, 0.0, 25.0, 12.0),
                        "",
                        enchant_level_badge_text_0104(""),
                        EnchantUiTextStyle0104::EnchantLevelMiddleCenter,
                        Color::srgb(0.0, 0.229_927, 0.332_116_78),
                        assets,
                    );
                });
            for (element, rect, style, color) in [
                (
                    EnchantUiElement0104::SuccessName,
                    ENCHANT_SUCCESS_NAME_RECT_0104,
                    EnchantUiTextStyle0104::Jeff8LightBlueUpperLeft,
                    Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                ),
                (
                    EnchantUiElement0104::SuccessLevel,
                    ENCHANT_SUCCESS_LEVEL_RECT_0104,
                    EnchantUiTextStyle0104::Jeff8LightBlueUpperLeft,
                    Color::srgb(1.0, 1.0, 0.0),
                ),
                (
                    EnchantUiElement0104::SuccessDescription,
                    ENCHANT_SUCCESS_DESCRIPTION_RECT_0104,
                    EnchantUiTextStyle0104::Cha10SkyBlueMiddleLeft,
                    Color::srgb(0.0, 1.0, 1.0),
                ),
            ] {
                spawn_enchant_text_0104(
                    group,
                    element,
                    rect,
                    "",
                    match element {
                        EnchantUiElement0104::SuccessLevel => enchant_item_level_text_0104(""),
                        _ => enchant_passthrough_text_0104(""),
                    },
                    style,
                    color,
                    assets,
                );
            }
            for (element, left) in [
                (EnchantUiElement0104::SuccessPoint, 58.0),
                (EnchantUiElement0104::SuccessGroupRating, 146.0),
                (EnchantUiElement0104::SuccessDefense, 236.0),
            ] {
                spawn_enchant_text_0104(
                    group,
                    element,
                    EnchantUiRect0104::new(left, 305.0, 59.0, 14.0),
                    "",
                    enchant_stat_value_text_0104(""),
                    EnchantUiTextStyle0104::Jeff8LightBlueMiddleCenter,
                    Color::srgb(0.8, 1.0, 1.0),
                    assets,
                );
            }
            for (element, rect, key, label) in [
                (
                    EnchantUiElement0104::SuccessTypeLabel,
                    EnchantUiRect0104::new(127.0, 348.0, 27.0, 14.0),
                    "ui.enchant.success.type",
                    "Type",
                ),
                (
                    EnchantUiElement0104::SuccessRangeLabel,
                    EnchantUiRect0104::new(120.0, 368.0, 34.0, 14.0),
                    "ui.enchant.success.range",
                    "Range",
                ),
                (
                    EnchantUiElement0104::SuccessRarityLabel,
                    EnchantUiRect0104::new(122.0, 389.0, 32.0, 14.0),
                    "ui.enchant.success.rarity",
                    "Rarity",
                ),
                (
                    EnchantUiElement0104::SuccessTradeLabel,
                    EnchantUiRect0104::new(66.0, 408.0, 88.0, 14.0),
                    "ui.enchant.success.trade_availability",
                    "Trade Availability",
                ),
            ] {
                spawn_enchant_text_0104(
                    group,
                    element,
                    rect,
                    label,
                    LocalizedText::new(key, label),
                    EnchantUiTextStyle0104::Cha10SkyBlueMiddleLeft,
                    Color::srgb(0.0, 1.0, 1.0),
                    assets,
                );
            }
            for (element, top) in [
                (EnchantUiElement0104::SuccessTypeValue, 348.0),
                (EnchantUiElement0104::SuccessRangeValue, 368.0),
                (EnchantUiElement0104::SuccessRarityValue, 388.0),
                (EnchantUiElement0104::SuccessTradeValue, 408.0),
            ] {
                spawn_enchant_text_0104(
                    group,
                    element,
                    EnchantUiRect0104::new(168.0, top, 115.0, 14.0),
                    "",
                    enchant_passthrough_text_0104(""),
                    EnchantUiTextStyle0104::Cha10LightBlueMiddleCenter,
                    Color::srgb(0.8, 1.0, 1.0),
                    assets,
                );
            }
            spawn_enchant_button_0104(
                group,
                EnchantUiElement0104::EnchantMoreItems,
                EnchantInteractiveControl0104::EnchantMoreItems,
                ENCHANT_SUCCESS_MORE_RECT,
                "ui.enchant.success.more_items",
                "ENCHANT MORE ITEMS",
                EnchantUiTextStyle0104::EnchantDefaultButtonMiddleCenter,
                assets,
            );
            spawn_enchant_button_0104(
                group,
                EnchantUiElement0104::GoToMyStuff,
                EnchantInteractiveControl0104::GoToMyStuff,
                ENCHANT_SUCCESS_STUFF_RECT,
                "ui.enchant.success.go_to_my_stuff",
                "GO TO MY STUFF",
                EnchantUiTextStyle0104::EnchantDefaultButtonMiddleCenter,
                assets,
            );
        });
}

pub(super) fn spawn_enchant_button_0104(
    parent: &mut ChildSpawnerCommands,
    element: EnchantUiElement0104,
    control: EnchantInteractiveControl0104,
    rect: EnchantUiRect0104,
    localization_key: &'static str,
    label: &'static str,
    style: EnchantUiTextStyle0104,
    assets: &EnchantUiAssets0104,
) {
    let mut button_node = style.node(rect);
    button_node.overflow = Overflow::clip();
    parent
        .spawn((
            Button,
            element,
            control,
            button_node,
            enchant_sliced_image_0104(
                assets.image(EnchantStaticAssetRole0104::ButtonNormal),
                ENCHANT_BUTTON_BORDER_0104,
            ),
        ))
        .with_children(|button| {
            let label_node = Node {
                max_width: percent(100),
                flex_shrink: 0.0,
                ..default()
            };
            button.spawn((
                label_node,
                EnchantButtonLabel0104,
                Text::new(label),
                LocalizedText::new(localization_key, label),
                style.font(assets),
                TextColor(Color::WHITE),
                style.layout(),
                style,
                UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}

pub(super) fn spawn_enchant_text_0104(
    parent: &mut ChildSpawnerCommands,
    element: EnchantUiElement0104,
    rect: EnchantUiRect0104,
    value: &'static str,
    localized: LocalizedText,
    style: EnchantUiTextStyle0104,
    color: Color,
    assets: &EnchantUiAssets0104,
) {
    let mut node = style.node(rect);
    if matches!(element, EnchantUiElement0104::TarosDigit(_)) {
        node.justify_content = JustifyContent::Center;
    }
    parent
        .spawn((node, Pickable::IGNORE, bevy::ui::FocusPolicy::Pass))
        .with_children(|container| {
            container.spawn((
                element,
                Node {
                    max_width: percent(100),
                    flex_shrink: 0.0,
                    ..default()
                },
                Text::new(value),
                localized,
                style.font(assets),
                TextColor(color),
                style.layout(),
                style,
                UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}
