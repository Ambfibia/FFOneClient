use super::*;

pub(super) fn spawn_email_buddy_row(
    parent: &mut ChildSpawnerCommands,
    row: usize,
    buddy: &EmailBuddy,
    assets: &EmailUiAssets,
) {
    let localized = if buddy.name_check_flag == 1 {
        email_passthrough_text(format!("{} {}", buddy.first_name, buddy.last_name))
    } else {
        email_player_label_text(buddy.pc_uid)
    };
    let style = EmailUiTextStyle::RightLabel;
    let rect = EmailUiRect::new(
        0.0,
        row as f32 * EMAIL_UI_BUDDY_ROW_HEIGHT,
        EMAIL_UI_BUDDY_ROW_WIDTH,
        EMAIL_UI_BUDDY_ROW_HEIGHT,
    );
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent
        .spawn((
            Button,
            EmailUiBuddyRow { row },
            node,
            BackgroundColor(Color::NONE),
        ))
        .with_children(|row| {
            spawn_text_entity(
                row,
                None,
                localized,
                assets,
                style,
                Color::WHITE,
                true,
                Some(style.content_bounds(rect)),
            );
        });
}

pub(super) fn spawn_email_calculator_popup(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    parent
        .spawn((
            EmailPopup::AddTaros,
            EMAIL_UI_CALCULATOR_POPUP_RECT.node(),
            stretch_image(assets.calculator_popup.clone()),
            GlobalZIndex(EMAIL_UI_POPUP_Z_INDEX),
            Pickable::IGNORE,
        ))
        .with_children(|popup| {
            spawn_text(
                popup,
                EmailUiRect::new(20.0, 15.0, 140.0, 20.0),
                None,
                LocalizedText::new("ui.email.add_taros", "ADD TAROS"),
                assets,
                EmailUiTextStyle::LabelUpperLeft,
                Color::WHITE,
            );
            spawn_text(
                popup,
                EmailUiRect::new(8.0, 47.0, 188.0, 20.0),
                None,
                LocalizedText::new("ui.email.calculator.amount_to_add", "Amount to Add"),
                assets,
                EmailUiTextStyle::CenterLabel,
                Color::WHITE,
            );
            spawn_image_button(
                popup,
                EmailUiButtonKind::CalculatorPopupClose,
                EmailUiRect::new(180.0, 2.0, 30.0, 30.0),
                assets.close.clone(),
            );
            popup
                .spawn((
                    EmailUiRect::new(25.0, 63.0, 136.0, 109.0).node(),
                    stretch_image(assets.calculator_pad.clone()),
                    Pickable::IGNORE,
                ))
                .with_children(|pad| {
                    spawn_text(
                        pad,
                        EmailUiRect::new(1.0, 1.0, 132.0, 20.0),
                        Some(EmailUiTextRole::CalculatorValue),
                        email_calculator_value_text(0),
                        assets,
                        EmailUiTextStyle::RightLabel,
                        Color::WHITE,
                    );
                    for (digit, rect) in [
                        (1, EmailUiRect::new(1.0, 29.0, 44.0, 19.0)),
                        (2, EmailUiRect::new(46.0, 29.0, 44.0, 19.0)),
                        (3, EmailUiRect::new(91.0, 29.0, 44.0, 19.0)),
                        (4, EmailUiRect::new(1.0, 49.0, 44.0, 19.0)),
                        (5, EmailUiRect::new(46.0, 49.0, 44.0, 19.0)),
                        (6, EmailUiRect::new(91.0, 49.0, 44.0, 19.0)),
                        (7, EmailUiRect::new(1.0, 69.0, 44.0, 19.0)),
                        (8, EmailUiRect::new(46.0, 69.0, 44.0, 19.0)),
                        (9, EmailUiRect::new(91.0, 69.0, 44.0, 19.0)),
                        (0, EmailUiRect::new(46.0, 89.0, 44.0, 19.0)),
                    ] {
                        spawn_transparent_button(
                            pad,
                            EmailUiButtonKind::CalculatorDigit(digit),
                            rect,
                            email_calculator_digit_text(digit),
                            assets,
                            EmailUiTextStyle::CalculatorButton,
                        );
                    }
                    spawn_transparent_button(
                        pad,
                        EmailUiButtonKind::CalculatorClear,
                        EmailUiRect::new(1.0, 89.0, 44.0, 19.0),
                        LocalizedText::new("ui.email.calculator.clear", "C"),
                        assets,
                        EmailUiTextStyle::CalculatorButton,
                    );
                    spawn_transparent_button(
                        pad,
                        EmailUiButtonKind::CalculatorBlank,
                        EmailUiRect::new(91.0, 89.0, 44.0, 19.0),
                        email_passthrough_text(""),
                        assets,
                        EmailUiTextStyle::CalculatorButton,
                    );
                });
            spawn_button(
                popup,
                EmailUiButtonKind::CalculatorAccept,
                EmailUiRect::new(30.0, 200.0, 132.0, 27.0),
                LocalizedText::new("ui.email.calculator.add", "ADD"),
                sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
                assets,
                EmailUiTextStyle::Button,
            );
        });
}

pub(super) fn spawn_email_right_panel(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    parent
        .spawn((
            EmailUiRightPanel,
            EMAIL_UI_RIGHT_PANEL_RECT.node(),
            UiTransform::default(),
            sliced_image(assets.right_panel.clone(), EMAIL_UI_RIGHT_PANEL_BORDER),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    EmailUiRect::new(0.0, 0.0, 380.0, 600.0).node(),
                    sliced_image(
                        assets.inventory_panel.clone(),
                        EMAIL_UI_INVENTORY_PANEL_BORDER,
                    ),
                    Pickable::IGNORE,
                ))
                .with_children(|inventory| {
                    inventory
                        .spawn((
                            Node {
                                overflow: Overflow::clip(),
                                ..EMAIL_UI_INVENTORY_VIEWPORT_RECT.node()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|viewport| {
                            for slot in 0..EMAIL_INVENTORY_SLOT_COUNT {
                                let column = slot % EMAIL_UI_INVENTORY_COLUMNS;
                                let row = slot / EMAIL_UI_INVENTORY_COLUMNS;
                                viewport
                                    .spawn((
                                        Button,
                                        EmailUiInventorySlot { slot },
                                        EmailUiRect::new(
                                            column as f32 * EMAIL_UI_INVENTORY_SLOT_STRIDE,
                                            row as f32 * EMAIL_UI_INVENTORY_SLOT_STRIDE,
                                            EMAIL_UI_INVENTORY_SLOT_SIZE,
                                            EMAIL_UI_INVENTORY_SLOT_SIZE,
                                        )
                                        .node(),
                                        stretch_image(assets.slot_empty.clone()),
                                    ))
                                    .with_children(|slot_node| {
                                        slot_node.spawn((
                                            EmailUiInventoryIcon { slot },
                                            EmailUiRect::new(3.0, 3.0, 61.0, 61.0).node(),
                                            stretch_image(assets.slot_empty.clone()),
                                            Pickable::IGNORE,
                                        ));
                                    });
                            }
                        });
                });
            spawn_image_button(
                panel,
                EmailUiButtonKind::RightClose,
                EMAIL_UI_RIGHT_CLOSE_RECT,
                assets.close.clone(),
            );
        });
}

pub(super) fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    rect: EmailUiRect,
    role: Option<EmailUiTextRole>,
    localized: LocalizedText,
    assets: &EmailUiAssets,
    style: EmailUiTextStyle,
    color: Color,
) {
    spawn_text_with(parent, rect, role, localized, assets, style, color, ());
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_text_with(
    parent: &mut ChildSpawnerCommands,
    rect: EmailUiRect,
    role: Option<EmailUiTextRole>,
    localized: LocalizedText,
    assets: &EmailUiAssets,
    style: EmailUiTextStyle,
    color: Color,
    marker: impl Bundle,
) {
    if role == Some(EmailUiTextRole::DetailBody) {
        body_scroll::spawn(parent, rect, localized, assets, style, color);
        return;
    }
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent
        .spawn((node, Pickable::IGNORE, marker))
        .with_children(|text| {
            spawn_text_entity(
                text,
                role,
                localized,
                assets,
                style,
                color,
                true,
                Some(style.content_bounds(rect)),
            );
        });
}

pub(super) fn spawn_text_entity(
    parent: &mut ChildSpawnerCommands,
    role: Option<EmailUiTextRole>,
    localized: LocalizedText,
    assets: &EmailUiAssets,
    style: EmailUiTextStyle,
    color: Color,
    fill_width: bool,
    fit_bounds: Option<Vec2>,
) {
    let value = email_fallback_text(&localized);
    let font = style.font(assets);
    let mut entity = parent.spawn((
        Node {
            width: if fill_width { percent(100) } else { Val::Auto },
            flex_shrink: 0.0,
            ..default()
        },
        Text::new(value),
        localized,
        style,
        font.clone(),
        TextColor(color),
        style.layout(),
        UiTransform::from_translation(Val2::px(0.0, style.y_offset())),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    if let Some(role) = role {
        entity.insert(EmailUiTextElement { role });
        if matches!(role, EmailUiTextRole::RowSubject(_)) {
            entity.insert(LocalizedTextLimit(24));
        }
    }
    if let Some(bounds) = fit_bounds {
        entity.insert(UiTextAutoFit::new(bounds.x, bounds.y, &font));
    }
}

pub(super) fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    kind: EmailUiButtonKind,
    rect: EmailUiRect,
    localized: LocalizedText,
    image: ImageNode,
    assets: &EmailUiAssets,
    style: EmailUiTextStyle,
) {
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent
        .spawn((Button, EmailUiButton { kind }, node, image))
        .with_children(|button| {
            spawn_text_entity(
                button,
                None,
                localized,
                assets,
                style,
                Color::WHITE,
                true,
                Some(style.content_bounds(rect)),
            );
        });
}

pub(super) fn spawn_transparent_button(
    parent: &mut ChildSpawnerCommands,
    kind: EmailUiButtonKind,
    rect: EmailUiRect,
    localized: LocalizedText,
    assets: &EmailUiAssets,
    style: EmailUiTextStyle,
) {
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent
        .spawn((
            Button,
            EmailUiButton { kind },
            node,
            BackgroundColor(Color::NONE),
        ))
        .with_children(|button| {
            spawn_text_entity(
                button,
                None,
                localized,
                assets,
                style,
                Color::WHITE,
                true,
                Some(style.content_bounds(rect)),
            );
        });
}

pub(super) fn spawn_hit_button(parent: &mut ChildSpawnerCommands, kind: EmailUiButtonKind, rect: EmailUiRect) {
    parent.spawn((
        Button,
        EmailUiButton { kind },
        rect.node(),
        BackgroundColor(Color::NONE),
    ));
}

pub(super) fn spawn_image_button(
    parent: &mut ChildSpawnerCommands,
    kind: EmailUiButtonKind,
    rect: EmailUiRect,
    image: Handle<Image>,
) {
    let mut button = parent.spawn((
        Button,
        EmailUiButton { kind },
        rect.node(),
        stretch_image(image),
    ));
    if kind == EmailUiButtonKind::PlayerTab {
        button.insert(crate::ui::shared::controller::ControllerUiTab(1));
    }
}
