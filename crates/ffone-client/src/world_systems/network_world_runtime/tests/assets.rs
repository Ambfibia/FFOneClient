use super::*;

pub(super) fn registry(models: Value) -> Value {
    serde_json::json!({
        "schema": CHARACTER_REGISTRY_SCHEMA,
        "models": models
    })
}

#[test]
fn duplicate_fusion_name_prefers_exact_npc_asset() {
    let table = table_set(
        serde_json::json!([{
            "m_iNpcNumber": 698,
            "m_iMesh": 0,
            "m_iHeight": 200,
            "m_fScale": 1.0,
            "m_fAnimationSpeed": 1.0,
            "m_fWalkAnimationSpeed": 1.0,
            "m_fRunAnimationSpeed": 1.0
        }]),
        serde_json::json!([{"m_pstrMMeshModelString": "npc_ampfibian"}]),
    );
    let registry = registry(serde_json::json!([
        {
            "logicalName": "npc_ampfibian",
            "category": "fusion",
            "glb": "characters/fusions/fusion_ampfibian/npc_ampfibian.glb"
        },
        {
            "logicalName": "npc_ampfibian",
            "category": "npc",
            "glb": "characters/npcs/npc_ampfibian/npc_ampfibian.glb"
        }
    ]));
    let catalog = NetworkNpcVisualCatalog0104::from_documents(
        &table,
        &registry,
        &texture_catalog(serde_json::json!([]), serde_json::json!([])),
        |_| Ok(()),
        |_| Ok(()),
    )
    .unwrap();

    assert_eq!(
        catalog.get(698).unwrap().glb,
        "characters/npcs/npc_ampfibian/npc_ampfibian.glb"
    );
}

#[test]
fn every_installed_character_is_reachable_by_its_package_route() {
    // `parse_character_registry` probes `by_logical_name` with the exact XDT
    // string. When a package directory and the model's Unity root name
    // differ, only a `legacyAliases` entry can bridge them, so an installed
    // model with neither is unreachable and its NPCs render nothing.
    // Nineteen entries were in that state until the generated registry was
    // repaired; this keeps the invariant.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root).unwrap();
    let registry: Value = locator.read_character_models().unwrap();
    let models = registry["models"].as_array().expect("registry models");

    let mut unreachable = Vec::new();
    for model in models {
        let logical_name = model["logicalName"].as_str().expect("logicalName");
        let glb = model["glb"].as_str().expect("glb");
        let package_route = glb
            .rsplit('/')
            .nth(1)
            .expect("registry GLB has a package directory");
        if package_route.eq_ignore_ascii_case(logical_name) {
            continue;
        }
        let aliased = model["legacyAliases"].as_array().is_some_and(|aliases| {
            aliases.iter().any(|alias| {
                alias
                    .as_str()
                    .is_some_and(|alias| alias.eq_ignore_ascii_case(package_route))
            })
        });
        if !aliased {
            unreachable.push(format!(
                "{} (package {package_route}, root {logical_name})",
                model["id"].as_str().unwrap_or("<no id>")
            ));
        }
    }
    assert!(
        unreachable.is_empty(),
        "installed models unreachable by their package route: {unreachable:#?}\n\
         repair with: ffone-asset-pipeline repair-character-registry-aliases <asset root> <report> --apply"
    );
}

#[test]
fn production_catalog_resolves_known_primary_npc_routes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root).unwrap();
    let catalog = NetworkNpcVisualCatalog0104::open(&locator).unwrap();

    assert!(
        catalog.len() >= 1_700,
        "unexpectedly small production NPC catalog: {} definitions, {} issues",
        catalog.len(),
        catalog.issues.len()
    );
    assert_eq!(catalog.hnpc_len(), 457);
    let chef = catalog.get(3206).expect("Mandark ship Croc Pot Mandroid");
    assert_eq!(chef.logical_name, "npc_mandroid_chef");
    assert_eq!(
        chef.glb,
        "characters/shared/npc_mandroid1/npc_mandroid_chef.glb"
    );
    assert_eq!(
        catalog.get(2498).unwrap().glb,
        "characters/shared/npc_mandroid1/npc_mandroid1.glb"
    );
    assert_eq!(catalog.get_hnpc(2586).unwrap().appearance_index, 174);
    assert_eq!(catalog.get_hnpc(2587).unwrap().appearance_index, 178);
    let father = catalog.get_hnpc(159).expect("Father HNPC WearItems route");
    assert_eq!(father.appearance_index, 148);
    assert_eq!(father.height_server_units, 170);
    let service_hnpc = catalog
        .get_hnpc(790)
        .expect("attachment-only HNPC appearance route");
    assert_eq!(service_hnpc.appearance_index, 149);
    let table_set: Value = locator.read_table_set().unwrap();
    let npc_table = table_set
        .get("tables")
        .and_then(Value::as_array)
        .and_then(|tables| {
            tables.iter().find(|table| {
                table.get("name").and_then(Value::as_str) == Some(CONSOLIDATED_TABLE)
            })
        })
        .and_then(|table| table.pointer("/value/m_pNpcTable"))
        .expect("production consolidated NPC table");
    let npc_rows = npc_table["m_pNpcData"].as_array().unwrap();
    let mesh_rows = npc_table["m_pNpcMeshData"].as_array().unwrap();
    let mut repaired_case_routes = 0;
    for row in npc_rows {
        let mesh = &mesh_rows[row["m_iMesh"].as_u64().unwrap() as usize];
        if matches!(
            mesh["m_pstrMMeshModelString"].as_str(),
            Some("mob_queenSpider" | "fusion_spidermonkey")
        ) {
            let npc_type = row["m_iNpcNumber"].as_i64().unwrap() as i32;
            assert!(
                catalog.get(npc_type).is_some(),
                "missing exact-case route {npc_type}"
            );
            repaired_case_routes += 1;
        }
    }
    assert_eq!(repaired_case_routes, 10);
    for (npc_type, original, route) in [
        (3464, 732, "omniverse_ben"),
        (3465, 738, "omniverse_gwen"),
        (3466, 749, "omniverse_kevin"),
        (3467, 697, "omniverse_albedo"),
        (3468, 725, "omniverse_max"),
    ] {
        let variant = catalog.get(npc_type).expect("separate Omniverse NPC");
        let existing = catalog.get(original).expect("preserved original NPC");
        assert_ne!(variant.glb, existing.glb);
        assert!(
            variant
                .glb
                .starts_with(&format!("characters/npcs/{route}/"))
        );
        assert_eq!(
            variant.main_texture.as_ref().unwrap().path,
            format!("characters/npcs/{route}/body.png")
        );
        assert_eq!(npc_rows[npc_type as usize]["m_iNpcType"], 3);
        assert_eq!(npc_rows[npc_type as usize]["m_iServiceNumber"], 0);
    }
    let hidden_rows = npc_rows
        .iter()
        .filter(|row| {
            row["m_iNpcNumber"].as_i64().unwrap_or_default() > 0
                && row["m_iHNpc"].as_i64().unwrap_or_default() == 0
                && row["m_iNpcType"].as_i64().unwrap_or_default() >= 100
        })
        .collect::<Vec<_>>();
    let hidden_objectnpc1_rows = hidden_rows
        .iter()
        .filter(|row| {
            let mesh_index = usize::try_from(row["m_iMesh"].as_i64().unwrap()).unwrap();
            mesh_rows[mesh_index]["m_pstrMMeshModelString"]
                .as_str()
                .is_some_and(|name| name.eq_ignore_ascii_case("ObjectNPC1"))
        })
        .count();
    assert_eq!(hidden_rows.len(), 1_158);
    assert_eq!(hidden_objectnpc1_rows, 1_153);
    assert!(catalog.get(798).is_none(), "hidden class-100 cube leaked");
    let visible_objectnpc1 = catalog
        .get(1175)
        .expect("primary class-23 interaction placeholder");
    assert_eq!(visible_objectnpc1.logical_name, "ObjectNPC1");
    assert_eq!(visible_objectnpc1.table_scale, 0.01);
    let dexter = catalog.get(728).expect("primary Dexter NPC route");
    assert_eq!(dexter.logical_name, "npc_dexter2");
    assert_eq!(dexter.glb, "characters/npcs/npc_dexter2/npc_dexter2.glb");
    assert_eq!(dexter.table_scale, 1.0);
    assert_eq!(dexter.height_server_units, 200);

    // These placeholder identities also display Dexter and must use the
    // same geometry/texture pairing as his updated ordinary NPC route.
    for npc_type in [753, 754] {
        let variant = catalog.get(npc_type).expect("Dexter placeholder route");
        assert_eq!(variant.logical_name, dexter.logical_name);
        assert_eq!(variant.glb, dexter.glb);
        assert_eq!(variant.main_texture, dexter.main_texture);
        assert_eq!(variant.sub_texture, dexter.sub_texture);
    }
    let deedee = catalog.get(701).expect("ordinary Dee Dee route");
    assert_eq!(deedee.logical_name, "npc_deedee");
    assert_eq!(deedee.glb, "characters/npcs/npc_deedee/npc_deedee.glb");
    assert_eq!(
        deedee.main_texture.as_ref().unwrap().path,
        "characters/npcs/npc_deedee/npc_deedee.textures/npc_deedee.png"
    );

    let bubbie = catalog.get(684).expect("primary Bubbie NPC route");
    assert_eq!(bubbie.logical_name, "npc_gubbie");
    assert_eq!(
        bubbie.collision_path.as_deref(),
        Some("characters/npcs/npc_bubbie2/npc_gubbie.collision.json")
    );
    let collision = bubbie
        .collision_contract
        .as_ref()
        .expect("Bubbie exact MeshCollider sidecar");
    assert_eq!(collision.colliders.len(), 1);
    assert_eq!(collision.colliders[0].node, "collision");
    assert_eq!(collision.colliders[0].mesh, 0);
    assert_eq!(collision.colliders[0].expected_vertex_count, 1_019);
    assert_eq!(collision.colliders[0].expected_index_count, 4_020);

    let sneaky_spawn = catalog.get(82).expect("Sneaky Spawn runtime route");
    assert_eq!(sneaky_spawn.logical_name, "mob_sneakyspawn");
    assert_eq!(
        sneaky_spawn.main_texture.as_ref().unwrap().path,
        "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.textures/mob_sneakyspawn.png"
    );

    let spawn_crew = catalog.get(96).expect("Spawn Crew runtime route");
    assert_eq!(spawn_crew.logical_name, "mob_spawncrew");
    assert_eq!(
        spawn_crew.main_texture.as_ref().unwrap().path,
        "characters/mobs/mob_spawncrew/mob_spawncrew.textures/mob_piratespawn.png"
    );

    let spawn_captain = catalog.get(484).expect("Spawn Captain runtime route");
    assert_eq!(spawn_captain.logical_name, "mob_spawncaptain");
    assert_eq!(
        spawn_captain.main_texture.as_ref().unwrap().path,
        "characters/mobs/mob_spawncaptain/mob_spawncaptain.textures/mob_piratespawn_boss.png"
    );

    let hostile_kevin = catalog
        .get(2226)
        .expect("hostile Kevin Levin runtime route");
    assert_eq!(hostile_kevin.logical_name, "npc_kevin");
    assert_eq!(
        hostile_kevin.main_texture.as_ref().unwrap().path,
        "characters/shared/npc_kevin/npc_kevin.textures/npc_kevin.png"
    );
    let kevin = catalog
        .get(3380)
        .expect("previous-version Kevin runtime route");
    assert_eq!(kevin.logical_name, "npc_kevin");
    assert_eq!(kevin.glb, hostile_kevin.glb);

    let weeper = catalog.get(3369).expect("The Weeper runtime route");
    assert_eq!(
        weeper.main_texture.as_ref().unwrap().path,
        "characters/mobs/mob_weeper/mob_weeper.textures/mob_weeper.png"
    );
    let sheet_music_shogun = catalog.get(3334).expect("Sheet Music Shogun runtime route");
    assert_eq!(
        sheet_music_shogun.main_texture.as_ref().unwrap().path,
        "characters/shared/runtime-textures/sheetmusicninjaboss.png"
    );
    let texture_document = locator.read_json(NPC_TEXTURE_CATALOG_PATH).unwrap();
    let overcharged_megawatt_catalog =
        parse_npc_texture_catalog(&texture_document, &mut |_| Ok(())).unwrap();
    let overcharged_megawatt = overcharged_megawatt_catalog
        .by_true_name
        .get("fusion_megawatt")
        .expect("Overcharged Megawatt texture route");
    assert_eq!(
        overcharged_megawatt.path,
        "characters/fusions/fusion_megawatt/npc_megawhatt.textures/fusion_megawatt.png"
    );

    let kevin_car = catalog.get(689).expect("Kevin car legacy-alias route");
    assert_eq!(kevin_car.logical_name, "EX_EXMN_Kevin_Car");
    assert_eq!(
        kevin_car.main_texture.as_ref().unwrap().path,
        "characters/shared/runtime-textures/ex_exmn_kevin_car_01_dds.png"
    );
    let penguin_sign = catalog.get(3292).expect("Penguin sign legacy-alias route");
    assert_eq!(penguin_sign.logical_name, "npc_guntersign");
    assert_eq!(
        penguin_sign.sub_texture.as_ref().unwrap().path,
        "characters/shared/runtime-textures/npc_penguinright.png"
    );
    // V.V. Argost is intentional patched-XDT content, so its NPC number is
    // not asserted through this clean-primary table-set.  The production
    // registry must still retain the patched spelling as a legacy alias.
    let registry = locator.read_character_models().unwrap();
    let models = parse_character_registry(&registry).unwrap();
    let argost = models
        .get("npc_argost")
        .and_then(|candidates| select_registry_model(candidates))
        .expect("V.V. Argost patched-XDT alias route");
    assert_eq!(argost.logical_name, "npc_arghost");
    assert_eq!(argost.glb, "characters/npcs/npc_arghost/npc_arghost.glb");
}
