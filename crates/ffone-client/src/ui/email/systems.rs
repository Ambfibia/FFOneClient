use super::*;

pub(super) fn tick_email_opening(time: Res<Time>, mut model: ResMut<EmailUiModel>) {
    model.tick_opening(time.delta_secs());
}

pub(super) fn apply_group(node: &mut Node, transform: &mut UiTransform, rect: EmailUiRect, scale: Vec2) {
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
    transform.scale = scale;
}

pub(super) fn sync_email_buddy_list_content(
    model: Res<EmailUiModel>,
    mut contents: Query<&mut Node, With<EmailUiBuddyListContent>>,
) {
    for mut node in &mut contents {
        node.top = px(-model.buddy_scroll_y());
        node.width = px(EMAIL_UI_BUDDY_LIST_CONTENT_WIDTH);
        node.height = px(EMAIL_UI_BUDDY_ROW_HEIGHT * model.buddies.len() as f32);
    }
}

/// `Panel_EmailList.DoData` guide branch: the slot backdrop for a selected row,
/// the sender NPC icon over it, and FROM/SUBJECT moved right by
/// `rectGuideIcon.width`. The player tab keeps the unshifted rectangles.
pub(super) fn sync_email_guide_sender_icon(
    model: Res<EmailUiModel>,
    asset_server: Res<AssetServer>,
    mut icons: Query<
        (&EmailUiGuideIcon, &mut Node, &mut ImageNode),
        Without<EmailUiGuideIconShift>,
    >,
    mut labels: Query<(&EmailUiGuideIconShift, &mut Node), Without<EmailUiGuideIcon>>,
) {
    if !model.is_changed() {
        return;
    }
    let guide = model.folder == EmailFolder::Guide;
    let selected = guide.then(|| model.selected_guide()).flatten();
    let sender_icon = selected.and_then(|message| message.sender_icon_path.as_deref());
    for (layer, mut node, mut image) in &mut icons {
        let visible = match layer {
            EmailUiGuideIcon::SlotBackdrop => selected.is_some(),
            EmailUiGuideIcon::Sender => {
                if let Some(path) = sender_icon {
                    image.image = asset_server.load(path.to_owned());
                }
                sender_icon.is_some()
            }
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    let offset = if guide {
        EMAIL_UI_DETAIL_ICON_RECT.width
    } else {
        0.0
    };
    for (shift, mut node) in &mut labels {
        node.left = px(shift.left + offset);
    }
}

pub(super) fn sync_email_buddy_row_visuals(
    model: Res<EmailUiModel>,
    rows: Query<(&Interaction, &Children), With<EmailUiBuddyRow>>,
    mut colors: Query<&mut TextColor>,
) {
    for (interaction, children) in &rows {
        let value = if model.input_enabled()
            && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
        {
            Color::BLACK
        } else {
            Color::WHITE
        };
        for child in children.iter() {
            if let Ok(mut color) = colors.get_mut(child) {
                color.0 = value;
            }
        }
    }
}
