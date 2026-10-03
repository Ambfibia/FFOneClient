use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailWirePacket0104 {
    pub packet_id: u32,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailEncodeError0104 {
    InteriorNul { field: &'static str },
}

impl fmt::Display for EmailEncodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul { field } => {
                write!(formatter, "EmailMode {field} contains an interior NUL")
            }
        }
    }
}

impl Error for EmailEncodeError0104 {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailDecodeError0104 {
    UnexpectedBodySize {
        packet_id: u32,
        expected: usize,
        actual: usize,
    },
    InvalidUtf16 {
        packet_id: u32,
        field: &'static str,
    },
}

impl fmt::Display for EmailDecodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedBodySize {
                packet_id,
                expected,
                actual,
            } => write!(
                formatter,
                "EmailMode packet {packet_id:#010x} has {actual} bytes; expected {expected}"
            ),
            Self::InvalidUtf16 { packet_id, field } => write!(
                formatter,
                "EmailMode packet {packet_id:#010x} has invalid UTF-16 in {field}"
            ),
        }
    }
}

impl Error for EmailDecodeError0104 {}

/// `EmailMode.Exit` is local and sends no gameplay request.
pub const EMAIL_NORMAL_EXIT_SENDS_PACKET_0104: bool = false;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailFrameDisposition0104 {
    /// The exact frame is returned to the next gameplay owner.
    Passthrough(DecodedFrame),
    /// An owned Email frame was accepted and applied.
    Applied {
        frame: DecodedFrame,
        output: EmailProductionOutput0104,
    },
    /// An owned Email frame was rejected without consuming correlation. The
    /// exact frame is retained for diagnostics/retry policy.
    Rejected {
        frame: DecodedFrame,
        error: EmailProductionError0104,
    },
}

pub fn encode_email_request_0104(
    request: &EmailRequest,
) -> Result<EmailWirePacket0104, EmailEncodeError0104> {
    let mut body = Vec::with_capacity(request.body_size());
    match request {
        EmailRequest::UpdateCheck => {}
        EmailRequest::Read { email_index }
        | EmailRequest::ReceiveCash { email_index }
        | EmailRequest::ReceiveAllItems { email_index } => write_i64(&mut body, *email_index),
        EmailRequest::PageList { page } => body.push(*page as u8),
        EmailRequest::Delete { email_indices } => {
            for email_index in email_indices {
                write_i64(&mut body, *email_index);
            }
        }
        EmailRequest::Send {
            recipient_pc_uid,
            subject,
            content,
            items,
            cash,
        } => {
            write_i64(&mut body, *recipient_pc_uid);
            write_marshaled_utf16(&mut body, subject, EMAIL_SUBJECT_UNITS, "subject")?;
            write_marshaled_utf16(&mut body, content, EMAIL_CONTENT_UNITS, "content")?;
            for item in items {
                write_outgoing_item(&mut body, *item);
            }
            write_i32(&mut body, *cash);
        }
        EmailRequest::ReceiveItem {
            email_index,
            inventory_slot,
            email_item_slot,
        } => {
            write_i64(&mut body, *email_index);
            write_i32(&mut body, *inventory_slot);
            write_i32(&mut body, *email_item_slot);
        }
    }
    debug_assert_eq!(body.len(), request.body_size());
    Ok(EmailWirePacket0104 {
        packet_id: request.packet_id(),
        body,
    })
}

pub fn decode_email_reply_0104(
    packet_id: u32,
    body: &[u8],
) -> Result<Option<EmailReply>, EmailDecodeError0104> {
    let expected = match packet_id {
        EMAIL_REP_NEW_ID => EMAIL_REP_NEW_SIZE,
        EMAIL_REP_READ_SUCCESS_ID => EMAIL_REP_READ_SUCCESS_SIZE,
        EMAIL_REP_READ_FAILURE_ID => EMAIL_REP_READ_FAILURE_SIZE,
        EMAIL_REP_PAGE_LIST_SUCCESS_ID => EMAIL_REP_PAGE_LIST_SUCCESS_SIZE,
        EMAIL_REP_PAGE_LIST_FAILURE_ID => EMAIL_REP_PAGE_LIST_FAILURE_SIZE,
        EMAIL_REP_DELETE_SUCCESS_ID => EMAIL_REP_DELETE_SUCCESS_SIZE,
        EMAIL_REP_DELETE_FAILURE_ID => EMAIL_REP_DELETE_FAILURE_SIZE,
        EMAIL_REP_SEND_SUCCESS_ID => EMAIL_REP_SEND_SUCCESS_SIZE,
        EMAIL_REP_SEND_FAILURE_ID => EMAIL_REP_SEND_FAILURE_SIZE,
        EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID => EMAIL_REP_RECEIVE_ITEM_SUCCESS_SIZE,
        EMAIL_REP_RECEIVE_ITEM_FAILURE_ID => EMAIL_REP_RECEIVE_ITEM_FAILURE_SIZE,
        EMAIL_REP_RECEIVE_CASH_SUCCESS_ID => EMAIL_REP_RECEIVE_CASH_SUCCESS_SIZE,
        EMAIL_REP_RECEIVE_CASH_FAILURE_ID => EMAIL_REP_RECEIVE_CASH_FAILURE_SIZE,
        EMAIL_REP_RECEIVE_ALL_SUCCESS_ID => EMAIL_REP_RECEIVE_ALL_SUCCESS_SIZE,
        EMAIL_REP_RECEIVE_ALL_FAILURE_ID => EMAIL_REP_RECEIVE_ALL_FAILURE_SIZE,
        _ => return Ok(None),
    };
    require_size(packet_id, body, expected)?;

    let reply = match packet_id {
        EMAIL_REP_NEW_ID => EmailReply::NewEmail {
            count: read_i32(body, 0),
        },
        EMAIL_REP_READ_SUCCESS_ID => {
            let email_index = read_i64(body, 0);
            let content = read_fixed_utf16(
                packet_id,
                body,
                8,
                EMAIL_CONTENT_UNITS,
                "read-success content",
            )?;
            let items = array::from_fn(|index| {
                read_wire_item(
                    body,
                    8 + EMAIL_CONTENT_UNITS * 2 + index * EMAIL_ITEM_BASE_SIZE,
                )
            });
            EmailReply::ReadSuccess(EmailReadMessage {
                email_index,
                content,
                items,
                cash: read_i32(body, EMAIL_REP_READ_SUCCESS_SIZE - 4),
            })
        }
        EMAIL_REP_READ_FAILURE_ID => EmailReply::ReadFailure {
            email_index: read_i64(body, 0),
            error_code: read_i32(body, 8),
        },
        EMAIL_REP_PAGE_LIST_SUCCESS_ID => {
            let page = body[0] as i8;
            let mut messages = Vec::with_capacity(5);
            for index in 0..5 {
                messages.push(read_summary(
                    packet_id,
                    body,
                    EMAIL_SUMMARY_FIRST_OFFSET + EMAIL_SUMMARY_SIZE * index,
                )?);
            }
            EmailReply::PageListSuccess { page, messages }
        }
        EMAIL_REP_PAGE_LIST_FAILURE_ID => EmailReply::PageListFailure {
            page: body[0] as i8,
            error_code: read_i32(body, 4),
        },
        EMAIL_REP_DELETE_SUCCESS_ID => EmailReply::DeleteSuccess {
            email_indices: read_email_indices(body),
        },
        EMAIL_REP_DELETE_FAILURE_ID => EmailReply::DeleteFailure {
            email_indices: read_email_indices(body),
            error_code: read_i32(body, EMAIL_REP_DELETE_FAILURE_SIZE - 4),
        },
        EMAIL_REP_SEND_SUCCESS_ID => EmailReply::SendSuccess {
            recipient_pc_uid: read_i64(body, 0),
            authoritative_cash: read_i32(body, 8),
            items: array::from_fn(|index| {
                read_outgoing_item(body, 12 + index * EMAIL_OUTGOING_ITEM_SIZE)
            }),
        },
        EMAIL_REP_SEND_FAILURE_ID => EmailReply::SendFailure {
            recipient_pc_uid: read_i64(body, 0),
            error_code: read_i32(body, 8),
        },
        EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID => EmailReply::ReceiveItemSuccess {
            email_index: read_i64(body, 0),
            inventory_slot: read_i32(body, 8),
            email_item_slot: read_i32(body, 12),
        },
        EMAIL_REP_RECEIVE_ITEM_FAILURE_ID => EmailReply::ReceiveItemFailure {
            email_index: read_i64(body, 0),
            inventory_slot: read_i32(body, 8),
            email_item_slot: read_i32(body, 12),
            error_code: read_i32(body, 16),
        },
        EMAIL_REP_RECEIVE_CASH_SUCCESS_ID => EmailReply::ReceiveCashSuccess {
            email_index: read_i64(body, 0),
            authoritative_cash: read_i32(body, 8),
        },
        EMAIL_REP_RECEIVE_CASH_FAILURE_ID => EmailReply::ReceiveCashFailure {
            email_index: read_i64(body, 0),
            error_code: read_i32(body, 8),
        },
        EMAIL_REP_RECEIVE_ALL_SUCCESS_ID => EmailReply::ReceiveAllItemsSuccess {
            email_index: read_i64(body, 0),
        },
        EMAIL_REP_RECEIVE_ALL_FAILURE_ID => EmailReply::ReceiveAllItemsFailure {
            email_index: read_i64(body, 0),
            error_code: read_i32(body, 8),
        },
        _ => unreachable!("known EmailMode reply id"),
    };
    Ok(Some(reply))
}

pub(super) fn write_wire_item(output: &mut Vec<u8>, item: EmailWireItem) {
    write_i16(output, item.item_type);
    write_i16(output, item.item_id);
    write_i32(output, item.option);
    write_i32(output, item.time_limit);
}

pub(super) fn read_wire_item(body: &[u8], offset: usize) -> EmailWireItem {
    EmailWireItem {
        item_type: read_i16(body, offset),
        item_id: read_i16(body, offset + 2),
        option: read_i32(body, offset + 4),
        time_limit: read_i32(body, offset + 8),
    }
}
