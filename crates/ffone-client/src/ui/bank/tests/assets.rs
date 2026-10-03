use super::*;

pub(super) struct AllCatalog;

impl UserEquipItemCatalog for AllCatalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        UserEquipIconRef::new(format!(
            "icons/items/test/type{}-{}.png",
            query.item_type, query.item_row_id
        ))
        .ok()
    }
}

pub(super) struct MissingCatalog;

impl UserEquipItemCatalog for MissingCatalog {
    fn resolve_icon(&self, _query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        None
    }
}

#[test]
fn asset_contract_is_semantic_relative_and_rejects_path_escape() {
    let contract = BankUiAssetContract::default();
    assert_eq!(contract.paths().count(), BankStaticAssetRole::COUNT + 2);
    assert!(contract.paths().all(is_safe_relative_asset_path));
    assert_eq!(
        contract.image_path(BankStaticAssetRole::TradeBack),
        BANK_TRADE_BACK_PATH
    );
    assert_eq!(contract.font_path(), USER_EQUIP_FONT_PATH);
    assert_eq!(
        contract.image_path(BankStaticAssetRole::BankSlotButton),
        BANK_SLOT_BUTTON_PATH
    );
    assert_eq!(
        contract.image_path(BankStaticAssetRole::DexlabsBanner),
        BANK_DEXLABS_PATH
    );
    assert_eq!(
        contract.image_path(BankStaticAssetRole::TarosCounter),
        BANK_TAROS_COUNTER_PATH
    );

    let mut unsafe_images = BANK_UI_DEFAULT_IMAGE_PATHS.map(str::to_owned);
    unsafe_images[BankStaticAssetRole::BankPanel.index()] =
        "../legacy/bankslotback.png".to_owned();
    assert!(matches!(
        BankUiAssetContract::new(unsafe_images, USER_EQUIP_FONT_PATH.to_owned()),
        Err(BankUiAssetContractError::UnsafeImagePath { index: 3, .. })
    ));
}
