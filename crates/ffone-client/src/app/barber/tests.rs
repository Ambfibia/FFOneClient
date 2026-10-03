use super::*;
#[test]
fn success_moves_clothes_atomically_and_malformed_slots_leave_authority_untouched() {
    let mut inventory =
        InventoryRuntime0104::from_pc_load(42, &ffone_protocol::PcLoadData0104::zeroed());
    let shirt = ItemBase0104 {
        item_type: 1,
        item_id: 50,
        option: 1,
        time_limit: 123,
    };
    inventory
        .apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: 0,
            from_slot_num: 1,
            from_slot_item: shirt,
            to_location: 0,
            to_slot_num: 1,
            to_slot_item: shirt,
        })
        .unwrap();
    let mut reply = PcBarberConfirmReply0104::decode(&[0; 56]).unwrap();
    reply.taros = 900;
    reply.unequip_slots = [4, -1, -1];
    reply.unequip_items[0] = ffone_protocol::wire_0104::ItemBase0104 {
        type_: 1,
        id: 50,
        opt: 1,
        time_limit: 123,
    };
    let next = committed_inventory(&inventory, &reply).unwrap();
    assert_eq!(next.inventory()[4], shirt);
    assert!(InventoryRuntime0104::item_is_empty(next.equipment()[1]));
    assert_eq!(inventory.equipment()[1], shirt);
    assert!(InventoryRuntime0104::item_is_empty(
        inventory.inventory()[4]
    ));
    reply.unequip_slots[1] = 4;
    reply.unequip_items[1] = reply.unequip_items[0].clone();
    assert!(committed_inventory(&inventory, &reply).is_err());
    assert_eq!(inventory.equipment()[1], shirt);
    reply.unequip_slots = [4, -1, -1];
    reply.error_code = 1;
    assert!(committed_inventory(&inventory, &reply).is_err());
}
