use super::*;

#[derive(Default)]
pub(super) struct TestCatalog {
    pub(super) icons: BTreeMap<UserEquipCatalogQuery, UserEquipIconRef>,
}

impl TestCatalog {
    pub(super) fn insert(&mut self, query: UserEquipCatalogQuery, path: &str) {
        self.icons
            .insert(query, UserEquipIconRef::new(path).unwrap());
    }
}

impl UserEquipItemCatalog for TestCatalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        self.icons.get(&query).cloned()
    }
}

pub(super) struct AllCatalog;

impl UserEquipItemCatalog for AllCatalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        UserEquipIconRef::new(format!(
            "icons/table-{}/row-{}.png",
            query.kind.item_table(),
            query.item_row_id
        ))
        .ok()
    }
}

#[test]
fn semantic_asset_contract_rejects_hashed_source_and_traversal_routes() {
    let contract = UserEquipUiAssetContract::default();
    assert_eq!(contract.paths(), USER_EQUIP_DEFAULT_ASSET_PATHS);

    let mut hashed_source = USER_EQUIP_DEFAULT_ASSET_PATHS.map(str::to_owned);
    hashed_source[UserEquipStaticAssetRole::Backdrop.index()] =
        USER_EQUIP_SOURCE_BACKDROP_PATH.to_owned();
    assert_eq!(
        UserEquipUiAssetContract::try_from_semantic_paths(hashed_source)
            .unwrap_err()
            .role,
        UserEquipStaticAssetRole::Backdrop
    );

    let mut traversal = USER_EQUIP_DEFAULT_ASSET_PATHS.map(str::to_owned);
    traversal[UserEquipStaticAssetRole::Close.index()] =
        "ui/en/user-equip/../close.png".to_owned();
    assert_eq!(
        UserEquipUiAssetContract::try_from_semantic_paths(traversal)
            .unwrap_err()
            .role,
        UserEquipStaticAssetRole::Close
    );

    let mut wrong_font = USER_EQUIP_DEFAULT_ASSET_PATHS.map(str::to_owned);
    wrong_font[UserEquipStaticAssetRole::Font.index()] =
        "ui/en/gameplay/fonts/JEFFE.png".to_owned();
    assert_eq!(
        UserEquipUiAssetContract::try_from_semantic_paths(wrong_font)
            .unwrap_err()
            .role,
        UserEquipStaticAssetRole::Font
    );
    assert!(is_user_equip_semantic_icon_path(
        "icons/items/general/generalitemicon_00.png"
    ));
    assert!(!is_user_equip_semantic_icon_path(
        "textures/generalitemicon_00.png"
    ));
    assert!(!is_user_equip_semantic_icon_path(
        "icons/items/../generalitemicon_00.png"
    ));
}

#[test]
fn production_table_data_catalog_resolves_installed_icons_and_combined_look_rows() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let assets = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();

    for (source, expected_path) in [
        (item(6, 1, 0, 0), "icons/items/cosmetics/cosicon_835.png"),
        (item(5, 1, 0, 0), "icons/items/cosmetics/cosicon_712.png"),
        (item(4, 1, 0, 0), "icons/items/cosmetics/cosicon_721.png"),
        (item(2, 1, 0, 0), "icons/items/cosmetics/cosicon_193.png"),
        (item(1, 1, 0, 0), "icons/items/cosmetics/cosicon_196.png"),
        (item(3, 1, 0, 0), "icons/items/cosmetics/cosicon_82.png"),
        (item(0, 1, 0, 0), "icons/items/weapons/wpnicon_13.png"),
        (item(10, 1, 0, 0), "icons/items/vehicles/vehicle_03.png"),
        (
            item(9, 1, 0, 0),
            "icons/items/general/generalitemicon_00.png",
        ),
        (
            item(19, 1, 0, 0),
            "icons/entities/nanos/nanoicon_buttercup.png",
        ),
        (item(24, 1, 0, 0), "icons/skills/skillicon_19.png"),
        (item(30, 664, 0, 0), "icons/entities/hnpc/hnpcicon_153.png"),
        (item(27, 1, 0, 0), "icons/skills/skillicon_10.png"),
        (item(138, 1, 0, 0), "icons/skills/skillicon_14.png"),
        (
            item(7, 2, 0, 0),
            "icons/items/general/generalitemicon_09.png",
        ),
    ] {
        let query = UserEquipCatalogQuery::from_non_empty_item(source).unwrap();
        let icon = content.resolve_icon(query).unwrap();
        assert_eq!(icon.runtime_path(), expected_path);
        assert!(
            root.join(expected_path).is_file(),
            "catalog returned an uninstalled path {expected_path}"
        );
    }

    let base = UserEquipCatalogQuery::from_non_empty_item(item(0, 1, 0, 0)).unwrap();
    let combined =
        UserEquipCatalogQuery::from_non_empty_item(item(0, 1, 2_i32 << 16, 0)).unwrap();
    assert_eq!(
        (base.base_item_id, base.item_row_id, base.combined_look_id),
        (1, 1, None)
    );
    assert_eq!(
        (
            combined.base_item_id,
            combined.item_row_id,
            combined.combined_look_id
        ),
        (1, 2, Some(2))
    );
    assert_eq!(
        content.resolve_icon(base).unwrap().runtime_path(),
        "icons/items/weapons/wpnicon_13.png"
    );
    assert_eq!(
        content.resolve_icon(combined).unwrap().runtime_path(),
        "icons/items/weapons/wpnicon_37.png",
        "AvatarUtil resolves the combined look row while retaining the base item identity"
    );

    assert!(
        content
            .resolve_icon(UserEquipCatalogQuery::from_non_empty_item(item(8, 1, 0, 0)).unwrap())
            .is_none(),
        "Quest keeps AvatarUtil's early-null quirk"
    );
    assert!(
        content
            .resolve_icon(
                UserEquipCatalogQuery::from_non_empty_item(item(10, 10, 0, 0)).unwrap()
            )
            .is_none(),
        "a valid TableData row whose native icon file is absent must fail closed"
    );

    let mut inconsistent = combined;
    inconsistent.item_row_id = 1;
    assert!(
        content.resolve_icon(inconsistent).is_none(),
        "manually inconsistent base/look queries must not bypass the exact adapter"
    );
}

#[test]
fn empty_malformed_quest_and_catalog_miss_project_fail_closed() {
    let noncanonical_empty = item(141, 0, 0x1234_0002_u32 as i32, 999);
    let missing_gear = item(0, 10, 0, 0);
    let quest = item(8, 20, 0, 0);
    let malformed = item(-1, 30, 0, 0);
    let resolved = item(7, 40, 3, 0);
    let resolved_query = UserEquipCatalogQuery::from_non_empty_item(resolved).unwrap();
    let mut catalog = TestCatalog::default();
    catalog.insert(resolved_query, "icons/items/generalitemicon_40.png");
    // Even a supplied Quest mapping must not erase the clean early-null quirk.
    catalog.insert(
        UserEquipCatalogQuery::from_non_empty_item(quest).unwrap(),
        "icons/items/questitemicon_20.png",
    );
    let runtime = runtime_with(
        &[],
        &[
            (0, noncanonical_empty),
            (1, missing_gear),
            (2, quest),
            (3, malformed),
            (4, resolved),
        ],
    );
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &catalog);

    assert!(projection.inventory[0].item.empty);
    assert_eq!(
        projection.inventory[0].item.item, noncanonical_empty,
        "empty serialized fields remain lossless"
    );
    assert_eq!(
        projection.inventory[0].item.icon,
        UserEquipProjectedIcon::Empty
    );
    assert!(matches!(
        projection.inventory[1].item.icon,
        UserEquipProjectedIcon::MissingChecker(UserEquipMissingIconReason::CatalogMiss { .. })
    ));
    assert!(matches!(
        projection.inventory[2].item.icon,
        UserEquipProjectedIcon::MissingChecker(
            UserEquipMissingIconReason::QuestLegacyNull { .. }
        )
    ));
    assert_eq!(
        projection.inventory[3].item.icon,
        UserEquipProjectedIcon::MissingChecker(UserEquipMissingIconReason::MalformedIdentity {
            item_type: -1,
            item_id: 30,
        })
    );
    assert_eq!(
        projection.inventory[4].item.icon,
        UserEquipProjectedIcon::Resolved(
            UserEquipIconRef::new("icons/items/generalitemicon_40.png").unwrap()
        )
    );
    assert_eq!(
        UserEquipIconRef::new("   "),
        Err(UserEquipIconRefError::EmptyRuntimePath)
    );
}
