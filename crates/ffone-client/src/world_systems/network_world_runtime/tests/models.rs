use super::*;

#[test]
fn resolves_exact_xdt_mesh_scale_and_registry_glb() {
    let table = table_set(
        serde_json::json!([
            {
                "m_iNpcNumber": 0,
                "m_iMesh": 0,
                "m_fScale": 0.0,
                "m_fAnimationSpeed": 0.0,
                "m_fWalkAnimationSpeed": 0.0,
                "m_fRunAnimationSpeed": 0.0
            },
            {
                "m_iNpcNumber": 728,
                "m_iMesh": 1,
                "m_iHeight": 200,
                "m_fScale": 1.25,
                "m_fAnimationSpeed": 1.0,
                "m_fWalkAnimationSpeed": 0.8,
                "m_fRunAnimationSpeed": 1.2
            }
        ]),
        serde_json::json!([
            {"m_pstrMMeshModelString": ""},
            {"m_pstrMMeshModelString": "npc_dexter"}
        ]),
    );
    let registry = registry(serde_json::json!([{
        "logicalName": "npc_dexter",
        "category": "npc",
        "glb": "characters/npcs/npc_dexter/npc_dexter.glb"
    }]));
    let mut required = Vec::new();
    let catalog = NetworkNpcVisualCatalog0104::from_documents(
        &table,
        &registry,
        &texture_catalog(serde_json::json!([]), serde_json::json!([])),
        |path| {
            required.push(path.to_owned());
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();

    assert_eq!(required, ["characters/npcs/npc_dexter/npc_dexter.glb"]);
    assert_eq!(catalog.issues, []);
    assert_eq!(
        catalog.get(728),
        Some(&NetworkNpcVisualDefinition0104 {
            npc_type: 728,
            logical_name: "npc_dexter".to_owned(),
            category: "npc".to_owned(),
            glb: "characters/npcs/npc_dexter/npc_dexter.glb".to_owned(),
            table_scale: 1.25,
            height_server_units: 200,
            main_texture: None,
            sub_texture: None,
            walk_animation_speed: 0.8,
            run_animation_speed: 1.2,
            collision_path: None,
            collision_contract: None,
            animation_effect_events: Arc::from([]),
            animation_sound_events: Arc::from([]),
            animation_ends: Arc::default(),
            material_animation_clips: Arc::from([]),
        })
    );
}

#[test]
fn legacy_registry_alias_resolves_to_the_true_glb_root() {
    let table = table_set(
        serde_json::json!([{
            "m_iNpcNumber": 689,
            "m_iMesh": 0,
            "m_iHeight": 200,
            "m_fScale": 0.7,
            "m_fWalkAnimationSpeed": 1.0,
            "m_fRunAnimationSpeed": 1.0
        }]),
        serde_json::json!([{"m_pstrMMeshModelString": "npc_kevincar"}]),
    );
    let registry = registry(serde_json::json!([{
        "logicalName": "EX_EXMN_Kevin_Car",
        "legacyAliases": ["npc_kevincar"],
        "category": "npc",
        "glb": "characters/npcs/npc_kevincar/EX_EXMN_Kevin_Car.glb"
    }]));
    let catalog = NetworkNpcVisualCatalog0104::from_documents(
        &table,
        &registry,
        &texture_catalog(serde_json::json!([]), serde_json::json!([])),
        |_| Ok(()),
        |_| Ok(()),
    )
    .unwrap();

    let definition = catalog.get(689).unwrap();
    assert_eq!(definition.logical_name, "EX_EXMN_Kevin_Car");
    assert_eq!(
        definition.glb,
        "characters/npcs/npc_kevincar/EX_EXMN_Kevin_Car.glb"
    );
}

#[test]
fn unresolved_mesh_is_a_typed_issue_not_placeholder_geometry() {
    let table = table_set(
        serde_json::json!([{
            "m_iNpcNumber": 99,
            "m_iMesh": 0,
            "m_iHeight": 200,
            "m_fScale": 1.0,
            "m_fAnimationSpeed": 1.0,
            "m_fWalkAnimationSpeed": 1.0,
            "m_fRunAnimationSpeed": 1.0
        }]),
        serde_json::json!([{"m_pstrMMeshModelString": "missing_model"}]),
    );
    let catalog = NetworkNpcVisualCatalog0104::from_documents(
        &table,
        &registry(serde_json::json!([])),
        &texture_catalog(serde_json::json!([]), serde_json::json!([])),
        |_| Ok(()),
        |_| Ok(()),
    )
    .unwrap();

    assert!(catalog.is_empty());
    assert_eq!(catalog.issues.len(), 1);
    assert_eq!(catalog.issues[0].npc_type, 99);
}

#[test]
fn npc_end_events_come_from_the_authored_glb_and_skip_zero_times() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let golem =
        std::fs::read(root.join("characters/mobs/mob_roadgorlem/mob_roadgorlem.glb")).unwrap();
    let ends = parse_network_npc_animation_end_events(&golem).unwrap();
    // Idle and wound hand over a quarter second before their last key.
    assert!((ends["stand1"] - 1.35).abs() < 1e-4, "{ends:?}");
    assert!((ends["wound"] - 0.55).abs() < 1e-4, "{ends:?}");
    let alien = std::fs::read(root.join("characters/npcs/npc_alienx/npc_alienx.glb")).unwrap();
    assert!(
        !parse_network_npc_animation_end_events(&alien)
            .unwrap()
            .contains_key("stand1")
    );
}
