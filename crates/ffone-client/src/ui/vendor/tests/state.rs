use super::*;

pub(super) fn runtime_with(
    inventory: &[(usize, ItemBase0104)],
    equipment: &[(usize, ItemBase0104)],
) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::zeroed();
    for &(slot, value) in inventory {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    for &(slot, value) in equipment {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::EQUIPMENT_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    InventoryRuntime0104::from_pc_load(OWNER_PC_ID, &load)
}

#[test]
fn inventory_full_matches_message_and_silent_general_restore_branches() {
    let full: Vec<_> = (0..INVENTORY_SLOT_COUNT_0104)
        .map(|slot| (slot, item(0, slot as i16 + 20, 0, 0)))
        .collect();
    let projected = projection(
        &[
            VendorCatalogEntry0104 {
                source_slot_id: 0,
                item: item(0, 1, 0, 0),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 1,
                item: item(7, 2, 0, 0),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 2,
                item: item(7, 3, 0, 0),
            },
        ],
        &[VendorRecentBuyEntry0104 {
            source_slot_id: 0,
            item: item(0, 1, 0, 123),
        }],
        &full,
    );
    assert_eq!(
        projected.request_buy(0, 0),
        VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
            VendorSystemMessageId0104::InventoryFull
        ))
    );
    assert_eq!(
        projected.request_buy(1, 1),
        VendorActionOutcome0104::SilentBlocked(
            VendorSilentBlock0104::InventoryFullForGeneralBuy
        )
    );
    assert!(matches!(
        projected.request_buy(2, 1),
        VendorActionOutcome0104::Intent(VendorIntent0104::Battery(_))
    ));
    assert_eq!(
        projected.request_restore(0),
        VendorActionOutcome0104::SilentBlocked(VendorSilentBlock0104::InventoryFullForRestore)
    );
}

#[test]
fn intents_never_mutate_inventory_currency_or_recent_authority() {
    let projected = projection(
        &[
            VendorCatalogEntry0104 {
                source_slot_id: 0,
                item: item(0, 1, 0, 0),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 1,
                item: item(7, 3, 0, 0),
            },
        ],
        &[VendorRecentBuyEntry0104 {
            source_slot_id: 0,
            item: item(7, 4, 2, 0),
        }],
        &[(1, item(7, 4, 10, 0))],
    );
    let before = projected.clone();
    let _ = projected.request_buy(0, 0);
    let _ = projected.request_buy(1, 5);
    let _ = projected.request_restore(0);
    let _ = projected.request_sell(1, 2);
    let _ = projected.request_delete(1);
    let _ = projected.request_disassemble(1);
    assert_eq!(projected, before);
}
