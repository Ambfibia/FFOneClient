use super::*;

pub(super) fn write_item(bytes: &mut [u8], offset: usize, value: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}
