//! Chat window spawning and its marker components.

use super::assets::GameplayUiAssets;
use super::chat_log::chat_line_color;
use super::chat_model::{
    CHAT_CHALET_SMALL_Y_OFFSET, CHAT_DEFAULT_HEIGHT, CHAT_DEFAULT_WIDTH,
    CHAT_EMPTY_STATE_TEXT_COLOR, CHAT_HISTORY_CAPACITY, CHAT_INACTIVE_TEXT_COLOR,
    CHAT_INPUT_PADDING, CHAT_JEFFE_14_VERTICAL_SCALE, CHAT_LOG_BOTTOM_PADDING, CHAT_LOG_LINE_GAP,
    CHAT_SCROLLBAR_ARROW_HEIGHT, CHAT_SCROLLBAR_THUMB_MIN_HEIGHT, CHAT_SCROLLBAR_THUMB_WIDTH,
    CHAT_SCROLLBAR_WIDTH, CHAT_TAB_NORMAL_TEXT_COLOR, CHAT_TAB_SELECTED_TEXT_COLOR, ChatChannel,
    ChatLineKind, ChatLineUi, chat_layout,
};
use super::quick_chat::{EmoteButton, MenuChatButton};
use super::text::{chat_chalet_small_font, chat_jeffe_14_font};
use crate::text_edit::{self, EditVisual};
use crate::{localization::LocalizedText, option_ui::TextColorSettings};
use bevy::{
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    ui::widget::NodeImageMode,
};

#[derive(Component)]
pub(super) struct ChatRoot;
#[derive(Component)]
pub(super) struct ChatBackground;
#[derive(Component)]
pub(super) struct ChatEntry;
#[derive(Component)]
pub(super) struct ChatTextFieldButton;
#[derive(Component)]
pub(super) struct ChatInputText;
#[derive(Component)]
pub(super) struct ChatResizeHandle;
#[derive(Component)]
pub(super) struct ChatLineText(pub(super) usize);
#[derive(Component)]
pub(super) struct ChatLogViewport;
#[derive(Component)]
pub(super) struct ChatEmptyState(pub(super) ChatChannel);
#[derive(Component)]
pub(super) struct ChatScrollbar;
#[derive(Clone, Copy, PartialEq, Eq, Component)]
pub(super) enum ChatScrollbarPart {
    Track,
    Up,
    Down,
    Thumb,
}
#[derive(Component)]
#[require(crate::ui::shared::controller::ControllerUiIgnore)]
pub(super) struct ChatTabButton(pub(super) ChatChannel);
#[derive(Component)]
pub(super) struct ChatTabImage(pub(super) ChatChannel);
#[derive(Component)]
pub(super) struct ChatTabLabel(pub(super) ChatChannel);
#[derive(Component)]
pub(super) struct ChatTabAlert(pub(super) ChatChannel);

#[derive(Component)]
pub(super) struct SendChatButton;
#[derive(Component)]
pub(super) struct SendChatLabel;

#[derive(Default)]
pub(super) struct RenderedChatHistory {
    pub(super) initialized: bool,
    pub(super) selected: ChatChannel,
    pub(super) text_colors: TextColorSettings,
    pub(super) lines: Vec<ChatLineUi>,
}

pub(super) fn spawn_chat(parent: &mut ChildSpawnerCommands, assets: &GameplayUiAssets) {
    let layout = chat_layout(Vec2::new(CHAT_DEFAULT_WIDTH, CHAT_DEFAULT_HEIGHT), false);
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                width: px(layout.size.x),
                height: px(layout.size.y),
                overflow: Overflow::clip(),
                ..default()
            },
            UiTransform::default(),
            ChatRoot,
            ZIndex(9),
        ))
        .with_children(|chat| {
            chat.spawn((
                layout.background.node(),
                ImageNode {
                    image: assets.chat_background.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::axes(8.0, 4.0),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 1.0,
                    }),
                    ..default()
                },
                ChatBackground,
            ));
            for (channel, key, label) in [
                (ChatChannel::All, "ui.hud.chat.channel.all", "ALL"),
                (ChatChannel::Group, "ui.hud.chat.channel.group", "GROUP"),
                (ChatChannel::Buddy, "ui.hud.chat.channel.buddy", "BUDDY"),
            ] {
                let rect = layout.tabs[channel.index()];
                chat.spawn((
                    Button,
                    rect.node(),
                    ImageNode {
                        image: if channel == ChatChannel::All {
                            assets.chat_tab_selected.clone()
                        } else {
                            assets.chat_tab_normal.clone()
                        },
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    ChatTabButton(channel),
                    ChatTabImage(channel),
                ))
                .with_children(|tab| {
                    tab.spawn((
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        Text::new(label),
                        LocalizedText::new(key, label),
                        chat_jeffe_14_font(&assets.jeffe_font),
                        UiTransform::from_scale(Vec2::new(1.0, CHAT_JEFFE_14_VERTICAL_SCALE)),
                        TextColor(if channel == ChatChannel::All {
                            CHAT_TAB_SELECTED_TEXT_COLOR
                        } else {
                            CHAT_TAB_NORMAL_TEXT_COLOR
                        }),
                        TextLayout::default().with_justify(Justify::Center),
                        ChatTabLabel(channel),
                    ));
                    tab.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            display: Display::None,
                            left: px(rect.width - 35.0),
                            top: px(0),
                            width: px(14),
                            height: px(14),
                            ..default()
                        },
                        ImageNode::new(assets.chat_alert.clone()),
                        Pickable::IGNORE,
                        ChatTabAlert(channel),
                    ));
                });
            }
            chat.spawn((
                Button,
                Node {
                    display: Display::None,
                    ..layout.resize.node()
                },
                ImageNode::new(assets.chat_resize_normal.clone()),
                ZIndex(2),
                ChatResizeHandle,
            ));
            chat.spawn((
                layout.entry.node(),
                ImageNode {
                    image: assets.chat_entry.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect { min_inset: Vec2::new(0.0, 0.0), max_inset: Vec2::new(20.0, 0.0) },
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 1.0,
                    }),
                    ..default()
                },
                ChatEntry,
            ));
            chat.spawn((
                Button,
                layout.menu.node(),
                ImageNode::new(assets.menu_chat.clone()),
                MenuChatButton,
            ));
            chat.spawn((
                Button,
                Node {
                    display: Display::None,
                    ..layout.emote.node()
                },
                ImageNode::new(assets.emote.clone()),
                EmoteButton,
            ));
            chat.spawn((
                Button,
                Node {
                    display: Display::None,
                    ..layout.send.node()
                },
                ImageNode {
                    image: assets.blue_button.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::axes(6.0, 4.0),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 1.0,
                    }),
                    ..default()
                },
                SendChatButton,
            ))
            .with_child((
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Text::new("SEND"),
                LocalizedText::new("ui.hud.chat.send", "SEND"),
                chat_jeffe_14_font(&assets.jeffe_font),
                UiTransform::from_scale(Vec2::new(1.0, CHAT_JEFFE_14_VERTICAL_SCALE)),
                TextColor(Color::WHITE),
                TextLayout::default().with_justify(Justify::Center),
                SendChatLabel,
            ));
            // Fresh Retrobution draws SEND first, then lets the active
            // `(75..233)` field cover five pixels of the `(228..297)` button.
            // Keep this sibling order; otherwise the visible seam moves left.
            chat.spawn((
                Button,
                Node {
                    overflow: Overflow::clip(),
                    ..layout.input.node()
                },
                ImageNode {
                    image: assets.chat_inactive_text_field.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::axes(2.0, 2.0),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 1.0,
                    }),
                    ..default()
                },
                ChatTextFieldButton,
            ))
            .with_children(|field| {
                field.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: px(23),
                    padding: UiRect::all(px(CHAT_INPUT_PADDING)),
                    ..default()
                },
                Text::new("Press ENTER to access chat and menus."),
                LocalizedText::new(
                    "ui.hud.chat.open_hint",
                    "Press ENTER to access chat and menus.",
                ),
                chat_chalet_small_font(&assets.chalet_font),
                UiTransform::from_translation(Val2::px(0.0, CHAT_CHALET_SMALL_Y_OFFSET)),
                TextColor(CHAT_INACTIVE_TEXT_COLOR),
                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                ChatInputText,
                EditVisual { inset: CHAT_INPUT_PADDING, ..default() },
                )).with_children(text_edit::spawn_decorations);
            });
            chat.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Start,
                    overflow: Overflow::scroll_y(),
                    scrollbar_width: 0.0,
                    row_gap: px(CHAT_LOG_LINE_GAP),
                    padding: UiRect::bottom(px(CHAT_LOG_BOTTOM_PADDING)),
                    ..layout.log.node()
                },
                ScrollPosition::default(),
                Interaction::None,
                ChatLogViewport,
            ))
            .with_children(|log| {
                for index in 0..CHAT_HISTORY_CAPACITY {
                    log.spawn((
                        Node {
                            display: Display::None,
                            width: px(layout.log.width - 16.0),
                            min_width: px(layout.log.width - 16.0),
                            max_width: px(layout.log.width - 16.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        Text::new(""),
                        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                        chat_chalet_small_font(&assets.chalet_font),
                        UiTransform::from_translation(Val2::px(0.0, CHAT_CHALET_SMALL_Y_OFFSET)),
                        TextColor(chat_line_color(
                            ChatLineKind::Normal,
                            TextColorSettings::default(),
                        )),
                        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                        // Decorative text must let the log viewport receive
                        // hover/wheel input over the glyphs as well as gaps.
                        bevy::ui::FocusPolicy::Pass,
                        Pickable::IGNORE,
                        ChatLineText(index),
                    ));
                }
            });
            chat.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    left: px(layout.log.x - 21.0),
                    top: px(layout.log.y),
                    width: px(CHAT_SCROLLBAR_WIDTH),
                    height: px(layout.log.height),
                    ..default()
                },
                Pickable::IGNORE,
                ChatScrollbar,
                bevy::ui::RelativeCursorPosition::default(),
                ZIndex(2),
            ))
            .with_children(|scrollbar| {
                for (part, image, top, width, height, sliced) in [
                    (
                        ChatScrollbarPart::Track,
                        assets.chat_scroll_track.clone(),
                        CHAT_SCROLLBAR_ARROW_HEIGHT,
                        CHAT_SCROLLBAR_WIDTH,
                        layout.log.height - CHAT_SCROLLBAR_ARROW_HEIGHT * 2.0,
                        true,
                    ),
                    (
                        ChatScrollbarPart::Up,
                        assets.chat_scroll_up.clone(),
                        0.0,
                        CHAT_SCROLLBAR_WIDTH,
                        CHAT_SCROLLBAR_ARROW_HEIGHT,
                        false,
                    ),
                    (
                        ChatScrollbarPart::Down,
                        assets.chat_scroll_down.clone(),
                        layout.log.height - CHAT_SCROLLBAR_ARROW_HEIGHT,
                        CHAT_SCROLLBAR_WIDTH,
                        CHAT_SCROLLBAR_ARROW_HEIGHT,
                        false,
                    ),
                    (
                        ChatScrollbarPart::Thumb,
                        assets.chat_scroll_thumb.clone(),
                        CHAT_SCROLLBAR_ARROW_HEIGHT,
                        CHAT_SCROLLBAR_THUMB_WIDTH,
                        CHAT_SCROLLBAR_THUMB_MIN_HEIGHT,
                        true,
                    ),
                ] {
                    scrollbar.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(top),
                            width: px(width),
                            height: px(height),
                            ..default()
                        },
                        ImageNode {
                            image,
                            image_mode: if sliced {
                                NodeImageMode::Sliced(TextureSlicer {
                                    border: BorderRect { min_inset: Vec2::new(2.0, 4.0), max_inset: Vec2::new(2.0, 4.0) },
                                    center_scale_mode: SliceScaleMode::Stretch,
                                    sides_scale_mode: SliceScaleMode::Stretch,
                                    max_corner_scale: 1.0,
                                })
                            } else {
                                NodeImageMode::Stretch
                            },
                            ..default()
                        },
                        Interaction::default(),
                        part,
                    ));
                }
            });
            for (channel, key, fallback, icon) in [
                (
                    ChatChannel::Buddy,
                    "ui.hud.chat.empty.buddy",
                    "Click on a player and select MAKE A BUDDY from the menu, or select ADD in your BUDDY LIST.",
                    assets.chat_buddy_icon.clone(),
                ),
                (
                    ChatChannel::Group,
                    "ui.hud.chat.empty.group",
                    "Click on a player and select INVITE TO GROUP from the menu.",
                    assets.chat_group_icon.clone(),
                ),
            ] {
                chat.spawn((
                    Node {
                        display: Display::None,
                        ..layout.empty_state.node()
                    },
                    ImageNode {
                        image: assets.chat_empty_state_background.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    ChatEmptyState(channel),
                ))
                .with_children(|empty| {
                    empty.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(50),
                            top: px(7),
                            right: px(5),
                            bottom: px(0),
                            ..default()
                        },
                        Text::new(fallback),
                        LocalizedText::new(key, fallback),
                        chat_chalet_small_font(&assets.chalet_font),
                        UiTransform::from_translation(Val2::px(
                            0.0,
                            CHAT_CHALET_SMALL_Y_OFFSET,
                        )),
                        TextColor(CHAT_EMPTY_STATE_TEXT_COLOR),
                        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                        Pickable::IGNORE,
                    ));
                    empty.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(10),
                            top: px(15),
                            width: px(29),
                            height: px(29),
                            ..default()
                        },
                        ImageNode::new(icon),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}
