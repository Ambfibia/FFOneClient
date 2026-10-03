use super::*;

pub(super) fn spawn_combi_overlays(parent: &mut ChildSpawnerCommands, assets: &CombiUiAssets) {
    parent.spawn((
        CombiUiElement0104::Shade,
        Node {
            display: Display::None,
            ..CombiUiRect::default().node()
        },
        ImageNode {
            color: Color::srgba(1.0, 1.0, 1.0, 0.75),
            ..stretched_image(assets.image(CombiStaticAssetRole::BlackShade))
        },
        // Blocks the main panel while the wait or success overlay is up.
        Pickable::IGNORE,
        FocusPolicy::Block,
        GlobalZIndex(COMBI_UI_Z_INDEX + 1),
    ));
    parent
        .spawn((
            CombiUiElement0104::SuccessGroup,
            Node {
                display: Display::None,
                ..CombiUiRect::default().node()
            },
            stretched_image(assets.image(CombiStaticAssetRole::Success)),
            GlobalZIndex(COMBI_UI_Z_INDEX + 2),
            combi_passive(),
        ))
        .with_children(|group| {
            group.spawn((
                CombiUiElement0104::SuccessNpcIcon,
                COMBI_SUCCESS_NPC_ICON_RECT.node(),
                stretched_image(assets.image(CombiStaticAssetRole::NpcIcon)),
                combi_passive(),
            ));
            spawn_combi_text(
                group,
                CombiUiElement0104::SuccessHooray,
                COMBI_SUCCESS_HOORAY_RECT,
                LocalizedText::new("ui.combi.success.hooray", "HOORAY!"),
                assets.jeffe_font.clone(),
                12.0,
                Color::srgb(0.0, 1.0, 1.0),
                Justify::Left,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::SuccessMessage,
                COMBI_SUCCESS_MESSAGE_RECT,
                LocalizedText::new(
                    "ui.combi.success.message",
                    "The combination was a success! Your new item is ready and piping hot.",
                ),
                assets.chalet_font.clone(),
                10.0,
                Color::srgb(0.8, 1.0, 1.0),
                Justify::Left,
                LineBreak::WordBoundary,
            );
            group.spawn((
                CombiUiElement0104::SuccessRestricted,
                Node {
                    display: Display::None,
                    ..COMBI_SUCCESS_ICON_RECT.node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::Restricted)),
                combi_passive(),
            ));
            group.spawn((
                CombiUiElement0104::SuccessIcon,
                Node {
                    display: Display::None,
                    ..COMBI_SUCCESS_ICON_RECT.node()
                },
                ImageNode::default(),
                combi_passive(),
            ));
            group.spawn((
                CombiUiElement0104::SuccessBadge,
                COMBI_SUCCESS_BADGE_RECT.node(),
                stretched_image(assets.image(CombiStaticAssetRole::Combined)),
                combi_passive(),
            ));
            spawn_combi_text(
                group,
                CombiUiElement0104::SuccessName,
                COMBI_SUCCESS_NAME_RECT,
                combi_passthrough_text(""),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Left,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::SuccessLevel,
                COMBI_SUCCESS_LEVEL_RECT,
                combi_item_level_text(0),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Left,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::SuccessDescription,
                COMBI_SUCCESS_DESCRIPTION_RECT,
                combi_passthrough_text(""),
                assets.chalet_font.clone(),
                10.0,
                Color::srgb(0.0, 1.0, 1.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );
            for (element, left) in [
                (CombiUiElement0104::SuccessSingle, 58.0),
                (CombiUiElement0104::SuccessMulti, 146.0),
                (CombiUiElement0104::SuccessDefense, 236.0),
            ] {
                spawn_combi_text(
                    group,
                    element,
                    CombiUiRect::new(left, 305.0, 59.0, 14.0),
                    combi_stat_value_text(0),
                    assets.jeffe_font.clone(),
                    8.0,
                    COMBI_COLOR_DEFAULT.bevy(),
                    Justify::Center,
                    LineBreak::NoWrap,
                );
            }
            for (element, rect, key, label) in [
                (
                    CombiUiElement0104::SuccessTypeLabel,
                    CombiUiRect::new(127.0, 348.0, 27.0, 14.0),
                    "ui.combi.info.type",
                    "Type",
                ),
                (
                    CombiUiElement0104::SuccessRangeLabel,
                    CombiUiRect::new(120.0, 368.0, 34.0, 14.0),
                    "ui.combi.info.range",
                    "Range",
                ),
                (
                    CombiUiElement0104::SuccessRarityLabel,
                    CombiUiRect::new(122.0, 389.0, 32.0, 14.0),
                    "ui.combi.info.rarity",
                    "Rarity",
                ),
                (
                    CombiUiElement0104::SuccessTradeLabel,
                    CombiUiRect::new(66.0, 408.0, 88.0, 14.0),
                    "ui.combi.info.trade_availability",
                    "Trade Availability",
                ),
            ] {
                spawn_combi_text(
                    group,
                    element,
                    rect,
                    LocalizedText::new(key, label),
                    assets.chalet_font.clone(),
                    10.0,
                    Color::srgb(0.0, 1.0, 1.0),
                    Justify::Center,
                    LineBreak::NoWrap,
                );
            }
            for (element, top) in [
                (CombiUiElement0104::SuccessTypeValue, 348.0),
                (CombiUiElement0104::SuccessRangeValue, 368.0),
                (CombiUiElement0104::SuccessRarityValue, 388.0),
                (CombiUiElement0104::SuccessTradeValue, 408.0),
            ] {
                spawn_combi_text(
                    group,
                    element,
                    CombiUiRect::new(168.0, top, 115.0, 14.0),
                    combi_passthrough_text(""),
                    assets.chalet_font.clone(),
                    10.0,
                    Color::srgb(0.8, 1.0, 1.0),
                    Justify::Center,
                    LineBreak::NoWrap,
                );
            }
            spawn_combi_button(
                group,
                CombiUiElement0104::CombineMore,
                CombiInteractiveControl0104::CombineMore,
                COMBI_SUCCESS_COMBINE_MORE_RECT,
                LocalizedText::new("ui.combi.success.combine_more", "COMBINE MORE ITEMS"),
                assets,
            );
            spawn_combi_button(
                group,
                CombiUiElement0104::GoToStuff,
                CombiInteractiveControl0104::GoToStuff,
                COMBI_SUCCESS_GO_TO_STUFF_RECT,
                LocalizedText::new("ui.combi.success.go_to_my_stuff", "GO TO MY STUFF"),
                assets,
            );
        });
    parent
        .spawn((
            CombiUiElement0104::WaitingGroup,
            Node {
                display: Display::None,
                ..CombiUiRect::default().node()
            },
            stretched_image(assets.image(CombiStaticAssetRole::Waiting)),
            GlobalZIndex(COMBI_UI_Z_INDEX + 2),
            combi_passive(),
        ))
        .with_children(|group| {
            group.spawn((
                CombiUiElement0104::WaitingNpcBoundary,
                crate::service_portrait::ServicePortraitSlot::CombiWaiting,
                ImageNode::default(),
                COMBI_WAITING_NPC_PREVIEW_RECT.node(),
                BackgroundColor(Color::NONE),
                combi_passive(),
            ));
        });
}

pub(super) fn spawn_combi_button(
    parent: &mut ChildSpawnerCommands,
    marker: CombiUiElement0104,
    control: CombiInteractiveControl0104,
    rect: CombiUiRect,
    localized: LocalizedText,
    assets: &CombiUiAssets,
) {
    let label = combi_fallback_text(&localized);
    parent
        .spawn((
            Button,
            marker,
            control,
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..rect.node()
            },
            sliced_image(
                assets.image(CombiStaticAssetRole::ButtonNormal),
                COMBI_BUTTON_BORDER,
            ),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                localized,
                TextFont {
                    font: (assets.jeffe_font.clone()).into(),
                    font_size: (8.0).into(),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                combi_passive(),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_combi_text(
    parent: &mut ChildSpawnerCommands,
    marker: CombiUiElement0104,
    rect: CombiUiRect,
    localized: LocalizedText,
    font: Handle<Font>,
    font_size: f32,
    color: Color,
    justify: Justify,
    linebreak: LineBreak,
) {
    let value = combi_fallback_text(&localized);
    parent.spawn((
        marker,
        Node {
            align_items: AlignItems::Center,
            ..rect.node()
        },
        Text::new(value),
        localized,
        TextFont {
            font: (font).into(),
            font_size: (font_size).into(),
            ..default()
        },
        TextColor(color),
        TextLayout::new(justify, linebreak),
        combi_passive(),
    ));
}
