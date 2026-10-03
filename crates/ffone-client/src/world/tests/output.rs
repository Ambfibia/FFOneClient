use super::*;

pub(super) fn write_pretty_json(path: &Path, value: &serde_json::Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, &bytes).unwrap();
    bytes
}

pub(super) fn write_runtime_world_fixture() -> TempDir {
    const SCENE: &str = "map/tiles/map_08_06/scene.json";
    const TERRAIN: &str = "map/tiles/map_08_06/terrain/terrain.json";
    const ENVIRONMENT: &str = "map/tiles/map_08_06/terrain/environment/environment.json";

    let temp = TempDir::new().unwrap();
    let mut environment: serde_json::Value =
        serde_json::from_slice(&fs::read(asset_root().join(ENVIRONMENT)).unwrap()).unwrap();
    sanitize_runtime_environment(&mut environment);
    let environment_bytes =
        write_pretty_json(&join_relative(temp.path(), ENVIRONMENT), &environment);
    let environment_hash = blake3::hash(&environment_bytes).to_hex().to_string();

    let mut terrain: serde_json::Value =
        serde_json::from_slice(&fs::read(asset_root().join(TERRAIN)).unwrap()).unwrap();
    sanitize_runtime_terrain(&mut terrain);
    terrain["environment"]["blake3"] = environment_hash.clone().into();
    let terrain_bytes = write_pretty_json(&join_relative(temp.path(), TERRAIN), &terrain);
    let terrain_hash = blake3::hash(&terrain_bytes).to_hex().to_string();

    let mut scene: serde_json::Value =
        serde_json::from_slice(&fs::read(asset_root().join(SCENE)).unwrap()).unwrap();
    scene.as_object_mut().unwrap().remove("provenance");
    let native_terrain = scene
        .get_mut("nativeTerrain")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    native_terrain.remove("sceneInstancePath");
    native_terrain.remove("sceneInstanceBlake3");
    native_terrain.insert("blake3".to_owned(), terrain_hash.clone().into());
    native_terrain["environment"]["blake3"] = environment_hash.clone().into();
    let scene_bytes = write_pretty_json(&join_relative(temp.path(), SCENE), &scene);
    let scene_hash = blake3::hash(&scene_bytes).to_hex().to_string();

    let registry = serde_json::json!({
        "schema": RUNTIME_WORLD_REGISTRY_SCHEMA,
        "entries": [{
            "id": "map_08_06",
            "scope": "worldMap",
            "tile": [8, 6],
            "scene": {"path": SCENE, "blake3": scene_hash},
            "terrain": {"path": TERRAIN, "blake3": terrain_hash},
            "environment": {"path": ENVIRONMENT, "blake3": environment_hash}
        }]
    });
    write_pretty_json(
        &join_relative(temp.path(), RUNTIME_WORLD_REGISTRY_PATH),
        &registry,
    );
    temp
}
