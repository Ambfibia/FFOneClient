use super::*;
use bevy::input::{ButtonState, keyboard::KeyboardInput};
use crate::gameplay_ui::{GameplayUiAudioCue, GameplayUiAudioOutbox};

#[derive(Default, Resource)]
pub(super) struct TarosEditor {
    draft: Option<String>,
}

pub(super) fn enable_controls(
    mut commands: Commands,
    elements: Query<(Entity, &Pc2pcUiElement), Added<Pc2pcUiElement>>,
) {
    for (entity, element) in &elements {
        if matches!(
            element,
            Pc2pcUiElement::MainButton
                | Pc2pcUiElement::Close
                | Pc2pcUiElement::AddTaros
                | Pc2pcUiElement::InventorySlotFrame(_)
                | Pc2pcUiElement::LocalOfferFrame(_)
        ) {
            commands
                .entity(entity)
                .insert((Button, Pickable::default(), bevy::ui::FocusPolicy::Block))
                .remove::<Pc2pcDisabledControl>();
        }
    }
}

pub(super) fn interact(
    controls: Query<(&Pc2pcUiElement, &Interaction), Changed<Interaction>>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut events: MessageReader<KeyboardInput>,
    modal: Res<Pc2pcModalState>,
    capabilities: Res<Pc2pcBackendCapabilities>,
    mut model: ResMut<Pc2pcUiModel0104>,
    mut editor: ResMut<TarosEditor>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
) {
    if !model.state.phase.accepts_actions() || !modal.controls_enabled() {
        events.clear();
        editor.draft = None;
        return;
    }
    if let Some(draft) = &mut editor.draft {
        for event in events
            .read()
            .filter(|event| event.state == ButtonState::Pressed)
        {
            if event.key_code == KeyCode::Backspace {
                draft.pop();
            }
            if let Some(text) = &event.text {
                for c in text.chars().filter(char::is_ascii_digit) {
                    if draft.len() < 9 {
                        draft.push(c);
                    }
                }
            }
        }
        if keyboard
            .as_ref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::Escape))
        {
            editor.draft = None;
            return;
        }
        if keyboard.as_ref().is_some_and(|keys| {
            keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)
        }) {
            let amount = draft.parse().unwrap_or(0);
            let _ = model.request_register_taros(*modal, *capabilities, amount);
            editor.draft = None;
        }
        return;
    }
    events.clear();
    if keyboard
        .as_ref()
        .is_some_and(|keys| keys.just_pressed(KeyCode::Escape))
    {
        let _ = model.request_cancel(*modal);
        return;
    }
    for (element, interaction) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *element {
            Pc2pcUiElement::MainButton => {
                if model.request_confirm(*modal).is_ok() {
                    audio.push(GameplayUiAudioCue::YesButton);
                }
            }
            Pc2pcUiElement::Close => {
                let _ = model.request_cancel(*modal);
            }
            Pc2pcUiElement::AddTaros => {
                editor.draft = Some(String::new());
                audio.push(GameplayUiAudioCue::ButtonSound);
            }
            Pc2pcUiElement::LocalOfferFrame(slot) => {
                if model.request_unregister_item(*modal, slot).is_ok() {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            Pc2pcUiElement::InventorySlotFrame(slot) => {
                if let Some(empty) = model
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.local_offer().iter().position(Option::is_none))
                {
                    if model.request_register_item(*modal, slot, empty, None).is_ok() {
                        audio.push(GameplayUiAudioCue::ButtonSound);
                    }
                }
            }
            _ => {}
        }
        break;
    }
}

#[derive(Component)]
pub(super) struct TarosEditorText;
pub(super) fn spawn_editor(
    mut commands: Commands,
    roots: Query<Entity, Added<Pc2pcUiRoot>>,
    assets: Res<Pc2pcUiAssets>,
) {
    for root in &roots {
        commands.entity(root).with_child((
            Node {
                position_type: PositionType::Absolute,
                left: px(200),
                top: px(280),
                width: px(400),
                height: px(80),
                display: Display::None,
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.08, 0.15)),
            ZIndex(20),
            Text::new(""),
            Pc2pcTextStyle0104::LabelUpperLeft.font(&assets),
            Pc2pcTextStyle0104::LabelUpperLeft.layout(),
            Pc2pcTextStyle0104::LabelUpperLeft,
            TextColor(Color::WHITE),
            LocalizedText::new(
                "ui.pc2pc.taros_edit",
                "Taros: {amount}\nEnter to submit, Esc to cancel",
            )
            .with_arg("amount", ""),
            TarosEditorText,
        ));
    }
}
pub(super) fn bind_editor(
    editor: Res<TarosEditor>,
    mut text: Query<(&mut Node, &mut LocalizedText), With<TarosEditorText>>,
) {
    for (mut node, mut localized) in &mut text {
        node.display = display_if(editor.draft.is_some());
        if let Some(draft) = &editor.draft {
            *localized = LocalizedText::new(
                "ui.pc2pc.taros_edit",
                "Taros: {amount}\nEnter to submit, Esc to cancel",
            )
            .with_arg("amount", format!("{draft}|"));
        }
    }
}
