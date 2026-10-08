use std::{fs, path::Path};

use ffone_packaging::{
    AssetValidationOptions, PackageOptions, VerifyOptions, package_loose,
    package_loose_from_snapshot, snapshot_release_source, validate_assets, verify_asset_root,
};
use serde_json::json;
use tempfile::tempdir;

fn write(root: &Path, relative: &str, bytes: &[u8]) {
    let path = relative
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn pretty(value: &serde_json::Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn reference(path: &str, bytes: &[u8]) -> serde_json::Value {
    json!({
        "path": path,
        "bytes": bytes.len(),
        "blake3": blake3::hash(bytes).to_hex().to_string()
    })
}

fn fixture(root: &Path) {
    let audio = b"OggS";
    let character = b"glTF-character";
    let map_model = b"glTF-map-object";
    let map_texture = b"PNG-map-atlas";
    let map_object = pretty(&json!({"schema": "fixture.map-object.v1"}));
    let map_model_path = "objects/nature/tree_set/models/tree/model.glb";
    let map_texture_path = "objects/nature/tree_set/textures/tree.png";
    let map_object_path = "objects/nature/tree_set/models/tree/object.json";
    let map_set_path = "objects/nature/tree_set/set.json";
    let map_set = pretty(&json!({
        "schema": "ffone.resource-set.v1",
        "id": "map-set-tree",
        "name": "tree_set",
        "domain": "map_object",
        "category": "nature",
        "prefix": "WD",
        "family": "TREE",
        "textures": [reference(map_texture_path, map_texture)],
        "members": [{
            "id": "map-object-tree",
            "name": "tree",
            "definition": reference(map_object_path, &map_object),
            "files": [reference(map_model_path, map_model)]
        }]
    }));
    let player_model = b"glTF-player-item";
    let player_texture = b"PNG-player-atlas";
    let player_item = pretty(&json!({"schema": "fixture.player-item.v1"}));
    let player_model_path = "characters/player/items/hat/hat_set/models/hat/model.glb";
    let player_texture_path = "characters/player/items/hat/hat_set/textures/hat.png";
    let player_item_path = "characters/player/items/hat/hat_set/models/hat/item.json";
    let player_set_path = "characters/player/items/hat/hat_set/set.json";
    let player_set = pretty(&json!({
        "schema": "ffone.resource-set.v1",
        "id": "player-item-set-hat",
        "name": "hat_set",
        "domain": "player_item",
        "category": "hat",
        "prefix": "PLAYER",
        "family": "HAT",
        "textures": [reference(player_texture_path, player_texture)],
        "members": [{
            "id": "player-item-hat",
            "name": "hat",
            "definition": reference(player_item_path, &player_item),
            "files": [reference(player_model_path, player_model)]
        }]
    }));

    write(root, "audio/sfx/ping.ogg", audio);
    write(root, "characters/mobs/mob_test/mob_test.glb", character);
    write(root, map_model_path, map_model);
    write(root, map_texture_path, map_texture);
    write(root, map_object_path, &map_object);
    write(root, map_set_path, &map_set);
    write(root, player_model_path, player_model);
    write(root, player_texture_path, player_texture);
    write(root, player_item_path, &player_item);
    write(root, player_set_path, &player_set);
    write(root, "data/tables/xdt.json", b"{}\n");
    write(root, "localization/en.json", b"{}\n");
    write(root, "localization/ru.json", b"{}\n");
    write(
        root,
        "localization/catalog.json",
        &pretty(&json!({
            "schema": "ffone.localization-catalog.v1",
            "fallback": "en",
            "locales": [
                {"id": "en", "text": "localization/en.json"},
                {"id": "ru", "text": "localization/ru.json"}
            ]
        })),
    );

    write(
        root,
        "data/tables/xdt.json",
        &pretty(&json!({
            "schema":"ffone.table-set.v1", "tables":[{"name":"native_asset_routes", "value":{
                "m_pAudioData":[{"category":"sfx", "path":"audio/sfx/ping.ogg"}],
                "m_pCharacterModelData":[{"id":"mob/mob_test", "category":"mob", "glb":"characters/mobs/mob_test/mob_test.glb"}]
            }}]
        })),
    );
    write(
        root,
        "map/catalog.json",
        &pretty(&json!({
            "schema": "ffone.map-catalog.v1",
            "resourceSets": [{
                "definition": reference(map_set_path, &map_set)
            }],
            "geometry": [{"model": reference(map_model_path, map_model)}],
            "objects": [{"definition": reference(map_object_path, &map_object)}],
            "tiles": [],
            "sharedFiles": [reference(map_texture_path, map_texture)]
        })),
    );
    write(
        root,
        "characters/player/items/catalog.json",
        &pretty(&json!({
            "schema": "ffone.player-item-set-catalog.v1",
            "sets": [{"definition": reference(player_set_path, &player_set)}],
            "models": [{
                "category": "hat",
                "trueName": "hat",
                "sourceRoute": "hat/hat.glb",
                "resourceSet": "player-item-set-hat",
                "model": reference(player_model_path, player_model)
            }],
            "renderingTextures": []
        })),
    );
}

#[test]
fn validation_is_read_only_and_accepts_modified_payloads() {
    let temp = tempdir().unwrap();
    fixture(temp.path());
    let fast = validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: false,
    })
    .unwrap();
    assert_eq!(fast.domains, 4);
    assert!(fast.references >= 7);
    assert_eq!(fast.root_blake3, None);

    let full = validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: true,
    })
    .unwrap();
    assert!(full.files >= 12);
    assert!(full.root_blake3.is_some());
    assert!(!temp.path().join("asset-manifest.json").exists());

    write(temp.path(), "audio/sfx/ping.ogg", b"BAD!");
    validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: true,
    })
    .unwrap();

    write(
        temp.path(),
        "characters/mobs/mob_test/mob_test.glb",
        b"BAD!-character",
    );
    validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: true,
    })
    .unwrap();
}

#[test]
fn server_shaped_xdt_preserves_native_routes_when_packaged() {
    let source = tempdir().unwrap();
    let release = tempdir().unwrap();
    fixture(source.path());
    let path = source.path().join("data/tables/xdt.json");
    let mut native: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    native["schema"] = json!("ffone.xdt.v1");
    native["gameplay_table"] = json!(1);
    native["tables"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"npc_imports_consolidated","key":"keep"}));
    let xdt = json!({"m_pMissionTable":{"m_pMissionData":[{"m_iHTaskID":77,"extra":true}]},"_ffone":native});
    write(source.path(), "data/tables/xdt.json", &pretty(&xdt));
    package_loose(&PackageOptions {
        source_root: source.path().into(),
        release_root: release.path().into(),
    })
    .unwrap();
    let published: serde_json::Value = serde_json::from_slice(
        &fs::read(release.path().join("assets/game/data/tables/xdt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(published, xdt);
    verify_asset_root(&VerifyOptions {
        asset_root: release.path().join("assets/game"),
        full: true,
    })
    .unwrap();
}

#[test]
fn shiny_routes_accept_logical_name_casing_but_stay_in_their_package() {
    let temp = tempdir().unwrap();
    fixture(temp.path());
    write(
        temp.path(),
        "characters/shinies/shineni_Item/shineni_Item.glb",
        b"glTF-shiny",
    );
    let tables: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.path().join("data/tables/xdt.json")).unwrap())
            .unwrap();
    let validate_with_shiny_route = |glb: &str| {
        let mut tables = tables.clone();
        tables["tables"][0]["value"]["m_pCharacterModelData"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id": "shiny/shineni_item", "category": "shiny", "glb": glb}));
        write(temp.path(), "data/tables/xdt.json", &pretty(&tables));
        validate_assets(&AssetValidationOptions {
            asset_root: temp.path().to_path_buf(),
            full: true,
        })
    };

    validate_with_shiny_route("characters/shinies/shineni_Item/shineni_Item.glb").unwrap();
    let error = validate_with_shiny_route("characters/mobs/mob_test/mob_test.glb").unwrap_err();
    assert!(error.contains("is outside package"), "{error}");
}

#[test]
fn reclassified_characters_keep_ids_but_follow_category_and_package() {
    let temp = tempdir().unwrap();
    fixture(temp.path());
    let glb = "characters/npcs/npc_mandroid_chef/npc_mandroid_chef.glb";
    write(temp.path(), glb, b"glTF-chef");
    let tables: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.path().join("data/tables/xdt.json")).unwrap())
            .unwrap();
    let validate_route = |id: &str, category: &str, glb: &str| {
        let mut tables = tables.clone();
        tables["tables"][0]["value"]["m_pCharacterModelData"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id": id, "category": category, "glb": glb}));
        write(temp.path(), "data/tables/xdt.json", &pretty(&tables));
        validate_assets(&AssetValidationOptions {
            asset_root: temp.path().to_path_buf(),
            full: true,
        })
    };

    validate_route("shared/npc_mandroid_chef", "npc", glb).unwrap();
    let error = validate_route("npc/npc_mandroid_chef", "fusion", glb).unwrap_err();
    assert!(error.contains("outside package"), "{error}");
    let error = validate_route("shared/npc_mandroid1", "npc", glb).unwrap_err();
    assert!(error.contains("outside package"), "{error}");
    let error = validate_route("shared/npc_mandroid_chef", "shared", glb).unwrap_err();
    assert!(error.contains("unsupported character category"), "{error}");
    for id in ["unknown/npc_mandroid_chef", "shared/..", "shared/a/b"] {
        let error = validate_route(id, "npc", glb).unwrap_err();
        assert!(error.contains("valid namespace/package slug"), "{error}");
    }
}

#[test]
fn map_shared_files_reject_paths_outside_map_owned_roots() {
    let temp = tempdir().unwrap();
    fixture(temp.path());
    let catalog_path = temp.path().join("map/catalog.json");
    let mut catalog: serde_json::Value =
        serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    catalog["sharedFiles"][0] =
        reference("characters/mobs/mob_test/mob_test.glb", b"glTF-character");
    write(temp.path(), "map/catalog.json", &pretty(&catalog));

    let error = validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: false,
    })
    .unwrap_err();
    assert!(
        error.contains("expected prefix \"map/\" or \"objects/\""),
        "{error}"
    );
}

#[test]
fn map_shared_files_accept_shared_png_mips_but_reject_other_domain_payloads() {
    let temp = tempdir().unwrap();
    fixture(temp.path());
    let catalog_path = temp.path().join("map/catalog.json");
    let mut catalog: serde_json::Value =
        serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    for relative in [
        "effects/shared/textures/tree.png",
        "effects/shared/textures/tree.mips/mip-01.png",
        "characters/npcs/nanomachine/textures/shared/tree.png",
    ] {
        write(temp.path(), relative, b"shared texture");
        catalog["sharedFiles"]
            .as_array_mut()
            .unwrap()
            .push(reference(relative, b"shared texture"));
    }
    write(temp.path(), "map/catalog.json", &pretty(&catalog));
    validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: true,
    })
    .unwrap();

    for relative in [
        "effects/shared/textures/character.glb",
        "textures/other/tree.png",
        "effects/shared/textures/../outside.png",
    ] {
        catalog["sharedFiles"][0] = reference(relative, b"invalid");
        write(temp.path(), "map/catalog.json", &pretty(&catalog));
        assert!(
            validate_assets(&AssetValidationOptions {
                asset_root: temp.path().to_path_buf(),
                full: true
            })
            .is_err(),
            "accepted {relative}"
        );
    }
}

#[test]
fn full_validation_accepts_stale_resource_set_identity_metadata() {
    let temp = tempdir().unwrap();
    fixture(temp.path());

    let set_path = "objects/nature/tree_set/set.json";
    let mut set: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.path().join(set_path)).unwrap()).unwrap();
    set["textures"][0]["blake3"] = "0".repeat(64).into();
    let set = pretty(&set);
    write(temp.path(), set_path, &set);

    let mut catalog: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.path().join("map/catalog.json")).unwrap()).unwrap();
    catalog["resourceSets"][0]["definition"] = reference(set_path, &set);
    write(temp.path(), "map/catalog.json", &pretty(&catalog));

    validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: true,
    })
    .unwrap();
}

#[test]
fn full_validation_accepts_modified_tutorial_payloads() {
    let temp = tempdir().unwrap();
    fixture(temp.path());

    let closure_path = "map/shared/effects/es60.closure.json";
    let closure = b"{\"schema\":\"fixture.effect.v1\"}\n";
    write(temp.path(), closure_path, closure);
    write(
        temp.path(),
        "map/shared/effects/catalog.json",
        &pretty(&json!({
            "schema": "ffone.tutorial-effect-catalog.v1",
            "effects": [{
                "closurePath": closure_path,
                "closureBytes": closure.len(),
                "closureBlake3": blake3::hash(closure).to_hex().to_string()
            }]
        })),
    );

    validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: true,
    })
    .unwrap();

    write(temp.path(), closure_path, b"stale closure\n");
    validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: true,
    })
    .unwrap();
}

#[test]
fn retired_global_metadata_is_rejected_even_by_the_fast_gate() {
    let temp = tempdir().unwrap();
    fixture(temp.path());
    write(temp.path(), "asset-index.json", b"{}\n");
    let error = validate_assets(&AssetValidationOptions {
        asset_root: temp.path().to_path_buf(),
        full: false,
    })
    .unwrap_err();
    assert!(error.contains("retired global asset metadata"), "{error}");
}

#[test]
fn release_keeps_every_asset_loose_and_editable() {
    let source = tempdir().unwrap();
    let release = tempdir().unwrap();
    fixture(source.path());

    let report = package_loose(&PackageOptions {
        source_root: source.path().to_path_buf(),
        release_root: release.path().to_path_buf(),
    })
    .unwrap();
    assert!(report.files >= 12);
    let game = release.path().join("assets/game");
    assert!(game.join("map/catalog.json").is_file());
    assert!(game.join("characters/player/items/catalog.json").is_file());
    assert!(game.join("data/tables/xdt.json").is_file());
    assert!(
        game.join("objects/nature/tree_set/models/tree/model.glb")
            .is_file()
    );
    assert!(
        game.join("objects/nature/tree_set/textures/tree.png")
            .is_file()
    );
    assert!(game.join("objects/nature/tree_set/set.json").is_file());
    assert!(
        game.join("objects/nature/tree_set/models/tree/object.json")
            .is_file()
    );
    assert!(!game.join(".ffone-packs").exists());

    verify_asset_root(&VerifyOptions {
        asset_root: game,
        full: true,
    })
    .unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(release.path().join("ffone-release-assets.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["packagingMode"], "loose-editable");
    assert!(receipt.get("rootGraphBlake3").is_none());
    assert!(receipt.get("payloadBytes").is_none());
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(release.path().join("ffone-package-cache.json")).unwrap())
            .unwrap();
    assert!(cache.get("rootGraphBlake3").is_none());
    for path in [
        "data/tables/xdt.json",
        "audio/sfx/ping.ogg",
        "localization/catalog.json",
        "localization/en.json",
        "localization/ru.json",
    ] {
        assert!(
            cache["files"][path].get("bytes").is_none()
                && cache["files"][path].get("blake3").is_none(),
            "package cache entry {path:?} stored content identity"
        );
    }
}

#[test]
fn release_verifier_accepts_modified_loose_payloads() {
    let source = tempdir().unwrap();
    let release = tempdir().unwrap();
    fixture(source.path());
    package_loose(&PackageOptions {
        source_root: source.path().to_path_buf(),
        release_root: release.path().to_path_buf(),
    })
    .unwrap();
    let game = release.path().join("assets/game");
    let set_path = "objects/nature/tree_set/set.json";
    let mut set: serde_json::Value =
        serde_json::from_slice(&fs::read(game.join(set_path)).unwrap()).unwrap();
    set["textures"][0]["blake3"] = "0".repeat(64).into();
    let set = pretty(&set);
    write(&game, set_path, &set);
    let mut catalog: serde_json::Value =
        serde_json::from_slice(&fs::read(game.join("map/catalog.json")).unwrap()).unwrap();
    catalog["resourceSets"][0]["definition"] = reference(set_path, &set);
    write(&game, "map/catalog.json", &pretty(&catalog));

    verify_asset_root(&VerifyOptions {
        asset_root: game,
        full: true,
    })
    .unwrap();
}

#[test]
fn release_rejects_retired_models_world_tree() {
    let source = tempdir().unwrap();
    let release = tempdir().unwrap();
    fixture(source.path());
    write(
        source.path(),
        "models/world/maps/map_01_02/orphan.glb",
        b"orphan-world-model",
    );

    let error = package_loose(&PackageOptions {
        source_root: source.path().to_path_buf(),
        release_root: release.path().to_path_buf(),
    })
    .unwrap_err();
    assert!(
        error.contains("has no declared runtime owner group"),
        "{error}"
    );
}

#[test]
fn release_rejects_retired_global_texture_tree() {
    let source = tempdir().unwrap();
    let release = tempdir().unwrap();
    fixture(source.path());
    write(
        source.path(),
        "textures/orphan.png",
        b"orphan-global-texture",
    );

    let error = package_loose(&PackageOptions {
        source_root: source.path().to_path_buf(),
        release_root: release.path().to_path_buf(),
    })
    .unwrap_err();
    assert!(
        error.contains("has no declared runtime owner group"),
        "{error}"
    );
}

#[test]
fn release_snapshot_accepts_translation_edits_without_integrity_checks() {
    let source = tempdir().unwrap();
    let release = tempdir().unwrap();
    fixture(source.path());
    let (_, snapshot) = snapshot_release_source(source.path().to_path_buf()).unwrap();
    write(source.path(), "localization/en.json", b"changed\n");
    package_loose_from_snapshot(
        &PackageOptions {
            source_root: source.path().to_path_buf(),
            release_root: release.path().to_path_buf(),
        },
        &snapshot,
    )
    .unwrap();
    assert_eq!(
        fs::read(release.path().join("assets/game/localization/en.json")).unwrap(),
        b"changed\n"
    );
}

#[test]
fn release_repackages_same_length_audio_replacements_without_hashes() {
    let source = tempdir().unwrap();
    let release = tempdir().unwrap();
    fixture(source.path());
    let options = PackageOptions {
        source_root: source.path().to_path_buf(),
        release_root: release.path().to_path_buf(),
    };
    package_loose(&options).unwrap();
    write(source.path(), "audio/sfx/ping.ogg", b"NEW!");
    package_loose(&options).unwrap();
    assert_eq!(
        fs::read(release.path().join("assets/game/audio/sfx/ping.ogg")).unwrap(),
        b"NEW!"
    );
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(release.path().join("ffone-package-cache.json")).unwrap())
            .unwrap();
    assert!(cache["files"]["audio/sfx/ping.ogg"].get("blake3").is_none());
}
