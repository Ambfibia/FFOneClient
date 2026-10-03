use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use bevy::{asset::AssetPlugin, prelude::*, window::PrimaryWindow};
use image::GenericImageView;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{
    localization::{Localization, LocalizationPlugin, LocalizedText},
    nanocom_message_ui::{
        NANOCOM_ACCEPT_LOCALIZATION_KEY, NANOCOM_COMPACT_BODY_LOCALIZATION_KEY,
        NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY, NANOCOM_DECLINE_LOCALIZATION_KEY,
        NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY, NANOCOM_EXPIRATION_LOCALIZATION_KEY,
        NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY,
        NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY,
        NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY,
        NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY, NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY,
        NANOCOM_INVITATION_LOCALIZATION_KEY, NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY,
        NANOCOM_REACHED_TEXTURES, NANOCOM_SOURCE_ICONS_ARCHIVE_SHA256,
        NANOCOM_SOURCE_MAIN_ARCHIVE_SHA256, NanocomGuiStyleRole, NanocomMessageUiModel,
        NanocomMessageUiPlugin,
    },
};

#[derive(Deserialize)]
struct TextBundle {
    schema: String,
    locale: String,
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
        .add_plugins((NanocomMessageUiPlugin, LocalizationPlugin));
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.update();
    app
}

#[test]
fn production_bundles_have_exact_key_and_placeholder_parity() {
    let en = bundle(&asset_root().join("localization/en.json"));
    let ru = bundle(&asset_root().join("localization/ru.json"));
    assert_eq!(en.schema, "ffone.text-bundle.v1");
    assert_eq!(ru.schema, "ffone.text-bundle.v1");
    assert_eq!(en.locale, "en");
    assert_eq!(ru.locale, "ru");
    assert_eq!(
        en.entries.keys().collect::<Vec<_>>(),
        ru.entries.keys().collect::<Vec<_>>()
    );

    for key in [
        NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY,
        NANOCOM_COMPACT_BODY_LOCALIZATION_KEY,
        NANOCOM_INVITATION_LOCALIZATION_KEY,
        NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY,
        NANOCOM_EXPIRATION_LOCALIZATION_KEY,
        NANOCOM_ACCEPT_LOCALIZATION_KEY,
        NANOCOM_DECLINE_LOCALIZATION_KEY,
        NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY,
        NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY,
        NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY,
        NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY,
        NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY,
        NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY,
    ] {
        let en_value = en
            .entries
            .get(key)
            .unwrap_or_else(|| panic!("missing EN {key}"));
        let ru_value = ru
            .entries
            .get(key)
            .unwrap_or_else(|| panic!("missing RU {key}"));
        assert_eq!(placeholders(en_value), placeholders(ru_value), "{key}");
    }
}

#[test]
fn every_spawned_text_is_key_first_and_has_one_typed_clean_style() {
    let mut app = localized_app("en");
    {
        let mut model = app.world_mut().resource_mut::<NanocomMessageUiModel>();
        model.enqueue_buddy_invite(13, "Dexter");
        model.set_expanded(true);
        while model.pop_sound().is_some() {}
    }
    app.update();

    let world = app.world_mut();
    let mut texts =
        world.query::<(&Text, Option<&LocalizedText>, Option<&NanocomGuiStyleRole>)>();
    let rows = texts.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), 7);
    assert!(rows.iter().all(|(_, localized, style)| {
        localized.is_some_and(|copy| !copy.key.is_empty()) && style.is_some()
    }));
}

#[test]
fn production_ru_bundle_reaches_all_buddy_modal_text_entities() {
    let mut app = localized_app("ru");
    {
        let mut model = app.world_mut().resource_mut::<NanocomMessageUiModel>();
        model.enqueue_buddy_invite(13, "Dexter");
        model.set_expanded(true);
        while model.pop_sound().is_some() {}
    }
    app.update();

    let world = app.world_mut();
    let mut texts = world.query::<(&Text, &LocalizedText)>();
    let rows = texts
        .iter(world)
        .map(|(text, localized)| (text.0.clone(), localized.key.clone()))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 7);
    assert!(rows.iter().all(|(text, _)| !text.is_empty()));
    assert!(rows.iter().any(|(text, _)| {
        text.chars()
            .any(|ch| ('\u{0400}'..='\u{04ff}').contains(&ch))
    }));
}

#[test]
fn production_bundles_render_keyed_computress_login_notice() {
    for locale in ["en", "ru"] {
        let mut app = localized_app(locale);
        {
            let mut model = app.world_mut().resource_mut::<NanocomMessageUiModel>();
            model.enqueue_type_9_localized(
                LocalizedText::new("content.npc.730.name", "Computress"),
                LocalizedText::new(
                    "content.tabledata.guide.guide_string.19.sz_string",
                    "Welcome back. Please check your email to learn about an important mission from me.",
                ),
                "ui/en/gameplay/guide/compu_icon.png",
                Some("Computress"),
            );
            while model.pop_sound().is_some() {}
        }
        app.update();
        let world = app.world_mut();
        let mut texts = world.query::<(&Text, &LocalizedText)>();
        let rows = texts
            .iter(world)
            .map(|(text, localized)| (text.0.clone(), localized.key.clone()))
            .collect::<Vec<_>>();
        for key in [
            "content.npc.730.name",
            "content.tabledata.guide.guide_string.19.sz_string",
        ] {
            assert!(
                rows.iter()
                    .any(|(text, row_key)| row_key == key && !text.is_empty()),
                "{locale} did not render {key}"
            );
        }
        if locale == "ru" {
            assert!(rows.iter().any(|(text, _)| {
                text.chars()
                    .any(|ch| ('\u{0400}'..='\u{04ff}').contains(&ch))
            }));
        }
    }
}

#[test]
fn production_localization_renders_reached_group_compact_and_modal_copy() {
    for locale in ["en", "ru"] {
        let mut app = localized_app(locale);
        {
            let mut model = app.world_mut().resource_mut::<NanocomMessageUiModel>();
            model.enqueue(
                super::nanocom_message_ui::NanocomMessageRequest::group_invite(
                    14,
                    "Remote Player",
                ),
            );
            model.set_expanded(true);
            while model.pop_sound().is_some() {}
        }
        app.update();
        let world = app.world_mut();
        let mut texts = world.query::<(&Text, &LocalizedText)>();
        let rows = texts
            .iter(world)
            .map(|(text, localized)| (text.0.clone(), localized.key.clone()))
            .collect::<Vec<_>>();
        for key in [
            NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY,
            NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY,
            NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY,
            NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY,
            NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY,
        ] {
            assert!(
                rows.iter()
                    .any(|(text, row_key)| row_key == key && !text.is_empty())
            );
        }
        if locale == "ru" {
            assert!(rows.iter().any(|(text, _)| {
                text.chars()
                    .any(|ch| ('\u{0400}'..='\u{04ff}').contains(&ch))
            }));
        }
    }
}

#[test]
fn reached_runtime_textures_match_published_bytes_dimensions_and_hashes() {
    for evidence in NANOCOM_REACHED_TEXTURES {
        let path = asset_root().join(evidence.runtime_path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        assert_eq!(
            bytes.len() as u64,
            evidence.runtime_bytes,
            "{}",
            path.display()
        );
        let digest = format!("{:X}", Sha256::digest(&bytes));
        assert_eq!(digest, evidence.runtime_sha256, "{}", path.display());
        let image = image::load_from_memory(&bytes).unwrap();
        assert_eq!(
            image.dimensions(),
            (evidence.width, evidence.height),
            "{}",
            path.display()
        );
    }
}

#[test]
fn checked_primary_archive_identity_is_not_a_runtime_path() {
    assert_eq!(
        NANOCOM_SOURCE_MAIN_ARCHIVE_SHA256,
        "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
    );
    assert_eq!(
        NANOCOM_SOURCE_ICONS_ARCHIVE_SHA256,
        "A05602D6E96E2E74ECAD207F42E519605259434210DE8DA19E422B30D642E544"
    );
    let source = concat!(
        include_str!("../../src/ui/nanocom_message/constants.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/containers.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/assets.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/layout.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/frame.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/interaction.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/localization_nanocom_content_localization_key.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/commands.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/types.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/operations.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/textures.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/audio_resolve_nanocom_voice.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/models.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/view.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/systems.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/mod.rs")
    );
    assert!(!source.contains("retrobution-20260613.ffclient\\cache"));
    assert!(!source.contains("builds/retrobution-20260613"));
}

#[test]
fn source_has_six_constructor_sites_for_seven_key_first_text_entities() {
    let source = concat!(
        include_str!("../../src/ui/nanocom_message/constants.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/containers.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/assets.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/layout.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/frame.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/interaction.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/localization_nanocom_content_localization_key.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/commands.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/types.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/operations.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/textures.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/audio_resolve_nanocom_voice.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/models.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/view.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/systems.rs"),
        "\n",
        include_str!("../../src/ui/nanocom_message/mod.rs")
    );
    assert_eq!(
        source
            .lines()
            .filter(|line| line.contains("Text::new(") && !line.contains("LocalizedText::new("))
            .count(),
        6
    );
    assert!(!source.contains("TextBundle"));
    assert!(source.contains(".before(LocalizationSet::Apply)"));
}
