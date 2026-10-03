//! Chat keyboard/pointer input, edit visuals, resizing and button visuals.

use super::actions::{
    GameplayUiAction, GameplayUiAudioCue, GameplayUiAudioOutbox, GameplayUiOutbox,
};
use super::assets::GameplayUiAssets;
use super::chat_model::{
    CHAT_INPUT_PADDING, CHAT_TAB_HOVER_TEXT_COLOR, CHAT_TAB_NORMAL_TEXT_COLOR,
    CHAT_TAB_SELECTED_TEXT_COLOR, ChatInputHistory, ChatKeyboardCommand, ChatResizeDrag, ChatUi,
    clamped_chat_size, reduce_chat_keyboard,
};
use super::chat_spawn::{
    ChatInputText, ChatResizeHandle, ChatTabButton, ChatTabLabel, ChatTextFieldButton,
    SendChatButton, SendChatLabel,
};
use super::model::{GameplayMenuTransition, GameplayUiModel};
use super::quick_chat::{EmoteButton, MenuChatButton, QuickChatMenuMode};
use crate::localization::LocalizedText;
use crate::text_edit::{self, EditVisual, TextEdit};
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    window::PrimaryWindow,
};

#[derive(Resource, Default)]
pub struct GameplayControllerMenuInput(pub bool);

pub(super) fn handle_chat_keyboard_input(
    keys: Option<MessageReader<KeyboardInput>>,
    buttons: Option<Res<ButtonInput<KeyCode>>>,
    mut model: ResMut<GameplayUiModel>,
    mut history: ResMut<ChatInputHistory>,
    controller: Option<ResMut<GameplayControllerMenuInput>>,
    mut outbox: ResMut<GameplayUiOutbox>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
) {
    let controller_enter = controller.is_some_and(|mut input| std::mem::take(&mut input.0));
    let input_enabled = model.visible && model.chat.visible && model.chat.input_enabled;
    if !input_enabled {
        // Retrobution's tutorial branch clears ChatString every IMGUI pass.
        // Fail closed without emitting menu/network intent from a gated state.
        if model.chat.active || !model.chat.input.is_empty() {
            model.chat.active = false;
            model.chat.input.clear();
            model.chat.edit = TextEdit::default();
        }
        return;
    }

    if controller_enter {
        for action in reduce_chat_keyboard(&mut model.chat, &mut history, ChatKeyboardCommand::Enter) {
            match &action {
                GameplayUiAction::OpenNanocomMenu => audio.push(GameplayUiAudioCue::OpenScreen),
                GameplayUiAction::CloseNanocomMenu => audio.push(GameplayUiAudioCue::CloseScreen),
                _ => {}
            }
            outbox.push(action);
        }
    }
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state != ButtonState::Pressed {
            continue;
        }
        let (control, shift) = text_edit::modifiers(buttons.as_deref());
        let command = match key.key_code {
            KeyCode::KeyA if control => Some(ChatKeyboardCommand::Edit {
                key: key.key_code,
                control,
                shift,
            }),
            KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::Delete => Some(ChatKeyboardCommand::Edit {
                key: key.key_code,
                control,
                shift,
            }),
            KeyCode::Enter | KeyCode::NumpadEnter => Some(ChatKeyboardCommand::Enter),
            KeyCode::ArrowUp => Some(ChatKeyboardCommand::PreviousHistory),
            KeyCode::ArrowDown => Some(ChatKeyboardCommand::NextHistory),
            KeyCode::Backspace => Some(ChatKeyboardCommand::Backspace),
            _ => None,
        };
        if let Some(command) = command {
            for action in reduce_chat_keyboard(&mut model.chat, &mut history, command) {
                match &action {
                    GameplayUiAction::OpenNanocomMenu => audio.push(GameplayUiAudioCue::OpenScreen),
                    GameplayUiAction::CloseNanocomMenu => {
                        audio.push(GameplayUiAudioCue::CloseScreen)
                    }
                    _ => {}
                }
                outbox.push(action);
            }
            continue;
        }
        if control {
            continue;
        }
        let produced_text = key.text.as_deref().or_else(|| match &key.logical_key {
            Key::Character(text) => Some(text.as_str()),
            _ => None,
        });
        if let Some(text) = produced_text {
            for action in reduce_chat_keyboard(
                &mut model.chat,
                &mut history,
                ChatKeyboardCommand::Text(text),
            ) {
                outbox.push(action);
            }
        }
    }
}

pub(super) fn bind_chat_edit_visual(
    model: Res<GameplayUiModel>,
    mut texts: Query<&mut EditVisual, With<ChatInputText>>,
) {
    for mut visual in &mut texts {
        let active =
            model.visible && model.chat.visible && model.chat.input_enabled && model.chat.active;
        if visual.active != active {
            visual.active = active;
        }
        if visual.edit != model.chat.edit {
            visual.edit = model.chat.edit.clone();
        }
    }
}

pub(super) fn handle_chat_edit_pointer(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    fields: Query<&Interaction, With<ChatTextFieldButton>>,
    texts: Query<
        (
            &bevy::text::ComputedTextBlock,
            &EditVisual,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        With<ChatInputText>,
    >,
    mut model: ResMut<GameplayUiModel>,
    mut dragging: Local<Option<bool>>,
) {
    let Some(mouse) = mouse else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused
        || !mouse.pressed(MouseButton::Left)
        || !model.visible
        || !model.chat.visible
        || !model.chat.input_enabled
        || !model.chat.active
    {
        *dragging = None;
        return;
    }
    let start =
        mouse.just_pressed(MouseButton::Left) && fields.iter().any(|i| *i == Interaction::Pressed);
    if mouse.just_pressed(MouseButton::Left) {
        let (_, shift) = text_edit::modifiers(keys.as_deref());
        *dragging = start.then_some(shift);
    }
    let Some(extend) = *dragging else {
        return;
    };
    let Some(cursor) = window.physical_cursor_position() else {
        return;
    };
    let Ok((block, visual, computed, transform)) = texts.single() else {
        return;
    };
    let Some(position) = text_edit::hit_position(
        block,
        computed,
        transform,
        &model.chat.input,
        visual,
        cursor,
        CHAT_INPUT_PADDING,
    ) else {
        return;
    };
    let chat = &mut model.chat;
    chat.edit.clamp(&chat.input);
    chat.edit.place(position, extend);
    *dragging = Some(true);
}

pub(super) fn handle_chat_resize(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    resize_handle: Query<&Interaction, With<ChatResizeHandle>>,
    mut drag: ResMut<ChatResizeDrag>,
    mut model: ResMut<GameplayUiModel>,
) {
    let Some(mouse_buttons) = mouse_buttons else {
        drag.start_cursor = None;
        return;
    };
    if !model.visible || !model.chat.visible || !model.chat.active {
        drag.start_cursor = None;
        return;
    }
    if !mouse_buttons.pressed(MouseButton::Left) {
        drag.start_cursor = None;
        return;
    }
    let Ok(window) = windows.single() else {
        drag.start_cursor = None;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if drag.start_cursor.is_none()
        && resize_handle
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        drag.start_cursor = Some(cursor);
        drag.start_size = clamped_chat_size(model.chat.window_size);
    }
    let Some(start_cursor) = drag.start_cursor else {
        return;
    };
    let scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
        model.ui_scale
    } else {
        1.0
    };
    // The chat is bottom-left anchored. Screen-space upward movement therefore
    // increases the source window height, while rightward movement increases
    // its width.
    let source_delta = Vec2::new(
        (cursor.x - start_cursor.x) / scale,
        (start_cursor.y - cursor.y) / scale,
    );
    model.chat.window_size = clamped_chat_size(drag.start_size + source_delta);
}

pub(super) fn bind_chat_button_visuals(
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    mut tab_labels: Query<(&ChatTabLabel, &mut TextColor)>,
    mut send_labels: Query<&mut TextColor, (With<SendChatLabel>, Without<ChatTabLabel>)>,
    mut buttons: Query<
        (
            &Interaction,
            &mut ImageNode,
            Option<&ChatTabButton>,
            Has<MenuChatButton>,
            Has<EmoteButton>,
            Has<SendChatButton>,
            Has<ChatResizeHandle>,
        ),
        Or<(
            With<ChatTabButton>,
            With<MenuChatButton>,
            With<EmoteButton>,
            With<SendChatButton>,
            With<ChatResizeHandle>,
        )>,
    >,
) {
    for (interaction, mut image, tab, menu, emote, send, resize) in &mut buttons {
        image.image = if let Some(tab) = tab {
            let selected = tab.0 == model.chat.selected;
            let menu_locks_tabs = model.chat.quick_menu.mode != QuickChatMenuMode::Closed;
            let text_color = if selected {
                CHAT_TAB_SELECTED_TEXT_COLOR
            } else if !menu_locks_tabs && *interaction == Interaction::Hovered {
                CHAT_TAB_HOVER_TEXT_COLOR
            } else {
                CHAT_TAB_NORMAL_TEXT_COLOR
            };
            for (label, mut color) in &mut tab_labels {
                if label.0 == tab.0 {
                    color.0 = text_color;
                }
            }
            if selected {
                assets.chat_tab_selected.clone()
            } else if !menu_locks_tabs && *interaction == Interaction::Hovered {
                assets.chat_tab_over.clone()
            } else {
                assets.chat_tab_normal.clone()
            }
        } else if menu {
            if *interaction == Interaction::Hovered {
                assets.menu_chat_over.clone()
            } else {
                assets.menu_chat.clone()
            }
        } else if emote {
            if *interaction == Interaction::Hovered {
                assets.emote_over.clone()
            } else {
                assets.emote.clone()
            }
        } else if send {
            let hover_color = Color::srgb(0.0, 0.342_741_94, 0.528_225_8);
            for mut color in &mut send_labels {
                color.0 = if *interaction == Interaction::Hovered {
                    hover_color
                } else {
                    Color::WHITE
                };
            }
            match *interaction {
                Interaction::Hovered => assets.blue_button_over.clone(),
                Interaction::Pressed | Interaction::None => assets.blue_button.clone(),
            }
        } else if resize {
            match *interaction {
                Interaction::Pressed | Interaction::Hovered => assets.chat_resize_hover.clone(),
                Interaction::None => assets.chat_resize_normal.clone(),
            }
        } else {
            continue;
        };
    }
}

pub(super) fn localized_chat_input(
    chat: &ChatUi,
    transition: &GameplayMenuTransition,
) -> LocalizedText {
    if chat.active || transition.visible_or_transitioning() {
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &chat.input)
    } else {
        LocalizedText::new(
            "ui.hud.chat.open_hint",
            "Press ENTER to access chat and menus.",
        )
    }
}
