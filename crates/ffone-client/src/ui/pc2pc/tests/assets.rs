use super::*;

pub(super) struct Catalog;

impl UserEquipItemCatalog for Catalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        UserEquipIconRef::new(format!(
            "icons/items/test/{}-{}.png",
            query.item_type, query.item_row_id
        ))
        .ok()
    }
}

#[test]
fn default_asset_contract_is_safe_complete_and_source_exact() {
    let contract = Pc2pcUiAssetContract::default();
    contract.validate().unwrap();
    assert_eq!(contract.image_paths.len(), Pc2pcStaticAssetRole::COUNT);
    assert_eq!(contract.font_path, PC2PC_JEFFE_FONT_PATH);
    assert_eq!(contract.chalet_font_path, PC2PC_CHALET_FONT_PATH);

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for path in &contract.image_paths {
        assert!(asset_root.join(path).is_file(), "missing {path}");
    }
    assert!(asset_root.join(&contract.font_path).is_file());
    assert!(asset_root.join(&contract.chalet_font_path).is_file());

    let expected = [
        (
            PC2PC_TRADE_BACK_PATH,
            "DA8131CC458AED9D97E57C193B5B298134E7A967CC17EDF08769FFFE85A5D455",
        ),
        (
            PC2PC_TRADE_AREA_PATH,
            "1957E0CD33CF9324DD7554C13BC3A54FBEBF929CA847FA342A443BA4A3426032",
        ),
        (
            PC2PC_LOCAL_OFFER_PATH,
            "CBF00F103FE9B373357B0B18F44B512FC49F9FDBBA56569E6CE5062B374CF830",
        ),
        (
            PC2PC_LOCAL_OFFER_READY_PATH,
            "C4089C42C6812516B05628E679977CFEB251928DEB7914BD203F06AE23A9B2A1",
        ),
        (
            PC2PC_REMOTE_OFFER_PATH,
            "BCD891556736C3AD3EC42CDEDDC9104C77A03896CFDFE4B5B473D7A1613FE189",
        ),
        (
            PC2PC_REMOTE_OFFER_READY_PATH,
            "A34F19CE1F10E07480D2BA9CA0EB2D393E7A2F9F7C458C2909AEAFB2710DD314",
        ),
        (
            PC2PC_CHAT_BOX_PATH,
            "C0EC818FA58E6AF880064B7E5FF3A545EF8F231006C2C0447234853A68279AAC",
        ),
        (
            PC2PC_MONEY_BACK_PATH,
            "280470E3F89B2182AA45C548E7FEEBCBFD49894F2B6DCABAF5DE59E9BFFAF143",
        ),
        (
            PC2PC_TAROS_PATH,
            "138BB01C82AB7AD0748ADF35565BDCF3AE6F44B38C552CAEFD61000B9283039D",
        ),
        (
            PC2PC_FREE_CHAT_PATH,
            "D433073C5550F7B4D959158D4B6C3F7DE7AB75B00822097A1082C451BDBAD499",
        ),
    ];
    for (path, hash) in expected {
        let bytes = fs::read(asset_root.join(path)).unwrap();
        assert_eq!(format!("{:X}", Sha256::digest(bytes)), hash);
    }
}
