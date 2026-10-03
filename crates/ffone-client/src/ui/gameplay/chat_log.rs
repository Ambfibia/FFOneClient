//! Chat log binding, colors and history scrolling.

use super::assets::GameplayUiAssets;
use super::chat_input::localized_chat_input;
use super::chat_model::{
    CHAT_ATTACK_TEXT_COLOR, CHAT_DAMAGE_TEXT_COLOR, CHAT_HISTORY_CAPACITY,
    CHAT_INACTIVE_TEXT_COLOR, CHAT_INPUT_PADDING, CHAT_NPC_TEXT_COLOR, CHAT_RECEIVE_TEXT_COLOR,
    CHAT_SCROLL_TO_LATEST, CHAT_SCROLL_WHEEL_LINE, CHAT_TAB_NORMAL_TEXT_COLOR,
    CHAT_TAB_SELECTED_TEXT_COLOR, ChatChannel, ChatLineKind, ChatLineUi, chat_layout,
};
use super::chat_spawn::{
    ChatBackground, ChatEmptyState, ChatEntry, ChatInputText, ChatLineText, ChatLogViewport,
    ChatResizeHandle, ChatRoot, ChatScrollbarPart, ChatTabAlert, ChatTabImage, ChatTabLabel,
    ChatTextFieldButton, RenderedChatHistory, SendChatButton,
};
use super::model::{GameplayMenuTransition, GameplayUiModel};
use super::quick_chat::{EmoteButton, MenuChatButton};
use crate::{
    localization::LocalizedText,
    option_ui::{OPTION_CHAT_PALETTE_RGB, TextColorSettings},
};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

pub(super) fn bind_chat(
    model: Res<GameplayUiModel>,
    transition: Res<GameplayMenuTransition>,
    assets: Res<GameplayUiAssets>,
    mut chat_root: Single<&mut Node, With<ChatRoot>>,
    mut chat_input: Single<
        (&mut Node, &mut LocalizedText, &mut TextColor),
        (
            With<ChatInputText>,
            Without<ChatRoot>,
            Without<ChatLineText>,
        ),
    >,
    mut chat_nodes: Query<
        (
            &mut Node,
            &mut ImageNode,
            Has<ChatBackground>,
            Has<ChatEntry>,
            Has<ChatTextFieldButton>,
            Has<MenuChatButton>,
            Has<EmoteButton>,
            Has<SendChatButton>,
            Has<ChatResizeHandle>,
        ),
        (
            Without<ChatRoot>,
            Without<ChatInputText>,
            Without<ChatLineText>,
            Without<ChatTabImage>,
            Without<ChatTabAlert>,
            Without<ChatLogViewport>,
            Without<ChatEmptyState>,
        ),
    >,
    mut chat_lines: Query<
        (&ChatLineText, &mut Node, &mut LocalizedText, &mut TextColor),
        (
            Without<ChatInputText>,
            Without<ChatRoot>,
            Without<ChatLogViewport>,
        ),
    >,
    mut chat_scroll: Single<
        (&mut Node, &mut ScrollPosition, &ComputedNode),
        (
            With<ChatLogViewport>,
            Without<ChatRoot>,
            Without<ChatInputText>,
            Without<ChatTabImage>,
            Without<ChatTabAlert>,
        ),
    >,
    mut chat_tabs: Query<
        (&ChatTabImage, &mut Node),
        (
            Without<ChatRoot>,
            Without<ChatInputText>,
            Without<ChatLineText>,
        ),
    >,
    mut chat_alerts: Query<
        (&ChatTabAlert, &mut Node),
        (
            Without<ChatTabImage>,
            Without<ChatRoot>,
            Without<ChatInputText>,
            Without<ChatLineText>,
        ),
    >,
    mut chat_tab_labels: Query<
        (&ChatTabLabel, &mut TextColor),
        (Without<ChatInputText>, Without<ChatLineText>),
    >,
    mut empty_states: Query<
        (&ChatEmptyState, &mut Node),
        (
            Without<ChatRoot>,
            Without<ChatInputText>,
            Without<ChatLineText>,
            Without<ChatLogViewport>,
            Without<ChatTabImage>,
            Without<ChatTabAlert>,
        ),
    >,
    mut rendered_history: Local<RenderedChatHistory>,
) {
    let visible_lines = retained_chat_lines(&model.chat.lines);
    let history_changed = !rendered_history.initialized
        || rendered_history.selected != model.chat.selected
        || rendered_history.text_colors != model.chat.text_colors
        || rendered_history.lines.as_slice() != visible_lines;
    if !model.is_changed() && !transition.is_changed() && !history_changed {
        return;
    }
    let controls_visible = model.chat.active || transition.visible_or_transitioning();
    let layout = chat_layout(model.chat.window_size, controls_visible);
    chat_root
        .reborrow()
        .map_unchanged(|value| &mut value.display)
        .set_if_neq(if model.chat.visible {
            Display::Flex
        } else {
            Display::None
        });
    chat_root
        .reborrow()
        .map_unchanged(|value| &mut value.width)
        .set_if_neq(px(layout.size.x));
    chat_root
        .reborrow()
        .map_unchanged(|value| &mut value.height)
        .set_if_neq(px(layout.size.y));

    for (mut node, mut image, background, entry, input, menu, emote, send, resize) in
        &mut chat_nodes
    {
        let rect = if background {
            layout.background
        } else if entry {
            layout.entry
        } else if input {
            layout.input
        } else if menu {
            layout.menu
        } else if emote {
            layout.emote
        } else if send {
            layout.send
        } else if resize {
            layout.resize
        } else {
            continue;
        };
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(rect.x));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(rect.y));
        node.reborrow()
            .map_unchanged(|value| &mut value.width)
            .set_if_neq(px(rect.width));
        node.reborrow()
            .map_unchanged(|value| &mut value.height)
            .set_if_neq(px(rect.height));
        if input {
            image
                .reborrow()
                .map_unchanged(|value| &mut value.image)
                .set_if_neq(if controls_visible {
                    assets.chat_active_text_field.clone()
                } else {
                    assets.chat_inactive_text_field.clone()
                });
        }
        node.reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(if (emote || send || resize) && !controls_visible {
                Display::None
            } else {
                Display::Flex
            });
    }

    let localized_chat_input = localized_chat_input(&model.chat, &transition);
    let (chat_input_node, chat_input_text, chat_input_color) = &mut *chat_input;
    chat_input_node
        .reborrow()
        .map_unchanged(|value| &mut value.padding)
        .set_if_neq(if controls_visible {
            UiRect::all(px(CHAT_INPUT_PADDING))
        } else {
            UiRect {
                left: px(10),
                ..default()
            }
        });
    if **chat_input_text != localized_chat_input {
        **chat_input_text = localized_chat_input;
    }
    chat_input_color
        .reborrow()
        .map_unchanged(|value| &mut value.0)
        .set_if_neq(if controls_visible {
            Color::WHITE
        } else {
            CHAT_INACTIVE_TEXT_COLOR
        });

    let (chat_scroll_node, chat_scroll_position, chat_scroll_computed) = &mut *chat_scroll;
    let empty_state_visible = match model.chat.selected {
        ChatChannel::Buddy => model.chat.buddy_count == 0,
        ChatChannel::Group => !model.chat.group_available,
        ChatChannel::All => false,
    };
    chat_scroll_node
        .reborrow()
        .map_unchanged(|value| &mut value.left)
        .set_if_neq(px(layout.log.x));
    chat_scroll_node
        .reborrow()
        .map_unchanged(|value| &mut value.top)
        .set_if_neq(px(layout.log.y));
    chat_scroll_node
        .reborrow()
        .map_unchanged(|value| &mut value.width)
        .set_if_neq(px(layout.log.width));
    chat_scroll_node
        .reborrow()
        .map_unchanged(|value| &mut value.height)
        .set_if_neq(px(layout.log.height));
    chat_scroll_node
        .reborrow()
        .map_unchanged(|value| &mut value.display)
        .set_if_neq(if empty_state_visible {
            Display::None
        } else {
            Display::Flex
        });
    for (marker, mut node) in &mut empty_states {
        let visible = empty_state_visible && marker.0 == model.chat.selected;
        node.reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(if visible {
                Display::Flex
            } else {
                Display::None
            });
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(layout.empty_state.x));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(layout.empty_state.y));
        node.reborrow()
            .map_unchanged(|value| &mut value.width)
            .set_if_neq(px(layout.empty_state.width));
        node.reborrow()
            .map_unchanged(|value| &mut value.height)
            .set_if_neq(px(layout.empty_state.height));
    }
    for (marker, mut node) in &mut chat_tabs {
        let rect = layout.tabs[marker.0.index()];
        node.reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(Display::Flex);
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(rect.x));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(rect.y));
        node.reborrow()
            .map_unchanged(|value| &mut value.width)
            .set_if_neq(px(rect.width));
        node.reborrow()
            .map_unchanged(|value| &mut value.height)
            .set_if_neq(px(rect.height));
    }
    for (marker, mut node) in &mut chat_alerts {
        node.reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(
                if marker.0 != model.chat.selected && model.chat.alerts[marker.0.index()] {
                    Display::Flex
                } else {
                    Display::None
                },
            );
    }
    for (marker, mut color) in &mut chat_tab_labels {
        color
            .reborrow()
            .map_unchanged(|value| &mut value.0)
            .set_if_neq(if marker.0 == model.chat.selected {
                CHAT_TAB_SELECTED_TEXT_COLOR
            } else {
                CHAT_TAB_NORMAL_TEXT_COLOR
            });
    }

    for (marker, mut node, mut text, mut color) in &mut chat_lines {
        node.reborrow()
            .map_unchanged(|value| &mut value.width)
            .set_if_neq(px(layout.log.width - 16.0));
        node.reborrow()
            .map_unchanged(|value| &mut value.min_width)
            .set_if_neq(px(layout.log.width - 16.0));
        node.reborrow()
            .map_unchanged(|value| &mut value.max_width)
            .set_if_neq(px(layout.log.width - 16.0));
        if history_changed {
            if let Some(line) = visible_lines.get(marker.0) {
                node.reborrow()
                    .map_unchanged(|value| &mut value.display)
                    .set_if_neq(Display::Flex);
                *text = line.localized.clone().unwrap_or_else(|| {
                    LocalizedText::new("ui.content.passthrough", "{text}")
                        .with_arg("text", line.text.clone())
                });
                color
                    .reborrow()
                    .map_unchanged(|value| &mut value.0)
                    .set_if_neq(chat_line_color(line.kind, model.chat.text_colors));
            } else {
                node.reborrow()
                    .map_unchanged(|value| &mut value.display)
                    .set_if_neq(Display::None);
                *text = LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "");
            }
        }
    }
    if history_changed {
        // Bevy's UI layout clamps this sentinel to the measured content extent
        // after word wrapping. Keeping the component at the sentinel until the
        // first wheel event also avoids a one-frame CPU-side height estimate.
        // New lines follow the latest entry only while the reader is already
        // there; NPC barks and rewards must not yank a scrolled-back log.
        let follow_latest = !rendered_history.initialized
            || rendered_history.selected != model.chat.selected
            || chat_log_shows_latest(
                chat_scroll_position.y,
                chat_log_max_scroll(chat_scroll_computed),
            );
        if follow_latest {
            chat_scroll_position.0 = Vec2::new(0.0, CHAT_SCROLL_TO_LATEST);
        }
        rendered_history.initialized = true;
        rendered_history.selected = model.chat.selected;
        rendered_history.text_colors = model.chat.text_colors;
        rendered_history.lines.clear();
        rendered_history.lines.extend_from_slice(visible_lines);
    }
}

pub(super) fn retained_chat_lines(lines: &[ChatLineUi]) -> &[ChatLineUi] {
    &lines[lines.len().saturating_sub(CHAT_HISTORY_CAPACITY)..]
}

pub(super) fn chat_palette_color(index: u8) -> Color {
    let (red, green, blue) = OPTION_CHAT_PALETTE_RGB
        .get(usize::from(index))
        .copied()
        .unwrap_or(OPTION_CHAT_PALETTE_RGB[0]);
    Color::srgb(red, green, blue)
}

pub(super) fn chat_line_color(kind: ChatLineKind, settings: TextColorSettings) -> Color {
    match kind {
        ChatLineKind::Normal => chat_palette_color(settings.general),
        ChatLineKind::Buddy => chat_palette_color(settings.buddy),
        ChatLineKind::Group => chat_palette_color(settings.group),
        ChatLineKind::Npc => CHAT_NPC_TEXT_COLOR,
        ChatLineKind::Receive => CHAT_RECEIVE_TEXT_COLOR,
        ChatLineKind::Attack => CHAT_ATTACK_TEXT_COLOR,
        ChatLineKind::System | ChatLineKind::Damage | ChatLineKind::Tutorial => {
            CHAT_DAMAGE_TEXT_COLOR
        }
    }
}

pub(super) fn chat_scroll_position(current: f32, max_scroll: f32, delta: f32) -> f32 {
    let max_scroll = if max_scroll.is_finite() {
        max_scroll.max(0.0)
    } else {
        0.0
    };
    (current.clamp(0.0, max_scroll) + delta).clamp(0.0, max_scroll)
}

pub(super) fn chat_log_max_scroll(computed: &ComputedNode) -> f32 {
    ((computed.content_size() - computed.size()) * computed.inverse_scale_factor())
        .max(Vec2::ZERO)
        .y
}

/// True while the log shows its newest line, including the scroll-to-latest
/// sentinel that layout clamps to the content extent.
pub(super) fn chat_log_shows_latest(position: f32, max_scroll: f32) -> bool {
    position >= max_scroll - 1.0
}

pub(super) fn scroll_chat_history(
    mut wheel_events: MessageReader<MouseWheel>,
    model: Res<GameplayUiModel>,
    scrollbar_parts: Query<&Interaction, With<ChatScrollbarPart>>,
    mut viewport: Single<(&Interaction, &mut ScrollPosition, &ComputedNode), With<ChatLogViewport>>,
) {
    let (interaction, scroll, computed) = &mut *viewport;
    // The scrollbar sits beside the log; the wheel must scroll over it too.
    let pointer_over = matches!(**interaction, Interaction::Hovered | Interaction::Pressed)
        || scrollbar_parts
            .iter()
            .any(|part| matches!(part, Interaction::Hovered | Interaction::Pressed));
    let hovered = model.visible && model.chat.visible && model.chat.input_enabled && pointer_over;
    let max_scroll = chat_log_max_scroll(computed);
    let mut position = scroll.y;
    let mut changed = false;
    for event in wheel_events.read() {
        if !hovered {
            continue;
        }
        let delta = match event.unit {
            MouseScrollUnit::Line => -event.y * CHAT_SCROLL_WHEEL_LINE,
            MouseScrollUnit::Pixel => -event.y,
        };
        position = chat_scroll_position(position, max_scroll, delta);
        changed = true;
    }
    if changed {
        scroll.y = position;
    }
}
