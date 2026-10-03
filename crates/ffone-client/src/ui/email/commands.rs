use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailRequest {
    UpdateCheck,
    Read {
        email_index: i64,
    },
    PageList {
        page: i8,
    },
    Delete {
        email_indices: [i64; EMAIL_DELETE_BATCH_COUNT],
    },
    Send {
        recipient_pc_uid: i64,
        subject: String,
        content: String,
        items: [EmailOutgoingItem; EMAIL_ATTACHMENT_COUNT],
        cash: i32,
    },
    ReceiveItem {
        email_index: i64,
        inventory_slot: i32,
        email_item_slot: i32,
    },
    ReceiveCash {
        email_index: i64,
    },
    ReceiveAllItems {
        email_index: i64,
    },
}

impl EmailRequest {
    #[must_use]
    pub const fn packet_id(&self) -> u32 {
        match self {
            Self::UpdateCheck => EMAIL_REQ_UPDATE_CHECK_ID,
            Self::Read { .. } => EMAIL_REQ_READ_ID,
            Self::PageList { .. } => EMAIL_REQ_PAGE_LIST_ID,
            Self::Delete { .. } => EMAIL_REQ_DELETE_ID,
            Self::Send { .. } => EMAIL_REQ_SEND_ID,
            Self::ReceiveItem { .. } => EMAIL_REQ_RECEIVE_ITEM_ID,
            Self::ReceiveCash { .. } => EMAIL_REQ_RECEIVE_CASH_ID,
            Self::ReceiveAllItems { .. } => EMAIL_REQ_RECEIVE_ALL_ID,
        }
    }

    #[must_use]
    pub const fn body_size(&self) -> usize {
        match self {
            Self::UpdateCheck => EMAIL_REQ_UPDATE_CHECK_SIZE,
            Self::Read { .. } => EMAIL_REQ_READ_SIZE,
            Self::PageList { .. } => EMAIL_REQ_PAGE_LIST_SIZE,
            Self::Delete { .. } => EMAIL_REQ_DELETE_SIZE,
            Self::Send { .. } => EMAIL_REQ_SEND_SIZE,
            Self::ReceiveItem { .. } => EMAIL_REQ_RECEIVE_ITEM_SIZE,
            Self::ReceiveCash { .. } => EMAIL_REQ_RECEIVE_CASH_SIZE,
            Self::ReceiveAllItems { .. } => EMAIL_REQ_RECEIVE_ALL_SIZE,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailReply {
    NewEmail {
        count: i32,
    },
    ReadSuccess(EmailReadMessage),
    ReadFailure {
        email_index: i64,
        error_code: i32,
    },
    PageListSuccess {
        page: i8,
        messages: Vec<EmailSummary>,
    },
    PageListFailure {
        page: i8,
        error_code: i32,
    },
    DeleteSuccess {
        email_indices: [i64; EMAIL_DELETE_BATCH_COUNT],
    },
    DeleteFailure {
        email_indices: [i64; EMAIL_DELETE_BATCH_COUNT],
        error_code: i32,
    },
    SendSuccess {
        recipient_pc_uid: i64,
        authoritative_cash: i32,
        items: [EmailOutgoingItem; EMAIL_ATTACHMENT_COUNT],
    },
    SendFailure {
        recipient_pc_uid: i64,
        error_code: i32,
    },
    ReceiveItemSuccess {
        email_index: i64,
        inventory_slot: i32,
        email_item_slot: i32,
    },
    ReceiveItemFailure {
        email_index: i64,
        inventory_slot: i32,
        email_item_slot: i32,
        error_code: i32,
    },
    ReceiveCashSuccess {
        email_index: i64,
        authoritative_cash: i32,
    },
    ReceiveCashFailure {
        email_index: i64,
        error_code: i32,
    },
    ReceiveAllItemsSuccess {
        email_index: i64,
    },
    ReceiveAllItemsFailure {
        email_index: i64,
        error_code: i32,
    },
}

impl EmailReply {
    #[must_use]
    pub const fn packet_id(&self) -> u32 {
        match self {
            Self::NewEmail { .. } => EMAIL_REP_NEW_ID,
            Self::ReadSuccess(_) => EMAIL_REP_READ_SUCCESS_ID,
            Self::ReadFailure { .. } => EMAIL_REP_READ_FAILURE_ID,
            Self::PageListSuccess { .. } => EMAIL_REP_PAGE_LIST_SUCCESS_ID,
            Self::PageListFailure { .. } => EMAIL_REP_PAGE_LIST_FAILURE_ID,
            Self::DeleteSuccess { .. } => EMAIL_REP_DELETE_SUCCESS_ID,
            Self::DeleteFailure { .. } => EMAIL_REP_DELETE_FAILURE_ID,
            Self::SendSuccess { .. } => EMAIL_REP_SEND_SUCCESS_ID,
            Self::SendFailure { .. } => EMAIL_REP_SEND_FAILURE_ID,
            Self::ReceiveItemSuccess { .. } => EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID,
            Self::ReceiveItemFailure { .. } => EMAIL_REP_RECEIVE_ITEM_FAILURE_ID,
            Self::ReceiveCashSuccess { .. } => EMAIL_REP_RECEIVE_CASH_SUCCESS_ID,
            Self::ReceiveCashFailure { .. } => EMAIL_REP_RECEIVE_CASH_FAILURE_ID,
            Self::ReceiveAllItemsSuccess { .. } => EMAIL_REP_RECEIVE_ALL_SUCCESS_ID,
            Self::ReceiveAllItemsFailure { .. } => EMAIL_REP_RECEIVE_ALL_FAILURE_ID,
        }
    }

    #[must_use]
    pub const fn body_size(&self) -> usize {
        match self {
            Self::NewEmail { .. } => EMAIL_REP_NEW_SIZE,
            Self::ReadSuccess(_) => EMAIL_REP_READ_SUCCESS_SIZE,
            Self::ReadFailure { .. } => EMAIL_REP_READ_FAILURE_SIZE,
            Self::PageListSuccess { .. } => EMAIL_REP_PAGE_LIST_SUCCESS_SIZE,
            Self::PageListFailure { .. } => EMAIL_REP_PAGE_LIST_FAILURE_SIZE,
            Self::DeleteSuccess { .. } => EMAIL_REP_DELETE_SUCCESS_SIZE,
            Self::DeleteFailure { .. } => EMAIL_REP_DELETE_FAILURE_SIZE,
            Self::SendSuccess { .. } => EMAIL_REP_SEND_SUCCESS_SIZE,
            Self::SendFailure { .. } => EMAIL_REP_SEND_FAILURE_SIZE,
            Self::ReceiveItemSuccess { .. } => EMAIL_REP_RECEIVE_ITEM_SUCCESS_SIZE,
            Self::ReceiveItemFailure { .. } => EMAIL_REP_RECEIVE_ITEM_FAILURE_SIZE,
            Self::ReceiveCashSuccess { .. } => EMAIL_REP_RECEIVE_CASH_SUCCESS_SIZE,
            Self::ReceiveCashFailure { .. } => EMAIL_REP_RECEIVE_CASH_FAILURE_SIZE,
            Self::ReceiveAllItemsSuccess { .. } => EMAIL_REP_RECEIVE_ALL_SUCCESS_SIZE,
            Self::ReceiveAllItemsFailure { .. } => EMAIL_REP_RECEIVE_ALL_FAILURE_SIZE,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailUiAction {
    SetCursorLocked(bool),
    StartUiModeSound,
    StopUiModeSound,
    RefreshGuideEmail {
        event_group: u8,
        event_function: u8,
    },
    SetInventoryMailMode {
        event_group: u8,
        event_function: u8,
        value: i32,
    },
    RequestEscapeCloseGate {
        event_group: u8,
        event_function: u8,
    },
    QueryComputressExitGate {
        event_group: u8,
        event_function: u8,
    },
    ExitMode {
        event_group: u8,
        event_function: u8,
    },
    RefreshInventory {
        event_group: u8,
        event_function: u8,
    },
    DetachComposeItems,
    ApplySendSuccessItems([EmailOutgoingItem; EMAIL_ATTACHMENT_COUNT]),
    RemoveBuddyConfirmation {
        message_id: u16,
        first_name: String,
        last_name: String,
    },
    DeleteEmailConfirmation {
        message_id: u16,
        email_index: i64,
    },
    SystemMessage {
        message_id: Option<u16>,
        fallback: String,
        localized: LocalizedText,
    },
    NewEmailCount(i32),
}

#[must_use]
pub fn request_email_close(
    source: EmailCloseSource,
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
) -> bool {
    if !model.visible || model.send_in_flight || model.system_popup_active {
        return false;
    }
    // EmailMode.Update excludes action 17 while NewEmail is enabled: P is
    // ordinary text in the compose form, while Escape still requests exit.
    if source == EmailCloseSource::EmailKey && model.screen == EmailScreen::Compose {
        return false;
    }
    model.popup = EmailPopup::None;
    match source {
        EmailCloseSource::Escape => {
            model.escape_gate_pending = true;
            actions.push(EmailUiAction::RequestEscapeCloseGate {
                event_group: 2,
                event_function: 24,
            });
        }
        EmailCloseSource::EmailKey | EmailCloseSource::RightPanelClose => {
            model.computress_query_pending = true;
            actions.push(EmailUiAction::QueryComputressExitGate {
                event_group: 11,
                event_function: 13,
            });
        }
    }
    true
}

pub fn apply_email_reply(
    reply: EmailReply,
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
) {
    model.send_in_flight = false;
    match reply {
        EmailReply::NewEmail { count } => actions.push(EmailUiAction::NewEmailCount(count)),
        EmailReply::ReadSuccess(read) => {
            model.read_message = Some(read);
            audio.push(EmailUiAudioCue::EmailArrived);
        }
        EmailReply::ReadFailure {
            email_index,
            error_code,
        } => actions.push(email_system_message(
            Some(167),
            LocalizedText::new(
                "ui.email.error.read",
                "Email {email_index} could not be read: {error_code}",
            )
            .with_arg("email_index", email_index.to_string())
            .with_arg("error_code", error_code.to_string()),
        )),
        EmailReply::PageListSuccess { page, messages } => {
            model.player_page = page;
            model.player_messages = messages
                .into_iter()
                .filter(|message| message.email_index != 0)
                .take(EMAIL_PAGE_SIZE)
                .collect();
            model.selected_row = None;
            model.read_message = None;
            if let Some(first) = model.player_messages.first() {
                model.selected_row = Some(0);
                model.send_in_flight = true;
                transport.push(EmailRequest::Read {
                    email_index: first.email_index,
                });
            }
        }
        EmailReply::PageListFailure {
            page,
            error_code: _,
        } => {
            model.player_page = page;
            model.player_messages.clear();
            model.selected_row = None;
            model.read_message = None;
        }
        EmailReply::DeleteSuccess { .. } => {
            model.send_in_flight = true;
            transport.push(EmailRequest::PageList {
                page: model.player_page,
            });
        }
        EmailReply::DeleteFailure { .. } => actions.push(email_system_message(
            Some(168),
            LocalizedText::new("ui.email.error.delete", "Email deletion failed."),
        )),
        EmailReply::SendSuccess {
            authoritative_cash,
            items,
            ..
        } => {
            model.mail_send_in_flight = false;
            model.available_cash = authoritative_cash;
            actions.push(EmailUiAction::ApplySendSuccessItems(items));
            actions.push(EmailUiAction::RefreshInventory {
                event_group: 11,
                event_function: 6,
            });
            actions.push(EmailUiAction::RefreshGuideEmail {
                event_group: 15,
                event_function: 7,
            });
            model.screen = EmailScreen::List;
            model.folder = EmailFolder::Guide;
            model.guide_page = 0;
            model.selected_row = (!model.guide_messages.is_empty()).then_some(0);
            model.read_message = None;
            model.draft = EmailComposeDraft::default();
            audio.push(EmailUiAudioCue::ActionSuccess);
        }
        EmailReply::SendFailure { error_code, .. } => {
            model.mail_send_in_flight = false;
            let (message_id, localized) = send_failure_message(error_code);
            actions.push(email_system_message(message_id, localized));
        }
        EmailReply::ReceiveItemSuccess {
            email_index,
            email_item_slot,
            ..
        } => {
            actions.push(EmailUiAction::RefreshInventory {
                event_group: 11,
                event_function: 6,
            });
            if let Some(read) = model.read_message.as_mut()
                && read.email_index == email_index
                && (1..=EMAIL_ATTACHMENT_COUNT as i32).contains(&email_item_slot)
            {
                read.items[email_item_slot as usize - 1].item_id = 0;
            }
        }
        EmailReply::ReceiveItemFailure { error_code, .. } => {
            let (message_id, localized) = receive_item_failure_message(error_code, "item");
            actions.push(email_system_message(message_id, localized));
        }
        EmailReply::ReceiveCashSuccess {
            email_index,
            authoritative_cash,
        } => {
            if let Some(read) = model.read_message.as_mut()
                && read.email_index == email_index
            {
                read.cash = 0;
            }
            model.available_cash = authoritative_cash;
        }
        EmailReply::ReceiveCashFailure { error_code, .. } => {
            let (message_id, localized) = receive_cash_failure_message(error_code);
            actions.push(email_system_message(message_id, localized));
        }
        EmailReply::ReceiveAllItemsSuccess { email_index } => {
            if let Some(read) = model.read_message.as_mut()
                && read.email_index == email_index
            {
                for item in &mut read.items {
                    item.item_id = 0;
                }
            }
        }
        EmailReply::ReceiveAllItemsFailure { error_code, .. } => {
            let (message_id, localized) = receive_item_failure_message(error_code, "item all");
            actions.push(email_system_message(message_id, localized));
        }
    }
}
