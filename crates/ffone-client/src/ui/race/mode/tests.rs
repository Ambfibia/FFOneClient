use std::{collections::BTreeSet, fs, path::Path};

use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::race_ui::mode::*;

fn player(active: bool) -> RacePlayerState {
    RacePlayerState {
        ring_race_active: active,
        current_ep_id: 15,
        top_record: RaceTopRecord {
            rank: 2,
            rings: 31,
            score: 12_345,
            time_seconds: 97,
        },
        fatigue: 100,
        fatigue_level: 1,
        fusion_matter: 1_000,
        cursor_was_locked: true,
        ..default()
    }
}

fn npc() -> RaceNpcContext {
    RaceNpcContext {
        instance_id: 8_123,
        has_race_start_voice: true,
    }
}

fn drain(model: &mut RaceModeModel) -> Vec<RaceModeOutput> {
    std::iter::from_fn(|| model.pop_output()).collect()
}

#[test]
fn start_open_queues_exact_record_request_abi() {
    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Start,
            npc: Some(npc()),
            player: player(false),
            current_ep_instance_exists: true,
        })
        .unwrap();
    assert_eq!(model.phase(), RaceModePhase::AwaitingStart);
    assert_eq!(
        drain(&mut model),
        vec![
            RaceModeOutput::Effect(RaceModeEffect::SetCursorLocked(false)),
            RaceModeOutput::Request(RaceRequestIntent::Start {
                request_id: 1,
                i_start_ecom_id: 8_123,
                i_ep_race_mode: 1,
                i_ep_ticket_item_slot_num: 0,
            }),
        ]
    );
    let intent = RaceRequestIntent::Start {
        request_id: 1,
        i_start_ecom_id: 8_123,
        i_ep_race_mode: 1,
        i_ep_ticket_item_slot_num: 0,
    };
    assert_eq!(intent.packet_id(), RACE_START_REQUEST_PACKET_ID);
    assert_eq!(intent.abi_size(), 12);
    assert_eq!(RACE_START_REQUEST_ABI.map(|field| field.offset), [0, 4, 8]);
}

#[test]
fn race_request_and_reply_codecs_are_exact_and_registry_checked() {
    let request = RaceRequestIntent::Start {
        request_id: 9,
        i_start_ecom_id: 8_123,
        i_ep_race_mode: 1,
        i_ep_ticket_item_slot_num: 4,
    }
    .encode_registered()
    .unwrap();
    assert_eq!(request.packet_type(), RACE_START_REQUEST_PACKET_ID);
    assert_eq!(request.payload().len(), 12);
    assert_eq!(read_i32(request.payload(), 0), 8_123);
    assert_eq!(read_i32(request.payload(), 8), 4);

    let mut end = vec![0; RACE_END_SUCCESS_SIZE];
    write_i32(&mut end, 4, 97);
    write_i32(&mut end, 8, 28);
    write_i32(&mut end, 12, 45_678);
    end[44..46].copy_from_slice(&2i16.to_le_bytes());
    end[46..48].copy_from_slice(&77i16.to_le_bytes());
    assert!(matches!(
        decode_race_reply_0104(RACE_END_SUCCESS_PACKET_ID, &end),
        Ok(RaceModeReply::EndSuccess(RaceEndSuccess {
            race_time_seconds: 97,
            ring_count: 28,
            score: 45_678,
            reward_item: RaceRewardItem {
                item_type: 2,
                item_id: 77,
                ..
            },
            ..
        }))
    ));
    assert!(matches!(
        decode_race_reply_0104(RACE_END_SUCCESS_PACKET_ID, &end[..71]),
        Err(RaceReplyCodecError0104::WrongSize { .. })
    ));
}

#[test]
fn start_failure_exits_without_start_audio_or_icon_swap() {
    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Start,
            npc: Some(npc()),
            player: player(false),
            current_ep_instance_exists: true,
        })
        .unwrap();
    assert_eq!(model.phase(), RaceModePhase::AwaitingStart);
    assert!(!model.phase().paints());
    assert!(!model.controls_enabled(false));
    model.clear_outputs();
    model
        .apply_reply(
            RaceReplyEnvelope {
                request_id: 1,
                reply: RaceModeReply::StartFailure { error_code: 8 },
            },
            55.0,
        )
        .unwrap();
    assert_eq!(model.phase(), RaceModePhase::Hidden);
    assert!(!model.player().ring_race_active);
    assert_eq!(
        drain(&mut model),
        vec![
            RaceModeOutput::Effect(RaceModeEffect::DeactivateRings),
            RaceModeOutput::Effect(RaceModeEffect::SystemMessage {
                message_id: 150,
                key: "",
            }),
            RaceModeOutput::Effect(RaceModeEffect::EndCameraSubTarget),
            RaceModeOutput::Effect(RaceModeEffect::SetCursorLocked(true)),
            RaceModeOutput::Effect(RaceModeEffect::ExitMode),
        ]
    );
}

#[test]
fn cancel_failure_clears_unowned_race_without_icon_swap() {
    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Start,
            npc: None,
            player: player(true),
            current_ep_instance_exists: true,
        })
        .unwrap();
    assert_eq!(
        model.pop_output(),
        Some(RaceModeOutput::Effect(RaceModeEffect::SetCursorLocked(
            false
        )))
    );
    assert_eq!(
        model.pop_output(),
        Some(RaceModeOutput::Request(RaceRequestIntent::Cancel {
            request_id: 1,
            i_start_ecom_id: 0,
        }))
    );
    model
        .apply_reply(
            RaceReplyEnvelope {
                request_id: 1,
                reply: RaceModeReply::CancelFailure { error_code: 99 },
            },
            0.0,
        )
        .unwrap();
    assert!(!model.player().ring_race_active);
    let output = drain(&mut model);
    assert!(output.contains(&RaceModeOutput::Effect(RaceModeEffect::DeactivateRings)));
    assert!(!output.iter().any(|entry| matches!(entry, RaceModeOutput::Effect(RaceModeEffect::Ecom(_)))));
    assert!(output.contains(&RaceModeOutput::Effect(RaceModeEffect::ExitMode)));
}

#[test]
fn end_success_updates_result_status_and_exact_inventory_effects() {
    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::End,
            npc: Some(npc()),
            player: player(true),
            current_ep_instance_exists: true,
        })
        .unwrap();
    model.clear_outputs();
    let reply = RaceEndSuccess {
        race_mode: 1,
        race_time_seconds: 97,
        ring_count: 28,
        score: 45_678,
        rank: 2,
        reward_fusion_matter: 750,
        top_score: 45_678,
        top_rank: 2,
        top_time_seconds: 97,
        top_ring_count: 28,
        fusion_matter: 1_750,
        reward_item: RaceRewardItem {
            item_type: 2,
            item_id: 77,
            item_opt: 3,
            time_limit: 0,
            e_il: 1,
            slot: 4,
        },
        fatigue: 0,
        fatigue_level: 2,
    };
    model
        .apply_reply(
            RaceReplyEnvelope {
                request_id: 1,
                reply: RaceModeReply::EndSuccess(reply),
            },
            0.0,
        )
        .unwrap();
    assert_eq!(model.phase(), RaceModePhase::Result);
    assert_eq!(*model.result(), RaceResult::from(reply));
    assert_eq!(model.player().top_record.score, 45_678);
    assert_eq!(model.player().fusion_matter, 1_750);
    assert_eq!(RACE_END_SUCCESS_ABI[0].offset, 0);
    assert_eq!(RACE_END_SUCCESS_ABI[11].offset, 44);
    assert_eq!(RACE_END_SUCCESS_ABI[18].offset, 68);
    assert_eq!(RACE_END_SUCCESS_SIZE, 72);
    let output = drain(&mut model);
    assert!(
        output.contains(&RaceModeOutput::Effect(RaceModeEffect::ReceiveRewardItem {
            inventory_location: 1,
            inventory_slot: 4,
            item: reply.reward_item,
        }))
    );
    assert!(output.contains(&RaceModeOutput::Effect(
        RaceModeEffect::CheckFirstUseCondition(2)
    )));
    assert!(output.contains(&RaceModeOutput::Effect(
        RaceModeEffect::CheckFirstUseCondition(13)
    )));
    assert!(
        output.contains(&RaceModeOutput::Effect(RaceModeEffect::MessageBox {
            message_id: 730,
            box_type: 9,
            copy: RACE_EMPTY_ENERGY_WARNING,
        }))
    );
}

#[test]
fn end_failure_clears_race_without_showing_a_zero_result() {
    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::End,
            npc: Some(npc()),
            player: player(true),
            current_ep_instance_exists: true,
        })
        .unwrap();
    assert!(!model.phase().paints());
    model.clear_outputs();
    model
        .apply_reply(
            RaceReplyEnvelope {
                request_id: 1,
                reply: RaceModeReply::EndFailure { error_code: 1 },
            },
            0.0,
        )
        .unwrap();
    assert_eq!(model.phase(), RaceModePhase::Hidden);
    assert_eq!(*model.result(), RaceResult::default());
    assert_eq!(
        drain(&mut model),
        vec![
            RaceModeOutput::Effect(RaceModeEffect::DeactivateRings),
            RaceModeOutput::Effect(RaceModeEffect::EndCameraSubTarget),
            RaceModeOutput::Effect(RaceModeEffect::SetCursorLocked(true)),
            RaceModeOutput::Effect(RaceModeEffect::ExitMode),
        ]
    );
}

#[test]
fn no_crate_reward_does_not_clear_inventory_slot_zero() {
    let mut model = RaceModeModel::default();
    model.open(RaceModeOpenContext {
        ecom_type: RaceEcomType::End,
        npc: Some(npc()),
        player: player(true),
        current_ep_instance_exists: true,
    }).unwrap();
    model.clear_outputs();
    let reply = RaceEndSuccess {
        reward_fusion_matter: 25,
        fusion_matter: 1_025,
        reward_item: RaceRewardItem::default(),
        ..default()
    };
    model.apply_reply(RaceReplyEnvelope {
        request_id: 1,
        reply: RaceModeReply::EndSuccess(reply),
    }, 0.0).unwrap();
    assert_eq!(model.result().reward_fusion_matter, 25);
    assert!(!model.result().reward_item.was_granted());
    assert!(!drain(&mut model).iter().any(|entry| matches!(entry,
        RaceModeOutput::Effect(RaceModeEffect::ReceiveRewardItem { .. }))));
}

#[test]
fn unreachable_start_and_fail_ui_are_not_repaired() {
    assert!(!RaceEcomType::Start.paints());
    assert!(RaceEcomType::End.paints());
    assert!(!RaceEcomType::Fail.paints());
    assert!(!RaceEcomType::Rank.paints());

    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Fail,
            npc: None,
            player: player(false),
            current_ep_instance_exists: true,
        })
        .unwrap();
    assert_eq!(model.phase(), RaceModePhase::FailSystemMessage);
    assert!(!model.phase().paints());
}

#[test]
fn stale_or_wrong_reply_fails_closed_without_consuming_pending_request() {
    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Start,
            npc: Some(npc()),
            player: player(false),
            current_ep_instance_exists: true,
        })
        .unwrap();
    model.clear_outputs();
    assert_eq!(
        model.apply_reply(
            RaceReplyEnvelope {
                request_id: 2,
                reply: RaceModeReply::StartSuccess {
                    start_tick: 1,
                    limit_time: 60,
                },
            },
            5.0,
        ),
        Err(RaceModeReplyError::StaleRequest {
            expected: 1,
            received: 2,
        })
    );
    assert_eq!(model.pending_request_id(), Some(1));
    assert!(model.outputs.is_empty());
}

#[test]
fn result_copy_geometry_and_overlap_quirks_are_exact() {
    assert_eq!(race_filled_star_count(1), 5);
    assert_eq!(race_rank_copy(1), "Genius!");
    assert_eq!(race_rank_copy(2), "Awesome!");
    assert_eq!(race_rank_copy(3), "Good");
    assert_eq!(race_rank_copy(4), "Not Bad");
    assert_eq!(race_rank_copy(5), "Bleh!");
    assert_eq!(race_time_copy(3_661), "1:01:01");
    assert_eq!(race_score_copy(1_234), "1,234.00");
    assert_eq!(
        RACE_INVENTORY_FULL_RECT,
        RaceUiRect::new(120.0, 254.0, -38.0, 20.0)
    );
    assert_eq!(RACE_ACCEPT_RECT, RaceUiRect::new(230.0, 393.0, 154.0, 26.0));
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
fn localization_contract_is_unique_and_keeps_en_ru_placeholders_identical() {
    let keys = RACE_MODE_LOCALIZATION_ENTRIES
        .iter()
        .map(|entry| entry.key)
        .collect::<BTreeSet<_>>();
    assert_eq!(keys.len(), RACE_MODE_LOCALIZATION_ENTRIES.len());
    assert!(keys.iter().all(|key| key.starts_with("ui.race.mode.")));
    for entry in RACE_MODE_LOCALIZATION_ENTRIES {
        assert_eq!(
            placeholders(entry.en),
            placeholders(entry.ru),
            "placeholder mismatch for {}",
            entry.key
        );
    }
    assert!(!keys.iter().any(|key| key.contains("start")));
    assert!(!keys.iter().any(|key| key.contains("fail")));
}

#[test]
fn dynamic_result_copy_is_published_as_keyed_templates_and_arguments() {
    let time = race_time_localized(3_661);
    assert_eq!(time.key, RACE_MODE_TIME_VALUE_KEY);
    assert_eq!(time.fallback, "{time}");
    assert_eq!(time.args.get("time").map(String::as_str), Some("1:01:01"));

    let score = race_score_localized(1_234);
    assert_eq!(score.key, RACE_MODE_SCORE_VALUE_KEY);
    assert_eq!(
        score.args.get("score").map(String::as_str),
        Some("1,234.00")
    );

    let item = race_reward_item_name_localized("Player-authored reward");
    assert_eq!(item.key, RACE_MODE_ITEM_NAME_KEY);
    assert_eq!(item.fallback, "{name}");
    assert_eq!(
        item.args.get("name").map(String::as_str),
        Some("Player-authored reward")
    );
    assert_eq!(race_rank_localized(2).key, RACE_MODE_RATING_AWESOME_KEY);
}

#[test]
fn presentation_ecs_is_key_first_and_never_spawns_dead_start_or_fail_copy() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(RaceModeUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut all_text =
        world.query_filtered::<(&Text, (&TextFont, &LineHeight), Option<&LocalizedText>), With<Text>>();
    let rows = all_text.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), 20);
    assert!(rows.iter().all(|(_, _, localized)| localized.is_some()));
    for (_, font, _) in &rows {
        assert_eq!(
            (*font.1),
            LineHeight::Px(font.0.font_size.eval(Vec2::ZERO, 16.0).max(1.0)),
            "localization must not invent replacement-font line-height compensation"
        );
    }
    let forbidden = ["Start", "Race Fail", "press [OK] and restart race.", "OK"];
    for (text, _, localized) in rows {
        let localized = localized.unwrap();
        assert!(!forbidden.contains(&text.0.as_str()));
        assert!(!forbidden.contains(&localized.fallback.as_str()));
    }
}

#[test]
fn source_audit_rejects_raw_text_literals_and_legacy_copy_constants_at_spawn_sites() {
    let source = concat!(
        include_str!("state.rs"),
        "\n",
        include_str!("containers.rs"),
        "\n",
        include_str!("assets.rs"),
        "\n",
        include_str!("codec.rs"),
        "\n",
        include_str!("commands.rs"),
        "\n",
        include_str!("constants.rs"),
        "\n",
        include_str!("types.rs"),
        "\n",
        include_str!("validation.rs"),
        "\n",
        include_str!("layout.rs"),
        "\n",
        include_str!("interaction.rs"),
        "\n",
        include_str!("textures.rs"),
        "\n",
        include_str!("output.rs"),
        "\n",
        include_str!("localization_race_mode_localization_entries.rs"),
        "\n",
        include_str!("input.rs"),
        "\n",
        include_str!("audio.rs"),
        "\n",
        include_str!("models.rs"),
        "\n",
        include_str!("operations.rs"),
        "\n",
        include_str!("view.rs"),
        "\n",
        include_str!("../mode.rs")
    );
    let production = source.split("#[cfg(test)]").next().unwrap();
    let raw_literal_spawn = ["Text::new(", "\""].concat();
    let raw_copy_spawn = ["Text::new(", "RACE_COPY_"].concat();
    assert!(!production.contains(&raw_literal_spawn));
    assert!(!production.contains(&raw_copy_spawn));
    assert!(production.contains("LocalizedText::new"));
    assert!(production.contains("before(LocalizationSet::Apply)"));
}

#[test]
fn every_result_texture_has_exact_source_hash_and_dimensions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for contract in RACE_RESULT_TEXTURES {
        let path = root.join(contract.runtime_path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let actual = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<String>();
        assert_eq!(actual, contract.sha256, "hash mismatch: {}", path.display());
        let dimensions = image::image_dimensions(&path).unwrap();
        assert_eq!(
            dimensions,
            (contract.source_width, contract.source_height),
            "dimension mismatch: {}",
            path.display()
        );
    }
}
