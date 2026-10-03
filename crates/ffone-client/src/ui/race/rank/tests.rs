use std::{collections::BTreeSet, fs, path::Path};

use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::race_ui::rank::*;

fn open_model() -> (RaceRankCatalog, RaceRankModel) {
    let catalog = RaceRankCatalog::embedded().unwrap();
    let mut model = RaceRankModel::default();
    model
        .open(
            &catalog,
            RaceRankOpenContext {
                pcuid: 42,
                current_ep_id: 15,
                npc_name: "Dexter".to_owned(),
                npc_target_instance_id: 1_234,
                rank_url: RACE_RANK_RETROBUTION_URL.to_owned(),
            },
        )
        .unwrap();
    (catalog, model)
}

fn response_with_top_count(count: usize) -> String {
    fn tag(pcuid: i32, score: i32, rank: i32) -> String {
        format!(
            "<score PCUID=\"{pcuid}\" Score=\"{score}\" Rank=\"{rank}\" FirstName=\"Player\" LastName=\"{pcuid}\"/>"
        )
    }
    let mut body = "SUCCESS".to_owned();
    for (period, personal) in [
        ("day", "myday"),
        ("week", "myweek"),
        ("month", "mymonth"),
        ("alltime", "myalltime"),
    ] {
        body.push_str(&format!("<{personal}>{}</{personal}>", tag(42, 900, 2)));
        body.push_str(&format!("<{period}>"));
        for index in 0..count {
            body.push_str(&tag(
                if index == 1 { 42 } else { 100 + index as i32 },
                1_000 - index as i32,
                index as i32 + 1,
            ));
        }
        body.push_str(&format!("</{period}>"));
    }
    body
}

fn placeholders(template: &str) -> BTreeSet<&str> {
    let mut values = BTreeSet::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else {
            break;
        };
        values.insert(&rest[..close]);
        rest = &rest[close + 1..];
    }
    values
}

#[test]
fn embedded_table_catalog_has_exact_clean_order_and_copy() {
    let catalog = RaceRankCatalog::embedded().unwrap();
    assert_eq!(catalog.locations().len(), 32);
    assert_eq!(catalog.locations()[0].ep_id, 1);
    assert_eq!(catalog.locations()[0].name, "KND Training Area");
    assert_eq!(catalog.locations()[0].area_name, "The Future");
    assert!(
        !catalog
            .locations()
            .iter()
            .any(|location| location.ep_id == 6)
    );
    assert_eq!(catalog.locations().last().unwrap().ep_id, 33);
    assert_eq!(catalog.source().instance_table_path_id, 7);
    assert_eq!(catalog.source().world_name_path_id, 8);
    assert_eq!(
        catalog.source().area_copy,
        "WorldNameData first coordinate match, ZoneName"
    );
}

#[test]
fn open_selects_current_ep_and_emits_exact_ordered_http_form() {
    let (catalog, mut model) = open_model();
    let (index, location) = catalog.location_by_ep(15).unwrap();
    assert_eq!(model.current_page(), index / 5);
    assert_eq!(model.selected_index(), Some(index));
    assert_eq!(model.phase(), RaceRankPhase::Loading);
    assert_eq!(
        model.pop_output(),
        Some(RaceRankOutput::Effect(RaceRankEffect::BindNpcCamera {
            target_instance_id: 1_234,
        }))
    );
    let RaceRankOutput::Http(intent) = model.pop_output().unwrap() else {
        panic!("expected HTTP intent");
    };
    assert_eq!(intent.request_id, 1);
    assert_eq!(intent.url, RACE_RANK_RETROBUTION_URL);
    assert_eq!(intent.ep_id, location.ep_id);
    assert_eq!(
        intent.ordered_form_fields(),
        [
            ("PCUID", "42".to_owned()),
            ("EP_ID", location.ep_id.to_string()),
        ]
    );
    assert!(RACE_RANK_PACKETS_ARE_DORMANT);
    assert_eq!(
        RACE_RANK_GET_PC_INFO_REQUEST_ABI.map(|field| field.offset),
        [0, 4, 22]
    );
    assert_eq!(
        RACE_RANK_GET_PC_INFO_REQUEST_ABI.map(|field| field.byte_width),
        [4, 18, 34]
    );
    assert_eq!(RACE_RANK_GET_PC_INFO_REQUEST_SIZE, 56);
}

#[test]
fn success_parser_requires_clean_attribute_order_and_caps_top_ten() {
    let scores = parse_clean_rank_response(&response_with_top_count(12)).unwrap();
    for period in 0..4 {
        assert_eq!(
            scores.personal[period].as_ref().unwrap().player,
            "Player 42"
        );
        assert_eq!(
            scores.top[period]
                .iter()
                .filter(|score| score.is_some())
                .count(),
            10
        );
        assert_eq!(scores.top[period][1].as_ref().unwrap().pcuid, 42);
    }
    let malformed = response_with_top_count(1).replacen(
        "PCUID=\"42\" Score=\"900\"",
        "Score=\"900\" PCUID=\"42\"",
        1,
    );
    assert!(matches!(
        parse_clean_rank_response(&malformed),
        Err(RaceRankParseError::WrongAttributeOrder { .. })
            | Err(RaceRankParseError::MissingAttribute(_))
    ));
}

#[test]
fn malformed_completion_clears_scores_and_fails_closed() {
    let (_catalog, mut model) = open_model();
    model.clear_outputs();
    let error = model.complete_fetch(1, "SUCCESS<myday>").unwrap_err();
    assert!(matches!(error, RaceRankCompletionError::Parse(_)));
    assert_eq!(model.phase(), RaceRankPhase::Browsing);
    assert_eq!(*model.scores(), RaceRankScores::default());
    assert!(model.last_parse_error().is_some());
}

#[test]
fn page_buttons_preserve_inclusive_divisible_page_bug_and_head_only_clear() {
    let (_catalog, mut model) = open_model();
    model
        .complete_fetch(1, &response_with_top_count(3))
        .unwrap();
    assert!(model.scores().top[0][1].is_some());
    model.next_page(10).unwrap();
    assert_eq!(model.current_page(), 2);
    assert_eq!(model.selected_index(), None);
    assert!(model.scores().personal[0].is_none());
    assert!(model.scores().top[0][0].is_none());
    assert!(model.scores().top[0][1].is_some());
    assert_eq!(model.page_copy(10), "11 - 10 of 10 Locations");
}

#[test]
fn tab_and_page_copy_match_clean_strings() {
    assert_eq!(RaceRankPeriod::Today.best_copy(), "MY BEST SCORE TODAY:");
    assert_eq!(RaceRankPeriod::Today.top_copy(), " TODAY'S TOP 10:");
    assert_eq!(RaceRankPeriod::Week.best_copy(), "MY BEST SCORE THIS WEEK:");
    assert_eq!(RaceRankPeriod::Week.top_copy(), " THIS WEEK'S TOP 10:");
    assert_eq!(
        RaceRankPeriod::Month.best_copy(),
        "MY BEST SCORE THIS MONTH:"
    );
    assert_eq!(RaceRankPeriod::Month.top_copy(), " THIS MONTH'S TOP 10:");
    assert_eq!(
        RaceRankPeriod::AllTime.best_copy(),
        "MY ALL TIME BEST SCORE:"
    );
    assert_eq!(RaceRankPeriod::AllTime.top_copy(), " ALL TIME TOP 10:");
    let (_catalog, model) = open_model();
    assert_eq!(model.page_copy(32), "11 - 15 of 32 Locations");
}

#[test]
fn localization_contract_is_unique_and_keeps_en_ru_placeholders_identical() {
    let keys = RACE_RANK_LOCALIZATION_ENTRIES
        .iter()
        .map(|entry| entry.key)
        .collect::<BTreeSet<_>>();
    assert_eq!(keys.len(), RACE_RANK_LOCALIZATION_ENTRIES.len());
    assert!(keys.iter().all(|key| key.starts_with("ui.race.rank.")));
    for entry in RACE_RANK_LOCALIZATION_ENTRIES {
        assert_eq!(
            placeholders(entry.en),
            placeholders(entry.ru),
            "placeholder mismatch for {}",
            entry.key
        );
    }
}

#[test]
fn dynamic_rank_copy_is_published_as_keyed_templates_and_arguments() {
    let page = open_model().1.page_localized(32);
    assert_eq!(page.key, RACE_RANK_PAGE_KEY);
    assert_eq!(page.fallback, "{first} - {last} of {total} Locations");
    assert_eq!(page.args.get("first").map(String::as_str), Some("11"));
    assert_eq!(page.args.get("last").map(String::as_str), Some("15"));
    assert_eq!(page.args.get("total").map(String::as_str), Some("32"));

    let player =
        race_rank_score_localized(RaceRankScoreColumn::Player, "Server-authored Player");
    assert_eq!(player.key, RACE_RANK_PLAYER_VALUE_KEY);
    assert_eq!(player.fallback, "{player}");
    assert_eq!(
        player.args.get("player").map(String::as_str),
        Some("Server-authored Player")
    );
    assert_eq!(
        RaceRankPeriod::Month.tab_localized().key,
        RACE_RANK_PERIOD_MONTH_KEY
    );
    assert_eq!(
        RaceRankPeriod::AllTime.top_localized().key,
        RACE_RANK_TOP_ALL_TIME_KEY
    );
}

#[test]
fn presentation_ecs_is_key_first_and_preserves_replacement_font_metrics() {
    let asset_root = tempdir().unwrap();
    let catalog = RaceRankCatalog::embedded().unwrap();
    let mut model = RaceRankModel::default();
    model
        .open(
            &catalog,
            RaceRankOpenContext {
                pcuid: 42,
                current_ep_id: 15,
                npc_name: "Dexter".to_owned(),
                npc_target_instance_id: 1_234,
                rank_url: RACE_RANK_RETROBUTION_URL.to_owned(),
            },
        )
        .unwrap();
    model
        .complete_fetch(1, &response_with_top_count(3))
        .unwrap();
    model.clear_outputs();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_resource(catalog)
        .insert_resource(model)
        .add_plugins(RaceRankUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut all_text =
        world.query_filtered::<(&Text, (&TextFont, &LineHeight), Option<&LocalizedText>), With<Text>>();
    let rows = all_text.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), 62);
    assert!(rows.iter().all(|(_, _, localized)| localized.is_some()));
    for (_, font, _) in &rows {
        assert_eq!(
            (*font.1),
            LineHeight::Px(font.0.font_size.eval(Vec2::ZERO, 16.0).max(1.0)),
            "localization must not invent replacement-font line-height compensation"
        );
    }

    let localized = rows
        .iter()
        .map(|(_, _, localized)| localized.unwrap())
        .collect::<Vec<_>>();
    assert!(localized.iter().any(|value| {
        value.key == RACE_RANK_NPC_NAME_KEY
            && value.fallback == "{name}"
            && value.args.get("name").map(String::as_str) == Some("Dexter")
    }));
    assert!(localized.iter().any(|value| {
        value.key == RACE_RANK_PLAYER_VALUE_KEY
            && value.fallback == "{player}"
            && value.args.get("player").map(String::as_str) == Some("Player 42")
    }));
    assert!(localized.iter().all(|value| {
        !value.fallback.contains("Dexter") && !value.fallback.contains("Player 42")
    }));
}

#[test]
fn source_audit_rejects_raw_text_literals_at_spawn_sites() {
    let source = concat!(
        include_str!("containers.rs"),
        "\n",
        include_str!("assets.rs"),
        "\n",
        include_str!("constants.rs"),
        "\n",
        include_str!("layout.rs"),
        "\n",
        include_str!("frame.rs"),
        "\n",
        include_str!("commands.rs"),
        "\n",
        include_str!("types.rs"),
        "\n",
        include_str!("textures.rs"),
        "\n",
        include_str!("localization_race_rank_localization_entries.rs"),
        "\n",
        include_str!("interaction.rs"),
        "\n",
        include_str!("validation.rs"),
        "\n",
        include_str!("input.rs"),
        "\n",
        include_str!("operations.rs"),
        "\n",
        include_str!("models.rs"),
        "\n",
        include_str!("state.rs"),
        "\n",
        include_str!("view.rs"),
        "\n",
        include_str!("systems.rs"),
        "\n",
        include_str!("../rank.rs")
    );
    let production = source.split("#[cfg(test)]").next().unwrap();
    let raw_literal_spawn = ["Text::new(", "\""].concat();
    assert!(!production.contains(&raw_literal_spawn));
    assert!(production.contains("LocalizedText::new"));
    assert!(production.contains("before(LocalizationSet::Apply)"));
}

#[test]
fn clean_slide_advances_twice_and_settled_1264x681_layout_is_exact() {
    let (_catalog, mut model) = open_model();
    model.advance_slide(0.1);
    assert!((model.window_scroll() - 0.2).abs() < f32::EPSILON);
    assert!((model.left_scroll_sample() - 0.1).abs() < f32::EPSILON);
    assert!((model.right_scroll_sample() - 0.2).abs() < f32::EPSILON);
    model.advance_slide(1.0);
    assert_eq!(model.window_scroll(), 1.0);
    let layout = race_rank_layout(Vec2::new(1_264.0, 681.0), 1.0);
    assert_eq!(
        layout.left_group,
        RaceUiRect::new(122.0, 21.5, 490.0, 632.0)
    );
    assert_eq!(
        layout.right_group,
        RaceUiRect::new(625.0, 21.5, 472.0, 632.0)
    );
    assert_eq!(layout.shell, RaceUiRect::new(114.0, 14.5, 1_035.0, 653.0));
    assert_eq!(layout.close, RaceUiRect::new(1_102.0, 21.5, 30.0, 30.0));
    assert_eq!(layout.help, RaceUiRect::new(1_107.0, 613.5, 30.0, 30.0));
}

#[test]
fn help_has_no_command_and_close_emits_only_external_lifecycle_effects() {
    let (_catalog, mut model) = open_model();
    model.complete_fetch(1, "").unwrap();
    model.clear_outputs();
    assert!(model.close());
    assert_eq!(
        std::iter::from_fn(|| model.pop_output()).collect::<Vec<_>>(),
        vec![
            RaceRankOutput::Effect(RaceRankEffect::ReleaseNpcCamera),
            RaceRankOutput::Effect(RaceRankEffect::FreeLegacyAssets),
            RaceRankOutput::Effect(RaceRankEffect::ExitMode),
        ]
    );
}

#[test]
fn all_semantic_rank_assets_exist_and_catalog_hash_is_exact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let catalog_path = root.join(RACE_RANK_CATALOG_PATH);
    let bytes = fs::read(&catalog_path).unwrap();
    let hash = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    assert_eq!(hash, RACE_RANK_CATALOG_SHA256);
    let catalog = RaceRankCatalog::embedded().unwrap();
    let mut paths = RACE_RANK_STATIC_ASSET_PATHS
        .into_iter()
        .chain(
            catalog
                .locations()
                .iter()
                .flat_map(|location| [&location.big_image[..], &location.small_image[..]]),
        )
        .collect::<Vec<_>>();
    paths.sort_unstable();
    assert_eq!(paths.len(), 101);
    let mut png_set = Sha256::new();
    for path in paths {
        let absolute = root.join(path);
        assert!(
            absolute.is_file(),
            "missing semantic rank asset {}",
            absolute.display()
        );
        assert!(
            fs::metadata(&absolute).unwrap().len() > 0,
            "empty semantic rank asset {}",
            absolute.display()
        );
        png_set.update(path.as_bytes());
        png_set.update([0_u8]);
        png_set.update(fs::read(&absolute).unwrap());
    }
    let png_set = png_set
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    assert_eq!(png_set, RACE_RANK_PNG_SET_SHA256);
}
