//! Shared HUD text spawn helpers and font selection.

use super::chat_model::{
    CHAT_CHALET_SMALL_LINE_HEIGHT, CHAT_INPUT_FONT_SIZE, CHAT_JEFFE_14_FONT_SIZE,
    CHAT_JEFFE_14_LINE_HEIGHT,
};
use super::hud::GameplayUiRect;
use super::minimap::MinimapNameShadow;
use crate::localization::LocalizedText;
use bevy::{prelude::*, text::LineHeight};

pub(super) fn spawn_shadowed_center_text<M: Component>(
    parent: &mut ChildSpawnerCommands,
    rect: GameplayUiRect,
    text: &str,
    font: &Handle<Font>,
    font_size: f32,
    color: Color,
    marker: M,
) {
    // FusionFallHUDSkin.centerbox2: UpperCenter, ChaletBook-Regular Small,
    // padding L10/R6/T4/B6. Retrobution draws the same label twice with a
    // one-pixel black offset before the yellow foreground.
    let style_node = || Node {
        padding: UiRect {
            left: px(10),
            right: px(6),
            top: px(4),
            bottom: px(6),
        },
        ..rect.node()
    };
    parent.spawn((
        style_node(),
        Text::new(text),
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", text),
        hud_font(font, font_size),
        TextColor(Color::BLACK),
        TextLayout::default().with_justify(Justify::Center),
        MinimapNameShadow,
        UiTransform::from_translation(Val2::px(1.0, 1.0)),
        ZIndex(7),
    ));
    parent.spawn((
        style_node(),
        Text::new(text),
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", text),
        hud_font(font, font_size),
        TextColor(color),
        TextLayout::default().with_justify(Justify::Center),
        ZIndex(7),
        marker,
    ));
}

pub(super) fn spawn_middle_left_text<M: Component>(
    parent: &mut ChildSpawnerCommands,
    rect: GameplayUiRect,
    text: &str,
    font: &Handle<Font>,
    marker: M,
) {
    // FusionFallHUDSkin.label uses ChaletBook-Regular Small with
    // TextAnchor.MiddleLeft (alignment 3), zero padding and a 0.9 gray tint.
    parent
        .spawn((Node {
            align_items: AlignItems::Center,
            ..rect.node()
        },))
        .with_child((
            Text::new(text),
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", text),
            hud_font(font, 12.0),
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextLayout::default().with_justify(Justify::Left),
            marker,
        ));
}

pub(super) fn spawn_localized_middle_left_text<M: Component>(
    parent: &mut ChildSpawnerCommands,
    rect: GameplayUiRect,
    font: &Handle<Font>,
    marker: M,
) {
    parent
        .spawn((Node {
            align_items: AlignItems::Center,
            ..rect.node()
        },))
        .with_child((
            Text::new(""),
            hud_font(font, 12.0),
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextLayout::default().with_justify(Justify::Left),
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
            marker,
        ));
}

pub(super) fn hud_font(font: &Handle<Font>, font_size: f32) -> TextFont {
    TextFont {
        font: (font.clone()).into(),
        font_size: (font_size).into(),
        ..default()
    }
}

pub(super) fn chat_chalet_small_font(font: &Handle<Font>) -> (TextFont, LineHeight) {
    (
        TextFont {
            font: (font.clone()).into(),
            font_size: (CHAT_INPUT_FONT_SIZE).into(),
            ..default()
        },
        LineHeight::Px(CHAT_CHALET_SMALL_LINE_HEIGHT),
    )
}

pub(super) fn chat_jeffe_14_font(font: &Handle<Font>) -> (TextFont, LineHeight) {
    (
        TextFont {
            font: (font.clone()).into(),
            font_size: (CHAT_JEFFE_14_FONT_SIZE).into(),
            ..default()
        },
        LineHeight::Px(CHAT_JEFFE_14_LINE_HEIGHT),
    )
}
