use super::*;

pub(super) fn tick_buddy_ui(
    time: Option<Res<Time>>,
    mut model: ResMut<BuddyUiModel>,
    mut outbox: ResMut<BuddyUiOutbox>,
) {
    let Some(time) = time else {
        return;
    };
    for action in model.advance_time(time.delta_secs()) {
        outbox.push(action);
    }
}

pub(super) fn update_buddy_control_visuals(
    model: Res<BuddyUiModel>,
    assets: Res<BuddyUiAssets>,
    mut controls: Query<(&Interaction, &BuddyControlMarker, &mut ImageNode, &Children)>,
    mut text_colors: Query<(&BuddyTextStyle, &mut TextColor)>,
) {
    let view = buddy_ui_view(&model);
    for (interaction, marker, mut image, children) in &mut controls {
        let enabled = match marker.0 {
            BuddyControl::Delete => view.delete_enabled && !view.add_dialog_open,
            BuddyControl::Warp => view.warp_enabled && !view.add_dialog_open,
            BuddyControl::Add => !view.add_dialog_open,
            BuddyControl::ModalCancel | BuddyControl::ModalAdd => view.add_dialog_open,
        };
        let effective_interaction = if enabled {
            *interaction
        } else {
            Interaction::None
        };
        let desired_image = assets.control_image(marker.0, effective_interaction);
        if image.image != desired_image {
            image.image = desired_image;
        }
        let desired_color = if enabled {
            Color::WHITE
        } else {
            Color::srgba(0.52, 0.52, 0.52, 0.78)
        };
        if image.color != desired_color {
            image.color = desired_color;
        }
        for child in children.iter() {
            if let Ok((style, mut color)) = text_colors.get_mut(child) {
                let desired_color = if enabled {
                    let spec = style.spec();
                    color_from_rgba(match effective_interaction {
                        Interaction::Pressed => spec.active_color,
                        Interaction::Hovered => spec.hover_color,
                        Interaction::None => spec.normal_color,
                    })
                } else {
                    Color::srgba(0.55, 0.55, 0.55, 0.78)
                };
                if color.0 != desired_color {
                    color.0 = desired_color;
                }
            }
        }
    }
}
