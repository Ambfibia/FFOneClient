use super::*;

#[test]
fn opens_an_asset_without_enforcing_legacy_identity_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("characters/nano/test/test.glb");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"glTF").unwrap();
    let assets = AssetLocator::open(temp.path()).unwrap();
    assert_eq!(
        assets.read("characters/nano/test/test.glb").unwrap(),
        b"glTF"
    );
    assert!(
        assets
            .read_verified(
                "characters/nano/test/test.glb",
                Some(999),
                "stale-hash-from-an-editable-catalog",
            )
            .is_ok()
    );
}

#[test]
fn rejects_escape_paths() {
    assert!(validate_relative_path("../outside.glb").is_err());
    assert!(validate_relative_path("models\\outside.glb").is_err());
}

#[test]
fn package_conventions_are_plural_and_stable() {
    assert_eq!(
        AssetLocator::character_package_root("mob", "mob_test").unwrap(),
        "characters/mobs/mob_test"
    );
    assert_eq!(
        AssetLocator::world_tile_root("worldMap", "map_01_02").unwrap(),
        "map/tiles/map_01_02"
    );
    assert_eq!(
        AssetLocator::world_tile_root("tutorial", "tile_01_01").unwrap(),
        "map/tiles/map_01_01"
    );
}
