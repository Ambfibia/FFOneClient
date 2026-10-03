use crate::enchant_runtime::*;
use crate::enchant_ui::empty_enchant_item_0104;

fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

fn snapshot() -> EnchantAuthoritySnapshot0104 {
    let mut inventory = [empty_enchant_item_0104(); INVENTORY_SLOT_COUNT_0104];
    inventory[7] = item(0, 501, 1);
    inventory[8] = item(7, 101, 40);
    EnchantAuthoritySnapshot0104 {
        owner_pc_id: 77,
        inventory,
        equipment: [empty_enchant_item_0104(); EQUIPMENT_SLOT_COUNT_0104],
        taros: 10_000,
        weapon_battery: 17,
        nano_battery: 23,
    }
}

fn success_reply(success_flag: i32) -> EnchantSuccess0104 {
    EnchantSuccess0104 {
        enchant_item_slot: 7,
        enchant_item: item(0, 501, 2),
        weapon_material_item_slot: 8,
        weapon_material_item: item(7, 101, 28),
        defence_material_item_slot: -1,
        defence_material_item: empty_enchant_item_0104(),
        cash_item_slot_1: -1,
        cash_item_slot_2: -1,
        taros: 9_800,
        success_flag,
    }
}

fn pending_runtime() -> EnchantProductionRuntime0104 {
    let snapshot = snapshot();
    let context = EnchantOpenContext0104::clean(
        1001,
        123,
        EnchantPlayerAuthority0104 {
            owner_pc_id: 77,
            taros: snapshot.taros,
            weapon_battery: snapshot.weapon_battery,
            nano_battery: snapshot.nano_battery,
        },
        true,
    );
    let mut model = EnchantModeModel0104::default();
    model.open(snapshot.taros, true);
    model.clear_intents();
    model
        .attach(
            EnchantAttachmentSlot0104::Target,
            EnchantSelectableItem0104 {
                source_slot: 7,
                item: snapshot.inventory[7],
                presentation: EnchantItemPresentation0104::default(),
            },
        )
        .unwrap();
    model
        .attach(
            EnchantAttachmentSlot0104::WeaponMaterial,
            EnchantSelectableItem0104 {
                source_slot: 8,
                item: snapshot.inventory[8],
                presentation: EnchantItemPresentation0104::default(),
            },
        )
        .unwrap();
    let material_quantity = model
        .selection()
        .visual_item(EnchantAttachmentSlot0104::WeaponMaterial)
        .unwrap()
        .item
        .option;
    model.clear_intents();
    model.activate_enchant().unwrap();
    model.accept_system_message().unwrap();
    model.advance(4.001).unwrap();
    let request_token = match model.phase() {
        EnchantPhase0104::AwaitingReply { request_token } => *request_token,
        phase => panic!("expected AwaitingReply, got {phase:?}"),
    };
    model.clear_intents();
    let mut reserved_by_source = [0; INVENTORY_SLOT_COUNT_0104];
    reserved_by_source[7] = 1;
    reserved_by_source[8] = material_quantity;
    let mut reservations_by_attachment = array::from_fn(|_| None);
    reservations_by_attachment[EnchantAttachmentSlot0104::Target.index()] =
        Some(EnchantReservation0104 {
            source_slot: 7,
            quantity: 1,
        });
    reservations_by_attachment[EnchantAttachmentSlot0104::WeaponMaterial.index()] =
        Some(EnchantReservation0104 {
            source_slot: 8,
            quantity: material_quantity,
        });
    EnchantProductionRuntime0104 {
        session: Some(EnchantProductionSession0104 {
            context,
            model,
            snapshot,
            drag: None,
            reserved_by_source,
            reservations_by_attachment,
            runtime_modal: None,
            close_decision_pending: false,
            pending_request: Some(PendingRequest0104::Enchant(PendingEnchant0104 {
                request_token,
                request: EnchantRequest0104 {
                    enchant_item_slot: 7,
                    weapon_material_item_slot: 8,
                    defence_material_item_slot: -1,
                    cash_item_slot_1: -1,
                    cash_item_slot_2: -1,
                },
            })),
        }),
    }
}

#[test]
fn authoritative_enchant_commit_updates_only_reply_owned_fields() {
    let before = snapshot();
    let reply = success_reply(1);
    let receipt = expected_enchant_receipt(reply);
    let commit = commit_enchant_reply(&before, reply, &receipt).unwrap();
    assert_eq!(commit.operation(), EnchantOperation0104::Enchant);
    assert_eq!(commit.success_flag(), Some(1));
    assert_eq!(commit.taros_after(), 9_800);
    assert_eq!(commit.weapon_battery_after(), 17);
    assert_eq!(commit.nano_battery_after(), 23);
    assert_eq!(
        commit.inventory_writes().collect::<Vec<_>>(),
        vec![
            EnchantInventoryWrite0104 {
                inventory_index: 7,
                item: item(0, 501, 2),
            },
            EnchantInventoryWrite0104 {
                inventory_index: 8,
                item: item(7, 101, 28),
            },
        ]
    );
    assert_eq!(before.taros(), 10_000, "input snapshot stays immutable");
}

#[test]
fn failure_receipt_keeps_clean_minus_one_intent_but_commit_filters_sentinel() {
    let before = snapshot();
    let mut reply = success_reply(0);
    reply.weapon_material_item = empty_enchant_item_0104();
    let receipt = expected_enchant_receipt(reply);
    assert!(receipt.mutations.iter().any(|mutation| mutation.slot == -1));
    let commit = commit_enchant_reply(&before, reply, &receipt).unwrap();
    assert_eq!(commit.success_flag(), Some(0));
    assert_eq!(commit.inventory_writes().count(), 2);
}

#[test]
fn malformed_or_mismatched_authority_is_rejected_before_any_snapshot_write() {
    let before = snapshot();
    let reply = success_reply(1);
    let mut wrong_receipt = expected_enchant_receipt(reply);
    wrong_receipt.taros += 1;
    assert_eq!(
        commit_enchant_reply(&before, reply, &wrong_receipt),
        Err(EnchantProductionError0104::AuthoritativeReceiptMismatch)
    );

    let mut negative = reply;
    negative.taros = -1;
    let receipt = expected_enchant_receipt(negative);
    assert_eq!(
        commit_enchant_reply(&before, negative, &receipt),
        Err(EnchantProductionError0104::NegativeAuthority {
            field: "reply.taros",
            value: -1,
        })
    );

    let mut malformed = reply;
    malformed.enchant_item = item(-1, 501, 2);
    let receipt = expected_enchant_receipt(malformed);
    assert_eq!(
        commit_enchant_reply(&before, malformed, &receipt),
        Err(EnchantProductionError0104::MalformedReplyItem {
            inventory_index: 7,
            item_type: -1,
            item_id: 501,
        })
    );
    assert_eq!(before, snapshot());
}

#[test]
fn reply_identity_correlation_covers_all_five_request_slots() {
    let request = EnchantRequest0104 {
        enchant_item_slot: 7,
        weapon_material_item_slot: 8,
        defence_material_item_slot: -1,
        cash_item_slot_1: 9,
        cash_item_slot_2: -1,
    };
    let mut reply = success_reply(1);
    reply.cash_item_slot_1 = 9;
    validate_enchant_success_identity(request, reply).unwrap();
    reply.cash_item_slot_1 = 10;
    assert_eq!(
        validate_enchant_success_identity(request, reply),
        Err(EnchantProductionError0104::ReplyIdentityMismatch {
            field: "iCashItemSlot1",
            expected: 9,
            actual: 10,
        })
    );
}

#[test]
fn production_reply_pipeline_is_correlated_and_transactional() {
    let mut runtime = pending_runtime();
    let before = runtime.session().unwrap().snapshot().clone();
    let mut wrong = success_reply(1);
    wrong.enchant_item_slot = 6;
    assert_eq!(
        runtime.apply_reply(EnchantReplyPacket0104::Success(wrong)),
        Err(EnchantProductionError0104::ReplyIdentityMismatch {
            field: "iEnchantItemSlot",
            expected: 7,
            actual: 6,
        })
    );
    assert_eq!(runtime.session().unwrap().snapshot(), &before);
    assert!(runtime.request_pending());

    let output = runtime
        .apply_reply(EnchantReplyPacket0104::Success(success_reply(1)))
        .unwrap();
    assert_eq!(output.commit.unwrap().taros_after(), 9_800);
    assert!(!runtime.request_pending());
    assert!(matches!(
        runtime.session().unwrap().model().phase(),
        EnchantPhase0104::Success { .. }
    ));
}

#[test]
fn failure_modal_release_uses_recorded_reservation_after_empty_reply_item() {
    let mut runtime = pending_runtime();
    let mut reply = success_reply(0);
    reply.weapon_material_item = empty_enchant_item_0104();
    let output = runtime
        .apply_reply(EnchantReplyPacket0104::Success(reply))
        .unwrap();
    assert_eq!(output.commit.unwrap().success_flag(), Some(0));
    assert!(matches!(
        runtime.session().unwrap().model().phase(),
        EnchantPhase0104::SystemMessage {
            callback: EnchantSystemCallback0104::EnchantFailed,
            ..
        }
    ));
    runtime
        .resolve_modal(EnchantModalChoice0104::Accept)
        .unwrap();
    let session = runtime.session().unwrap();
    assert_eq!(session.reserved_by_source[7], 0);
    assert_eq!(session.reserved_by_source[8], 0);
    assert!(matches!(session.model().phase(), EnchantPhase0104::Ready));
}
