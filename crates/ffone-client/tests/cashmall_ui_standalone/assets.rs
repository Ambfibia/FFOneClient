use super::*;

#[test]
fn semantic_asset_contract_rejects_traversal_and_wrong_ownership() {
    let mut paths = CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104.map(str::to_owned);
    paths[CashmallStaticAssetRole0104::Cash.index()] = "../cash.png".to_owned();
    assert!(matches!(
        CashmallUiAssetContract0104::new(paths, "fonts/jeffe.otf".to_owned()),
        Err(CashmallUiAssetContractError0104::Image {
            role: CashmallStaticAssetRole0104::Cash,
            ..
        })
    ));

    let mut paths = CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104.map(str::to_owned);
    paths[CashmallStaticAssetRole0104::Cash.index()] = "ui/en/vendor/cashmall-cash.png".to_owned();
    assert!(CashmallUiAssetContract0104::new(paths, "fonts/jeffe.otf".to_owned()).is_err());
    assert!(CashmallIconRef0104::new("icons/items/general/generalitemicon_00.png").is_ok());
    assert!(CashmallIconRef0104::new("icons/../ui/cash.png").is_err());
    assert!(CashmallIconRef0104::new("ui/en/cashmall/cash.png").is_err());
}

#[test]
fn default_asset_contract_accepts_the_approved_font_root_and_rejects_ui_fonts() {
    let contract = CashmallUiAssetContract0104::default();
    assert_eq!(contract.font_path(), "fonts/jeffe.otf");
    assert!(
        CashmallUiAssetContract0104::new(
            CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104.map(str::to_owned),
            "ui/fonts/jeffe.otf".to_owned(),
        )
        .is_err()
    );
}
