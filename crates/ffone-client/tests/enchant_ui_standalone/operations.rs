use super::*;

pub(super) fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

pub(super) fn presentation(name: &str, icon: &str, cashable: i32) -> EnchantItemPresentation0104 {
    EnchantItemPresentation0104 {
        name: name.to_owned(),
        description: format!("{name} description"),
        icon_path: Some(icon.to_owned()),
        minimum_level: 5,
        cashable,
        can_equip: true,
        point_rating: 11,
        group_rating: 12,
        defense_rating: 13,
        type_label: "Weapon".to_owned(),
        range_label: "Ranged".to_owned(),
        rarity_label: "Common".to_owned(),
        trade_label: "Tradable".to_owned(),
    }
}

pub(super) fn selectable(
    source_slot: i32,
    item_type: i16,
    item_id: i16,
    option: i32,
) -> EnchantSelectableItem0104 {
    EnchantSelectableItem0104 {
        source_slot,
        item: item(item_type, item_id, option),
        presentation: presentation(
            &format!("item-{item_type}-{item_id}"),
            "icons/items/weapons/wpnicon_01.png",
            0,
        ),
    }
}

#[test]
fn semantic_pngs_are_exact_clean_extractions() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    assert_eq!(ENCHANT_SOURCE_TEXTURES_0104.len(), 12);
    for evidence in ENCHANT_SOURCE_TEXTURES_0104 {
        let bytes = fs::read(asset_root.join(evidence.runtime_path)).unwrap();
        assert_eq!(bytes.len(), evidence.png_bytes, "{}", evidence.source_name);
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            evidence.width
        );
        assert_eq!(
            u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
            evidence.height
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)).to_uppercase(),
            evidence.png_sha256
        );
    }
    assert!(
        ENCHANT_UI_DEFAULT_IMAGE_PATHS_0104
            .iter()
            .all(|path| enchant_safe_relative_asset_path_0104(path))
    );
    for path in ENCHANT_UI_DEFAULT_IMAGE_PATHS_0104 {
        assert!(
            asset_root.join(path).is_file(),
            "missing native asset {path}"
        );
    }
    assert!(!enchant_safe_relative_asset_path_0104(
        "../legacy/main.unity3d"
    ));
    assert!(!enchant_safe_relative_asset_path_0104(
        "C:/legacy/main.unity3d"
    ));
    let table_bytes = fs::read(asset_root.join(ENCHANT_RECIPE_TABLE_PATH)).unwrap();
    let tables: serde_json::Value = ffone_client::xdt::from_slice(&table_bytes).unwrap();
    let recipes = tables["tables"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|table| table["value"]["m_pEnchantTable"]["m_pEnchantData"].as_array())
        .collect::<Vec<_>>();
    assert_eq!(recipes.len(), 1, "one authoritative native enchant table");
    assert_eq!(recipes[0].len(), CLEAN_ENCHANT_RECIPES_0104.len());
    // Changes to unrelated NPC/audio rows must not invalidate the Croc Pot.
    // Compare every recipe field, rather than a hash of the entire table set.
    for (index, (row, expected)) in recipes[0]
        .iter()
        .zip(CLEAN_ENCHANT_RECIPES_0104)
        .enumerate()
    {
        for (key, value) in [
            ("m_iEnchantGrade", expected.enchant_grade),
            ("m_iCost", expected.cost),
            ("m_iClass", expected.class),
            ("m_iWpnMatter", expected.weapon_matter),
            ("m_iCostumeMatter", expected.costume_matter),
            ("m_iProbability", expected.probability),
            ("m_iOffenceUp", expected.offence_up),
            ("m_iDefenceUp", expected.defence_up),
            ("m_iFailType", expected.fail_type),
            ("m_iNoDropProb", expected.no_drop_probability),
            ("m_iOneDropProb", expected.one_drop_probability),
            ("m_iTwoDropProb", expected.two_drop_probability),
            ("m_iThreeDropProb", expected.three_drop_probability),
            ("m_iFourDropProb", expected.four_drop_probability),
        ] {
            assert_eq!(
                row[key].as_i64(),
                Some(i64::from(value)),
                "recipe {index} field {key}"
            );
        }
    }
}

#[test]
fn preview_mutates_attached_item_and_emits_exact_popup_boundary() {
    let mut model = EnchantModeModel0104::default();
    model.open(1_000, false);
    model.clear_intents();
    model
        .attach(EnchantAttachmentSlot0104::Target, selectable(7, 0, 501, 1))
        .unwrap();
    let cached_requirements = model.requirements().unwrap().unwrap();
    model.clear_intents();
    let popup_item = model.preview().unwrap();
    assert_eq!(popup_item.item.option, 2);
    assert_eq!(
        model
            .selection()
            .visual_item(EnchantAttachmentSlot0104::Target)
            .unwrap()
            .item
            .option,
        2
    );
    // Preview increments the InventoryManager object directly. The clean
    // recipe fields remain those cached by ReceiveSetItemAttached.
    assert_eq!(model.requirements().unwrap(), Some(cached_requirements));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Popup(EnchantPopupIntent0104::Preview {
            gui_mode: 9,
            source_slot_type: 21,
            action: 7,
            popup_rect: (-1, -1, 0, 0),
            ..
        })
    )));
}

#[test]
fn confirmation_wait_is_strict_and_success_keeps_send_lock_for_overlay() {
    let mut model = EnchantModeModel0104::default();
    model.open(10_000, true);
    model.clear_intents();
    attach_ready_weapon(&mut model);
    model.clear_intents();
    model.activate_enchant().unwrap();
    assert!(matches!(
        model.phase(),
        EnchantPhase0104::SystemMessage {
            message_id: ENCHANT_MESSAGE_CONFIRM_0104,
            callback: EnchantSystemCallback0104::EnchantConfirmed,
            ..
        }
    ));
    assert!(!model.input_capabilities().base_gui_enabled);
    model.accept_system_message().unwrap();
    assert!(model.send_locked());
    model.advance(4.0).unwrap();
    assert!(matches!(model.phase(), EnchantPhase0104::Waiting { .. }));
    assert_eq!(model.waiting_progress_width(), 414.0);
    model.advance(0.001).unwrap();
    assert!(matches!(
        model.phase(),
        EnchantPhase0104::AwaitingReply { .. }
    ));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Wire {
            packet_id: ENCHANT_REQUEST_PACKET_ID_0104,
            request: EnchantRequest0104 {
                enchant_item_slot: 7,
                weapon_material_item_slot: 8,
                defence_material_item_slot: -1,
                cash_item_slot_1: -1,
                cash_item_slot_2: -1,
            },
            ..
        }
    )));

    model.clear_intents();
    let disposition = model
        .receive_success(EnchantSuccess0104 {
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
        })
        .unwrap();
    assert_eq!(disposition, EnchantReplyDisposition0104::SuccessOverlay);
    assert!(model.send_locked());
    assert!(!model.input_capabilities().base_gui_enabled);
    assert!(model.input_capabilities().success_buttons_enabled);
    let receipt = model.intents().find_map(|intent| match intent {
        EnchantIntent0104::AuthoritativeReceipt(receipt) => Some(receipt),
        _ => None,
    });
    let receipt = receipt.unwrap();
    assert_eq!(receipt.mutations.len(), 2);
    assert!(receipt.mutations.iter().all(|mutation| mutation.slot != -1));

    model.enchant_more_items().unwrap();
    assert!(matches!(model.phase(), EnchantPhase0104::Ready));
    assert!(!model.send_locked());
    assert!(model.selection().any_orphaned());
    assert!(
        model
            .selection()
            .visual_item(EnchantAttachmentSlot0104::Target)
            .is_some()
    );
    assert_eq!(
        model
            .selection()
            .visual_item(EnchantAttachmentSlot0104::Target)
            .unwrap()
            .item,
        item(0, 501, 2)
    );
    assert!(
        !model
            .selection()
            .is_attached(EnchantAttachmentSlot0104::Target)
    );
}
