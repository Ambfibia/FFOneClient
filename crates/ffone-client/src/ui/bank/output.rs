use super::*;

pub(super) fn write_item_base_into_pc_load(bytes: &mut [u8], offset: usize, item: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&item.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&item.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&item.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&item.time_limit.to_le_bytes());
}
