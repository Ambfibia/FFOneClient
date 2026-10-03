use super::*;

pub(super) fn write_marshaled_utf16(
    output: &mut Vec<u8>,
    value: &str,
    units: usize,
    field: &'static str,
) -> Result<(), EmailEncodeError0104> {
    if value.contains('\0') {
        return Err(EmailEncodeError0104::InteriorNul { field });
    }
    let mut encoded = value.encode_utf16();
    for _ in 0..units.saturating_sub(1) {
        write_u16(output, encoded.next().unwrap_or(0));
    }
    // `ByValTStr(SizeConst = N)` reserves the final UTF-16 code unit for NUL.
    write_u16(output, 0);
    Ok(())
}

pub(super) fn write_outgoing_item(output: &mut Vec<u8>, item: EmailOutgoingItem) {
    write_i32(output, item.inventory_slot);
    write_wire_item(output, item.item);
}

pub(super) fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn write_i16(output: &mut Vec<u8>, value: i16) {
    output.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn write_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn write_i64(output: &mut Vec<u8>, value: i64) {
    output.extend_from_slice(&value.to_le_bytes());
}
