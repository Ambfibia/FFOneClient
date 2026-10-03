use super::*;

pub(super) fn invite_response(invite: &BuddyInvite, accepted: bool) -> BuddyUiAction {
    BuddyUiAction::InviteResponse {
        invite_id: invite.invite_id,
        requester_pc_id: invite.requester_pc_id,
        requester_pc_uid: invite.requester_pc_uid,
        accepted,
    }
}

#[must_use]
pub fn buddy_ui_view(model: &BuddyUiModel) -> BuddyUiView {
    let rows = model
        .visible_slot_indices()
        .enumerate()
        .filter_map(|(visible_index, slot)| {
            let entry = model.slot(slot)?;
            let top = visible_index as f32 * BUDDY_LIST_ROW_STEP - model.scroll_y;
            Some(BuddyRowView {
                slot,
                target: BuddyTarget {
                    slot,
                    pc_uid: entry.pc_uid,
                },
                display_name: entry.display_name(),
                name_is_verified: entry.name_check_flag == 1,
                free_chat: entry.free_chat,
                presence: entry.presence,
                selected: model.selected_slot == Some(slot),
                top,
                fully_visible: top >= 0.0
                    && top + BUDDY_LIST_ROW_STEP <= BUDDY_INNER_LIST_RECT.height,
            })
        })
        .collect::<Vec<_>>();
    let delete_enabled = model.selected_target().is_some();
    let warp_enabled = model
        .selected_slot
        .and_then(|slot| model.slot(slot))
        .is_some_and(|entry| entry.presence.is_online())
        && !model.player_moving;
    BuddyUiView {
        visible: model.visible,
        rows,
        delete_enabled,
        warp_enabled,
        add_dialog_open: model.add_dialog_open,
    }
}

pub(super) fn buddy_text_font(assets: &BuddyUiAssets, style: BuddyTextStyle) -> (TextFont, LineHeight) {
    let spec = style.spec();
    (
        TextFont {
            font: (match spec.font_role {
                BuddyFontRole::Jeffe => assets.jeffe_font.clone(),
                BuddyFontRole::Chalet => assets.chalet_font.clone(),
            })
            .into(),
            font_size: (spec.font_size).into(),
            ..default()
        },
        LineHeight::Px(spec.line_height),
    )
}

pub(crate) fn buddy_text_color(style: BuddyTextStyle) -> TextColor {
    TextColor(color_from_rgba(style.spec().normal_color))
}

pub(crate) fn buddy_text_node(rect: BuddyUiRect, style: BuddyTextStyle) -> Node {
    let spec = style.spec();
    let [left, right, top, bottom] = spec.padding;
    let mut node = rect.node();
    node.align_items = match spec.anchor {
        BuddyTextAnchor::UpperLeft => AlignItems::Start,
        BuddyTextAnchor::MiddleLeft | BuddyTextAnchor::MiddleCenter => AlignItems::Center,
    };
    node.justify_content = match spec.anchor {
        BuddyTextAnchor::UpperLeft | BuddyTextAnchor::MiddleLeft => JustifyContent::Start,
        BuddyTextAnchor::MiddleCenter => JustifyContent::Center,
    };
    node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
    node
}

pub(crate) fn buddy_text_transform(style: BuddyTextStyle) -> UiTransform {
    UiTransform::from_translation(Val2::px(0.0, style.spec().y_offset))
}

pub(super) fn bind_buddy_ui(
    model: Res<BuddyUiModel>,
    mut previous_view: Local<Option<BuddyUiView>>,
    mut previous_input: Local<Option<String>>,
    mut panels: Query<
        &mut Node,
        (
            With<BuddyPanel>,
            Without<BuddyModalRoot>,
            Without<BuddyRow>,
            Without<BuddyRowPart>,
            Without<BuddyScrollbarPart>,
        ),
    >,
    mut modals: Query<
        &mut Node,
        (
            With<BuddyModalRoot>,
            Without<BuddyPanel>,
            Without<BuddyRow>,
            Without<BuddyRowPart>,
            Without<BuddyScrollbarPart>,
        ),
    >,
    mut rows: Query<
        (&BuddyRow, &mut Node),
        (
            Without<BuddyPanel>,
            Without<BuddyModalRoot>,
            Without<BuddyRowPart>,
            Without<BuddyScrollbarPart>,
        ),
    >,
    mut row_parts: Query<
        (
            &BuddyRowPart,
            &mut Node,
            Option<&mut Visibility>,
            Option<&mut TextColor>,
            Option<&mut LocalizedText>,
        ),
        (
            Without<BuddyPanel>,
            Without<BuddyModalRoot>,
            Without<BuddyRow>,
            Without<BuddyAddInputText>,
            Without<BuddyScrollbarPart>,
        ),
    >,
    mut input_text: Query<&mut LocalizedText, (With<BuddyAddInputText>, Without<BuddyRowPart>)>,
    mut scrollbars: Query<(&BuddyScrollbarPart, &mut Node), Without<BuddyRowPart>>,
) {
    let view = buddy_ui_view(&model);
    if previous_view.as_ref() == Some(&view)
        && previous_input.as_deref() == Some(model.add_name_input.as_str())
    {
        return;
    }
    *previous_view = Some(view.clone());
    *previous_input = Some(model.add_name_input.clone());
    let scrollbar_layout = buddy_scrollbar_layout(view.rows.len(), model.scroll_y());
    for mut panel in &mut panels {
        panel.display = if view.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut modal in &mut modals {
        modal.display = if view.add_dialog_open {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (marker, mut node) in &mut rows {
        let row = view.rows.iter().find(|row| row.slot == marker.slot);
        let intersects = row.is_some_and(|row| {
            row.top < BUDDY_INNER_LIST_RECT.height && row.top + BUDDY_LIST_ROW_HEIGHT > 0.0
        });
        node.display = if intersects {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(row) = row {
            node.top = px(row.top);
        }
    }

    for (marker, mut node, visibility, text_color, localized) in &mut row_parts {
        let row = view.rows.iter().find(|row| row.slot == marker.slot);
        match marker.kind {
            BuddyRowPartKind::Selection => {
                node.display = if row.is_some_and(|row| row.selected) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            BuddyRowPartKind::SelectionSpike => {
                if let Some(mut visibility) = visibility {
                    *visibility = if row.is_some_and(|row| row.selected && row.fully_visible) {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            BuddyRowPartKind::FreeChat => {
                if let Some(mut visibility) = visibility {
                    *visibility = if row.is_some_and(|row| row.free_chat && row.fully_visible) {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            BuddyRowPartKind::Label => {
                if let Some(mut localized) = localized {
                    *localized = buddy_row_localized(row);
                }
                if let (Some(row), Some(mut color)) = (row, text_color) {
                    color.0 = if row.selected {
                        Color::srgb(0.1, 0.1, 0.1)
                    } else if row.presence.is_online() {
                        Color::srgb(1.0, 0.92, 0.02)
                    } else {
                        Color::srgb(0.5, 0.5, 0.5)
                    };
                }
            }
        }
    }
    for mut localized in &mut input_text {
        *localized = buddy_add_name_localized(model.add_name_input.clone());
    }
    for (part, mut node) in &mut scrollbars {
        node.display = if scrollbar_layout.visible {
            Display::Flex
        } else {
            Display::None
        };
        let rect = match part {
            BuddyScrollbarPart::Track => scrollbar_layout.track,
            BuddyScrollbarPart::Thumb => scrollbar_layout.thumb,
            BuddyScrollbarPart::Up => scrollbar_layout.up,
            BuddyScrollbarPart::Down => scrollbar_layout.down,
        };
        node.left = px(rect.x);
        node.top = px(rect.y);
        node.width = px(rect.width);
        node.height = px(rect.height);
    }
}
