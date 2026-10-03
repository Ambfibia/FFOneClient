use super::*;

#[test]
fn failed_success_reply_mutates_minus_one_slots_and_uses_message_259() {
    let mut model = awaiting_weapon_model();
    model.clear_intents();
    let disposition = model
        .receive_success(EnchantSuccess0104 {
            enchant_item_slot: 7,
            enchant_item: item(0, 501, 0),
            weapon_material_item_slot: 8,
            weapon_material_item: item(7, 101, 28),
            defence_material_item_slot: -1,
            defence_material_item: empty_enchant_item_0104(),
            cash_item_slot_1: -1,
            cash_item_slot_2: -1,
            taros: 9_800,
            success_flag: 0,
        })
        .unwrap();
    assert_eq!(disposition, EnchantReplyDisposition0104::FailureMessage);
    assert!(!model.send_locked());
    assert!(matches!(
        model.phase(),
        EnchantPhase0104::SystemMessage {
            message_id: ENCHANT_MESSAGE_FAILURE_0104,
            callback: EnchantSystemCallback0104::EnchantFailed,
            ..
        }
    ));
    let receipt = model
        .intents()
        .find_map(|intent| match intent {
            EnchantIntent0104::AuthoritativeReceipt(receipt) => Some(receipt),
            _ => None,
        })
        .unwrap();
    assert_eq!(receipt.mutations.len(), 3);
    assert_eq!(receipt.mutations[2].slot, -1);
    assert_eq!(
        model
            .selection()
            .visual_item(EnchantAttachmentSlot0104::Target)
            .unwrap()
            .item,
        item(0, 501, 0)
    );
    assert_eq!(
        model
            .selection()
            .visual_item(EnchantAttachmentSlot0104::WeaponMaterial)
            .unwrap()
            .item,
        item(7, 101, 28)
    );
    assert_eq!(
        ENCHANT_MESSAGE_FAILURE_TEXT_0104,
        "CANNOT EQUIP\nYou cannot equip items here. Exit and go to MY STUFF to change your equipped items."
    );
    model.accept_system_message().unwrap();
    assert!(matches!(model.phase(), EnchantPhase0104::Ready));
    assert!(!model.selection().any_attached());
}
