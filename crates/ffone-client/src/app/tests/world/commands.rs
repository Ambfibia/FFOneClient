use super::*;

#[test]
fn local_attack_success_is_a_combat_event_even_without_damage_results() {
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_ATTACK_NPCS_SUCC,
        flags: 0,
        checksum: 0,
        payload: PcAttackNpcsSuccess0104 {
            battery_w: 90,
            results: Vec::new(),
        }
        .encode()
        .unwrap(),
    };
    assert!(frame_observes_local_combat(&frame, 41));
}

#[test]
fn gm_speed_command_requires_one_exact_i32_argument() {
    assert_eq!(
        parse_gm_speed_command("/speed 1200"),
        Some(GmSpeedCommand::Set(1_200))
    );
    assert_eq!(
        parse_gm_speed_command("  /speed -1  "),
        Some(GmSpeedCommand::Set(-1))
    );
    assert_eq!(
        parse_gm_speed_command("/speed"),
        Some(GmSpeedCommand::Invalid)
    );
    assert_eq!(
        parse_gm_speed_command("/speed fast"),
        Some(GmSpeedCommand::Invalid)
    );
    assert_eq!(
        parse_gm_speed_command("/speed 600 extra"),
        Some(GmSpeedCommand::Invalid)
    );
    assert_eq!(parse_gm_speed_command("/Speed 1200"), None);
    assert_eq!(parse_gm_speed_command("/speedrun 1200"), None);

    let command = GmSpeedCommand::Set(1_200);
    assert_eq!(
        build_gm_speed_request(command, 50, Some(81)),
        Ok(GmSetValueRequest0104::speed(81, 1_200))
    );
    assert_eq!(
        build_gm_speed_request(command, 51, Some(81)),
        Err(GmSpeedRequestError::AccessDenied)
    );
    assert_eq!(
        build_gm_speed_request(command, 30, None),
        Err(GmSpeedRequestError::MissingPlayer)
    );
    assert_eq!(
        build_gm_speed_request(GmSpeedCommand::Invalid, 30, Some(81)),
        Err(GmSpeedRequestError::Usage)
    );
}

#[test]
fn gm_nano_command_builds_the_exact_registered_give_nano_request() {
    assert_eq!(
        parse_gm_nano_command("/nano 36"),
        Some(GmNanoCommand::Give(36))
    );
    assert_eq!(
        parse_gm_nano_command("  /nano 1  "),
        Some(GmNanoCommand::Give(1))
    );
    assert_eq!(parse_gm_nano_command("/nano"), Some(GmNanoCommand::Invalid));
    assert_eq!(
        parse_gm_nano_command("/nano 0"),
        Some(GmNanoCommand::Invalid)
    );
    assert_eq!(
        parse_gm_nano_command("/nano 1 extra"),
        Some(GmNanoCommand::Invalid)
    );
    assert_eq!(parse_gm_nano_command("/Nano 1"), None);
    assert_eq!(parse_gm_nano_command("/nanobot 1"), None);

    let request = build_gm_nano_request(GmNanoCommand::Give(36), 50, |id| id == 36).unwrap();
    assert_eq!(request.nano_id, 36);
    assert_eq!(request.encode(), 36_i16.to_le_bytes());
    let registered = ffone_protocol::RegisteredGameplayRequest0104::new(
        packet::P_CL2FE_REQ_PC_GIVE_NANO,
        request.encode(),
    )
    .unwrap();
    assert_eq!(registered.packet_type(), 0x1300_0044);
    assert_eq!(registered.payload(), 36_i16.to_le_bytes());

    assert_eq!(
        build_gm_nano_request(GmNanoCommand::Give(36), 51, |_| true),
        Err(GmNanoRequestError::AccessDenied)
    );
    assert_eq!(
        build_gm_nano_request(GmNanoCommand::Give(37), 30, |_| false),
        Err(GmNanoRequestError::UnknownNano(37))
    );
    assert_eq!(
        build_gm_nano_request(GmNanoCommand::Invalid, 30, |_| true),
        Err(GmNanoRequestError::Usage)
    );
}

#[test]
fn world_map_present_reply_requires_clock_and_applies_transactionally() {
    let mut presentation = WorldMapPresentation::default();
    let reply = PresentNpcTypesReply0104 {
        clear: 1,
        npc_types: vec![664, 707],
    };
    assert!(
        apply_world_map_present_reply(&mut presentation, WorldMapServerClock::default(), &reply,)
            .unwrap_err()
            .contains("independently observed server time")
    );
    assert!(presentation.model.present_npc_types().is_empty());
    assert_eq!(presentation.model.last_npc_type_sync_time(), 0);

    let clock = WorldMapServerClock {
        last_observed_server_time: Some(91),
    };
    apply_world_map_present_reply(&mut presentation, clock, &reply).unwrap();
    assert_eq!(
        presentation.model.present_npc_types(),
        &BTreeSet::from([664, 707])
    );
    assert_eq!(presentation.model.last_npc_type_sync_time(), 91);

    let duplicate = PresentNpcTypesReply0104 {
        clear: 0,
        npc_types: vec![664],
    };
    assert!(
        apply_world_map_present_reply(
            &mut presentation,
            WorldMapServerClock {
                last_observed_server_time: Some(92),
            },
            &duplicate,
        )
        .unwrap_err()
        .contains("DuplicatePresentNpcType")
    );
    assert_eq!(
        presentation.model.present_npc_types(),
        &BTreeSet::from([664, 707])
    );
    assert_eq!(presentation.model.last_npc_type_sync_time(), 91);
}

#[test]
fn applying_non_window_options_does_not_request_a_window_resize() {
    let current = OptionSettings::default();
    let mut next = current.clone();
    next.sound.voice.volume = 0.8;
    next.display.game_hint = !next.display.game_hint;
    assert!(!option_window_mode_changed(&current, &next));

    next.graphics.width = 1_280;
    next.graphics.height = 720;
    assert!(option_window_mode_changed(&current, &next));

    next = current.clone();
    next.graphics.windowed = !next.graphics.windowed;
    assert!(option_window_mode_changed(&current, &next));
}

#[test]
fn normal_world_nano_activation_commits_only_a_correlated_validated_reply() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            nano_slots: [
                RuntimeNanoSlot {
                    nano_id: Some(1),
                    skill_id: 1,
                    stamina: 80,
                    active: true,
                },
                RuntimeNanoSlot {
                    nano_id: Some(2),
                    skill_id: 7,
                    stamina: 90,
                    active: false,
                },
                RuntimeNanoSlot::default(),
            ],
            pending_nano_activation: Some(1),
            ..default()
        },
        ..default()
    };
    let success = NanoActiveSuccess0104 {
        active_nano_slot: 1,
        pack_padding: [0, 0],
        condition_status_add: 0,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_ACTIVE_SUCC,
        flags: 0,
        checksum: 0,
        payload: success.encode(),
    };
    let mut inbox = WorldNanoAuthorityInbox0104::default();
    let mut skill_buffs = SkillBuffUiModel::default();

    assert_eq!(
        apply_world_nano_response_frame(&frame, &mut runtime, &mut skill_buffs, &mut inbox,),
        Ok(true)
    );
    assert!(inbox.local_movements.is_empty());
    assert_eq!(
        runtime.nano_slots.map(|slot| slot.active),
        [false, true, false]
    );
    assert_eq!(runtime.pending_nano_activation, None);

    let committed = runtime.nano_slots;
    assert!(apply_world_nano_active_success(success, &mut runtime).is_err());
    assert_eq!(runtime.nano_slots, committed);
    assert_eq!(runtime.pending_nano_activation, None);

    runtime.pending_nano_activation = Some(0);
    assert!(
        apply_world_nano_active_success(
            NanoActiveSuccess0104 {
                active_nano_slot: 3,
                ..success
            },
            &mut runtime,
        )
        .is_err()
    );
    assert_eq!(runtime.nano_slots, committed);
    assert_eq!(runtime.pending_nano_activation, Some(0));

    apply_world_nano_active_success(
        NanoActiveSuccess0104 {
            active_nano_slot: 0,
            condition_status_add: 128,
            ..success
        },
        &mut runtime,
    )
    .unwrap();
    assert_eq!(
        runtime.pending_passive_nano_voice,
        Some(TutorialNanoGameplayLoadout {
            nano_id: 1,
            skill_id: 1,
        })
    );

    apply_world_nano_active_success(
        NanoActiveSuccess0104 {
            active_nano_slot: -1,
            ..success
        },
        &mut runtime,
    )
    .unwrap();
    assert_eq!(runtime.nano_slots.map(|slot| slot.active), [false; 3]);
    assert_eq!(runtime.pending_passive_nano_voice, None);
    assert_eq!(runtime.pending_nano_activation, None);

    runtime.pending_nano_activation = Some(2);
    runtime.clear_world();
    assert_eq!(runtime.pending_nano_activation, None);
}

#[test]
fn mentor_reply_is_transactional_and_openfusion_count_one_remains_a_later_change() {
    let (mut guide_runtime, mut model, mut outbox) =
        pending_guide_change(GuideMentor::Dexter, GuideMentor::Edd);
    let reply = PcChangeMentorSuccess0104 {
        mentor: GuideMentor::Edd.wire_id(),
        mentor_count: 1,
        fusion_matter: 321,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_CHANGE_MENTOR_SUCC,
        flags: 0,
        checksum: 0,
        payload: reply.encode(),
    };

    assert_eq!(
        apply_guide_mentor_reply_transactional(&frame, &mut guide_runtime, &mut model, &mut outbox,),
        Ok(Some(GuideMentorReplyOutcome::Success {
            mentor: GuideMentor::Edd,
            raw_mentor_count: 1,
            fusion_matter: 321,
            intent: GuidePostChangeIntent::RefreshGuideMissions,
        }))
    );
    assert!(!model.visible);
    assert_eq!(
        outbox.pop_front(),
        Some(GuideUiAction::MentorChangeSucceeded {
            mentor: GuideMentor::Edd,
            mentor_count: 1,
            fusion_matter: 321,
            refresh_guide_missions: true,
        })
    );
    let content = runtime_test_mission_content();
    let mut messages = NanocomMessageUiModel::default();
    assert!(enqueue_login_guide_nanocom_0104(
        &EmailProductionCatalog0104 {
            items: BTreeMap::new(),
            guide_rows: Vec::new()
        },
        &content,
        &WorldMissionRuntime::default(),
        &guide_runtime,
        &NanoFreeTuningBank0104::default(),
        &LocalInventoryRuntime::default(),
        10,
        &mut messages,
    ));
    let request = &messages.active().unwrap().request;
    let definition = content
        .gameplay_guide_nanocom(GuideMentor::Edd.wire_id())
        .unwrap();
    assert_eq!(
        request.compact_title_localized().key,
        format!("content.npc.{}.name", definition.npc_type)
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let voice = request
        .voice_true_name
        .as_ref()
        .expect("changed guide requests speech");
    let matches = catalog.by_true_name(voice);
    assert!(!matches.is_empty(), "{voice}");
    for locale in ["en", "ru"] {
        assert!(
            matches
                .iter()
                .any(|asset| catalog.path_for_locale(asset, locale).is_some())
        );
    }
}

#[test]
fn authoritative_warp_reply_decoder_requires_exact_0104_abi() {
    let target = runtime_test_mission_content()
        .gameplay_warp(GUIDE_FIRST_WARP_ID)
        .unwrap()
        .target;
    let success = PcWarpUseNpcSuccess0104 {
        position: [target.x, target.y, target.z],
        e_il: 4,
        item_slot_num: 0,
        item: ffone_protocol::ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        candy: 88,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_WARP_USE_NPC_SUCC,
        flags: 0,
        checksum: 0,
        payload: success.encode(),
    };
    assert_eq!(
        decode_authoritative_warp_reply_0104(&frame),
        Ok(Some(AuthoritativeWarpReply0104::NpcSuccess(success)))
    );
    let malformed = DecodedFrame {
        payload: vec![0; PcWarpUseNpcSuccess0104::SIZE + 1],
        ..frame
    };
    assert!(decode_authoritative_warp_reply_0104(&malformed).is_err());

    let goto = PcGotoSuccess0104 {
        position: [target.x, target.y, target.z],
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_GOTO_SUCC,
        flags: 0,
        checksum: 0,
        payload: goto.encode(),
    };
    assert_eq!(
        decode_authoritative_warp_reply_0104(&frame),
        Ok(Some(AuthoritativeWarpReply0104::GotoSuccess(goto)))
    );
    let malformed = DecodedFrame {
        payload: vec![0; PcGotoSuccess0104::SIZE - 1],
        ..frame
    };
    assert!(decode_authoritative_warp_reply_0104(&malformed).is_err());
}

#[test]
fn completed_mission_barker_request_uses_terminal_task_and_npc_barker_type() {
    let content = runtime_test_mission_content();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    // Quest flag bit zero is mission 1; bit two therefore restores clean
    // completed mission 3, whose terminal task is task 2.
    load.as_bytes_mut()[ffone_protocol::PcLoadData0104::QUEST_FLAGS_OFFSET
        ..ffone_protocol::PcLoadData0104::QUEST_FLAGS_OFFSET + 8]
        .copy_from_slice(&(1_i64 << 2).to_le_bytes());
    let mut mission = WorldMissionRuntime::default();
    mission.seed(&load, &content).unwrap();
    let nearby = [WorldMissionNearbyNpc {
        npc_type: 651,
        npc_id: 90210,
        position: [1.0, 0.0, 0.0],
        sight_range_server_units: 700,
    }];
    let mut random = LegacyNanoStandRandomStream::with_seed(7);

    let request =
        clean_world_mission_barker_request(&mission, &content, [0.0; 3], &nearby, &mut random)
            .expect("task 2 Barker type 1 line should produce a request");
    assert_eq!(request.mission_task_id, 2);
    assert_eq!(request.npc_id, 90210);
    assert_eq!(
        content.mission(2).unwrap().provenance.barker_text_ids,
        [12, 13, 14, 15]
    );
    assert_eq!(content.gameplay_npc_barker_type(651), Some(1));
    assert_eq!(random.draw_count(), 1);
}

#[test]
fn forged_normal_npc_warp_action_clears_the_ui_pending_latch() {
    let warp = WarpUiEntry {
        npc_id: 91,
        npc_type: 681,
        warp_id: 5,
        required_task_id: Some(7),
        target: TutorialWarpTarget {
            map_id: 14,
            x: 100,
            y: 200,
            z: 300,
        },
        label: "WARP".to_owned(),
    };
    let exact = NormalNpcWarpIdentity {
        npc_id: warp.npc_id,
        npc_type: warp.npc_type,
        warp_id: warp.warp_id,
        required_task_id: warp.required_task_id,
        target: warp.target,
    };
    let mut model = MissionUiModel::default();
    model.enabled = true;
    model.npc_icon_mode_visible = false;
    model.npc_interaction = Some(NpcInteractionUi {
        npc_id: warp.npc_id,
        warp: Some(warp.clone()),
        ..default()
    });
    model.pending_warp = Some(warp);

    assert!(validate_normal_npc_warp_ui_action(&mut model, exact));
    assert!(model.pending_warp.is_some());
    assert!(!validate_normal_npc_warp_ui_action(
        &mut model,
        NormalNpcWarpIdentity {
            warp_id: exact.warp_id + 1,
            ..exact
        }
    ));
    assert!(model.pending_warp.is_none());
    assert!(model.npc_interaction.is_none());
}

#[test]
fn gm_speed_reply_is_strict_local_and_preserves_the_authoritative_value() {
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_GM_REP_PC_SET_VALUE,
        flags: 0,
        checksum: 0,
        payload: ffone_protocol::GmSetValueReply0104 {
            pc_id: 81,
            value_type: GM_SET_VALUE_SPEED_0104,
            value: 1_200,
        }
        .encode(),
    };
    assert_eq!(
        decode_local_gm_speed_reply_0104(&frame, Some(81)),
        Ok(Some(1_200))
    );
    assert!(
        decode_local_gm_speed_reply_0104(&frame, Some(82))
            .unwrap_err()
            .contains("targets PC 81")
    );

    let unsupported = DecodedFrame {
        payload: ffone_protocol::GmSetValueReply0104 {
            pc_id: 81,
            value_type: 7,
            value: 900,
        }
        .encode(),
        ..frame.clone()
    };
    assert!(
        decode_local_gm_speed_reply_0104(&unsupported, Some(81))
            .unwrap_err()
            .contains("unsupported GM set-value reply type 7")
    );

    let malformed = DecodedFrame {
        payload: vec![0; 11],
        ..frame
    };
    assert!(
        decode_local_gm_speed_reply_0104(&malformed, Some(81))
            .unwrap_err()
            .contains("payload must be 12 bytes, got 11")
    );
}

#[test]
fn production_race_reply_bridge_requires_the_single_matching_pending_request() {
    let mut model = RaceModeModel::default();
    let mut production = RaceProductionRuntime::default();
    production.player.current_ep_id = 15;
    production
        .begin_mode(
            RACE_MODE_GAME_MODE_ID,
            9_001,
            6_501,
            "M_SACTRace1".to_owned(),
        )
        .unwrap();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Start,
            npc: Some(RaceNpcContext {
                instance_id: 9_001,
                has_race_start_voice: true,
            }),
            player: production.player,
            current_ep_instance_exists: true,
        })
        .unwrap();
    let pending = model.pending_request_id().unwrap();

    let wrong_kind = 8_i32.to_le_bytes();
    assert!(
        apply_correlated_race_reply_0104(
            RACE_END_FAILURE_PACKET_ID,
            &wrong_kind,
            2.0,
            &mut model,
            &mut production,
        )
        .unwrap_err()
        .contains("WrongReplyKind")
    );
    assert_eq!(model.pending_request_id(), Some(pending));
    assert!(!production.player.ring_race_active);

    let mut start_success = vec![0_u8; 12];
    start_success[..8].copy_from_slice(&123_u64.to_le_bytes());
    start_success[8..12].copy_from_slice(&90_i32.to_le_bytes());
    apply_correlated_race_reply_0104(
        RACE_START_SUCCESS_PACKET_ID,
        &start_success,
        12.5,
        &mut model,
        &mut production,
    )
    .unwrap();
    assert_eq!(model.pending_request_id(), None);
    assert!(production.player.ring_race_active);
    assert_eq!(production.player.instance_race_mode, 1);
    assert_eq!(production.player.ring_count, 0);
    assert_eq!(production.player.local_start_time, 12.5);
    assert_eq!(production.player.race_limit_time, 90);

    let mut hidden = RaceModeModel::default();
    assert!(
        apply_correlated_race_reply_0104(
            RACE_START_SUCCESS_PACKET_ID,
            &start_success,
            1.0,
            &mut hidden,
            &mut RaceProductionRuntime::default(),
        )
        .unwrap_err()
        .contains("uncorrelated")
    );
}
