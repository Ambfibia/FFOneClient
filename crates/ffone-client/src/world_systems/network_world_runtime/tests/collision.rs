use super::*;

#[test]
fn production_bubbie_collision_sidecar_binds_exact_installed_glb() {
    let locator = AssetLocator::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let registry: Value = locator.read_character_models().unwrap();
    let bubbie = registry
        .get("models")
        .and_then(Value::as_array)
        .and_then(|models| {
            models.iter().find(|model| {
                model.get("id").and_then(Value::as_str) == Some("npc/npc_bubbie2")
            })
        })
        .expect("Bubbie registry entry");
    let glb = bubbie.get("glb").and_then(Value::as_str).unwrap();
    let collision_path = bubbie
        .get("collision")
        .and_then(Value::as_str)
        .expect("Bubbie collision sidecar route");
    let glb_bytes = locator.read(glb).unwrap();
    let glb_blake3 = blake3::hash(&glb_bytes).to_hex().to_string();
    let contract: NativeCharacterCollisionContract0104 =
        locator.read_json(collision_path).unwrap();
    contract
        .validate(collision_path, glb, &glb_blake3, None)
        .unwrap();
    assert_eq!(contract.colliders.len(), 1);
    assert_eq!(contract.colliders[0].node, "collision");
    assert_eq!(contract.colliders[0].mesh, 0);
    assert_eq!(contract.colliders[0].primitive, 0);
    assert_eq!(contract.colliders[0].expected_vertex_count, 1_019);
    assert_eq!(contract.colliders[0].expected_index_count, 4_020);
}

#[test]
fn production_nano_stations_bind_exact_native_collision() {
    let locator = AssetLocator::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let registry: Value = locator.read_character_models().unwrap();
    let nanomachine = registry
        .get("models")
        .and_then(Value::as_array)
        .and_then(|models| {
            models.iter().find(|model| {
                model.get("id").and_then(Value::as_str) == Some("npc/nanomachine")
            })
        })
        .expect("Nanomachine registry entry");
    let glb = nanomachine.get("glb").and_then(Value::as_str).unwrap();
    let collision_path = nanomachine
        .get("collision")
        .and_then(Value::as_str)
        .expect("Nanomachine collision sidecar route");
    let visual_hash = blake3::hash(&locator.read(glb).unwrap())
        .to_hex()
        .to_string();
    let contract: NativeCharacterCollisionContract0104 =
        locator.read_json(collision_path).unwrap();
    let collider_glb = contract.collider_glb.as_deref().unwrap();
    let collider_bytes = locator.read(collider_glb).unwrap();
    let collider_hash = blake3::hash(&collider_bytes).to_hex().to_string();
    contract
        .validate(collision_path, glb, &visual_hash, Some(&collider_hash))
        .unwrap();

    assert_eq!(contract.schema, CHARACTER_COLLISION_SCHEMA_V2);
    assert!(contract.source.is_null());
    assert!(contract.conversion.is_null());
    assert_eq!(contract.colliders.len(), 1);
    assert_eq!(contract.colliders[0].node, "collision");
    assert_eq!(contract.colliders[0].expected_vertex_count, 332);
    assert_eq!(contract.colliders[0].expected_index_count, 1_182);

    let visual_gltf = gltf::Gltf::from_slice(&locator.read(glb).unwrap()).unwrap();
    let collision_nodes = visual_gltf
        .nodes()
        .filter(|node| node.name() == Some("collision"))
        .collect::<Vec<_>>();
    assert_eq!(collision_nodes.len(), 1);
    assert!(collision_nodes[0].mesh().is_none());

    let collider_gltf = gltf::Gltf::from_slice(&collider_bytes).unwrap();
    let primitive = collider_gltf
        .meshes()
        .next()
        .and_then(|mesh| mesh.primitives().next())
        .expect("Nanomachine collider primitive");
    assert_eq!(
        primitive.get(&gltf::Semantic::Positions).unwrap().count(),
        332
    );
    assert_eq!(primitive.indices().unwrap().count(), 1_182);

    let catalog = NetworkNpcVisualCatalog0104::open(&locator).unwrap();
    for npc_type in [639, 640] {
        let station = catalog.get(npc_type).expect("Nano Station visual route");
        assert_eq!(station.glb, glb);
        assert!(station.collision_contract.is_some());
    }
}
