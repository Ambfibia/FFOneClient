use super::*;

#[must_use]
pub fn attach_email_inventory_item(
    inventory_slot: usize,
    attachment_slot: usize,
    model: &mut EmailUiModel,
) -> bool {
    if model.screen != EmailScreen::Compose
        || model.popup != EmailPopup::None
        || !model.input_enabled()
        || !model.compose_controls_open()
        || inventory_slot >= EMAIL_INVENTORY_SLOT_COUNT
        || attachment_slot >= EMAIL_ATTACHMENT_COUNT
        || model.draft.attachments[attachment_slot].is_some()
        || model
            .draft
            .attachments
            .iter()
            .flatten()
            .any(|attachment| attachment.inventory_slot == inventory_slot as i32)
    {
        return false;
    }
    let Some(item) = model
        .inventory
        .get(inventory_slot)
        .and_then(Option::as_ref)
        .map(|slot| slot.item)
        .filter(|item| !item.is_empty())
    else {
        return false;
    };
    model.draft.attachments[attachment_slot] = Some(EmailOutgoingItem {
        inventory_slot: inventory_slot as i32,
        item,
    });
    true
}

#[must_use]
pub fn detach_email_inventory_item(attachment_slot: usize, model: &mut EmailUiModel) -> bool {
    if model.screen != EmailScreen::Compose
        || model.popup != EmailPopup::None
        || !model.input_enabled()
        || !model.compose_controls_open()
        || attachment_slot >= EMAIL_ATTACHMENT_COUNT
        || model.draft.attachments[attachment_slot].is_none()
    {
        return false;
    }
    model.draft.attachments[attachment_slot] = None;
    true
}

pub(super) fn spawn_email_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = EmailUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            EmailUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            GlobalZIndex(EMAIL_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                EmailUiBackground,
                EmailUiRect::new(
                    0.0,
                    0.0,
                    EMAIL_UI_BACKGROUND_WIDTH,
                    EMAIL_UI_BACKGROUND_HEIGHT,
                )
                .node(),
                UiTransform::default(),
                stretch_image(assets.backdrop.clone()),
                Pickable::IGNORE,
            ));
            root.spawn((
                EmailUiLeftBackplate,
                EmailUiRect::new(0.0, 0.0, 585.0, 653.0).node(),
                UiTransform::default(),
                sliced_image(assets.frame.clone(), EMAIL_UI_FRAME_BORDER),
                Pickable::IGNORE,
            ));
            root.spawn((
                EmailUiRightBackplate,
                EmailUiRect::new(0.0, 0.0, 451.0, 653.0).node(),
                UiTransform::default(),
                sliced_image(assets.right_panel.clone(), EMAIL_UI_RIGHT_PANEL_BORDER),
                Pickable::IGNORE,
            ));
            spawn_email_list_panel(root, &assets);
            spawn_email_compose_panel(root, &assets);
            spawn_email_right_panel(root, &assets);
        });
}

pub(super) fn spawn_email_list_panel(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    parent
        .spawn((
            EmailUiListPanel,
            EMAIL_UI_LIST_WINDOW_RECT.node(),
            UiTransform::default(),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            spawn_button(
                panel,
                EmailUiButtonKind::SendMail,
                EMAIL_UI_SEND_MAIL_RECT,
                LocalizedText::new("ui.email.send_mail", "SEND MAIL"),
                sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
                assets,
                EmailUiTextStyle::Button,
            );
            panel
                .spawn((EMAIL_UI_LIST_INNER_RECT.node(), Pickable::IGNORE))
                .with_children(|inner| {
                    inner
                        .spawn((EMAIL_UI_LIST_BACK_RECT.node(), Pickable::IGNORE))
                        .with_children(|list| {
                            list.spawn((
                                EMAIL_UI_INACTIVE_TAB_FILL_RECT.node(),
                                stretch_image(assets.inactive_tab_fill.clone()),
                                Pickable::IGNORE,
                            ));
                            // The player tab tucks under the list's top rim and
                            // the guide tab. Keep its hover/selected art in the
                            // same layer instead of painting over that rim.
                            spawn_image_button(
                                list,
                                EmailUiButtonKind::PlayerTab,
                                EMAIL_UI_PLAYER_TAB_RECT,
                                assets.player_tab.clone(),
                            );
                            list.spawn((
                                EmailUiRect::new(0.0, 0.0, 561.0, 229.0).node(),
                                stretch_image(assets.list.clone()),
                                Pickable::IGNORE,
                            ));
                            list.spawn((
                                Button,
                                EmailUiButton {
                                    kind: EmailUiButtonKind::GuideTab,
                                },
                                crate::ui::shared::controller::ControllerUiTab(0),
                                EMAIL_UI_GUIDE_TAB_RECT.node(),
                                stretch_image(assets.guide_tab.clone()),
                                ZIndex(0),
                            ));
                            spawn_text(
                                list,
                                EMAIL_UI_GUIDE_TAB_HIT_RECT,
                                None,
                                LocalizedText::new("ui.email.folder.guide", "GUIDE EMAIL"),
                                assets,
                                EmailUiTextStyle::BlankBoxUpperLeft,
                                Color::WHITE,
                            );
                            spawn_text(
                                list,
                                EMAIL_UI_PLAYER_TAB_HIT_RECT,
                                None,
                                LocalizedText::new("ui.email.folder.player", "PLAYER EMAIL"),
                                assets,
                                EmailUiTextStyle::BlankBoxUpperLeft,
                                Color::WHITE,
                            );
                            spawn_static_list_headers(list, assets);
                            for row in 0..EMAIL_PAGE_SIZE {
                                let top = EMAIL_UI_ROW_SELECTION_RECT.top + row as f32 * 28.0;
                                list.spawn((
                                    EmailUiRowSelection { row },
                                    EmailUiRect::new(2.0, top, 556.0, 23.0).node(),
                                    stretch_image(assets.selection.clone()),
                                    Pickable::IGNORE,
                                ));
                                list.spawn((
                                    Button,
                                    EmailUiRowButton { row },
                                    EmailUiRect::new(1.0, 58.0 + row as f32 * 28.0, 556.0, 28.0)
                                        .node(),
                                    BackgroundColor(Color::NONE),
                                ));
                                list.spawn((
                                    EmailUiRowAttachment { row },
                                    EmailUiRect::new(530.0, 65.5 + row as f32 * 28.0, 19.0, 13.0)
                                        .node(),
                                    stretch_image(assets.attachment.clone()),
                                    Pickable::IGNORE,
                                ));
                                spawn_text(
                                    list,
                                    EmailUiRect::new(3.0, 58.0 + row as f32 * 28.0, 199.0, 28.0),
                                    Some(EmailUiTextRole::RowFrom(row)),
                                    email_passthrough_text(""),
                                    assets,
                                    EmailUiTextStyle::BlankBoxUpperLeft,
                                    Color::WHITE,
                                );
                                spawn_text(
                                    list,
                                    EmailUiRect::new(204.0, 58.0 + row as f32 * 28.0, 217.0, 28.0),
                                    Some(EmailUiTextRole::RowSubject(row)),
                                    email_passthrough_text(""),
                                    assets,
                                    EmailUiTextStyle::BlankBoxUpperLeft,
                                    Color::WHITE,
                                );
                                spawn_text(
                                    list,
                                    EmailUiRect::new(421.0, 58.0 + row as f32 * 28.0, 100.0, 28.0),
                                    Some(EmailUiTextRole::RowDate(row)),
                                    email_passthrough_text(""),
                                    assets,
                                    EmailUiTextStyle::BlankBoxUpperCenter,
                                    Color::WHITE,
                                );
                            }
                            spawn_text(
                                list,
                                EmailUiRect::new(0.0, 58.0, 560.0, 115.0),
                                Some(EmailUiTextRole::FolderEmpty),
                                email_passthrough_text(""),
                                assets,
                                EmailUiTextStyle::ChaletLabelMiddleCenter,
                                Color::srgb(0.8, 1.0, 1.0),
                            );
                            spawn_image_button(
                                list,
                                EmailUiButtonKind::Previous,
                                EMAIL_UI_PREVIOUS_RECT,
                                assets.previous.clone(),
                            );
                            spawn_image_button(
                                list,
                                EmailUiButtonKind::Next,
                                EMAIL_UI_NEXT_RECT,
                                assets.next.clone(),
                            );
                            spawn_text(
                                list,
                                EMAIL_UI_PREVIOUS_RECT,
                                None,
                                LocalizedText::new("ui.email.previous_five", "PREVIOUS 5"),
                                assets,
                                EmailUiTextStyle::MailPageButton,
                                Color::WHITE,
                            );
                            spawn_text(
                                list,
                                EMAIL_UI_NEXT_RECT,
                                None,
                                LocalizedText::new("ui.email.next_five", "NEXT 5"),
                                assets,
                                EmailUiTextStyle::MailPageButton,
                                Color::WHITE,
                            );
                            spawn_text(
                                list,
                                EMAIL_UI_PAGE_SELECTION_RECT,
                                Some(EmailUiTextRole::PageSelection),
                                email_passthrough_text(""),
                                assets,
                                EmailUiTextStyle::CenterLabel,
                                Color::srgb(1.0, 1.0, 0.0),
                            );
                            spawn_text(
                                list,
                                EMAIL_UI_PAGE_OF_RECT,
                                None,
                                LocalizedText::new("ui.email.page.of", "of"),
                                assets,
                                EmailUiTextStyle::CenterLabel,
                                Color::WHITE,
                            );
                            spawn_text(
                                list,
                                EMAIL_UI_PAGE_NUMBER_RECT,
                                Some(EmailUiTextRole::PageNumber),
                                email_page_number_text(1),
                                assets,
                                EmailUiTextStyle::CenterLabel,
                                Color::srgb(1.0, 1.0, 0.0),
                            );
                            spawn_text(
                                list,
                                EMAIL_UI_PAGE_LABEL_RECT,
                                None,
                                LocalizedText::new("ui.email.page.emails", "Emails"),
                                assets,
                                EmailUiTextStyle::CenterLabel,
                                Color::WHITE,
                            );
                        });
                    inner
                        .spawn((
                            EMAIL_UI_DATA_BACK_RECT.node(),
                            sliced_image(assets.data_back.clone(), EMAIL_UI_DATA_BACK_BORDER),
                            Pickable::IGNORE,
                        ))
                        .with_children(|detail| spawn_email_detail(detail, assets));
                });
        });
}

pub(super) fn spawn_static_list_headers(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    for (rect, key, label) in [
        (EMAIL_UI_FROM_TITLE_RECT, "ui.email.column.from", "FROM"),
        (
            EMAIL_UI_SUBJECT_TITLE_RECT,
            "ui.email.column.subject",
            "SUBJECT",
        ),
        (EMAIL_UI_DATE_TITLE_RECT, "ui.email.column.day", "DAY"),
    ] {
        spawn_text(
            parent,
            rect,
            None,
            LocalizedText::new(key, label),
            assets,
            EmailUiTextStyle::LabelMiddleCenter,
            Color::WHITE,
        );
    }
    parent.spawn((
        EMAIL_UI_CLIP_TITLE_RECT.node(),
        stretch_image(assets.attachment.clone()),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_email_detail(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    // `DoData` labels the slot backdrop, then the sender icon, before its text.
    for layer in [EmailUiGuideIcon::SlotBackdrop, EmailUiGuideIcon::Sender] {
        parent.spawn((
            Node {
                display: Display::None,
                ..EMAIL_UI_DETAIL_GUIDE_ICON_IMAGE_RECT.node()
            },
            stretch_image(match layer {
                EmailUiGuideIcon::SlotBackdrop => assets.slot_empty.clone(),
                EmailUiGuideIcon::Sender => Handle::default(),
            }),
            layer,
            Pickable::IGNORE,
        ));
    }
    spawn_text_with(
        parent,
        EMAIL_UI_DETAIL_FROM_RECT,
        None,
        LocalizedText::new("ui.email.detail.from", "FROM:"),
        assets,
        EmailUiTextStyle::LabelUpperLeft,
        Color::srgb(1.0, 1.0, 0.0),
        EmailUiGuideIconShift {
            left: EMAIL_UI_DETAIL_FROM_RECT.left,
        },
    );
    spawn_text_with(
        parent,
        EMAIL_UI_DETAIL_FROM_NAME_RECT,
        Some(EmailUiTextRole::DetailFrom),
        email_passthrough_text(""),
        assets,
        EmailUiTextStyle::ChaletLabelMiddleLeft,
        Color::srgb(0.8, 1.0, 1.0),
        EmailUiGuideIconShift {
            left: EMAIL_UI_DETAIL_FROM_NAME_RECT.left,
        },
    );
    spawn_text_with(
        parent,
        EMAIL_UI_DETAIL_SUBJECT_RECT,
        None,
        LocalizedText::new("ui.email.detail.subject", "SUBJECT:"),
        assets,
        EmailUiTextStyle::LabelUpperLeft,
        Color::srgb(1.0, 1.0, 0.0),
        EmailUiGuideIconShift {
            left: EMAIL_UI_DETAIL_SUBJECT_RECT.left,
        },
    );
    spawn_text_with(
        parent,
        EMAIL_UI_DETAIL_SUBJECT_NAME_RECT,
        Some(EmailUiTextRole::DetailSubject),
        email_passthrough_text(""),
        assets,
        EmailUiTextStyle::ChaletLabelMiddleLeft,
        Color::srgb(0.8, 1.0, 1.0),
        EmailUiGuideIconShift {
            left: EMAIL_UI_DETAIL_SUBJECT_NAME_RECT.left,
        },
    );
    spawn_text(
        parent,
        EMAIL_UI_DETAIL_RECEIVED_RECT,
        None,
        LocalizedText::new("ui.email.detail.received", "RECEIVED:"),
        assets,
        EmailUiTextStyle::LabelUpperLeft,
        Color::srgb(1.0, 1.0, 0.0),
    );
    spawn_text(
        parent,
        EMAIL_UI_DETAIL_RECEIVED_DAY_RECT,
        Some(EmailUiTextRole::DetailReceived),
        email_passthrough_text(""),
        assets,
        EmailUiTextStyle::ChaletLabelMiddleLeft,
        Color::srgb(0.8, 1.0, 1.0),
    );
    parent
        .spawn((
            EMAIL_UI_DETAIL_TEXT_RECT.node(),
            sliced_image(assets.data_box.clone(), EMAIL_UI_DATA_BOX_BORDER),
            Pickable::IGNORE,
        ))
        .with_children(|body| {
            spawn_text(
                body,
                EmailUiRect::new(12.0, 10.0, 510.0, 115.0),
                Some(EmailUiTextRole::DetailBody),
                LocalizedText::new(
                    "ui.email.detail.no_selection",
                    "There is no message selected.",
                ),
                assets,
                EmailUiTextStyle::TextArea,
                Color::srgb(0.1, 0.1, 0.1),
            );
        });
    for slot in 0..EMAIL_ATTACHMENT_COUNT {
        parent.spawn((
            Button,
            EmailUiAttachmentSlot {
                slot,
                outgoing: false,
            },
            EMAIL_UI_DETAIL_ITEM_RECT
                .translated(slot as f32 * EMAIL_UI_DETAIL_ITEM_RECT.width, 0.0)
                .node(),
            stretch_image(assets.slot_empty.clone()),
        ));
    }
    spawn_text(
        parent,
        EMAIL_UI_TAROS_LABEL_RECT,
        Some(EmailUiTextRole::DetailTaros),
        email_detail_taros_text(0),
        assets,
        EmailUiTextStyle::LabelUpperLeft,
        Color::WHITE,
    );
    spawn_button(
        parent,
        EmailUiButtonKind::AcceptAll,
        EMAIL_UI_ACCEPT_ALL_RECT,
        LocalizedText::new("ui.email.accept_all_items", "ACCEPT ALL ITEMS"),
        sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
        assets,
        EmailUiTextStyle::ButtonLabelFont,
    );
    spawn_button(
        parent,
        EmailUiButtonKind::AcceptTaros,
        EMAIL_UI_ACCEPT_TAROS_RECT,
        LocalizedText::new("ui.email.accept_taros", "ACCEPT TAROS"),
        sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
        assets,
        EmailUiTextStyle::ButtonLabelFont,
    );
    spawn_button(
        parent,
        EmailUiButtonKind::RemoveBuddy,
        EMAIL_UI_REMOVE_BUDDY_RECT,
        LocalizedText::new("ui.email.remove_buddy", "REMOVE BUDDY"),
        sliced_image(assets.red_button.clone(), EMAIL_UI_RED_BUTTON_BORDER),
        assets,
        EmailUiTextStyle::RedButtonLabelFont,
    );
    spawn_button(
        parent,
        EmailUiButtonKind::Delete,
        EMAIL_UI_DELETE_RECT,
        LocalizedText::new("ui.email.delete", "DELETE"),
        sliced_image(assets.red_button.clone(), EMAIL_UI_RED_BUTTON_BORDER),
        assets,
        EmailUiTextStyle::RedButtonLabelFont,
    );
    spawn_button(
        parent,
        EmailUiButtonKind::Reply,
        EMAIL_UI_REPLY_RECT,
        LocalizedText::new("ui.email.reply", "REPLY"),
        sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
        assets,
        EmailUiTextStyle::ButtonLabelFont,
    );
}

pub(super) fn spawn_email_compose_panel(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    parent
        .spawn((
            EmailUiComposePanel,
            EMAIL_UI_COMPOSE_WINDOW_RECT.node(),
            UiTransform::default(),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    EMAIL_UI_COMPOSE_INNER_RECT.node(),
                    stretch_image(assets.compose.clone()),
                    Pickable::IGNORE,
                ))
                .with_children(|compose| {
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_TITLE_RECT,
                        None,
                        LocalizedText::new("ui.email.compose.title", "NEW EMAIL"),
                        assets,
                        EmailUiTextStyle::BlankBoxUpperLeft,
                        Color::srgb(0.1, 0.2, 1.0),
                    );
                    spawn_image_button(
                        compose,
                        EmailUiButtonKind::ComposeClose,
                        EMAIL_UI_COMPOSE_CLOSE_RECT,
                        assets.close.clone(),
                    );
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_TO_RECT,
                        None,
                        LocalizedText::new("ui.email.compose.to", "TO:"),
                        assets,
                        EmailUiTextStyle::LabelMiddleRight,
                        Color::WHITE,
                    );
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_TO_NAME_RECT,
                        Some(EmailUiTextRole::ComposeTo),
                        email_passthrough_text(""),
                        assets,
                        EmailUiTextStyle::LabelUpperLeft,
                        Color::srgb(1.0, 1.0, 0.0),
                    );
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_SUBJECT_RECT,
                        None,
                        LocalizedText::new("ui.email.compose.subject", "SUBJECT:"),
                        assets,
                        EmailUiTextStyle::LabelMiddleRight,
                        Color::WHITE,
                    );
                    spawn_hit_button(
                        compose,
                        EmailUiButtonKind::ComposeSubjectField,
                        EMAIL_UI_COMPOSE_SUBJECT_FIELD_RECT,
                    );
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_SUBJECT_FIELD_RECT,
                        Some(EmailUiTextRole::ComposeSubject),
                        email_passthrough_text(""),
                        assets,
                        EmailUiTextStyle::BlankBoxUpperLeft,
                        Color::srgb(0.1, 0.1, 0.1),
                    );
                    spawn_button(
                        compose,
                        EmailUiButtonKind::BuddyList,
                        EMAIL_UI_COMPOSE_BUDDY_RECT,
                        LocalizedText::new("ui.email.buddy_list", "BUDDY LIST"),
                        sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
                        assets,
                        EmailUiTextStyle::ButtonLabelFont,
                    );
                    spawn_hit_button(
                        compose,
                        EmailUiButtonKind::ComposeBodyField,
                        EMAIL_UI_COMPOSE_BODY_FIELD_RECT,
                    );
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_BODY_FIELD_RECT,
                        Some(EmailUiTextRole::ComposeBody),
                        email_passthrough_text(""),
                        assets,
                        EmailUiTextStyle::BlankBoxUpperLeft,
                        Color::srgb(0.1, 0.1, 0.1),
                    );
                    compose.spawn((
                        EMAIL_UI_COMPOSE_ATTACHMENT_ICON_RECT.node(),
                        stretch_image(assets.attachment.clone()),
                        Pickable::IGNORE,
                    ));
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_ATTACHMENT_LABEL_RECT,
                        None,
                        LocalizedText::new("ui.email.compose.attachments", "ATTACHMENTS:"),
                        assets,
                        EmailUiTextStyle::LabelUpperLeft,
                        Color::WHITE,
                    );
                    for slot in 0..EMAIL_ATTACHMENT_COUNT {
                        compose.spawn((
                            Button,
                            EmailUiAttachmentSlot {
                                slot,
                                outgoing: true,
                            },
                            EMAIL_UI_COMPOSE_ITEM_RECT
                                .translated(slot as f32 * EMAIL_UI_COMPOSE_ITEM_RECT.width, 0.0)
                                .node(),
                            stretch_image(assets.slot_empty.clone()),
                        ));
                    }
                    spawn_text(
                        compose,
                        EMAIL_UI_COMPOSE_TAROS_RECT,
                        Some(EmailUiTextRole::ComposeTaros),
                        email_compose_taros_text(0),
                        assets,
                        EmailUiTextStyle::BlankBoxMiddleRight,
                        Color::WHITE,
                    );
                    compose.spawn((
                        EmailUiRect::new(488.0, 389.0, 32.0, 32.0).node(),
                        stretch_image(assets.taros.clone()),
                        Pickable::IGNORE,
                    ));
                    spawn_button(
                        compose,
                        EmailUiButtonKind::AddTaros,
                        EMAIL_UI_COMPOSE_ADD_TAROS_RECT,
                        LocalizedText::new("ui.email.add_taros", "ADD TAROS"),
                        sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
                        assets,
                        EmailUiTextStyle::ButtonLabelFont,
                    );
                    spawn_email_postage(compose, assets);
                    spawn_button(
                        compose,
                        EmailUiButtonKind::ComposeCancel,
                        EMAIL_UI_COMPOSE_CANCEL_RECT,
                        LocalizedText::new("ui.email.cancel", "CANCEL"),
                        sliced_image(assets.red_button.clone(), EMAIL_UI_RED_BUTTON_BORDER),
                        assets,
                        EmailUiTextStyle::RedButtonLabelFont,
                    );
                    spawn_button(
                        compose,
                        EmailUiButtonKind::ComposeSend,
                        EMAIL_UI_COMPOSE_SEND_RECT,
                        LocalizedText::new("ui.email.send", "SEND"),
                        sliced_image(assets.button.clone(), EMAIL_UI_BUTTON_BORDER),
                        assets,
                        EmailUiTextStyle::ButtonLabelFont,
                    );
                });
            spawn_email_buddy_popup(panel, assets);
            spawn_email_calculator_popup(panel, assets);
        });
}

pub(super) fn spawn_email_postage(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    let mut node = EMAIL_UI_COMPOSE_POSTAGE_RECT.node();
    node.justify_content = JustifyContent::Center;
    node.align_items = AlignItems::Center;
    node.column_gap = px(EMAIL_UI_POSTAGE_LABEL_GAP);
    node.overflow = Overflow::clip();
    parent
        .spawn((node, Pickable::IGNORE))
        .with_children(|postage| {
            spawn_text_entity(
                postage,
                None,
                LocalizedText::new("ui.email.compose.postage.label", "postage :"),
                assets,
                EmailUiTextStyle::PostageLabel,
                Color::srgb(0.0, 0.2, 0.5),
                false,
                None,
            );
            spawn_text_entity(
                postage,
                Some(EmailUiTextRole::ComposePostage),
                email_postage_amount_text(EMAIL_BASE_POSTAGE),
                assets,
                EmailUiTextStyle::BlankBoxMiddleLeft,
                Color::srgb(0.0, 0.2, 0.5),
                false,
                None,
            );
        });
}

pub(super) fn spawn_email_buddy_popup(parent: &mut ChildSpawnerCommands, assets: &EmailUiAssets) {
    parent
        .spawn((
            EmailPopup::BuddyList,
            EMAIL_UI_BUDDY_POPUP_RECT.node(),
            stretch_image(assets.buddy_popup.clone()),
            GlobalZIndex(EMAIL_UI_POPUP_Z_INDEX),
            Pickable::IGNORE,
        ))
        .with_children(|popup| {
            spawn_text(
                popup,
                EmailUiRect::new(96.0, 14.0, 130.0, 17.0),
                None,
                LocalizedText::new("ui.email.buddy_list", "BUDDY LIST"),
                assets,
                EmailUiTextStyle::LabelUpperLeft,
                Color::WHITE,
            );
            spawn_image_button(
                popup,
                EmailUiButtonKind::BuddyPopupClose,
                EmailUiRect::new(326.0, 2.0, 30.0, 30.0),
                assets.close.clone(),
            );
            popup
                .spawn((
                    Node {
                        overflow: Overflow::clip(),
                        ..EMAIL_UI_BUDDY_LIST_VIEWPORT_RECT.node()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|viewport| {
                    viewport.spawn((
                        EmailUiBuddyListContent,
                        EmailUiRect::new(0.0, 0.0, EMAIL_UI_BUDDY_LIST_CONTENT_WIDTH, 0.0).node(),
                        Pickable::IGNORE,
                    ));
                });
        });
}
