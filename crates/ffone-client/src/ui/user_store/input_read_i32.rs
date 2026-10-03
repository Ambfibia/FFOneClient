
pub(super) fn read_i32(payload: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        payload[offset..offset + 4]
            .try_into()
            .expect("four-byte slice"),
    )
}

pub(super) fn read_f32(payload: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(
        payload[offset..offset + 4]
            .try_into()
            .expect("four-byte slice"),
    )
}
