use crate::user_equip_runtime::*;
use ffone_protocol::{
    ItemUseFailure0104, ItemUseSuccessPacket0104, ItemUseSuccessPrefix0104, PcLoadData0104,
};

fn item(item_type: i16, item_id: i16) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option: 0,
        time_limit: 0,
    }
}

fn write_item(load: &mut PcLoadData0104, offset: usize, value: ItemBase0104) {
    let bytes = load.as_bytes_mut();
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}

fn runtime_with(
    equipment: &[(usize, ItemBase0104)],
    inventory: &[(usize, ItemBase0104)],
) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::zeroed();
    for &(slot, value) in equipment {
        write_item(
            &mut load,
            PcLoadData0104::EQUIPMENT_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    for &(slot, value) in inventory {
        write_item(
            &mut load,
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    InventoryRuntime0104::from_pc_load(77, &load)
}

fn inventory(slot_index: usize) -> UserEquipSlotEndpoint {
    UserEquipSlotEndpoint::Inventory { slot_index }
}

fn equipment(visual_index: usize) -> UserEquipSlotEndpoint {
    let wire_slot_index = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index].wire_slot_index;
    UserEquipSlotEndpoint::Equipment {
        visual_index,
        wire_slot_index,
    }
}

#[test]
fn exact_visual_to_wire_endpoints_build_equip_and_unequip_requests() {
    let runtime = runtime_with(&[(1, item(1, 40))], &[(3, item(4, 21))]);
    assert_eq!(
        prepare_user_equip_move_0104(&runtime, inventory(3), equipment(0)),
        Ok(ItemMoveRequest0104 {
            from_location: 1,
            from_slot_num: 3,
            to_location: 0,
            to_slot_num: 4,
        })
    );
    assert_eq!(
        prepare_user_equip_move_0104(&runtime, equipment(3), inventory(0)),
        Ok(ItemMoveRequest0104 {
            from_location: 0,
            from_slot_num: 1,
            to_location: 1,
            to_slot_num: 0,
        })
    );
}

#[test]
fn invalid_equipment_type_and_occupied_unequip_target_fail_before_send() {
    let runtime = runtime_with(&[(1, item(1, 40))], &[(0, item(7, 5)), (3, item(4, 21))]);
    assert_eq!(
        prepare_user_equip_move_0104(&runtime, inventory(3), equipment(3)),
        Err(UserEquipProductionError0104::IncompatibleEquipmentTarget {
            item_type: 4,
            wire_slot_index: 1,
        })
    );
    assert_eq!(
        prepare_user_equip_move_0104(&runtime, equipment(3), inventory(0)),
        Err(UserEquipProductionError0104::OccupiedUnequipTarget(0))
    );
}

#[test]
fn move_reply_correlation_accepts_openfusion_reversed_endpoint_labels() {
    let request = ItemMoveRequest0104 {
        from_location: 0,
        from_slot_num: 1,
        to_location: 1,
        to_slot_num: 8,
    };
    let reversed = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 8,
        from_slot_item: item(1, 40),
        to_location: 0,
        to_slot_num: 1,
        to_slot_item: item(0, 0),
    };
    assert!(item_move_reply_matches_request(request, reversed));
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Move(request))
        .unwrap();
    assert!(
        production.observe_inventory_packet(&InventoryPacket0104::ItemMoveSuccess(reversed))
    );
    assert!(!production.send_pending());
}

#[test]
fn pending_lock_times_out_into_bounded_late_reply_ownership() {
    let request = ItemUseRequest0104 {
        item_location: 1,
        slot_num: 2,
        nano_slot: 0,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Use(request))
        .unwrap();
    assert_eq!(
        production.begin(UserEquipPendingRequest0104::Use(request)),
        Err(UserEquipProductionError0104::RequestPending(
            UserEquipPendingKind0104::Use
        ))
    );
    assert_eq!(production.tick(9.9), None);
    assert_eq!(
        production.tick(0.1),
        Some(UserEquipPendingRequest0104::Use(request))
    );
    assert!(!production.send_pending());
    assert_eq!(
        production.begin(UserEquipPendingRequest0104::Use(request)),
        Err(UserEquipProductionError0104::AmbiguousLateReply(
            UserEquipPendingKind0104::Use
        ))
    );
    assert!(
        production.observe_item_use_packet(&ItemUsePacket0104::Failure(ItemUseFailure0104 {
            error_code: 7
        }))
    );
    production
        .begin(UserEquipPendingRequest0104::Use(request))
        .unwrap();
    assert!(
        production.observe_item_use_packet(&ItemUsePacket0104::Failure(ItemUseFailure0104 {
            error_code: 7
        }))
    );
}

#[test]
fn combat_gate_only_tracks_an_active_equipped_hand_move() {
    let apparel = ItemMoveRequest0104 {
        from_location: InventoryLocation0104::Inventory.wire_value(),
        from_slot_num: 4,
        to_location: InventoryLocation0104::Equipment.wire_value(),
        to_slot_num: CharacterEquipSlot0104::UpperBody as i32,
    };
    let hand = ItemMoveRequest0104 {
        from_location: InventoryLocation0104::Inventory.wire_value(),
        from_slot_num: 8,
        to_location: InventoryLocation0104::Equipment.wire_value(),
        to_slot_num: CharacterEquipSlot0104::Hand as i32,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Move(apparel))
        .unwrap();
    assert!(!production.active_hand_move_pending());
    production.tick(USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104);
    assert!(production.owns_move_reply_family());
    assert!(
        !production.active_hand_move_pending(),
        "a bounded non-hand tombstone must not suppress world combat"
    );
    production.tick(USER_EQUIP_LATE_REPLY_GRACE_SECONDS_0104);
    production
        .begin(UserEquipPendingRequest0104::Move(hand))
        .unwrap();
    assert!(production.active_hand_move_pending());
}

#[test]
fn timed_out_packet_families_keep_independent_bounded_tombstones() {
    let delete = PcItemDeleteRequest0104 {
        item_location: 1,
        slot_num: 4,
    };
    let move_request = ItemMoveRequest0104 {
        from_location: 0,
        from_slot_num: 1,
        to_location: 1,
        to_slot_num: 8,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Delete(delete))
        .unwrap();
    assert_eq!(
        production.tick(USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104),
        Some(UserEquipPendingRequest0104::Delete(delete))
    );
    production
        .begin(UserEquipPendingRequest0104::Move(move_request))
        .unwrap();
    assert_eq!(
        production.tick(USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104),
        Some(UserEquipPendingRequest0104::Move(move_request))
    );
    assert!(production.owns_delete_reply_family());
    assert!(production.owns_move_reply_family());

    assert!(production.observe_delete_success(PcItemDeleteSuccess0104 {
        item_location: delete.item_location,
        slot_num: delete.slot_num,
    }));
    assert!(!production.owns_delete_reply_family());
    assert!(production.owns_move_reply_family());

    production.tick(USER_EQUIP_LATE_REPLY_GRACE_SECONDS_0104);
    assert!(!production.owns_move_reply_family());
}

#[test]
fn closing_the_shell_preserves_in_flight_reply_ownership_for_bounded_grace() {
    let request = ItemMoveRequest0104 {
        from_location: 0,
        from_slot_num: 1,
        to_location: 1,
        to_slot_num: 8,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Move(request))
        .unwrap();

    assert_eq!(
        production.close_session_preserving_late_replies(),
        Some(UserEquipPendingRequest0104::Move(request))
    );
    assert!(!production.send_pending());
    assert!(production.owns_move_reply_family());

    production.tick(USER_EQUIP_LATE_REPLY_GRACE_SECONDS_0104);
    assert!(!production.owns_move_reply_family());
}

#[test]
fn nonce_less_retry_conflicts_follow_actual_reply_identity() {
    let old = ItemUseRequest0104 {
        item_location: 1,
        slot_num: 2,
        nano_slot: 0,
    };
    let distinct = ItemUseRequest0104 { slot_num: 3, ..old };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Use(old))
        .unwrap();
    production.tick(USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104);

    assert_eq!(
        production.begin(UserEquipPendingRequest0104::Use(old)),
        Err(UserEquipProductionError0104::AmbiguousLateReply(
            UserEquipPendingKind0104::Use
        ))
    );
    assert!(!production.send_pending());
    assert_eq!(
        production.begin(UserEquipPendingRequest0104::Use(distinct)),
        Err(UserEquipProductionError0104::AmbiguousLateReply(
            UserEquipPendingKind0104::Use
        )),
        "Use failure has no endpoint, so every retry in the family is ambiguous"
    );

    let first_move = ItemMoveRequest0104 {
        from_location: 0,
        from_slot_num: 1,
        to_location: 1,
        to_slot_num: 8,
    };
    let reversed_move = ItemMoveRequest0104 {
        from_location: first_move.to_location,
        from_slot_num: first_move.to_slot_num,
        to_location: first_move.from_location,
        to_slot_num: first_move.from_slot_num,
    };
    let mut moves = UserEquipProductionRuntime0104::default();
    moves
        .begin(UserEquipPendingRequest0104::Move(first_move))
        .unwrap();
    moves.tick(USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104);
    assert_eq!(
        moves.begin(UserEquipPendingRequest0104::Move(reversed_move)),
        Err(UserEquipProductionError0104::AmbiguousLateReply(
            UserEquipPendingKind0104::Move
        ))
    );

    production.tick(USER_EQUIP_LATE_REPLY_GRACE_SECONDS_0104);
    production
        .begin(UserEquipPendingRequest0104::Use(old))
        .expect("the exact retry becomes safe after bounded grace");
}

#[test]
fn item_use_success_requires_the_authoritative_local_owner() {
    let request = ItemUseRequest0104 {
        item_location: 1,
        slot_num: 2,
        nano_slot: 0,
    };
    let success = |pc_id| {
        ItemUsePacket0104::Success(
            ItemUseSuccessPacket0104::decode(
                &ItemUseSuccessPrefix0104 {
                    pc_id,
                    item_location: request.item_location,
                    slot_num: request.slot_num,
                    remaining_item: item(7, 134),
                    skill_id: 0,
                    pack_padding: [0; 2],
                    skill_type: 3,
                    target_count: 0,
                }
                .encode_prefix(),
            )
            .unwrap(),
        )
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Use(request))
        .unwrap();
    assert!(!production.observe_item_use_packet_for_owner(&success(88), Some(77)));
    assert!(production.send_pending());
    assert!(production.observe_item_use_packet_for_owner(&success(77), Some(77)));
    assert!(!production.send_pending());
}

#[test]
fn late_reply_for_an_older_same_family_request_keeps_the_new_lock() {
    let old_delete = PcItemDeleteRequest0104 {
        item_location: 1,
        slot_num: 4,
    };
    let new_delete = PcItemDeleteRequest0104 {
        item_location: 1,
        slot_num: 5,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Delete(old_delete))
        .unwrap();
    production.tick(USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104);
    production
        .begin(UserEquipPendingRequest0104::Delete(new_delete))
        .unwrap();

    assert!(production.observe_delete_success(PcItemDeleteSuccess0104 {
        item_location: old_delete.item_location,
        slot_num: old_delete.slot_num,
    }));
    assert_eq!(
        production.pending(),
        Some(UserEquipPendingRequest0104::Delete(new_delete))
    );
}

#[test]
fn use_and_delete_revalidate_bounds_emptiness_and_clean_item_type() {
    let runtime = runtime_with(
        &[],
        &[(2, item(1, 40)), (3, item(7, 134)), (4, item(9, 424))],
    );
    assert_eq!(
        prepare_user_equip_use_0104(&runtime, 2, 0),
        Err(UserEquipProductionError0104::UnsupportedUseItemType(1))
    );
    assert_eq!(
        prepare_user_equip_use_0104(&runtime, 3, 0),
        Ok(ItemUseRequest0104 {
            item_location: 1,
            slot_num: 3,
            nano_slot: 0,
        })
    );
    assert_eq!(
        prepare_user_equip_use_0104(&runtime, 4, 0),
        Err(UserEquipProductionError0104::UnsupportedUseItemType(9))
    );
    assert_eq!(
        prepare_user_equip_delete_0104(&runtime, 49),
        Err(UserEquipProductionError0104::EmptySource(inventory(49)))
    );
    assert_eq!(
        prepare_user_equip_delete_0104(&runtime, 50),
        Err(UserEquipProductionError0104::InvalidInventorySlot(50))
    );
}

#[test]
fn chest_open_builds_the_dedicated_clean_request_and_correlates_its_reply() {
    let mut chest = item(9, 424);
    chest.option = 3;
    chest.time_limit = 99;
    let runtime = runtime_with(&[], &[(7, chest), (8, item(7, 134))]);
    let request = prepare_user_equip_chest_open_0104(&runtime, 7).unwrap();
    assert_eq!(
        request,
        ItemChestOpenRequest0104 {
            item_location: 1,
            slot_num: 7,
            chest_item: ItemBase0104 {
                time_limit: 0,
                ..chest
            },
        }
    );
    assert_eq!(
        prepare_user_equip_chest_open_0104(&runtime, 8),
        Err(UserEquipProductionError0104::UnsupportedChestItemType(7))
    );

    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::ChestOpen(request))
        .unwrap();
    assert!(production.owns_chest_open_reply_family());
    assert!(production.observe_chest_open_success(ItemChestOpenSuccess0104 { slot_num: 7 }));
    assert!(!production.send_pending());
}
