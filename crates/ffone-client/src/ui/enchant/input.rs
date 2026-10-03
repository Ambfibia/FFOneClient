use super::*;

pub(super) fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("four checked bytes"),
    )
}

pub(super) fn read_item_base(bytes: &[u8], offset: usize) -> ItemBase0104 {
    ItemBase0104 {
        item_type: i16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("two checked bytes"),
        ),
        item_id: i16::from_le_bytes(
            bytes[offset + 2..offset + 4]
                .try_into()
                .expect("two checked bytes"),
        ),
        option: read_i32(bytes, offset + 4),
        time_limit: read_i32(bytes, offset + 8),
    }
}
