use super::*;

#[test]
fn quick_slot_item_use_preserves_pointer_plus_one_event_index() {
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    for (column, item_id) in [(2_usize, 134_i16), (49, 167)] {
        let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET
            + column * ffone_protocol::ItemBase0104::SIZE;
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&7_i16.to_le_bytes());
        load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&item_id.to_le_bytes());
    }
    let inventory = InventoryRuntime0104::from_pc_load(77, &load);
    let mut model = QuickSlotUiModel::default();
    model.slots[0] = LegacyQuickSlotEntry {
        item_type: 7,
        item_id: 134,
        ..default()
    };
    model.slots[1] = LegacyQuickSlotEntry {
        item_type: 7,
        item_id: 167,
        ..default()
    };

    let (hotkey, hotkey_identity) = resolve_quick_slot_item_use_request(
        QuickSlotUiAction {
            logical_slot: 0,
            legacy_event_index: 0,
            source: QuickSlotActivationSource::Hotkey,
        },
        &model,
        &inventory,
    )
    .unwrap();
    assert_eq!(hotkey.slot_num, 2);
    assert_eq!(hotkey_identity.item_id(), 134);

    let (pointer, pointer_identity) = resolve_quick_slot_item_use_request(
        QuickSlotUiAction {
            logical_slot: 0,
            legacy_event_index: 1,
            source: QuickSlotActivationSource::Pointer,
        },
        &model,
        &inventory,
    )
    .unwrap();
    assert_eq!(pointer.slot_num, 49);
    assert_eq!(pointer_identity.item_id(), 167);
}

#[test]
fn email_catalog_uses_base_trade_metadata_and_combined_appearance_icon() {
    let catalog = runtime_test_email_catalog_0104();
    let base = catalog
        .resolve(ItemBase0104 {
            item_type: 0,
            item_id: 1,
            option: 0,
            time_limit: 0,
        })
        .expect("clean weapon row one");
    assert_eq!(base.icon_path.as_deref(), Some("icons/items/weapons/wpnicon_13.png"));
    assert_eq!(base.tradeable, Some(true));
    assert_eq!(base.general_item_type, None);

    let combined = catalog
        .resolve(ItemBase0104 {
            item_type: 0,
            item_id: 1,
            option: 2_i32 << 16,
            time_limit: 0,
        })
        .expect("row-one weapon with row-two combined look");
    assert_eq!(combined.icon_path.as_deref(), Some("icons/items/weapons/wpnicon_37.png"));
    assert_eq!(
        combined.tradeable,
        Some(true),
        "eligibility remains owned by the base row, not the appearance row"
    );

    let general = catalog
        .resolve(ItemBase0104 {
            item_type: 7,
            item_id: 2,
            option: 123,
            time_limit: 0,
        })
        .expect("clean general row two");
    assert_eq!(
        general.icon_path.as_deref(),
        Some("icons/items/general/generalitemicon_09.png")
    );
    assert_eq!(general.tradeable, Some(false));
    assert_eq!(general.general_item_type, Some(6));
}

#[test]
fn combi_catalog_preserves_clean_table_metadata_and_serialized_labels() {
    let catalog = runtime_test_combi_catalog_0104();
    let item = catalog.resolve(0, 1).expect("weapon table row one");
    assert_eq!(item.name, "Disco Bomb");
    assert_eq!(
        item.description,
        "Throw the Boogie Man's Disco Bombs, and your enemies will dance with fear."
    );
    assert_eq!(item.minimum_level, 35);
    assert_eq!(item.required_gender, 0);
    assert_eq!(item.rarity, 2);
    assert_eq!(item.rarity_label, "Special");
    assert_eq!(item.mentor, 0);
    assert_eq!(item.cashable, 0);
    assert_eq!(item.item_price, 1_358);
    assert_eq!(item.point_rating, 584);
    assert_eq!(item.group_rating, 674);
    assert_eq!(item.defense_rating, 0);
    assert_eq!(item.delay_time, 20);
    assert_eq!(item.equip_type, 10);
    assert_eq!(item.target_mode, 6);
    assert_eq!(item.type_label, "Weapon");
    assert_eq!(item.trade_label, "Non-tradable");
    assert!(
        item.icon_path.is_none(),
        "test resolver is deliberately empty"
    );
    assert_eq!(
        catalog
            .resolve(0, 768)
            .expect("recovered Copper Mace row")
            .cashable,
        0,
        "an omitted managed integer keeps its clean default zero"
    );
}

#[test]
fn combi_service_route_is_exactly_category_twenty_six_mode_twenty_five() {
    assert!(combi_service_allowed_0104(26, NpcServiceKind::Combine));
    assert!(!combi_service_allowed_0104(27, NpcServiceKind::Combine));
    assert!(!combi_service_allowed_0104(26, NpcServiceKind::Enchant));

    let context = CombiOpenContext0104::clean(
        9_001,
        6_501,
        CombiPlayerAuthority0104 {
            owner_pc_id: 77,
            gender: 1,
            level: 36,
            guide: 0,
            taros: 10_000,
        },
    );
    let lease = CombiModeLease0104::from(context);
    assert_eq!(context.source.npc_type, 26);
    assert_eq!(context.npc_button_type, 20);
    assert_eq!(context.game_mode, 25);
    assert_eq!(lease.active_inventory_tab, 0);
    assert!(!lease.cursor_locked_during_mode);
    assert_eq!(lease.camera_npc_id, 3_219);
    assert_eq!(lease.first_use_condition_checked_by_service_menu, 67);
    assert!(!lease.normal_exit_sends_packet);
}

#[test]
fn enchant_service_route_is_exactly_category_twenty_seven_mode_twenty_nine() {
    assert!(enchant_service_allowed_0104(27, NpcServiceKind::Enchant));
    assert!(!enchant_service_allowed_0104(26, NpcServiceKind::Enchant));
    assert!(!enchant_service_allowed_0104(28, NpcServiceKind::Enchant));
    assert!(!enchant_service_allowed_0104(27, NpcServiceKind::Combine));

    let context = EnchantOpenContext0104::clean(
        9_001,
        6_501,
        EnchantPlayerAuthority0104 {
            owner_pc_id: 77,
            taros: 10_000,
            weapon_battery: 17,
            nano_battery: 23,
        },
        true,
    );
    assert_eq!(context.source.npc_type, 27);
    assert_eq!(context.game_mode, 29);
}

#[test]
fn email_catalog_keeps_trade_metadata_when_all_artwork_is_missing() {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/data/tables/xdt.json")).unwrap();
    let catalog = EmailProductionCatalog0104::from_table_set_bytes(&bytes, |_| None).unwrap();
    let item = ItemBase0104 { item_type: 7, item_id: 5, option: 1, time_limit: 0 };
    let metadata = catalog.resolve(item).unwrap();
    assert!(metadata.icon_path.is_none());
    assert_eq!(metadata.tradeable, Some(true));
    assert_eq!(metadata.general_item_type, Some(7));
}
