use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use bevy::{input::keyboard::KeyboardInput, prelude::*};
use serde::Deserialize;

use super::{
    character_selection_ui::{
        CharacterLocationBackground, CharacterSelectionCapability, CharacterSelectionTextStyle,
        CharacterSelectionUiModel, CharacterSlotUi, NativeCharacterSelectionUiPlugin,
        OccupiedCharacterSlotUi,
    },
    localization::{Localization, LocalizationPlugin, LocalizedText},
};

#[derive(Deserialize)]
struct TextBundle {
    entries: BTreeMap<String, String>,
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn asset_root() -> PathBuf {
    project_root().join("assets/game")
}

fn bundle(path: &Path) -> TextBundle {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn placeholders(value: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut rest = value;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else {
            break;
        };
        result.insert(rest[..close].to_owned());
        rest = &rest[close + 1..];
    }
    result
}

fn localized_app(locale: &str) -> App {
    let asset_root = asset_root();
    let (localization, language) = Localization::open(&asset_root, locale).unwrap();
    let mut app = App::new();
    app.insert_resource(localization)
        .insert_resource(language)
        .add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins((NativeCharacterSelectionUiPlugin, LocalizationPlugin));
    app.update();
    app
}

fn occupied() -> CharacterSlotUi {
    CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
        pc_uid: 7,
        display_name: "Test Ser".to_owned(),
        level: 36,
        district: "TECH SQUARE".to_owned(),
        zone: "THE FUTURE".to_owned(),
        background: CharacterLocationBackground::Future,
    })
}

fn reached_slots() -> [CharacterSlotUi; 4] {
    [
        occupied(),
        CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
            pc_uid: 8,
            display_name: "Dexter Bell".to_owned(),
            level: 12,
            district: "POKÉ PLAZA".to_owned(),
            zone: "THE SUBURBS".to_owned(),
            background: CharacterLocationBackground::Suburbs,
        }),
        CharacterSlotUi::SubscriptionLockedOccupied(OccupiedCharacterSlotUi {
            pc_uid: 9,
            display_name: "Gaia Roundbreath".to_owned(),
            level: 36,
            district: "GENIUS GROVE".to_owned(),
            zone: "THE FUTURE".to_owned(),
            background: CharacterLocationBackground::Future,
        }),
        CharacterSlotUi::SubscriptionLocked,
    ]
}

#[test]
fn production_bundles_match_and_cover_every_character_selection_text() {
    let root = project_root();
    let runtime_en_path = root.join("assets/game/localization/en.json");
    let runtime_ru_path = root.join("assets/game/localization/ru.json");

    let en = bundle(&runtime_en_path).entries;
    let ru = bundle(&runtime_ru_path).entries;
    assert_eq!(
        en.keys().cloned().collect::<BTreeSet<_>>(),
        ru.keys().cloned().collect::<BTreeSet<_>>()
    );
    for (key, en_value) in &en {
        assert_eq!(placeholders(en_value), placeholders(&ru[key]), "{key}");
    }

    let mut app = localized_app("en");
    let localized = {
        let world = app.world_mut();
        let mut query = world.query::<(&LocalizedText, &CharacterSelectionTextStyle)>();
        query
            .iter(world)
            .map(|(text, _)| text.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(localized.len(), 27);
    for text in localized {
        assert!(en.contains_key(&text.key), "missing EN key {}", text.key);
        assert!(ru.contains_key(&text.key), "missing RU key {}", text.key);
        assert_eq!(
            text.args.keys().cloned().collect::<BTreeSet<_>>(),
            placeholders(&en[&text.key]),
            "runtime args for {}",
            text.key
        );
    }
}

#[test]
fn reached_en_and_ru_states_cover_the_exact_semantic_keys_and_all_styles() {
    let expected_keys = [
        "ui.character_select.title",
        "ui.character_select.empty",
        "ui.character_select.unlimited_only",
        "ui.character_select.level",
        "ui.character_select.location.spaced",
        "ui.character_select.location.hyphen",
        "ui.character_select.delete",
        "ui.character_select.create",
        "ui.character_select.enter",
        "ui.character_select.delete_prompt",
        "ui.common.cancel",
        "ui.common.delete",
        "ui.common.quit",
        "ui.content.passthrough",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    let expected_styles = [
        "Transparent2",
        "Transparent3",
        "CharNameUp",
        "CharNameDown",
        "CharLevelUp",
        "DeleteText",
        "AvatarName",
        "QuitButton",
        "CreateButton",
        "EnterGame",
        "Cancel",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    for locale in ["en", "ru"] {
        let mut app = localized_app(locale);
        {
            let mut model = app.world_mut().resource_mut::<CharacterSelectionUiModel>();
            model.visible = true;
            model.slots = reached_slots();
            model.selected_slot = Some(0);
            model.create = CharacterSelectionCapability::Enabled;
            model.delete = CharacterSelectionCapability::Enabled;
            model.delete_confirmation_pc_uid = Some(7);
            model.delete_name_input = "Test".to_owned();
        }
        app.update();

        let (keys, styles, total_texts, owned_texts) = {
            let world = app.world_mut();
            let total_texts = world.query::<&Text>().iter(world).count();
            let mut query = world.query::<(&LocalizedText, &CharacterSelectionTextStyle)>();
            let rows = query.iter(world).collect::<Vec<_>>();
            (
                rows.iter()
                    .map(|(localized, _)| localized.key.clone())
                    .collect::<BTreeSet<_>>(),
                rows.iter()
                    .map(|(_, style)| format!("{style:?}"))
                    .collect::<BTreeSet<_>>(),
                total_texts,
                rows.len(),
            )
        };
        assert_eq!(owned_texts, total_texts, "{locale} every-Text ownership");
        assert_eq!(keys, expected_keys, "{locale} reached semantic keys");
        assert_eq!(styles, expected_styles, "{locale} reached GUIStyles");
    }
}

#[test]
fn ru_populated_empty_and_delete_states_are_key_first_and_cyrillic() {
    let mut app = localized_app("ru");
    {
        let mut model = app.world_mut().resource_mut::<CharacterSelectionUiModel>();
        model.visible = true;
        model.slots = [
            occupied(),
            CharacterSlotUi::Empty,
            CharacterSlotUi::SubscriptionLocked,
            CharacterSlotUi::Empty,
        ];
        model.selected_slot = Some(0);
        model.create = CharacterSelectionCapability::Enabled;
        model.delete = CharacterSelectionCapability::Enabled;
    }
    app.update();
    let values = {
        let world = app.world_mut();
        let mut query = world.query::<(&Text, &LocalizedText)>();
        query
            .iter(world)
            .map(|(text, localized)| (text.0.clone(), localized.key.clone()))
            .collect::<Vec<_>>()
    };
    assert!(values.iter().any(|(text, key)| {
        key == "ui.character_select.title" && text == "ВЫБЕРИТЕ ПЕРСОНАЖА"
    }));
    assert!(values.iter().any(|(text, key)| {
        key == "ui.character_select.unlimited_only" && text == "ТОЛЬКО ПОЛНЫЙ ДОСТУП"
    }));
    assert!(values.iter().any(|(text, key)| {
        key == "ui.character_select.level" && text == "УРОВЕНЬ 36"
    }));

    app.world_mut()
        .resource_mut::<CharacterSelectionUiModel>()
        .delete_confirmation_pc_uid = Some(7);
    app.update();
    let modal_values = {
        let world = app.world_mut();
        let mut query = world.query::<(&Text, &LocalizedText)>();
        query
            .iter(world)
            .filter(|(text, _)| text.0.chars().any(|ch| ('А'..='я').contains(&ch)))
            .map(|(text, localized)| (text.0.clone(), localized.key.clone()))
            .collect::<Vec<_>>()
    };
    assert!(
        modal_values
            .iter()
            .any(|(_, key)| key == "ui.character_select.delete_prompt")
    );
    assert!(
        modal_values
            .iter()
            .any(|(_, key)| key == "ui.common.cancel")
    );
    assert!(
        modal_values
            .iter()
            .any(|(_, key)| key == "ui.common.delete")
    );
}

#[test]
fn source_has_no_unkeyed_text_constructor_or_manual_text_writer() {
    let source = fs::read_to_string(
        project_root().join("crates/ffone-client/src/ui/character_selection/mod.rs"),
    )
    .unwrap();
    let spawn_start = source.find("fn spawn_character_selection_ui(").unwrap();
    let bind_start = source
        .find("fn bind_character_selection_background_and_slots")
        .unwrap();
    let spawn_source = &source[spawn_start..bind_start];
    let constructors = spawn_source
        .match_indices("Text::new(")
        .filter(|(offset, _)| !spawn_source[..*offset].ends_with("Localized"))
        .collect::<Vec<_>>();
    assert_eq!(constructors.len(), 14);
    for (offset, _) in constructors {
        let tail = &spawn_source[offset..spawn_source.len().min(offset + 900)];
        assert!(
            tail.contains("LocalizedText::new("),
            "Text constructor at source offset {offset} is not key-first"
        );
    }
    assert!(
        !source
            .lines()
            .any(|line| line.contains("text.0 =") && !line.contains("text.0 =="))
    );
    assert!(!source.contains("*text = Text::new"));
}
