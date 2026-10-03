use crate::inventory_runtime::*;
use ffone_protocol::{
    ItemUseBroadcastPrefix0104, ItemUseFailure0104, ItemUseSuccessPrefix0104,
    SkillResultBuff0104, decode_item_use_packet_0104, packet,
};

const OWNER_PC_ID: i32 = 4242;

fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

fn empty() -> ItemBase0104 {
    item(0, 0, 0, 0)
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_item(bytes: &mut [u8], offset: usize, value: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    write_i32(bytes, offset + 4, value.option);
    write_i32(bytes, offset + 8, value.time_limit);
}

fn load_with(
    equipment: &[(usize, ItemBase0104)],
    inventory: &[(usize, ItemBase0104)],
) -> PcLoadData0104 {
    let mut load = PcLoadData0104::zeroed();
    for &(index, value) in equipment {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::EQUIPMENT_OFFSET + index * ItemBase0104::SIZE,
            value,
        );
    }
    for &(index, value) in inventory {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + index * ItemBase0104::SIZE,
            value,
        );
    }
    load
}

fn runtime_with(
    equipment: &[(usize, ItemBase0104)],
    inventory: &[(usize, ItemBase0104)],
) -> InventoryRuntime0104 {
    InventoryRuntime0104::from_pc_load(OWNER_PC_ID, &load_with(equipment, inventory))
}

fn encode_move(packet: ItemMoveSuccessPacket0104) -> Vec<u8> {
    let mut payload = vec![0; ItemMoveSuccessPacket0104::SIZE];
    write_i32(&mut payload, 0, packet.from_location);
    write_i32(&mut payload, 4, packet.from_slot_num);
    write_item(&mut payload, 8, packet.from_slot_item);
    write_i32(&mut payload, 20, packet.to_location);
    write_i32(&mut payload, 24, packet.to_slot_num);
    write_item(&mut payload, 28, packet.to_slot_item);
    payload
}

fn encode_equip(packet: EquipChangePacket0104) -> Vec<u8> {
    let mut payload = vec![0; EquipChangePacket0104::SIZE];
    write_i32(&mut payload, 0, packet.pc_id);
    write_i32(&mut payload, 4, packet.equip_slot_num);
    write_item(&mut payload, 8, packet.equip_slot_item);
    payload
}

fn encode_use(
    pc_id: i32,
    location: i32,
    slot: i32,
    remaining: ItemBase0104,
    target_count: i32,
) -> Vec<u8> {
    ItemUseSuccessPrefix0104 {
        pc_id,
        item_location: location,
        slot_num: slot,
        remaining_item: remaining,
        skill_id: 77,
        pack_padding: [0xA5, 0x5A],
        skill_type: 3,
        target_count,
    }
    .encode_prefix()
}

#[test]
fn pc_load_preserves_exact_nine_and_fifty_slots_and_empty_sentinel_fields() {
    let equipment_last = item(10, 901, 0x0102_0304, -99);
    let inventory_last = item(7, 777, 31, 123_456);
    let noncanonical_empty = item(141, 0, -55, 888);
    let runtime = runtime_with(
        &[(8, equipment_last), (3, noncanonical_empty)],
        &[(49, inventory_last), (17, noncanonical_empty)],
    );

    assert_eq!(runtime.equipment().len(), 9);
    assert_eq!(runtime.inventory().len(), 50);
    assert_eq!(runtime.equipment()[8], equipment_last);
    assert_eq!(runtime.inventory()[49], inventory_last);
    assert_eq!(runtime.equipment()[3], noncanonical_empty);
    assert_eq!(runtime.inventory()[17], noncanonical_empty);
    assert!(InventoryRuntime0104::item_is_empty(noncanonical_empty));
    assert!(
        runtime
            .equipment_authority
            .iter()
            .all(|value| *value == InventorySlotAuthority0104::PcLoadData2Cl)
    );
    assert!(
        runtime
            .inventory_authority
            .iter()
            .all(|value| *value == InventorySlotAuthority0104::PcLoadData2Cl)
    );
}

#[test]
fn identity_lookup_is_type_aware_and_rejects_empty_or_missing_keys() {
    let runtime = runtime_with(&[], &[(4, item(7, 55, 2, 0)), (8, item(9, 55, 0, 0))]);
    let identity = ItemIdentity0104::new(7, 55).unwrap();
    let slot = runtime.find_unique_inventory_slot(identity).unwrap();
    assert_eq!(slot.location(), InventoryLocation0104::Inventory);
    assert_eq!(slot.index(), 4);
    assert_eq!(slot.wire_index(), 4);

    let missing = ItemIdentity0104::new(7, 999).unwrap();
    assert_eq!(
        runtime.find_unique_inventory_slot(missing),
        Err(InventoryLookupError0104::NotFound { identity: missing })
    );
    assert_eq!(
        ItemIdentity0104::new(7, 0),
        Err(InventoryLookupError0104::MalformedIdentity {
            item_type: 7,
            item_id: 0,
        })
    );
    assert_eq!(
        ItemIdentity0104::new(-1, 55),
        Err(InventoryLookupError0104::MalformedIdentity {
            item_type: -1,
            item_id: 55,
        })
    );
}

#[test]
fn duplicate_identity_fails_closed_even_when_options_and_limits_differ() {
    let runtime = runtime_with(
        &[],
        &[
            (2, item(7, 80, 1, 0)),
            (19, item(7, 80, 999, 60_000)),
            (23, item(8, 80, 1, 0)),
        ],
    );
    let identity = ItemIdentity0104::new(7, 80).unwrap();
    assert_eq!(
        runtime.find_unique_inventory_slot(identity),
        Err(InventoryLookupError0104::AmbiguousIdentity {
            identity,
            first: InventorySlotRef0104 {
                location: InventoryLocation0104::Inventory,
                index: 2,
            },
            second: InventorySlotRef0104 {
                location: InventoryLocation0104::Inventory,
                index: 19,
            },
        })
    );
}

#[test]
fn semantic_lookup_supports_resurrection_table_filter_and_rejects_duplicates() {
    let one = runtime_with(
        &[],
        &[
            (6, item(7, 100, 1, 0)),
            (14, item(7, 101, 1, 0)),
            (20, item(0, 0, 77, 88)),
        ],
    );
    let slot = one
        .find_unique_inventory_slot_matching(|value| {
            value.item_type == 7 && value.item_id == 101
        })
        .unwrap();
    assert_eq!(slot.index(), 14);

    let duplicate = runtime_with(&[], &[(6, item(7, 101, 1, 0)), (14, item(7, 102, 1, 0))]);
    assert_eq!(
        duplicate.find_unique_inventory_slot_matching(|value| {
            value.item_type == 7 && matches!(value.item_id, 101 | 102)
        }),
        Err(InventorySelectionError0104::Ambiguous {
            first: InventorySlotRef0104 {
                location: InventoryLocation0104::Inventory,
                index: 6,
            },
            second: InventorySlotRef0104 {
                location: InventoryLocation0104::Inventory,
                index: 14,
            },
        })
    );
}

#[test]
fn item_move_success_uses_authoritative_post_state_for_inventory_swap() {
    let first = item(7, 10, 3, 0);
    let second = item(7, 20, 8, 100);
    let mut runtime = runtime_with(&[], &[(3, first), (11, second)]);
    let packet = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 3,
        from_slot_item: second,
        to_location: 1,
        to_slot_num: 11,
        to_slot_item: first,
    };

    let receipt = runtime.apply_item_move_success(packet).unwrap();
    assert_eq!(runtime.inventory()[3], second);
    assert_eq!(runtime.inventory()[11], first);
    assert_eq!(
        receipt.provenance,
        InventoryMutationProvenance0104::PcItemMoveSuccess
    );
    assert_eq!(receipt.primary.index(), 3);
    assert_eq!(receipt.secondary.unwrap().index(), 11);
    assert_eq!(
        runtime.inventory_authority[3],
        InventorySlotAuthority0104::PcItemMoveSuccess
    );
    assert_eq!(
        runtime.inventory_authority[11],
        InventorySlotAuthority0104::PcItemMoveSuccess
    );
}

#[test]
fn item_move_success_applies_equipment_inventory_swap_in_packet_order() {
    let equipped = item(0, 501, 0, 0);
    let bagged = item(0, 502, 0x0002_0001, 500);
    let mut runtime = runtime_with(&[(0, equipped)], &[(42, bagged)]);
    let packet = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 42,
        from_slot_item: equipped,
        to_location: 0,
        to_slot_num: 0,
        to_slot_item: bagged,
    };

    runtime.apply_item_move_success(packet).unwrap();
    assert_eq!(runtime.inventory()[42], equipped);
    assert_eq!(runtime.equipment()[0], bagged);
    assert_eq!(
        runtime.inventory_authority[42],
        InventorySlotAuthority0104::PcItemMoveSuccess
    );
    assert_eq!(
        runtime.equipment_authority[0],
        InventorySlotAuthority0104::PcItemMoveSuccess
    );
}

#[test]
fn item_move_exact_payload_can_move_into_empty_slot_without_normalizing_empty_value() {
    let moved = item(4, 66, 12, 90);
    let post_source = item(141, 0, -3, 321);
    let mut runtime = runtime_with(&[], &[(0, moved)]);
    let payload = encode_move(ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 0,
        from_slot_item: post_source,
        to_location: 1,
        to_slot_num: 49,
        to_slot_item: moved,
    });

    runtime.apply_item_move_success_payload(&payload).unwrap();
    assert_eq!(runtime.inventory()[0], post_source);
    assert_eq!(runtime.inventory()[49], moved);
    assert!(InventoryRuntime0104::item_is_empty(runtime.inventory()[0]));
}

#[test]
fn item_move_same_slot_conflict_is_transactional() {
    let original = item(7, 1, 1, 0);
    let mut runtime = runtime_with(&[], &[(5, original)]);
    let before = runtime.clone();
    let error = runtime
        .apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: 1,
            from_slot_num: 5,
            from_slot_item: item(7, 2, 1, 0),
            to_location: 1,
            to_slot_num: 5,
            to_slot_item: item(7, 3, 1, 0),
        })
        .unwrap_err();

    assert!(matches!(
        error,
        InventoryMutationError0104::ConflictingSameSlotValues { .. }
    ));
    assert_eq!(runtime, before);
}

#[test]
fn item_move_bounds_location_identity_and_size_fail_without_mutation() {
    let original = item(7, 1, 1, 0);
    let valid = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 0,
        from_slot_item: empty(),
        to_location: 1,
        to_slot_num: 1,
        to_slot_item: original,
    };
    let mut cases = [
        ItemMoveSuccessPacket0104 {
            from_slot_num: -1,
            ..valid
        },
        ItemMoveSuccessPacket0104 {
            to_slot_num: 50,
            ..valid
        },
        ItemMoveSuccessPacket0104 {
            to_location: 3,
            ..valid
        },
        ItemMoveSuccessPacket0104 {
            to_slot_item: item(-1, 44, 0, 0),
            ..valid
        },
    ];
    for packet in cases.iter_mut() {
        let mut runtime = runtime_with(&[], &[(0, original)]);
        let before = runtime.clone();
        assert!(runtime.apply_item_move_success(*packet).is_err());
        assert_eq!(runtime, before);
    }

    for bad_size in [39, 41] {
        let mut runtime = runtime_with(&[], &[(0, original)]);
        let before = runtime.clone();
        let payload = vec![0; bad_size];
        assert!(matches!(
            runtime.apply_item_move_success_payload(&payload),
            Err(InventoryMutationError0104::ExactPayloadSize { .. })
        ));
        assert_eq!(runtime, before);
    }
}

#[test]
fn equip_change_is_owner_checked_bounded_and_exact() {
    let old = item(4, 10, 0, 0);
    let new = item(4, 11, 77, 999);
    let mut runtime = runtime_with(&[(4, old)], &[]);
    let payload = encode_equip(EquipChangePacket0104 {
        pc_id: OWNER_PC_ID,
        equip_slot_num: 4,
        equip_slot_item: new,
    });
    let receipt = runtime.apply_equip_change_payload(&payload).unwrap();
    assert_eq!(runtime.equipment()[4], new);
    assert_eq!(receipt.primary.index(), 4);
    assert_eq!(
        runtime.equipment_authority[4],
        InventorySlotAuthority0104::PcEquipChange
    );

    let before = runtime.clone();
    assert!(matches!(
        runtime.apply_equip_change(EquipChangePacket0104 {
            pc_id: OWNER_PC_ID + 1,
            equip_slot_num: 4,
            equip_slot_item: old,
        }),
        Err(InventoryMutationError0104::OwnerMismatch { .. })
    ));
    assert_eq!(runtime, before);

    assert!(matches!(
        runtime.apply_equip_change(EquipChangePacket0104 {
            pc_id: OWNER_PC_ID,
            equip_slot_num: 9,
            equip_slot_item: old,
        }),
        Err(InventoryMutationError0104::SlotOutOfBounds { .. })
    ));
    assert_eq!(runtime, before);

    let malformed = &payload[..payload.len() - 1];
    assert!(matches!(
        runtime.apply_equip_change_payload(malformed),
        Err(InventoryMutationError0104::ExactPayloadSize { .. })
    ));
    assert_eq!(runtime, before);
}

#[test]
fn zero_target_item_use_success_replaces_exact_remaining_item() {
    let stack = item(7, 200, 5, 10_000);
    let remaining = item(7, 200, 4, 9_000);
    let mut runtime = runtime_with(&[], &[(7, stack)]);
    let payload = encode_use(OWNER_PC_ID, 1, 7, remaining, 0);
    let receipt = runtime.apply_item_use_success_payload(&payload).unwrap();

    assert_eq!(runtime.inventory()[7], remaining);
    assert_eq!(receipt.primary.index(), 7);
    assert_eq!(
        runtime.inventory_authority[7],
        InventorySlotAuthority0104::PcItemUseSuccess
    );
}

#[test]
fn item_use_can_authoritatively_leave_a_noncanonical_empty_sentinel() {
    let stack = item(7, 200, 1, 0);
    let consumed = item(7, 0, -1, 444);
    let mut runtime = runtime_with(&[], &[(7, stack)]);
    let payload = encode_use(OWNER_PC_ID, 1, 7, consumed, 0);
    runtime.apply_item_use_success_payload(&payload).unwrap();
    assert_eq!(runtime.inventory()[7], consumed);
    assert!(InventoryRuntime0104::item_is_empty(consumed));
}

#[test]
fn item_use_failure_and_broadcast_never_own_inventory_mutation() {
    let original = item(7, 200, 5, 10_000);
    let mut runtime = runtime_with(&[], &[(7, original)]);
    let before = runtime.clone();

    let mut broadcast_payload = ItemUseBroadcastPrefix0104 {
        pc_id: OWNER_PC_ID,
        skill_id: 144,
        pack_padding: [0, 0],
        skill_type: 33,
        target_count: 1,
    }
    .encode_prefix();
    broadcast_payload.resize(
        ItemUseBroadcastPrefix0104::SIZE + SkillResultBuff0104::SIZE,
        0,
    );
    let broadcast =
        decode_item_use_packet_0104(packet::P_FE2CL_PC_ITEM_USE, &broadcast_payload)
            .unwrap()
            .unwrap();
    assert_eq!(runtime.apply_item_use_packet(&broadcast), Ok(None));
    assert_eq!(runtime, before);

    let failure = decode_item_use_packet_0104(
        packet::P_FE2CL_REP_PC_ITEM_USE_FAIL,
        &ItemUseFailure0104 { error_code: 3 }.encode(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(runtime.apply_item_use_packet(&failure), Ok(None));
    assert_eq!(runtime, before);
}

#[test]
fn proven_positive_item_use_tail_mutates_and_every_invalid_variant_is_transactional() {
    let original = item(7, 200, 5, 10_000);
    let remaining = item(7, 200, 4, 0);

    let mut positive_target = runtime_with(&[], &[(7, original)]);
    let mut payload = encode_use(OWNER_PC_ID, 1, 7, remaining, 1);
    payload.extend_from_slice(&[0; 32]);
    positive_target
        .apply_item_use_success_payload(&payload)
        .unwrap();
    assert_eq!(positive_target.inventory()[7], remaining);

    let unsupported = ItemUseSuccessPrefix0104 {
        pc_id: OWNER_PC_ID,
        item_location: 1,
        slot_num: 7,
        remaining_item: remaining,
        skill_id: 77,
        pack_padding: [0, 0],
        skill_type: 7,
        target_count: 1,
    }
    .encode_prefix();
    let mut runtime = runtime_with(&[], &[(7, original)]);
    let before = runtime.clone();
    assert!(matches!(
        runtime.apply_item_use_success_payload(&unsupported),
        Err(InventoryMutationError0104::ItemUseDecode {
            error: ItemUseDecodeError0104::UnsupportedPositiveTargetSkillType {
                skill_type: 7,
                target_count: 1,
            }
        })
    ));
    assert_eq!(runtime, before);

    let mut truncated_tail = encode_use(OWNER_PC_ID, 1, 7, remaining, 1);
    truncated_tail.extend_from_slice(&[0; 31]);
    let mut extended_tail = encode_use(OWNER_PC_ID, 1, 7, remaining, 1);
    extended_tail.extend_from_slice(&[0; 33]);

    let failures = [
        encode_use(OWNER_PC_ID + 1, 1, 7, empty(), 0),
        encode_use(OWNER_PC_ID, 3, 7, empty(), 0),
        encode_use(OWNER_PC_ID, 1, 50, empty(), 0),
        encode_use(OWNER_PC_ID, 1, -1, empty(), 0),
        encode_use(OWNER_PC_ID, 1, 7, item(-1, 5, 0, 0), 0),
        encode_use(OWNER_PC_ID, 1, 7, empty(), -1),
        truncated_tail,
        extended_tail,
        vec![0; ItemUseSuccessPrefix0104::SIZE - 1],
        vec![0; ItemUseSuccessPrefix0104::SIZE + 1],
    ];
    for payload in failures {
        let mut runtime = runtime_with(&[], &[(7, original)]);
        let before = runtime.clone();
        assert!(runtime.apply_item_use_success_payload(&payload).is_err());
        assert_eq!(runtime, before);
    }
}

#[test]
fn nano_tune_applies_every_non_sentinel_inventory_post_state_atomically() {
    let original = item(7, 42, 8, 700);
    let mut runtime = runtime_with(&[], &[(0, original), (9, original)]);
    let empty = item(0, 0, 0, 0);
    let remaining = item(7, 42, 3, 650);
    let mut slots = [-1; ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104];
    let mut items = [empty; ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104];
    slots[0] = 0;
    items[0] = remaining;
    slots[1] = 9;
    items[1] = empty;
    slots[2] = 0;
    items[2] = remaining;

    let written = runtime
        .apply_nano_tune_success(NanoTuneSuccess0104 {
            nano_id: 36,
            skill_id: 144,
            fusion_matter: 12_345,
            item_slots: slots,
            items,
        })
        .unwrap();

    assert_eq!(written.len(), 2);
    assert_eq!(runtime.inventory()[0], remaining);
    assert_eq!(runtime.inventory()[9], empty);
    assert_eq!(
        runtime.inventory_authority[0],
        InventorySlotAuthority0104::NanoTuneSuccess
    );
    assert_eq!(
        runtime.inventory_authority[9],
        InventorySlotAuthority0104::NanoTuneSuccess
    );
}

#[test]
fn nano_tune_rejects_conflicting_or_out_of_range_slots_before_any_write() {
    let original = item(7, 42, 8, 700);
    let mut cases = Vec::new();

    let mut duplicate_slots = [-1; ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104];
    let mut duplicate_items =
        [item(0, 0, 0, 0); ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104];
    duplicate_slots[0] = 4;
    duplicate_slots[1] = 4;
    duplicate_items[0] = item(7, 42, 7, 700);
    duplicate_items[1] = item(7, 42, 6, 700);
    cases.push((duplicate_slots, duplicate_items));

    let mut out_of_range_slots = [-1; ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104];
    out_of_range_slots[0] = INVENTORY_SLOT_COUNT_0104 as i32;
    cases.push((
        out_of_range_slots,
        [item(0, 0, 0, 0); ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104],
    ));

    let mut invalid_negative_slots = [-1; ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104];
    invalid_negative_slots[0] = -2;
    cases.push((
        invalid_negative_slots,
        [item(0, 0, 0, 0); ffone_protocol::NANO_TUNE_ITEM_SLOT_COUNT_0104],
    ));

    for (item_slots, items) in cases {
        let mut runtime = runtime_with(&[], &[(4, original)]);
        let before = runtime.clone();
        assert!(
            runtime
                .apply_nano_tune_success(NanoTuneSuccess0104 {
                    nano_id: 36,
                    skill_id: 144,
                    fusion_matter: 12_345,
                    item_slots,
                    items,
                })
                .is_err()
        );
        assert_eq!(runtime, before);
    }
}

#[test]
fn correlated_vendor_successes_apply_only_the_clean_authoritative_post_state() {
    let general = item(7, 42, 5, 700);
    let mut runtime = runtime_with(&[], &[(4, general)]);

    let bought = item(4, 104, 1, 0);
    runtime
        .apply_vendor_buy_success(VendorItemBuySuccess0104 {
            candy: 9_000,
            inventory_slot: 8,
            item: bought,
        })
        .unwrap();
    assert_eq!(runtime.inventory()[8], bought);
    assert_eq!(
        runtime.inventory_authority[8],
        InventorySlotAuthority0104::VendorItemBuySuccess
    );

    let stay = item(7, 42, 3, 650);
    runtime
        .apply_vendor_sell_success(VendorItemSellSuccess0104 {
            candy: 9_500,
            inventory_slot: 4,
            item: general,
            item_stay: stay,
        })
        .unwrap();
    assert_eq!(runtime.inventory()[4], stay);

    let gear_reply = item(4, 104, 0x0102_0304, 123);
    runtime
        .apply_vendor_sell_success(VendorItemSellSuccess0104 {
            candy: 9_800,
            inventory_slot: 8,
            item: gear_reply,
            item_stay: item(99, 999, 999, 999),
        })
        .unwrap();
    assert_eq!(
        runtime.inventory()[8],
        ItemBase0104 {
            item_id: 0,
            ..gear_reply
        },
        "non-general sell clears only iID exactly like cnVendor"
    );

    let restored_wire = item(4, 104, 2, 999_999);
    runtime
        .apply_vendor_restore_success(VendorItemRestoreBuySuccess0104 {
            candy: 8_000,
            inventory_slot: 9,
            item: restored_wire,
        })
        .unwrap();
    assert_eq!(
        runtime.inventory()[9],
        ItemBase0104 {
            time_limit: 0,
            ..restored_wire
        }
    );
}

#[test]
fn delete_and_disassemble_mutate_only_after_bounded_success_replies() {
    let original = item(7, 42, 5, 777);
    let mut runtime = runtime_with(&[], &[(7, original)]);
    runtime
        .apply_item_delete_success(PcItemDeleteSuccess0104 {
            item_location: 1,
            slot_num: 7,
        })
        .unwrap();
    assert_eq!(runtime.inventory()[7], item(0, 0, 0, 777));
    assert_eq!(
        runtime.inventory_authority[7],
        InventorySlotAuthority0104::PcItemDeleteSuccess
    );

    let material = item(7, 900, 3, 0);
    runtime
        .apply_disassemble_success(PcDisassembleItemSuccess0104 {
            new_item_slot: 7,
            new_item: material,
        })
        .unwrap();
    assert_eq!(runtime.inventory()[7], material);
    assert_eq!(
        runtime.inventory_authority[7],
        InventorySlotAuthority0104::PcDisassembleItemSuccess
    );
}

#[test]
fn malformed_vendor_successes_are_transactional() {
    let original = item(7, 42, 5, 777);
    let cases = [
        (
            VendorItemBuySuccess0104 {
                candy: 1,
                inventory_slot: -1,
                item: original,
            },
            0,
        ),
        (
            VendorItemBuySuccess0104 {
                candy: 1,
                inventory_slot: 50,
                item: original,
            },
            0,
        ),
        (
            VendorItemBuySuccess0104 {
                candy: 1,
                inventory_slot: 8,
                item: item(-1, 44, 0, 0),
            },
            0,
        ),
    ];
    for (packet, _) in cases {
        let mut runtime = runtime_with(&[], &[(7, original)]);
        let before = runtime.clone();
        assert!(runtime.apply_vendor_buy_success(packet).is_err());
        assert_eq!(runtime, before);
    }

    let mut runtime = runtime_with(&[], &[(7, original)]);
    let before = runtime.clone();
    assert!(
        runtime
            .apply_item_delete_success(PcItemDeleteSuccess0104 {
                item_location: 3,
                slot_num: 7,
            })
            .is_err()
    );
    assert_eq!(runtime, before);

    assert!(
        runtime
            .apply_disassemble_success(PcDisassembleItemSuccess0104 {
                new_item_slot: 50,
                new_item: original,
            })
            .is_err()
    );
    assert_eq!(runtime, before);
}
