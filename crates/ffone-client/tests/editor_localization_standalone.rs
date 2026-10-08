use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bundle(locale: &str) -> Value {
    let path = repo_root()
        .join("assets/game/localization")
        .join(format!("{locale}.json"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn editor_entries(document: &Value) -> Vec<(String, String)> {
    document["entries"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(key, _)| key.starts_with("ui.editor."))
        .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_owned()))
        .collect()
}

fn placeholders(value: &str) -> BTreeSet<String> {
    value
        .split('{')
        .skip(1)
        .filter_map(|tail| tail.split_once('}').map(|(name, _)| name.to_owned()))
        .collect()
}

#[test]
fn editor_keys_and_placeholders_match_between_english_and_russian() {
    let en = editor_entries(&bundle("en"));
    let ru = editor_entries(&bundle("ru"));
    assert!(!en.is_empty(), "editor localization must not be empty");
    assert_eq!(
        en.iter().map(|(key, _)| key).collect::<Vec<_>>(),
        ru.iter().map(|(key, _)| key).collect::<Vec<_>>()
    );
    for ((key, en_value), (_, ru_value)) in en.iter().zip(&ru) {
        assert_eq!(
            placeholders(en_value),
            placeholders(ru_value),
            "placeholder mismatch for {key}"
        );
    }
}

#[test]
fn every_editor_source_key_exists_in_the_production_bundles() {
    let source =
        fs::read_to_string(repo_root().join("crates/ffone-client/src/bin/ffone-editor/main.rs"))
            .unwrap();
    let equipment =
        fs::read_to_string(repo_root().join("crates/ffone-client/src/bin/editor/equipment.rs"))
            .unwrap();
    let strings =
        fs::read_to_string(repo_root().join("crates/ffone-client/src/bin/editor/strings.rs"))
            .unwrap();
    let exchange = fs::read_to_string(
        repo_root().join("crates/ffone-client/src/bin/editor/strings_exchange.rs"),
    )
    .unwrap();
    let editing_tools =
        fs::read_to_string(repo_root().join("crates/ffone-client/src/bin/editor/strings_tools.rs"))
            .unwrap();
    let all_sources = format!("{source}\n{equipment}\n{strings}\n{exchange}\n{editing_tools}");
    let text_helper = fs::read_to_string(repo_root().join(
        "crates/ffone-client/src/bin/ffone-editor/operations_handle_editor_buttons.rs")).unwrap();
    let authored = all_sources
        .split('"')
        .filter(|token| token.starts_with("ui.editor."))
        .collect::<BTreeSet<_>>();
    let en = editor_entries(&bundle("en"))
        .into_iter()
        .map(|(key, _)| key)
        .collect::<BTreeSet<_>>();
    let ru = editor_entries(&bundle("ru"))
        .into_iter()
        .map(|(key, _)| key)
        .collect::<BTreeSet<_>>();
    for key in authored {
        assert!(en.contains(key), "missing EN editor key {key}");
        assert!(ru.contains(key), "missing RU editor key {key}");
    }
    assert_eq!(
        text_helper.matches("text: Text::new(").count(),
        1,
        "all editor Text must be created through the key-first editor_text helper"
    );
}

#[test]
fn production_text_bundles_have_identical_keys_and_placeholders() {
    let en = bundle("en");
    let ru = bundle("ru");
    let en = en["entries"].as_object().unwrap();
    let ru = ru["entries"].as_object().unwrap();
    assert_eq!(en.keys().collect::<Vec<_>>(), ru.keys().collect::<Vec<_>>());
    for (key, value) in en {
        assert_eq!(
            placeholders(value.as_str().unwrap()),
            placeholders(ru[key].as_str().unwrap()),
            "{key}"
        );
    }
}

#[test]
fn mission_boundary_commands_and_npc_creation_are_localized() {
    let en = bundle("en");
    let ru = bundle("ru");
    for key in [
        "ui.editor.xdt.mission.make_first",
        "ui.editor.xdt.mission.make_last",
        "ui.editor.xdt.mission.end",
        "ui.editor.xdt.mission.error_boundary_chain",
        "ui.editor.xdt.mission.error_boundary_branch",
        "ui.editor.xdt.mission.rewrite",
        "ui.editor.xdt.mission.pending_work",
        "ui.editor.xdt.mission.saved_work",
        "ui.editor.xdt.mission.restored_work",
        "ui.editor.npc.new_template",
    ] {
        let english = en["entries"][key].as_str().expect(key);
        let russian = ru["entries"][key].as_str().expect(key);
        assert!(!english.trim().is_empty(), "empty EN {key}");
        assert!(!russian.trim().is_empty(), "empty RU {key}");
        assert_ne!(english, russian, "untranslated RU {key}");
    }
}
