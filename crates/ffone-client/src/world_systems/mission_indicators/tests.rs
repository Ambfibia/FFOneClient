use std::path::Path;

use serde_json::Value;

use crate::assets::TABLE_SET_PATH;
use crate::world_mission_indicators::*;

fn production_assets() -> AssetLocator {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    AssetLocator::open(root).expect("production asset root")
}

fn catalog_with_rows(rows: Vec<ClientNpcWaypointRow>) -> ClientNpcWaypointCatalog {
    ClientNpcWaypointCatalog {
        provenance: None,
        rows: rows.into_boxed_slice(),
    }
}

fn f32_array(bits: [u32; 3]) -> [f32; 3] {
    bits.map(f32::from_bits)
}

#[test]
fn game_icons_follow_all_clean_class_and_state_branches() {
    assert_eq!(
        world_npc_game_icon_effect(651, 2, 672, false, false),
        Some(672)
    );
    assert_eq!(
        world_npc_game_icon_effect(1906, 12, 679, false, false),
        Some(679)
    );
    assert_eq!(world_npc_game_icon_effect(3439, 2, 0, false, false), None);
    assert_eq!(world_npc_game_icon_effect(1, 2, 30, false, false), None);
    assert_eq!(world_npc_game_icon_effect(685, 3, 106, false, false), None);
    assert_eq!(world_npc_game_icon_effect(1, 0, 106, false, false), None);
    assert_eq!(world_npc_game_icon_effect(1, 6, 106, false, false), None);

    assert_eq!(
        world_npc_game_icon_effect(1, 13, 685, false, false),
        Some(685)
    );
    assert_eq!(world_npc_game_icon_effect(1, 13, 685, true, false), None);
    assert_eq!(world_npc_game_icon_effect(1, 14, 685, false, false), None);
    assert_eq!(
        world_npc_game_icon_effect(1, 14, 685, true, false),
        Some(685)
    );

    assert_eq!(
        world_npc_game_icon_effect(1386, 17, 446, false, false),
        Some(446)
    );
    assert_eq!(
        world_npc_game_icon_effect(1386, 17, 446, false, true),
        Some(811)
    );
}

#[test]
fn static_props_do_not_receive_scamper_operator_icons() {
    assert_eq!(
        world_npc_game_icon_effect(686, 111, 0, false, false),
        Some(679)
    );
    for npc_type in [1186, 1677, 2215] {
        assert_eq!(
            world_npc_game_icon_effect(npc_type, 100, 0, false, false),
            None
        );
    }
    assert_eq!(world_npc_game_icon_effect(1589, 111, 683, false, false), None);
    assert_eq!(world_npc_game_icon_effect(798, 100, 0, false, false), None);
}

#[test]
fn reported_hnpc_transport_and_holosuit_rows_keep_their_table_icons() {
    // Numbuh 1014: HNPC transport operator, TableData ES683.
    assert_eq!(
        world_npc_game_icon_effect(972, 15, 683, false, false),
        Some(683)
    );
    // Holosuit Harvey: HNPC service character, TableData ES678.
    assert_eq!(
        world_npc_game_icon_effect(3281, 2, 678, false, false),
        Some(678)
    );
}

#[test]
fn production_npc_rows_have_complete_native_game_icon_effect_coverage() {
    let locator = production_assets();
    let table_set: Value = locator.read_table_set().unwrap();
    let table = table_set["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|table| table["name"].as_str() == Some("npc_imports_consolidated"))
        .unwrap();
    let game_icon_effects = table["value"]["m_pNpcTable"]["m_pNpcData"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| {
            let npc_type = row["m_iNpcNumber"].as_i64().unwrap() as i32;
            let npc_class = row["m_iNpcType"].as_i64().unwrap() as i32;
            let effect_id = row["m_iEffect"].as_i64().unwrap() as i32;
            [
                world_npc_game_icon_effect(npc_type, npc_class, effect_id, false, false),
                world_npc_game_icon_effect(npc_type, npc_class, effect_id, true, false),
                world_npc_game_icon_effect(npc_type, npc_class, effect_id, false, true),
                world_npc_game_icon_effect(npc_type, npc_class, effect_id, true, true),
            ]
            .into_iter()
            .flatten()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        game_icon_effects,
        BTreeSet::from([
            66, 395, 446, 672, 673, 674, 675, 676, 677, 678, 679, 680, 681, 682, 683, 685, 811,
            824, 825, 826,
        ])
    );

    let effect_catalog: Value = locator
        .read_json("map/shared/effects/catalog.json")
        .unwrap();
    let published_effects = effect_catalog["effects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["effectId"].as_i64().unwrap() as i32)
        .collect::<BTreeSet<_>>();
    assert!(game_icon_effects.is_subset(&published_effects));
}

fn consolidated_table(table_set: &Value) -> &Value {
    let tables = table_set["tables"].as_array().expect("TableData tables");
    let matches = tables
        .iter()
        .filter(|table| table["name"].as_str() == Some("npc_imports_consolidated"))
        .collect::<Vec<_>>();
    let [table] = matches.as_slice() else {
        panic!("expected exactly one consolidated TableData table");
    };
    &table["value"]
}

#[test]
fn production_catalog_preserves_native_identity_and_order() {
    let catalog = ClientNpcWaypointCatalog::open(&production_assets()).unwrap();

    assert!(catalog.rows().len() >= CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT);
    assert_eq!(catalog.provenance(), None);
    assert_eq!(
        catalog.rows()[0],
        ClientNpcWaypointRow {
            row_index: 0,
            npc_type: 2_379,
            client_position: f32_array([0x452A_8214, 0xC214_70A4, 0x456B_4429]),
        }
    );

    // Duplicate types remain source-ordered; clean returns row 3, not the
    // later type-2671 row 4.
    assert_eq!(catalog.rows()[3].npc_type, 2_671);
    assert_eq!(catalog.rows()[4].npc_type, 2_671);
    assert_eq!(
        catalog.first_matching_type(2_671).map(|row| row.row_index),
        Some(3)
    );
}

#[test]
fn authored_destinations_extend_the_catalog_without_changing_first_match_order() {
    let locator = production_assets();
    let mut document: Value = serde_json::from_slice(&locator.read(CLIENT_NPC_WAYPOINT_CATALOG_PATH).unwrap()).unwrap();
    let count = document["rows"].as_array().unwrap().len();
    document["rows"].as_array_mut().unwrap().push(serde_json::json!({
        "npcType": 900001, "clientPosition": [12.5, 2., 25.]
    }));
    let catalog = ClientNpcWaypointCatalog::from_json_bytes(&serde_json::to_vec(&document).unwrap()).unwrap();
    assert_eq!(catalog.first_matching_type(900001).unwrap().client_position, [12.5, 2., 25.]);
    assert_eq!(catalog.first_matching_type(900001).unwrap().row_index as usize, count);
    assert_eq!(catalog.first_matching_type(2671).unwrap().row_index, 3);
    document["rowCount"] = Value::from(count);
    assert!(ClientNpcWaypointCatalog::from_json_bytes(&serde_json::to_vec(&document).unwrap()).is_err());
}

fn legacy_waypoint_document() -> Value {
    serde_json::json!({
        "schema": LEGACY_CLIENT_NPC_WAYPOINT_CATALOG_SCHEMA,
        "source": expected_provenance(),
        "rowCount": 2,
        "rows": [
            {"rowIndex": 0, "npcType": 7, "clientPosition": [12.123456789, 2., 25.]},
            {"rowIndex": 1, "npcType": 7, "clientPosition": [50., 0., 60.]}
        ]
    })
}

#[test]
fn legacy_catalog_migrates_losslessly_and_keeps_first_matching_destination() {
    let legacy = legacy_waypoint_document();
    let before = ClientNpcWaypointCatalog::from_json_bytes(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(before.provenance(), Some(&expected_provenance()));
    let native = ClientNpcWaypointCatalog::into_native_document(legacy.clone()).unwrap();
    assert_eq!(native["schema"], CLIENT_NPC_WAYPOINT_CATALOG_SCHEMA);
    assert!(native.get("source").is_none());
    assert!(native.get("rowCount").is_none());
    for (old, new) in legacy["rows"].as_array().unwrap().iter().zip(native["rows"].as_array().unwrap()) {
        assert_eq!(old["npcType"], new["npcType"]);
        assert_eq!(old["clientPosition"], new["clientPosition"]);
        assert!(new.get("rowIndex").is_none());
    }
    let after = ClientNpcWaypointCatalog::from_json_bytes(&serde_json::to_vec(&native).unwrap()).unwrap();
    assert_eq!(before.rows(), after.rows());
    assert_eq!(resolve_client_npc_waypoint_update(Some(7), &before), resolve_client_npc_waypoint_update(Some(7), &after));
    assert_eq!(after.provenance(), None);
    assert_eq!(ClientNpcWaypointCatalog::into_native_document(native.clone()).unwrap(), native);
}

#[test]
fn legacy_catalog_still_rejects_bad_provenance_counts_and_indices_before_migration() {
    let original = legacy_waypoint_document();
    for (pointer, value) in [
        ("/source/build", Value::from("other-build")),
        ("/rowCount", Value::from(1)),
        ("/rows/1/rowIndex", Value::from(0)),
    ] {
        let mut invalid = original.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(ClientNpcWaypointCatalog::into_native_document(invalid).is_err(), "{pointer}");
    }
    for field in ["source", "rowCount"] {
        let mut invalid = original.clone();
        invalid.as_object_mut().unwrap().remove(field);
        assert!(ClientNpcWaypointCatalog::into_native_document(invalid).is_err(), "{field}");
    }
    let mut invalid = original;
    invalid["rows"][0].as_object_mut().unwrap().remove("rowIndex");
    assert!(ClientNpcWaypointCatalog::into_native_document(invalid).is_err());
}

#[test]
fn native_catalog_rejects_unknown_fields_schemas_and_non_finite_runtime_coordinates() {
    let native = ClientNpcWaypointCatalog::into_native_document(legacy_waypoint_document()).unwrap();
    for (pointer, value) in [
        ("/schema", Value::from("ffone.client-npc-waypoint-catalog.v99")),
        ("/rows/0/clientPosition/0", serde_json::json!(1e100)),
        ("/rows/0/npcType", serde_json::json!(2147483648_u64)),
    ] {
        let mut invalid = native.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(ClientNpcWaypointCatalog::into_native_document(invalid).is_err(), "{pointer}");
    }
    let mut invalid = native.clone();
    invalid["rows"][0]["rowIndex"] = Value::from(0);
    assert!(ClientNpcWaypointCatalog::into_native_document(invalid).is_err());
    let mut invalid = native;
    invalid["source"] = serde_json::json!(expected_provenance());
    assert!(ClientNpcWaypointCatalog::into_native_document(invalid).is_err());
}

#[test]
fn primary_mission_row_1919_resolves_clientnpc_row_473_not_a_live_npc() {
    let assets = production_assets();
    let table_set: Value = assets.read_table_set().unwrap();
    let mission = &consolidated_table(&table_set)["m_pMissionTable"]["m_pMissionData"][1_919];
    assert_eq!(mission["m_iHTaskID"].as_i64(), Some(657));
    assert_eq!(mission["m_iHMissionID"].as_i64(), Some(533));
    assert_eq!(mission["m_iHNPCID"].as_i64(), Some(768));
    assert_eq!(mission["m_iHTerminatorNPCID"].as_i64(), Some(967));
    assert_eq!(mission["m_iSTGrantWayPoint"].as_i64(), Some(1_111));

    let projection = project_selected_mission_waypoints(
        Some(533),
        &[ActiveMissionWaypointTask {
            task_id: 657,
            mission_id: 533,
            grant_waypoint_npc_type: 1_111,
        }],
    );
    assert_eq!(projection.current_task_id, Some(657));
    assert_eq!(projection.current_grant_waypoint_npc_type, Some(1_111));
    assert_eq!(
        projection.smart_indicator_npc_types,
        BTreeSet::from([1_111])
    );

    let catalog = ClientNpcWaypointCatalog::open(&assets).unwrap();
    let source = catalog.first_matching_type(1_111).unwrap();
    assert_eq!(source.row_index, 473);
    assert_eq!(
        source.client_position,
        f32_array([0x45C5_8629, 0xC263_51EB, 0x44F0_4DC2])
    );
    assert_eq!(
        resolve_client_npc_waypoint_update(Some(1_111), &catalog),
        ClientNpcWaypointUpdate::Set {
            npc_type: 1_111,
            source_row_index: 473,
            native_position: Vec3::from_array(f32_array([
                0xC5C5_8629,
                0xC263_51EB,
                0x44F0_4DC2,
            ])),
        }
    );
}

#[test]
fn waypoint_update_clears_or_preserves_at_the_clean_edges() {
    let catalog = catalog_with_rows(vec![
        ClientNpcWaypointRow {
            row_index: 0,
            npc_type: 7,
            client_position: [10.0, 20.0, 30.0],
        },
        ClientNpcWaypointRow {
            row_index: 1,
            npc_type: 8,
            client_position: [0.0; 3],
        },
    ]);

    assert_eq!(
        resolve_client_npc_waypoint_update(None, &catalog),
        ClientNpcWaypointUpdate::Clear
    );
    assert_eq!(
        resolve_client_npc_waypoint_update(Some(0), &catalog),
        ClientNpcWaypointUpdate::Clear
    );
    assert_eq!(
        resolve_client_npc_waypoint_update(Some(9), &catalog),
        ClientNpcWaypointUpdate::PreservePrevious {
            npc_type: 9,
            reason: ClientNpcWaypointPreserveReason::MissingNpcType,
        }
    );
    assert_eq!(
        resolve_client_npc_waypoint_update(Some(8), &catalog),
        ClientNpcWaypointUpdate::PreservePrevious {
            npc_type: 8,
            reason: ClientNpcWaypointPreserveReason::ZeroSourcePosition {
                source_row_index: 1,
            },
        }
    );
    assert_eq!(
        resolve_client_npc_waypoint_update(Some(7), &catalog),
        ClientNpcWaypointUpdate::Set {
            npc_type: 7,
            source_row_index: 0,
            native_position: Vec3::new(-10.0, 20.0, 30.0),
        }
    );
}

#[test]
fn first_selected_task_drives_waypoint_while_all_selected_tasks_drive_smart() {
    let tasks = [
        ActiveMissionWaypointTask {
            task_id: 10,
            mission_id: 5,
            grant_waypoint_npc_type: 0,
        },
        ActiveMissionWaypointTask {
            task_id: 11,
            mission_id: 5,
            grant_waypoint_npc_type: 101,
        },
        ActiveMissionWaypointTask {
            task_id: 12,
            mission_id: 6,
            grant_waypoint_npc_type: 202,
        },
        ActiveMissionWaypointTask {
            task_id: 13,
            mission_id: 5,
            grant_waypoint_npc_type: 303,
        },
    ];

    assert_eq!(
        project_selected_mission_waypoints(Some(5), &tasks),
        SelectedMissionWaypointProjection {
            current_task_id: Some(10),
            current_grant_waypoint_npc_type: Some(0),
            smart_indicator_npc_types: BTreeSet::from([101, 303]),
        }
    );
    assert_eq!(
        project_selected_mission_waypoints(None, &tasks),
        SelectedMissionWaypointProjection::default()
    );
}

fn indicator_input(npc_class: i32) -> WorldMissionIndicatorEligibilityInput {
    WorldMissionIndicatorEligibilityInput {
        in_refresh_near_list: true,
        npc_class,
        selected_waypoint_target: true,
        smart_force_deleted: false,
        radius_server_units: 90,
        has_active_terminating_task: true,
        has_new_mission_available: true,
    }
}

#[test]
fn advance_wins_over_new_and_mission_finder_is_not_a_three_dimensional_input() {
    let WorldMissionIndicatorRefresh::Reconcile(desired) =
        project_world_mission_indicator_refresh(indicator_input(3))
    else {
        panic!("ordinary NPC must be reconciled");
    };
    assert_eq!(
        desired.smart,
        Some(DesiredWorldSmartIndicator {
            effect_id: 668,
            scale: 1.8,
        })
    );
    assert_eq!(
        desired.quest_symbol,
        Some(WorldMissionIndicatorSymbol::Advance)
    );
    assert_eq!(
        desired
            .quest_symbol
            .map(WorldMissionIndicatorSymbol::effect_id),
        Some(865)
    );

    let mut new_only = indicator_input(3);
    new_only.has_active_terminating_task = false;
    assert_eq!(
        project_world_mission_indicator_refresh(new_only),
        WorldMissionIndicatorRefresh::Reconcile(DesiredWorldMissionIndicators {
            smart: Some(DesiredWorldSmartIndicator {
                effect_id: 668,
                scale: 1.8,
            }),
            quest_symbol: Some(WorldMissionIndicatorSymbol::New),
        })
    );
}

#[test]
fn smart_and_quest_class_gates_remain_distinct() {
    for npc_class in [0, 25] {
        assert_eq!(
            project_world_mission_indicator_refresh(indicator_input(npc_class)),
            WorldMissionIndicatorRefresh::PreservePrevious
        );
    }

    // Row 1919's waypoint target type 1111 is class 111. Clean may show
    // ES668 on it, but the New/Advance predicates reject that class.
    for npc_class in [100, 101, 105, 110, 111] {
        assert_eq!(
            project_world_mission_indicator_refresh(indicator_input(npc_class)),
            WorldMissionIndicatorRefresh::Reconcile(DesiredWorldMissionIndicators {
                smart: Some(DesiredWorldSmartIndicator {
                    effect_id: 668,
                    scale: 1.8,
                }),
                quest_symbol: None,
            })
        );
    }
}

#[test]
fn near_list_and_interaction_preserve_clean_lifecycle_boundaries() {
    let mut outside = indicator_input(3);
    outside.in_refresh_near_list = false;
    assert_eq!(
        project_world_mission_indicator_refresh(outside),
        WorldMissionIndicatorRefresh::PreservePrevious
    );

    let mut interacting = indicator_input(3);
    interacting.smart_force_deleted = true;
    assert_eq!(
        project_world_mission_indicator_refresh(interacting),
        WorldMissionIndicatorRefresh::Reconcile(DesiredWorldMissionIndicators {
            smart: None,
            quest_symbol: Some(WorldMissionIndicatorSymbol::Advance),
        })
    );
}

#[test]
fn exact_effect_and_attachment_constants_match_primary() {
    assert_eq!(
        (
            WORLD_SMART_INDICATOR_EFFECT_ID,
            WORLD_ADVANCE_SYMBOL_EFFECT_ID,
            WORLD_NEW_SYMBOL_EFFECT_ID,
        ),
        (668, 865, 866)
    );
    assert_eq!(
        (
            WORLD_SMART_INDICATOR_EFFECT_PRIORITY,
            WORLD_QUEST_SYMBOL_EFFECT_PRIORITY,
        ),
        (1, 0)
    );
    assert_eq!(WORLD_QUEST_SYMBOL_SCALE, 1.0);
    assert_eq!(WORLD_QUEST_SYMBOL_ROOT_HEIGHT_FACTOR, 0.85);
    assert_eq!(world_smart_indicator_scale(90), 1.8);
}
