use crate::vendor_runtime::*;
use ffone_protocol::{
    ItemBase0104, ItemVendor0104, PcLoadData0104, VENDOR_TABLE_ITEM_COUNT_0104,
    VendorFailure0104, VendorItemBuySuccess0104, VendorItemRestoreBuySuccess0104,
    VendorItemSellSuccess0104, VendorStartSuccess0104, VendorTableUpdateSuccess0104,
};

fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

fn inventory() -> InventoryRuntime0104 {
    InventoryRuntime0104::from_pc_load(77, &PcLoadData0104::zeroed())
}

fn active() -> ActiveVendorSourceNpc0104 {
    ActiveVendorSourceNpc0104 {
        runtime_npc_id: 9_001,
        table_npc_id: 650,
        ai_type: 1,
    }
}

fn accept_session(
    runtime: &mut VendorProductionRuntime0104,
    inventory: &mut InventoryRuntime0104,
) {
    assert!(matches!(
        runtime.begin_start(active()).unwrap(),
        VendorOutboundRequest0104::Start(_)
    ));
    assert!(matches!(
        runtime
            .apply_packet(
                VendorPacket0104::StartSuccess(VendorStartSuccess0104 {
                    npc_id: 9_001,
                    vendor_id: 650,
                }),
                inventory,
            )
            .unwrap(),
        VendorProductionEvent0104::StartAccepted { .. }
    ));
    let empty = ItemVendor0104 {
        vendor_id: 650,
        buy_cost: 0.0,
        item: item(0, 0, 0),
        sort_num: 0,
    };
    let mut items = [empty; VENDOR_TABLE_ITEM_COUNT_0104];
    items[0].item = item(4, 104, 0);
    items[1].item = item(7, 42, 0);
    runtime
        .apply_packet(
            VendorPacket0104::TableSuccess(VendorTableUpdateSuccess0104 { items }),
            inventory,
        )
        .unwrap();
}

#[test]
fn placed_npc_identity_and_table_identity_unlock_close() {
    use crate::vendor_ui::{
        VendorCloseGate0104, VendorLifecyclePhase, VendorModalState, VendorUiCommand0104,
        VendorUiOutbox0104, VendorUiState,
    };
    let mut runtime = VendorProductionRuntime0104::default();
    let mut inventory = inventory();
    let mut ui = VendorUiState::default();
    let mut outbox = VendorUiOutbox0104::default();
    ui.begin_open(active().runtime_npc_id, active().table_npc_id, &mut outbox);
    outbox.pop_front();
    let VendorOutboundRequest0104::Start(start) = runtime.begin_start(active()).unwrap() else {
        panic!("start request")
    };
    // OpenFusion echoes Start; its TableUpdate requires both fields to select
    // the vendor table, while RustyFusion checks the placed NPC at Start.
    assert_eq!(start.npc_id, active().runtime_npc_id);
    assert_eq!(start.vendor_id, active().table_npc_id);
    let event = runtime
        .apply_packet(
            VendorPacket0104::StartSuccess(VendorStartSuccess0104 {
                npc_id: start.npc_id,
                vendor_id: start.vendor_id,
            }),
            &mut inventory,
        )
        .unwrap();
    ui.accept_start_success();
    let VendorProductionEvent0104::StartAccepted { table_request } = event else {
        panic!("table request")
    };
    assert_eq!(table_request.npc_id, table_request.vendor_id);
    assert_eq!(
        runtime.session().unwrap().requested_npc_id,
        active().runtime_npc_id
    );
    let items = [ItemVendor0104 {
        vendor_id: start.vendor_id,
        buy_cost: 0.0,
        item: item(0, 0, 0),
        sort_num: 0,
    }; VENDOR_TABLE_ITEM_COUNT_0104];
    assert_eq!(
        runtime
            .apply_packet(
                VendorPacket0104::TableSuccess(VendorTableUpdateSuccess0104 { items }),
                &mut inventory
            )
            .unwrap(),
        VendorProductionEvent0104::TableAccepted
    );
    ui.accept_authoritative_table();
    ui.tick(1.0);
    assert!(!runtime.request_pending());
    let gate = VendorCloseGate0104 {
        mode_accepts_escape: true,
        target_action_idle: true,
    };
    for escape in [false, true] {
        let mut opened = ui;
        if escape {
            opened
                .request_escape(VendorModalState::default(), gate, &mut outbox)
                .unwrap();
        } else {
            opened
                .request_close_button(VendorModalState::default(), gate, &mut outbox)
                .unwrap();
        }
        assert_eq!(opened.phase, VendorLifecyclePhase::Hidden);
        assert_eq!(outbox.pop_front(), Some(VendorUiCommand0104::ExitMode));
    }
}

#[test]
fn start_table_sequence_requires_exact_ids_and_preserves_nonempty_order() {
    let mut runtime = VendorProductionRuntime0104::default();
    let mut inventory = inventory();
    runtime.begin_start(active()).unwrap();
    let before = inventory.clone();
    assert!(matches!(
        runtime.apply_packet(
            VendorPacket0104::StartSuccess(VendorStartSuccess0104 {
                npc_id: 9_002,
                vendor_id: 650,
            }),
            &mut inventory,
        ),
        Err(VendorProductionError0104::ReplyIdentityMismatch { .. })
    ));
    assert_eq!(inventory, before);
    assert!(runtime.request_pending());

    runtime.reset();
    runtime.begin_start(active()).unwrap();
    runtime
        .apply_packet(
            VendorPacket0104::StartSuccess(VendorStartSuccess0104 {
                npc_id: 9_001,
                vendor_id: 650,
            }),
            &mut inventory,
        )
        .unwrap();
    let mut items = [ItemVendor0104 {
        vendor_id: 650,
        buy_cost: 0.0,
        item: item(0, 0, 0),
        sort_num: 0,
    }; VENDOR_TABLE_ITEM_COUNT_0104];
    items[2].item = item(4, 104, 1);
    items[2].sort_num = 8;
    items[5].item = item(4, 100, 1);
    items[5].sort_num = 8;
    runtime
        .apply_packet(
            VendorPacket0104::TableSuccess(VendorTableUpdateSuccess0104 { items }),
            &mut inventory,
        )
        .unwrap();
    assert_eq!(
        runtime.catalog_entries(),
        &[
            VendorCatalogEntry0104 {
                source_slot_id: 0,
                item: item(4, 104, 1),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 1,
                item: item(4, 100, 1),
            },
        ]
    );
}

#[test]
fn wrong_reply_and_failure_do_not_apply_optimistic_inventory_or_tarros() {
    let mut runtime = VendorProductionRuntime0104::default();
    let mut inventory = inventory();
    accept_session(&mut runtime, &mut inventory);
    let identity = runtime.session().unwrap().request_identity();
    assert_eq!(identity.npc_id, active().runtime_npc_id);
    assert_eq!(identity.vendor_id, active().table_npc_id);
    let VendorOutboundRequest0104::Buy(outbound) = runtime
        .begin_intent(
            VendorIntent0104::Buy(crate::vendor_ui::VendorBuyIntent0104 {
                identity,
                item: item(4, 104, 1),
                inventory_slot: 4,
            }),
            &inventory,
        )
        .unwrap()
    else {
        panic!("buy intent must encode a buy request");
    };
    assert_eq!(
        outbound.list_id, -118,
        "clean `(sbyte)iVendorID` preserves the signed low byte on wire"
    );
    let before = inventory.clone();
    assert!(matches!(
        runtime.apply_packet(
            VendorPacket0104::SellFailure(VendorFailure0104 { error_code: 0 }),
            &mut inventory,
        ),
        Err(VendorProductionError0104::UnexpectedReply { .. })
    ));
    assert_eq!(inventory, before);
    assert!(runtime.request_pending());

    assert!(matches!(
        runtime
            .apply_packet(
                VendorPacket0104::BuyFailure(VendorFailure0104 { error_code: 1 }),
                &mut inventory,
            )
            .unwrap(),
        VendorProductionEvent0104::Failed {
            ui_failure: Some(VendorServerFailure0104::Buy { error_code: 1 }),
            ..
        }
    ));
    assert_eq!(inventory, before);
    assert!(!runtime.request_pending());
}

#[test]
fn buy_sell_restore_commit_inventory_and_fifo_only_after_correlated_success() {
    let mut runtime = VendorProductionRuntime0104::default();
    let mut inventory = inventory();
    accept_session(&mut runtime, &mut inventory);
    let identity = runtime.session().unwrap().request_identity();
    assert_eq!(identity.npc_id, active().runtime_npc_id);
    assert_eq!(identity.vendor_id, active().table_npc_id);
    let bought = item(7, 42, 3);
    runtime
        .begin_intent(
            VendorIntent0104::Buy(crate::vendor_ui::VendorBuyIntent0104 {
                identity,
                item: bought,
                inventory_slot: 4,
            }),
            &inventory,
        )
        .unwrap();
    runtime
        .apply_packet(
            VendorPacket0104::BuySuccess(VendorItemBuySuccess0104 {
                candy: 9_000,
                inventory_slot: 4,
                item: bought,
            }),
            &mut inventory,
        )
        .unwrap();
    assert_eq!(inventory.inventory()[4], bought);

    runtime
        .begin_intent(
            VendorIntent0104::Sell(crate::vendor_ui::VendorSellIntent0104 {
                item: bought,
                inventory_slot: 4,
                count: 2,
            }),
            &inventory,
        )
        .unwrap();
    runtime
        .apply_packet(
            VendorPacket0104::SellSuccess(VendorItemSellSuccess0104 {
                candy: 9_200,
                inventory_slot: 4,
                item: ItemBase0104 {
                    option: 2,
                    ..bought
                },
                item_stay: ItemBase0104 {
                    option: 1,
                    ..bought
                },
            }),
            &mut inventory,
        )
        .unwrap();
    assert_eq!(inventory.inventory()[4].option, 1);
    assert_eq!(runtime.recent_entries().len(), 1);
    let recent = runtime.recent_entries().next().copied().unwrap();

    runtime
        .begin_intent(
            VendorIntent0104::Restore(crate::vendor_ui::VendorRestoreIntent0104 {
                identity,
                restore_list_id: 1,
                item: recent.item,
                inventory_slot: 5,
            }),
            &inventory,
        )
        .unwrap();
    runtime
        .apply_packet(
            VendorPacket0104::RestoreSuccess(VendorItemRestoreBuySuccess0104 {
                candy: 9_000,
                inventory_slot: 5,
                item: recent.item,
            }),
            &mut inventory,
        )
        .unwrap();
    assert_eq!(inventory.inventory()[5], recent.item);
    assert_eq!(runtime.recent_entries().len(), 0);
}
