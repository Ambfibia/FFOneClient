use super::*;
use serde_json::{Value, json};

fn packages_root() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("packages");
    fs::create_dir_all(&root).unwrap();
    (temp, root)
}

fn write_manifest(root: &Path, directory: &str, id: &str, requires: Value) -> PathBuf {
    let package = root.join(directory);
    fs::create_dir_all(&package).unwrap();
    write_json(
        &package.join(PACKAGE_MANIFEST_FILE),
        &json!({
            "schema": PACKAGE_SCHEMA,
            "id": id,
            "version": "1.0.0",
            "protocol": "native-v1",
            "requires": requires,
        }),
    );
    package
}

fn write_json(path: &Path, value: &Value) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn npc(id: &str, network_id: u32) -> Value {
    json!({
        "schema": NPC_DEFINITION_SCHEMA,
        "id": id,
        "networkId": network_id,
    })
}

#[test]
fn discovers_only_direct_packages_in_deterministic_dependency_order() {
    let (_temp, root) = packages_root();
    let base = write_manifest(&root, "z-folder", "base", json!([]));
    write_json(
        &base.join("npcs/vendor/npc.ffdef.json"),
        &json!({
            "schema": NPC_DEFINITION_SCHEMA,
            "id": "vendor",
            "networkId": 41,
            "assets": { "model": "art/vendor.glb" },
            "data": { "oldOnly": true }
        }),
    );
    write_json(
        &base.join("nanos/blossom/nano.ffdef.json"),
        &json!({
            "schema": NANO_DEFINITION_SCHEMA,
            "id": "blossom",
            "networkId": 41
        }),
    );
    write_json(
        &base.join("items/weapon/hammer/item.ffdef.json"),
        &json!({
            "schema": ITEM_DEFINITION_SCHEMA,
            "id": "hammer",
            "networkId": { "type": 1, "number": 8 }
        }),
    );

    let addon = write_manifest(
        &root,
        "a-folder",
        "addon",
        json!([{ "id": "base", "version": "1.0.0" }]),
    );
    write_json(
        &addon.join("npcs/vendor-remaster/npc.ffdef.json"),
        &json!({
            "schema": NPC_DEFINITION_SCHEMA,
            "id": "vendor-remaster",
            "networkId": 41,
            "replaces": "base:npc/vendor",
            "data": { "newOnly": true }
        }),
    );
    write_manifest(&root, "m-folder", "independent", json!([]));

    let ignored = root.join("not-a-package/nested");
    fs::create_dir_all(&ignored).unwrap();
    write_json(
        &ignored.join(PACKAGE_MANIFEST_FILE),
        &json!({
            "schema": PACKAGE_SCHEMA,
            "id": "nested",
            "version": "1.0.0",
            "protocol": "native-v1"
        }),
    );

    let registry = discover_native_packages(&root).unwrap();
    let ordered_ids = registry
        .packages()
        .iter()
        .map(|package| package.manifest().id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ordered_ids, ["base", "addon", "independent"]);
    assert_eq!(registry.npcs().len(), 1);
    assert_eq!(registry.nanos().len(), 1);
    assert_eq!(registry.items().len(), 1);

    let replacement = registry.npc("base:npc/vendor").unwrap();
    assert_eq!(replacement.canonical_id(), "addon:npc/vendor-remaster");
    assert_eq!(
        replacement.definition().data.get("newOnly"),
        Some(&json!(true))
    );
    assert!(!replacement.definition().data.contains_key("oldOnly"));
    assert_eq!(
        registry
            .nano("base:nano/blossom")
            .unwrap()
            .definition()
            .network_id,
        41
    );
    assert_eq!(
        registry
            .item("base:item/weapon/hammer")
            .unwrap()
            .definition()
            .network_id,
        ItemNetworkId {
            item_type: 1,
            number: 8,
        }
    );

    let original_model = registry.replacements()["base:npc/vendor"].as_str();
    assert_eq!(original_model, "addon:npc/vendor-remaster");
}

#[test]
fn asset_paths_are_leaf_relative_and_traversal_is_rejected() {
    let (_temp, root) = packages_root();
    let package = write_manifest(&root, "base", "base", json!([]));
    write_json(
        &package.join("npcs/vendor/npc.ffdef.json"),
        &json!({
            "schema": NPC_DEFINITION_SCHEMA,
            "id": "vendor",
            "networkId": 4,
            "assets": { "model": "art/vendor.glb" }
        }),
    );
    let registry = discover_native_packages(&root).unwrap();
    let loaded = registry.npc("base:npc/vendor").unwrap();
    assert_eq!(
        loaded.resolve_asset_path(&loaded.definition().assets["model"]),
        package.join("npcs/vendor/art/vendor.glb")
    );

    write_json(
        &package.join("npcs/vendor/npc.ffdef.json"),
        &json!({
            "schema": NPC_DEFINITION_SCHEMA,
            "id": "vendor",
            "networkId": 4,
            "assets": { "model": "../outside.glb" }
        }),
    );
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::UnsafeRelativePath { .. })
    ));
}

#[test]
fn rejects_missing_dependencies_version_mismatches_and_cycles() {
    let (_temp, root) = packages_root();
    write_manifest(
        &root,
        "addon",
        "addon",
        json!([{ "id": "missing", "version": "1.0.0" }]),
    );
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::MissingDependency { .. })
    ));

    let (_temp, root) = packages_root();
    write_manifest(&root, "base", "base", json!([]));
    write_manifest(
        &root,
        "addon",
        "addon",
        json!([{ "id": "base", "version": "2.0.0" }]),
    );
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::DependencyVersionMismatch { .. })
    ));

    let (_temp, root) = packages_root();
    write_manifest(
        &root,
        "one",
        "one",
        json!([{ "id": "two", "version": "1.0.0" }]),
    );
    write_manifest(
        &root,
        "two",
        "two",
        json!([{ "id": "one", "version": "1.0.0" }]),
    );
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::DependencyCycle { .. })
    ));
}

#[test]
fn rejects_duplicate_package_canonical_and_typed_network_ids() {
    let (_temp, root) = packages_root();
    write_manifest(&root, "one", "same", json!([]));
    write_manifest(&root, "two", "same", json!([]));
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::DuplicatePackageId { .. })
    ));

    let (_temp, root) = packages_root();
    let package = write_manifest(&root, "base", "base", json!([]));
    write_json(&package.join("npcs/one/npc.ffdef.json"), &npc("same", 1));
    write_json(&package.join("npcs/two/npc.ffdef.json"), &npc("same", 2));
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::DuplicateCanonicalId { .. })
    ));

    let (_temp, root) = packages_root();
    let package = write_manifest(&root, "base", "base", json!([]));
    write_json(&package.join("npcs/one/npc.ffdef.json"), &npc("one", 7));
    write_json(&package.join("npcs/two/npc.ffdef.json"), &npc("two", 7));
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::DuplicateNetworkId { kind: "NPC", .. })
    ));

    // Numeric IDs are intentionally namespaced by definition kind.
    write_json(&package.join("npcs/two/npc.ffdef.json"), &npc("two", 8));
    write_json(
        &package.join("nanos/one/nano.ffdef.json"),
        &json!({
            "schema": NANO_DEFINITION_SCHEMA,
            "id": "one",
            "networkId": 7
        }),
    );
    assert!(discover_native_packages(&root).is_ok());
}

#[test]
fn replacement_requires_dependency_target_and_preserved_network_key() {
    let (_temp, root) = packages_root();
    let base = write_manifest(&root, "base", "base", json!([]));
    write_json(&base.join("npcs/base/npc.ffdef.json"), &npc("base", 9));
    let addon = write_manifest(&root, "addon", "addon", json!([]));
    write_json(
        &addon.join("npcs/new/npc.ffdef.json"),
        &json!({
            "schema": NPC_DEFINITION_SCHEMA,
            "id": "new",
            "networkId": 9,
            "replaces": "base:npc/base"
        }),
    );
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::ReplacementNotDependency { .. })
    ));

    write_json(
        &addon.join(PACKAGE_MANIFEST_FILE),
        &json!({
            "schema": PACKAGE_SCHEMA,
            "id": "addon",
            "version": "1.0.0",
            "protocol": "native-v1",
            "requires": [{ "id": "base", "version": "1.0.0" }]
        }),
    );
    write_json(
        &addon.join("npcs/new/npc.ffdef.json"),
        &json!({
            "schema": NPC_DEFINITION_SCHEMA,
            "id": "new",
            "networkId": 10,
            "replaces": "base:npc/base"
        }),
    );
    assert!(matches!(
        discover_native_packages(&root),
        Err(PackageRegistryError::ReplacementNetworkMismatch { .. })
    ));
}
