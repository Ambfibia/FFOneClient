use crate::localization::*;
use crate::{
    assets::AssetLocator,
    tutorial::{TUTORIAL_CHAPTER_COUNT, TutorialStage},
    tutorial_mission_content::TutorialMissionContent,
};

#[test]
fn production_appearance_nano_and_minimap_keys_resolve_in_both_languages() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, mut language) = Localization::open(&root, "en").unwrap();
    let en = &localization.bundles["en"];
    let ru = &localization.bundles["ru"];
    assert_eq!(en.keys().collect::<Vec<_>>(), ru.keys().collect::<Vec<_>>());
    for (key, value) in en {
        assert_eq!(template_args(value), template_args(&ru[key]), "{key}");
    }
    let appearance: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("data/character_creation/appearance.json")).unwrap(),
    )
    .unwrap();
    for choice in appearance["choices"].as_array().unwrap() {
        let category = choice["category"].as_str().unwrap();
        if !matches!(category, "hair" | "face") {
            continue;
        }
        let key = format!(
            "content.appearance.{}.{category}.{}.name",
            choice["gender"].as_str().unwrap(),
            choice["creationIndex"]
        );
        assert_eq!(en[&key], choice["label"].as_str().unwrap());
        assert!(ru[&key].chars().any(|c| ('А'..='я').contains(&c)), "{key}");
    }
    let tables: serde_json::Value =
        crate::xdt::from_slice(&fs::read(root.join("data/tables/xdt.json")).unwrap())
            .unwrap();
    let nano = &tables["tables"][0]["value"]["m_pNanoTable"];
    let used_tune_rows = nano["m_pNanoData"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["m_iNanoNumber"].as_i64().unwrap() > 0)
        .flat_map(|row| row["m_iTune"].as_array().unwrap().iter())
        .map(|index| index.as_u64().unwrap() as usize)
        .collect::<std::collections::BTreeSet<_>>();
    for (rows, strings, id, reference, domain, fields) in [
        (
            "m_pNanoData",
            "m_pNanoStringData",
            "m_iNanoNumber",
            "m_iNanoName",
            "nano",
            [
                ("name", "m_strName"),
                ("attribute", "m_strComment1"),
                ("description", "m_strComment"),
            ],
        ),
        (
            "m_pNanoTuneData",
            "m_pNanoTuneStringData",
            "m_iTuneNumber",
            "m_iTuneName",
            "nano_tune",
            [
                ("name", "m_strName"),
                ("type_label", "m_strComment1"),
                ("description", "m_strComment"),
            ],
        ),
    ] {
        for (row_index, row) in nano[rows].as_array().unwrap().iter().enumerate() {
            // Unreferenced placeholder tuning rows have unique wire IDs too,
            // but do not create visible powers or localization requirements.
            if domain == "nano_tune" && !used_tune_rows.contains(&row_index) {
                continue;
            }
            if row[id].as_i64().unwrap() <= 0 {
                continue;
            }
            let source = &nano[strings][row[reference].as_u64().unwrap() as usize];
            for (field, column) in fields {
                let fallback = source[column].as_str().unwrap();
                if fallback.is_empty() {
                    continue;
                }
                let key = format!("content.{domain}.{}.{field}", row[id]);
                assert!(en.contains_key(&key), "missing {key}");
                assert!(!ru[&key].is_empty(), "empty {key}");
            }
        }
    }
    let mut app = App::new();
    let texts = [
        LocalizedText::new("content.appearance.male.hair.2.name", "RAZOR CUT"),
        LocalizedText::new("content.nano.1.name", "Buttercup"),
        LocalizedText::new("content.nano_tune.1.name", "FIRE MISS"),
        localized_world_location_text("Sector V"),
    ];
    let expected: Vec<_> = texts
        .iter()
        .map(|text| {
            let english = localization.text(&language, text);
            localization.select(&mut language, "ru");
            let russian = localization.text(&language, text);
            localization.select(&mut language, "en");
            assert_ne!(english, russian, "{}", text.key);
            russian
        })
        .collect();
    app.insert_resource(localization)
        .insert_resource(language)
        .add_systems(Update, apply_localized_texts);
    let entities: Vec<_> = texts
        .into_iter()
        .map(|text| app.world_mut().spawn((Text::default(), text)).id())
        .collect();
    app.update();
    app.world_mut().resource_mut::<Language>().effective = "ru".to_owned();
    app.update();
    for (entity, expected) in entities.into_iter().zip(expected) {
        assert_eq!(app.world().get::<Text>(entity).unwrap().0, expected);
    }
}

fn rust_sources_below(path: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).expect("read Rust source directory") {
        let path = entry.expect("read Rust source entry").path();
        let stem = path.file_stem().and_then(|name| name.to_str()).unwrap_or_default();
        // Separated regression modules are not production writers or binders.
        if stem == "tests" || stem == "test" || stem.ends_with("_tests") {
            continue;
        }
        if path.is_dir() {
            rust_sources_below(&path, output);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            output.push(path);
        }
    }
}

fn production_source_owner(source_root: &Path, path: &Path) -> PathBuf {
    let relative = path.strip_prefix(source_root).expect("source below crate root");
    let mut components = relative.iter();
    let first = components.next().expect("source component");
    let mut owner = source_root.join(first);
    if matches!(
        first.to_str(),
        Some("ui" | "characters" | "rendering" | "world_systems" | "gameplay" | "tutorial_runtime")
    ) {
        if let Some(domain) = components.next() {
            owner.push(domain);
        }
    }
    owner
}

fn raw_text_mutation_count(source: &str) -> usize {
    source
        .match_indices("&mut Text")
        .filter(|(offset, _)| {
            source[*offset + "&mut Text".len()..]
                .chars()
                .next()
                .is_none_or(|next| !next.is_ascii_alphanumeric() && next != '_')
        })
        .count()
}

#[test]
fn production_ui_updates_text_only_through_the_localization_apply_system() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    rust_sources_below(&source_root, &mut sources);
    sources.sort();

    let mut raw_writers = 0;
    let mut owners = std::collections::BTreeMap::<PathBuf, String>::new();
    for path in sources {
        let source = fs::read_to_string(&path).expect("read production Rust source");
        let mutation_count = raw_text_mutation_count(&source);
        if mutation_count != 0 {
            assert!(
                path == source_root.join("localization.rs")
                    || path.starts_with(source_root.join("localization")),
                "production code bypasses LocalizedText in {}", path.display()
            );
            raw_writers += mutation_count;
        }
        let owner = production_source_owner(&source_root, &path);
        let source_group = owners.entry(owner).or_default();
        source_group.push_str(&source);
        source_group.push('\n');
    }
    assert_eq!(raw_writers, 1, "Localization::Apply must remain the sole raw Text writer");
    // A binder and its Plugin/schedule may now be separate files of one owner.
    // Keep the architecture check at that boundary, not at an arbitrary .rs file.
    for (owner, production) in owners {
        if production.contains("&mut LocalizedText") {
            assert!(
                production.contains("LocalizationSet::Apply"),
                "LocalizedText owner has no ordering against Apply: {}", owner.display()
            );
        }
    }
}

#[test]
fn every_recovered_tutorial_scene_line_has_distinct_russian_copy() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&asset_root).expect("open production asset graph");
    let content = TutorialMissionContent::open(&locator).expect("open tutorial TableData");
    let (localization, en) = Localization::open(&asset_root, "en").unwrap();
    let (_, ru) = Localization::open(&asset_root, "ru").unwrap();

    for (event, lines) in [(1, 1..=10), (2, 1..=91)] {
        for line in lines {
            let fallback = content.scene_text(event, line).unwrap();
            let localized = localized_tutorial_scene_text(event, line, fallback);
            let english = localization.text(&en, &localized);
            let russian = localization.text(&ru, &localized);
            assert_ne!(russian, english, "tutorial scene {event}:{line}");
            assert!(
                russian
                    .chars()
                    .any(|character| ('А'..='я').contains(&character)),
                "tutorial scene {event}:{line} has no Cyrillic copy: {russian:?}"
            );
        }
    }
}

#[test]
fn every_tabledata_mission_field_has_a_semantic_en_ru_entry() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&asset_root).expect("open production asset graph");
    let content = TutorialMissionContent::open(&locator).expect("open production TableData");
    let (localization, en) = Localization::open(&asset_root, "en").unwrap();
    let (_, ru) = Localization::open(&asset_root, "ru").unwrap();

    for mission in content.missions() {
        let task_id = mission.provenance.task_id;
        for (field, fallback) in [
            ("title", mission.title.as_str()),
            ("objective", mission.objective.as_str()),
            (
                "offer_description",
                mission.journal.offer_description.as_str(),
            ),
            (
                "task_description",
                mission.journal.active_task_description.as_str(),
            ),
            ("mission_summary", mission.journal.mission_summary.as_str()),
            (
                "mission_complete_summary",
                mission.journal.mission_complete_summary.as_str(),
            ),
            (
                "completion_description",
                mission.journal.completion_description.as_str(),
            ),
        ] {
            let localized = localized_tutorial_mission_text(task_id, field, fallback);
            assert_eq!(
                localized.key,
                format!("content.mission.task.{task_id}.{field}")
            );
            assert_eq!(localization.text(&en, &localized), fallback);
            assert!(
                !localization.text(&ru, &localized).is_empty(),
                "mission task {task_id} field {field} has empty RU copy"
            );
        }
    }

    let ordinary = content.mission(2).expect("ordinary Dee Dee mission");
    let title = localized_tutorial_mission_text(2, "title", &ordinary.title);
    assert_ne!(
        localization.text(&ru, &title),
        localization.text(&en, &title)
    );
    assert!(
        localization
            .text(&ru, &title)
            .chars()
            .any(|character| ('А'..='я').contains(&character) || character == 'ё')
    );
}

#[test]
fn every_known_source_literal_has_a_semantic_key() {
    for source in [
        "USERNAME :",
        "SELECT A CHARACTER",
        "CHARACTER CREATION",
        "Press ENTER to access chat and menus.",
        "ALL",
        "UNLIMITED ACCESS ONLY",
        "Disconnected from OpenFusion",
    ] {
        assert!(semantic_key_for_source(source).is_some(), "{source}");
    }
}

#[test]
fn every_tutorial_stage_instruction_has_a_semantic_key() {
    let mut recovered_sources = BTreeSet::new();
    for chapter in 0..TUTORIAL_CHAPTER_COUNT {
        for step in i16::MIN..=i16::MAX {
            let Some(source) = TutorialStage::from_legacy(chapter, step)
                .and_then(|stage| stage.metadata().instruction)
            else {
                continue;
            };
            recovered_sources.insert(source);
            assert!(
                tutorial_instruction_key(source).is_some(),
                "tutorial chapter {chapter}, step {step} has no semantic localization key: \
                 {source:?}"
            );
        }
    }
    for (source, key) in TUTORIAL_INSTRUCTION_KEYS {
        assert!(
            recovered_sources.contains(source),
            "unused tutorial localization mapping {key:?} for {source:?}"
        );
    }
}

#[test]
fn every_recovered_tutorial_literal_has_a_semantic_key() {
    assert_eq!(
        localized_tutorial_literal(
            "I need some help with a sooper important mission! Report to me right away!"
        )
        .key,
        "content.tutorial.message.minimap_numbuh_two"
    );
    assert_eq!(
        localized_tutorial_literal("Use the warp gate to exit.").key,
        "tutorial.instruction.use_exit_warp_gate"
    );

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, russian) =
        Localization::open(&asset_root, "ru").expect("open production localization");
    let source = "I need some help with a sooper important mission! Report to me right away!";
    assert_ne!(
        localization.text(&russian, &localized_tutorial_literal(source)),
        source
    );
}

/// `source` without `use` declarations, so audits count code rather than import lists.
fn without_use_declarations(source: &str) -> String {
    let mut kept = String::with_capacity(source.len());
    let mut in_use = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        let starts_use = trimmed.starts_with("use ")
            || trimmed.starts_with("pub use ")
            || (trimmed.starts_with("pub(") && trimmed.contains(") use "));
        if in_use || starts_use {
            in_use = !line.trim_end().ends_with(';');
            continue;
        }
        kept.push_str(line);
        kept.push('\n');
    }
    kept
}

// `app/mod.rs` is split into child modules; audit it together with every file that
// holds code moved out of it (pre-existing `app` children were never part of this source).
const SPLIT_APP_SOURCE: &str = concat!(
    concat!(
        include_str!("../app/assets_register_dexter_hologram_asset.rs"),
        "\n",
        include_str!("../app/models.rs"),
        "\n",
        include_str!("../app/constants.rs"),
        "\n",
        include_str!("../app/animation.rs"),
        "\n",
        include_str!("../app/operations.rs"),
        "\n",
        include_str!("../app/materials.rs"),
        "\n",
        include_str!("../app/types.rs"),
        "\n",
        include_str!("../app/systems.rs"),
        "\n",
        include_str!("../app/projects.rs"),
        "\n",
        include_str!("../app/state_client_state_uses_world_nano_aut.rs"),
        "\n",
        include_str!("../app/audio_consume_gameplay_ui_audio_outbox.rs"),
        "\n",
        include_str!("../app/commands.rs"),
        "\n",
        include_str!("../app/mod.rs")
    ),
    include_str!("../app/loading_screen.rs"),
    include_str!("../app/login.rs"),
    include_str!("../app/network_smoke.rs"),
    include_str!("../app/asset_residency.rs"),
    include_str!("../app/tutorial_session.rs"),
    include_str!("../app/mission_indicators.rs"),
    include_str!("../app/world_mission.rs"),
    include_str!("../app/group_pc2pc.rs"),
    include_str!("../app/guide.rs"),
    include_str!("../app/npc_warp.rs"),
    include_str!("../app/transportation.rs"),
    include_str!("../app/email_catalog.rs"),
    include_str!("../app/email_production.rs"),
    include_str!("../app/email_outputs.rs"),
    concat!(
        include_str!("../app/combi/constants.rs"),
        "\n",
        include_str!("../app/combi/audio_combi_ui_mode_sound_true_name.rs"),
        "\n",
        include_str!("../app/combi/animation.rs"),
        "\n",
        include_str!("../app/combi/containers.rs"),
        "\n",
        include_str!("../app/combi/types.rs"),
        "\n",
        include_str!("../app/combi/operations.rs"),
        "\n",
        include_str!("../app/combi/state.rs"),
        "\n",
        include_str!("../app/combi/codec.rs"),
        "\n",
        include_str!("../app/combi/projects.rs"),
        "\n",
        include_str!("../app/combi/entities.rs"),
        "\n",
        include_str!("../app/combi/systems.rs"),
        "\n",
        include_str!("../app/combi.rs")
    ),
    include_str!("../app/enchant.rs"),
    concat!(
        include_str!("../app/race/codec.rs"),
        "\n",
        include_str!("../app/race/constants.rs"),
        "\n",
        include_str!("../app/race/audio_pending_race_npc_voice.rs"),
        "\n",
        include_str!("../app/race/types.rs"),
        "\n",
        include_str!("../app/race/input.rs"),
        "\n",
        include_str!("../app/race/projects.rs"),
        "\n",
        include_str!("../app/race/state.rs"),
        "\n",
        include_str!("../app/race/assets.rs"),
        "\n",
        include_str!("../app/race/operations.rs"),
        "\n",
        include_str!("../app/race/commands.rs"),
        "\n",
        include_str!("../app/race/systems.rs"),
        "\n",
        include_str!("../app/race/entities.rs"),
        "\n",
        include_str!("../app/race.rs")
    ),
    include_str!("../app/bank.rs"),
    include_str!("../app/vendor.rs"),
    include_str!("../app/user_equip.rs"),
    include_str!("../app/local_inventory.rs"),
    include_str!("../app/option_runtime.rs"),
    include_str!("../app/launcher.rs"),
    include_str!("../app/runtime_status.rs"),
    include_str!("../app/world_combat.rs"),
    include_str!("../app/world_nano.rs"),
    include_str!("../app/nano_free_tuning.rs"),
    include_str!("../app/nano_free_tuning_production.rs"),
    include_str!("../app/world_map.rs"),
    include_str!("../app/dexter_ship_scene.rs"),
    include_str!("../app/dexter_ship_drive.rs"),
    include_str!("../app/sky.rs"),
    include_str!("../app/tutorial_ambience.rs"),
    include_str!("../app/tutorial_startup.rs"),
    include_str!("../app/world_scene.rs"),
    include_str!("../app/character_loading.rs"),
    include_str!("../app/quit_menu.rs"),
    include_str!("../app/cashmall.rs"),
    include_str!("../app/resurrect.rs"),
    include_str!("../app/modal_gates.rs"),
    include_str!("../app/input_gates.rs"),
    include_str!("../app/local_avatar.rs"),
    include_str!("../app/world_npc_interaction.rs"),
    include_str!("../app/tutorial_combat.rs"),
    include_str!("../app/tutorial_mission_flow.rs"),
    include_str!("../app/tutorial_indicators.rs"),
    include_str!("../app/tutorial_scene_audio.rs"),
    include_str!("../app/chat_commands.rs"),
    include_str!("../app/quick_slots.rs"),
    include_str!("../app/rule.rs"),
    include_str!("../app/world_intents.rs"),
    include_str!("../app/skyway_traversal_motion_tests.rs"),
);

#[test]
fn production_tutorial_chat_is_key_first() {
    let main_source = SPLIT_APP_SOURCE;
    for forbidden in [
        "ChatLineUi::normal(\"",
        "ChatLineUi::normal(format!",
        "ChatLineUi::tutorial(\"",
        "stage.metadata().instruction.map(str::to_uppercase)",
    ] {
        assert!(
            !main_source.contains(forbidden),
            "production tutorial UI bypasses localization via {forbidden:?}"
        );
    }
}

#[test]
fn native_ui_text_follows_the_language_resource() {
    let localization = Localization {
        fallback: "en".to_owned(),
        bundles: BTreeMap::from([
            (
                "en".to_owned(),
                BTreeMap::from([("ui.test".to_owned(), "English".to_owned())]),
            ),
            (
                "ru".to_owned(),
                BTreeMap::from([("ui.test".to_owned(), "Русский".to_owned())]),
            ),
        ]),
        fallback_keys_by_source: BTreeMap::from([("English".to_owned(), "ui.test".to_owned())]),
    };
    let mut app = App::new();
    app.insert_resource(localization)
        .insert_resource(Language {
            requested: "en".to_owned(),
            effective: "en".to_owned(),
        })
        .add_systems(Update, apply_localized_texts);

    let text = app
        .world_mut()
        .spawn((
            Text::new("English"),
            LocalizedText::new("ui.test", "English"),
        ))
        .id();
    app.update();
    app.world_mut().resource_mut::<Language>().effective = "ru".to_owned();
    app.update();

    assert_eq!(app.world().entity(text).get::<Text>().unwrap().0, "Русский");
}

#[test]
fn localized_uppercase_is_applied_after_language_resolution() {
    let localization = Localization {
        fallback: "en".to_owned(),
        bundles: BTreeMap::from([(
            "en".to_owned(),
            BTreeMap::from([("ui.test".to_owned(), "Computress".to_owned())]),
        )]),
        fallback_keys_by_source: BTreeMap::new(),
    };
    let mut app = App::new();
    app.insert_resource(localization)
        .insert_resource(Language {
            requested: "en".to_owned(),
            effective: "en".to_owned(),
        })
        .add_systems(Update, apply_localized_texts);

    let text = app
        .world_mut()
        .spawn((
            Text::new(""),
            LocalizedText::new("ui.test", "Computress"),
            LocalizedTextCase::Uppercase,
        ))
        .id();
    app.update();

    assert_eq!(
        app.world().entity(text).get::<Text>().unwrap().0,
        "COMPUTRESS"
    );
}

#[test]
fn dynamic_status_keeps_opaque_values_as_arguments() {
    assert_eq!(
        localized_status("Connecting to 127.0.0.1:23000..."),
        LocalizedText::new("status.connecting", "Connecting to {server}...")
            .with_arg("server", "127.0.0.1:23000")
    );
    assert_eq!(
        localized_status("Network error: socket closed"),
        LocalizedText::new("status.login.network_error", "Network error: {error}")
            .with_arg("error", "socket closed")
    );
    assert_eq!(
        localized_status("Server-authored message"),
        LocalizedText::new("status.message", "{message}")
            .with_arg("message", "Server-authored message")
    );
}

#[test]
fn russian_bundle_covers_the_canonical_bundle_with_matching_placeholders() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
    let load = |locale: &str| {
        let bytes = fs::read(root.join(format!("{locale}.json"))).unwrap();
        serde_json::from_slice::<TextBundle>(&bytes).unwrap()
    };
    let en = load("en");
    let ru = load("ru");
    assert_eq!(
        en.entries.keys().collect::<Vec<_>>(),
        ru.entries.keys().collect::<Vec<_>>()
    );
    for (key, source) in &en.entries {
        let translated = ru
            .entries
            .get(key)
            .unwrap_or_else(|| panic!("missing RU {key}"));
        assert_eq!(
            template_args(source),
            template_args(translated),
            "{key} has mismatched placeholders"
        );
    }
    Localization::open(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        "ru",
    )
    .expect("production EN/RU bundles satisfy the runtime contract");
}

#[test]
fn npc_greeting_and_barker_fields_resolve_in_both_production_locales() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&asset_root).expect("open production asset graph");
    let content = TutorialMissionContent::open(&locator).expect("open production TableData");
    let npc = content.gameplay_npc(643).expect("Edd guide shop NPC");
    let barker = npc.barker.as_ref().expect("Edd guide shop barker row");
    let (localization, en) = Localization::open(&asset_root, "en").unwrap();
    let (_, ru) = Localization::open(&asset_root, "ru").unwrap();

    let greeting = localized_tabledata_npc_greeting(npc.greeting_string_id, &npc.greeting);
    assert_eq!(localization.text(&en, &greeting), npc.greeting);
    assert!(!localization.text(&ru, &greeting).trim().is_empty());
    for (index, fallback) in barker.lines.iter().enumerate() {
        let localized = localized_tabledata_npc_barker(barker.string_id, index, fallback);
        assert_eq!(localization.text(&en, &localized), *fallback);
        assert!(!localization.text(&ru, &localized).trim().is_empty());
    }
}

#[test]
fn switching_to_english_silences_a_russian_only_voice_and_can_restore_it() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("data/tables")).unwrap();
    fs::create_dir_all(dir.path().join("audio/voice/ru/test")).unwrap();
    fs::write(
        dir.path().join("audio/voice/ru/test/line.ogg"),
        b"OggSfixture",
    )
    .unwrap();
    fs::write(dir.path().join("data/tables/xdt.json"), serde_json::to_vec(&serde_json::json!({
        "schema":"ffone.table-set.v1","tables":[{"name":"native_asset_routes","value":{
            "m_pAudioData":[{"logicalKey":"voice/test/line","trueName":"Test_Line","category":"voice","owner":"test","path":"test/line.ogg"}],
            "m_pCharacterModelData":[]
        }}]
    })).unwrap()).unwrap();
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: dir.path().to_string_lossy().into_owned(),
            ..default()
        },
    ))
    .init_asset::<AudioSource>()
    .insert_resource(NativeAudioCatalog::open(dir.path(), false).unwrap())
    .insert_resource(VoiceLanguage {
        requested: "ru".into(),
        effective: "ru".into(),
    })
    .add_systems(Update, refresh_localized_voice_players);
    let entity = app
        .world_mut()
        .spawn(LocalizedVoice::by_true_name("Test_Line"))
        .id();
    app.update();
    assert!(app.world().get::<AudioPlayer>(entity).is_some());
    app.world_mut().resource_mut::<VoiceLanguage>().effective = "en".into();
    app.update();
    assert!(app.world().get::<AudioPlayer>(entity).is_none());
    app.world_mut().resource_mut::<VoiceLanguage>().effective = "ru".into();
    app.update();
    assert!(app.world().get::<AudioPlayer>(entity).is_some());
}

#[test]
fn localized_voice_resolves_both_locales_from_the_semantic_catalog() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let voice = LocalizedVoice::by_true_name("Albedo_Farewell01");
    assert_eq!(
        localized_voice_path(&catalog, "en", &voice)
            .unwrap()
            .unwrap(),
        "audio/voice/en/albedo/farewell01.ogg"
    );
    assert_eq!(
        localized_voice_path(&catalog, "ru", &voice)
            .unwrap()
            .unwrap(),
        "audio/voice/ru/albedo/farewell01.ogg"
    );
}

#[test]
fn production_voice_players_are_semantic_and_never_build_locale_paths() {
    let main_source = SPLIT_APP_SOURCE;
    assert_eq!(
        // Import lists of the split modules name the component too; count spawn sites.
        without_use_declarations(main_source)
            .matches("TutorialVoiceAudio,")
            .count(),
        2,
        "every TutorialVoiceAudio spawn must remain auditable"
    );
    assert!(
        main_source.matches("LocalizedVoice::by_true_name").count() >= 3,
        "tutorial and Dexter voice players must carry semantic locale identity"
    );
    for forbidden in ["audio/voice/en/", "audio/voice/ru/"] {
        assert!(
            !main_source.contains(forbidden),
            "runtime voice code hardcodes locale path {forbidden:?}"
        );
    }
}
#[test]
fn missing_translation_falls_back_to_english() {
    let localization = Localization {
        fallback: "en".to_owned(),
        bundles: BTreeMap::from([
            (
                "en".to_owned(),
                BTreeMap::from([("test.key".to_owned(), "English {value}".to_owned())]),
            ),
            ("ru".to_owned(), BTreeMap::new()),
        ]),
        fallback_keys_by_source: BTreeMap::new(),
    };
    let language = Language {
        requested: "ru".to_owned(),
        effective: "ru".to_owned(),
    };
    assert_eq!(
        localization.text(
            &language,
            &LocalizedText::new("test.key", "fallback").with_arg("value", "42")
        ),
        "English 42"
    );
}

#[test]
fn localization_paths_reject_raw_dot_and_empty_segments() {
    for path in [
        "a//b.json",
        "a/./b.json",
        "a/../b.json",
        "/a.json",
        "a\\b.json",
    ] {
        assert!(native_path(path).is_err(), "{path:?} must be rejected");
    }
    assert_eq!(
        native_path("localization/en.json").unwrap(),
        PathBuf::from("localization").join("en.json")
    );
}

#[test]
fn arbitrary_catalog_locales_normalize_and_use_language_fallbacks() {
    let bundles = BTreeMap::from([
        ("de".to_owned(), BTreeMap::new()),
        ("en".to_owned(), BTreeMap::new()),
        ("pt-br".to_owned(), BTreeMap::new()),
    ]);
    assert_eq!(normalize_language("PT_BR"), "pt-br");
    assert_eq!(resolve_locale(&bundles, "de-de", "en"), "de");
    assert_eq!(resolve_locale(&bundles, "pt-br", "en"), "pt-br");
    assert_eq!(resolve_locale(&bundles, "fr", "en"), "en");
    for locale in ["de", "pt-br", "zh-hans"] {
        validate_locale(locale).unwrap();
    }
}

mod text_fit;
