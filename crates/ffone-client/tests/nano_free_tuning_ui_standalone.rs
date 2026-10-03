//! Standalone contract tests for clean-Retrobution NanoFreeTuning while the
//! shared crate export remains intentionally untouched.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

use std::{fs, path::Path};

use bevy::prelude::*;
use sha2::{Digest, Sha256};

pub use ffone_client::localization;

#[path = "../src/ui/nano_free_tuning/mod.rs"]
mod nano_free_tuning_ui;

use localization::LocalizedText;
use nano_free_tuning_ui::*;

fn sample_content(nano_id: i16) -> NanoFreeTuningContent {
    NanoFreeTuningContent {
        nano_id,
        nano_style: 0,
        nano_name: "Buttercup".to_owned(),
        powers: [
            NanoFreeTuningPower {
                tune_id: 1,
                skill_id: 1,
                icon_path: "ui/en/gameplay/nano/icons/skill/skillicon_10.png".to_owned(),
                name: "MISS FIRE".to_owned(),
                power_type: "STUN - CONE".to_owned(),
                description: "Mange uses fire to stun enemies in the target area.".to_owned(),
            },
            NanoFreeTuningPower {
                tune_id: 2,
                skill_id: 2,
                icon_path: "ui/en/gameplay/nano/icons/skill/skillicon_04.png".to_owned(),
                name: "RALLYING CRY".to_owned(),
                power_type: "HEALTH - GROUP".to_owned(),
                description: "Buttercup's warcry heals her group.".to_owned(),
            },
            NanoFreeTuningPower {
                tune_id: 3,
                skill_id: 3,
                icon_path: "ui/en/gameplay/nano/icons/skill/skillicon_26.png".to_owned(),
                name: "BUTTERCUP BURST".to_owned(),
                power_type: "SCAVENGE".to_owned(),
                description: "Collect even more Fusion Matter.".to_owned(),
            },
        ],
    }
}

fn open_context(killed_fusion: bool, nano_id: i16) -> NanoFreeTuningOpenContext {
    NanoFreeTuningOpenContext {
        player_id: 7_001,
        killed_fusion,
        content: sample_content(nano_id),
        world: NanoFreeTuningWorldSnapshot {
            player: NanoFreeTuningTransform {
                position: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::IDENTITY,
            },
            first_defeated_fusion: None,
        },
    }
}

fn selecting_model() -> NanoFreeTuningModel {
    let mut model = NanoFreeTuningModel::default();
    model.open(open_context(false, 1)).unwrap();
    model.clear_intents();
    model
        .advance(NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS)
        .unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::CameraSettle);
    model.advance(0.001).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::PowerSelection);
    model.clear_intents();
    model
}

fn awaiting_model(power_index: usize) -> (NanoFreeTuningModel, u64) {
    let mut model = selecting_model();
    let token = model.click_power(power_index).unwrap();
    model.clear_intents();
    (model, token)
}

fn success_for(nano_id: i16, skill_id: i16) -> NanoTuneSuccess {
    NanoTuneSuccess {
        nano_id,
        skill_id,
        fusion_matter: 12_345,
        item_slots: [-1; NANO_TUNE_ITEM_SLOT_COUNT],
        items: [NanoTuneItemBase::default(); NANO_TUNE_ITEM_SLOT_COUNT],
    }
}

#[test]
fn dropped_tune_reply_releases_modal_and_restores_camera_and_cursor() {
    let (mut model, _) = awaiting_model(0);
    model.advance(10.0).unwrap();
    assert!(model.pending_request().is_some());
    model.advance(5.1).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::Closed);
    assert!(model.pending_request().is_none());
    assert!(model.intents().contains(&NanoFreeTuningIntent::Cinematic(
        NanoFreeTuningCinematicIntent::EndSubTarget
    )));
    assert!(model.intents().contains(&NanoFreeTuningIntent::Ui(
        NanoFreeTuningUiIntent::RestoreCapturedCursor
    )));
}

#[test]
fn resumed_tuning_positions_preview_without_replaying_creation() {
    let mut model = NanoFreeTuningModel::default();
    model.open(open_context(false, 1)).unwrap();
    assert!(model.intents().iter().any(|intent| matches!(intent,
        NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::RevealPreviewNano { position, .. })
            if *position == Vec3::new(1.0, 5.0, 0.0))));
    assert!(!model.intents().iter().any(|intent| matches!(
        intent,
        NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::CallPreviewNano)
    )));
}

#[test]
fn clean_slot_serialized_anchors_and_packet_abis_are_exact() {
    assert_eq!(NANO_FREE_TUNING_GAME_MODE_SLOT, 22);
    assert_eq!(NANO_FREE_TUNING_GAME_OBJECT_PATH_ID, 1_353);
    assert_eq!(NANO_FREE_TUNING_TRANSFORM_PATH_ID, 1_266);
    assert_eq!(NANO_FREE_TUNING_MODE_COMPONENT_PATH_ID, 1_486);
    assert_eq!(NANO_FREE_TUNING_UI_COMPONENT_PATH_ID, 1_487);
    assert_eq!(NANO_FREE_TUNING_SKIN_PATH_ID, 1_379);

    assert_eq!(NANO_TUNE_REQUEST_PACKET_ID, 318_767_120);
    assert_eq!(NANO_TUNE_REQUEST_SIZE, 44);
    assert_eq!(NANO_TUNE_REQUEST_ABI[0].offset, 0);
    assert_eq!(NANO_TUNE_REQUEST_ABI[1].offset, 2);
    assert_eq!(NANO_TUNE_REQUEST_ABI[2].offset, 4);
    assert_eq!(NANO_TUNE_REQUEST_ABI[2].count, 10);

    assert_eq!(NANO_TUNE_SUCCESS_PACKET_ID, 822_083_625);
    assert_eq!(NANO_TUNE_SUCCESS_SIZE, 168);
    assert_eq!(NANO_TUNE_SUCCESS_ABI[3].offset, 8);
    assert_eq!(NANO_TUNE_SUCCESS_ABI[4].offset, 48);
    assert_eq!(
        NANO_TUNE_SUCCESS_ABI[4].scalar,
        NanoFreeTuningAbiScalar::ItemBase
    );

    assert_eq!(NANO_TUNE_FAILURE_PACKET_ID, 822_083_669);
    assert_eq!(NANO_TUNE_FAILURE_SIZE, 8);
    assert_eq!(NANO_TUNE_FAILURE_ABI[1].offset, 4);
}

#[test]
fn viewport_bars_panel_and_three_power_geometry_match_clean_gui() {
    let bars = nano_free_tuning_black_bar_rects(1_280.0, 720.0);
    assert!((bars[0].height - 112.852_67).abs() < 0.000_1);
    assert_eq!(bars[0].x, 0.0);
    assert_eq!(bars[0].y, 0.0);
    assert!((bars[1].y - 607.147_34).abs() < 0.000_1);
    assert_eq!(bars[1].width, 1_280.0);

    assert_eq!(
        nano_free_tuning_panel_rect(1_280.0, 720.0),
        NanoFreeTuningRect::new(740.0, 130.0, 356.0, 460.0)
    );
    assert_eq!(
        nano_free_tuning_dormant_panel_rect(1_280.0, 720.0),
        NanoFreeTuningRect::new(782.0, 130.0, 356.0, 460.0)
    );
    assert_eq!(
        NANO_FREE_TUNING_TITLE_RECT,
        NanoFreeTuningRect::new(100.0, 100.0, 300.0, 15.0)
    );
    assert_eq!(
        NANO_FREE_TUNING_POWER_LAYOUTS[0].select_button,
        NanoFreeTuningRect::new(237.0, 145.0, 97.0, 20.0)
    );
    assert_eq!(
        NANO_FREE_TUNING_POWER_LAYOUTS[1].description,
        NanoFreeTuningRect::new(20.0, 261.0, 315.0, 25.0)
    );
    assert_eq!(
        NANO_FREE_TUNING_POWER_LAYOUTS[2].icon,
        NanoFreeTuningRect::new(18.0, 304.0, 35.0, 35.0)
    );
}

fn assert_node_rect(node: &Node, rect: NanoFreeTuningRect) {
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.left, px(rect.x));
    assert_eq!(node.top, px(rect.y));
    assert_eq!(node.width, px(rect.width));
    assert_eq!(node.height, px(rect.height));
}

#[test]
fn reached_gui_styles_draw_order_and_every_text_contract_are_exact() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin {
            file_path: workspace_root
                .join("assets/game")
                .to_string_lossy()
                .into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(NanoFreeTuningUiPlugin);
    app.update();

    let world = app.world_mut();
    let text_entities = {
        let mut query = world.query_filtered::<Entity, With<Text>>();
        query.iter(world).collect::<Vec<_>>()
    };
    assert_eq!(text_entities.len(), 16);
    for entity in text_entities {
        let localized = world
            .get::<LocalizedText>(entity)
            .unwrap_or_else(|| panic!("every NanoFreeTuning Text must be key-first: {entity:?}"));
        let style = world
            .get::<NanoFreeTuningUiTextStyle>(entity)
            .unwrap_or_else(|| {
                panic!("every NanoFreeTuning Text needs source GUIStyle: {entity:?}")
            });
        let font = world.get::<TextFont>(entity).unwrap();
        let transform = world.get::<UiTransform>(entity).unwrap();
        assert!(!localized.key.trim().is_empty());
        assert_ne!(localized.key, "ui.content.passthrough");
        assert_eq!(style.source_skin_path_id(), NANO_FREE_TUNING_SKIN_PATH_ID);
        assert_eq!(style.replacement_font_path(), NANO_FREE_TUNING_FONT_PATH);
        assert_eq!(font.font_size, style.font_size().into());
        assert_eq!(
            *world.get::<bevy::text::LineHeight>(entity).unwrap(),
            bevy::text::LineHeight::Px(style.line_height())
        );
        assert_eq!(
            transform.translation,
            Val2::px(0.0, style.replacement_y_offset())
        );
    }

    let panel = {
        let mut query = world.query_filtered::<Entity, With<NanoFreeTuningPresentationPanel>>();
        query.single(world).unwrap()
    };
    let panel_children = world
        .get::<Children>(panel)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(panel_children.len(), 16);

    let title = panel_children[0];
    assert_node_rect(
        world.get::<Node>(title).unwrap(),
        NANO_FREE_TUNING_TITLE_RECT,
    );
    assert_eq!(
        world.get::<LocalizedText>(title).unwrap().key,
        NANO_FREE_TUNING_TITLE_LOCALIZATION_KEY
    );
    assert_eq!(
        *world.get::<NanoFreeTuningUiTextStyle>(title).unwrap(),
        NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft
    );

    for (index, layout) in NANO_FREE_TUNING_POWER_LAYOUTS.iter().copied().enumerate() {
        let base = 1 + index * 5;
        let icon_slot = panel_children[base];
        assert_node_rect(world.get::<Node>(icon_slot).unwrap(), layout.icon);
        assert!(
            world
                .get::<NanoFreeTuningIconLabelStyle>(icon_slot)
                .is_some()
        );
        assert_eq!(
            world.get::<Node>(icon_slot).unwrap().padding,
            UiRect::new(px(0), px(0), px(3), px(3))
        );

        let name = panel_children[base + 1];
        assert_node_rect(world.get::<Node>(name).unwrap(), layout.name);
        assert_eq!(
            world.get::<LocalizedText>(name).unwrap().key,
            NANO_FREE_TUNING_POWER_NAME_LOCALIZATION_KEY
        );
        assert_eq!(
            *world.get::<NanoFreeTuningUiTextStyle>(name).unwrap(),
            NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft
        );

        let power_type = panel_children[base + 2];
        assert_node_rect(world.get::<Node>(power_type).unwrap(), layout.power_type);
        assert_eq!(
            world.get::<LocalizedText>(power_type).unwrap().key,
            NANO_FREE_TUNING_POWER_TYPE_LOCALIZATION_KEY
        );
        assert_eq!(
            *world.get::<NanoFreeTuningUiTextStyle>(power_type).unwrap(),
            NanoFreeTuningUiTextStyle::TransparentYellowSmallUpperLeft
        );

        let description = panel_children[base + 3];
        assert_node_rect(world.get::<Node>(description).unwrap(), layout.description);
        assert_eq!(
            world.get::<LocalizedText>(description).unwrap().key,
            NANO_FREE_TUNING_POWER_DESCRIPTION_LOCALIZATION_KEY
        );
        assert_eq!(
            *world.get::<NanoFreeTuningUiTextStyle>(description).unwrap(),
            NanoFreeTuningUiTextStyle::TransparentBlueUpperLeft
        );

        let button = panel_children[base + 4];
        assert_node_rect(world.get::<Node>(button).unwrap(), layout.select_button);
        assert_eq!(
            world
                .get::<NanoFreeTuningPowerButton>(button)
                .unwrap()
                .power_index,
            index
        );
        let label = world
            .get::<Children>(button)
            .unwrap()
            .iter()
            .next()
            .unwrap();
        assert_eq!(
            world.get::<LocalizedText>(label).unwrap().key,
            NANO_FREE_TUNING_SELECT_LOCALIZATION_KEY
        );
        assert_eq!(
            *world.get::<NanoFreeTuningUiTextStyle>(label).unwrap(),
            NanoFreeTuningUiTextStyle::ButtonMiddleCenter
        );
    }
}

#[test]
fn serialized_fusionfall_nano_skin_text_metrics_are_locked_per_style() {
    let exact = [
        (
            NanoFreeTuningUiTextStyle::BigBlueMiddleLeft,
            "bigblue",
            1_012,
            3,
            [0.0; 4],
            true,
            0,
        ),
        (
            NanoFreeTuningUiTextStyle::BigYellowMiddleLeft,
            "bigyellow",
            1_012,
            3,
            [0.0; 4],
            true,
            0,
        ),
        (
            NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft,
            "TransparentLightBlue",
            903,
            0,
            [0.0; 4],
            false,
            0,
        ),
        (
            NanoFreeTuningUiTextStyle::TransparentYellowSmallUpperLeft,
            "TransparentYellowSmall",
            948,
            0,
            [0.0; 4],
            false,
            0,
        ),
        (
            NanoFreeTuningUiTextStyle::TransparentBlueUpperLeft,
            "TransparentBlue",
            948,
            0,
            [0.0; 4],
            true,
            0,
        ),
        (
            NanoFreeTuningUiTextStyle::ButtonMiddleCenter,
            "button",
            903,
            4,
            [6.0, 6.0, 3.0, 3.0],
            false,
            1,
        ),
    ];
    for (style, name, font_path_id, alignment, padding, word_wrap, clipping) in exact {
        assert_eq!(style.source_style_name(), name);
        assert_eq!(style.source_skin_path_id(), 1_379);
        assert_eq!(style.source_font_path_id(), font_path_id);
        assert_eq!(style.legacy_alignment(), alignment);
        assert_eq!(style.padding(), padding);
        assert_eq!(style.content_offset(), [0.0, 0.0]);
        assert_eq!(style.word_wrap(), word_wrap);
        assert_eq!(style.legacy_text_clipping(), clipping);
        assert_eq!(style.replacement_y_offset(), 0.0);
    }
    assert_eq!(NanoFreeTuningIconLabelStyle::SOURCE_STYLE_NAME, "label");
    assert_eq!(NanoFreeTuningIconLabelStyle::PADDING, [0.0, 0.0, 3.0, 3.0]);
}

#[test]
fn killed_fusion_entry_obeys_effect_gate_and_strict_phase_thresholds() {
    let mut model = NanoFreeTuningModel::default();
    model.open(open_context(true, 1)).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::EffectDelay);
    assert_eq!(model.phase().legacy_index(), Some(0));
    assert!(!model.panel_visible());
    assert!(model.intents().iter().any(|intent| matches!(
        intent,
        NanoFreeTuningIntent::Effect(NanoFreeTuningEffectIntent::Preload { effect_id: 705 })
    )));
    model.clear_intents();

    model.advance(0.5).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::EffectDelay);
    model.mark_effect_ready().unwrap();
    model.advance(0.0).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::EffectDelay);
    model.advance(0.001).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::ProjectileDelay);
    assert!(model.intents().iter().any(|intent| matches!(
        intent,
        NanoFreeTuningIntent::Effect(NanoFreeTuningEffectIntent::Instantiate {
            effect_id: 705,
            position,
            ..
        }) if *position == Vec3::new(1.0, 2.0, 1.0)
    )));
    model.clear_intents();

    model.advance(1.2).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::ProjectileDelay);
    model.advance(0.001).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::RevealDelay);
    assert!(model.intents().iter().any(|intent| matches!(
        intent,
        NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::SpawnCreationBullets {
            bullet_types: [76, 77],
            position,
        }) if *position == Vec3::new(1.0, 5.0, 0.0)
    )));
    model.clear_intents();

    model.advance(0.8).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::RevealDelay);
    model.advance(0.001).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::CameraApproach);
    assert!(model.intents().iter().any(|intent| matches!(
        intent,
        NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::RevealPreviewNano {
            position,
            rotation,
        }) if *position == Vec3::new(1.0, 5.0, 0.0)
            && rotation.abs_diff_eq(Quat::from_rotation_y(std::f32::consts::PI), 1.0e-6)
    )));
    assert!(model.intents().contains(&NanoFreeTuningIntent::Cinematic(
        NanoFreeTuningCinematicIntent::CallPreviewNano,
    )));
    model.clear_intents();

    model.advance(0.3).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::CameraApproach);
    model.advance(0.001).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::CameraSettle);
    // Phase 3 -> 4 deliberately preserves the timer started by phase 2.
    model.advance(1.698).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::CameraSettle);
    model.advance(0.002).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::PowerSelection);
    assert!(model.panel_visible());
}

#[test]
fn non_kill_entry_starts_at_phase_four_and_nano_two_checks_condition_23() {
    let mut model = NanoFreeTuningModel::default();
    model.open(open_context(false, 2)).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::CameraSettle);
    model.clear_intents();
    model.advance(2.0).unwrap();
    assert!(!model.panel_visible());
    model.advance(0.001).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::PowerSelection);
    assert!(model.panel_visible());
    assert!(model.controls_enabled());
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::FirstUseCheck(23))
    );
}

#[test]
fn entry_first_use_mapping_is_exact_and_other_nanos_do_not_guess() {
    assert_eq!(nano_free_tuning_entry_first_use_condition(3), Some(52));
    assert_eq!(nano_free_tuning_entry_first_use_condition(4), Some(39));
    assert_eq!(nano_free_tuning_entry_first_use_condition(5), Some(53));
    assert_eq!(nano_free_tuning_entry_first_use_condition(8), Some(51));
    assert_eq!(nano_free_tuning_entry_first_use_condition(9), Some(49));
    assert_eq!(nano_free_tuning_entry_first_use_condition(11), Some(50));
    assert_eq!(nano_free_tuning_entry_first_use_condition(1), None);
    assert_eq!(nano_free_tuning_entry_first_use_condition(49), None);
}

#[test]
fn select_then_confirm_emits_one_exact_zero_cost_request() {
    let mut model = selecting_model();
    model.select_power(2).unwrap();
    assert_eq!(model.selected_power(), Some(2));
    assert!(model.pending_request().is_none());
    assert!(model.intents().is_empty());

    let token = model.confirm_selection().unwrap();
    assert_eq!(token, 1);
    assert!(!model.gui_enabled());
    assert!(!model.controls_enabled());
    assert_eq!(
        model.pending_request(),
        Some(NanoFreeTuningPendingRequest {
            request_token: 1,
            power_index: 2,
            nano_id: 1,
            tune_id: 3,
            skill_id: 3,
        })
    );
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::Wire(NanoTuneWireIntent {
                request_token: 1,
                packet_id: 318_767_120,
                payload_size: 44,
                body: NanoTuneRequest {
                    nano_id: 1,
                    tune_id: 3,
                    needed_item_slots: [0; 10],
                },
            }))
    );
}

#[test]
fn only_correlated_matching_success_crosses_the_authoritative_commit_boundary() {
    let (mut model, token) = awaiting_model(1);
    let success = success_for(1, 2);
    model
        .apply_reply(NanoFreeTuningReplyEnvelope {
            request_token: token,
            packet_id: NANO_TUNE_SUCCESS_PACKET_ID,
            payload_size: NANO_TUNE_SUCCESS_SIZE,
            body: NanoFreeTuningReplyBody::Success(success.clone()),
        })
        .unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::ResultSkill);
    assert!(model.pending_request().is_none());
    assert_eq!(model.fault(), None);
    assert!(model.intents().contains(&NanoFreeTuningIntent::Cinematic(
        NanoFreeTuningCinematicIntent::PlaySelectedSkill { power_index: 1 }
    )));
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::AuthoritativeCommit(success))
    );
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::DeleteEcomIcon(5))
    );
}

#[test]
fn malformed_or_mismatched_success_fails_closed_without_a_commit() {
    let (mut wrong_size, token) = awaiting_model(0);
    let error = wrong_size
        .apply_reply(NanoFreeTuningReplyEnvelope {
            request_token: token,
            packet_id: NANO_TUNE_SUCCESS_PACKET_ID,
            payload_size: NANO_TUNE_SUCCESS_SIZE - 1,
            body: NanoFreeTuningReplyBody::Success(success_for(1, 1)),
        })
        .unwrap_err();
    assert!(matches!(
        error,
        NanoFreeTuningTransitionError::Protocol(NanoFreeTuningProtocolFault::PayloadSize { .. })
    ));
    assert!(!wrong_size.gui_enabled());
    assert!(wrong_size.panel_visible());
    assert!(
        !wrong_size
            .intents()
            .iter()
            .any(|intent| matches!(intent, NanoFreeTuningIntent::AuthoritativeCommit(_)))
    );

    let (mut wrong_skill, token) = awaiting_model(0);
    assert!(
        wrong_skill
            .apply_reply(NanoFreeTuningReplyEnvelope {
                request_token: token,
                packet_id: NANO_TUNE_SUCCESS_PACKET_ID,
                payload_size: NANO_TUNE_SUCCESS_SIZE,
                body: NanoFreeTuningReplyBody::Success(success_for(1, 3)),
            })
            .is_err()
    );
    assert_eq!(
        wrong_skill.fault(),
        Some(NanoFreeTuningFault::Protocol(
            NanoFreeTuningProtocolFault::SkillMismatch {
                expected: 1,
                actual: 3,
            }
        ))
    );
    assert!(
        !wrong_skill
            .intents()
            .iter()
            .any(|intent| matches!(intent, NanoFreeTuningIntent::AuthoritativeCommit(_)))
    );
}

#[test]
fn exact_failure_is_authoritative_but_never_mutates_nano_state() {
    let (mut model, token) = awaiting_model(2);
    let failure = NanoTuneFailure {
        player_id: 7_001,
        error_code: 17,
    };
    model
        .apply_reply(NanoFreeTuningReplyEnvelope {
            request_token: token,
            packet_id: NANO_TUNE_FAILURE_PACKET_ID,
            payload_size: NANO_TUNE_FAILURE_SIZE,
            body: NanoFreeTuningReplyBody::Failure(failure),
        })
        .unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::PowerSelection);
    assert_eq!(
        model.fault(),
        Some(NanoFreeTuningFault::ServerRejected {
            player_id: 7_001,
            error_code: 17,
        })
    );
    assert!(!model.gui_enabled());
    assert!(model.panel_visible());
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::AuthoritativeFailure(failure))
    );
    assert!(
        !model
            .intents()
            .iter()
            .any(|intent| matches!(intent, NanoFreeTuningIntent::AuthoritativeCommit(_)))
    );
    assert_eq!(
        model.select_power(0),
        Err(NanoFreeTuningTransitionError::ControlsDisabled)
    );
    assert_eq!(
        model.confirm_selection(),
        Err(NanoFreeTuningTransitionError::ControlsDisabled)
    );
}

#[test]
fn result_animation_hide_continuation_and_cancel_preserve_clean_flow() {
    let (mut model, token) = awaiting_model(0);
    model
        .apply_reply(NanoFreeTuningReplyEnvelope {
            request_token: token,
            packet_id: NANO_TUNE_SUCCESS_PACKET_ID,
            payload_size: NANO_TUNE_SUCCESS_SIZE,
            body: NanoFreeTuningReplyBody::Success(success_for(1, 1)),
        })
        .unwrap();
    model.clear_intents();
    model.set_result_animation_duration(1.5).unwrap();
    model.advance(1.5).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::ResultSkill);
    model.advance(0.001).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::ResultHide);
    assert!(model.intents().contains(&NanoFreeTuningIntent::Cinematic(
        NanoFreeTuningCinematicIntent::HidePreviewNano
    )));
    model.clear_intents();
    model.advance(1.0).unwrap();
    assert!(model.panel_visible());
    model.advance(0.001).unwrap();
    assert!(!model.panel_visible());
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::QueryAcquisitionContinuation)
    );
    model.clear_intents();

    model.resolve_acquisition_continuation(1).unwrap();
    model.advance(0.0).unwrap();
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::QueryAcquisitionContinuation)
    );
    model.resolve_acquisition_continuation(0).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::Closed);
    assert!(model.intents().contains(&NanoFreeTuningIntent::Cinematic(
        NanoFreeTuningCinematicIntent::EndSubTarget
    )));
    assert!(model.intents().contains(&NanoFreeTuningIntent::World(
        NanoFreeTuningWorldIntent::DestroyPreviewNano
    )));
    assert!(model.intents().contains(&NanoFreeTuningIntent::ExitMode));
}

#[test]
fn clean_idle_range_can_play_happy_but_never_reaches_sad_case_two() {
    let mut model = selecting_model();
    model.advance(3.0).unwrap();
    assert!(model.intents().is_empty());
    model.advance(0.001).unwrap();
    assert!(
        model
            .intents()
            .contains(&NanoFreeTuningIntent::RequestIdleRoll { exclusive_max: 2 })
    );
    model.apply_idle_roll(1).unwrap();
    assert!(model.intents().contains(&NanoFreeTuningIntent::Cinematic(
        NanoFreeTuningCinematicIntent::PlayIdleHappy
    )));

    model.clear_intents();
    model.advance(3.001).unwrap();
    assert_eq!(
        model.apply_idle_roll(2),
        Err(NanoFreeTuningTransitionError::InvalidIdleRoll(2))
    );
}

#[test]
fn converted_assets_are_hash_locked() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let asset_root = workspace_root.join("assets/game");
    for contract in NANO_FREE_TUNING_ASSET_CONTRACTS {
        let path = asset_root.join(contract.path);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!("cannot read {}: {error}", path.display());
        });
        assert_eq!(bytes.get(..8), Some(&[137, 80, 78, 71, 13, 10, 26, 10][..]));
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!(
            (width, height),
            (contract.source_width, contract.source_height)
        );
        let hash = format!("{:X}", Sha256::digest(&bytes));
        assert_eq!(hash, contract.png_sha256);
    }
}

#[test]
fn every_runtime_asset_path_resolves_without_legacy_runtime_dependencies() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for path in [
        NANO_FREE_TUNING_PANEL_PATH,
        NANO_FREE_TUNING_BLACK_PATH,
        NANO_FREE_TUNING_SELECT_NORMAL_PATH,
        NANO_FREE_TUNING_SELECT_HOVER_PATH,
        NANO_FREE_TUNING_FONT_PATH,
    ] {
        assert!(
            asset_root.join(path).is_file(),
            "missing runtime asset {path}"
        );
    }
    for power in sample_content(1).powers {
        assert!(
            asset_root.join(&power.icon_path).is_file(),
            "missing sample skill icon {}",
            power.icon_path
        );
    }
}
