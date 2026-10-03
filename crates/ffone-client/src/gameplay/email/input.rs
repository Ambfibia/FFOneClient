use super::*;

pub(super) fn read_fixed_utf16(
    packet_id: u32,
    body: &[u8],
    offset: usize,
    units: usize,
    field: &'static str,
) -> Result<String, EmailDecodeError0104> {
    let mut encoded = Vec::with_capacity(units);
    for index in 0..units {
        let unit = read_u16(body, offset + index * 2);
        if unit == 0 {
            break;
        }
        encoded.push(unit);
    }
    String::from_utf16(&encoded)
        .map_err(|_| EmailDecodeError0104::InvalidUtf16 { packet_id, field })
}

pub(super) fn read_outgoing_item(body: &[u8], offset: usize) -> EmailOutgoingItem {
    EmailOutgoingItem {
        inventory_slot: read_i32(body, offset),
        item: read_wire_item(body, offset + 4),
    }
}

pub(super) fn read_email_indices(body: &[u8]) -> [i64; EMAIL_DELETE_BATCH_COUNT] {
    array::from_fn(|index| read_i64(body, index * 8))
}

pub(super) fn read_summary(
    packet_id: u32,
    body: &[u8],
    offset: usize,
) -> Result<EmailSummary, EmailDecodeError0104> {
    Ok(EmailSummary {
        email_index: read_i64(body, offset),
        from_pc_uid: read_i64(body, offset + 8),
        first_name: read_fixed_utf16(
            packet_id,
            body,
            offset + 16,
            EMAIL_FIRST_NAME_UNITS,
            "page-list first name",
        )?,
        last_name: read_fixed_utf16(
            packet_id,
            body,
            offset + 34,
            EMAIL_LAST_NAME_UNITS,
            "page-list last name",
        )?,
        subject: read_fixed_utf16(
            packet_id,
            body,
            offset + 68,
            EMAIL_SUBJECT_UNITS,
            "page-list subject",
        )?,
        read_flag: read_i32(body, offset + 132),
        send_time: read_system_time(body, offset + 136),
        delete_time: read_system_time(body, offset + 168),
        item_cash_flag: read_i32(body, offset + 200),
    })
}

pub(super) fn read_system_time(body: &[u8], offset: usize) -> EmailSystemTime {
    EmailSystemTime {
        year: read_i32(body, offset),
        month: read_i32(body, offset + 4),
        day_of_week: read_i32(body, offset + 8),
        day: read_i32(body, offset + 12),
        hour: read_i32(body, offset + 16),
        minute: read_i32(body, offset + 20),
        second: read_i32(body, offset + 24),
        milliseconds: read_i32(body, offset + 28),
    }
}

pub(super) fn read_u16(body: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        body[offset..offset + 2]
            .try_into()
            .expect("validated packet"),
    )
}

pub(super) fn read_i16(body: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(
        body[offset..offset + 2]
            .try_into()
            .expect("validated packet"),
    )
}

pub(super) fn read_i32(body: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        body[offset..offset + 4]
            .try_into()
            .expect("validated packet"),
    )
}

pub(super) fn read_i64(body: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(
        body[offset..offset + 8]
            .try_into()
            .expect("validated packet"),
    )
}
