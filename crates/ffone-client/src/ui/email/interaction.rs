use super::*;

pub const EMAIL_SUBJECT_INPUT_LIMIT: usize = 24;

pub const EMAIL_CONTENT_INPUT_LIMIT: usize = 512;

pub const EMAIL_UI_GUIDE_TAB_HOVER_PATH: &str = "ui/en/email/guide-tab-hover.png";

pub const EMAIL_UI_PLAYER_TAB_HOVER_PATH: &str = "ui/en/email/player-tab-hover.png";

pub const EMAIL_UI_PREVIOUS_HOVER_PATH: &str = "ui/en/email/previous-hover.png";

pub const EMAIL_UI_NEXT_HOVER_PATH: &str = "ui/en/email/next-hover.png";

pub const EMAIL_UI_BUTTON_PATH: &str = "ui/en/email/button.png";

pub const EMAIL_UI_BUTTON_HOVER_PATH: &str = "ui/en/email/button-hover.png";

pub const EMAIL_UI_RED_BUTTON_PATH: &str = "ui/en/email/red-button.png";

pub const EMAIL_UI_RED_BUTTON_HOVER_PATH: &str = "ui/en/email/red-button-hover.png";

pub const EMAIL_UI_CLOSE_HOVER_PATH: &str = "ui/en/email/close-hover.png";

pub const EMAIL_UI_BUTTON_PADDING_LEFT: f32 = 6.0;

pub const EMAIL_UI_BUTTON_PADDING_RIGHT: f32 = 6.0;

pub const EMAIL_UI_BUTTON_PADDING_TOP: f32 = 3.0;

pub const EMAIL_UI_BUTTON_PADDING_BOTTOM: f32 = 3.0;

pub const EMAIL_UI_SHARED_SCROLL_VELOCITY: f32 = 200.0;

pub const EMAIL_UI_BUDDY_SCROLL_STEP_LIMIT: f32 = 30.0;

#[must_use]
pub fn email_buddy_scroll_max(buddy_count: usize) -> f32 {
    (EMAIL_UI_BUDDY_ROW_HEIGHT * buddy_count as f32 - EMAIL_UI_BUDDY_LIST_VIEWPORT_RECT.height)
        .max(0.0)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EmailCalculatorInput {
    Digit(u8),
    Clear,
    Blank,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailInputBoundary {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub cursor_locked_while_visible: bool,
    pub cursor_locked_after_exit: bool,
    pub mouse_controls_enabled: bool,
    pub escape_close_gate_enabled: bool,
}

#[must_use]
pub fn input_email_calculator(input: EmailCalculatorInput, model: &mut EmailUiModel) -> bool {
    if model.screen != EmailScreen::Compose
        || model.popup != EmailPopup::AddTaros
        || !model.input_enabled()
        || !model.compose_controls_open()
    {
        return false;
    }
    match input {
        EmailCalculatorInput::Digit(digit) if digit <= 9 => {
            let max = i64::from(model.available_cash.max(0));
            model.calculator_value = model
                .calculator_value
                .saturating_mul(10)
                .saturating_add(i64::from(digit))
                .min(max);
            true
        }
        EmailCalculatorInput::Clear => {
            model.calculator_value = 0;
            true
        }
        EmailCalculatorInput::Blank | EmailCalculatorInput::Digit(_) => false,
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum EmailUiButtonKind {
    SendMail,
    GuideTab,
    PlayerTab,
    Previous,
    Next,
    AcceptAll,
    AcceptTaros,
    RemoveBuddy,
    Delete,
    Reply,
    RightClose,
    ComposeClose,
    ComposeSubjectField,
    ComposeBodyField,
    BuddyList,
    AddTaros,
    ComposeCancel,
    ComposeSend,
    BuddyPopupClose,
    CalculatorPopupClose,
    CalculatorAccept,
    CalculatorDigit(u8),
    CalculatorClear,
    CalculatorBlank,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiButton {
    pub kind: EmailUiButtonKind,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiRowButton {
    pub row: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EmailUiItemDragSource {
    Inventory(usize),
    IncomingAttachment(usize),
    OutgoingAttachment(usize),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub(super) struct EmailUiItemDragState {
    pub(super) source: Option<EmailUiItemDragSource>,
}

pub(super) fn handle_email_keyboard(
    keys: Option<MessageReader<KeyboardInput>>,
    mut model: ResMut<EmailUiModel>,
    mut actions: ResMut<EmailUiOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
) {
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state != ButtonState::Pressed || key.repeat {
            continue;
        }
        if key.key_code == KeyCode::Escape {
            if request_email_close(EmailCloseSource::Escape, &mut model, &mut actions) {
                break;
            }
            continue;
        }
        if model.screen != EmailScreen::Compose
            || model.popup != EmailPopup::None
            || !model.input_enabled()
            || !model.compose_controls_open()
            || model.compose_focus == EmailComposeFocus::None
        {
            continue;
        }
        if key.key_code == KeyCode::Backspace {
            match model.compose_focus {
                EmailComposeFocus::None => {}
                EmailComposeFocus::Subject => {
                    model.draft.subject.pop();
                }
                EmailComposeFocus::Body => {
                    model.draft.content.pop();
                }
            }
            continue;
        }
        if matches!(key.key_code, KeyCode::Enter | KeyCode::NumpadEnter) {
            if model.compose_focus == EmailComposeFocus::Body {
                let mut content = model.draft.content.clone();
                content.push('\n');
                model.draft.set_content(content);
            }
            continue;
        }
        let Some(produced) = key.text.as_deref() else {
            continue;
        };
        let mut subject_limit_reached = false;
        for character in produced.chars() {
            if character.is_control() {
                continue;
            }
            match model.compose_focus {
                EmailComposeFocus::None => {}
                EmailComposeFocus::Subject => {
                    let mut subject = model.draft.subject.clone();
                    subject.push(character);
                    if subject.encode_utf16().count() > EMAIL_SUBJECT_INPUT_LIMIT {
                        subject_limit_reached = true;
                    }
                    model.draft.set_subject(subject);
                }
                EmailComposeFocus::Body => {
                    let mut content = model.draft.content.clone();
                    content.push(character);
                    model.draft.set_content(content);
                }
            }
        }
        if subject_limit_reached {
            audio.push(EmailUiAudioCue::CharacterLimitMax);
        }
    }
}

pub(super) fn handle_email_buddy_scroll(
    scroll: Option<Res<AccumulatedMouseScroll>>,
    rows: Query<&Interaction, With<EmailUiBuddyRow>>,
    mut model: ResMut<EmailUiModel>,
) {
    let Some(scroll) = scroll else {
        return;
    };
    if rows
        .iter()
        .any(|interaction| matches!(interaction, Interaction::Hovered | Interaction::Pressed))
    {
        let _ = model.apply_buddy_scroll_axis(scroll.delta.y);
    }
}

pub(super) fn handle_email_item_interactions(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    inventory_slots: Query<(&Interaction, &EmailUiInventorySlot)>,
    attachment_slots: Query<(&Interaction, &EmailUiAttachmentSlot)>,
    mut drag: ResMut<EmailUiItemDragState>,
    mut model: ResMut<EmailUiModel>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
) {
    let Some(mouse) = mouse else {
        drag.source = None;
        return;
    };
    if !model.input_enabled() {
        drag.source = None;
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        drag.source = pressed_email_item_source(&model, &inventory_slots, &attachment_slots);
    }

    if mouse.just_released(MouseButton::Right) {
        drag.source = None;
        let Some(email_item_slot) = single_hovered_incoming_attachment(&attachment_slots) else {
            return;
        };
        let Some(inventory_slot) = first_free_email_inventory_slot(&model) else {
            return;
        };
        let _ = accept_email_item(
            email_item_slot,
            inventory_slot,
            &mut model,
            &mut transport,
            &mut audio,
        );
        return;
    }

    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    let Some(source) = drag.source.take() else {
        return;
    };
    let handled = match source {
        EmailUiItemDragSource::Inventory(inventory_slot) => {
            if single_hovered_outgoing_attachment(&attachment_slots).is_none() {
                false
            } else if let Some(attachment_slot) = first_free_email_attachment_slot(&model) {
                attach_email_inventory_item(inventory_slot, attachment_slot, &mut model)
            } else {
                false
            }
        }
        EmailUiItemDragSource::OutgoingAttachment(attachment_slot) => {
            if single_hovered_inventory_slot(&inventory_slots).is_some() {
                detach_email_inventory_item(attachment_slot, &mut model)
            } else {
                false
            }
        }
        EmailUiItemDragSource::IncomingAttachment(email_item_slot) => {
            if single_hovered_inventory_slot(&inventory_slots).is_none() {
                false
            } else if let Some(inventory_slot) = first_free_email_inventory_slot(&model) {
                accept_email_item(
                    email_item_slot,
                    inventory_slot,
                    &mut model,
                    &mut transport,
                    &mut audio,
                )
            } else {
                false
            }
        }
    };
    if handled && !matches!(source, EmailUiItemDragSource::IncomingAttachment(_)) {
        audio.push(EmailUiAudioCue::ButtonSound);
    }
}

pub(super) fn single_hovered_inventory_slot(
    slots: &Query<(&Interaction, &EmailUiInventorySlot)>,
) -> Option<usize> {
    exactly_one(slots.iter().filter_map(|(interaction, slot)| {
        (*interaction == Interaction::Hovered).then_some(slot.slot)
    }))
}

pub(super) fn single_hovered_incoming_attachment(
    slots: &Query<(&Interaction, &EmailUiAttachmentSlot)>,
) -> Option<usize> {
    exactly_one(slots.iter().filter_map(|(interaction, slot)| {
        (!slot.outgoing && *interaction == Interaction::Hovered).then_some(slot.slot)
    }))
}

pub(super) fn single_hovered_outgoing_attachment(
    slots: &Query<(&Interaction, &EmailUiAttachmentSlot)>,
) -> Option<usize> {
    exactly_one(slots.iter().filter_map(|(interaction, slot)| {
        (slot.outgoing && *interaction == Interaction::Hovered).then_some(slot.slot)
    }))
}

pub(super) fn handle_email_interactions(
    buttons: Query<(&Interaction, &EmailUiButton), Changed<Interaction>>,
    rows: Query<(&Interaction, &EmailUiRowButton), Changed<Interaction>>,
    buddies: Query<(&Interaction, &EmailUiBuddyRow), Changed<Interaction>>,
    mut model: ResMut<EmailUiModel>,
    mut actions: ResMut<EmailUiOutbox>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
) {
    for (interaction, buddy) in &buddies {
        if *interaction == Interaction::Pressed
            && select_email_buddy(buddy.row, &mut model, &mut audio)
        {
            return;
        }
    }
    for (interaction, row) in &rows {
        if *interaction == Interaction::Pressed
            && select_email_row(row.row, &mut model, &mut transport, &mut audio)
        {
            return;
        }
    }
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if !email_ui_button_enabled(&model, button.kind) {
            continue;
        }
        let handled = match button.kind {
            EmailUiButtonKind::SendMail => begin_email_compose(None, &mut model, &mut audio),
            EmailUiButtonKind::GuideTab => {
                switch_email_folder(EmailFolder::Guide, &mut model, &mut transport, &mut audio)
            }
            EmailUiButtonKind::PlayerTab => {
                switch_email_folder(EmailFolder::Player, &mut model, &mut transport, &mut audio)
            }
            EmailUiButtonKind::Previous => {
                change_email_page(-1, &mut model, &mut transport, &mut audio)
            }
            EmailUiButtonKind::Next => change_email_page(1, &mut model, &mut transport, &mut audio),
            EmailUiButtonKind::AcceptAll => accept_all_email_items(
                model.inventory.iter().filter(|slot| slot.is_none()).count(),
                &mut model,
                &mut actions,
                &mut transport,
                &mut audio,
            ),
            EmailUiButtonKind::AcceptTaros => {
                accept_email_cash(&mut model, &mut transport, &mut audio)
            }
            EmailUiButtonKind::RemoveBuddy => {
                let selected = model
                    .selected_read_matches()
                    .then(|| model.selected_summary().cloned())
                    .flatten();
                if let Some(selected) = selected {
                    actions.push(EmailUiAction::RemoveBuddyConfirmation {
                        message_id: 165,
                        first_name: selected.first_name,
                        last_name: selected.last_name,
                    });
                    audio.push(EmailUiAudioCue::ButtonSound);
                    true
                } else {
                    false
                }
            }
            EmailUiButtonKind::Delete => {
                request_delete_selected_email(&mut model, &mut actions, &mut audio)
            }
            EmailUiButtonKind::Reply => {
                let selected = model.selected_read_matches().then(|| {
                    model
                        .selected_summary()
                        .map(|message| (message.from_pc_uid, message.sender_label()))
                });
                let selected = selected.flatten();
                begin_email_compose(selected, &mut model, &mut audio)
            }
            EmailUiButtonKind::RightClose => {
                request_email_close(EmailCloseSource::RightPanelClose, &mut model, &mut actions)
            }
            EmailUiButtonKind::ComposeClose | EmailUiButtonKind::ComposeCancel => {
                close_email_compose(&mut model, &mut actions, &mut audio)
            }
            EmailUiButtonKind::ComposeSubjectField => {
                if model.screen == EmailScreen::Compose && model.input_enabled() {
                    model.compose_focus = EmailComposeFocus::Subject;
                    true
                } else {
                    false
                }
            }
            EmailUiButtonKind::ComposeBodyField => {
                if model.screen == EmailScreen::Compose && model.input_enabled() {
                    model.compose_focus = EmailComposeFocus::Body;
                    true
                } else {
                    false
                }
            }
            EmailUiButtonKind::BuddyList => {
                if model.input_enabled() {
                    model.popup = EmailPopup::BuddyList;
                    model.compose_focus = EmailComposeFocus::None;
                    audio.push(EmailUiAudioCue::ButtonSound);
                    true
                } else {
                    false
                }
            }
            EmailUiButtonKind::AddTaros => {
                if model.input_enabled() {
                    model.popup = EmailPopup::AddTaros;
                    model.compose_focus = EmailComposeFocus::None;
                    model.calculator_value = 0;
                    audio.push(EmailUiAudioCue::ButtonSound);
                    true
                } else {
                    false
                }
            }
            EmailUiButtonKind::ComposeSend => {
                send_composed_email(&mut model, &mut actions, &mut transport, &mut audio)
                    .unwrap_or(false)
            }
            EmailUiButtonKind::BuddyPopupClose => {
                if model.popup == EmailPopup::BuddyList {
                    model.popup = EmailPopup::None;
                    audio.push(EmailUiAudioCue::ButtonSound);
                    true
                } else {
                    false
                }
            }
            EmailUiButtonKind::CalculatorPopupClose => {
                if model.popup == EmailPopup::AddTaros {
                    model.calculator_value = 0;
                    model.popup = EmailPopup::None;
                    true
                } else {
                    false
                }
            }
            EmailUiButtonKind::CalculatorAccept => commit_email_calculator(&mut model, &mut audio),
            EmailUiButtonKind::CalculatorDigit(digit) => {
                input_email_calculator(EmailCalculatorInput::Digit(digit), &mut model)
            }
            EmailUiButtonKind::CalculatorClear => {
                input_email_calculator(EmailCalculatorInput::Clear, &mut model)
            }
            EmailUiButtonKind::CalculatorBlank => false,
        };
        if handled {
            break;
        }
    }
}

pub(super) fn email_ui_button_enabled(model: &EmailUiModel, kind: EmailUiButtonKind) -> bool {
    if !model.input_enabled()
        || (kind == EmailUiButtonKind::RightClose && !model.right_panel_controls_open())
        || (model.screen == EmailScreen::Compose
            && kind != EmailUiButtonKind::RightClose
            && !model.compose_controls_open())
    {
        return false;
    }
    match model.popup {
        EmailPopup::None => true,
        EmailPopup::BuddyList => matches!(kind, EmailUiButtonKind::BuddyPopupClose),
        EmailPopup::AddTaros => matches!(
            kind,
            EmailUiButtonKind::CalculatorPopupClose
                | EmailUiButtonKind::CalculatorAccept
                | EmailUiButtonKind::CalculatorDigit(_)
                | EmailUiButtonKind::CalculatorClear
                | EmailUiButtonKind::CalculatorBlank
        ),
    }
}

pub(super) fn sync_email_ui_button_visuals(
    assets: Res<EmailUiAssets>,
    model: Res<EmailUiModel>,
    mut buttons: Query<(
        &EmailUiButton,
        &Interaction,
        &mut ImageNode,
        Option<&mut ZIndex>,
    )>,
) {
    for (button, interaction, mut image, z_index) in &mut buttons {
        let hovered = *interaction == Interaction::Hovered
            && model.input_enabled()
            && email_ui_button_enabled(&model, button.kind);
        if button.kind == EmailUiButtonKind::GuideTab {
            if let Some(mut z_index) = z_index {
                z_index.set_if_neq(ZIndex(if hovered || model.folder == EmailFolder::Guide {
                    0
                } else {
                    -1
                }));
            }
        }
        image.image = match button.kind {
            EmailUiButtonKind::GuideTab => {
                if hovered || model.folder == EmailFolder::Guide {
                    assets.guide_tab_hover.clone()
                } else {
                    assets.guide_tab.clone()
                }
            }
            EmailUiButtonKind::PlayerTab => {
                if hovered || model.folder == EmailFolder::Player {
                    assets.player_tab_hover.clone()
                } else {
                    assets.player_tab.clone()
                }
            }
            EmailUiButtonKind::Previous => {
                if hovered {
                    assets.previous_hover.clone()
                } else {
                    assets.previous.clone()
                }
            }
            EmailUiButtonKind::Next => {
                if hovered {
                    assets.next_hover.clone()
                } else {
                    assets.next.clone()
                }
            }
            EmailUiButtonKind::RemoveBuddy
            | EmailUiButtonKind::Delete
            | EmailUiButtonKind::ComposeCancel => {
                if hovered {
                    assets.red_button_hover.clone()
                } else {
                    assets.red_button.clone()
                }
            }
            EmailUiButtonKind::RightClose
            | EmailUiButtonKind::ComposeClose
            | EmailUiButtonKind::BuddyPopupClose
            | EmailUiButtonKind::CalculatorPopupClose => {
                if hovered {
                    assets.close_hover.clone()
                } else {
                    assets.close.clone()
                }
            }
            _ => {
                if hovered {
                    assets.button_hover.clone()
                } else {
                    assets.button.clone()
                }
            }
        };
    }
}
