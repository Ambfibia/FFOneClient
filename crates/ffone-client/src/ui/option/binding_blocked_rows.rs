use super::*;

pub(super) fn bind_blocked_rows(
    model: Res<OptionUiModel>,
    mut rows: Query<
        (
            &OptionBlockedRow,
            &mut Node,
            &mut ImageNode,
            &mut Pickable,
            &Children,
        ),
        Without<OptionBlockedScrollThumb>,
    >,
    mut labels: Query<
        (&OptionBlockedRowLabel, &mut LocalizedText, &mut TextColor),
        Without<OptionBlockedRow>,
    >,
    mut scroll_buttons: Query<
        (&OptionBlockedScrollButton, &mut Pickable),
        Without<OptionBlockedRow>,
    >,
    mut thumbs: Query<&mut Node, (With<OptionBlockedScrollThumb>, Without<OptionBlockedRow>)>,
    mut scroll_chrome: Query<&mut Visibility, With<OptionBlockedScrollChrome>>,
) {
    let projection = model.blocked_projection();
    let maximum = projection.len().saturating_sub(OPTION_BLOCKED_VISIBLE_ROWS);
    let scroll_start = model.blocked_scroll_row.min(maximum);
    let enabled =
        model.selected_tab == OptionTab::Social && model.page_body_enabled(OptionTab::Social);
    for (row, mut node, mut image, mut pickable, children) in &mut rows {
        let Some(projected) = projection.get(scroll_start + row.projection_index) else {
            node.display = Display::None;
            *pickable = Pickable::IGNORE;
            continue;
        };
        node.display = Display::Flex;
        *pickable = if enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
        let selected = model.selected_blocked_slot == Some(projected.slot);
        image.color = if selected {
            Color::WHITE
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.0)
        };
        for child in children.iter() {
            if let Ok((label, mut localized_component, mut color)) = labels.get_mut(child)
                && label.projection_index == row.projection_index
            {
                *localized_component = option_passthrough_text(&projected.display_name);
                color.0 = if selected {
                    Color::BLACK
                } else {
                    Color::srgb(1.0, 0.86, 0.08)
                };
            }
        }
    }
    for (button, mut pickable) in &mut scroll_buttons {
        let can_scroll = match button {
            OptionBlockedScrollButton::Up => scroll_start > 0,
            OptionBlockedScrollButton::Down => scroll_start < maximum,
        };
        *pickable = if enabled && can_scroll {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
    let scrollbar_visible = maximum > 0;
    for mut visibility in &mut scroll_chrome {
        *visibility = if scrollbar_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let ratio = if maximum == 0 {
        0.0
    } else {
        scroll_start as f32 / maximum as f32
    };
    let thumb_height = if projection.is_empty() {
        266.0
    } else {
        (266.0 * OPTION_BLOCKED_VISIBLE_ROWS as f32 / projection.len() as f32).clamp(15.0, 266.0)
    };
    for mut node in &mut thumbs {
        node.top = px(117.0 + ratio * (266.0 - thumb_height) - OPTION_SCROLL_THUMB_OVERFLOW);
        node.height = px(thumb_height + OPTION_SCROLL_THUMB_OVERFLOW * 2.0);
    }
}
