//! Focused production BankMode tests kept independent from the shared crate
//! export while `main.rs` integration is performed separately.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

mod bank_ui {
    pub use ffone_client::bank_ui::*;
}
mod inventory_runtime {
    pub use ffone_client::inventory_runtime::*;
}
#[path = "../src/gameplay/bank/mod.rs"]
mod bank_runtime;

use bank_runtime::*;
use ffone_client_network::network::{NetworkCommand, NetworkEvent};
use ffone_net::{BankGameplayFrame0104, InventoryGameplayFrame0104};
use ffone_protocol::{
    BANK_SLOT_COUNT_0104, DecodedFrame, ItemBase0104, ItemMoveRequest0104,
    ItemMoveSuccessPacket0104, PcBankCloseSuccess0104, PcBankFailure0104, PcBankOpenRequest0104,
    PcBankOpenSuccess0104, PcBankReply0104, PcLoadData0104, WirePayload, packet,
};
use inventory_runtime::InventoryRuntime0104;

const OWNER_PC_ID: i32 = 4_242;
const NPC_ID: i32 = 9_001;

fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

fn empty() -> ItemBase0104 {
    item(0, 0, 0)
}

fn write_item(bytes: &mut [u8], offset: usize, value: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}

fn inventory_with(owner_pc_id: i32, entries: &[(usize, ItemBase0104)]) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::zeroed();
    for &(slot, value) in entries {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    InventoryRuntime0104::from_pc_load(owner_pc_id, &load)
}

fn open_success(extra_bank: i32, entries: &[(usize, ItemBase0104)]) -> PcBankOpenSuccess0104 {
    let mut bank_items = [empty(); BANK_SLOT_COUNT_0104];
    for &(slot, value) in entries {
        bank_items[slot] = value;
    }
    PcBankOpenSuccess0104 {
        bank_items,
        extra_bank,
    }
}

fn frame(packet_type: u32, payload: Vec<u8>) -> DecodedFrame {
    DecodedFrame {
        packet_type,
        flags: 0x1234,
        checksum: 0x5678,
        payload,
    }
}

fn begin(runtime: &mut BankProductionRuntime0104, inventory: &InventoryRuntime0104) {
    assert_eq!(
        runtime
            .begin_open(
                PcBankOpenRequest0104 {
                    pc_id: OWNER_PC_ID,
                    npc_id: NPC_ID,
                },
                inventory,
            )
            .unwrap(),
        BankOutboundRequest0104::Open(PcBankOpenRequest0104 {
            pc_id: OWNER_PC_ID,
            npc_id: NPC_ID,
        })
    );
}

fn accept_open(
    runtime: &mut BankProductionRuntime0104,
    inventory: &InventoryRuntime0104,
    open: PcBankOpenSuccess0104,
) -> BankProductionEvent0104 {
    runtime
        .apply_bank_reply(PcBankReply0104::OpenSuccess(open), inventory)
        .unwrap()
}

#[test]
fn clean_mode_lease_tabs_and_local_exit_never_emit_bank_close() {
    let inventory = inventory_with(OWNER_PC_ID, &[]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);

    let lease = runtime.mode_lease().unwrap();
    assert_eq!(lease.game_mode, 11);
    assert_eq!(lease.linked_game_mode, 5);
    assert_eq!(lease.gui_mode, 1);
    assert!(lease.hides_main_menu);
    assert!(lease.blocks_gameplay_input);
    assert!(!lease.cursor_locked_during_mode);
    assert_eq!(lease.special_state_flag, 16);
    assert_eq!(lease.tab_policy.active_inventory_tab, 0);
    assert!(!lease.tab_policy.tab_switching_enabled);
    assert!(!lease.normal_exit_sends_bank_close_packet);
    assert!(!BANK_NORMAL_EXIT_SENDS_CLOSE_PACKET_0104);

    accept_open(&mut runtime, &inventory, open_success(1, &[]));
    let event = runtime.close_locally().unwrap();
    assert_eq!(
        event,
        BankProductionEvent0104::ClosedLocally {
            source: BankOpenIdentity0104 {
                owner_pc_id: OWNER_PC_ID,
                npc_id: NPC_ID,
            },
        }
    );
    assert!(!runtime.modal_active());
    assert!(!runtime.request_pending());
}

#[test]
fn only_exact_extra_bank_one_unlocks_all_two_hundred_slots() {
    for (raw, expected, slots) in [
        (-1, BankAccessTier0104::BaseSixty, 60),
        (0, BankAccessTier0104::BaseSixty, 60),
        (1, BankAccessTier0104::FullTwoHundred, 200),
        (2, BankAccessTier0104::BaseSixty, 60),
        (i32::MAX, BankAccessTier0104::BaseSixty, 60),
    ] {
        let inventory = inventory_with(OWNER_PC_ID, &[]);
        let mut runtime = BankProductionRuntime0104::default();
        begin(&mut runtime, &inventory);
        let event = accept_open(&mut runtime, &inventory, open_success(raw, &[]));
        assert_eq!(
            event,
            BankProductionEvent0104::OpenAccepted {
                source: BankOpenIdentity0104 {
                    owner_pc_id: OWNER_PC_ID,
                    npc_id: NPC_ID,
                },
                raw_extra_bank: raw,
                access_tier: expected,
                tab_policy: BankTabPolicy0104::default(),
            }
        );
        assert_eq!(runtime.session().unwrap().raw_extra_bank(), raw);
        assert_eq!(runtime.session().unwrap().access_tier(), expected);
        assert_eq!(expected.accessible_slots(), slots);
    }
}

#[test]
fn open_failure_two_waits_for_popup_ack_and_other_failures_close_immediately() {
    let inventory = inventory_with(OWNER_PC_ID, &[]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);
    let source = BankOpenIdentity0104 {
        owner_pc_id: OWNER_PC_ID,
        npc_id: NPC_ID,
    };
    assert_eq!(
        runtime
            .apply_bank_reply(
                PcBankReply0104::OpenFailure(PcBankFailure0104 { error_code: 2 }),
                &inventory,
            )
            .unwrap(),
        BankProductionEvent0104::OpenFailed {
            source,
            error_code: 2,
            show_access_required_popup: true,
            close_immediately: false,
        }
    );
    assert!(runtime.modal_active());
    assert!(!runtime.request_pending());
    assert_eq!(
        runtime.close_locally().unwrap(),
        BankProductionEvent0104::ClosedLocally { source }
    );

    begin(&mut runtime, &inventory);
    assert_eq!(
        runtime
            .apply_bank_reply(
                PcBankReply0104::OpenFailure(PcBankFailure0104 { error_code: 7 }),
                &inventory,
            )
            .unwrap(),
        BankProductionEvent0104::OpenFailed {
            source,
            error_code: 7,
            show_access_required_popup: false,
            close_immediately: true,
        }
    );
    assert!(!runtime.modal_active());
}

#[test]
fn authoritative_move_commit_carries_bank_and_optional_inventory_post_state_atomically() {
    let moved = item(7, 77, 12);
    let inventory = inventory_with(OWNER_PC_ID, &[]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);
    accept_open(&mut runtime, &inventory, open_success(1, &[(3, moved)]));

    let request = ItemMoveRequest0104 {
        from_location: 3,
        from_slot_num: 3,
        to_location: 1,
        to_slot_num: 5,
    };
    assert_eq!(
        runtime.begin_item_move(request, &inventory).unwrap(),
        BankOutboundRequest0104::ItemMove(request)
    );
    let reply = ItemMoveSuccessPacket0104 {
        from_location: 3,
        from_slot_num: 3,
        from_slot_item: empty(),
        to_location: 1,
        to_slot_num: 5,
        to_slot_item: moved,
    };
    let BankProductionEvent0104::ItemMoveCommitted(commit) =
        runtime.apply_item_move_success(reply).unwrap()
    else {
        panic!("move success must emit the atomic commit")
    };
    assert_eq!(commit.request(), request);
    assert_eq!(commit.reply(), reply);
    assert_eq!(commit.post_state_writes().count(), 2);
    assert_eq!(
        commit.bank_writes().collect::<Vec<_>>(),
        vec![BankPostStateWrite0104 {
            slot: bank_ui::BankSlotRef0104::new(bank_ui::BankSlotLocation0104::Bank, 3).unwrap(),
            item: empty(),
        }]
    );
    assert_eq!(
        commit.inventory_writes().collect::<Vec<_>>(),
        vec![BankPostStateWrite0104 {
            slot: bank_ui::BankSlotRef0104::new(bank_ui::BankSlotLocation0104::Inventory, 5)
                .unwrap(),
            item: moved,
        }]
    );
    assert_eq!(commit.snapshot_after().bank()[3], empty());
    assert_eq!(commit.snapshot_after().inventory()[5], moved);
    assert_eq!(
        runtime.session().unwrap().snapshot(),
        commit.snapshot_after()
    );
    assert!(!runtime.request_pending());
}

#[test]
fn move_endpoint_and_post_state_faults_preserve_pending_request_and_snapshot() {
    let moved = item(7, 88, 3);
    let inventory = inventory_with(OWNER_PC_ID, &[(0, moved)]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);
    accept_open(&mut runtime, &inventory, open_success(1, &[]));
    let before = runtime.session().unwrap().snapshot().clone();
    let request = ItemMoveRequest0104 {
        from_location: 1,
        from_slot_num: 0,
        to_location: 3,
        to_slot_num: 4,
    };
    runtime.begin_item_move(request, &inventory).unwrap();

    let mismatched = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 0,
        from_slot_item: empty(),
        to_location: 3,
        to_slot_num: 5,
        to_slot_item: moved,
    };
    assert!(matches!(
        runtime.apply_item_move_success(mismatched),
        Err(BankProductionError0104::MoveReplyEndpointMismatch { .. })
    ));
    assert!(runtime.request_pending());
    assert_eq!(runtime.session().unwrap().snapshot(), &before);

    let malformed = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 0,
        from_slot_item: empty(),
        to_location: 3,
        to_slot_num: 4,
        to_slot_item: item(-1, 99, 0),
    };
    assert!(matches!(
        runtime.apply_item_move_success(malformed),
        Err(BankProductionError0104::BankAuthority(_))
    ));
    assert!(runtime.request_pending());
    assert_eq!(runtime.session().unwrap().snapshot(), &before);

    let valid = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 0,
        from_slot_item: empty(),
        to_location: 3,
        to_slot_num: 4,
        to_slot_item: moved,
    };
    assert!(matches!(
        runtime.apply_item_move_success(valid),
        Ok(BankProductionEvent0104::ItemMoveCommitted(_))
    ));
}

#[test]
fn openfusion_reversed_swap_endpoints_keep_exact_bank_request_ownership() {
    let moved = item(7, 88, 3);
    let inventory = inventory_with(OWNER_PC_ID, &[(0, moved)]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);
    accept_open(&mut runtime, &inventory, open_success(1, &[]));
    let request = ItemMoveRequest0104 {
        from_location: 1,
        from_slot_num: 0,
        to_location: 3,
        to_slot_num: 4,
    };
    runtime.begin_item_move(request, &inventory).unwrap();
    let reply = ItemMoveSuccessPacket0104 {
        from_location: 3,
        from_slot_num: 4,
        from_slot_item: moved,
        to_location: 1,
        to_slot_num: 0,
        to_slot_item: empty(),
    };

    assert_eq!(runtime.pending_item_move_request(), Some(request));
    assert!(runtime.owns_item_move_reply(reply));
    let BankProductionEvent0104::ItemMoveCommitted(commit) =
        runtime.apply_item_move_success(reply).unwrap()
    else {
        panic!("reversed OpenFusion reply must commit the exact bank move")
    };
    assert_eq!(commit.snapshot_after().inventory()[0], empty());
    assert_eq!(commit.snapshot_after().bank()[4], moved);
    assert!(!runtime.request_pending());
}

#[test]
fn half_bank_rejects_locked_source_or_destination_before_sending() {
    let moved = item(7, 91, 1);
    let inventory = inventory_with(OWNER_PC_ID, &[(0, moved)]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);
    accept_open(&mut runtime, &inventory, open_success(0, &[(61, moved)]));

    assert!(matches!(
        runtime.begin_item_move(
            ItemMoveRequest0104 {
                from_location: 3,
                from_slot_num: 61,
                to_location: 1,
                to_slot_num: 1,
            },
            &inventory,
        ),
        Err(BankProductionError0104::LockedBankSlot { .. })
    ));
    assert!(matches!(
        runtime.begin_item_move(
            ItemMoveRequest0104 {
                from_location: 1,
                from_slot_num: 0,
                to_location: 3,
                to_slot_num: 60,
            },
            &inventory,
        ),
        Err(BankProductionError0104::LockedBankSlot { .. })
    ));
    assert!(!runtime.request_pending());
}

#[test]
fn malformed_open_and_item_move_frames_fail_closed_and_passthrough_is_lossless() {
    let inventory = inventory_with(OWNER_PC_ID, &[(0, item(7, 7, 1))]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);

    let malformed_open = BankGameplayFrame0104::decode(frame(
        packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC,
        vec![0; PcBankOpenSuccess0104::SIZE - 1],
    ));
    assert!(matches!(
        runtime.apply_bank_frame(malformed_open, &inventory),
        Err(BankProductionError0104::MalformedBankFrame { .. })
    ));
    assert!(runtime.request_pending());
    assert!(runtime.session().is_none());

    assert_eq!(
        runtime
            .apply_bank_frame(
                BankGameplayFrame0104::decode(frame(0x3100_0abc, vec![1, 2, 3])),
                &inventory,
            )
            .unwrap(),
        None
    );
    accept_open(&mut runtime, &inventory, open_success(1, &[]));
    runtime
        .begin_item_move(
            ItemMoveRequest0104 {
                from_location: 1,
                from_slot_num: 0,
                to_location: 3,
                to_slot_num: 0,
            },
            &inventory,
        )
        .unwrap();
    let malformed_move = InventoryGameplayFrame0104::decode(frame(
        packet::P_FE2CL_PC_ITEM_MOVE_SUCC,
        vec![0; ItemMoveSuccessPacket0104::SIZE + 1],
    ));
    assert!(matches!(
        runtime.apply_inventory_frame(malformed_move),
        Err(BankProductionError0104::MalformedItemMoveFrame { .. })
    ));
    assert!(runtime.request_pending());
}

#[test]
fn close_replies_are_strict_and_success_identity_mismatch_cannot_close_another_pc() {
    let inventory = inventory_with(OWNER_PC_ID, &[]);
    let mut runtime = BankProductionRuntime0104::default();
    begin(&mut runtime, &inventory);
    accept_open(&mut runtime, &inventory, open_success(1, &[]));

    assert_eq!(
        runtime.apply_bank_reply(
            PcBankReply0104::CloseSuccess(PcBankCloseSuccess0104 {
                pc_id: OWNER_PC_ID + 1,
            }),
            &inventory,
        ),
        Err(BankProductionError0104::CloseReplyOwnerMismatch {
            expected_pc_id: OWNER_PC_ID,
            reply_pc_id: OWNER_PC_ID + 1,
        })
    );
    assert!(runtime.modal_active());
    assert_eq!(
        runtime
            .apply_bank_reply(
                PcBankReply0104::CloseFailure(PcBankFailure0104 { error_code: 9 }),
                &inventory,
            )
            .unwrap(),
        BankProductionEvent0104::ClosedByServerFailure {
            source: BankOpenIdentity0104 {
                owner_pc_id: OWNER_PC_ID,
                npc_id: NPC_ID,
            },
            error_code: 9,
        }
    );
    assert!(!runtime.modal_active());
}

#[test]
fn transport_and_client_network_keep_exact_typed_bank_surface() {
    let open = open_success(1, &[]);
    let event = NetworkEvent::Frame(frame(packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC, open.encode()));
    assert!(matches!(
        event.bank_frame_0104(),
        Some(BankGameplayFrame0104::Decoded {
            packet: PcBankReply0104::OpenSuccess(_),
            ..
        })
    ));

    let open_command = NetworkCommand::OpenBank(PcBankOpenRequest0104 {
        pc_id: OWNER_PC_ID,
        npc_id: NPC_ID,
    });
    assert_eq!(
        format!("{open_command:?}"),
        "OpenBank(PcBankOpenRequest0104 { pc_id: 4242, npc_id: 9001 })"
    );
    let close_command =
        NetworkCommand::CloseBank(ffone_protocol::PcBankCloseRequest0104 { pc_id: OWNER_PC_ID });
    assert!(format!("{close_command:?}").starts_with("CloseBank("));
    let move_command = NetworkCommand::MoveItem(ItemMoveRequest0104 {
        from_location: 1,
        from_slot_num: 0,
        to_location: 3,
        to_slot_num: 0,
    });
    assert!(format!("{move_command:?}").starts_with("MoveItem("));
}
