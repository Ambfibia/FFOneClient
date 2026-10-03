use super::*;

#[test]
fn reference_native_intents_never_enter_the_deferred_queue() {
    let stage = TutorialStage::Mission(MissionStage::OpenMenu);
    let mut native = TutorialNativeMechanics::default();
    native.sync_stable_stage(stage);
    native.mark_tutorial_pointer_visible();
    let mut presentation = TutorialAuxiliaryPresentation {
        cursor: Some((
            "tutorial/live-pointer.png",
            TutorialScreenPoint::new(
                AuxiliaryScreenAxis::Pixels(1),
                AuxiliaryScreenAxis::Pixels(2),
            ),
            AuxiliaryScreenPivot::Legacy(1),
        )),
        ..default()
    };
    let mut logic = TutorialLogicRuntime::default();

    for intent in [
        TutorialIntent::SetFatigueLevel(1),
        TutorialIntent::SetInstanceMap(true),
        TutorialIntent::SetEpisode(0),
        TutorialIntent::LockUi,
        TutorialIntent::UnlockUi,
        TutorialIntent::PushInputFilter,
        TutorialIntent::PopInputFilter,
        TutorialIntent::HideTutorialPointer,
    ] {
        if !apply_tutorial_native_intent(intent, &mut native, &mut presentation) {
            logic.defer(intent);
        }
    }

    assert!(logic.deferred_intents.is_empty());
    assert_eq!(native.fatigue_level(), 1);
    assert!(native.instance_map());
    assert_eq!(native.episode(), 0);
    assert!(!native.tutorial_pointer_visible());
    assert!(presentation.cursor_touched);
    assert!(presentation.cursor.is_none());
}

#[test]
fn gameplay_light_never_queues_the_unsupported_standard_shadow_prepass() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, setup_scene);
    app.update();

    let lights = app
        .world_mut()
        .query::<&DirectionalLight>()
        .iter(app.world())
        .collect::<Vec<_>>();
    assert_eq!(lights.len(), 1);
    assert!(!lights[0].shadow_maps_enabled);
}

#[test]
fn original_gameplay_cursor_is_locked_only_during_ready_pointer_free_play() {
    for state in [
        ClientState::Bootstrap,
        ClientState::Login,
        ClientState::CharacterSelect,
        ClientState::CharacterCreate,
    ] {
        assert!(!legacy_gameplay_cursor_locked(state, true, false));
    }
    for state in [ClientState::Tutorial, ClientState::World] {
        assert!(legacy_gameplay_cursor_locked(state, true, false));
        assert!(!legacy_gameplay_cursor_locked(state, false, false));
        assert!(!legacy_gameplay_cursor_locked(state, true, true));
    }
}

#[test]
fn npc_subtarget_camera_uses_exact_radius_and_height_and_faces_the_npc() {
    let target = Vec3::new(10.0, 2.0, 3.0);
    let (position, focus) = tutorial_npc_subtarget_pose(target, Quat::IDENTITY, 0.9, 1.7);
    assert!(position.abs_diff_eq(Vec3::new(10.0, 3.02, 0.3), 0.000_01));
    assert!(focus.abs_diff_eq(Vec3::new(10.0, 3.02, 3.0), 0.000_01));

    let mut camera = Transform::from_translation(position);
    camera.look_at(focus, Vec3::Y);
    assert!((camera.rotation * Vec3::NEG_Z).abs_diff_eq((focus - position).normalize(), 0.000_01));

    let quarter_turn = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let (rotated_position, _) = tutorial_npc_subtarget_pose(target, quarter_turn, 0.9, 1.7);
    assert!(rotated_position.abs_diff_eq(Vec3::new(7.3, 3.02, 3.0), 0.000_01));

    let (world_position, world_focus) = world_npc_subtarget_pose(target, Quat::IDENTITY, 90, 170);
    assert!(world_position.abs_diff_eq(position, 0.000_01));
    assert!(world_focus.abs_diff_eq(focus, 0.000_01));

    let (taller_position, taller_focus) = world_npc_subtarget_pose(target, Quat::IDENTITY, 90, 220);
    assert!(taller_position.abs_diff_eq(Vec3::new(10.0, 3.32, 0.3), 0.000_01));
    assert!(taller_focus.abs_diff_eq(Vec3::new(10.0, 3.32, 3.0), 0.000_01));
}

#[test]
fn kill_three_replaces_the_first_attack_mouse_with_the_exact_up_pointer() {
    use ffone_client::gameplay_ui::TutorialCuePosition;

    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, language) = Localization::open(&asset_root, "en").unwrap();
    let first_attack = tutorial_ui_for_stage(
        TutorialStage::Combat(CombatStage::FirstAttack),
        0.0,
        1264,
        681,
        &localization,
        &language,
    );
    assert_eq!(
        first_attack
            .illustration
            .expect("first attack mouse")
            .illustration,
        TutorialIllustration::LeftMouse
    );
    assert!(first_attack.arrow.is_none());

    let kill_three = tutorial_ui_for_stage(
        TutorialStage::Combat(CombatStage::KillThree),
        0.0,
        1264,
        681,
        &localization,
        &language,
    );
    assert!(kill_three.illustration.is_none());
    let arrow = kill_three.arrow.expect("kill-three up pointer");
    assert_eq!(arrow.direction, TutorialArrowDirection::Up);
    assert_eq!(arrow.scale_pivot, TutorialCueScalePivot::CenterTop);
    assert_eq!(
        arrow.position,
        TutorialCuePosition::TopLeft {
            left: 597.0,
            top: 40.0,
        }
    );
}

#[test]
fn minimap_fallback_pointer_tracks_the_live_viewport_width() {
    use ffone_client::gameplay_ui::TutorialCuePosition;

    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, language) = Localization::open(&asset_root, "en").unwrap();
    let pointer = |width| {
        tutorial_ui_for_stage(
            TutorialStage::Minimap(MinimapStage::MinimapSequence),
            0.0,
            width,
            681,
            &localization,
            &language,
        )
        .arrow
        .expect("minimap fallback pointer")
    };

    let at_1252 = pointer(1252);
    assert_eq!(at_1252.direction, TutorialArrowDirection::Right);
    assert_eq!(at_1252.scale_pivot, TutorialCueScalePivot::TopRight);
    assert_eq!(
        at_1252.position,
        TutorialCuePosition::TopLeft {
            left: 972.0,
            top: 50.0,
        }
    );
    assert_eq!(
        pointer(1920).position,
        TutorialCuePosition::TopLeft {
            left: 1640.0,
            top: 50.0,
        }
    );
}

#[test]
fn legacy_avatar_hp_matches_retrobution_growth_and_class_bonuses() {
    assert_eq!(legacy_avatar_max_hp(1, 0, 999), 1000);
    assert_eq!(legacy_avatar_max_hp(40, 0, 999), 3925);
    assert_eq!(legacy_avatar_max_hp(40, 2, 999), 5887);
    assert_eq!(legacy_avatar_max_hp(40, 3, 999), 4710);
    assert_eq!(legacy_avatar_max_hp(41, 0, 4321), 4321);
    assert_eq!(legacy_avatar_max_hp(0, 0, 0), 1);
}

#[test]
fn legacy_avatar_fusion_caps_match_retrobution_growth_rows() {
    assert_eq!(legacy_avatar_max_fusion_matter(1, 0), 220);
    assert_eq!(legacy_avatar_max_fusion_matter(2, 0), 700);
    assert_eq!(legacy_avatar_max_fusion_matter(36, 0), 293_580);
    assert_eq!(legacy_avatar_max_fusion_matter(40, 0), 450_000);
    assert_eq!(legacy_avatar_max_fusion_matter(41, 123), 123);
    assert_eq!(legacy_avatar_max_fusion_matter(0, 0), 1);
}

#[test]
fn movement_intents_reach_network_in_both_shared_gameplay_scopes() {
    assert!(client_state_sends_movement_intents(ClientState::Tutorial));
    assert!(client_state_sends_movement_intents(ClientState::World));
    for state in [
        ClientState::Bootstrap,
        ClientState::Login,
        ClientState::CharacterSelect,
        ClientState::CharacterCreate,
        ClientState::CharacterCreateIntro,
        ClientState::TutorialIntro,
    ] {
        assert!(!client_state_sends_movement_intents(state), "{state:?}");
    }
}

#[test]
fn exit_in_flight_teardown_is_immediate_persistent_and_idempotent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialLogicRuntime>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialChoreographyPlayer>()
        .init_resource::<TutorialChoreographyExecution>()
        .init_resource::<TutorialMissionRuntime>()
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialAuxiliaryPresentation>()
        .init_resource::<TutorialActorCommandQueue>()
        .init_resource::<TutorialNanoPresentationCommandQueue>()
        .init_resource::<TutorialNanoGameplayCommandQueue>()
        .init_resource::<TutorialNanoPresentationState>()
        .init_resource::<TutorialPlayerPresentationCommandQueue>()
        .init_resource::<PendingTutorialActorEffects>()
        .init_resource::<TutorialAmbientRuntime>()
        .init_resource::<TutorialOverlayUiModel>()
        .add_systems(Update, apply_pending_tutorial_exit);

    {
        let mut tutorial = app.world_mut().resource_mut::<TutorialSession>();
        tutorial.completion_requested = true;
        tutorial.scene = TutorialScene::BasicMove;
        tutorial.presentation_cursor = 7;
    }
    {
        let mut logic = app.world_mut().resource_mut::<TutorialLogicRuntime>();
        logic.delay_remaining_seconds = Some(9.0);
        logic
            .deferred_intents
            .push(TutorialIntent::StartReminderTimer);
    }
    {
        let mut native = app.world_mut().resource_mut::<TutorialNativeMechanics>();
        native.apply_intent(TutorialIntent::SetInstanceMap(true));
        native.sync_stable_stage(TutorialStage::Movement(MovementStage::MoveForward));
    }
    app.world_mut()
        .resource_mut::<TutorialChoreographyPlayer>()
        .start(TutorialScene::BasicMove);
    app.world_mut()
        .resource_mut::<TutorialMissionRuntime>()
        .auxiliary
        .start(TutorialAuxiliarySequence::BasicArrowKey);
    app.world_mut()
        .resource_mut::<PendingTutorialActorEffects>()
        .spawns
        .push_back(PendingTutorialActorEffect {
            sequence: TutorialAuxiliarySequence::MinimapEvent,
            runtime_id: 1005,
            offset: [0.0, -1.0, 0.0],
            effect_id: 653,
            source_line: 3679,
        });
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .set_interacting(1005, true);
    app.world_mut()
        .resource_mut::<TutorialNanoPresentationCommandQueue>()
        .destroy();
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .equip(1, 1, 100);
    app.world_mut()
        .resource_mut::<TutorialPlayerPresentationCommandQueue>()
        .stand_force(ffone_runtime_contracts::PlayerRigGender::Male);
    app.world_mut().resource_mut::<MissionUiModel>().enabled = true;
    *app.world_mut().resource_mut::<TutorialOverlayUiModel>() =
        TutorialOverlayUiModel::retrobution_reference_frame();

    let dome = app
        .world_mut()
        .spawn(TutorialDome {
            gltf: Handle::default(),
            scene: Handle::default(),
        })
        .id();
    let ambient_audio = app
        .world_mut()
        .spawn(TutorialAmbientAudio {
            cue: TutorialAmbientCue::TutorialMain,
        })
        .id();
    let music_audio = app.world_mut().spawn(TutorialMusicAudio).id();
    let loop_audio = app
        .world_mut()
        .spawn(TutorialLoopAudio {
            cue: "DexterHologram_LOOP",
        })
        .id();
    {
        let mut ambient = app.world_mut().resource_mut::<TutorialAmbientRuntime>();
        ambient.phase = TutorialAmbientPhase::Stable;
        ambient.current_cue = Some(TutorialAmbientCue::TutorialMain);
        ambient.current_entity = Some(ambient_audio);
        ambient.target_cue = Some(TutorialAmbientCue::TutorialMain);
    }
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.movement_enabled = true;
    controller.velocity = Vec3::ONE;
    let player = app.world_mut().spawn((LocalPlayer, controller)).id();
    let actor = app
        .world_mut()
        .spawn(TutorialActor {
            id: 1005,
            npc_type: 2674,
            team: 1,
            hp: 100,
            max_hp: 100,
            damaged: false,
            interacting: true,
            invulnerable: false,
        })
        .id();

    app.update();

    for entity in [dome, ambient_audio, music_audio, loop_audio] {
        assert!(app.world().get_entity(entity).is_err());
    }
    let tutorial = app.world().resource::<TutorialSession>();
    assert!(tutorial.completion_requested);
    assert!(tutorial.exit_teardown_applied);
    assert_eq!(tutorial.scene, TutorialScene::None);
    assert_eq!(tutorial.presentation_cursor, 0);
    let native = app.world().resource::<TutorialNativeMechanics>();
    assert!(!native.instance_map());
    assert!(native.live_input_locks().iter().all(|locked| *locked));
    assert!(
        app.world()
            .resource::<TutorialChoreographyPlayer>()
            .active_scene()
            .is_none()
    );
    assert!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .auxiliary
            .active()
            .is_none()
    );
    assert!(
        app.world()
            .resource::<PendingTutorialActorEffects>()
            .spawns
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<TutorialActorCommandQueue>()
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<TutorialNanoPresentationCommandQueue>()
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayCommandQueue>()
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<TutorialPlayerPresentationCommandQueue>()
            .is_empty()
    );
    assert_eq!(
        app.world()
            .resource::<TutorialNanoPresentationState>()
            .entity(),
        None
    );
    assert!(!app.world().resource::<MissionUiModel>().enabled);
    assert_eq!(
        *app.world().resource::<TutorialOverlayUiModel>(),
        TutorialOverlayUiModel::default()
    );
    let presentation = app.world().resource::<TutorialAuxiliaryPresentation>();
    assert_eq!(presentation.sequence, None);
    assert!(!presentation.choreography_owned);
    assert_eq!(presentation.primary_subtitle, None);
    assert_eq!(presentation.secondary_subtitle, None);
    assert_eq!(presentation.picture, None);
    assert_eq!(presentation.cursor, None);
    assert!(!app.world().get::<TutorialActor>(actor).unwrap().interacting);
    let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!(!controller.movement_enabled);
    assert_eq!(controller.velocity, Vec3::ZERO);

    app.update();
    assert!(
        app.world()
            .resource::<TutorialSession>()
            .exit_teardown_applied
    );
    assert!(!app.world().resource::<MissionUiModel>().enabled);
    assert!(!app.world().get::<TutorialActor>(actor).unwrap().interacting);
}

#[test]
fn warp_away_instance_fallback_preserves_clean_zero_without_map_zone_conflation() {
    let content = runtime_test_mission_content();
    assert_eq!(warp_away_xcom_index(&content, Some(14), [1, 2, 3]), Some(0));
    assert_eq!(warp_away_xcom_index(&content, None, [1, 2, 3]), None);
}

#[test]
fn confirmed_task_stop_revalidates_authoritative_and_viewed_active_task() {
    let content = runtime_test_mission_content();
    let definition = content.mission(500).expect("primary task 500");
    let task_id = definition.provenance.task_id;
    let mut mission = WorldMissionRuntime::default();
    mission
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(
                ffone_protocol::PcTaskStartSuccess0104 {
                    task_id,
                    remaining_time: definition.provenance.grant_timer,
                },
            ),
            &content,
        )
        .unwrap();
    let entry = content.journal_entry(task_id, "Sector V").unwrap();
    let mut ui = MissionUiModel::default();
    ui.enabled = true;
    ui.journal = MissionJournalUi::Other(JournalOtherUi {
        active_missions: vec![entry],
        ..default()
    });
    ui.viewed_journal_task_id = Some(task_id);
    ui.selected_journal_task_id = Some(task_id);

    assert_eq!(
        begin_confirmed_world_task_stop(&mut ui, &mission, task_id),
        Some(PendingMissionUiRequest::TaskStop { task_id })
    );
    assert!(matches!(
        ui.pending,
        Some(PendingMissionUiRequest::TaskStop { task_id: pending }) if pending == task_id
    ));
    assert_eq!(
        begin_confirmed_world_task_stop(&mut ui, &mission, task_id),
        None,
        "the installed network lock blocks a duplicate"
    );
    assert!(ui.reject_pending(task_id));
    ui.viewed_journal_task_id = None;
    assert_eq!(
        begin_confirmed_world_task_stop(&mut ui, &mission, task_id),
        None,
        "confirmation cannot stop a task that is no longer viewed"
    );
}

#[test]
fn normal_npc_warp_ui_projects_only_the_category_five_first_row() {
    let content = runtime_test_mission_content();
    let row = content.normal_gameplay_warp_for_npc(681).unwrap();

    assert_eq!(
        normal_npc_warp_ui_entry(&content, 44_001, 681),
        Some(WarpUiEntry {
            npc_id: 44_001,
            npc_type: 681,
            warp_id: row.warp_id,
            required_task_id: (row.limit_task_id != 0).then_some(row.limit_task_id),
            target: row.target,
            label: "WARP".to_owned(),
        })
    );
    assert_eq!(row.warp_id, 5);
    assert_eq!(normal_npc_warp_ui_entry(&content, 7, 1425), None);
}

#[test]
fn normal_npc_warp_delay_and_authoritative_correlation_are_exact() {
    let identity = NormalNpcWarpIdentity {
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
    };
    let request = PcWarpUseNpcRequest0104 {
        npc_id: identity.npc_id,
        warp_id: identity.warp_id,
        e_il_1: 4,
        item_slot_1: 2,
        e_il_2: 4,
        item_slot_2: 3,
    };
    let mut pending = PendingNormalNpcWarp {
        identity,
        request,
        elapsed_seconds: 0.0,
        sent: false,
    };

    assert_eq!(
        pending.advance(NORMAL_NPC_WARP_DELAY_SECONDS - f32::EPSILON),
        None
    );
    assert_eq!(pending.advance(f32::EPSILON), Some(request));
    assert_eq!(pending.advance(10.0), None, "a sent request is one-shot");

    let mut runtime = NormalNpcWarpRuntime {
        pending: Some(pending),
        movement_packet_emission: false,
        ..default()
    };
    assert_eq!(runtime.correlated_sent(), Some(pending));
    assert_eq!(runtime.take_correlated_sent(), Some(pending));
    assert_eq!(runtime.pending, None);
    assert!(runtime.movement_packet_emission);

    pending.sent = false;
    runtime.pending = Some(pending);
    assert_eq!(runtime.take_correlated_sent(), None);
    assert_eq!(runtime.pending, Some(pending));
}

#[test]
fn normal_npc_warp_window_in_is_the_clean_half_second_white_fade() {
    let mut production = NormalNpcWarpRuntime::default();
    production.start_window_in_fade();
    assert_eq!(production.window_in_fade_alpha, 1.0);
    production.advance_window_in_fade(0.25);
    assert_eq!(production.window_in_fade_alpha, 0.5);
    production.advance_window_in_fade(0.25);
    assert_eq!(production.window_in_fade_alpha, 0.0);
    production.advance_window_in_fade(10.0);
    assert_eq!(production.window_in_fade_alpha, 0.0);
}

#[test]
fn normal_warp_denials_and_failure_use_exact_tabledata_system_messages() {
    let content = runtime_test_mission_content();
    let mut runtime = NormalNpcWarpRuntime::default();
    let mut messages = SystemMessageUiModel::default();

    for message_id in [110, 111, 112, 113, 114, 115, 173] {
        let request_id = runtime
            .queue_system_message(&content, &mut messages, message_id)
            .unwrap();
        let definition = content.system_message_definition(message_id).unwrap();
        let request = messages.current().unwrap();
        assert_eq!(request.request_id, request_id);
        assert_eq!(request.text, definition.exact_text);
        assert_eq!(request.button_type, definition.runtime_button_type);
        assert_eq!(
            runtime.pending_system_messages.get(&request_id),
            Some(&message_id)
        );
    }
}

#[test]
fn minimap_effect_resolves_the_grounded_actor_offset_in_native_space() {
    let actor_position = ProtocolPosition::new([65_100, 73_900, -8_200]).to_native();
    assert!(
        tutorial_actor_effect_world_position(actor_position, [0.0, -1.0, 0.0])
            .abs_diff_eq(Vec3::new(-651.0, -83.0, 739.0), 0.000_01)
    );
}

#[test]
fn scripted_travel_blocks_actions_and_releases_them_after_dismount() {
    use ffone_client::avatar_action::{LegacyTargetSelection, schedule_legacy_action_frame};
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialNanoGameplayState>()
        .init_resource::<MissionUiModel>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<WorldNanoCooldownRuntime>()
        .insert_resource(inventory)
        .insert_resource(runtime_test_mission_content())
        .init_resource::<PlayerWeaponAnimationCatalog>()
        .init_resource::<TransportationProductionRuntime>()
        .add_systems(Update, sync_tutorial_action_gate);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::World);
    let player = app
        .world_mut()
        .spawn((LocalPlayer, LegacyAvatarActionContext::default()))
        .id();
    let check_actions = |app: &App, blocked: bool| {
        let mut context = app
            .world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .clone();
        assert_eq!(context.move_mode != LegacyMoveMode::None, blocked);
        assert!(
            !context.attack_locked,
            "travel must not replace the hand/equipment gate"
        );
        // Supply usable skill/equipment facts independently to prove that the
        // transport gate suppresses all three action routes, not only Fire1.
        context.nano_locked = false;
        context.nano_skill_usable = true;
        context.weapon_swap_available = true;
        for input in [
            LegacyAvatarActionInput {
                primary_held: true,
                ..default()
            },
            LegacyAvatarActionInput {
                nano_just_pressed: true,
                ..default()
            },
            LegacyAvatarActionInput {
                weapon_cycle_just_pressed: true,
                ..default()
            },
        ] {
            let result = schedule_legacy_action_frame(
                input,
                &context,
                &LegacyTargetSelection::default(),
                &mut LegacyAvatarActionState::default(),
                &LegacyAvatarClipBindings::default(),
                0.016,
            );
            assert_eq!(result.actions.is_empty(), blocked);
        }
    };
    app.update();
    check_actions(&app, false);
    app.world_mut()
        .resource_mut::<TransportationProductionRuntime>()
        .skyway_active = true;
    app.update();
    check_actions(&app, true);
    app.world_mut()
        .resource_mut::<TransportationProductionRuntime>()
        .skyway_active = false;
    app.world_mut()
        .entity_mut(player)
        .insert(WorldZiplineTraversal {
            start: Vec3::ZERO,
            end: Vec3::X,
            speed: 5.0,
            travelled: 0.0,
            packet_elapsed: 0.0,
            hang_height: 1.0,
        });
    app.update();
    check_actions(&app, true);
    app.world_mut()
        .entity_mut(player)
        .remove::<WorldZiplineTraversal>();
    // A stale visual pose must not keep input locked after travel finishes.
    app.world_mut()
        .entity_mut(player)
        .insert(LegacyAvatarPresentationContext {
            traversal: LegacyAvatarTraversalPresentation::Zipline,
            ..default()
        });
    app.update();
    check_actions(&app, false);
}

#[test]
fn tutorial_objective_combat_waits_for_the_oil_ogre_to_engage() {
    use ffone_client::tutorial_logic::ObservedNpc;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialNanoGameplayState>()
        .init_resource::<MissionUiModel>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<WorldNanoCooldownRuntime>()
        .init_resource::<LocalInventoryRuntime>()
        .insert_resource(runtime_test_mission_content())
        .init_resource::<PlayerWeaponAnimationCatalog>()
        .init_resource::<TutorialNpcObservationSnapshot>()
        .add_systems(Update, sync_tutorial_action_gate);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);
    {
        let mut tutorial = app.world_mut().resource_mut::<TutorialSession>();
        tutorial.init_chapter(3).unwrap();
        tutorial.init_step(4);
        assert_eq!(
            tutorial.progress.stage(),
            Some(TutorialStage::Mission(MissionStage::ObjectiveCombat))
        );
    }
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            LegacyAvatarActionContext::default(),
            LegacyAvatarEnvironmentState::default(),
        ))
        .id();
    let mut combat_with = |target: ObservedNpc, remaining: f32| {
        app.world_mut()
            .get_mut::<LegacyAvatarEnvironmentState>(player)
            .unwrap()
            .local_combat_timeout_remaining_seconds = remaining;
        app.world_mut()
            .resource_mut::<TutorialNpcObservationSnapshot>()
            .0
            .mission_target = target;
        app.update();
        app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .combat_condition
    };

    // Leaving Numbuh Two after accepting the mission: no DANGER overlay yet.
    assert!(!combat_with(
        ObservedNpc {
            distance: Some(120.0),
            ..default()
        },
        0.0
    ));
    assert!(!combat_with(ObservedNpc::default(), 0.0));
    // Proximity starts the NPC's attack intent, not GameFrame's combat flag.
    assert!(!combat_with(
        ObservedNpc {
            distance: Some(29.0),
            ..default()
        },
        0.0
    ));
    assert!(combat_with(
        ObservedNpc {
            distance: Some(120.0),
            damaged: true,
            ..default()
        },
        5.0
    ));
    // A previously damaged NPC cannot hold DANGER forever after activity stops.
    assert!(!combat_with(
        ObservedNpc {
            distance: Some(29.0),
            damaged: true,
            ..default()
        },
        0.0
    ));
}

#[test]
fn tutorial_actions_are_not_drained_by_the_ordinary_world_collector() {
    assert!(client_state_uses_ordinary_world_action_collector(
        ClientState::World
    ));
    assert!(!client_state_uses_ordinary_world_action_collector(
        ClientState::Tutorial
    ));
}
