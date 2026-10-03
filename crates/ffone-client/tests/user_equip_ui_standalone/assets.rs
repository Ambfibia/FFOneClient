use super::*;

#[test]
fn runtime_nano_catalog_has_only_named_unique_publications() {
    let nano_root = workspace_root().join("assets/game/icons/entities/nanos");
    for (directory, prefix) in [
        (&nano_root, "nanoicon_"),
        (&nano_root.join("ready"), "nanoready_"),
    ] {
        let mut hashes = BTreeMap::<String, String>::new();
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with(prefix) || !name.ends_with(".png") {
                continue;
            }
            let stem = name
                .strip_prefix(prefix)
                .unwrap()
                .strip_suffix(".png")
                .unwrap();
            assert!(
                stem.chars()
                    .any(|character| character.is_ascii_alphabetic()),
                "numeric Nano publication remains at {}",
                entry.path().display()
            );
            let hash = sha256_lower(&fs::read(entry.path()).unwrap());
            assert!(
                hashes.insert(hash.clone(), name.clone()).is_none(),
                "duplicate Nano bytes {hash} at {name} and {:?}",
                hashes.get(&hash)
            );
        }
    }
}
