use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use bevy::{asset::AssetPlugin, input::keyboard::KeyboardInput, prelude::*};
use serde::Deserialize;

use super::{
    localization::LocalizedText,
    login_ui::{LoginTextStyle0104, NativeLoginUiPlugin},
    option_ui::OptionUiModel,
};

#[derive(Deserialize)]
struct TextBundle {
    entries: BTreeMap<String, String>,
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

#[test]
fn production_bundles_match_and_cover_every_spawned_login_text() {
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

    let asset_root = root.join("assets/game");
    let mut app = App::new();
    app.insert_resource(OptionUiModel::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins(NativeLoginUiPlugin);
    app.update();

    let localized = {
        let world = app.world_mut();
        let mut query = world.query::<(&LocalizedText, &LoginTextStyle0104)>();
        query
            .iter(world)
            .map(|(localized, _)| localized.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(localized.len(), 8);
    for localized in localized {
        assert!(
            en.contains_key(&localized.key),
            "missing EN {}",
            localized.key
        );
        assert!(
            ru.contains_key(&localized.key),
            "missing RU {}",
            localized.key
        );
        assert_eq!(
            localized.args.keys().cloned().collect::<BTreeSet<_>>(),
            placeholders(&en[&localized.key]),
            "runtime args for {}",
            localized.key
        );
    }
}

#[test]
fn exact_clean_en_and_ru_login_values_are_stable() {
    let root = project_root();
    let en = bundle(&root.join("assets/game/localization/en.json")).entries;
    let ru = bundle(&root.join("assets/game/localization/ru.json")).entries;
    for (key, en_value, ru_value) in [
        ("ui.login.username", "Username :", "ЛОГИН :"),
        ("ui.login.password", "Password :", "ПАРОЛЬ :"),
        ("ui.login.submit", "Log In", "ВОЙТИ"),
        (
            "ui.login.discord",
            "Discord Community",
            "СООБЩЕСТВО DISCORD",
        ),
        (
            "ui.login.register",
            "How Do I Register?",
            "КАК ЗАРЕГИСТРИРОВАТЬСЯ?",
        ),
        (
            "ui.login.registration_instructions",
            "To Register:\nChoose an Username and Password into the appropriate boxes and press Log In. Make sure to remember these as there is no account recovery!",
            "Регистрация:\nВведите логин и пароль в соответствующие поля и нажмите «ВОЙТИ». Запомните их: восстановление учётной записи недоступно!",
        ),
    ] {
        assert_eq!(en[key], en_value, "{key}");
        assert_eq!(ru[key], ru_value, "{key}");
    }
}
