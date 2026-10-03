use super::*;

pub(super) fn write_utf16_at(body: &mut [u8], offset: usize, units: usize, value: &str) {
    for (index, unit) in value
        .encode_utf16()
        .take(units.saturating_sub(1))
        .enumerate()
    {
        body[offset + index * 2..offset + index * 2 + 2].copy_from_slice(&unit.to_le_bytes());
    }
}
