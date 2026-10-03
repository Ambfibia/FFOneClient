use super::*;

pub const BUDDY_SCROLL_VELOCITY: f32 = 200.0;

pub const BUDDY_BLUE_BUTTON_PATH: &str = "ui/en/gameplay/chat/blue_button_normal.png";

pub const BUDDY_BLUE_BUTTON_OVER_PATH: &str = "ui/en/gameplay/chat/blue_button_over.png";

pub const BUDDY_RED_BUTTON_PATH: &str = "ui/en/character/selection/controls/red_button_normal.png";

pub const BUDDY_RED_BUTTON_OVER_PATH: &str =
    "ui/en/character/selection/controls/red_button_over.png";

pub const BUDDY_CANCEL_BUTTON_PATH: &str = "ui/en/character/selection/controls/CancelNormal.png";

// `FusionFallChatSkin` path ID 1368 owns the default BeginScrollView
// primitives below (Texture2D path IDs 374/324/63/415). These published
// Option assets are byte-identical clean-primary conversions of those four
// objects, so Buddy reuses the validated bytes instead of duplicating them.
pub const BUDDY_SCROLL_TRACK_PATH: &str = "ui/en/option/scroll-bar.png";

pub const BUDDY_SCROLL_THUMB_PATH: &str = "ui/en/option/scroll-thumb.png";

pub const BUDDY_SCROLL_UP_PATH: &str = "ui/en/option/scroll-up.png";

pub const BUDDY_SCROLL_DOWN_PATH: &str = "ui/en/option/scroll-down.png";

pub const BUDDY_BLUE_BUTTON_TEXT_Y_OFFSET: f32 = 0.0;

pub const BUDDY_RED_BUTTON_TEXT_Y_OFFSET: f32 = 0.0;

pub const BUDDY_QUIT_BUTTON_TEXT_Y_OFFSET: f32 = 0.0;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyAddInputText;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum BuddyScrollbarPart {
    Track,
    Thumb,
    Up,
    Down,
}

#[derive(Debug, Default, Resource)]
pub(super) struct BuddyKeyboardCapture(pub(super) bool);

#[derive(Debug, Default, Resource)]
pub(super) struct BuddyScrollbarDrag {
    pub(super) source_y_offset: Option<f32>,
}

pub(super) fn begin_buddy_keyboard_capture(
    model: Res<BuddyUiModel>,
    mut capture: ResMut<BuddyKeyboardCapture>,
) {
    capture.0 = model.add_dialog_open;
}

pub(super) fn handle_buddy_keyboard(
    keys: Option<MessageReader<KeyboardInput>>,
    mut model: ResMut<BuddyUiModel>,
    mut outbox: ResMut<BuddyUiOutbox>,
) {
    if !model.add_dialog_open {
        return;
    }
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state != ButtonState::Pressed {
            continue;
        }
        match key.key_code {
            KeyCode::Escape => {
                model.cancel_add_dialog();
                continue;
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                outbox.push(model.submit_add_dialog());
                continue;
            }
            KeyCode::Backspace => {
                model.pop_add_name_character();
                continue;
            }
            _ => {}
        }
        if let Some(text) = key.text.as_deref() {
            model.push_add_name_text(text);
        }
    }
}

pub(super) fn clear_captured_buddy_keyboard_messages(
    capture: Res<BuddyKeyboardCapture>,
    messages: Option<ResMut<Messages<KeyboardInput>>>,
) {
    if !capture.0 {
        return;
    }
    if let Some(mut messages) = messages {
        messages.clear();
    }
}

pub(super) fn handle_buddy_scroll_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    wheel: Option<MessageReader<MouseWheel>>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    mut drag: ResMut<BuddyScrollbarDrag>,
    mut model: ResMut<BuddyUiModel>,
) {
    let Ok(window) = windows.single() else {
        drag.source_y_offset = None;
        return;
    };
    let cursor = window.cursor_position();
    let layout = buddy_panel_layout(
        Vec2::new(window.width(), window.height()),
        model.chat_window_style,
        model.quick_slot_active,
        model.effective_ui_scale(),
    );
    let source_cursor = cursor.map(|cursor| {
        Vec2::new(
            (cursor.x - layout.painted_left) / layout.scale - BUDDY_INNER_LIST_RECT.x,
            (cursor.y - layout.painted_top) / layout.scale - BUDDY_INNER_LIST_RECT.y,
        )
    });
    let in_list = source_cursor.is_some_and(|cursor| {
        cursor.x >= 0.0
            && cursor.x <= BUDDY_INNER_LIST_RECT.width
            && cursor.y >= 0.0
            && cursor.y <= BUDDY_INNER_LIST_RECT.height
    });

    let mut wheel_axis = 0.0;
    if let Some(mut wheel) = wheel {
        for event in wheel.read() {
            wheel_axis += match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y / BUDDY_SCROLL_VELOCITY,
            };
        }
    }
    if model.visible && !model.add_dialog_open && in_list && wheel_axis != 0.0 {
        model.scroll_by_legacy_axis(wheel_axis);
    }

    let scrollbar = buddy_scrollbar_layout(model.visible_count(), model.scroll_y());
    if !model.visible || model.add_dialog_open || !scrollbar.visible {
        drag.source_y_offset = None;
        return;
    }
    let Some(mouse_buttons) = mouse_buttons else {
        return;
    };
    if mouse_buttons.just_released(MouseButton::Left) {
        drag.source_y_offset = None;
    }
    let Some(cursor) = source_cursor else {
        return;
    };
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if buddy_rect_contains(scrollbar.up, cursor) {
            let next_scroll = model.scroll_y() - BUDDY_LIST_ROW_STEP;
            model.set_scroll_y(next_scroll);
        } else if buddy_rect_contains(scrollbar.down, cursor) {
            let next_scroll = model.scroll_y() + BUDDY_LIST_ROW_STEP;
            model.set_scroll_y(next_scroll);
        } else if buddy_rect_contains(scrollbar.thumb, cursor) {
            drag.source_y_offset = Some(cursor.y - scrollbar.thumb.y);
        } else if buddy_rect_contains(scrollbar.track, cursor) {
            let page = BUDDY_INNER_LIST_RECT.height;
            if cursor.y < scrollbar.thumb.y {
                let next_scroll = model.scroll_y() - page;
                model.set_scroll_y(next_scroll);
            } else if cursor.y > scrollbar.thumb.y + scrollbar.thumb.height {
                let next_scroll = model.scroll_y() + page;
                model.set_scroll_y(next_scroll);
            }
        }
    }
    if mouse_buttons.pressed(MouseButton::Left) {
        if let Some(offset) = drag.source_y_offset {
            let maximum_scroll = (model.visible_count() as f32 * BUDDY_LIST_ROW_STEP
                - BUDDY_INNER_LIST_RECT.height)
                .max(0.0);
            let travel = BUDDY_SCROLL_TRACK_RECT.height - scrollbar.thumb.height;
            if maximum_scroll > 0.0 && travel > 0.0 {
                let thumb_y = (cursor.y - offset).clamp(
                    BUDDY_SCROLL_TRACK_RECT.y,
                    BUDDY_SCROLL_TRACK_RECT.y + travel,
                );
                let fraction = (thumb_y - BUDDY_SCROLL_TRACK_RECT.y) / travel;
                model.set_scroll_y(fraction * maximum_scroll);
            }
        }
    }
}

pub(super) fn handle_buddy_interactions(
    controls: Query<(&Interaction, &BuddyControlMarker), Changed<Interaction>>,
    rows: Query<(&Interaction, &BuddyRow), Changed<Interaction>>,
    mut model: ResMut<BuddyUiModel>,
    mut outbox: ResMut<BuddyUiOutbox>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
) {
    if !model.add_dialog_open {
        for (interaction, row) in &rows {
            if *interaction == Interaction::Pressed {
                let _ = model.select_slot(row.slot);
            }
        }
    }
    for (interaction, marker) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match marker.0 {
            BuddyControl::Delete if !model.add_dialog_open => {
                audio.push(GameplayUiAudioCue::ButtonSound);
                if let Ok(action) = model.request_remove() {
                    outbox.push(action);
                }
            }
            BuddyControl::Warp if !model.add_dialog_open => {
                audio.push(GameplayUiAudioCue::ButtonSound);
                if let Ok(action) = model.request_warp() {
                    outbox.push(action);
                }
            }
            BuddyControl::Add if !model.add_dialog_open => model.open_add_dialog(),
            BuddyControl::ModalCancel if model.add_dialog_open => {
                audio.push(GameplayUiAudioCue::NoButton);
                model.cancel_add_dialog();
            }
            BuddyControl::ModalAdd if model.add_dialog_open => {
                audio.push(GameplayUiAudioCue::YesButton);
                outbox.push(model.submit_add_dialog());
            }
            _ => {}
        }
    }
}
