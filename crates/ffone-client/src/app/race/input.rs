
pub(in super::super) fn read_race_i32(payload: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        payload[offset..offset + 4]
            .try_into()
            .expect("race i32 offset was size-checked"),
    )
}
