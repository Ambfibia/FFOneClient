use super::*;

#[test]
fn packet_abis_and_little_endian_round_trips_match_pack_four() {
    assert_eq!(ENCHANT_REQUEST_PACKET_ID_0104, 318_767_268);
    assert_eq!(ENCHANT_REQUEST_PACKET_SIZE_0104, 20);
    assert_eq!(ENCHANT_REQUEST_ABI_0104[4].offset, 16);
    let request = EnchantRequest0104 {
        enchant_item_slot: 7,
        weapon_material_item_slot: 8,
        defence_material_item_slot: -1,
        cash_item_slot_1: 9,
        cash_item_slot_2: -1,
    };
    let encoded = request.encode();
    assert_eq!(&encoded[0..4], &7_i32.to_le_bytes());
    assert_eq!(&encoded[8..12], &(-1_i32).to_le_bytes());
    assert_eq!(EnchantRequest0104::decode(&encoded).unwrap(), request);
    assert!(EnchantRequest0104::decode(&encoded[..19]).is_err());

    assert_eq!(ENCHANT_DELETE_REQUEST_PACKET_ID_0104, 318_767_129);
    assert_eq!(ENCHANT_DELETE_REQUEST_PACKET_SIZE_0104, 8);
    let delete = EnchantDeleteRequest0104 {
        inventory_location: 1,
        item_slot: 37,
    };
    let delete_bytes = delete.encode();
    assert_eq!(&delete_bytes[0..4], &1_i32.to_le_bytes());
    assert_eq!(&delete_bytes[4..8], &37_i32.to_le_bytes());
    assert_eq!(
        EnchantDeleteRequest0104::decode(&delete_bytes).unwrap(),
        delete
    );

    assert_eq!(ENCHANT_DISASSEMBLE_REQUEST_PACKET_ID_0104, 318_767_266);
    assert_eq!(ENCHANT_DISASSEMBLE_REQUEST_PACKET_SIZE_0104, 4);
    let disassemble = EnchantDisassembleRequest0104 { item_slot: 12 };
    assert_eq!(
        EnchantDisassembleRequest0104::decode(&disassemble.encode()).unwrap(),
        disassemble
    );

    assert_eq!(ENCHANT_SUCCESS_PACKET_ID_0104, 822_083_885);
    assert_eq!(ENCHANT_SUCCESS_PACKET_SIZE_0104, 64);
    assert_eq!(ENCHANT_SUCCESS_ABI_0104[1].offset, 4);
    assert_eq!(ENCHANT_SUCCESS_ABI_0104[9].offset, 60);
    let success = EnchantSuccess0104 {
        enchant_item_slot: 7,
        enchant_item: item(0, 501, 2),
        weapon_material_item_slot: 8,
        weapon_material_item: item(7, 101, 28),
        defence_material_item_slot: -1,
        defence_material_item: empty_enchant_item_0104(),
        cash_item_slot_1: -1,
        cash_item_slot_2: -1,
        taros: 9_800,
        success_flag: 1,
    };
    assert_eq!(
        EnchantSuccess0104::decode(&success.encode()).unwrap(),
        success
    );

    assert_eq!(ENCHANT_FAILURE_PACKET_ID_0104, 822_083_886);
    assert_eq!(ENCHANT_FAILURE_PACKET_SIZE_0104, 24);
    let failure = EnchantFailure0104 {
        error_code: 3,
        enchant_item_slot: 7,
        weapon_material_item_slot: 8,
        defence_material_item_slot: -1,
        cash_item_slot_1: -1,
        cash_item_slot_2: -1,
    };
    assert_eq!(
        EnchantFailure0104::decode(&failure.encode()).unwrap(),
        failure
    );

    assert_eq!(ENCHANT_REDEEM_REQUEST_PACKET_ID_0104, 318_767_111);
    assert_eq!(ENCHANT_REDEEM_REQUEST_PACKET_SIZE_0104, 260);
    let redeem = enchant_redeem_wire_0104("DEX123").unwrap();
    assert_eq!(redeem.packet_id, ENCHANT_REDEEM_REQUEST_PACKET_ID_0104);
    assert_eq!(redeem.request.message.to_string_lossy(), "/redeem DEX123 ");
    assert_eq!(redeem.request.emote_code, 0);
    assert_eq!(&redeem.payload[..2], &(b'/' as u16).to_le_bytes());
    assert_eq!(&redeem.payload[256..260], &0_i32.to_le_bytes());
    assert_eq!(
        enchant_redeem_wire_0104("ab"),
        Err(EnchantRedeemError0104::TooShort)
    );
    assert_eq!(
        enchant_redeem_wire_0104("has space"),
        Err(EnchantRedeemError0104::ContainsSpace)
    );
}

#[test]
fn failure_packet_and_auxiliary_replies_only_unlock_the_clean_send_gate() {
    let mut model = awaiting_weapon_model();
    model
        .receive_failure(EnchantFailure0104 {
            error_code: 42,
            enchant_item_slot: 7,
            weapon_material_item_slot: 8,
            defence_material_item_slot: -1,
            cash_item_slot_1: -1,
            cash_item_slot_2: -1,
        })
        .unwrap();
    assert!(!model.send_locked());
    assert!(matches!(
        model.phase(),
        EnchantPhase0104::AwaitingReply { .. }
    ));
    assert!(model.input_capabilities().base_gui_enabled);
    assert!(
        model
            .selection()
            .is_attached(EnchantAttachmentSlot0104::Target)
    );

    let mut awaiting = awaiting_weapon_model();
    assert!(awaiting.receive_auxiliary_unlock(ENCHANT_DELETE_SUCCESS_PACKET_ID_0104));
    assert!(!awaiting.send_locked());
    assert!(matches!(
        awaiting.phase(),
        EnchantPhase0104::AwaitingReply { .. }
    ));
    assert!(!awaiting.receive_auxiliary_unlock(123));
}
