
pub(super) fn write_i32(payload: &mut [u8], offset: usize, value: i32) {
    payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
