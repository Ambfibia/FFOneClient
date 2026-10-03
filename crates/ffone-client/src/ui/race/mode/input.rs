
pub(super) fn read_i16(payload: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(payload[offset..offset + 2].try_into().expect("two bytes"))
}

pub(super) fn read_i32(payload: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(payload[offset..offset + 4].try_into().expect("four bytes"))
}

pub(super) fn read_u64(payload: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(payload[offset..offset + 8].try_into().expect("eight bytes"))
}
