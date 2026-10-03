use super::*;

pub(super) fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_item_base(bytes: &mut [u8], offset: usize, item: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&item.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&item.item_id.to_le_bytes());
    write_i32(bytes, offset + 4, item.option);
    write_i32(bytes, offset + 8, item.time_limit);
}
