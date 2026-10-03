use super::*;

#[test]
fn inventory_availability_tracks_guide_level_and_base_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let assets = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    // Shipped weapon row 2 requires Ben (4), level 24. Its icon can be
    // combined with row 1 without changing the base requirements.
    let mut item = ffone_protocol::ItemBase0104 {
        item_type: 0, item_id: 2, option: 1, time_limit: 0,
    };
    assert!(!content.gameplay_inventory_item_available(item, 36, 1, 2));
    assert!(content.gameplay_inventory_item_available(item, 36, 1, 4));
    assert!(!content.gameplay_inventory_item_available(item, 23, 1, 4));
    assert!(content.gameplay_inventory_item_available(item, 24, 1, 4));
    item.option = (1 << 16) | 1;
    assert!(!content.gameplay_inventory_item_available(item, 36, 1, 2));
    assert!(content.gameplay_inventory_item_available(item, 36, 1, 4));
    // Find a published gender-restricted row rather than depending on its name.
    let (&(item_type, item_id), metadata) = content.gameplay_vendor_item_metadata.iter()
        .find(|(_, m)| matches!(m.try_on_gender, 1 | 2)).unwrap();
    item.item_type = item_type;
    item.item_id = item_id;
    item.option = 1;
    assert!(content.gameplay_inventory_item_available(item, 36, metadata.try_on_gender, metadata.try_on_guide));
    assert!(!content.gameplay_inventory_item_available(item, 36, 3 - metadata.try_on_gender, metadata.try_on_guide));
    item.item_type = 8; // Quest items have no equipment requirements.
    assert!(content.gameplay_inventory_item_available(item, 1, 1, 0));
}

#[test]
fn real_assets_game_table_set_has_exact_tutorial_provenance_when_available() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    if !root.join(TABLE_SET_PATH).is_file() {
        return;
    }
    let assets = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    let late_nano = content
        .journal_nano(48)
        .expect("complete optional Nano 48 should be projected");
    assert_eq!(
        late_nano
            .skills
            .iter()
            .map(|skill| (skill.tune_id, skill.skill_id))
            .collect::<Vec<_>>(),
        vec![(199, 228), (200, 229), (201, 230)]
    );
    assert!(content.journal_nano(51).is_none());

    assert_eq!(content.provenance.asset_path, TABLE_SET_PATH);
    assert_eq!(content.provenance.schema, TABLE_SET_SCHEMA);
    assert_eq!(content.provenance.table_name, CONSOLIDATED_TABLE);
    assert_eq!(content.gameplay_npc_barker_type(651), Some(1));
    assert_eq!(
        content.mission(2).unwrap().provenance.barker_text_ids,
        [12, 13, 14, 15]
    );
    assert_eq!(
        content.gameplay_mission_name_string(12),
        Some("You saved Dee Dee's garden? Gee, that's swell.")
    );
    assert_eq!(
        content.gameplay_npc(664),
        Some(&GameplayNpcUiDefinition {
            npc_type: 664,
            name: "Time Squad Officer James".to_owned(),
            greeting_string_id: 664,
            greeting:
                "Yeah, Larry's so incompetent Time Squad sent in more of us....Me? Oh I'm here for morale!"
                    .to_owned(),
            barker: None,
            team: 1,
            npc_level: 1,
            npc_style: 0,
            attack_effect: 824,
            service_category: 2,
            service_number: Some(9),
            npc_class: 2,
            ai_type: 0,
            mesh_id: Some(0),
            table_scale: Some(XdtF32::from_value(1.0)),
            attack_range_server_units: Some(600),
            move_voice_owner: "mvehicle".to_owned(),
            radius_server_units: 120,
            height_server_units: 190,
            sight_range_server_units: 700,
            max_hp: 439,
        })
    );
    assert_eq!(content.gameplay_npc_map_icon(664), Some(31));
    let edd_shop = content
        .gameplay_npc(643)
        .expect("production Edd guide shop NPC");
    assert_eq!(edd_shop.greeting_string_id, 643);
    assert_eq!(
        edd_shop.greeting,
        "If you've lost some of Edd's guide items, you can buy them back from me!"
    );
    assert_eq!(
        edd_shop.barker,
        Some(GameplayNpcBarkerDefinition {
            string_id: 428,
            lines: [
                "Missplaced Edd's valuable guide items? No worries, I can help you out."
                    .to_owned(),
                "Edd has some of the coolest gear around, including an Urban Ranger outfit, a Bloo Skate Set, and the SACT Armor!"
                    .to_owned(),
                "Come to me if you need to grab the guide items you've lost. Buy some duplicates too, I could use the money."
                    .to_owned(),
                "We're here so you don't have to worry about misplacing your guide items. If you do, you can buy them back from us!"
                    .to_owned(),
            ],
        })
    );
    assert_eq!(
        WorldMapCatalog::npc(&content, 664),
        WorldMapCatalogLookup::Unique(WorldMapNpcCatalogEntry {
            display_name: "Time Squad Officer James".to_owned(),
            map_icon: 31,
            mission: WorldMapMissionAvailability::None,
        })
    );
    assert_eq!(
        content.gameplay_npc(2676),
        Some(&GameplayNpcUiDefinition {
            npc_type: 2676,
            name: "Oil Ogre".to_owned(),
            greeting_string_id: 2676,
            greeting: " ".to_owned(),
            barker: None,
            team: 2,
            npc_level: 1,
            npc_style: 1,
            attack_effect: 106,
            service_category: 0,
            service_number: Some(0),
            npc_class: 0,
            ai_type: 2,
            mesh_id: Some(113),
            table_scale: Some(XdtF32::from_value(1.2000000476837158)),
            attack_range_server_units: Some(1_680),
            move_voice_owner: String::new(),
            radius_server_units: 120,
            height_server_units: 350,
            sight_range_server_units: 1_800,
            max_hp: 1_300,
        })
    );
    assert_eq!(
        content.gameplay_npc_portrait_icon_path(2071),
        Some("icons/entities/mobs/mobicon_83.png"),
        "Great Shellslug must retain its exact m_iIcon1 portrait route"
    );
    assert_eq!(
        content.gameplay_npc_portrait_icon_path(2096),
        Some("icons/entities/mobs/mobicon_83.png")
    );
    assert_eq!(
        content.gameplay_npc_portrait_icon_path(2676),
        Some("icons/entities/mobs/mobicon_11.png")
    );
    for (npc_type, npc_class, radius, height, sight_range) in [
        (707, 18, 90, 190, 600),
        (728, 19, 90, 200, 700),
        (731, 20, 100, 170, 700),
        (732, 21, 90, 200, 700),
        (1171, 22, 120, 240, 600),
        (1175, 23, 150, 370, 1_500),
    ] {
        let guide = content.gameplay_npc(npc_type).unwrap();
        assert_eq!(guide.team, 1, "Guide NPC {npc_type} team");
        assert_eq!(guide.ai_type, 0, "Guide NPC {npc_type} AI type");
        assert_eq!(
            (guide.service_category, guide.npc_class),
            (npc_class, npc_class),
            "Guide NPC {npc_type} class/service"
        );
        assert_eq!(
            (
                guide.radius_server_units,
                guide.height_server_units,
                guide.sight_range_server_units,
            ),
            (radius, height, sight_range),
            "Guide NPC {npc_type} targeting geometry"
        );
    }
    assert_eq!(
        content.gameplay_warp(GUIDE_FIRST_WARP_ID).map(|warp| (
            warp.npc_type,
            warp.target,
            warp.warp_group_type,
            warp.limit_level,
            warp.limit_task_id,
            warp.limit_item_id,
            warp.limit_item_type,
            warp.limit_use_item_id,
            warp.limit_use_item_type,
            warp.mission_id,
            warp.is_instance,
            warp.cost,
        )),
        Some((
            GUIDE_FIRST_WARP_NPC_TYPE,
            GUIDE_FIRST_WARP_TARGET,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ))
    );
    let gameplay_warps = content.gameplay_warps().collect::<Vec<_>>();
    assert_eq!(gameplay_warps.len(), 321);
    assert_eq!(
        gameplay_warps
            .iter()
            .map(|warp| warp.npc_type)
            .collect::<BTreeSet<_>>()
            .len(),
        321,
        "published m_pWarpData has one serialized row per normal warp NPC"
    );
    assert_eq!(
        gameplay_warps.iter().filter(|warp| warp.cost != 0).count(),
        0
    );
    assert_eq!(
        gameplay_warps
            .iter()
            .filter(|warp| warp.limit_level != 0)
            .count(),
        8
    );
    assert_eq!(
        gameplay_warps
            .iter()
            .filter(|warp| warp.limit_item_id != 0)
            .count(),
        0
    );
    assert_eq!(
        gameplay_warps
            .iter()
            .filter(|warp| warp.limit_use_item_id != 0)
            .count(),
        2
    );
    assert_eq!(
        gameplay_warps
            .iter()
            .filter(|warp| warp.limit_task_id != 0)
            .count(),
        103
    );
    assert_eq!(
        gameplay_warps
            .iter()
            .filter(|warp| warp.warp_group_type == 1)
            .count(),
        221
    );
    assert_eq!(
        content.first_gameplay_warp_for_npc(681).map(|warp| (
            warp.warp_id,
            warp.limit_level,
            warp.limit_item_type,
            warp.limit_use_item_id,
            warp.limit_use_item_type,
            warp.warp_group_type,
        )),
        Some((5, 2, 0, 1, 7, 1))
    );
    assert_eq!(content.gameplay_npc(681).unwrap().service_category, 5);
    assert_eq!(
        content
            .normal_gameplay_warp_for_npc(681)
            .map(|warp| warp.warp_id),
        Some(5)
    );
    assert_eq!(
        content.gameplay_nano(1),
        Some(&GameplayNanoUiDefinition {
            nano_id: 1,
            sort_number: 5,
            name: "Buttercup".to_owned(),
            attribute: "Blastons".to_owned(),
            icon_number: 34,
            icon_path: Some("icons/entities/nanos/nanoicon_buttercup.png".to_owned()),
            ready_icon_path: Some(
                "icons/entities/nanos/ready/nanoready_buttercup.png".to_owned(),
            ),
            style: 1,
            max_stamina: 150,
        })
    );
    assert_eq!(content.gameplay_nano_tune_fusion_matter(36), Some(8_679));
    assert_eq!(content.gameplay_nano_tune_fusion_matter(41), None);
    let clean_nano_ready_numbers = [
        (1, 34),
        (2, 28),
        (3, 20),
        (4, 1),
        (5, 7),
        (6, 33),
        (7, 17),
        (8, 3),
        (9, 19),
        (10, 25),
        (11, 6),
        (12, 31),
        (13, 23),
        (14, 14),
        (15, 35),
        (16, 18),
        (17, 5),
        (18, 26),
        (19, 2),
        (20, 4),
        (21, 10),
        (22, 8),
        (23, 32),
        (24, 30),
        (25, 12),
        (26, 15),
        (27, 24),
        (28, 29),
        (29, 22),
        (30, 11),
        (31, 27),
        (32, 9),
        (33, 16),
        (34, 0),
        (35, 13),
        (36, 21),
    ];
    for (nano_id, icon_number) in clean_nano_ready_numbers {
        let nano = content
            .gameplay_nano(nano_id)
            .unwrap_or_else(|| panic!("clean Nano {nano_id}"));
        let slug = merged_nano_named_icon_slug(&nano.name)
            .unwrap_or_else(|| panic!("semantic slug for clean Nano {nano_id}"));
        let expected_path = format!("icons/entities/nanos/ready/nanoready_{slug}.png");
        assert_eq!(nano.icon_number, icon_number, "clean Nano {nano_id}");
        assert_eq!(
            nano.ready_icon_path.as_deref(),
            Some(expected_path.as_str()),
            "clean Nano {nano_id} must use its IconElement number, not its Nano ID or icon-row index"
        );
    }
    assert_eq!(
        content.gameplay_skill(1),
        Some(&GameplaySkillUiDefinition {
            skill_id: 1,
            skill_type: 8,
            values_a: [1; 4],
            icon_number: 10,
            active: true,
            effect_type: 1,
            target_effect: 0,
            target: 3,
            target_type: 1,
            range: 1000,
            area: 0,
            angle: 90,
            target_number: 2,
            cooldown: 80,
            cool_type: 22,
        })
    );

    let transmitter = content.mission(2248).unwrap();
    assert_eq!(transmitter.provenance.title_string_id, 11666);
    assert_eq!(transmitter.title, "Transmitter Critters");
    assert_eq!(transmitter.provenance.objective_string_id, 11672);
    assert_eq!(transmitter.objective, "Defeat the Oil Ogre.");
    assert_eq!(transmitter.mission_type, TutorialMissionType::World);
    assert_eq!(transmitter.provenance.mission_type, 3);
    assert_eq!(transmitter.provenance.journal_row_id, 2248);
    assert_eq!(
        transmitter.offer_description(),
        "This attack was no accident. An Oil Ogre is using a transmitter to broadcast our position. We need to find that monster and get that transmitter! You ready?"
    );
    assert_eq!(
        transmitter.active_description(),
        "Defeat the Oil Ogre.\n\nYou need to find the Oil Ogre that is broadcasting our position. Defeat him and get the transmitter. You can do it!"
    );
    assert_eq!(transmitter.provenance.grant_waypoint_npc_type, 2374);
    let transmitter_delivery = content.mission(2249).unwrap();
    assert_eq!(
        transmitter_delivery.provenance.grant_waypoint_npc_type,
        2671
    );
    assert_eq!(transmitter_delivery.provenance.journal_row_id, 2249);
    assert_eq!(
        transmitter_delivery.active_description(),
        "Deliver transmitter to Numbuh Two.\n\nYou've really got some skills. Good work defeating that monster! Now bring that transmitter you recovered back to me."
    );
    assert_eq!(
        transmitter_delivery.completion_description(),
        "Thanks! But the battle isn't over. Dexter says that you're the kid from the time machine. He's in trouble. You need to go to the infected zone right away."
    );

    let fusion = content.mission(2250).unwrap();
    assert_eq!(fusion.provenance.title_string_id, 11676);
    assert_eq!(fusion.title, "A Fusion Matter");
    assert_eq!(fusion.provenance.objective_string_id, 11682);
    assert_eq!(fusion.objective, "Talk to Buttercup.");
    assert_eq!(fusion.provenance.nano_id, 1);
    let buttercup = content.journal_nano(1).unwrap();
    assert_eq!(
        buttercup,
        &TutorialNanoJournalUi {
            nano_id: 1,
            icon_number: 34,
            name: "Buttercup".to_owned(),
            attribute: "Blastons".to_owned(),
            description: "REQUIRED NANO: Create to reach LVL 1.".to_owned(),
            skills: [
                TutorialNanoJournalSkillUi {
                    tune_id: 1,
                    skill_id: 1,
                    icon_number: 10,
                    name: "MISS FIRE".to_owned(),
                    type_label: "STUN - CONE".to_owned(),
                    required_item_id: 0,
                    required_item_count: 0,
                    description: "Buttercup’s alter-ego Mange uses fire to stun enemies in the target area.".to_owned(),
                },
                TutorialNanoJournalSkillUi {
                    tune_id: 2,
                    skill_id: 2,
                    icon_number: 4,
                    name: "RALLYING CRY".to_owned(),
                    type_label: "HEALTH - GROUP".to_owned(),
                    required_item_id: 0,
                    required_item_count: 0,
                    description: "Buttercup’s warcry pumps up your group, healing their injuries.".to_owned(),
                },
                TutorialNanoJournalSkillUi {
                    tune_id: 3,
                    skill_id: 3,
                    icon_number: 26,
                    name: "BUTTERCUP BURST".to_owned(),
                    type_label: "SCAVENGE".to_owned(),
                    required_item_id: 0,
                    required_item_count: 0,
                    description: "Buttercup intimidates friend and foe alike to get you even more Fusion Matter!".to_owned(),
                },
            ],
        }
    );
    assert_eq!(
        content
            .journal_entry(2250, "Pokey Oaks North")
            .unwrap()
            .nano
            .as_ref(),
        Some(buttercup)
    );

    let transmitter_reward = content.reward(531).unwrap();
    assert_eq!(transmitter_reward.cash, 0);
    assert_eq!(transmitter_reward.fusion_matter, 75);
    assert_eq!(transmitter_reward.box1_choice, 0);
    assert_eq!(transmitter_reward.box2_choice, 0);
    for task_id in [2248, 2249] {
        assert_eq!(
            content
                .mission_entry(task_id, 2, "Tech Square")
                .unwrap()
                .rewards,
            MissionUiRewards {
                cash: 0,
                fusion_matter: 75,
            }
        );
    }

    assert_eq!(content.npc_name(2671).unwrap(), "Numbuh Two");
    assert_eq!(content.npc_name(2672).unwrap(), "Buttercup");
    assert_eq!(content.npc_name(2673).unwrap(), "Dexter");
    assert_eq!(content.npc_name(2674).unwrap(), "Fusion Spawn");
    assert_eq!(content.npc_name(2675).unwrap(), "Cyberus");
    assert_eq!(content.npc_name(2676).unwrap(), "Oil Ogre");
    assert_eq!(content.npc_name(2677).unwrap(), "Tech Wing");
    assert_eq!(content.npc_name(2678).unwrap(), "Fusion Buttercup");
    assert_eq!(content.npc_name(2696).unwrap(), "Tech Square");
    assert_eq!(content.npc_name(2800).unwrap(), "Scamper");

    let tutorial_scene = content.scene_event(2).unwrap();
    assert_eq!(tutorial_scene.provenance.declared_element_count, 91);
    assert_eq!(tutorial_scene.texts().len(), 91);
    for line in 1..=91 {
        let definition = tutorial_scene.text(line).unwrap();
        assert_eq!(definition.provenance.line, line);
        assert!(!definition.text.is_empty());
    }
    assert_eq!(
        content.scene_text(2, 1).unwrap(),
        "COMPUTRESS: Error. You have been sent much farther into the future than Dexter intended."
    );
    assert_eq!(
        content.scene_text(2, 91).unwrap(),
        "COMPUTRESS: Now that your enemy is stunned, you can get to the SCAMPER safely."
    );

    assert_eq!(content.warp(2694).unwrap().provenance.warp_id, 253);
    assert_eq!(content.warp(2695).unwrap().required_task_id, Some(2252));
    assert_eq!(
        content.warp(2696).unwrap().target,
        TutorialWarpTarget {
            map_id: 0,
            x: 90_765,
            y: 71_558,
            z: 1_043
        }
    );
}

#[test]
fn vendor_table_discovery_preserves_source_order_duplicate_sorts_and_service_text() {
    let mut document = compact_document();
    let value = document["tables"][0]["value"].as_object_mut().unwrap();
    value["m_pNpcTable"]["m_pNpcServiceData"] = json!([
        { "iMember": 0, "m_strService": "" },
        { "iMember": 1, "m_strService": "Exact fixture shop service." }
    ]);
    let officer = value["m_pNpcTable"]["m_pNpcData"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == 664)
        .unwrap();
    officer["m_iServiceNumber"] = json!(1);
    value.insert(
        "m_pVendorTable".to_owned(),
        json!({
            "m_pItemData": [
                {
                    "m_iNpcNumber": 664,
                    "m_iSortNumber": 8,
                    "m_iItemType": 4,
                    "m_iitemID": 104,
                    "m_iSellCost": 0
                },
                {
                    "m_iNpcNumber": 664,
                    "m_iSortNumber": 8,
                    "m_iItemType": 4,
                    "m_iitemID": 100,
                    "m_iSellCost": 25
                },
                {
                    "m_iNpcNumber": 2671,
                    "m_iSortNumber": 1,
                    "m_iItemType": 7,
                    "m_iitemID": 42,
                    "m_iSellCost": 0
                }
            ]
        }),
    );

    let fixture = Fixture::new(document);
    let content = TutorialMissionContent::from_project_assets(&fixture.assets).unwrap();
    assert!(content.gameplay_npc_is_vendor(664));
    assert!(!content.gameplay_npc_is_vendor(2672));
    assert_eq!(
        content.gameplay_npc_service(664),
        Some("Exact fixture shop service.")
    );
    assert_eq!(
        content.gameplay_vendor_items(664),
        Some(
            [
                GameplayVendorItemDefinition {
                    row_index: 0,
                    npc_number: 664,
                    sort_number: 8,
                    item_type: 4,
                    item_id: 104,
                    sell_cost: 0,
                },
                GameplayVendorItemDefinition {
                    row_index: 1,
                    npc_number: 664,
                    sort_number: 8,
                    item_type: 4,
                    item_id: 100,
                    sell_cost: 25,
                },
            ]
            .as_slice()
        )
    );
}

#[test]
fn early_guide_shops_share_only_their_paired_catalog() {
    let rows = json!([
        {"m_iNpcNumber": 644, "m_iSortNumber": 0, "m_iItemType": 0, "m_iitemID": 419, "m_iSellCost": 0},
        {"m_iNpcNumber": 646, "m_iSortNumber": 0, "m_iItemType": 0, "m_iitemID": 222, "m_iSellCost": 0},
        {"m_iNpcNumber": 648, "m_iSortNumber": 0, "m_iItemType": 0, "m_iitemID": 346, "m_iSellCost": 0},
        {"m_iNpcNumber": 650, "m_iSortNumber": 0, "m_iItemType": 0, "m_iitemID": 433, "m_iSellCost": 0}
    ]);
    let vendors = extract_gameplay_vendors(rows.as_array().unwrap()).unwrap();
    for (early, later) in [(643, 644), (645, 646), (647, 648), (649, 650)] {
        let early_rows = &vendors[&early];
        let later_rows = &vendors[&later];
        assert_eq!(early_rows.len(), later_rows.len());
        assert_eq!(early_rows[0].npc_number, early);
        assert_eq!(early_rows[0].item_id, later_rows[0].item_id);
    }
    assert!(!vendors.contains_key(&3277));
}

#[test]
fn bundled_guide_and_holosuit_service_metadata_remains_distinct() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    for (early, later, count) in [(643, 644, 36), (645, 646, 36), (647, 648, 36), (649, 650, 37)] {
        assert_eq!(content.gameplay_npc(early).unwrap().service_category, 2);
        assert_eq!(content.gameplay_vendor_items(early).unwrap().len(), count);
        assert_eq!(content.gameplay_vendor_items(later).unwrap().len(), count);
        assert_eq!(content.gameplay_npc_service(early), content.gameplay_npc_service(later));
    }
    for holosuit in 3277..=3281 {
        assert_eq!(content.gameplay_npc(holosuit).unwrap().service_category, 2);
        assert!(!content.gameplay_npc_is_vendor(holosuit));
        assert!(content.gameplay_npc_service(holosuit).unwrap().contains("Holosuits"));
    }
}

#[test]
fn vendor_vehicle_speed_uses_clean_table_26_i32_row_projection_and_fails_closed() {
    let mut document = compact_user_equip_document();
    let rows = document["tables"][0]["value"]["m_pVehicleItemTable"]["m_pItemData"]
        .as_array_mut()
        .unwrap();
    rows[0]["m_iUp_runSpeed"] = json!(0);
    rows[1]["m_iUp_runSpeed"] = json!(1_100);
    rows[0]["m_iEquipType"] = json!(0);
    rows[1]["m_iEquipType"] = json!(2);
    // Clean lookup is the table row index, not this serialized field.
    rows[1]["m_iItemNumber"] = json!(99);

    let fixture = Fixture::new(document.clone());
    let content = TutorialMissionContent::from_project_assets(&fixture.assets).unwrap();
    let item = |item_type, item_id| ItemBase0104 {
        item_type,
        item_id,
        option: 0,
        time_limit: 0,
    };
    assert_eq!(content.vehicle_speed_class(item(10, 0)), Some(0));
    assert_eq!(content.vehicle_speed_class(item(10, 1)), Some(1_100));
    assert_eq!(content.vehicle_speed_class(item(10, 2)), None);
    assert_eq!(content.vehicle_speed_class(item(4, 1)), None);
    assert_eq!(content.gameplay_vehicle_equip_type(0), Some(0));
    assert_eq!(content.gameplay_vehicle_equip_type(1), Some(2));
    assert_eq!(content.gameplay_vehicle_equip_type(2), None);

    document["tables"][0]["value"]["m_pVehicleItemTable"]["m_pItemData"][0]
        .as_object_mut()
        .unwrap()
        .remove("m_iUp_runSpeed");
    let partial = Fixture::new(document.clone());
    assert!(matches!(
        TutorialMissionContent::from_project_assets(&partial.assets),
        Err(TutorialMissionContentError::Invalid(message))
            if message.contains("m_pVehicleItemTable.m_pItemData[0] has no m_iUp_runSpeed")
    ));

    document["tables"][0]["value"]["m_pVehicleItemTable"]["m_pItemData"][0]["m_iUp_runSpeed"] =
        json!(0);
    document["tables"][0]["value"]["m_pVehicleItemTable"]["m_pItemData"][1]["m_iUp_runSpeed"] =
        json!(i64::from(i32::MAX) + 1);
    let outside_i32 = Fixture::new(document);
    assert!(matches!(
        TutorialMissionContent::from_project_assets(&outside_i32.assets),
        Err(TutorialMissionContentError::Invalid(message))
            if message.contains("m_pVehicleItemTable.m_pItemData[1].m_iUp_runSpeed is outside i32")
    ));
}
