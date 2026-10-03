use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use ffone_client::buddy_ui::{
    BUDDY_LARGE_LIST_BACKGROUND_PATH, BUDDY_LARGE_LIST_BACKGROUND_SOURCE_TEXTURE_PATH_ID,
    BUDDY_SCROLL_DOWN_PATH, BUDDY_SCROLL_DOWN_SOURCE_TEXTURE_PATH_ID, BUDDY_SCROLL_THUMB_PATH,
    BUDDY_SCROLL_THUMB_SOURCE_TEXTURE_PATH_ID, BUDDY_SCROLL_TRACK_PATH,
    BUDDY_SCROLL_TRACK_SOURCE_TEXTURE_PATH_ID, BUDDY_SCROLL_UP_PATH,
    BUDDY_SCROLL_UP_SOURCE_TEXTURE_PATH_ID,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bundle(path: &Path) -> BTreeMap<String, String> {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let document: Value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
    let entries = serde_json::from_value(document["entries"].clone())
        .unwrap_or_else(|error| panic!("parse {} entries: {error}", path.display()));
    entries
}

fn placeholders(value: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut remainder = value;
    while let Some(start) = remainder.find('{') {
        remainder = &remainder[start + 1..];
        let Some(end) = remainder.find('}') else {
            break;
        };
        result.insert(remainder[..end].to_owned());
        remainder = &remainder[end + 1..];
    }
    result
}

fn sha256(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    format!("{:X}", Sha256::digest(bytes))
}

#[test]
fn buddy_semantic_keys_are_exact_in_runtime_en_ru_publications() {
    let root = workspace_root();
    let runtime_en = bundle(&root.join("assets/game/localization/en.json"));
    let runtime_ru = bundle(&root.join("assets/game/localization/ru.json"));

    assert_eq!(
        runtime_en.keys().collect::<Vec<_>>(),
        runtime_ru.keys().collect::<Vec<_>>()
    );

    for key in runtime_en.keys() {
        assert_eq!(
            placeholders(&runtime_en[key]),
            placeholders(&runtime_ru[key]),
            "placeholder mismatch for {key}"
        );
    }
    for (key, expected) in [
        ("ui.buddy.row.verified", "    {display_name}"),
        ("ui.buddy.row.empty", ""),
        ("ui.buddy.add_name_input", "{name}"),
    ] {
        assert_eq!(runtime_en.get(key).map(String::as_str), Some(expected));
        assert_eq!(runtime_ru.get(key).map(String::as_str), Some(expected));
    }
}

#[test]
fn reused_scroll_primitives_are_byte_exact_validated_primary_conversions() {
    let asset_root = workspace_root().join("assets/game");
    for (path, path_id, bytes, expected_sha256) in [
        (
            BUDDY_SCROLL_TRACK_PATH,
            BUDDY_SCROLL_TRACK_SOURCE_TEXTURE_PATH_ID,
            345,
            "9868058BDACCC651B3C0B6D1AA27D29587EA9484F453DEFCA0D20908463AB930",
        ),
        (
            BUDDY_SCROLL_THUMB_PATH,
            BUDDY_SCROLL_THUMB_SOURCE_TEXTURE_PATH_ID,
            413,
            "5D9312B400EC1FED440A5C59097AF34E3D16352C9F0C5715E450AE50B4DF5B19",
        ),
        (
            BUDDY_SCROLL_UP_PATH,
            BUDDY_SCROLL_UP_SOURCE_TEXTURE_PATH_ID,
            525,
            "4B1A86AD7CD3D5A885F1E0137769D0DE9F3CA2226572573BF541E14278F660EE",
        ),
        (
            BUDDY_SCROLL_DOWN_PATH,
            BUDDY_SCROLL_DOWN_SOURCE_TEXTURE_PATH_ID,
            499,
            "79403866BF80A0EECFEBEFC065653420E8EA8C1DD659B1A8A8B330B0AFDF8691",
        ),
    ] {
        assert!(matches!(path_id, 374 | 324 | 63 | 415));
        let file = asset_root.join(path);
        assert_eq!(fs::metadata(&file).unwrap().len(), bytes, "{path}");
        assert_eq!(sha256(&file), expected_sha256, "pathId {path_id}: {path}");
    }
}

#[test]
fn reached_large_chat_background_is_the_exact_primary_style_texture() {
    assert_eq!(BUDDY_LARGE_LIST_BACKGROUND_SOURCE_TEXTURE_PATH_ID, 610);
    let file = workspace_root()
        .join("assets/game")
        .join(BUDDY_LARGE_LIST_BACKGROUND_PATH);
    assert_eq!(fs::metadata(&file).unwrap().len(), 1_656);
    assert_eq!(
        sha256(&file),
        "F57D7E487D991A0BA4C64F861A75A731A1157417489261A0D6F144A9CD291360"
    );
}

#[test]
fn buddy_production_source_has_no_generic_passthrough_or_raw_text_write() {
    let source = concat!(
        include_str!("../src/ui/buddy/constants.rs"),
        "\n",
        include_str!("../src/ui/buddy/state.rs"),
        "\n",
        include_str!("../src/ui/buddy/layout.rs"),
        "\n",
        include_str!("../src/ui/buddy/interaction.rs"),
        "\n",
        include_str!("../src/ui/buddy/assets.rs"),
        "\n",
        include_str!("../src/ui/buddy/textures.rs"),
        "\n",
        include_str!("../src/ui/buddy/validation.rs"),
        "\n",
        include_str!("../src/ui/buddy/types.rs"),
        "\n",
        include_str!("../src/ui/buddy/commands.rs"),
        "\n",
        include_str!("../src/ui/buddy/models.rs"),
        "\n",
        include_str!("../src/ui/buddy/operations.rs"),
        "\n",
        include_str!("../src/ui/buddy/view.rs"),
        "\n",
        include_str!("../src/ui/buddy/localization_buddy_static_localized.rs"),
        "\n",
        include_str!("../src/ui/buddy/systems.rs"),
        "\n",
        include_str!("../src/ui/buddy/mod.rs")
    );
    let production = source.split("#[cfg(test)]").next().unwrap();
    assert!(!production.contains("ui.content.passthrough"));
    assert!(!production.contains("buddy_passthrough"));
    assert!(!production.contains("**text ="));
    assert!(!production.contains("text.0 ="));
}
