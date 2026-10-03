use super::*;

pub(super) fn runtime_with(inventory: &[(usize, ItemBase0104)]) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::zeroed();
    for &(slot, value) in inventory {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    InventoryRuntime0104::from_pc_load(OWNER_PC_ID, &load)
}
