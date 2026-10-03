use super::*;

#[test]
fn local_flight_command_supports_toggle_and_explicit_state() {
    assert_eq!(
        parse_local_flight_command("/fly"),
        Some(FlightCommand::Toggle)
    );
    assert_eq!(
        parse_local_flight_command("  /FLY on  "),
        Some(FlightCommand::Set(true))
    );
    assert_eq!(
        parse_local_flight_command("/fly OFF"),
        Some(FlightCommand::Set(false))
    );
    assert_eq!(
        parse_local_flight_command("/fly maybe"),
        Some(FlightCommand::Invalid)
    );
    assert_eq!(parse_local_flight_command("/flying"), None);
}

#[test]
fn option_runtime_seeds_only_from_real_live_owners_once() {
    let window = Window {
        resolution: bevy::window::WindowResolution::new(1_920, 1_080)
            .with_scale_factor_override(1.5),
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        ..default()
    };
    let mut runtime = OptionProductionRuntime::default();
    runtime.initialize_from_live_runtime(&window, false, Some(8.5));

    // Persist the real monitor mode, not the 1280x720 logical UI canvas.
    assert_eq!(runtime.options.graphics.width, 1_920);
    assert_eq!(runtime.options.graphics.height, 1_080);
    assert!(!runtime.options.graphics.windowed);
    assert!(!runtime.options.graphics.glow);
    assert_eq!(runtime.input.camera_sensitivity, 8.5);

    let other = Window {
        resolution: (800, 600).into(),
        ..default()
    };
    runtime.initialize_from_live_runtime(&other, true, Some(2.0));
    assert_eq!(runtime.options.graphics.width, 1_920);
    assert_eq!(runtime.input.camera_sensitivity, 8.5);
}

#[test]
fn option_world_reset_drops_only_transient_mode_state() {
    let committed = OptionSettings::default();
    let input = InputSettings::default();
    let mut model = OptionUiModel::default();
    let mut outbox = OptionUiOutbox::default();
    model.open(
        committed.clone(),
        input,
        OptionOpenAudioRoute {
            main_game_transition: true,
            inventory_transition: false,
        },
        &mut outbox,
    );
    model.modal.system_popup = true;
    model.reset_runtime_session();
    outbox.clear();

    assert!(!model.visible);
    assert!(!model.modal.disables_all());
    assert!(model.key_capture.is_none());
    assert_eq!(model.persisted_options, committed);
    assert!(outbox.is_empty());
}

#[test]
fn normal_world_uses_world_ambience_and_preserves_runtime_only_for_world_ready() {
    assert!(client_state_uses_tutorial_ambience(ClientState::Tutorial));
    assert!(!client_state_uses_tutorial_ambience(ClientState::World));
    assert!(!client_state_uses_tutorial_ambience(
        ClientState::CharacterSelect
    ));
    assert!(tutorial_exit_preserves_world_runtime(true));
    assert!(!tutorial_exit_preserves_world_runtime(false));
}

#[test]
fn normal_world_npc_corruption_hit_commits_signed_hp_and_exact_nano_state_atomically() {
    let prefix = ffone_protocol::NpcSkillCorruptionHitPrefix0104 {
        npc_id: 314,
        skill_id: 52,
        style: 2,
        position: [100, 200, 300],
        target_count: 1,
    };
    let mut payload = prefix.encode_prefix();
    let mut result = vec![0; 40];
    result[0..4].copy_from_slice(&1_i32.to_le_bytes());
    result[4..8].copy_from_slice(&77_i32.to_le_bytes());
    result[12..16].copy_from_slice(&450_i32.to_le_bytes());
    result[16..20].copy_from_slice(&(-25_i32).to_le_bytes());
    result[20] = 16;
    result[22..24].copy_from_slice(&2_i16.to_le_bytes());
    result[24..28].copy_from_slice(&1_i32.to_le_bytes());
    result[28..30].copy_from_slice(&17_i16.to_le_bytes());
    result[30..32].copy_from_slice(&0_i16.to_le_bytes());
    result[32..36].copy_from_slice(&0x400_i32.to_le_bytes());
    result[36..40].copy_from_slice(&0x200_i32.to_le_bytes());
    payload.extend_from_slice(&result);
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT,
        flags: 0,
        checksum: 0,
        payload,
    };
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            hp: Some(700),
            nano_slots: [
                RuntimeNanoSlot::default(),
                RuntimeNanoSlot::default(),
                RuntimeNanoSlot {
                    nano_id: Some(17),
                    skill_id: 144,
                    stamina: 80,
                    active: true,
                },
            ],
            ..default()
        },
        ..default()
    };
    let mut skill_buffs = SkillBuffUiModel::default();
    let mut inbox = WorldNanoAuthorityInbox0104::default();

    assert_eq!(
        apply_world_npc_skill_response_frame(&frame, &mut runtime, &mut skill_buffs, &mut inbox,),
        Ok(true)
    );
    assert_eq!(runtime.hp, Some(-25));
    assert_eq!(skill_buffs.local_condition_bit_flag, 0x400);
    assert_eq!(runtime.nano_slots[2].nano_id, Some(17));
    assert_eq!(runtime.nano_slots[2].skill_id, 144);
    assert_eq!(runtime.nano_slots[2].stamina, 0);
    assert_eq!(runtime.nano_slots.map(|slot| slot.active), [false; 3]);
    assert!(inbox.local_movements.is_empty());

    let committed = (runtime.hp, runtime.nano_slots, skill_buffs.clone());
    let mut malformed = frame;
    malformed.payload.pop();
    assert!(
        apply_world_npc_skill_response_frame(
            &malformed,
            &mut runtime,
            &mut skill_buffs,
            &mut inbox,
        )
        .is_err()
    );
    assert_eq!(
        (runtime.hp, runtime.nano_slots, skill_buffs),
        committed,
        "a truncated tail must commit no local post-state"
    );
}

#[test]
fn regen_success_applies_authoritative_state_before_modal_close_boundary() {
    let success = PcRegenSuccess0104 {
        regen_data: PcRegenData0104 {
            hp: 1_025,
            map_number: 14,
            position: [37_333, 44_260, -570],
            active_nano_slot: 1,
            nanos: [
                ffone_protocol::Nano0104 {
                    id: 1,
                    skill_id: 11,
                    stamina: 21,
                },
                ffone_protocol::Nano0104 {
                    id: 2,
                    skill_id: 12,
                    stamina: 22,
                },
                ffone_protocol::Nano0104 {
                    id: -1,
                    skill_id: 0,
                    stamina: 0,
                },
            ],
        },
        move_location: 1,
        fusion_matter: 350,
    };
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_level: 2,
            hp: Some(0),
            map_number: Some(0),
            fusion_matter: 1,
            pending_nano_activation: Some(0),
            ..default()
        },
        ..default()
    };

    let position = apply_pc_regen_success_to_runtime(success, &mut runtime);

    assert_eq!(
        position,
        ProtocolPosition::new(success.regen_data.position).to_native()
    );
    assert_eq!(runtime.hp, Some(1_025));
    assert_eq!(runtime.map_number, Some(14));
    assert_eq!(runtime.fusion_matter, 350);
    assert_eq!(runtime.max_fusion_matter, 700);
    assert_eq!(runtime.pending_nano_activation, None);
    assert_eq!(
        runtime
            .nano_slots
            .map(|slot| (slot.nano_id, slot.skill_id, slot.stamina, slot.active)),
        [
            (Some(1), 11, 21, false),
            (Some(2), 12, 22, true),
            (None, 0, 0, false),
        ]
    );
}

#[test]
fn malformed_or_mismatched_mentor_reply_cannot_mutate_runtime_ui_or_generic_fm() {
    for payload in [
        vec![0; PcChangeMentorSuccess0104::SIZE - 1],
        PcChangeMentorSuccess0104 {
            mentor: GuideMentor::BenTennyson.wire_id(),
            mentor_count: 1,
            fusion_matter: 999,
        }
        .encode(),
    ] {
        let (mut guide_runtime, mut model, mut outbox) =
            pending_guide_change(GuideMentor::Dexter, GuideMentor::Edd);
        let before_runtime = guide_runtime.clone();
        let before_model = model.clone();
        let frame = DecodedFrame {
            packet_type: packet::P_FE2CL_REP_PC_CHANGE_MENTOR_SUCC,
            flags: 0,
            checksum: 0,
            payload,
        };
        assert!(
            apply_guide_mentor_reply_transactional(
                &frame,
                &mut guide_runtime,
                &mut model,
                &mut outbox,
            )
            .is_err()
        );
        assert_eq!(guide_runtime, before_runtime);
        assert_eq!(model, before_model);
        assert!(outbox.is_empty());

        let mut status = RuntimeStatus {
            core: RuntimePlayerStatus {
                player_id: Some(77),
                fusion_matter: 123,
                ..default()
            },
            ..default()
        };
        assert_eq!(apply_runtime_frame(&frame, &mut status), None);
        assert_eq!(
            status.fusion_matter, 123,
            "the generic status path must not decode mentor success"
        );
    }
}

#[test]
fn runtime_fusion_meter_tracks_reward_and_level_change_packets() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            player_level: 1,
            fusion_matter: 0,
            max_fusion_matter: 220,
            ..Default::default()
        },
        ..default()
    };
    let mut reward_payload = vec![0; 36];
    reward_payload[4..8].copy_from_slice(&110_i32.to_le_bytes());
    let _ = apply_runtime_frame(
        &ffone_protocol::DecodedFrame {
            packet_type: packet::P_FE2CL_REP_REWARD_ITEM,
            flags: 0,
            checksum: 0,
            payload: reward_payload,
        },
        &mut runtime,
    );
    assert_eq!(runtime.fusion_matter, 110);
    assert_eq!(runtime.max_fusion_matter, 220);

    let mut level_payload = vec![0; 8];
    level_payload[..4].copy_from_slice(&2_i32.to_le_bytes());
    level_payload[4..].copy_from_slice(&350_i32.to_le_bytes());
    let _ = apply_runtime_frame(
        &ffone_protocol::DecodedFrame {
            packet_type: packet::P_FE2CL_REP_PC_CHANGE_LEVEL_SUCC,
            flags: 0,
            checksum: 0,
            payload: level_payload,
        },
        &mut runtime,
    );
    assert_eq!(runtime.player_level, 2);
    assert_eq!(runtime.fusion_matter, 350);
    assert_eq!(runtime.max_fusion_matter, 700);
}

#[test]
fn runtime_gm_level_notification_updates_owner_without_spending_fm() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            player_level: 1,
            fusion_matter: 110,
            max_fusion_matter: 220,
            ..Default::default()
        },
        ..default()
    };
    let mut frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_CHANGE_LEVEL,
        flags: 0,
        checksum: 0,
        payload: ffone_protocol::wire_0104::PcChangeLevelReply0104 {
            pc_id: 77,
            pc_level: 2,
        }
        .encode(),
    };
    // The final two bytes are padding, not the high half of an i32 level.
    frame.payload[6..8].copy_from_slice(&[0xff, 0xff]);
    assert_eq!(apply_runtime_frame(&frame, &mut runtime), None);
    assert_eq!(runtime.player_level, 2);
    assert_eq!(runtime.fusion_matter, 110);
    assert_eq!(runtime.max_fusion_matter, 700);
    for (owner, level) in [(99, 3), (77, 0), (77, -1)] {
        frame.payload = ffone_protocol::wire_0104::PcChangeLevelReply0104 {
            pc_id: owner,
            pc_level: level,
        }
        .encode();
        apply_runtime_frame(&frame, &mut runtime);
        assert_eq!(runtime.player_level, 2);
        assert_eq!(runtime.fusion_matter, 110);
        assert_eq!(runtime.max_fusion_matter, 700);
    }
    frame.payload.truncate(7);
    apply_runtime_frame(&frame, &mut runtime);
    assert_eq!(runtime.player_level, 2);
}

#[test]
fn task_start_ack_returns_to_exact_close_only_npc_mode() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .insert_resource(NetworkBridge::start())
        .insert_resource(runtime_test_mission_content())
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialMissionRuntime>()
        .init_resource::<TutorialActorCommandQueue>()
        .init_resource::<TutorialAmbientRuntime>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<TutorialLogicRuntime>()
        .add_systems(
            Update,
            (
                consume_tutorial_mission_outbox,
                sync_tutorial_mission_interaction,
            )
                .chain(),
        );
    app.world_mut().spawn(TutorialActor {
        id: 1005,
        npc_type: 2671,
        team: 1,
        hp: 100,
        max_hp: 100,
        damaged: false,
        interacting: true,
        invulnerable: false,
    });

    app.update();
    {
        let model = app.world().resource::<MissionUiModel>();
        assert!(model.npc_icon_mode_visible);
        assert_eq!(
            model
                .npc_interaction
                .as_ref()
                .expect("Numbuh Two interaction")
                .available_missions
                .iter()
                .map(|mission| mission.task_id)
                .collect::<Vec<_>>(),
            vec![2248]
        );
    }

    let mut queued = GameplayUiOutbox::default();
    {
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        assert!(model.select_npc_mission(0, &mut queued));
        assert!(model.accept_mission(&mut queued));
    }
    *app.world_mut().resource_mut::<GameplayUiOutbox>() = queued;
    app.update();

    {
        let model = app.world().resource::<MissionUiModel>();
        assert!(matches!(model.journal, MissionJournalUi::Hidden));
        assert!(model.npc_icon_mode_visible);
        assert!(model.gameplay_input_blocked());
        assert_eq!(model.npc_subtarget_actor_id(), Some(1005));
        let interaction = model
            .npc_interaction
            .as_ref()
            .expect("post-ack close-only interaction");
        assert!(interaction.available_missions.is_empty());
        assert!(interaction.completed_missions.is_empty());
        assert!(interaction.services.is_empty());
        assert!(interaction.warp.is_none());
    }

    let voice = app.world_mut().spawn(TutorialVoiceAudio).id();
    app.world_mut()
        .resource_mut::<TutorialMissionRuntime>()
        .auxiliary
        .start(TutorialAuxiliarySequence::TalkNumTwo1);
    let mut queued = GameplayUiOutbox::default();
    assert!(
        app.world_mut()
            .resource_mut::<MissionUiModel>()
            .close_npc_interaction(&mut queued)
    );
    *app.world_mut().resource_mut::<GameplayUiOutbox>() = queued;
    app.update();
    assert!(
        app.world().get_entity(voice).is_err(),
        "closing tutorial NPC dialogue must stop its voice"
    );
    assert!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .auxiliary
            .active()
            .is_none()
    );
    assert!(
        !app.world()
            .resource::<MissionUiModel>()
            .npc_icon_mode_visible
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<TutorialActorCommandQueue>()
            .take_all()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![ffone_client::tutorial_actors::TutorialActorCommand::ClearInteractions]
    );
    assert_eq!(
        app.world()
            .resource::<TutorialSession>()
            .progress
            .event_value(TutorialEvent::NpcIconClose),
        1
    );
}

#[test]
fn runtime_source_pack_and_legacy_source_flags_are_rejected() {
    for flag in [
        "--content-pack",
        "--build",
        "--project",
        "--character-npc-scale",
    ] {
        assert!(
            ClientConfig::from_args([flag.to_owned(), "legacy-source".to_owned()]).is_err(),
            "{flag} unexpectedly became a runtime input"
        );
    }
}

#[test]
fn actor_events_set_original_flags_and_mission_reward_state() {
    let mut progress = TutorialProgress::default();
    progress.init_chapter(3).unwrap();
    progress.init_step(MissionStage::ObjectiveCombat as i16);
    let entity = Entity::from_bits(42);
    let mut hp = Some(125);

    apply_tutorial_actor_event(
        TutorialActorEvent::Damaged {
            id: MISSION_TARGET_ID,
            entity,
            amount: 100,
            remaining_hp: 1_200,
            first_hit: true,
        },
        &mut progress,
        &mut hp,
        false,
    );
    assert_eq!(progress.event_value(TutorialEvent::DamageNpc), 1);

    apply_tutorial_actor_event(
        TutorialActorEvent::Dead {
            id: MISSION_TARGET_ID,
            entity,
        },
        &mut progress,
        &mut hp,
        true,
    );
    assert_eq!(progress.event_value(TutorialEvent::DeadNpc), 1);
    assert_eq!(progress.event_value(TutorialEvent::TaskStart), 1);
    assert_eq!(
        progress.stage(),
        Some(TutorialStage::Mission(MissionStage::ObjectiveCombat)),
        "events must not invent a direct tutorial transition"
    );

    apply_tutorial_actor_event(
        TutorialActorEvent::AttackedPlayer {
            id: 1005,
            entity,
            damage: 50,
        },
        &mut progress,
        &mut hp,
        false,
    );
    assert_eq!(hp, Some(100));
    assert_eq!(progress.event_value(TutorialEvent::DamageUser), 1);
}

#[test]
fn failed_world_ready_spawn_rolls_back_deferred_and_authoritative_state() {
    fn rollback_once(
        mut commands: Commands,
        mut lifecycle_ingress: ResMut<NetworkEntityLifecycleIngress0104>,
        mut lifecycle_session: ResMut<NetworkLifecycleSession>,
        mut runtime: ResMut<RuntimeStatus>,
        mut inventory: ResMut<LocalInventoryRuntime>,
        mut effects: ResMut<TutorialEffectRuntime>,
        mut player_commands: ResMut<TutorialPlayerPresentationCommandQueue>,
        mut rig_assets: ResMut<NativePlayerRigAssetCache>,
        mut loading: ResMut<GameplayLoadingState>,
    ) {
        rollback_failed_world_ready_spawn(
            &mut commands,
            &mut lifecycle_ingress,
            &mut lifecycle_session,
            &mut runtime,
            &mut inventory,
            &mut effects,
            &mut player_commands,
            &mut rig_assets,
            &mut loading,
            ResourceLoadingScope::World,
        );
    }

    let mut app = App::new();
    app.init_resource::<NetworkEntityLifecycleIngress0104>()
        .init_resource::<NetworkLifecycleSession>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<LocalInventoryRuntime>()
        .init_resource::<TutorialEffectRuntime>()
        .init_resource::<TutorialPlayerPresentationCommandQueue>()
        .init_resource::<NativePlayerRigAssetCache>()
        .init_resource::<GameplayLoadingState>()
        .insert_resource(NativeWorldStreamingStatus {
            blocker: Some("failed transaction blocker".to_owned()),
            ..default()
        })
        .add_systems(Update, rollback_once);

    let epoch = app
        .world_mut()
        .resource_mut::<NetworkLifecycleSession>()
        .begin();
    {
        let mut ingress = app
            .world_mut()
            .resource_mut::<NetworkEntityLifecycleIngress0104>();
        ingress.begin_session(epoch, 77);
        ingress.push_bootstrap(
            epoch,
            ffone_protocol::InitialAroundPacket0104::Npcs(Vec::new()),
        );
    }
    let character = CharacterSummary {
        slot: 1,
        level: 12,
        pc_uid: 42,
        first_name: "Rollback".to_owned(),
        last_name: "Test".to_owned(),
        position: [1, 2, 3],
        style: ffone_protocol::CharacterStyle0104 {
            name_check: 1,
            gender: 1,
            face_style: 0,
            hair_style: 0,
            hair_color: 0,
            skin_color: 0,
            eye_color: 0,
            height: 0,
            body: 0,
            class: 0,
            appearance_flag: 1,
            tutorial_flag: 1,
            payzone_flag: 0,
        },
        equipment: [ffone_protocol::EquippedItem0104::default();
            ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
    };
    {
        let mut runtime = app.world_mut().resource_mut::<RuntimeStatus>();
        runtime.roster.characters.push(character.clone());
        runtime.roster.selected_uid = Some(character.pc_uid);
        runtime.roster.pending_character_entry_uid = Some(character.pc_uid);
        runtime.player_id = Some(77);
        runtime.diagnostics.bootstrap_packets = 3;
        runtime.diagnostics.bootstrap_decode_errors = 1;
    }
    app.world_mut()
        .resource_mut::<LocalInventoryRuntime>()
        .seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    app.world_mut()
        .resource_mut::<TutorialPlayerPresentationCommandQueue>()
        .push_equipment(TutorialPlayerEquipmentRequest {
            slot: ffone_protocol::CharacterEquipSlot0104::Hand,
            item: ffone_protocol::EquippedItem0104 {
                item_type: 0,
                item_id: 43,
                option: 0,
                time_limit: 0,
            },
        });
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .enqueue(TutorialEffectRuntimeCommand::ClearTracked { source_line: 1 });
    {
        let mut loading = app.world_mut().resource_mut::<GameplayLoadingState>();
        loading.begin(ResourceLoadingScope::World);
        loading.finish();
    }
    let world_slice = app.world_mut().spawn(WorldSliceEntity).id();
    let partial_native_scene = app.world_mut().spawn(NativeWorldSceneEntity).id();

    app.update();

    assert!(app.world().get_entity(world_slice).is_err());
    assert!(app.world().get_entity(partial_native_scene).is_err());
    assert!(
        app.world()
            .resource::<NetworkLifecycleSession>()
            .active
            .is_none()
    );
    assert_eq!(
        app.world()
            .resource::<NetworkEntityLifecycleIngress0104>()
            .len(),
        1,
        "stale begin/bootstrap events must be replaced by one disconnect"
    );
    let runtime = app.world().resource::<RuntimeStatus>();
    assert_eq!(runtime.roster.characters, vec![character]);
    assert_eq!(runtime.roster.selected_uid, Some(42));
    assert_eq!(runtime.roster.pending_character_entry_uid, None);
    assert_eq!(runtime.player_id, None);
    assert_eq!(runtime.diagnostics.bootstrap_packets, 0);
    assert_eq!(runtime.diagnostics.bootstrap_decode_errors, 0);
    assert!(
        app.world()
            .resource::<LocalInventoryRuntime>()
            .snapshot()
            .is_none()
    );
    assert!(
        app.world()
            .resource::<TutorialPlayerPresentationCommandQueue>()
            .is_empty()
    );
    {
        let mut effects = app.world_mut().resource_mut::<TutorialEffectRuntime>();
        effects.process_pending();
        assert_eq!(effects.drain_records().count(), 0);
    }
    let loading = app.world().resource::<GameplayLoadingState>();
    assert!(loading.visible, "failure must re-arm the loading barrier");
    assert_eq!(loading.scope, Some(ResourceLoadingScope::World));
    assert_eq!(loading.phase, GameplayLoadingPhase::Assets);
    assert_eq!(loading.overall_progress, 0.0);
    assert_eq!(loading.phase_progress, 0.0);
    assert!(loading.blocked.is_none());
    assert!(
        app.world()
            .resource::<NativeWorldStreamingStatus>()
            .blocker
            .is_none()
    );
}
