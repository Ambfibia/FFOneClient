use super::*;

pub(super) fn rebuild_system_message_ui(
    mut commands: Commands,
    model: Res<SystemMessageUiModel>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    assets: Res<SystemMessageUiAssets>,
    asset_server: Res<AssetServer>,
    mut roots: Query<(Entity, &mut Node), With<SystemMessageUiRoot>>,
    layers: Query<Entity, With<SystemMessageLayer>>,
) {
    let language_changed = language.as_ref().is_some_and(|value| value.is_changed());
    if !model.is_changed() && !language_changed {
        return;
    }
    let Ok((root, mut root_node)) = roots.single_mut() else {
        return;
    };
    root_node.display = if model.is_empty() {
        Display::None
    } else {
        Display::Flex
    };
    for layer in &layers {
        commands.entity(layer).despawn();
    }
    if model.is_empty() {
        return;
    }

    let current_index = model.len() - 1;
    commands.entity(root).with_children(|root| {
        for (index, message) in model.stack().iter().enumerate() {
            let resolved_text = match (localization.as_deref(), language.as_deref()) {
                (Some(localization), Some(language)) => {
                    localization.text(language, &message.localized)
                }
                _ => message.text.clone(),
            };
            spawn_system_message_layer(
                root,
                index,
                message,
                &resolved_text,
                index == current_index,
                !model.focus_out(),
                &assets,
                &asset_server,
            );
        }
    });
}

pub(super) fn style_padding(padding: [f32; 4]) -> UiRect {
    UiRect {
        left: px(padding[0]),
        right: px(padding[1]),
        top: px(padding[2]),
        bottom: px(padding[3]),
    }
}

pub(super) fn array_color(value: [f32; 4]) -> Color {
    Color::srgba(value[0], value[1], value[2], value[3])
}

pub(super) fn system_message_text_color() -> Color {
    gui_style("FusionFallSysMessageSkin", "label")
        .and_then(|style| style.states.get("normal"))
        .map_or(Color::srgb(0.923_357_67, 0.9, 0.9), |state| {
            bevy_color(&state.text_color)
        })
}

pub(super) fn bevy_color(color: &GuiColor) -> Color {
    Color::srgba(
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    )
}
