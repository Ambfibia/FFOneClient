use super::*;

pub(super) struct TestCatalog;

impl VendorItemCatalog0104 for TestCatalog {
    fn resolve(&self, value: ItemBase0104) -> Option<VendorItemMetadata0104> {
        if value.item_id == 90 {
            return None;
        }
        let (general_item_type, battery_recharge, sellable) = match value.item_id {
            2 => (Some(2), Some(0), true),
            3 => (Some(2), Some(100), true),
            4 => (Some(1), None, true),
            5 => (None, None, false),
            _ => (None, None, true),
        };
        Some(VendorItemMetadata0104 {
            name: format!("Test Item {}", value.item_id),
            level: i32::from(value.item_id),
            buy_price: 100 + i32::from(value.item_id),
            sell_price: 10 + i32::from(value.item_id),
            sellable,
            general_item_type,
            battery_recharge,
            stack_size: (value.item_type == 7).then_some(20),
            icon: VendorIconRef::new(format!(
                "icons/items/test/type{}-{}.png",
                value.item_type, value.item_id
            ))
            .ok(),
        })
    }
}

#[test]
fn authoritative_catalog_and_recent_fifo_are_filtered_capped_and_not_reordered() {
    let vendor: Vec<_> = (0..55)
        .map(|index| VendorCatalogEntry0104 {
            source_slot_id: index,
            item: item(0, index as i16 + 1, 0, index as i32),
        })
        .collect();
    let recent: Vec<_> = (0..20)
        .map(|index| VendorRecentBuyEntry0104 {
            source_slot_id: index,
            item: item(7, index as i16 + 1, index as i32 + 1, 999),
        })
        .collect();
    let projected = projection(&vendor, &recent, &[]);
    assert_eq!(projected.catalog_rows.len(), VENDOR_CATALOG_CAPACITY);
    assert_eq!(
        projected
            .catalog_rows
            .iter()
            .map(|row| row.source_slot_id)
            .collect::<Vec<_>>(),
        (0..50).collect::<Vec<_>>()
    );
    assert_eq!(projected.recent_buy_rows.len(), VENDOR_RECENT_BUY_CAPACITY);
    assert_eq!(projected.recent_buy_rows[0].restore_list_id, 1);
    assert_eq!(projected.recent_buy_rows[14].restore_list_id, 15);
    assert_eq!(projected.recent_buy_rows[2].price, Some((10 + 3) * 3));

    let invalid = projection(
        &[
            VendorCatalogEntry0104 {
                source_slot_id: 0,
                item: empty(),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 1,
                item: item(0, -1, 0, 0),
            },
            VendorCatalogEntry0104 {
                source_slot_id: 49,
                item: item(0, 1, 0, 0),
            },
        ],
        &[],
        &[],
    );
    assert_eq!(invalid.catalog_rows.len(), 1);
    assert_eq!(invalid.catalog_rows[0].source_slot_id, 49);
}

#[test]
fn asset_contract_is_semantic_relative_and_rejects_escape() {
    let contract = VendorUiAssetContract::default();
    assert_eq!(contract.paths().count(), VendorStaticAssetRole::COUNT + 2);
    assert!(contract.paths().all(is_safe_relative_asset_path));
    assert_eq!(
        contract.image_path(VendorStaticAssetRole::VendorBackplate),
        VENDOR_PANEL_PATH
    );
    assert_eq!(contract.font_path(), USER_EQUIP_FONT_PATH);
    assert_eq!(contract.service_font_path(), VENDOR_SERVICE_FONT_PATH);

    let mut unsafe_images = VENDOR_UI_DEFAULT_IMAGE_PATHS.map(str::to_owned);
    unsafe_images[VendorStaticAssetRole::Info.index()] =
        "../patched/vendor-info.png".to_owned();
    assert!(matches!(
        VendorUiAssetContract::new(unsafe_images, USER_EQUIP_FONT_PATH.to_owned()),
        Err(VendorUiAssetContractError::UnsafeImagePath { index: 3, .. })
    ));
}
