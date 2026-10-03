use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use bevy::{input::keyboard::KeyboardInput, prelude::*};
use serde::Deserialize;

use ffone_client::{
    character_creation_ui::{
        CharacterCreationScreen, CharacterCreationUiModel, CharacterNameMode,
        NativeCharacterCreationUiPlugin,
    },
    localization::{Localization, LocalizationPlugin, LocalizedText},
};

#[derive(Deserialize)]
struct TextBundle {
    entries: BTreeMap<String, String>,
}

fn asset_root() -> PathBuf {
    project_root().join("assets/game")
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bundle(locale: &str) -> TextBundle {
    let path = asset_root()
        .join("localization")
        .join(format!("{locale}.json"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn placeholders(value: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut rest = value;
    while let Some(start) = rest.find('{') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('}') else {
            break;
        };
        result.insert(rest[..end].to_owned());
        rest = &rest[end + 1..];
    }
    result
}

fn localized_app(locale: &str) -> App {
    let root = asset_root();
    let (localization, language) = Localization::open(&root, locale).unwrap();
    let mut app = App::new();
    app.insert_resource(localization)
        .insert_resource(language)
        .add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins(AssetPlugin {
            file_path: root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins((LocalizationPlugin, NativeCharacterCreationUiPlugin));
    app.update();
    app.update();
    app
}

#[test]
fn production_en_ru_bundles_have_exact_character_creation_key_and_placeholder_parity() {
    let en = bundle("en");
    let ru = bundle("ru");
    assert_eq!(
        en.entries.keys().collect::<BTreeSet<_>>(),
        ru.entries.keys().collect::<BTreeSet<_>>()
    );
    for (key, en_value) in &en.entries {
        assert_eq!(
            placeholders(en_value),
            placeholders(&ru.entries[key]),
            "placeholder mismatch for {key}"
        );
    }

    let mut app = localized_app("en");
    let world = app.world_mut();
    let mut texts = world.query::<&LocalizedText>();
    for localized in texts.iter(world) {
        assert!(
            en.entries.contains_key(&localized.key),
            "spawned key {} is absent from production bundles",
            localized.key
        );
        assert_eq!(
            placeholders(&en.entries[&localized.key]),
            localized.args.keys().cloned().collect(),
            "runtime arguments mismatch template {}",
            localized.key
        );
    }
}

#[test]
fn generated_custom_and_appearance_states_render_cyrillic_from_ru_bundle() {
    let mut app = localized_app("ru");
    for (screen, mode) in [
        (CharacterCreationScreen::Name, CharacterNameMode::Generated),
        (CharacterCreationScreen::Name, CharacterNameMode::Custom),
        (
            CharacterCreationScreen::Appearance,
            CharacterNameMode::Generated,
        ),
    ] {
        {
            let mut model = app.world_mut().resource_mut::<CharacterCreationUiModel>();
            model.visible = true;
            model.screen = screen;
            model.name_mode = mode;
            model.custom_name = "Astra Nova".to_owned();
        }
        app.update();
        let world = app.world_mut();
        let expected_key = match (screen, mode) {
            (CharacterCreationScreen::Name, CharacterNameMode::Generated) => {
                "ui.character_create.generated_name_rules"
            }
            (CharacterCreationScreen::Name, CharacterNameMode::Custom) => {
                "ui.character_create.custom_name_question"
            }
            (CharacterCreationScreen::Appearance, _) => "ui.character_create.step.body",
        };
        let mut texts = world.query::<(&Text, &LocalizedText)>();
        assert!(texts.iter(world).any(|(text, localized)| {
            localized.key == expected_key
                && text
                    .0
                    .chars()
                    .any(|character| ('\u{0400}'..='\u{052f}').contains(&character))
        }));
    }
}

#[test]
fn source_has_no_unkeyed_text_constructor_or_manual_rendered_text_writer() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/character_creation_ui.rs"),
    )
    .unwrap();
    let lines = source.lines().collect::<Vec<_>>();
    let constructors = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("Text::new("))
        .collect::<Vec<_>>();
    assert_eq!(constructors.len(), 11);
    for (index, line) in constructors {
        assert!(
            lines[index + 1..lines.len().min(index + 7)]
                .iter()
                .any(|candidate| candidate.contains("LocalizedText::new(")),
            "unkeyed constructor: {line}"
        );
    }
    assert!(!source.contains("text.0 ="));
    assert!(!source.contains("text.0.clone_from"));
}
