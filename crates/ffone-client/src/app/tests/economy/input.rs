use super::*;

#[test]
fn pc_load_resurrection_item_projection_uses_general_table_and_last_column() {
    let content = runtime_test_mission_content();
    assert_eq!(content.general_item_type(134), Some(10));
    assert_eq!(content.general_item_type(167), Some(10));
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    for (column, item_id) in [(2usize, 134i16), (49, 167)] {
        let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET
            + column * ffone_protocol::ItemBase0104::SIZE;
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&7i16.to_le_bytes());
        load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&item_id.to_le_bytes());
    }

    let inventory = InventoryRuntime0104::from_pc_load(77, &load);
    assert_eq!(
        resolve_runtime_resurrection_item_slot(&inventory, &content),
        Some(49)
    );
}
