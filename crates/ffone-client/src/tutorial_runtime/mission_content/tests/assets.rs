use super::*;

#[test]
fn normal_warp_npc_lookup_uses_first_serialized_row_not_warp_id_order() {
    let mut document = compact_document();
    push_gameplay_warp_row(&mut document, 9_001, 9_001, false);
    push_gameplay_warp_row(&mut document, 9_001, 50, false);
    push_gameplay_warp_row(&mut document, 664, 51, false);
    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();

    let first = content.first_gameplay_warp_for_npc(9_001).unwrap();
    assert_eq!((first.row_index, first.warp_id), (4, 9_001));
    assert_eq!(content.gameplay_warp(50).unwrap().row_index, 5);
    assert!(content.first_gameplay_warp_for_npc(664).is_some());
    assert_eq!(
        content.normal_gameplay_warp_for_npc(664),
        None,
        "service category 3 must not expose the clean WARP action"
    );
    assert_eq!(content.first_gameplay_warp_for_npc(i32::MAX), None);
}

#[test]
fn compact_optional_user_equip_tables_cover_every_avatar_util_route() {
    const ICON_PATHS: [&str; 14] = [
        "icons/items/cosmetics/cosicon_835.png",
        "icons/items/cosmetics/cosicon_712.png",
        "icons/items/cosmetics/cosicon_721.png",
        "icons/items/cosmetics/cosicon_193.png",
        "icons/items/cosmetics/cosicon_196.png",
        "icons/items/cosmetics/cosicon_82.png",
        "icons/items/weapons/wpnicon_13.png",
        "icons/items/vehicles/vehicle_03.png",
        "icons/items/general/generalitemicon_00.png",
        "icons/entities/nanos/nanoicon_eddy.png",
        "icons/skills/skillicon_19.png",
        "icons/entities/hnpc/hnpcicon_01.png",
        "icons/skills/skillicon_11.png",
        "icons/items/general/generalitemicon_09.png",
    ];
    let fixture = Fixture::with_icon_paths(compact_user_equip_document(), &ICON_PATHS);
    let content = TutorialMissionContent::from_project_assets(&fixture.assets).unwrap();

    for (
        item_table,
        item_subtable,
        item_row_id,
        icon_subtable,
        declared_item_number,
        icon_type,
        icon_number,
        path,
    ) in [
        (17, 0, 1, 2, 1, 3, 835, ICON_PATHS[0]),
        (19, 0, 1, 2, 1, 3, 712, ICON_PATHS[1]),
        (20, 0, 1, 2, 1, 3, 721, ICON_PATHS[2]),
        (22, 0, 1, 2, 1, 3, 193, ICON_PATHS[3]),
        (23, 0, 1, 2, 1, 3, 196, ICON_PATHS[4]),
        (24, 0, 1, 2, 1, 3, 82, ICON_PATHS[5]),
        (25, 0, 1, 2, 1, 0, 13, ICON_PATHS[6]),
        (26, 0, 1, 2, 1, 12, 3, ICON_PATHS[7]),
        (28, 0, 1, 2, 1, 7, 0, ICON_PATHS[8]),
        (9, 0, 1, 3, 1, 1, 20, ICON_PATHS[9]),
        (9, 0, 1, 6, 1, 2, 19, ICON_PATHS[10]),
        (10, 0, 1, 3, 2672, 10, 1, ICON_PATHS[11]),
        (12, 0, 1, 2, 1, 2, 11, ICON_PATHS[12]),
        (12, 1, 1, 2, 1, 2, 11, ICON_PATHS[12]),
        (27, 0, 1, 2, 1, 7, 9, ICON_PATHS[13]),
    ] {
        let definition = content
            .gameplay_user_equip_icon(item_table, item_subtable, item_row_id, icon_subtable)
            .unwrap();
        assert_eq!(definition.declared_item_number, declared_item_number);
        assert_eq!(
            (definition.icon_type, definition.icon_number),
            (icon_type, icon_number)
        );
        assert_eq!(definition.icon_path.as_deref(), Some(path));
        assert!(fixture.assets.root().join(path).is_file());
    }
    assert_eq!(
        content.gameplay_npc_portrait_icon_path(2672),
        Some("icons/entities/hnpc/hnpcicon_01.png")
    );
    assert_eq!(content.gameplay_npc_portrait_icon_path(i32::MAX), None);

    let compact = Fixture::new(compact_document());
    let compact = TutorialMissionContent::from_project_assets(&compact.assets).unwrap();
    assert!(
        compact.gameplay_user_equip_icon(17, 0, 1, 2).is_none(),
        "the new gear tables remain strictly optional"
    );
}

#[test]
fn nearest_xcom_uses_clean_row_index_zone_zero_skip_and_full_3d_distance() {
    let rows = [
        GameplayXcomDefinition {
            row_index: 0,
            xcom_number: 0,
            zone: 0,
            position: [1, 1, 1],
        },
        GameplayXcomDefinition {
            row_index: 1,
            xcom_number: 99,
            zone: 0,
            position: [0, 0, 0],
        },
        GameplayXcomDefinition {
            row_index: 2,
            xcom_number: 200,
            zone: 1,
            position: [101, 100, 100],
        },
        GameplayXcomDefinition {
            row_index: 3,
            xcom_number: 300,
            zone: 0,
            position: [110, 100, 100],
        },
        GameplayXcomDefinition {
            row_index: 4,
            xcom_number: 400,
            zone: 0,
            position: [101, 100, 130],
        },
    ];
    assert_eq!(
        clean_nearest_xcom_index(&rows, 0, [100, 100, 100]),
        Some(3),
        "the request carries row index 3, not m_iXcomNumber 300"
    );
    assert_eq!(clean_nearest_xcom_index(&rows, 1, [100, 100, 100]), Some(2));
    assert_eq!(clean_nearest_xcom_index(&rows, 2, [100, 100, 100]), None);

    let mut document = compact_document();
    document
        .pointer_mut("/tables/0/value")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            "m_pXComTable".to_owned(),
            json!({
                "m_pXComData": rows.map(|row| json!({
                    "m_iFree": 1,
                    "m_iRegenRangeMax": 0,
                    "m_iRegenRangeMin": 0,
                    "m_iWarpNumber": 0,
                    "m_iXcomNumber": row.xcom_number,
                    "m_iXpos": row.position[0],
                    "m_iYpos": row.position[1],
                    "m_iZone": row.zone,
                    "m_iZpos": row.position[2],
                }))
            }),
        );
    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();
    assert_eq!(content.nearest_xcom_index(0, [100, 100, 100]), Some(3));

    let compact =
        TutorialMissionContent::from_project_assets(&Fixture::new(compact_document()).assets)
            .unwrap();
    assert_eq!(
        compact.nearest_xcom_index(0, [100, 100, 100]),
        None,
        "an omitted optional table remains unresolved"
    );
}

#[test]
fn optional_general_item_catalog_resolves_resurrection_item_semantics() {
    let mut document = compact_document();
    document
        .pointer_mut("/tables/0/value")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            "m_pGeneralItemTable".to_owned(),
            json!({
                "m_pItemData": [
                    {
                        "m_iItemNumber": 7,
                        "m_iItemType": 2,
                        "m_iLinkSkill": 17,
                        "m_iIcon": 0
                    },
                    {
                        "m_iItemNumber": 119,
                        "m_iItemType": 3,
                        "m_iLinkSkill": 144,
                        "m_iStimPackAttri": 1,
                        "m_iIcon": 3
                    },
                    {
                        "m_iItemNumber": 134,
                        "m_iItemType": 10,
                        "m_iLinkSkill": 18,
                        "m_iIcon": 1
                    },
                    {
                        "m_iItemNumber": 167,
                        "m_iItemType": 10,
                        "m_iLinkSkill": 19,
                        "m_iIcon": 2
                    }
                ],
                "m_pItemIconData": [
                    { "m_iIconNumber": 42, "m_iIconType": 7 },
                    { "m_iIconNumber": 143, "m_iIconType": 7 },
                    { "m_iIconNumber": 5, "m_iIconType": 7 },
                    { "m_iIconNumber": 13, "m_iIconType": 7 }
                ]
            }),
        );
    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();
    assert_eq!(content.general_item_type(7), Some(2));
    assert_eq!(content.general_item_type(119), Some(3));
    assert_eq!(content.general_item_stim_pack_attribute(119), Some(1));
    assert_eq!(content.general_item_type(134), Some(10));
    assert_eq!(content.general_item_type(167), Some(10));
    assert_eq!(content.general_item_type(999), None);
    assert_eq!(
        content.general_item(134),
        Some(&GameplayGeneralItemUiDefinition {
            item_id: 134,
            item_type: 10,
            link_skill: Some(18),
            stim_pack_attribute: None,
            icon_type: Some(7),
            icon_number: Some(143),
            icon_path: Some("icons/items/general/generalitemicon_143.png".to_owned()),
        })
    );
    assert_eq!(
        content.general_item_icon_path(167),
        Some("icons/items/general/generalitemicon_05.png")
    );

    let compact =
        TutorialMissionContent::from_project_assets(&Fixture::new(compact_document()).assets)
            .unwrap();
    assert_eq!(compact.general_item_type(134), None);
}

#[test]
fn gameplay_npc_catalog_covers_non_tutorial_rows_and_gates_zero_hp() {
    let fixture = Fixture::new(compact_document());
    let content = TutorialMissionContent::from_project_assets(&fixture.assets).unwrap();
    let officer = content.gameplay_npc(664).unwrap();
    assert_eq!(
        officer,
        &GameplayNpcUiDefinition {
            npc_type: 664,
            name: "Time Squad Officer James".to_owned(),
            greeting_string_id: 7,
            greeting: " ".to_owned(),
            barker: None,
            team: 1,
            npc_level: 1,
            npc_style: 0,
            attack_effect: 0,
            service_category: 3,
            service_number: None,
            npc_class: 3,
            ai_type: 0,
            mesh_id: None,
            table_scale: None,
            attack_range_server_units: None,
            move_voice_owner: String::new(),
            radius_server_units: 120,
            height_server_units: 190,
            sight_range_server_units: 700,
            max_hp: 439,
        }
    );
    assert_eq!(officer.hp_fraction(-1), Some(0.0));
    assert_eq!(officer.hp_fraction(439), Some(1.0));
    assert_eq!(officer.hp_fraction(1_000), Some(1.0));
    assert!(content.gameplay_npc(0).is_none());
    assert!(content.gameplay_npc(i32::MAX).is_none());
    assert_eq!(
        content.npcs().len(),
        TUTORIAL_MISSION_NPC_TYPES.len(),
        "the reusable catalog must not widen the tutorial-only NPC subset"
    );

    let mut noncombat = compact_document();
    npc_rows_mut(&mut noncombat)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iHP"] = json!(0);
    let noncombat =
        TutorialMissionContent::from_project_assets(&Fixture::new(noncombat).assets).unwrap();
    let officer = noncombat.gameplay_npc(664).unwrap();
    assert_eq!(officer.max_hp, 0);
    assert_eq!(officer.hp_fraction(0), None);
    assert_eq!(officer.hp_fraction(100), None);
}

#[test]
fn world_map_catalog_requires_an_exact_map_icon_and_never_infers_missions() {
    let compact =
        TutorialMissionContent::from_project_assets(&Fixture::new(compact_document()).assets)
            .unwrap();
    assert_eq!(compact.gameplay_npc_map_icon(664), None);
    assert_eq!(compact.gameplay_npc_minimap(664), None);
    assert_eq!(
        WorldMapCatalog::npc(&compact, 664),
        WorldMapCatalogLookup::Missing
    );

    let mut document = compact_document();
    npc_rows_mut(&mut document)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iMapIcon"] = json!(19);
    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();
    assert_eq!(content.gameplay_npc_map_icon(664), Some(19));
    assert_eq!(
        content.gameplay_npc_minimap(664),
        Some(GameplayNpcMinimapDefinition {
            npc_type: 664,
            npc_class: 3,
            sound: 1,
            map_icon: 19,
        })
    );
    assert_eq!(
        WorldMapCatalog::npc(&content, 664),
        WorldMapCatalogLookup::Unique(WorldMapNpcCatalogEntry {
            display_name: "Time Squad Officer James".to_owned(),
            map_icon: 19,
            mission: WorldMapMissionAvailability::None,
        })
    );
}

#[test]
fn stable_table_path_and_single_document_are_verified() {
    let mut with_routes = compact_document();
    with_routes["tables"].as_array_mut().unwrap().insert(
        0,
        json!({
            "name": "native_asset_routes", "value": {}
        }),
    );
    let with_routes = Fixture::new(with_routes);
    assert!(TutorialMissionContent::open(&with_routes.assets).is_ok());

    let length_fixture = Fixture::new(compact_document());
    fs::write(&length_fixture.table_path, b"tampered").unwrap();
    let error =
        TutorialMissionContent::from_project_assets(&length_fixture.assets).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("invalid tutorial table-set JSON")
    );

    let hash_fixture = Fixture::new(compact_document());
    let mut bytes = fs::read(&hash_fixture.table_path).unwrap();
    bytes[0] ^= 1;
    fs::write(&hash_fixture.table_path, bytes).unwrap();
    let error = TutorialMissionContent::from_project_assets(&hash_fixture.assets).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("invalid tutorial table-set JSON")
    );

    let mut duplicate_table = compact_document();
    let table = duplicate_table["tables"][0].clone();
    duplicate_table["tables"]
        .as_array_mut()
        .unwrap()
        .push(table);
    let duplicate_table = Fixture::new(duplicate_table);
    assert!(
        TutorialMissionContent::from_project_assets(&duplicate_table.assets)
            .unwrap_err()
            .to_string()
            .contains("exactly one table document")
    );

    let duplicate_route = Fixture::with_duplicate_table_route(compact_document());
    let content = TutorialMissionContent::open(&duplicate_route.assets).unwrap();
    assert_eq!(content.provenance.asset_path, TABLE_SET_PATH);
}

#[test]
fn vendor_catalog_metadata_is_table_backed_and_malformed_rows_fail_closed() {
    let mut document = compact_user_equip_document();
    let value = document["tables"][0]["value"].as_object_mut().unwrap();
    value["m_pHatItemTable"] = json!({
        "m_pItemData": [
            {
                "m_iItemNumber": 0,
                "m_iIcon": 0,
                "m_iItemName": 0,
                "m_iItemPrice": 0,
                "m_iItemSellPrice": 0,
                "m_iSellAble": 0,
                "m_iMinReqLev": 0,
                "m_iPointRat": 0, "m_iGroupRat": 0, "m_iDefenseRat": 0,
                "m_iEquipType": 0, "m_iTargetMode": 0, "m_iRarity": 0, "m_iTradeAble": 1
            },
            {
                "m_iItemNumber": 1,
                "m_iIcon": 1,
                "m_iItemName": 1,
                "m_iItemPrice": 425,
                "m_iItemSellPrice": 106,
                "m_iSellAble": 1,
                "m_iMinReqLev": 7,
                "m_iPointRat": 0, "m_iGroupRat": 0, "m_iDefenseRat": 0,
                "m_iEquipType": 0, "m_iTargetMode": 0, "m_iRarity": 0, "m_iTradeAble": 1
            }
        ],
        "m_pItemIconData": [
            { "m_iIconType": 0, "m_iIconNumber": 0 },
            { "m_iIconType": 3, "m_iIconNumber": 721 }
        ],
        "m_pItemStringData": [
            { "m_strName": "", "m_strComment": "" },
            { "m_strName": "Exact Fixture Hat", "m_strComment": "Fixture description" }
        ]
    });
    value.insert(
        "m_pVendorTable".to_owned(),
        json!({
            "m_pItemData": [{
                "m_iNpcNumber": 664,
                "m_iSortNumber": 1,
                "m_iItemType": 4,
                "m_iitemID": 1,
                "m_iSellCost": 0
            }]
        }),
    );
    let icon_path = "icons/items/cosmetics/cosicon_721.png";
    let fixture = Fixture::with_icon_paths(document.clone(), &[icon_path]);
    let content = TutorialMissionContent::from_project_assets(&fixture.assets).unwrap();
    assert_eq!(
        content.resolve(ItemBase0104 {
            item_type: 4,
            item_id: 1,
            option: 0,
            time_limit: 0,
        }),
        Some(VendorItemMetadata0104 {
            name: "Exact Fixture Hat".to_owned(),
            level: 7,
            buy_price: 425,
            sell_price: 106,
            sellable: true,
            general_item_type: None,
            battery_recharge: None,
            stack_size: None,
            icon: Some(VendorIconRef::new(icon_path).unwrap()),
        })
    );

    document["tables"][0]["value"]["m_pVendorTable"]["m_pItemData"][0]
        .as_object_mut()
        .unwrap()
        .remove("m_iitemID");
    let malformed = Fixture::new(document);
    assert!(matches!(
        TutorialMissionContent::from_project_assets(&malformed.assets),
        Err(TutorialMissionContentError::Invalid(message))
            if message.contains("m_iitemID")
    ));
}
