use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use bevy::{
    asset::{AssetApp, AssetPlugin},
    ecs::world::CommandQueue,
    prelude::*,
};

use crate::transportation_ui::*;

const EPSILON: f32 = 0.000_1;

#[test]
fn registration_notice_requires_the_exact_location_to_be_newly_unlocked() {
    for (transportation_type, location_id, unlocked) in [
        (1, 13, unlocks_for_warp(13)),
        (2, 34, unlocks_for_wyvern(34)),
        (2, 64, unlocks_for_wyvern(64)),
        (2, 127, unlocks_for_wyvern(127)),
    ] {
        let intent = TransportationRegistrationIntent {
            transportation_type,
            npc_id: 9_001,
            location_id,
        };
        let empty = TransportationUnlocks::default();
        assert!(!intent.is_registered(empty));
        assert!(intent.became_registered(empty, unlocked));
        // Includes flags restored by world-ready on character re-entry.
        assert!(intent.is_registered(unlocked));
        assert!(!intent.became_registered(unlocked, unlocked));
        assert!(!intent.became_registered(empty, empty));
        let other_stop = TransportationRegistrationIntent {
            location_id: 2,
            ..intent
        };
        assert!(!other_stop.became_registered(empty, unlocked));
        let other_service = TransportationRegistrationIntent {
            transportation_type: if transportation_type == 1 { 2 } else { 1 },
            ..intent
        };
        assert!(!other_service.became_registered(empty, unlocked));
    }
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}

fn asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap()
}

fn clean_catalog() -> &'static TransportationCatalog {
    static CATALOG: OnceLock<TransportationCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let assets = AssetLocator::open(asset_root()).unwrap();
        TransportationCatalog::open(&assets).unwrap()
    })
}

fn unlocks_for_warp(end_location: i32) -> TransportationUnlocks {
    TransportationUnlocks {
        warp_location_flags: 1_u32 << (end_location - 1),
        wyvern_location_flags: [0, 0],
    }
}

fn unlocks_for_wyvern(end_location: i32) -> TransportationUnlocks {
    let mut flags = [0_u64; 2];
    if end_location > 63 {
        flags[1] = 1_u64 << (end_location - 64);
    } else {
        flags[0] = 1_u64 << (end_location - 1);
    }
    TransportationUnlocks {
        warp_location_flags: 0,
        wyvern_location_flags: flags,
    }
}

fn npc_context(
    npc_instance_id: i32,
    npc_table_id: i32,
    taros: i32,
    unlocks: TransportationUnlocks,
) -> TransportationOpenContext {
    TransportationOpenContext {
        player: TransportationPlayerSnapshot {
            position: TransportationWorldPoint::new(4_000.0, 120.0, 4_000.0),
            taros,
            unlocks,
            cursor_was_locked: true,
        },
        target: TransportationTarget::Npc {
            npc_instance_id,
            npc_table_id,
            npc_position: TransportationWorldPoint::new(4_100.0, 100.0, 4_100.0),
            has_move_ok_voice: true,
        },
    }
}

fn drain_outbox(model: &mut TransportationModel) -> Vec<TransportationOutboxEvent> {
    let mut events = Vec::new();
    while let Some(event) = model.pop_outbox() {
        events.push(event);
    }
    events
}

#[test]
fn normal_transport_close_marks_one_non_departure_edge() {
    let mut model = TransportationModel::default();
    let context = npc_context(9_004, 964, 193, unlocks_for_warp(2));
    model.open(clean_catalog(), context).unwrap();
    drain_outbox(&mut model);
    let gates = TransportationInputGates {
        system_popup_open: false,
        escape_close_allowed: true,
    };
    assert_eq!(model.close_button(gates), TransportationInputResult::Closed);
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::PlaySound(TransportationSound::Button),
            TransportationOutboxEvent::EndCameraSubTarget,
            TransportationOutboxEvent::RestoreCursorLocked(true),
            TransportationOutboxEvent::ExitMode {
                reason: TransportationCloseReason::UserClose,
            },
        ]
    );
    assert_eq!(model.escape(gates), TransportationInputResult::Ignored);
    assert!(drain_outbox(&mut model).is_empty());

    model.open(clean_catalog(), context).unwrap();
    drain_outbox(&mut model);
    assert_eq!(model.escape(gates), TransportationInputResult::Closed);
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::EndCameraSubTarget,
            TransportationOutboxEvent::RestoreCursorLocked(true),
            TransportationOutboxEvent::ExitMode {
                reason: TransportationCloseReason::UserClose,
            },
        ]
    );
}

#[test]
fn exact_mode_typo_shell_geometry_and_missing_zone_tabs_are_preserved() {
    assert_eq!(TRANSPOTATION_GAME_MODE_ID, 19);
    assert_eq!(TRANSPORTATION_MODE_NAME, "TransportMode");
    assert!(!TRANSPORTATION_HAS_ZONE_TABS);
    assert!(TRANSPORTATION_TYPE1_CAMERA_BINDING_OWNED_EXTERNALLY);
    assert_eq!(
        TRANSPORTATION_WINDOW_RECT,
        TransportationUiRect::new(0.0, 0.0, 1_036.0, 654.0)
    );
    assert_eq!(
        TRANSPORTATION_SELECT_RECT,
        TransportationUiRect::new(20.0, 148.0, 340.0, 445.0)
    );
    assert_eq!(
        TRANSPORTATION_MAP_RECT,
        TransportationUiRect::new(390.0, 10.0, 590.0, 630.0)
    );
    assert_eq!(
        TRANSPORTATION_GO_RECT,
        TransportationUiRect::new(213.0, 604.0, 132.0, 26.0)
    );
    assert_eq!(TRANSPORTATION_SCROLL_CONTENT_WIDTH, 322.0);
    assert_eq!(
        TRANSPORTATION_SCROLLBAR_RECT,
        TransportationUiRect::new(323.0, 12.0, 17.0, 421.0)
    );
    assert_eq!(
        TRANSPORTATION_SCROLL_THUMB_RECT,
        TransportationUiRect::new(324.0, 12.0, 15.0, 15.0)
    );
    assert_eq!(TRANSPORTATION_ROUTE_STRIDE, 82.0);
    assert_eq!(TRANSPORTATION_ICON_CONTENT_PADDING, 2.0);
    assert_eq!(TRANSPORTATION_JEFFE_14_FONT_SIZE, 12.0);
    assert_eq!(TRANSPORTATION_JEFFE_14_LINE_HEIGHT, 13.710_000_04);
    assert_eq!(TRANSPORTATION_JEFFE_16_FONT_SIZE, 14.0);
    assert_eq!(TRANSPORTATION_JEFFE_16_LINE_HEIGHT, 16.451_999_66);

    let font = Handle::<Font>::default();
    let exact_styles = [
        (
            TransportationUiTextStyle::BigFont14UpperLeft,
            "bigfont14",
            RETROBUTION_TRANSPORT_BIGFONT14_PATH_ID,
            0,
            [0.0; 4],
            false,
        ),
        (
            TransportationUiTextStyle::BigFont16UpperLeft,
            "bigfont16",
            RETROBUTION_TRANSPORT_BIGFONT16_PATH_ID,
            0,
            [0.0; 4],
            true,
        ),
        (
            TransportationUiTextStyle::RightTextUpperRight,
            "rightText",
            RETROBUTION_TRANSPORT_BIGFONT14_PATH_ID,
            2,
            [2.0; 4],
            true,
        ),
        (
            TransportationUiTextStyle::ButtonMiddleCenter,
            "button",
            RETROBUTION_TRANSPORT_BIGFONT14_PATH_ID,
            4,
            [6.0, 6.0, 3.0, 6.0],
            false,
        ),
    ];
    for (style, source_name, source_font, alignment, padding, word_wrap) in exact_styles {
        assert_eq!(style.source_style_name(), source_name);
        assert_eq!(
            style.source_skin_path_id(),
            RETROBUTION_TRANSPORT_SKIN_PATH_ID
        );
        assert_eq!(style.source_font_path_id(), source_font);
        assert_eq!(style.legacy_alignment(), alignment);
        assert_eq!(style.padding(), padding);
        assert_eq!(style.content_offset(), [0.0, 0.0]);
        assert_eq!(style.word_wrap(), word_wrap);
        assert_eq!(style.replacement_y_offset(), 0.0);
        let runtime_font = style.font(&font);
        assert_eq!(
            runtime_font.0.font_size.eval(Vec2::ZERO, 16.0),
            style.font_size()
        );
        assert_eq!(runtime_font.1, LineHeight::Px(style.line_height()));
    }

    let mut model = TransportationModel::default();
    model.open(clean_catalog(), npc_context(9_004, 964, 193, unlocks_for_warp(2))).unwrap();
    let endpoint = &model.routes()[0];
    let route = transportation_route_name_text(endpoint);
    assert_eq!(route.key, endpoint.name_localization_key);
    assert_eq!(route.fallback, endpoint.name);
    assert!(route.args.is_empty());
    let region = transportation_route_region_text("The Suburbs");
    assert_eq!(region.key, "content.transportation.region.suburbs");
    assert_eq!(region.fallback, "The Suburbs");
    assert!(region.args.is_empty());
    let subtitle = transportation_subtitle_text("Peach Creek Commons", "The Suburbs");
    assert_eq!(subtitle.key, "ui.transportation.subtitle");
    assert_eq!(subtitle.fallback, "{name} - {region}");
    assert_eq!(
        subtitle.args.get("name").map(String::as_str),
        Some("Peach Creek Commons")
    );
    assert_eq!(
        subtitle.args.get("region").map(String::as_str),
        Some("The Suburbs")
    );
    let empty = transportation_empty_subtitle_text();
    assert_eq!(empty.key, "ui.transportation.subtitle.empty");
    assert!(empty.fallback.is_empty());
    assert!(empty.args.is_empty());
    let cost = transportation_cost_text(125);
    assert_eq!(cost.key, "ui.transportation.cost");
    assert_eq!(cost.fallback, "{amount}");
    assert_eq!(cost.args.get("amount").map(String::as_str), Some("125"));
}

#[test]
fn clean_catalog_is_the_exact_manifest_owned_table_projection() {
    let catalog = clean_catalog();
    assert_eq!(catalog.provenance.bytes, RETROBUTION_NATIVE_TABLE_SET_BYTES);
    assert_eq!(
        catalog.provenance.sha256,
        RETROBUTION_NATIVE_TABLE_SET_SHA256
    );
    assert_eq!(catalog.provenance.schema, TABLE_SET_SCHEMA);
    assert_eq!(catalog.provenance.table_name, CONSOLIDATED_TABLE);
    assert_eq!(
        catalog.routes().len(),
        RETROBUTION_TRANSPORTATION_ROUTE_COUNT
    );
    assert_eq!(
        catalog.warp_locations().len(),
        RETROBUTION_TRANSPORTATION_WARP_LOCATION_COUNT
    );
    assert_eq!(
        catalog.broomstick_locations().len(),
        RETROBUTION_TRANSPORTATION_BROOM_LOCATION_COUNT
    );
    assert_eq!(catalog.npc_class(964), Some(15));
    assert_eq!(catalog.npc_class(984), Some(16));

    let warp = &catalog.warp_locations()[2];
    assert_eq!(warp.name, "Peach Creek Commons");
    assert_eq!(warp.region, "The Suburbs");
    assert_close(warp.position.x, 4_222.98);
    assert_close(warp.position.y, 3_785.89);
    let city_station = &catalog.broomstick_locations()[18];
    assert_eq!(city_station.name, "City Station");
    assert_eq!(city_station.region, "Downtown");
    let route = catalog
        .routes()
        .iter()
        .find(|route| route.npc_id == 964)
        .unwrap();
    assert_eq!(
        (
            route.vehicle_id,
            route.start_location,
            route.end_location,
            route.cost,
        ),
        (2, 1, 2, 193)
    );
}

#[test]
fn all_semantic_pngs_match_their_clean_pathid_proofs_and_set_digest() {
    assert_eq!(
        TRANSPORTATION_TEXTURE_PROOFS.len(),
        TRANSPORTATION_SEMANTIC_ASSET_FILES
    );
    let root = asset_root();
    let mut proofs = TRANSPORTATION_TEXTURE_PROOFS.to_vec();
    proofs.sort_by_key(|proof| proof.asset_path);
    let mut set_digest = Sha256::new();
    let mut total_bytes = 0_u64;
    for proof in proofs {
        assert!(proof.source_path_id > 0);
        let bytes = fs::read(root.join(proof.asset_path)).unwrap();
        assert_eq!(
            blake3::hash(&bytes).to_hex().to_string(),
            proof.source_blake3
        );
        assert_eq!(
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            proof.width
        );
        assert_eq!(
            u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
            proof.height
        );
        set_digest.update(proof.asset_path.as_bytes());
        set_digest.update([0]);
        set_digest.update((bytes.len() as u64).to_le_bytes());
        set_digest.update(&bytes);
        total_bytes += bytes.len() as u64;
    }
    assert_eq!(total_bytes, TRANSPORTATION_SEMANTIC_ASSET_BYTES);
    assert_eq!(
        format!("{:x}", set_digest.finalize()),
        TRANSPORTATION_SEMANTIC_ASSET_SET_SHA256
    );
    assert_eq!(
        blake3::hash(&fs::read(root.join(WORLD_MAP_JEFFE_FONT_PATH)).unwrap())
            .to_hex()
            .to_string(),
        RETROBUTION_TRANSPORT_JEFFE_FONT_BLAKE3
    );
    assert_eq!(
        blake3::hash(
            &fs::read(root.join(crate::world_map::WORLD_MAP_CHALET_FONT_PATH),).unwrap(),
        )
        .to_hex()
        .to_string(),
        RETROBUTION_TRANSPORT_CHALET_FONT_BLAKE3
    );
    assert_eq!(RETROBUTION_TRANSPORT_BIGFONT14_PATH_ID, 903);
    assert_eq!(RETROBUTION_TRANSPORT_BIGFONT16_PATH_ID, 1_012);
    assert_eq!(RETROBUTION_TRANSPORT_SKIN_DEFAULT_FONT_PATH_ID, 1_018);
}

#[test]
fn registration_bits_follow_the_exact_int_and_two_long_branches() {
    let unlocks = TransportationUnlocks {
        warp_location_flags: (1 << 0) | (1 << 12),
        wyvern_location_flags: [(1 << 1) | (1_u64 << 62), 1],
    };
    assert!(unlocks.warp_registered(1));
    assert!(unlocks.warp_registered(13));
    // C# masks integral shift counts; this is dormant for the clean
    // 14-row warp table but still part of the exact controller contract.
    assert!(unlocks.warp_registered(33));
    assert!(!unlocks.warp_registered(0));
    assert!(!unlocks.warp_registered(12));
    assert!(unlocks.wyvern_registered(2));
    assert!(unlocks.wyvern_registered(63));
    assert!(unlocks.wyvern_registered(64));
    assert!(unlocks.wyvern_registered(128));
    assert!(!unlocks.wyvern_registered(0));
    assert!(!unlocks.wyvern_registered(65));
}

#[test]
fn npcicon_registration_intents_and_replies_preserve_the_exact_0104_abi() {
    let catalog = clean_catalog();
    let scamp = catalog.registration_intent(9_001, 964).unwrap();
    assert_eq!(scamp.transportation_type, 1);
    assert_eq!(scamp.location_id, 1);
    let scamp_request = scamp.encode_registered().unwrap();
    assert_eq!(
        scamp_request.packet_type(),
        TRANSPORTATION_REGISTRATION_REQUEST_PACKET_ID
    );
    assert_eq!(
        scamp_request.payload(),
        &[1, 0, 0, 0, 41, 35, 0, 0, 1, 0, 0, 0]
    );

    let monkey = catalog.registration_intent(9_002, 984).unwrap();
    assert_eq!(monkey.transportation_type, 2);
    assert_eq!(monkey.location_id, 1);
    assert!(catalog.registration_intent(9_003, 664).is_none());

    let mut success = vec![0; 28];
    success[0..4].copy_from_slice(&2_i32.to_le_bytes());
    success[4..8].copy_from_slice(&29_i32.to_le_bytes());
    success[8..12].copy_from_slice(&(1_i32 << 12).to_le_bytes());
    success[12..20].copy_from_slice(&(1_u64 << 62).to_le_bytes());
    success[20..28].copy_from_slice(&(1_u64 << 17).to_le_bytes());
    assert_eq!(
        decode_transportation_registration_reply_0104(
            TRANSPORTATION_REGISTRATION_SUCCESS_PACKET_ID,
            &success,
        ),
        Ok(TransportationRegistrationReply0104::Success {
            transportation_type: 2,
            location_id: 29,
            unlocks: TransportationUnlocks {
                warp_location_flags: 1 << 12,
                wyvern_location_flags: [1_u64 << 62, 1_u64 << 17],
            },
        })
    );

    let mut failure = vec![0; 12];
    failure[0..4].copy_from_slice(&1_i32.to_le_bytes());
    failure[4..8].copy_from_slice(&13_i32.to_le_bytes());
    failure[8..12].copy_from_slice(&7_i32.to_le_bytes());
    assert_eq!(
        decode_transportation_registration_reply_0104(
            TRANSPORTATION_REGISTRATION_FAILURE_PACKET_ID,
            &failure,
        ),
        Ok(TransportationRegistrationReply0104::Failure {
            transportation_type: 1,
            location_id: 13,
            error_code: 7,
        })
    );
}

#[test]
fn warp_request_waits_strictly_longer_than_one_point_five_and_never_mutates_player() {
    let mut model = TransportationModel::default();
    model
        .open(
            clean_catalog(),
            npc_context(9_001, 964, 193, unlocks_for_warp(2)),
        )
        .unwrap();
    assert_eq!(model.service(), TransportationService::Warp);
    assert_eq!(model.routes().len(), 1);
    assert!(model.routes()[0].registered);
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::SetCursorLocked(false),
            TransportationOutboxEvent::SetTransportCameraTarget(9_001),
        ]
    );
    assert_eq!(
        model.select_route(0),
        Ok(TransportationInputResult::Changed)
    );
    assert_eq!(model.press_go(), Ok(TransportationInputResult::BeganWarp));
    assert_eq!(model.phase(), TransportationPhase::PendingWarp);
    assert_eq!(model.player_taros(), Some(193));
    assert_eq!(
        model.player_position(),
        Some(TransportationWorldPoint::new(4_000.0, 120.0, 4_000.0))
    );
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::PlaySound(TransportationSound::TransportationWarp,),
            TransportationOutboxEvent::SetGameConditionCooldown(14),
            TransportationOutboxEvent::EndCameraSubTarget,
            TransportationOutboxEvent::InstantiatePlayerEffect(394),
            TransportationOutboxEvent::PlayNpcMoveOkVoice(9_001),
            TransportationOutboxEvent::PlaySound(TransportationSound::TransportationWarp,),
        ]
    );

    assert_eq!(
        model.advance(TRANSPORTATION_WARP_DELAY_SECONDS),
        Ok(TransportationInputResult::Ignored)
    );
    assert_eq!(model.pop_travel_intent(), None);
    assert_eq!(model.advance(0.001), Ok(TransportationInputResult::Changed));
    assert_eq!(
        model.pop_travel_intent(),
        Some(TransportationTravelIntent {
            npc_id: 9_001,
            transporation_id: 2,
            e_il: 4,
            slot_number: 0,
            turbo: false,
        })
    );
    assert_eq!(model.phase(), TransportationPhase::AwaitingServer);
    assert_eq!(model.player_taros(), Some(193));
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::SetMovementPacketEmission(false),
            TransportationOutboxEvent::Fade(TransportationFade::WindowIn),
        ]
    );
}

#[test]
fn turbo_is_wyvern_only_doubles_cost_and_uses_the_same_authoritative_taros_snapshot() {
    let mut model = TransportationModel::default();
    model
        .open(
            clean_catalog(),
            npc_context(9_002, 984, 385, unlocks_for_wyvern(2)),
        )
        .unwrap();
    drain_outbox(&mut model);
    assert_eq!(model.service(), TransportationService::Wyvern);
    assert_eq!(model.route_effective_cost(0), Some(193));
    assert_eq!(model.toggle_turbo(), Ok(TransportationInputResult::Changed));
    assert_eq!(model.route_effective_cost(0), Some(386));
    assert_eq!(model.route_affordable(0), Some(false));
    assert_eq!(model.player_taros(), Some(385));
    drain_outbox(&mut model);
    model.select_route(0).unwrap();
    assert_eq!(model.press_go(), Ok(TransportationInputResult::Ignored));
    assert_eq!(model.phase(), TransportationPhase::Browsing);
    assert_eq!(model.player_taros(), Some(385));
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::PlaySound(TransportationSound::TransportationWarp,),
            TransportationOutboxEvent::SystemMessage(
                TRANSPORTATION_INSUFFICIENT_TAROS_MESSAGE_ID,
            ),
        ]
    );

    let mut warp = TransportationModel::default();
    warp.open(
        clean_catalog(),
        npc_context(9_003, 964, 10_000, unlocks_for_warp(2)),
    )
    .unwrap();
    drain_outbox(&mut warp);
    assert_eq!(warp.toggle_turbo(), Ok(TransportationInputResult::Ignored));
    assert!(!warp.turbo());
    assert!(drain_outbox(&mut warp).is_empty());
}

#[test]
fn transportation_wire_codecs_are_exact_and_registry_checked() {
    let request = TransportationTravelIntent {
        npc_id: 9_001,
        transporation_id: 2,
        e_il: 4,
        slot_number: 7,
        turbo: true,
    }
    .encode_registered()
    .unwrap();
    assert_eq!(request.packet_type(), TRANSPORTATION_REQUEST_PACKET_ID);
    assert_eq!(request.payload().len(), 16);
    assert_eq!(&request.payload()[0..4], &9_001i32.to_le_bytes());
    assert_eq!(&request.payload()[12..16], &7i32.to_le_bytes());

    let mut success = vec![0; 20];
    for (offset, value) in [3i32, 100, 200, 300, 450].into_iter().enumerate() {
        success[offset * 4..offset * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    assert_eq!(
        decode_transportation_warp_reply_0104(TRANSPORTATION_SUCCESS_PACKET_ID, &success),
        Ok(TransportationWarpReply0104::Success {
            transportation_type: 3,
            position: [100, 200, 300],
            candy: 450,
        })
    );
    assert!(matches!(
        decode_transportation_warp_reply_0104(TRANSPORTATION_FAILURE_PACKET_ID, &[0; 7]),
        Err(TransportationReplyCodecError0104::WrongSize { .. })
    ));
}

#[test]
fn item_use_projects_zone_two_and_three_and_sends_location_not_vehicle() {
    let context = TransportationOpenContext {
        player: TransportationPlayerSnapshot {
            position: TransportationWorldPoint::new(6_000.0, 42.0, 1_000.0),
            taros: 0,
            unlocks: TransportationUnlocks {
                warp_location_flags: 0,
                wyvern_location_flags: [u64::MAX, u64::MAX],
            },
            cursor_was_locked: false,
        },
        target: TransportationTarget::ItemUse {
            e_il: 2,
            slot_number: 7,
        },
    };
    let mut model = TransportationModel::default();
    model.open(clean_catalog(), context).unwrap();
    assert_eq!(model.service(), TransportationService::ItemUse);
    assert_eq!(model.routes().len(), 28);
    assert_eq!(
        model.start_position(),
        TransportationUiPoint::new(6_000.0, 42.0)
    );
    assert!(model.is_future());
    assert_eq!(model.map(), TransportationMap::Future);
    assert_eq!(
        drain_outbox(&mut model),
        vec![TransportationOutboxEvent::SetCursorLocked(false)]
    );
    let dark_route = model
        .routes()
        .iter()
        .position(|route| route.definition.end_location == 26)
        .unwrap();
    model.select_route(dark_route).unwrap();
    assert_eq!(model.map(), TransportationMap::Darklands);
    assert!(model.is_future());
    assert_eq!(model.route_effective_cost(dark_route), Some(0));
    assert!(model.target_view().x < TRANSPORTATION_FUTURE_ZONE_RECT.x / 8_192.0);

    model.press_go().unwrap();
    drain_outbox(&mut model);
    model
        .advance(TRANSPORTATION_WARP_DELAY_SECONDS + 0.001)
        .unwrap();
    assert_eq!(
        model.pop_travel_intent(),
        Some(TransportationTravelIntent {
            npc_id: 0,
            transporation_id: 26,
            e_il: 2,
            slot_number: 7,
            turbo: false,
        })
    );
    assert_eq!(model.player_taros(), Some(0));
    assert_eq!(model.player_position(), Some(context.player.position));
}

#[test]
fn go_feedback_failure_mapping_and_close_gates_match_clean_cntrans() {
    let mut model = TransportationModel::default();
    model
        .open(
            clean_catalog(),
            npc_context(9_004, 964, 193, unlocks_for_warp(2)),
        )
        .unwrap();
    drain_outbox(&mut model);
    assert_eq!(model.press_go(), Ok(TransportationInputResult::Ignored));
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::SystemMessage(TRANSPORTATION_NO_SELECTION_MESSAGE_ID,),
            TransportationOutboxEvent::PlaySound(TransportationSound::ActionFailure,),
        ]
    );
    assert_eq!(
        model.escape(TransportationInputGates {
            system_popup_open: false,
            escape_close_allowed: false,
        }),
        TransportationInputResult::Ignored
    );
    assert_eq!(
        model.close_button(TransportationInputGates {
            system_popup_open: true,
            escape_close_allowed: true,
        }),
        TransportationInputResult::Ignored
    );

    model.select_route(0).unwrap();
    model.press_go().unwrap();
    drain_outbox(&mut model);
    model.advance(1.501).unwrap();
    drain_outbox(&mut model);
    model.receive_failure(8);
    assert_eq!(model.phase(), TransportationPhase::AwaitingServer);
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::SystemMessage(TRANSPORTATION_FAILURE_MESSAGE_ID_8,),
            TransportationOutboxEvent::MarkPacketHandled(TRANSPORTATION_FAILURE_PACKET_ID,),
        ]
    );
    model.receive_failure(999);
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::SystemMessage(TRANSPORTATION_GENERIC_FAILURE_MESSAGE_ID,),
            TransportationOutboxEvent::MarkPacketHandled(TRANSPORTATION_FAILURE_PACKET_ID,),
        ]
    );
    assert_eq!(model.receive_success(), TransportationInputResult::Closed);
    assert_eq!(model.phase(), TransportationPhase::Hidden);
    assert_eq!(
        drain_outbox(&mut model),
        vec![
            TransportationOutboxEvent::EndCameraSubTarget,
            TransportationOutboxEvent::RestoreCursorLocked(true),
            TransportationOutboxEvent::ExitMode {
                reason: TransportationCloseReason::Departure,
            },
            TransportationOutboxEvent::MarkPacketHandled(TRANSPORTATION_SUCCESS_PACKET_ID,),
            TransportationOutboxEvent::PlaySound(TransportationSound::ActionSuccess,),
        ]
    );
}

#[test]
fn map_projection_uses_exact_smoothing_marker_size_and_dark_filter() {
    let mut model = TransportationModel::default();
    model
        .open(
            clean_catalog(),
            npc_context(
                9_005,
                965,
                10_000,
                TransportationUnlocks {
                    warp_location_flags: u32::MAX,
                    wyvern_location_flags: [0, 0],
                },
            ),
        )
        .unwrap();
    drain_outbox(&mut model);
    let previous = model.display_view();
    model.select_route(0).unwrap();
    let target = model.target_view();
    model.advance_map_paint().unwrap();
    assert_close(model.display_view().x, target.x * 0.2 + previous.x * 0.8);
    assert_close(model.display_view().y, target.y * 0.2 + previous.y * 0.8);
    assert!(
        model
            .marker_projections()
            .iter()
            .all(|marker| marker.rect.width == 36.0 && marker.rect.height == 36.0)
    );
}

fn transportation_template_args(template: &str) -> BTreeSet<&str> {
    let mut output = BTreeSet::new();
    let mut remaining = template;
    while let Some(open) = remaining.find('{') {
        remaining = &remaining[open + 1..];
        let Some(close) = remaining.find('}') else {
            break;
        };
        output.insert(&remaining[..close]);
        remaining = &remaining[close + 1..];
    }
    output
}

#[test]
fn production_bundles_publish_every_reached_transportation_key_exactly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut bundles = BTreeMap::new();
    for locale in ["en", "ru"] {
        let runtime = fs::read(
            root.join("assets/game/localization")
                .join(format!("{locale}.json")),
        )
        .unwrap();
        let value: Value = serde_json::from_slice(&runtime).unwrap();
        let entries = value["entries"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    value
                        .as_str()
                        .expect("string localization value")
                        .to_owned(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        bundles.insert(locale, entries);
    }
    assert_eq!(
        bundles["en"].keys().collect::<BTreeSet<_>>(),
        bundles["ru"].keys().collect::<BTreeSet<_>>()
    );
    let exact = [
        ("ui.transportation.title", "TRANSPORTATION", "ТРАНСПОРТ"),
        ("ui.transportation.where_to", "WHERE TO?", "КУДА?"),
        (
            "ui.transportation.unregistered",
            "Unregistered",
            "Не открыто",
        ),
        (
            "ui.transportation.turbo_travel",
            "TURBO TRAVEL",
            "ТУРБО-МАРШРУТ",
        ),
        ("ui.transportation.go_now", "GO NOW!", "В ПУТЬ!"),
        ("ui.transportation.cost", "{amount}", "{amount}"),
        ("ui.transportation.subtitle.empty", "", ""),
        (
            "ui.transportation.subtitle",
            "{name} - {region}",
            "{name} - {region}",
        ),
        ("ui.transportation.route.name", "{name}", "{name}"),
        ("ui.transportation.route.region", "{region}", "{region}"),
    ];
    for (key, en, ru) in exact {
        assert_eq!(bundles["en"].get(key).map(String::as_str), Some(en));
        assert_eq!(bundles["ru"].get(key).map(String::as_str), Some(ru));
        assert_eq!(
            transportation_template_args(en),
            transportation_template_args(ru),
            "placeholder mismatch for {key}"
        );
    }
}

#[test]
fn every_dynamic_route_text_is_semantic_key_first_and_typed() {
    let mut model = TransportationModel::default();
    model
        .open(
            clean_catalog(),
            npc_context(9_002, 984, 10_000, unlocks_for_wyvern(2)),
        )
        .unwrap();
    drain_outbox(&mut model);
    model.select_route(0).unwrap();

    let mut world = World::new();
    let layer = world.spawn_empty().id();
    let assets = TransportationPresentationAssets {
        images: transportation_presentation_asset_paths()
            .into_iter()
            .map(|path| (path, Handle::default()))
            .collect(),
        jeffe: Handle::default(),
    };
    let copy = TransportationUiCopy::default();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        for (route_index, route) in model.routes().iter().enumerate() {
            spawn_transportation_route(
                &mut commands,
                layer,
                route_index,
                route,
                &model,
                &copy,
                &assets,
            );
        }
    }
    queue.apply(&mut world);

    let text_entities = {
        let mut query = world.query_filtered::<Entity, With<Text>>();
        query.iter(&world).collect::<Vec<_>>()
    };
    assert_eq!(text_entities.len(), model.routes().len() * 3);
    for entity in text_entities {
        let localized = world
            .get::<LocalizedText>(entity)
            .unwrap_or_else(|| panic!("dynamic Text is not key-first: {entity:?}"));
        let style = world
            .get::<TransportationUiTextStyle>(entity)
            .unwrap_or_else(|| panic!("dynamic Text has no source GUIStyle: {entity:?}"));
        let transform = world
            .get::<UiTransform>(entity)
            .unwrap_or_else(|| panic!("dynamic Text has no font offset: {entity:?}"));
        assert_ne!(localized.key, "ui.content.passthrough", "{entity:?}");
        assert_eq!(
            transform.translation,
            Val2::px(0.0, style.replacement_y_offset()),
            "{entity:?}"
        );
        let expected_args: &[&str] = match localized.key.as_str() {
            "ui.transportation.route.name" => &["name"],
            "ui.transportation.route.region" => &["region"],
            "ui.transportation.cost" => &["amount"],
            _ => &[],
        };
        assert!(
            localized
                .args
                .keys()
                .map(String::as_str)
                .eq(expected_args.iter().copied()),
            "unexpected key/args on {entity:?}: {} {:?}",
            localized.key,
            localized.args
        );
    }
}

#[test]
fn transportation_source_has_no_dynamic_passthrough_or_raw_text_mutation() {
    let source = concat!(
        include_str!("state.rs"),
        "\n",
        include_str!("constants.rs"),
        "\n",
        include_str!("codec.rs"),
        "\n",
        include_str!("assets_transportation_catalog.rs"),
        "\n",
        include_str!("layout.rs"),
        "\n",
        include_str!("interaction.rs"),
        "\n",
        include_str!("types.rs"),
        "\n",
        include_str!("textures.rs"),
        "\n",
        include_str!("operations.rs"),
        "\n",
        include_str!("validation.rs"),
        "\n",
        include_str!("containers.rs"),
        "\n",
        include_str!("audio.rs"),
        "\n",
        include_str!("commands.rs"),
        "\n",
        include_str!("models.rs"),
        "\n",
        include_str!("output.rs"),
        "\n",
        include_str!("animation.rs"),
        "\n",
        include_str!("view.rs"),
        "\n",
        include_str!("systems.rs"),
        "\n",
        include_str!("mod.rs")
    );
    let production = source.split_once("#[cfg(test)]").unwrap().0;
    assert!(!production.contains("ui.content.passthrough"));
    assert!(!production.contains("&mut Text,"));
    assert!(!production.contains("Mut<Text>"));
}

#[test]
fn presentation_plugin_spawns_hidden_fail_closed_tree_and_typed_outbox() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(AssetPlugin {
        file_path: asset_root().to_string_lossy().into_owned(),
        ..default()
    });
    app.init_resource::<ButtonInput<KeyCode>>()
        .add_message::<MouseWheel>()
        .add_plugins(TransportationUiPlugin);
    app.init_asset::<Image>().init_asset::<Font>();
    app.update();

    let mut roots = app.world_mut().query_filtered::<
        (&Node, &Pickable, &GlobalZIndex),
        With<TransportationPresentationRoot>,
    >();
    let (root, pickable, z_index) = roots.single(app.world()).unwrap();
    assert_eq!(root.display, Display::None);
    assert!(pickable.should_block_lower);
    assert!(!pickable.is_hoverable);
    assert_eq!(*z_index, GlobalZIndex(TRANSPORTATION_UI_Z_INDEX));
    assert_eq!(
        app.world()
            .resource::<TransportationPresentationAssets>()
            .images
            .len(),
        transportation_presentation_asset_paths().len()
    );
    assert!(
        app.world()
            .resource::<TransportationUiCommandOutbox>()
            .is_empty()
    );

    let text_entities = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Text>>();
        query.iter(world).collect::<Vec<_>>()
    };
    assert_eq!(text_entities.len(), 5, "the static shell owns five labels");
    for entity in text_entities {
        let world = app.world();
        let localized = world.get::<LocalizedText>(entity).unwrap_or_else(|| {
            panic!("every static Transportation label must be key-first: {entity:?}")
        });
        let style = world
            .get::<TransportationUiTextStyle>(entity)
            .unwrap_or_else(|| panic!("every Text must carry its source GUIStyle: {entity:?}"));
        let transform = world
            .get::<UiTransform>(entity)
            .unwrap_or_else(|| panic!("every Text must carry a font offset: {entity:?}"));
        assert!(!localized.key.trim().is_empty());
        assert_eq!(
            transform.translation,
            Val2::px(0.0, style.replacement_y_offset())
        );
    }
}

#[test]
fn transport_endpoint_keys_resolve_russian_for_both_service_tables() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, ru) = crate::localization::Localization::open(&root, "ru").unwrap();
    let (_, en) = crate::localization::Localization::open(&root, "en").unwrap();
    let catalog = clean_catalog();
    for npc_type in [984, 964] {
        let mut model = TransportationModel::default();
        model.open(&catalog, npc_context(9_002, npc_type, 10_000, if npc_type == 964 { unlocks_for_warp(2) } else { unlocks_for_wyvern(2) })).unwrap();
        for route in model.routes() {
            let name = super::transportation_route_name_text(route);
            assert_eq!(localization.text(&en, &name), route.name);
            assert_ne!(localization.text(&ru, &name), route.name, "{}", name.key);
        }
    }
    assert_eq!(localization.text(&ru, &super::transportation_route_region_text("The Suburbs")), "Пригород");
    let future = super::transportation_route_region_text("The Future");
    assert_eq!(localization.text(&en, &future), "The Future");
    assert_eq!(localization.text(&ru, &future), "Будущее");
}
