use super::*;

#[test]
fn selection_splits_materials_and_keeps_clean_helper_and_preview_bugs() {
    let mut model = EnchantModeModel0104::default();
    model.open(10_000, false);
    model.clear_intents();
    model
        .attach(EnchantAttachmentSlot0104::Target, selectable(1, 1, 700, 3))
        .unwrap();
    model
        .attach(
            EnchantAttachmentSlot0104::WeaponMaterial,
            selectable(2, 7, 101, 99),
        )
        .unwrap();
    assert_eq!(
        model
            .selection()
            .visual_item(EnchantAttachmentSlot0104::WeaponMaterial)
            .unwrap()
            .item
            .option,
        10
    );
    assert_eq!(
        model.attach(
            EnchantAttachmentSlot0104::Helper1,
            selectable(3, 7, ENCHANT_HELP_ITEM_1_ID_0104, 1)
        ),
        Err(EnchantModelError0104::IntendedHelperRejected {
            slot: EnchantAttachmentSlot0104::Helper1,
            item_id: ENCHANT_HELP_ITEM_1_ID_0104,
        })
    );
    // Clean can split a stack into slot 22 while the remaining source keeps
    // the same SlotID and is then used by another attachment.
    model
        .attach(
            EnchantAttachmentSlot0104::Helper1,
            selectable(2, 7, 101, 12),
        )
        .unwrap();
    assert!(EnchantModeModel0104::helper_drop_accepts(
        EnchantAttachmentSlot0104::Helper2,
        item(7, 999, 1)
    ));
    assert!(!EnchantModeModel0104::helper_drop_accepts(
        EnchantAttachmentSlot0104::Helper2,
        item(7, ENCHANT_HELP_ITEM_2_ID_0104, 1)
    ));
    assert_eq!(
        EnchantModeModel0104::idle_drag_payload_slot(EnchantAttachmentSlot0104::Helper2),
        EnchantAttachmentSlot0104::Target
    );
    assert_eq!(
        EnchantModeModel0104::unattached_armor_preview_role(),
        EnchantAttachmentSlot0104::WeaponMaterial
    );
}

#[test]
fn source_hashes_and_explicit_dead_state_inventory_are_pinned() {
    assert_eq!(
        ENCHANT_SOURCE_MAIN_ARCHIVE_SHA256,
        "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
    );
    assert_eq!(
        ENCHANT_RECIPE_TABLE_SHA256,
        "EA0BC1B274979D819982EFC5651E44EF26645D1B29D529DFEC4069B96D306031"
    );
    assert_eq!(
        ENCHANT_MODE_MANAGED_SOURCE_SHA256,
        "4BF6C33E03E55D0F9FB17BAE9D835529EDD4B98BA0346621736B3E3A195265E7"
    );
    assert_eq!(
        ENCHANT_GUI_MANAGED_SOURCE_SHA256,
        "EF546C88A834322152A9D7E5ECC3CE4D327F9F90B991546D73AD4699F77F21AE"
    );
    assert_eq!(
        ENCHANT_PC_STUFF_MANAGED_SOURCE_SHA256,
        "03653A1002F9DD0C6C58FCB1C4D6172A66D3ABF0CF1DB34F3970CAA3D64EA70F"
    );
    assert_eq!(
        ENCHANT_EQUIPMENT_PANEL_MANAGED_SOURCE_SHA256,
        "A0EA72A50035A48E3B0C5762BFE67BD50872A8E3A657EBDE5CCE978ABF797C60"
    );
    assert_eq!(
        ENCHANT_INVENTORY_MANAGER_MANAGED_SOURCE_SHA256,
        "55096F59C1C1E041EF97E38172EDCE48F0439183BB280AB782BEEBB504905C7B"
    );
    assert_eq!(
        ENCHANT_DELETE_REQUEST_MANAGED_SOURCE_SHA256,
        "9017539F451A0F97CB558DC10036819F41BC6453755B233E5DFC5F06F0E2EFDA"
    );
    assert_eq!(
        ENCHANT_DISASSEMBLE_REQUEST_MANAGED_SOURCE_SHA256,
        "7F74C188A7B0AB36D71BEFC7A20A32FD6197008E83518132FD9476C77A1B92C8"
    );
    assert_eq!(
        ENCHANT_REDEEM_REQUEST_MANAGED_SOURCE_SHA256,
        "C94D43FF206B11733722BC2FC0401BEB65811CDBB9E4E5CBEE0CEE2BADED32E6"
    );
    assert_eq!(ENCHANT_LEGACY_STATES_0104.len(), 13);
    assert!(
        ENCHANT_LEGACY_STATES_0104.contains(&EnchantLegacyStateKind0104::UnreachableCashWarning)
    );
    assert!(
        ENCHANT_LEGACY_STATES_0104
            .contains(&EnchantLegacyStateKind0104::EnchantMoreLeavesSlotObjects)
    );
    assert!(
        ENCHANT_LEGACY_STATES_0104
            .contains(&EnchantLegacyStateKind0104::MaterialLabelsUseEquipmentTable)
    );
    assert!(
        ENCHANT_LEGACY_STATES_0104
            .contains(&EnchantLegacyStateKind0104::DeclaredScrollDelegateNeverAssigned)
    );
    assert!(
        ENCHANT_LEGACY_STATES_0104
            .contains(&EnchantLegacyStateKind0104::KoreanHammerControlHiddenByCleanConfiguration)
    );
    assert!(
        ENCHANT_LEGACY_STATES_0104
            .contains(&EnchantLegacyStateKind0104::SuccessEquipCheckBranchesUseSameColors)
    );
}
