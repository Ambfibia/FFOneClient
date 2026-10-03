//! Chat scrollbar geometry, pointer handling and binding.

use super::chat_log::chat_scroll_position;
use super::chat_model::{
    CHAT_SCROLLBAR_ARROW_HEIGHT, CHAT_SCROLLBAR_THUMB_MIN_HEIGHT, CHAT_SCROLLBAR_THUMB_WIDTH,
    CHAT_SCROLLBAR_WIDTH, ChatChannel, chat_layout,
};
use super::chat_spawn::{ChatLogViewport, ChatScrollbar, ChatScrollbarPart};
use super::model::{GameplayMenuTransition, GameplayUiModel};
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Default)]
pub(super) struct ChatScrollbarPointer {
    pub(super) drag: Option<(f32, f32)>,
    pub(super) held: Option<ChatScrollbarPart>,
    pub(super) repeat_at: f32,
    pub(super) channel: Option<ChatChannel>,
}

pub(super) fn chat_scrollbar_geometry(height: f32, max_scroll: f32) -> (f32, f32) {
    let track = (height - CHAT_SCROLLBAR_ARROW_HEIGHT * 2.0).max(0.0);
    let thumb = if height + max_scroll > 0.0 {
        (height / (height + max_scroll) * track)
            .clamp(CHAT_SCROLLBAR_THUMB_MIN_HEIGHT.min(track), track)
    } else {
        track
    };
    (track, thumb)
}

pub(super) fn handle_chat_scrollbar(
    time: Res<Time>,
    model: Res<GameplayUiModel>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    scrollbar: Single<(&Node, &bevy::ui::RelativeCursorPosition), With<ChatScrollbar>>,
    parts: Query<(&ChatScrollbarPart, &Interaction)>,
    mut viewport: Single<(&ComputedNode, &mut ScrollPosition), With<ChatLogViewport>>,
    mut pointer: Local<ChatScrollbarPointer>,
) {
    let (computed, scroll) = &mut *viewport;
    let height = computed.size().y * computed.inverse_scale_factor();
    let max_scroll = ((computed.content_size().y - computed.size().y)
        * computed.inverse_scale_factor())
    .max(0.0);
    let Some(buttons) = buttons else {
        *pointer = default();
        return;
    };
    if !model.visible
        || !model.chat.visible
        || !model.chat.input_enabled
        || scrollbar.0.display == Display::None
        || max_scroll <= 0.0
        || windows.iter().any(|window| !window.focused)
        || !buttons.pressed(MouseButton::Left)
        || pointer
            .channel
            .is_some_and(|channel| channel != model.chat.selected)
    {
        *pointer = default();
        return;
    }
    let Some(cursor) = scrollbar.1.normalized.filter(|cursor| cursor.is_finite()) else {
        *pointer = default();
        return;
    };
    let cursor_y = (cursor.y + 0.5) * height - CHAT_SCROLLBAR_ARROW_HEIGHT;
    let (track, thumb) = chat_scrollbar_geometry(height, max_scroll);
    let travel = track - thumb;
    let current = scroll.y.clamp(0.0, max_scroll);
    if let Some((start_cursor, start_scroll)) = pointer.drag {
        if travel > 0.0 {
            scroll.y = chat_scroll_position(
                start_scroll,
                max_scroll,
                (cursor_y - start_cursor) * max_scroll / travel,
            );
        }
        return;
    }
    let pressed = parts
        .iter()
        .filter(|(_, interaction)| **interaction == Interaction::Pressed)
        .map(|(part, _)| *part)
        .min_by_key(|part| {
            if *part == ChatScrollbarPart::Thumb {
                0
            } else {
                1
            }
        });
    if buttons.just_pressed(MouseButton::Left) {
        pointer.channel = Some(model.chat.selected);
        pointer.held = pressed;
        pointer.repeat_at = time.elapsed_secs() + 0.25;
        if pressed == Some(ChatScrollbarPart::Thumb) {
            pointer.drag = Some((cursor_y, current));
            return;
        }
    } else if pressed != pointer.held || time.elapsed_secs() < pointer.repeat_at {
        return;
    } else {
        pointer.repeat_at = time.elapsed_secs() + 0.03;
    }
    let delta = match pointer.held {
        Some(ChatScrollbarPart::Up) => -10.0,
        Some(ChatScrollbarPart::Down) => 10.0,
        Some(ChatScrollbarPart::Track) => {
            let thumb_top = current / max_scroll * travel;
            if cursor_y < thumb_top {
                -height * 0.9
            } else if cursor_y > thumb_top + thumb {
                height * 0.9
            } else {
                0.0
            }
        }
        _ => 0.0,
    };
    if delta != 0.0 {
        scroll.y = chat_scroll_position(current, max_scroll, delta);
    }
}

pub(super) fn bind_chat_scrollbar(
    model: Res<GameplayUiModel>,
    transition: Res<GameplayMenuTransition>,
    mut scrollbar: Single<&mut Node, With<ChatScrollbar>>,
    viewport: Single<
        (&ComputedNode, &ScrollPosition),
        (With<ChatLogViewport>, Without<ChatScrollbar>),
    >,
    mut parts: Query<(&ChatScrollbarPart, &mut Node), Without<ChatScrollbar>>,
) {
    let layout = chat_layout(
        model.chat.window_size,
        model.chat.active || transition.visible_or_transitioning(),
    );
    let (computed, scroll) = *viewport;
    let max_scroll = ((computed.content_size() - computed.size())
        * computed.inverse_scale_factor())
    .max(Vec2::ZERO)
    .y;
    let empty_state_visible = match model.chat.selected {
        ChatChannel::Buddy => model.chat.buddy_count == 0,
        ChatChannel::Group => !model.chat.group_available,
        ChatChannel::All => false,
    };
    scrollbar
        .reborrow()
        .map_unchanged(|value| &mut value.display)
        .set_if_neq(if max_scroll > 0.0 && !empty_state_visible {
            Display::Flex
        } else {
            Display::None
        });
    scrollbar
        .reborrow()
        .map_unchanged(|value| &mut value.left)
        .set_if_neq(px(layout.log.x - 21.0));
    scrollbar
        .reborrow()
        .map_unchanged(|value| &mut value.top)
        .set_if_neq(px(layout.log.y));
    scrollbar
        .reborrow()
        .map_unchanged(|value| &mut value.width)
        .set_if_neq(px(CHAT_SCROLLBAR_WIDTH));
    scrollbar
        .reborrow()
        .map_unchanged(|value| &mut value.height)
        .set_if_neq(px(layout.log.height));

    let (track_height, thumb_height) = chat_scrollbar_geometry(layout.log.height, max_scroll);
    let fraction = if max_scroll > 0.0 {
        scroll.y.clamp(0.0, max_scroll) / max_scroll
    } else {
        0.0
    };
    for (part, mut node) in &mut parts {
        match part {
            ChatScrollbarPart::Track => {
                node.reborrow()
                    .map_unchanged(|value| &mut value.top)
                    .set_if_neq(px(CHAT_SCROLLBAR_ARROW_HEIGHT));
                node.reborrow()
                    .map_unchanged(|value| &mut value.width)
                    .set_if_neq(px(CHAT_SCROLLBAR_WIDTH));
                node.reborrow()
                    .map_unchanged(|value| &mut value.height)
                    .set_if_neq(px(track_height));
            }
            ChatScrollbarPart::Up => {
                node.reborrow()
                    .map_unchanged(|value| &mut value.top)
                    .set_if_neq(px(0));
                node.reborrow()
                    .map_unchanged(|value| &mut value.width)
                    .set_if_neq(px(CHAT_SCROLLBAR_WIDTH));
                node.reborrow()
                    .map_unchanged(|value| &mut value.height)
                    .set_if_neq(px(CHAT_SCROLLBAR_ARROW_HEIGHT));
            }
            ChatScrollbarPart::Down => {
                node.reborrow()
                    .map_unchanged(|value| &mut value.top)
                    .set_if_neq(px(layout.log.height - CHAT_SCROLLBAR_ARROW_HEIGHT));
                node.reborrow()
                    .map_unchanged(|value| &mut value.width)
                    .set_if_neq(px(CHAT_SCROLLBAR_WIDTH));
                node.reborrow()
                    .map_unchanged(|value| &mut value.height)
                    .set_if_neq(px(CHAT_SCROLLBAR_ARROW_HEIGHT));
            }
            ChatScrollbarPart::Thumb => {
                node.reborrow()
                    .map_unchanged(|value| &mut value.top)
                    .set_if_neq(px(
                        CHAT_SCROLLBAR_ARROW_HEIGHT + fraction * (track_height - thumb_height)
                    ));
                node.reborrow()
                    .map_unchanged(|value| &mut value.width)
                    .set_if_neq(px(CHAT_SCROLLBAR_THUMB_WIDTH));
                node.reborrow()
                    .map_unchanged(|value| &mut value.height)
                    .set_if_neq(px(thumb_height));
            }
        }
    }
}
