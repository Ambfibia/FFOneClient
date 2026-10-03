use super::*;

pub(super) fn email_system_message(message_id: Option<u16>, localized: LocalizedText) -> EmailUiAction {
    EmailUiAction::SystemMessage {
        message_id,
        fallback: email_fallback_text(&localized),
        localized,
    }
}

pub fn open_email_ui(
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    guide_messages: Vec<EmailGuideMessage>,
    available_cash: i32,
) {
    let ui_scale_override = model.ui_scale_override;
    let inventory = std::mem::take(&mut model.inventory);
    let buddies = std::mem::take(&mut model.buddies);
    let current_local_time = model.current_local_time;
    *model = EmailUiModel::default();
    model.ui_scale_override = ui_scale_override;
    model.inventory = if inventory.len() == EMAIL_INVENTORY_SLOT_COUNT {
        inventory
    } else {
        vec![None; EMAIL_INVENTORY_SLOT_COUNT]
    };
    model.buddies = buddies;
    model.current_local_time = current_local_time;
    model.visible = true;
    model.right_opening_elapsed_seconds = 0.0;
    model.guide_messages = guide_messages;
    model.available_cash = available_cash;
    model.selected_row = (!model.guide_messages.is_empty()).then_some(0);
    actions.push(EmailUiAction::StartUiModeSound);
    actions.push(EmailUiAction::SetCursorLocked(false));
    actions.push(EmailUiAction::RefreshGuideEmail {
        event_group: 15,
        event_function: 7,
    });
    actions.push(EmailUiAction::SetInventoryMailMode {
        event_group: 11,
        event_function: 0,
        value: 4,
    });
}

#[must_use]
pub fn switch_email_folder(
    folder: EmailFolder,
    model: &mut EmailUiModel,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if model.screen != EmailScreen::List || !model.input_enabled() || model.folder == folder {
        return false;
    }
    model.folder = folder;
    model.selected_row = None;
    model.read_message = None;
    match folder {
        EmailFolder::Guide => {
            model.guide_page = 0;
            model.selected_row = (!model.guide_messages.is_empty()).then_some(0);
        }
        EmailFolder::Player => {
            model.player_page = 1;
            model.player_messages.clear();
            model.send_in_flight = true;
            transport.push(EmailRequest::PageList { page: 1 });
        }
    }
    audio.push(EmailUiAudioCue::TabClick);
    true
}

#[must_use]
pub fn select_email_row(
    row: usize,
    model: &mut EmailUiModel,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if row >= EMAIL_PAGE_SIZE || model.screen != EmailScreen::List || !model.input_enabled() {
        return false;
    }
    match model.folder {
        EmailFolder::Guide => {
            if row >= model.visible_guide_messages().len() {
                return false;
            }
            model.selected_row = Some(row);
            model.read_message = None;
        }
        EmailFolder::Player => {
            let Some(summary) = model.player_messages.get(row) else {
                return false;
            };
            if summary.email_index <= 0 {
                return false;
            }
            model.selected_row = Some(row);
            model.read_message = None;
            model.send_in_flight = true;
            transport.push(EmailRequest::Read {
                email_index: summary.email_index,
            });
        }
    }
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn change_email_page(
    delta: i8,
    model: &mut EmailUiModel,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if !matches!(delta, -1 | 1) || model.screen != EmailScreen::List || !model.input_enabled() {
        return false;
    }
    match model.folder {
        EmailFolder::Guide => {
            let max = model.page_count().saturating_sub(1);
            let next = if delta < 0 {
                model
                    .guide_page
                    .saturating_sub(delta.unsigned_abs() as usize)
            } else {
                model.guide_page.saturating_add(delta as usize).min(max)
            };
            if next == model.guide_page {
                return false;
            }
            model.guide_page = next;
            model.selected_row = (!model.visible_guide_messages().is_empty()).then_some(0);
        }
        EmailFolder::Player => {
            if delta > 0 && !model.player_can_next() {
                return false;
            }
            let next = model
                .player_page
                .saturating_add(delta)
                .clamp(1, EMAIL_PAGE_MAX);
            if next == model.player_page {
                return false;
            }
            model.player_page = next;
            model.player_messages.clear();
            model.selected_row = None;
            model.read_message = None;
            model.send_in_flight = true;
            transport.push(EmailRequest::PageList { page: next });
        }
    }
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn begin_email_compose(
    reply_to: Option<(i64, String)>,
    model: &mut EmailUiModel,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if model.screen != EmailScreen::List
        || !model.input_enabled()
        || model.popup != EmailPopup::None
    {
        return false;
    }
    model.screen = EmailScreen::Compose;
    model.popup = EmailPopup::None;
    model.opening_elapsed_seconds = 0.0;
    model.draft = EmailComposeDraft::default();
    model.compose_focus = EmailComposeFocus::None;
    if let Some((pc_uid, _display_name)) = reply_to {
        if let Some(buddy) = model.buddies.iter().find(|buddy| buddy.pc_uid == pc_uid) {
            model.draft.recipient_pc_uid = buddy.pc_uid;
            model.draft.recipient_name = buddy.selected_recipient_label();
        }
    }
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn select_email_buddy(
    buddy_index: usize,
    model: &mut EmailUiModel,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if model.screen != EmailScreen::Compose
        || model.popup != EmailPopup::BuddyList
        || !model.input_enabled()
        || !model.compose_controls_open()
    {
        return false;
    }
    let Some(buddy) = model.buddies.get(buddy_index) else {
        return false;
    };
    model.draft.recipient_pc_uid = buddy.pc_uid;
    model.draft.recipient_name = buddy.selected_recipient_label();
    model.popup = EmailPopup::None;
    model.compose_focus = EmailComposeFocus::None;
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn commit_email_calculator(model: &mut EmailUiModel, audio: &mut EmailUiAudioOutbox) -> bool {
    if model.screen != EmailScreen::Compose
        || model.popup != EmailPopup::AddTaros
        || !model.input_enabled()
        || !model.compose_controls_open()
    {
        return false;
    }
    model.draft.cash = model.calculator_value.min(i64::from(i32::MAX)) as i32;
    model.calculator_value = 0;
    model.popup = EmailPopup::None;
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn close_email_compose(
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if model.screen != EmailScreen::Compose
        || !model.input_enabled()
        || model.popup != EmailPopup::None
        || !model.compose_controls_open()
    {
        return false;
    }
    actions.push(EmailUiAction::DetachComposeItems);
    model.screen = EmailScreen::List;
    model.folder = EmailFolder::Guide;
    model.guide_page = 0;
    model.selected_row = (!model.guide_messages.is_empty()).then_some(0);
    model.read_message = None;
    model.popup = EmailPopup::None;
    model.compose_focus = EmailComposeFocus::None;
    model.draft = EmailComposeDraft::default();
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn send_composed_email(
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> Result<bool, EmailComposeError> {
    if model.screen != EmailScreen::Compose
        || !model.input_enabled()
        || model.popup != EmailPopup::None
        || !model.compose_controls_open()
    {
        return Ok(false);
    }
    match model.draft.request(model.available_cash) {
        Ok(request) => {
            transport.push(request);
            model.send_in_flight = true;
            model.mail_send_in_flight = true;
            audio.push(EmailUiAudioCue::OutgoingChat);
            audio.push(EmailUiAudioCue::ButtonSound);
            Ok(true)
        }
        Err(error @ EmailComposeError::InsufficientCash { .. }) => {
            let EmailComposeError::InsufficientCash {
                available,
                required,
            } = error
            else {
                unreachable!("matched insufficient-cash compose error")
            };
            actions.push(email_system_message(
                Some(181),
                LocalizedText::new(
                    "ui.email.error.insufficient_cash",
                    "email requires {required} Taros but only {available} are available",
                )
                .with_arg("required", required.to_string())
                .with_arg("available", available.to_string()),
            ));
            audio.push(EmailUiAudioCue::ButtonSound);
            Err(error)
        }
        Err(error) => {
            audio.push(EmailUiAudioCue::ButtonSound);
            Err(error)
        }
    }
}

#[must_use]
pub fn accept_email_item(
    email_item_slot: usize,
    inventory_slot: usize,
    model: &mut EmailUiModel,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if model.screen != EmailScreen::List
        || model.folder != EmailFolder::Player
        || model.popup != EmailPopup::None
        || !model.input_enabled()
        || !model.selected_read_matches()
        || email_item_slot >= EMAIL_ATTACHMENT_COUNT
        || inventory_slot >= EMAIL_INVENTORY_SLOT_COUNT
        || model
            .inventory
            .get(inventory_slot)
            .is_none_or(Option::is_some)
    {
        return false;
    }
    let Some(read) = model.read_message.as_ref() else {
        return false;
    };
    if read.email_index <= 0 || read.items[email_item_slot].is_empty() {
        return false;
    }
    transport.push(EmailRequest::ReceiveItem {
        email_index: read.email_index,
        inventory_slot: inventory_slot as i32,
        email_item_slot: email_item_slot as i32 + 1,
    });
    model.send_in_flight = true;
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn accept_all_email_items(
    free_inventory_slots: usize,
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if model.folder != EmailFolder::Player || !model.input_enabled() {
        return false;
    }
    let Some(read) = model.read_message.as_ref() else {
        return false;
    };
    let item_count = read.items.iter().filter(|item| !item.is_empty()).count();
    if item_count == 0 {
        actions.push(email_system_message(
            Some(179),
            LocalizedText::new(
                "ui.email.message.no_attached_items",
                "There are no attached items.",
            ),
        ));
    } else if free_inventory_slots < item_count {
        actions.push(email_system_message(
            Some(178),
            LocalizedText::new(
                "ui.email.message.not_enough_inventory_space",
                "There is not enough inventory space.",
            ),
        ));
    } else {
        transport.push(EmailRequest::ReceiveAllItems {
            email_index: read.email_index,
        });
        model.send_in_flight = true;
    }
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[must_use]
pub fn accept_email_cash(
    model: &mut EmailUiModel,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    if model.folder != EmailFolder::Player || !model.input_enabled() {
        return false;
    }
    let Some(read) = model.read_message.as_ref() else {
        return false;
    };
    if read.email_index <= 0 || read.cash <= 0 {
        audio.push(EmailUiAudioCue::ButtonSound);
        return true;
    }
    transport.push(EmailRequest::ReceiveCash {
        email_index: read.email_index,
    });
    model.send_in_flight = true;
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

pub fn confirm_delete_email(
    email_index: i64,
    model: &mut EmailUiModel,
    transport: &mut EmailTransportOutbox,
) -> bool {
    if !model.visible || model.send_in_flight || email_index <= 0 {
        return false;
    }
    let mut indices = [0; EMAIL_DELETE_BATCH_COUNT];
    indices[0] = email_index;
    transport.push(EmailRequest::Delete {
        email_indices: indices,
    });
    model.send_in_flight = true;
    true
}

pub(super) fn exit_email_ui(model: &mut EmailUiModel, actions: &mut EmailUiOutbox) {
    model.visible = false;
    model.popup = EmailPopup::None;
    model.send_in_flight = false;
    model.mail_send_in_flight = false;
    actions.push(EmailUiAction::SetInventoryMailMode {
        event_group: 11,
        event_function: 0,
        value: 10,
    });
    actions.push(EmailUiAction::ExitMode {
        event_group: 2,
        event_function: 1,
    });
    actions.push(EmailUiAction::StopUiModeSound);
    actions.push(EmailUiAction::SetCursorLocked(false));
}

pub(super) fn send_failure_message(error_code: i32) -> (Option<u16>, LocalizedText) {
    match error_code {
        1 => (
            None,
            LocalizedText::new(
                "ui.email.error.send.insufficient_taros",
                "ERROR!\nInsufficient taros.",
            ),
        ),
        2 => (
            None,
            LocalizedText::new(
                "ui.email.error.send.invalid_player",
                "ERROR!\nInvalid Player to send email to.",
            ),
        ),
        3 => (
            None,
            LocalizedText::new("ui.email.error.send.fatal", "ERROR!\nFATAL Email ERROR!"),
        ),
        4 => (
            None,
            LocalizedText::new(
                "ui.email.error.send.improper_language",
                "ERROR!\nYour message contains improper language and cannot be sent. Please make corrections and try sending the email again.",
            ),
        ),
        5 => (Some(171), email_passthrough_text("")),
        9 => (
            None,
            LocalizedText::new(
                "ui.email.error.send.time_travel",
                "EMAIL TIME TRAVEL ERROR!\nYou cannot send messages with attachments (items and/or Taros) through time. Please remove any attachments and try again. ",
            ),
        ),
        value => (
            None,
            LocalizedText::new(
                "ui.email.error.send.generic",
                "ERROR!\nMail Error. {error_code}",
            )
            .with_arg("error_code", value.to_string()),
        ),
    }
}

pub(super) fn receive_item_failure_message(error_code: i32, label: &str) -> (Option<u16>, LocalizedText) {
    match error_code {
        1 => (Some(221), email_passthrough_text("")),
        5 => (Some(222), email_passthrough_text("")),
        value => {
            let (key, fallback) = match label {
                "item all" => (
                    "ui.email.error.receive_all_items",
                    "Email item all fail. {error_code}",
                ),
                _ => (
                    "ui.email.error.receive_item",
                    "Email item fail. {error_code}",
                ),
            };
            (
                None,
                LocalizedText::new(key, fallback).with_arg("error_code", value.to_string()),
            )
        }
    }
}

pub(super) fn receive_cash_failure_message(error_code: i32) -> (Option<u16>, LocalizedText) {
    match error_code {
        1 => (Some(225), email_passthrough_text("")),
        5 => (Some(222), email_passthrough_text("")),
        value => (
            None,
            LocalizedText::new(
                "ui.email.error.receive_taros",
                "Email taros fail. {error_code}",
            )
            .with_arg("error_code", value.to_string()),
        ),
    }
}

#[must_use]
pub fn email_compose_open_fraction(elapsed_seconds: f32) -> f32 {
    if !elapsed_seconds.is_finite() || elapsed_seconds >= EMAIL_UI_OPEN_SECONDS {
        return 1.0;
    }
    if elapsed_seconds <= 0.0 {
        return 0.0;
    }
    (elapsed_seconds / EMAIL_UI_OPEN_SECONDS * std::f32::consts::FRAC_PI_2).sin()
}

pub(super) fn truncate_utf16(value: &str, limit: usize) -> String {
    let mut used: usize = 0;
    value
        .chars()
        .take_while(|character| {
            let units = character.len_utf16();
            if used.saturating_add(units) > limit {
                false
            } else {
                used += units;
                true
            }
        })
        .collect()
}

pub(super) fn truncate_legacy_list_text(value: &str) -> String {
    if utf16_units(value) <= 24 {
        value.to_owned()
    } else {
        let mut value = truncate_utf16(value, 21);
        value.push_str("...");
        value
    }
}

pub(super) fn utf16_units(value: &str) -> usize {
    value.encode_utf16().count()
}

pub(super) fn flush_email_transport_outbox_0104(
    bridge: Option<Res<NetworkBridge>>,
    mut runtime: ResMut<EmailNetworkRuntime0104>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut actions: ResMut<EmailUiOutbox>,
) {
    let Some(bridge) = bridge else {
        return;
    };
    if runtime.0.pending().is_some() {
        return;
    }
    let Some(request) = transport.pop() else {
        return;
    };
    let wire = match runtime.0.begin(&request) {
        Ok(wire) => wire,
        Err(error) => {
            actions.push(email_system_message(
                None,
                LocalizedText::new(
                    "ui.email.error.transport_rejected",
                    "Email transport rejected the request: {error}",
                )
                .with_arg("error", error.to_string()),
            ));
            return;
        }
    };
    let registered = match RegisteredGameplayRequest0104::new(wire.packet_id, wire.body) {
        Ok(registered) => registered,
        Err(error) => {
            runtime.0.cancel_pending();
            actions.push(email_system_message(
                None,
                LocalizedText::new(
                    "ui.email.error.packet_not_accepted",
                    "Email packet is not accepted by OpenFusion: {error}",
                )
                .with_arg("error", error.to_string()),
            ));
            return;
        }
    };
    if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(registered)) {
        runtime.0.cancel_pending();
        actions.push(email_system_message(
            None,
            LocalizedText::new(
                "ui.email.error.packet_queue",
                "Email packet could not be queued: {error}",
            )
            .with_arg("error", error.to_string()),
        ));
    }
}

pub(super) fn consume_email_network_inbox_0104(
    mut inbox: ResMut<EmailNetworkInbox0104>,
    mut runtime: ResMut<EmailNetworkRuntime0104>,
    mut model: ResMut<EmailUiModel>,
    mut actions: ResMut<EmailUiOutbox>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
) {
    for frame in inbox.0.drain(..) {
        match runtime.0.accept(frame.packet_type, &frame.payload) {
            Ok(Some(EmailRuntimeDelivery0104::Correlated(reply)))
            | Ok(Some(EmailRuntimeDelivery0104::UnsolicitedNewEmail(reply))) => {
                apply_email_reply(reply, &mut model, &mut actions, &mut transport, &mut audio);
            }
            Ok(None) => {}
            Err(error) => actions.push(email_system_message(
                None,
                LocalizedText::new(
                    "ui.email.error.reply_rejected",
                    "Email reply was rejected: {error}",
                )
                .with_arg("error", error.to_string()),
            )),
        }
    }
}

pub(super) fn rebuild_email_buddy_rows(
    mut commands: Commands,
    mut model: ResMut<EmailUiModel>,
    assets: Res<EmailUiAssets>,
    contents: Query<Entity, With<EmailUiBuddyListContent>>,
    mut cached_buddies: Local<Vec<EmailBuddy>>,
) {
    if cached_buddies.as_slice() == model.buddies.as_slice() {
        return;
    }
    let Ok(content) = contents.single() else {
        return;
    };
    model.buddy_scroll_y = model
        .buddy_scroll_y
        .clamp(0.0, email_buddy_scroll_max(model.buddies.len()));
    let buddies = model.buddies.clone();
    *cached_buddies = buddies.clone();
    commands
        .entity(content)
        .despawn_related::<Children>()
        .with_children(|content| {
            for (row, buddy) in buddies.iter().enumerate() {
                spawn_email_buddy_row(content, row, buddy, &assets);
            }
        });
}

pub(super) fn pressed_email_item_source(
    model: &EmailUiModel,
    inventory_slots: &Query<(&Interaction, &EmailUiInventorySlot)>,
    attachment_slots: &Query<(&Interaction, &EmailUiAttachmentSlot)>,
) -> Option<EmailUiItemDragSource> {
    let mut source = None;
    let mut ambiguous = false;
    let mut note = |candidate| {
        if source.replace(candidate).is_some() {
            ambiguous = true;
        }
    };
    for (interaction, slot) in inventory_slots {
        if *interaction == Interaction::Pressed && email_inventory_source_valid(model, slot.slot) {
            note(EmailUiItemDragSource::Inventory(slot.slot));
        }
    }
    for (interaction, slot) in attachment_slots {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if slot.outgoing && email_outgoing_attachment_source_valid(model, slot.slot) {
            note(EmailUiItemDragSource::OutgoingAttachment(slot.slot));
        } else if !slot.outgoing && email_incoming_attachment_source_valid(model, slot.slot) {
            note(EmailUiItemDragSource::IncomingAttachment(slot.slot));
        }
    }
    (!ambiguous).then_some(source).flatten()
}

pub(super) fn email_outgoing_attachment_source_valid(model: &EmailUiModel, slot: usize) -> bool {
    model.screen == EmailScreen::Compose
        && model.popup == EmailPopup::None
        && model.compose_controls_open()
        && model
            .draft
            .attachments
            .get(slot)
            .and_then(|item| *item)
            .is_some_and(|item| !item.item.is_empty())
}

pub(super) fn email_incoming_attachment_source_valid(model: &EmailUiModel, slot: usize) -> bool {
    model.screen == EmailScreen::List
        && model.folder == EmailFolder::Player
        && model.popup == EmailPopup::None
        && model.selected_read_matches()
        && model
            .read_message
            .as_ref()
            .and_then(|read| read.items.get(slot))
            .is_some_and(|item| !item.is_empty())
}

pub(super) fn first_free_email_attachment_slot(model: &EmailUiModel) -> Option<usize> {
    model.draft.attachments.iter().position(Option::is_none)
}

pub(super) fn exactly_one<T>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let value = values.next()?;
    values.next().is_none().then_some(value)
}

pub(super) fn email_fallback_text(localized: &LocalizedText) -> String {
    let mut value = localized.fallback.clone();
    for (name, replacement) in &localized.args {
        value = value.replace(&format!("{{{name}}}"), replacement);
    }
    value
}

pub(super) fn email_passthrough_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn email_player_label_text(pc_uid: i64) -> LocalizedText {
    LocalizedText::new("ui.email.player_label", "Player {pc_uid}")
        .with_arg("pc_uid", pc_uid.to_string())
}
