//! Focused clean-Combi production tests. The source module is included
//! directly so this boundary can also be exercised with `rustc --test` while
//! unrelated crate modules are under active reconstruction.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

mod combi_ui {
    pub use ffone_client::combi_ui::*;
}
mod inventory_runtime {
    pub use ffone_client::inventory_runtime::*;
}
#[path = "../src/gameplay/combi/mod.rs"]
mod combi_runtime;

use std::collections::HashMap;

use combi_runtime::*;
use combi_ui::*;
use ffone_protocol::{DecodedFrame, ItemBase0104, PcLoadData0104};
use inventory_runtime::InventoryRuntime0104;

const OWNER_PC_ID: i32 = 4_242;
const CLEAN_TABLE_SET: &[u8] = include_bytes!("../../../assets/game/data/tables/xdt.json");

#[derive(Default)]
struct FixtureCatalog {
    rows: HashMap<(i16, i16), CombiItemMetadata0104>,
}

impl FixtureCatalog {
    fn insert(&mut self, item_type: i16, item_id: i16, metadata: CombiItemMetadata0104) {
        self.rows.insert((item_type, item_id), metadata);
    }
}

impl CombiItemCatalog0104 for FixtureCatalog {
    fn resolve(&self, item_type: i16, item_id: i16) -> Option<CombiItemMetadata0104> {
        self.rows.get(&(item_type, item_id)).cloned()
    }

    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }

    fn enable_equip_combi(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }
}

fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

fn metadata(
    name: &str,
    minimum_level: i32,
    rarity: i32,
    item_price: i32,
    target_mode: i32,
) -> CombiItemMetadata0104 {
    CombiItemMetadata0104 {
        name: name.to_owned(),
        description: format!("{name} description"),
        minimum_level,
        required_gender: 1,
        rarity,
        rarity_label: format!("Rarity {rarity}"),
        mentor: 0,
        cashable: 0,
        item_price,
        point_rating: minimum_level * 10,
        group_rating: minimum_level * 11,
        defense_rating: minimum_level * 12,
        delay_time: 10,
        equip_type: 4,
        target_mode,
        type_label: "Weapon".to_owned(),
        trade_label: "Tradable".to_owned(),
        icon_path: Some(format!("icons/items/{name}.png")),
    }
}

fn write_item(bytes: &mut [u8], offset: usize, value: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}

fn fixture_inventory(owner_pc_id: i32) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::zeroed();
    for (slot, value) in [
        (4, item(0, 10, 11 << 16, 55)),
        (7, item(0, 20, 0, 66)),
        (9, item(0, 30, 0, 77)),
    ] {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    write_item(
        load.as_bytes_mut(),
        PcLoadData0104::EQUIPMENT_OFFSET,
        item(0, 30, 0, 0),
    );
    InventoryRuntime0104::from_pc_load(owner_pc_id, &load)
}

fn fixture_catalog() -> FixtureCatalog {
    let mut catalog = FixtureCatalog::default();
    catalog.insert(0, 10, metadata("base-style", 3, 0, 90, 5));
    catalog.insert(0, 11, metadata("visible-style", 6, 1, 100, 5));
    catalog.insert(0, 20, metadata("stats", 4, 2, 200, 5));
    catalog.insert(0, 30, metadata("equipped", 2, 1, 50, 5));
    catalog
}

fn fixture_recipes() -> CombiRecipeTable0104 {
    CombiRecipeTable0104::from_table_set_bytes(CLEAN_TABLE_SET).expect("published clean recipes")
}

fn player(taros: i32) -> CombiPlayerAuthority0104 {
    CombiPlayerAuthority0104 {
        owner_pc_id: OWNER_PC_ID,
        gender: 1,
        level: 8,
        guide: 2,
        taros,
    }
}

fn context(taros: i32) -> CombiOpenContext0104 {
    CombiOpenContext0104::clean(90_001, 3_133, player(taros))
}

fn opened() -> (
    CombiProductionRuntime0104,
    InventoryRuntime0104,
    FixtureCatalog,
    CombiRecipeTable0104,
) {
    let inventory = fixture_inventory(OWNER_PC_ID);
    let catalog = fixture_catalog();
    let recipes = fixture_recipes();
    let mut runtime = CombiProductionRuntime0104::default();
    runtime
        .open(context(701), &inventory, &catalog, &recipes)
        .expect("open clean Combi");
    (runtime, inventory, catalog, recipes)
}

fn attach_pair(
    runtime: &mut CombiProductionRuntime0104,
    catalog: &FixtureCatalog,
    recipes: &CombiRecipeTable0104,
) {
    for (inventory_index, slot) in [
        (4, CombiSelectionSlot0104::Style),
        (7, CombiSelectionSlot0104::Stats),
    ] {
        runtime
            .apply_ui_command(
                CombiUiCommand0104::BeginInventoryDrag { inventory_index },
                catalog,
                recipes,
            )
            .unwrap();
        runtime
            .apply_ui_command(
                CombiUiCommand0104::DropOnSelection { slot },
                catalog,
                recipes,
            )
            .unwrap();
    }
}

#[test]
fn cancelled_drag_leaves_nothing_to_drop() {
    let (mut runtime, _inventory, catalog, recipes) = opened();
    runtime
        .apply_ui_command(
            CombiUiCommand0104::BeginInventoryDrag { inventory_index: 4 },
            &catalog,
            &recipes,
        )
        .unwrap();
    runtime
        .apply_ui_command(CombiUiCommand0104::CancelInventoryDrag, &catalog, &recipes)
        .unwrap();
    assert_eq!(
        runtime.session().unwrap().drag_source_inventory_index(),
        None
    );
    assert!(matches!(
        runtime.apply_ui_command(
            CombiUiCommand0104::DropOnSelection {
                slot: CombiSelectionSlot0104::Style,
            },
            &catalog,
            &recipes,
        ),
        Err(CombiProductionError0104::MissingDragSource)
    ));
}

fn begin_awaiting(
    runtime: &mut CombiProductionRuntime0104,
    catalog: &FixtureCatalog,
    recipes: &CombiRecipeTable0104,
) -> Vec<u8> {
    attach_pair(runtime, catalog, recipes);
    let modal = runtime
        .apply_ui_command(CombiUiCommand0104::Combine, catalog, recipes)
        .unwrap();
    assert!(modal.effects.contains(&CombiShellEffect0104::SystemMessage(
        CombiSystemMessage0104::Modal {
            modal: CombiSystemModal0104::AttemptConfirmation,
            style_item: None,
            stats_item: None,
        }
    )));
    let waiting = runtime
        .resolve_modal(CombiModalChoice0104::Continue, catalog, recipes)
        .unwrap();
    assert!(
        waiting
            .effects
            .contains(&CombiShellEffect0104::NpcAnimation {
                npc_id: COMBI_NPC_ID_0104,
                animation: CombiNpcAnimation0104::MakingInOut,
            })
    );
    assert!(runtime.tick(4.0).unwrap().request.is_none());
    let sent = runtime.tick(0.001).unwrap();
    let request = sent.request.expect("strict >4 second request");
    assert_eq!(request.packet_type(), COMBI_REQUEST_PACKET_ID_0104);
    assert!(sent.effects.contains(&CombiShellEffect0104::NpcAnimation {
        npc_id: COMBI_NPC_ID_0104,
        animation: CombiNpcAnimation0104::Stand,
    }));
    request.payload().to_vec()
}

fn frame(packet_type: u32, payload: Vec<u8>) -> DecodedFrame {
    DecodedFrame {
        packet_type,
        flags: 0x1234,
        checksum: 0x5678,
        payload,
    }
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn success_payload(
    new_item_slot: i32,
    new_item: ItemBase0104,
    stat_item_slot: i32,
    taros_after: i32,
    success_flag: i32,
) -> Vec<u8> {
    let mut payload = vec![0; COMBI_SUCCESS_PACKET_SIZE_0104];
    write_i32(
        &mut payload,
        COMBI_SUCCESS_NEW_ITEM_SLOT_OFFSET_0104,
        new_item_slot,
    );
    write_item(&mut payload, COMBI_SUCCESS_NEW_ITEM_OFFSET_0104, new_item);
    write_i32(
        &mut payload,
        COMBI_SUCCESS_STAT_SLOT_OFFSET_0104,
        stat_item_slot,
    );
    write_i32(&mut payload, COMBI_SUCCESS_CASH_SLOT_1_OFFSET_0104, 0);
    write_i32(&mut payload, COMBI_SUCCESS_CASH_SLOT_2_OFFSET_0104, 0);
    write_i32(&mut payload, COMBI_SUCCESS_TAROS_OFFSET_0104, taros_after);
    write_i32(&mut payload, COMBI_SUCCESS_FLAG_OFFSET_0104, success_flag);
    payload
}

fn failure_payload(error_code: i32, style_slot: i32, stats_slot: i32) -> Vec<u8> {
    let mut payload = vec![0; COMBI_FAILURE_PACKET_SIZE_0104];
    write_i32(&mut payload, COMBI_FAILURE_ERROR_OFFSET_0104, error_code);
    write_i32(
        &mut payload,
        COMBI_FAILURE_COSTUME_SLOT_OFFSET_0104,
        style_slot,
    );
    write_i32(
        &mut payload,
        COMBI_FAILURE_STAT_SLOT_OFFSET_0104,
        stats_slot,
    );
    write_i32(&mut payload, COMBI_FAILURE_CASH_SLOT_1_OFFSET_0104, 0);
    write_i32(&mut payload, COMBI_FAILURE_CASH_SLOT_2_OFFSET_0104, 0);
    payload
}

fn apply_success(
    runtime: &mut CombiProductionRuntime0104,
    catalog: &FixtureCatalog,
    recipes: &CombiRecipeTable0104,
) -> CombiProductionOutput0104 {
    begin_awaiting(runtime, catalog, recipes);
    let snapshot = runtime.session().unwrap().snapshot();
    let combined = expected_success_style_item(snapshot.inventory[4], snapshot.inventory[7]);
    runtime
        .apply_frame(frame(
            COMBI_SUCCESS_PACKET_ID_0104,
            success_payload(4, combined, 7, 1, 1),
        ))
        .unwrap()
        .unwrap()
}

#[test]
fn exact_open_context_and_mode_lease_drive_passive_ui_without_claiming_cameras_bound() {
    let inventory = fixture_inventory(OWNER_PC_ID);
    let catalog = fixture_catalog();
    let recipes = fixture_recipes();
    let mut runtime = CombiProductionRuntime0104::default();
    let open_context = context(701);
    let output = runtime
        .open(open_context, &inventory, &catalog, &recipes)
        .unwrap();
    let lease = runtime.mode_lease().unwrap();
    assert_eq!(lease.game_mode, 25);
    assert_eq!(lease.source.runtime_npc_id, 90_001);
    assert_eq!(lease.source.table_npc_id, 3_133);
    assert_eq!(lease.source.npc_type, 26);
    assert_eq!(lease.active_inventory_tab, 0);
    assert!(!lease.cursor_locked_during_mode);
    assert_eq!(lease.camera_npc_id, 3_219);
    assert_eq!(lease.first_use_condition_checked_by_service_menu, 67);
    assert!(!lease.normal_exit_sends_packet);
    assert_eq!(
        output.effects,
        vec![CombiShellEffect0104::ModeOpened {
            context: open_context,
            lease,
        }]
    );

    let mut state = CombiUiState0104::default();
    let mut projection = CombiModeProjection0104::default();
    runtime.write_presentation(&mut state, &mut projection);
    assert_eq!(state.phase, CombiPhase0104::Ready);
    assert!(!state.primary_npc_camera_bound);
    assert!(!state.waiting_npc_camera_bound);
    assert_eq!(projection.owner_pc_id, OWNER_PC_ID);
    assert_eq!(
        projection.inventory[4].item,
        Some(item(0, 10, 11 << 16, 55))
    );
}

#[test]
fn wrong_service_owner_or_negative_authority_fails_before_opening() {
    let inventory = fixture_inventory(OWNER_PC_ID);
    let catalog = fixture_catalog();
    let recipes = fixture_recipes();
    let mut runtime = CombiProductionRuntime0104::default();
    let mut wrong = context(701);
    wrong.source.npc_type = 27;
    assert_eq!(
        runtime.open(wrong, &inventory, &catalog, &recipes),
        Err(CombiProductionError0104::WrongNpcType {
            expected: 26,
            actual: 27,
        })
    );
    let mut wrong_owner = context(701);
    wrong_owner.player.owner_pc_id += 1;
    assert!(matches!(
        runtime.open(wrong_owner, &inventory, &catalog, &recipes),
        Err(CombiProductionError0104::InventoryOwnerMismatch { .. })
    ));
    assert!(matches!(
        runtime.open(context(-1), &inventory, &catalog, &recipes),
        Err(CombiProductionError0104::NegativeTaros { taros: -1 })
    ));
    assert!(!runtime.modal_active());
}

#[test]
fn drag_attach_detach_clear_and_equipped_rejection_preserve_local_inventory() {
    let (mut runtime, _, catalog, recipes) = opened();
    let before = runtime.session().unwrap().snapshot().inventory;
    attach_pair(&mut runtime, &catalog, &recipes);
    assert_eq!(runtime.session().unwrap().snapshot().inventory, before);
    assert_eq!(
        runtime
            .session()
            .unwrap()
            .machine()
            .selection
            .style_inventory_index(),
        Some(4)
    );
    assert_eq!(
        runtime
            .session()
            .unwrap()
            .machine()
            .selection
            .stats_inventory_index(),
        Some(7)
    );

    let rejected = runtime
        .apply_ui_command(
            CombiUiCommand0104::RejectEquippedItem {
                equipment_index: 0,
                message_id: 260,
            },
            &catalog,
            &recipes,
        )
        .unwrap();
    assert_eq!(
        rejected.effects,
        vec![CombiShellEffect0104::SystemMessage(
            CombiSystemMessage0104::EquippedItem {
                message_id: 260,
                equipment_index: 0,
                item: item(0, 30, 0, 0),
            }
        )]
    );

    let cleared = runtime
        .apply_ui_command(CombiUiCommand0104::ClearAll, &catalog, &recipes)
        .unwrap();
    assert_eq!(
        cleared.effects,
        vec![
            CombiShellEffect0104::SelectionChanged(CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Style,
                inventory_index: 4,
            }),
            CombiShellEffect0104::SelectionChanged(CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Stats,
                inventory_index: 7,
            }),
        ]
    );
    assert_eq!(runtime.session().unwrap().snapshot().inventory, before);
}

#[test]
fn confirmation_wait_and_registered_request_match_clean_order_and_bytes() {
    let (mut runtime, _, catalog, recipes) = opened();
    let before = runtime.session().unwrap().snapshot().clone();
    let payload = begin_awaiting(&mut runtime, &catalog, &recipes);
    assert_eq!(payload.len(), 16);
    assert_eq!(payload[0..4], 4_i32.to_le_bytes());
    assert_eq!(payload[4..8], 7_i32.to_le_bytes());
    assert_eq!(payload[8..16], [0; 8]);
    assert!(runtime.request_pending());
    assert_eq!(runtime.session().unwrap().snapshot(), &before);
    assert_eq!(
        runtime.session().unwrap().machine().phase,
        CombiPhase0104::AwaitingAuthoritativeReply
    );
}

#[test]
fn equal_taros_produces_exact_not_enough_modal_and_never_enters_waiting() {
    let inventory = fixture_inventory(OWNER_PC_ID);
    let catalog = fixture_catalog();
    let recipes = fixture_recipes();
    let mut runtime = CombiProductionRuntime0104::default();
    runtime
        .open(context(700), &inventory, &catalog, &recipes)
        .unwrap();
    attach_pair(&mut runtime, &catalog, &recipes);
    let output = runtime
        .apply_ui_command(CombiUiCommand0104::Combine, &catalog, &recipes)
        .unwrap();
    assert_eq!(
        output.effects,
        vec![CombiShellEffect0104::SystemMessage(
            CombiSystemMessage0104::Modal {
                modal: CombiSystemModal0104::NotEnoughTaros,
                style_item: None,
                stats_item: None,
            }
        )]
    );
    assert!(!runtime.request_pending());
    assert_eq!(
        runtime.session().unwrap().machine().phase,
        CombiPhase0104::Modal(CombiSystemModal0104::NotEnoughTaros)
    );
    runtime
        .resolve_modal(CombiModalChoice0104::Ok, &catalog, &recipes)
        .unwrap();
    assert_eq!(
        runtime.session().unwrap().machine().phase,
        CombiPhase0104::Ready
    );
}

#[test]
fn success_reply_emits_atomic_two_slot_and_taros_commit_then_supports_repeat() {
    let (mut runtime, _, catalog, recipes) = opened();
    let before = runtime.session().unwrap().snapshot().clone();
    let output = apply_success(&mut runtime, &catalog, &recipes);
    let commit = output.commit.as_ref().expect("authoritative commit");
    let combined = expected_success_style_item(before.inventory[4], before.inventory[7]);
    assert_eq!(
        commit.inventory_writes().collect::<Vec<_>>(),
        vec![
            CombiInventoryWrite0104 {
                inventory_index: 4,
                item: combined,
            },
            CombiInventoryWrite0104 {
                inventory_index: 7,
                item: empty_item_0104(),
            },
        ]
    );
    assert_eq!(commit.taros_after(), 1);
    assert_eq!(commit.snapshot_after().inventory[4], combined);
    assert_eq!(commit.snapshot_after().inventory[7], empty_item_0104());
    assert_eq!(
        runtime.session().unwrap().snapshot(),
        commit.snapshot_after()
    );
    assert!(!runtime.request_pending());
    assert!(
        output
            .effects
            .contains(&CombiShellEffect0104::PlaySuccessSound {
                true_name: "CrocPot_success",
            })
    );

    runtime
        .apply_ui_command(CombiUiCommand0104::CombineMoreItems, &catalog, &recipes)
        .unwrap();
    let session = runtime.session().unwrap();
    assert_eq!(session.machine().phase, CombiPhase0104::Ready);
    assert_eq!(
        session.machine().selection,
        CombiSelectionOverlay0104::default()
    );
    assert_eq!(session.projection().taros, 1);
}

#[test]
fn unsuccessful_result_commits_only_server_taros_and_modal_ack_clears_style_then_stats() {
    let (mut runtime, _, catalog, recipes) = opened();
    begin_awaiting(&mut runtime, &catalog, &recipes);
    let before = runtime.session().unwrap().snapshot().clone();
    let output = runtime
        .apply_reply(CombiReplyPacket0104::Success(CombiSuccessReply0104 {
            new_item_slot: 4,
            new_item: before.inventory[4],
            stat_item_slot: 7,
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
            taros_after: 2,
            success_flag: 0,
        }))
        .unwrap();
    let commit = output.commit.unwrap();
    assert_eq!(commit.inventory_writes().count(), 0);
    assert_eq!(commit.taros_after(), 2);
    assert_eq!(commit.snapshot_after().inventory, before.inventory);
    assert!(
        output
            .effects
            .contains(&CombiShellEffect0104::SystemMessage(
                CombiSystemMessage0104::Modal {
                    modal: CombiSystemModal0104::CombinationFailed,
                    style_item: Some(before.inventory[4]),
                    stats_item: Some(before.inventory[7]),
                }
            ))
    );

    let ack = runtime
        .resolve_modal(CombiModalChoice0104::Ok, &catalog, &recipes)
        .unwrap();
    assert_eq!(
        ack.effects,
        vec![
            CombiShellEffect0104::SelectionChanged(CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Style,
                inventory_index: 4,
            }),
            CombiShellEffect0104::SelectionChanged(CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Stats,
                inventory_index: 7,
            }),
        ]
    );
    assert_eq!(
        runtime.session().unwrap().machine().phase,
        CombiPhase0104::Ready
    );
}

#[test]
fn fail_packet_releases_send_lock_and_never_creates_authority_commit() {
    let (mut runtime, _, catalog, recipes) = opened();
    begin_awaiting(&mut runtime, &catalog, &recipes);
    let before = runtime.session().unwrap().snapshot().clone();
    let output = runtime
        .apply_frame(frame(
            COMBI_FAILURE_PACKET_ID_0104,
            failure_payload(42, 4, 7),
        ))
        .unwrap()
        .unwrap();
    assert!(output.commit.is_none());
    assert_eq!(runtime.session().unwrap().snapshot(), &before);
    assert!(
        !runtime.request_pending(),
        "wire reply consumed correlation"
    );
    assert_eq!(
        runtime.session().unwrap().machine().phase,
        CombiPhase0104::Ready
    );
    assert_eq!(
        output.effects,
        vec![CombiShellEffect0104::SystemMessage(
            CombiSystemMessage0104::WireFailure { error_code: 42 }
        )]
    );
    assert!(runtime.apply_ui_command(CombiUiCommand0104::Close, &catalog, &recipes).is_ok());
}

#[test]
fn malformed_or_mismatched_replies_preserve_pending_attempt_and_snapshot() {
    let (mut runtime, _, catalog, recipes) = opened();
    begin_awaiting(&mut runtime, &catalog, &recipes);
    let before = runtime.session().unwrap().snapshot().clone();
    assert!(matches!(
        runtime.apply_frame(frame(
            COMBI_SUCCESS_PACKET_ID_0104,
            vec![0; COMBI_SUCCESS_PACKET_SIZE_0104 - 1],
        )),
        Err(CombiProductionError0104::MalformedFrame { .. })
    ));
    assert!(runtime.request_pending());
    assert_eq!(runtime.session().unwrap().snapshot(), &before);

    let combined = expected_success_style_item(before.inventory[4], before.inventory[7]);
    assert!(matches!(
        runtime.apply_reply(CombiReplyPacket0104::Success(CombiSuccessReply0104 {
            new_item_slot: 5,
            new_item: combined,
            stat_item_slot: 7,
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
            taros_after: 1,
            success_flag: 1,
        })),
        Err(CombiProductionError0104::Reply(
            CombiReplyError0104::MismatchedSuccessEnvelope { .. }
        ))
    ));
    assert!(runtime.request_pending());
    assert_eq!(runtime.session().unwrap().snapshot(), &before);
    assert_eq!(
        runtime
            .apply_frame(frame(0x3100_0abc, vec![1, 2, 3]))
            .unwrap(),
        None
    );
}

#[test]
fn authority_refresh_rejects_selected_item_replacement_and_any_non_ready_phase() {
    let (mut runtime, _, catalog, recipes) = opened();
    attach_pair(&mut runtime, &catalog, &recipes);
    let replacement_inventory = {
        let mut load = PcLoadData0104::zeroed();
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + 4 * ItemBase0104::SIZE,
            item(0, 30, 0, 0),
        );
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + 7 * ItemBase0104::SIZE,
            item(0, 20, 0, 66),
        );
        InventoryRuntime0104::from_pc_load(OWNER_PC_ID, &load)
    };
    assert!(matches!(
        runtime.refresh_authority(&replacement_inventory, player(701), &catalog, &recipes),
        Err(CombiProductionError0104::SelectedItemChanged {
            slot: CombiSelectionSlot0104::Style,
            inventory_index: 4,
            ..
        })
    ));

    runtime
        .apply_ui_command(CombiUiCommand0104::Combine, &catalog, &recipes)
        .unwrap();
    assert!(matches!(
        runtime.refresh_authority(
            &fixture_inventory(OWNER_PC_ID),
            player(701),
            &catalog,
            &recipes,
        ),
        Err(CombiProductionError0104::AuthorityRefreshBlocked {
            phase: CombiPhase0104::Modal(CombiSystemModal0104::AttemptConfirmation)
        })
    ));
}

#[test]
fn help_close_outbox_and_go_to_my_stuff_are_typed_local_routes() {
    let (mut runtime, _, catalog, recipes) = opened();
    assert_eq!(
        runtime
            .apply_ui_command(CombiUiCommand0104::Help, &catalog, &recipes)
            .unwrap()
            .effects,
        vec![CombiShellEffect0104::HelpRequested]
    );
    let mut outbox = CombiUiOutbox0104::default();
    outbox.push(CombiUiCommand0104::Close);
    let closed = runtime
        .apply_next_ui_command(&mut outbox, &catalog, &recipes)
        .unwrap()
        .unwrap();
    assert!(closed.request.is_none());
    assert!(closed.effects.contains(&CombiShellEffect0104::ModeClosed {
        context: context(701),
        reason: CombiCloseReason0104::Close,
    }));
    assert!(!runtime.modal_active());

    let (mut runtime, _, catalog, recipes) = opened();
    apply_success(&mut runtime, &catalog, &recipes);
    let go = runtime
        .apply_ui_command(CombiUiCommand0104::GoToMyStuff, &catalog, &recipes)
        .unwrap();
    assert!(go.request.is_none());
    assert!(
        go.effects
            .contains(&CombiShellEffect0104::GoToMyStuffRequested {
                game_mode: 6,
                first_use_condition: 3,
            })
    );
    assert!(!runtime.modal_active());
}

#[test]
fn reply_decoder_is_exact_for_success_failure_and_passthrough_ids() {
    let combined = item(0, 20, 11 << 16, 55);
    let success = frame(
        COMBI_SUCCESS_PACKET_ID_0104,
        success_payload(4, combined, 7, 123, 1),
    );
    assert!(matches!(
        decode_combi_gameplay_frame_0104(success),
        CombiGameplayFrame0104::Decoded {
            packet: CombiReplyPacket0104::Success(CombiSuccessReply0104 {
                new_item_slot: 4,
                new_item,
                stat_item_slot: 7,
                taros_after: 123,
                success_flag: 1,
                ..
            }),
            ..
        } if new_item == combined
    ));
    assert!(matches!(
        decode_combi_gameplay_frame_0104(frame(
            COMBI_FAILURE_PACKET_ID_0104,
            failure_payload(9, 4, 7),
        )),
        CombiGameplayFrame0104::Decoded {
            packet: CombiReplyPacket0104::Failure(CombiFailureReply0104 {
                error_code: 9,
                costume_item_slot: 4,
                stat_item_slot: 7,
                ..
            }),
            ..
        }
    ));
    assert!(matches!(
        decode_combi_gameplay_frame_0104(frame(0x3100_ffff, vec![1])),
        CombiGameplayFrame0104::Passthrough(_)
    ));
}
