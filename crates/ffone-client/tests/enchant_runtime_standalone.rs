#![forbid(unsafe_code)]
#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

// `cargo test` currently reaches an unrelated, user-owned missing generated
// module.  This wrapper lets the controller compile directly against the last
// complete library artifact while still exercising the source under test.
extern crate ffone_client as compiled_client;

mod enchant_ui {
    pub use compiled_client::enchant_ui::*;
}

mod inventory_runtime {
    pub use compiled_client::inventory_runtime::*;
}

#[path = "../src/gameplay/enchant/mod.rs"]
mod enchant_runtime;

use enchant_runtime::*;
use enchant_ui::{
    ENCHANT_FAILURE_PACKET_ID_0104, ENCHANT_REQUEST_PACKET_ID_0104, ENCHANT_SUCCESS_PACKET_ID_0104,
    EnchantAttachmentSlot0104, EnchantItemPresentation0104, EnchantPhase0104, EnchantUiCommand0104,
};
use ffone_protocol::{DecodedFrame, ItemBase0104, PcItemDeleteSuccess0104, PcLoadData0104};
use inventory_runtime::InventoryRuntime0104;

const OWNER: i32 = 77;

fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

fn write_item(load: &mut PcLoadData0104, inventory_index: usize, value: ItemBase0104) {
    let offset = PcLoadData0104::INVENTORY_OFFSET + inventory_index * ItemBase0104::SIZE;
    let bytes = load.as_bytes_mut();
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}

fn inventory_with(values: &[(usize, ItemBase0104)]) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::default();
    for (slot, value) in values.iter().copied() {
        write_item(&mut load, slot, value);
    }
    InventoryRuntime0104::from_pc_load(OWNER, &load)
}

fn context(taros: i32) -> EnchantOpenContext0104 {
    EnchantOpenContext0104::clean(
        1001,
        123,
        EnchantPlayerAuthority0104 {
            owner_pc_id: OWNER,
            taros,
            weapon_battery: 17,
            nano_battery: 23,
        },
        true,
    )
}

fn catalog(_: ItemBase0104) -> Option<EnchantItemPresentation0104> {
    Some(EnchantItemPresentation0104::default())
}

fn drag_drop(
    runtime: &mut EnchantProductionRuntime0104,
    inventory_index: usize,
    attachment: EnchantAttachmentSlot0104,
) {
    runtime
        .apply_ui_command(
            EnchantUiCommand0104::BeginInventoryDrag(inventory_index),
            &catalog,
        )
        .unwrap();
    runtime
        .apply_ui_command(EnchantUiCommand0104::DropOnAttachment(attachment), &catalog)
        .unwrap();
}

fn ready_weapon_runtime() -> (EnchantProductionRuntime0104, InventoryRuntime0104) {
    let inventory = inventory_with(&[
        (7, item(0, 501, 1)),
        (8, item(7, enchant_ui::ENCHANT_WEAPON_MATERIAL_ID_0104, 40)),
    ]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(10_000), &inventory).unwrap();
    drag_drop(&mut runtime, 7, EnchantAttachmentSlot0104::Target);
    drag_drop(&mut runtime, 8, EnchantAttachmentSlot0104::WeaponMaterial);
    (runtime, inventory)
}

#[test]
fn pinned_openfusion_rejects_the_clean_enchant_request() {
    let error = prove_enchant_transport_registered_0104().unwrap_err();
    assert_eq!(
        error,
        EnchantProductionError0104::RequestRegistration(
            ffone_protocol::RegisteredGameplayRequestError0104::UnregisteredPacket {
                packet_type: ENCHANT_REQUEST_PACKET_ID_0104,
            },
        )
    );
}

#[test]
fn open_validates_context_and_emits_clean_shell_contract() {
    let inventory = inventory_with(&[]);
    let mut runtime = EnchantProductionRuntime0104::default();
    let output = runtime.open(context(500), &inventory).unwrap();
    let session = runtime.session().unwrap();
    assert_eq!(session.snapshot().taros(), 500);
    assert_eq!(session.snapshot().weapon_battery(), 17);
    assert_eq!(session.snapshot().nano_battery(), 23);
    assert!(output.effects.iter().any(|effect| matches!(
        effect,
        EnchantShellEffect0104::Audio(enchant_ui::EnchantAudioIntent0104::PlayUiMode)
    )));
    assert_eq!(
        session.model().phase(),
        &enchant_ui::EnchantPhase0104::Ready
    );
}

#[test]
fn classifier_is_exact_and_unrelated_frames_pass_through() {
    let unrelated = DecodedFrame {
        packet_type: 0x3100_ffff,
        flags: 1,
        checksum: 1,
        payload: vec![1, 2, 3],
    };
    assert!(matches!(
        decode_enchant_gameplay_frame_0104(unrelated),
        EnchantGameplayFrame0104::Passthrough(_)
    ));

    let malformed = DecodedFrame {
        packet_type: ENCHANT_SUCCESS_PACKET_ID_0104,
        flags: 0,
        checksum: 0,
        payload: vec![0; 63],
    };
    assert!(matches!(
        decode_enchant_gameplay_frame_0104(malformed),
        EnchantGameplayFrame0104::Malformed {
            error: ffone_protocol::PayloadError::WrongSize {
                expected: 64,
                actual: 63
            },
            ..
        }
    ));

    let failure = DecodedFrame {
        packet_type: ENCHANT_FAILURE_PACKET_ID_0104,
        flags: 0,
        checksum: 0,
        payload: vec![0; 24],
    };
    assert!(matches!(
        decode_enchant_gameplay_frame_0104(failure),
        EnchantGameplayFrame0104::Decoded {
            packet: EnchantReplyPacket0104::Failure(_),
            ..
        }
    ));
}

#[test]
fn drag_uses_authoritative_item_and_rejects_empty_or_bad_stack() {
    let inventory = inventory_with(&[(4, item(7, 101, 9)), (5, item(7, 102, 0))]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(500), &inventory).unwrap();
    let output = runtime
        .apply_ui_command(
            enchant_ui::EnchantUiCommand0104::BeginInventoryDrag(4),
            &catalog,
        )
        .unwrap();
    assert!(
        output
            .effects
            .contains(&EnchantShellEffect0104::DragCaptured {
                inventory_index: 4,
                available_quantity: 9,
            })
    );

    assert_eq!(
        runtime
            .apply_ui_command(
                enchant_ui::EnchantUiCommand0104::BeginInventoryDrag(5),
                &catalog,
            )
            .unwrap_err(),
        EnchantProductionError0104::InvalidStackQuantity {
            inventory_index: 5,
            quantity: 0,
        }
    );
    assert_eq!(
        runtime
            .apply_ui_command(
                enchant_ui::EnchantUiCommand0104::BeginInventoryDrag(6),
                &catalog,
            )
            .unwrap_err(),
        EnchantProductionError0104::EmptyDragSource { inventory_index: 6 }
    );
}

#[test]
fn confirmation_fails_closed_without_changing_modal_or_authority() {
    let (mut runtime, _) = ready_weapon_runtime();
    let before = runtime.session().unwrap().snapshot().clone();
    let output = runtime
        .apply_ui_command(EnchantUiCommand0104::Enchant, &catalog)
        .unwrap();
    assert!(output.effects.iter().any(|effect| matches!(
        effect,
        EnchantShellEffect0104::Popup(enchant_ui::EnchantPopupIntent0104::SystemMessage {
            message_id: enchant_ui::ENCHANT_MESSAGE_CONFIRM_0104,
            callback: enchant_ui::EnchantSystemCallback0104::EnchantConfirmed,
            ..
        })
    )));
    assert!(matches!(
        runtime.session().unwrap().model().phase(),
        EnchantPhase0104::SystemMessage {
            callback: enchant_ui::EnchantSystemCallback0104::EnchantConfirmed,
            ..
        }
    ));

    assert_eq!(
        runtime
            .resolve_modal(EnchantModalChoice0104::Accept)
            .unwrap_err(),
        EnchantProductionError0104::RequestRegistration(
            ffone_protocol::RegisteredGameplayRequestError0104::UnregisteredPacket {
                packet_type: ENCHANT_REQUEST_PACKET_ID_0104,
            },
        )
    );
    let session = runtime.session().unwrap();
    assert_eq!(session.snapshot(), &before);
    assert!(!runtime.request_pending());
    assert!(matches!(
        session.model().phase(),
        EnchantPhase0104::SystemMessage {
            callback: enchant_ui::EnchantSystemCallback0104::EnchantConfirmed,
            ..
        }
    ));
}

#[test]
fn stack_reservations_follow_clean_split_and_prevent_over_reservation() {
    let (mut runtime, _) = ready_weapon_runtime();
    let reserved_for_material = runtime
        .session()
        .unwrap()
        .model()
        .selection()
        .visual_item(EnchantAttachmentSlot0104::WeaponMaterial)
        .unwrap()
        .item
        .option;
    let output = runtime
        .apply_ui_command(EnchantUiCommand0104::BeginInventoryDrag(8), &catalog)
        .unwrap();
    assert!(
        output
            .effects
            .contains(&EnchantShellEffect0104::DragCaptured {
                inventory_index: 8,
                available_quantity: 40 - reserved_for_material,
            })
    );
    runtime
        .apply_ui_command(
            EnchantUiCommand0104::DropOnAttachment(EnchantAttachmentSlot0104::Helper1),
            &catalog,
        )
        .unwrap();
    assert_eq!(
        runtime
            .apply_ui_command(EnchantUiCommand0104::BeginInventoryDrag(8), &catalog)
            .unwrap_err(),
        EnchantProductionError0104::DragSourceAlreadyReserved { inventory_index: 8 }
    );
}

#[test]
fn registered_delete_is_correlated_and_commits_only_after_exact_reply() {
    let original = ItemBase0104 {
        item_type: 7,
        item_id: 999,
        option: 3,
        time_limit: 456,
    };
    let inventory = inventory_with(&[(4, original)]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(500), &inventory).unwrap();
    runtime
        .apply_ui_command(EnchantUiCommand0104::BeginInventoryDrag(4), &catalog)
        .unwrap();
    let modal = runtime
        .apply_ui_command(EnchantUiCommand0104::DropOnTrash, &catalog)
        .unwrap();
    assert!(modal.effects.iter().any(|effect| matches!(
        effect,
        EnchantShellEffect0104::RuntimeModal(EnchantRuntimeModal0104::DeleteItem {
            message_id: enchant_ui::ENCHANT_MESSAGE_DELETE_ITEM_0104,
            inventory_index: 4,
            ..
        })
    )));
    let send = runtime
        .resolve_modal(EnchantModalChoice0104::Accept)
        .unwrap();
    let request = send.request.unwrap();
    assert_eq!(
        request.packet_type(),
        enchant_ui::ENCHANT_DELETE_REQUEST_PACKET_ID_0104
    );
    assert_eq!(request.payload(), &[1, 0, 0, 0, 4, 0, 0, 0]);

    assert_eq!(
        runtime
            .apply_reply(EnchantReplyPacket0104::DeleteSuccess(
                PcItemDeleteSuccess0104 {
                    item_location: 1,
                    slot_num: 5,
                }
            ))
            .unwrap_err(),
        EnchantProductionError0104::ReplyIdentityMismatch {
            field: "iSlotNum",
            expected: 4,
            actual: 5,
        }
    );
    assert_eq!(
        runtime.session().unwrap().snapshot().inventory()[4],
        original
    );
    assert!(runtime.request_pending());

    let accepted = runtime
        .apply_reply(EnchantReplyPacket0104::DeleteSuccess(
            PcItemDeleteSuccess0104 {
                item_location: 1,
                slot_num: 4,
            },
        ))
        .unwrap();
    let commit = accepted.commit.unwrap();
    assert_eq!(commit.operation(), EnchantOperation0104::Delete);
    assert_eq!(
        commit.inventory_writes().collect::<Vec<_>>(),
        vec![EnchantInventoryWrite0104 {
            inventory_index: 4,
            item: ItemBase0104 {
                item_type: 0,
                item_id: 0,
                option: 0,
                time_limit: 456,
            },
        }]
    );
    assert_eq!(commit.taros_after(), 500);
    assert_eq!(commit.weapon_battery_after(), 17);
    assert_eq!(commit.nano_battery_after(), 23);
    assert!(!runtime.request_pending());
}

#[test]
fn close_waits_for_inventory_decision_and_restores_shell_state() {
    let inventory = inventory_with(&[]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(500), &inventory).unwrap();
    let request = runtime
        .apply_ui_command(EnchantUiCommand0104::Close, &catalog)
        .unwrap();
    assert!(request.effects.contains(&EnchantShellEffect0104::Input(
        EnchantInputEffect0104::InventoryCloseDecisionRequested,
    )));
    assert!(runtime.session().unwrap().close_decision_pending());

    let rejected = runtime.resolve_close(false).unwrap();
    assert!(rejected.effects.contains(&EnchantShellEffect0104::Input(
        EnchantInputEffect0104::InventoryCloseRejected,
    )));
    assert!(runtime.is_active());

    runtime
        .apply_ui_command(EnchantUiCommand0104::Close, &catalog)
        .unwrap();
    let accepted = runtime.resolve_close(true).unwrap();
    assert!(accepted.effects.contains(&EnchantShellEffect0104::Input(
        EnchantInputEffect0104::SetCursorLock(true),
    )));
    assert!(accepted.effects.iter().any(|effect| matches!(
        effect,
        EnchantShellEffect0104::Lifecycle(
            enchant_ui::EnchantLifecycleIntent0104::RestoreGameplayInventory {
                inventory_event_dispatch: 10
            }
        )
    )));
    assert!(!runtime.is_active());
}

#[test]
fn refresh_updates_authoritative_counters_and_projection_without_guessing() {
    let inventory = inventory_with(&[]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(500), &inventory).unwrap();
    runtime
        .refresh_authority(
            &inventory,
            EnchantPlayerAuthority0104 {
                owner_pc_id: OWNER,
                taros: 725,
                weapon_battery: 31,
                nano_battery: 47,
            },
        )
        .unwrap();
    assert_eq!(runtime.session().unwrap().model().taros(), 725);

    let inventory_projection =
        std::array::from_fn(|_| enchant_ui::EnchantInventorySlotProjection0104::default());
    let equipment_projection =
        std::array::from_fn(|_| enchant_ui::EnchantInventorySlotProjection0104::default());
    let mut projection = enchant_ui::EnchantModeProjection0104::default();
    runtime.write_presentation(
        &mut projection,
        enchant_ui::EnchantSupportPresentation0104::default(),
        inventory_projection,
        equipment_projection,
    );
    assert_eq!(projection.taros, 725);
    assert_eq!(projection.weapon_battery, 31);
    assert_eq!(projection.nano_battery, 47);
}

#[test]
fn refresh_rejects_a_changed_selected_source_atomically() {
    let (mut runtime, _) = ready_weapon_runtime();
    let before = runtime.session().unwrap().snapshot().clone();
    let changed = inventory_with(&[
        (7, item(0, 501, 2)),
        (8, item(7, enchant_ui::ENCHANT_WEAPON_MATERIAL_ID_0104, 40)),
    ]);
    assert_eq!(
        runtime
            .refresh_authority(
                &changed,
                EnchantPlayerAuthority0104 {
                    owner_pc_id: OWNER,
                    taros: 10_000,
                    weapon_battery: 17,
                    nano_battery: 23,
                },
            )
            .unwrap_err(),
        EnchantProductionError0104::SelectedItemChanged {
            inventory_index: 7,
            expected: item(0, 501, 1),
            actual: item(0, 501, 2),
        }
    );
    assert_eq!(runtime.session().unwrap().snapshot(), &before);
}

#[test]
fn redeem_uses_the_registered_freechat_boundary_without_gameplay_pending_state() {
    let inventory = inventory_with(&[]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(500), &inventory).unwrap();
    runtime
        .apply_ui_command(EnchantUiCommand0104::OpenRedeemCode, &catalog)
        .unwrap();
    let output = runtime.submit_redeem_code("ABC123").unwrap();
    let request = output.request.unwrap();
    assert_eq!(request.packet_type(), 0x1300_0007);
    assert_eq!(request.payload().len(), 260);
    assert!(!runtime.request_pending());
    assert!(output.effects.contains(&EnchantShellEffect0104::Audio(
        enchant_ui::EnchantAudioIntent0104::ActionSuccess,
    )));
}

#[test]
fn not_enough_taros_modal_is_local_but_never_spends_currency() {
    let inventory = inventory_with(&[
        (7, item(0, 501, 1)),
        (8, item(7, enchant_ui::ENCHANT_WEAPON_MATERIAL_ID_0104, 40)),
    ]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(0), &inventory).unwrap();
    drag_drop(&mut runtime, 7, EnchantAttachmentSlot0104::Target);
    drag_drop(&mut runtime, 8, EnchantAttachmentSlot0104::WeaponMaterial);
    let output = runtime
        .apply_ui_command(EnchantUiCommand0104::Enchant, &catalog)
        .unwrap();
    assert!(output.effects.iter().any(|effect| matches!(
        effect,
        EnchantShellEffect0104::Popup(enchant_ui::EnchantPopupIntent0104::SystemMessage {
            message_id: enchant_ui::ENCHANT_MESSAGE_NOT_ENOUGH_TAROS_0104,
            callback: enchant_ui::EnchantSystemCallback0104::None,
            ..
        })
    )));
    assert_eq!(runtime.session().unwrap().snapshot().taros(), 0);
    assert!(output.commit.is_none());
    assert!(output.request.is_none());
    runtime
        .resolve_modal(EnchantModalChoice0104::Accept)
        .unwrap();
    assert!(matches!(
        runtime.session().unwrap().model().phase(),
        EnchantPhase0104::Ready
    ));
}

#[test]
fn every_known_auxiliary_frame_is_classified_and_exact_sized() {
    for (packet_type, size, expected_operation) in [
        (
            enchant_ui::ENCHANT_DELETE_SUCCESS_PACKET_ID_0104,
            8,
            EnchantOperation0104::Delete,
        ),
        (
            enchant_ui::ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104,
            16,
            EnchantOperation0104::Disassemble,
        ),
        (
            enchant_ui::ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104,
            8,
            EnchantOperation0104::Disassemble,
        ),
    ] {
        let frame = DecodedFrame {
            packet_type,
            flags: 0,
            checksum: 0,
            payload: vec![0; size],
        };
        match decode_enchant_gameplay_frame_0104(frame) {
            EnchantGameplayFrame0104::Decoded { packet, .. } => {
                assert_eq!(packet.operation(), expected_operation);
            }
            other => panic!("expected decoded frame, got {other:?}"),
        }
        let malformed = DecodedFrame {
            packet_type,
            flags: 0,
            checksum: 0,
            payload: vec![0; size - 1],
        };
        assert!(matches!(
            decode_enchant_gameplay_frame_0104(malformed),
            EnchantGameplayFrame0104::Malformed { .. }
        ));
    }
}

#[test]
fn passive_outbox_is_consumed_one_command_at_a_time() {
    let inventory = inventory_with(&[(4, item(7, 101, 9))]);
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime.open(context(500), &inventory).unwrap();
    let mut outbox = enchant_ui::EnchantUiOutbox0104::default();
    outbox.push(EnchantUiCommand0104::BeginInventoryDrag(4));
    assert!(
        runtime
            .apply_next_ui_command(&mut outbox, &catalog)
            .unwrap()
            .is_ok()
    );
    assert!(
        runtime
            .apply_next_ui_command(&mut outbox, &catalog)
            .is_none()
    );
}
