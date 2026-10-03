use super::*;

pub(super) fn absolute_node(rect: LegacySelectionRect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.x),
        top: px(rect.y),
        width: px(rect.width),
        height: px(rect.height),
        ..default()
    }
}

pub(super) fn image_node(handle: Handle<Image>) -> ImageNode {
    ImageNode {
        image: handle,
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

pub(super) fn tinted_image_node(handle: Handle<Image>, color: Color) -> ImageNode {
    let mut image = image_node(handle);
    image.color = color;
    image
}

pub(super) fn edit_character_delete_name(
    keys: Option<MessageReader<KeyboardInput>>,
    mut model: ResMut<CharacterSelectionUiModel>,
) {
    if !model.visible || model.delete_confirmation_pc_uid.is_none() || model.delete_request_pending
    {
        return;
    }
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state != ButtonState::Pressed {
            continue;
        }
        if key.key_code == KeyCode::Backspace {
            model.delete_name_input.pop();
            continue;
        }
        let Some(produced) = key.text.as_deref() else {
            continue;
        };
        for character in produced.chars() {
            if character.is_control() || model.delete_name_input.chars().count() >= 32 {
                continue;
            }
            model.delete_name_input.push(character);
        }
    }
}
