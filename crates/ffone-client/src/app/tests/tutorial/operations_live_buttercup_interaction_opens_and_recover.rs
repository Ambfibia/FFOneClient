use super::*;

#[test]
fn post_cutscene_tutorial_entry_cannot_be_blocked_by_future_mob_or_effect_preloads() {
    let character = CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid: 8,
        first_name: "Test".to_owned(),
        last_name: "Ser".to_owned(),
        position: TUTORIAL_START_SERVER_POSITION,
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
            tutorial_flag: 0,
            payzone_flag: 0,
        },
        equipment: [ffone_protocol::EquippedItem0104::default();
            ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
    };
    let bridge = NetworkBridge::start();
    let mut runtime = RuntimeStatus::default();
    let mut loading = GameplayLoadingState::default();
    let mut creation_session = CharacterCreationSession::default();
    let mut creation_ui = CharacterCreationUiModel::default();
    let mut preview = NativePlayerPreviewModel::default();
    let mut tutorial = TutorialSession::default();
    let mut next_state = NextState::<ClientState>::default();

    request_character_entry(
        &character,
        &bridge,
        &mut runtime,
        &mut loading,
        &mut creation_session,
        &mut creation_ui,
        &mut preview,
        &mut tutorial,
        &mut next_state,
    );
    assert!(matches!(
        next_state,
        NextState::Pending(ClientState::TutorialIntro)
    ));

    let mut cutscene = DexterShipCutsceneRuntime::default();
    request_tutorial_world_after_cutscene(
        &tutorial,
        &bridge,
        &mut runtime,
        &mut loading,
        &mut cutscene,
    )
    .expect("the completed welcome cutscene must request tutorial world entry");

    assert!(cutscene.tutorial_world_requested);
    assert_eq!(runtime.roster.pending_character_entry_uid, Some(8));
    assert!(loading.visible);
    assert_eq!(loading.scope, Some(ResourceLoadingScope::Tutorial));
    assert_eq!(
        client_state_for_world_scope(native_world_scope(character.style.tutorial_flag).unwrap()),
        ClientState::Tutorial,
        "WorldReady must acknowledge this request as tutorial gameplay"
    );

    let plan = TutorialStartupProbePlan {
        admission: vec![
            TutorialStartupProbe::Ready,
            TutorialStartupProbe::Ready,
            TutorialStartupProbe::Ready,
        ],
        background: vec![
            TutorialStartupProbe::Loading("future tutorial mob package"),
            TutorialStartupProbe::Blocked("future tutorial effect failed".to_owned()),
        ],
    };
    assert_eq!(plan.readiness(), TutorialStartupProbe::Ready);

    for _ in 0..GAMEPLAY_LOADING_RENDER_SETTLE_FRAMES {
        loading.settle_render_presentation();
    }
    assert!(loading.settle_render_presentation());
    loading.finish();
    assert!(!loading.visible);
    assert_eq!(loading.scope, None);
}

#[test]
fn chapter_02_movement_marker_plan_matches_every_clear_and_add_effect_call() {
    use TutorialMovementStageEffect::{Clear, Marker, Preload};

    assert_eq!(
        tutorial_movement_stage_effects(TutorialStage::Movement(MovementStage::PostIntro)),
        vec![Preload { source_line: 2845 }]
    );
    assert_eq!(
        tutorial_movement_stage_effects(TutorialStage::Movement(MovementStage::LookRight)),
        vec![
            Clear { source_line: 2875 },
            Marker {
                unity_position: Vec3::new(547.0, -103.4, 647.0),
                source_line: 2876,
            },
        ]
    );
    assert_eq!(
        tutorial_movement_stage_effects(TutorialStage::Movement(MovementStage::LookUp)),
        vec![
            Clear { source_line: 2906 },
            Marker {
                unity_position: Vec3::new(554.0, -100.4, 655.0),
                source_line: 2907,
            },
        ]
    );
    assert_eq!(
        tutorial_movement_stage_effects(TutorialStage::Movement(MovementStage::MoveBackward)),
        vec![Clear { source_line: 2947 }]
    );
    assert_eq!(
        tutorial_movement_stage_effects(TutorialStage::Movement(MovementStage::MoveAndSteer)),
        vec![Clear { source_line: 2959 }]
    );
    assert_eq!(
        tutorial_movement_stage_effects(TutorialStage::Movement(MovementStage::ReachLedge)),
        vec![
            Clear { source_line: 2966 },
            Marker {
                unity_position: Vec3::new(555.82, -104.0, 654.2),
                source_line: 2970,
            },
        ]
    );
    assert_eq!(
        tutorial_movement_stage_effects(TutorialStage::Movement(MovementStage::JumpAndLand)),
        vec![
            Clear { source_line: 2981 },
            Marker {
                unity_position: Vec3::new(566.0, -99.0, 665.0),
                source_line: 2983,
            },
        ]
    );
}

#[test]
fn chapter_02_camera_look_at_is_one_shot_on_only_steps_seven_and_eleven() {
    let mut presentation = TutorialChoreographyPresentation::default();
    apply_tutorial_movement_stage_camera(
        TutorialStage::Movement(MovementStage::MoveForward),
        &mut presentation,
    );
    assert_eq!(
        presentation.camera.resolved_look_at,
        Some(unity_to_native_vector(Vec3::new(556.0, -104.4, 655.0)))
    );
    assert_eq!(presentation.camera.look_at_revision, 1);

    apply_tutorial_movement_stage_camera(
        TutorialStage::Movement(MovementStage::MoveBackward),
        &mut presentation,
    );
    assert_eq!(presentation.camera.look_at_revision, 1);

    apply_tutorial_movement_stage_camera(
        TutorialStage::Movement(MovementStage::JumpAndLand),
        &mut presentation,
    );
    assert_eq!(
        presentation.camera.resolved_look_at,
        Some(unity_to_native_vector(Vec3::new(566.0, -99.0, 665.0)))
    );
    assert_eq!(presentation.camera.look_at_revision, 2);
}

#[test]
fn chapter_02_timeout_retries_match_the_shifted_legacy_conditions() {
    for stage in [
        MovementStage::LookRight,
        MovementStage::LookLeft,
        MovementStage::LookUp,
        MovementStage::LookDown,
        MovementStage::JumpAndLand,
    ] {
        assert!(!tutorial_movement_timeout_restarts(stage, 25.0));
        assert!(tutorial_movement_timeout_restarts(stage, 25.001));
    }
    for stage in [
        MovementStage::AwaitIntro,
        MovementStage::IntroCutscene,
        MovementStage::PostIntro,
        MovementStage::MoveForward,
        MovementStage::MoveBackward,
        MovementStage::MoveAndSteer,
        MovementStage::ReachLedge,
    ] {
        assert!(!tutorial_movement_timeout_restarts(stage, 26.0));
    }
}

#[test]
fn tutorial_nano_shortcut_toggles_the_equipped_presentation() {
    assert_eq!(
        tutorial_nano_shortcut_action(false, true, false, false),
        Some(TutorialNanoShortcutAction::Summon)
    );
    assert_eq!(
        tutorial_nano_shortcut_action(false, true, true, true),
        Some(TutorialNanoShortcutAction::Dismiss)
    );
    assert_eq!(
        tutorial_nano_shortcut_action(false, true, false, true),
        Some(TutorialNanoShortcutAction::Dismiss),
        "a second press during the call animation must still recall the Nano"
    );
    assert_eq!(tutorial_nano_shortcut_action(true, true, true, true), None);
    assert_eq!(
        tutorial_nano_shortcut_action(false, false, false, false),
        None
    );
}

#[test]
fn tutorial_excludes_ordinary_world_nano_authority() {
    assert!(!client_state_uses_world_nano_authority(
        ClientState::Tutorial
    ));
    assert!(client_state_uses_world_nano_authority(ClientState::World));
    assert!(!client_state_uses_world_nano_authority(
        ClientState::CharacterSelect
    ));
}

#[test]
fn finale_names_both_effect_references_for_their_exact_destroy_calls() {
    assert_eq!(
        tutorial_named_world_effect(4697, 739),
        Some("Dexter hologram effect 739")
    );
    assert_eq!(
        tutorial_named_world_effect(4701, 741),
        Some("portal effect 741")
    );
    assert_eq!(
        tutorial_named_world_effect(4841, 739),
        Some("Dexter hologram effect 739")
    );
    assert_eq!(
        tutorial_named_world_effect(4711, 741),
        Some("portal effect 741")
    );
    assert_eq!(tutorial_named_world_effect(4697, 741), None);
    assert_eq!(tutorial_named_world_effect(4635, 739), None);
}

#[test]
fn tutorial_music_channel_is_exactly_the_eight_play_bgm_stings() {
    for cue in [
        "Flythru_Sting",
        "FusionSpawns_Sting",
        "Cyberus_Sting",
        "PlanetFusion_Sting",
        "InfectedZone_Sting",
        "SCAMPER_Sting",
        "TechWing_sting",
        "DexCarrier_sting",
    ] {
        assert!(tutorial_audio_cue_is_music(cue), "{cue}");
    }
    for cue in [
        "Explosion_01",
        "TechSquareText_Typed",
        "DexterHologram_LOOP",
        "Computress_Tut07",
    ] {
        assert!(!tutorial_audio_cue_is_music(cue), "{cue}");
    }
}

#[test]
fn tutorial_npc_attack_is_exactly_fifty_with_floor_one_hundred() {
    assert_eq!(apply_tutorial_player_damage(1_000, 50), 950);
    assert_eq!(apply_tutorial_player_damage(125, 50), 100);
    assert_eq!(apply_tutorial_player_damage(100, 50), 100);
}

#[test]
fn tutorial_ambient_routes_only_exact_source_cues_and_none_substrings() {
    assert_eq!(
        tutorial_ambient_cue_from_legacy(TUTORIAL_MAIN_AMBIENT_TRUE_NAME).unwrap(),
        Some(TutorialAmbientCue::TutorialMain)
    );
    assert_eq!(
        tutorial_ambient_cue_from_legacy(TUTORIAL_LAIR_AMBIENT_TRUE_NAME).unwrap(),
        Some(TutorialAmbientCue::ButtercupLair)
    );
    assert_eq!(tutorial_ambient_cue_from_legacy("none").unwrap(), None);
    assert_eq!(
        tutorial_ambient_cue_from_legacy("legacy_NONE_sentinel").unwrap(),
        None
    );
    assert!(tutorial_ambient_cue_from_legacy("invented ambient").is_err());
    assert_eq!(
        tutorial_ambient_after_confirmed_warp(254),
        Some(TutorialAmbientCue::ButtercupLair)
    );
    assert_eq!(tutorial_ambient_after_confirmed_warp(253), None);
    assert_eq!(tutorial_ambient_after_confirmed_warp(255), None);
    assert_eq!(
        tutorial_ambient_after_chapter_init(TutorialStage::NanoPower(
            NanoPowerStage::AwaitCollapse
        )),
        Some(TutorialAmbientCue::TutorialMain)
    );
    assert_eq!(
        tutorial_ambient_after_chapter_init(TutorialStage::Infection(InfectionStage::WarpOut)),
        None
    );
}

#[test]
fn tutorial_ambient_crossfade_has_exact_ten_fifty_millisecond_steps() {
    assert_eq!(TUTORIAL_AMBIENT_FADE_STEPS, 10);
    assert_eq!(TUTORIAL_AMBIENT_SECONDS_PER_STEP, 0.05);
    assert_eq!(
        f32::from(TUTORIAL_AMBIENT_FADE_STEPS) * TUTORIAL_AMBIENT_SECONDS_PER_STEP,
        0.5
    );
    for step in 0..=TUTORIAL_AMBIENT_FADE_STEPS {
        let expected_in = f32::from(step) / 10.0;
        assert!((tutorial_ambient_fade_volume(true, step) - expected_in).abs() <= f32::EPSILON);
        assert!(
            (tutorial_ambient_fade_volume(false, step) - (1.0 - expected_in)).abs() <= f32::EPSILON
        );
    }
}

#[test]
fn tutorial_startup_probe_is_fail_closed_and_blockers_outrank_loading() {
    assert_eq!(
        combine_tutorial_startup_probes(
            [TutorialStartupProbe::Ready, TutorialStartupProbe::Ready,]
        ),
        TutorialStartupProbe::Ready
    );
    assert_eq!(
        combine_tutorial_startup_probes([
            TutorialStartupProbe::Ready,
            TutorialStartupProbe::Loading("hidden exact tutorial Nano"),
            TutorialStartupProbe::Ready,
        ]),
        TutorialStartupProbe::Loading("hidden exact tutorial Nano")
    );
    assert_eq!(
        combine_tutorial_startup_probes([
            TutorialStartupProbe::Loading("exact tutorial dome"),
            TutorialStartupProbe::Blocked("NpcTexture closure failed".to_owned()),
        ]),
        TutorialStartupProbe::Blocked("NpcTexture closure failed".to_owned())
    );
}

#[test]
fn tutorial_task_chain_is_source_owned_ordered_and_idempotent() {
    let content = runtime_test_mission_content();
    let mut runtime = TutorialMissionRuntime::default();
    assert!(runtime.complete_task(&content, 2249).is_err());

    assert!(runtime.start_task(&content, 2248).unwrap());
    assert_eq!(runtime.selected_mission_id, Some(1));
    assert!(!runtime.start_task(&content, 2248).unwrap());
    let mutation = runtime.complete_task(&content, 2248).unwrap();
    assert_eq!(mutation.completion_audio_true_name(), Some("Task_Completed"));
    assert_eq!(mutation.outgoing_task_id, Some(2249));
    assert!(mutation.outgoing_started);
    assert_eq!(runtime.active_tasks, vec![2249]);
    assert_eq!(runtime.selected_mission_id, Some(1));
    assert_eq!(
        runtime
            .complete_task(&content, 2249)
            .unwrap()
            .completion_audio_true_name(),
        Some("Mission_Completed")
    );
    assert!(runtime.active_tasks.is_empty());
    assert_eq!(runtime.selected_mission_id, None);

    assert!(runtime.start_task(&content, 2250).unwrap());
    assert_eq!(runtime.selected_mission_id, Some(2));
    for (task_id, outgoing) in [
        (2250, Some(2251)),
        (2251, Some(2252)),
        (2252, Some(2253)),
        (2253, Some(2254)),
        (2254, None),
    ] {
        let mutation = runtime.complete_task(&content, task_id).unwrap();
        assert_eq!(
            mutation.completion_audio_true_name(),
            Some(if outgoing.is_some() {
                "Task_Completed"
            } else {
                "Mission_Completed"
            })
        );
        assert!(mutation.state_changed);
        assert_eq!(mutation.outgoing_task_id, outgoing);
        assert_eq!(mutation.outgoing_started, outgoing.is_some());
        assert_eq!(
            runtime.active_tasks,
            outgoing.into_iter().collect::<Vec<_>>()
        );
    }
    let duplicate = runtime.complete_task(&content, 2254).unwrap();
    assert_eq!(duplicate.completion_audio_true_name(), None);
    assert!(!duplicate.state_changed);
    assert!(!duplicate.outgoing_started);
    let unique = runtime
        .completed_tasks
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), runtime.completed_tasks.len());
}

#[test]
fn live_buttercup_interaction_opens_and_recovers_the_nano_mission_list() {
    let content = runtime_test_mission_content();
    let mut mission_runtime = TutorialMissionRuntime::default();
    assert!(mission_runtime.start_task(&content, 2250).unwrap());

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(content)
        .insert_resource(mission_runtime)
        .init_resource::<TutorialSession>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialLogicRuntime>()
        .add_systems(Update, sync_tutorial_mission_interaction);
    let buttercup = app
        .world_mut()
        .spawn(TutorialActor {
            id: ffone_client::tutorial_logic::BUTTERCUP_ID,
            npc_type: 2672,
            team: 1,
            hp: 100,
            max_hp: 100,
            damaged: false,
            interacting: true,
            invulnerable: false,
        })
        .id();
    {
        let mut tutorial = app.world_mut().resource_mut::<TutorialSession>();
        tutorial.progress.init_chapter(4).unwrap();
        tutorial
            .progress
            .init_step(InfectionStage::SelectMission as i16);
    }

    app.update();
    {
        let model = app.world().resource::<MissionUiModel>();
        assert!(model.enabled);
        assert!(model.npc_icon_mode_visible);
        let interaction = model
            .npc_interaction
            .as_ref()
            .expect("Buttercup NpcIconMode interaction");
        assert_eq!(
            interaction.npc_id,
            ffone_client::tutorial_logic::BUTTERCUP_ID
        );
        assert_eq!(interaction.npc_type, 2672);
        assert_eq!(
            interaction
                .completed_missions
                .iter()
                .map(|mission| (mission.task_id, mission.title.as_str()))
                .collect::<Vec<_>>(),
            vec![(2250, "A Fusion Matter")]
        );
    }

    // Chapter 06 owns the selection modal until task 2250 advances. A
    // transient interaction reset must not dismiss it before the player can
    // select the row.
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .npc_icon_mode_visible = false;
    app.world_mut()
        .entity_mut(buttercup)
        .get_mut::<TutorialActor>()
        .unwrap()
        .interacting = false;
    app.update();
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .npc_icon_mode_visible
    );

    let mut outbox = GameplayUiOutbox::default();
    assert!(
        app.world_mut()
            .resource_mut::<MissionUiModel>()
            .select_npc_mission(0, &mut outbox)
    );
    app.update();
    let model = app.world().resource::<MissionUiModel>();
    // Clean NpcIconMode submits this rewardless first-Nano TaskEnd directly:
    // the NPC list stays up while pending and no ACCEPT MISSION journal opens.
    assert!(model.npc_icon_mode_visible);
    assert!(matches!(
        model.journal,
        ffone_client::mission_ui::MissionJournalUi::Hidden
    ));
    assert!(matches!(
        model.pending,
        Some(PendingMissionUiRequest::QuestEnd { task_id: 2250, .. })
    ));
}

#[test]
fn tutorial_first_nano_task_start_enqueues_exactly_one_type_10_nanocom_notice() {
    let content = runtime_test_mission_content();
    let mission = content.mission(2250).expect("tutorial Buttercup Nano task");
    let source = mission
        .start_nanocom_message
        .as_ref()
        .expect("task 2250 start edge must own NanoCom copy");
    let nano = content.gameplay_nano(1).expect("Buttercup Nano definition");
    let mut runtime = TutorialMissionRuntime::default();
    let mut messages = NanocomMessageUiModel::default();

    for _ in 0..2 {
        if runtime.start_task(&content, 2250).unwrap() {
            enqueue_mission_nanocom(2250, MissionNanocomEdge::Start, &content, &mut messages);
        }
    }

    assert_eq!(
        messages.len(),
        1,
        "idempotent task start must not notify twice"
    );
    let request = &messages.active().unwrap().request;
    assert_eq!(request.kind, NanocomMessageKind::Nano);
    assert_eq!(
        request.compact_title_localized().key,
        ffone_client::nanocom_message_ui::NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY
    );
    assert_eq!(
        request.compact_body_localized(10.0).key,
        format!(
            "content.tabledata.mission.mission_string.{}.str_name_string",
            source.string_id
        )
    );
    assert_eq!(request.compact_icon_path.as_ref(), nano.icon_path.as_ref());
    assert_eq!(
        request.voice_true_name.is_some(),
        !content
            .gameplay_npc(source.npc_type)
            .unwrap()
            .move_voice_owner
            .is_empty(),
        "voice must follow the source NPC CommOut owner"
    );
    assert_eq!(
        messages.pop_sound(),
        Some(ffone_client::nanocom_message_ui::NanocomMessageSound::SlideIn)
    );
    assert_eq!(
        messages.pop_sound(),
        Some(ffone_client::nanocom_message_ui::NanocomMessageSound::NanoCreationComplete),
        "the clean type-10 NanoCom path must play Nano_Creation_Complete"
    );
    assert_eq!(messages.pop_sound(), None);
}

#[test]
fn tutorial_nanocom_context_tracks_scene_events_that_must_freeze_notifications() {
    let mut app = super::super::social::nanocom_context_test_app(ClientState::Tutorial);
    app.world_mut()
        .resource_mut::<TutorialChoreographyPresentation>()
        .event_scene = true;
    app.update();
    assert!(
        app.world()
            .resource::<TutorialNanocomMessageContext>()
            .scene_event_active,
        "the NanoCom lifetime must be frozen while the tutorial scene owns the screen"
    );

    app.world_mut()
        .resource_mut::<TutorialChoreographyPresentation>()
        .event_scene = false;
    app.update();
    assert!(
        !app.world()
            .resource::<TutorialNanocomMessageContext>()
            .scene_event_active,
        "the queued Nano mission notice must be released after the scene"
    );
}

#[test]
fn tutorial_npc_mission_symbols_follow_retrobution_priority_and_effect_ids() {
    let actor = |npc_type| TutorialActor {
        id: 1005,
        npc_type,
        team: 1,
        hp: 100,
        max_hp: 100,
        damaged: false,
        interacting: false,
        invulnerable: false,
    };
    let mut mission = TutorialMissionRuntime::default();
    let new = tutorial_npc_mission_symbol(&actor(2671), &mission).unwrap();
    assert_eq!((new, new.effect_id()), (MinimapMissionSymbol::New, 866));

    mission.active_tasks.push(2248);
    assert_eq!(tutorial_npc_mission_symbol(&actor(2671), &mission), None);
    mission.active_tasks = vec![2249];
    let advance = tutorial_npc_mission_symbol(&actor(2671), &mission).unwrap();
    assert_eq!(
        (advance, advance.effect_id()),
        (MinimapMissionSymbol::Advance, 865)
    );
    mission.active_tasks = vec![2250];
    assert_eq!(
        tutorial_npc_mission_symbol(&actor(2672), &mission),
        Some(MinimapMissionSymbol::Advance)
    );
    mission.active_tasks = vec![2253];
    assert_eq!(
        tutorial_npc_mission_symbol(&actor(2673), &mission),
        Some(MinimapMissionSymbol::Advance)
    );
}

#[test]
fn tutorial_minimap_uses_regular_icons_and_mission_replacements() {
    for npc_type in 2671..=2673 {
        assert_eq!(
            tutorial_minimap_marker_icon(npc_type, None),
            Some(MinimapMarkerIcon::ShowNpc)
        );
    }
    for npc_type in 2674..=2677 {
        assert_eq!(
            tutorial_minimap_marker_icon(npc_type, None),
            Some(MinimapMarkerIcon::Mob)
        );
    }
    assert_eq!(
        tutorial_minimap_marker_icon(2678, None),
        Some(MinimapMarkerIcon::Fusion)
    );
    assert_eq!(tutorial_minimap_marker_icon(2374, None), None);
    assert_eq!(
        tutorial_minimap_marker_icon(2671, Some(MinimapMissionSymbol::New)),
        Some(MinimapMarkerIcon::New)
    );
    assert_eq!(
        tutorial_minimap_marker_icon(2672, Some(MinimapMissionSymbol::Advance)),
        Some(MinimapMarkerIcon::Advance)
    );
}
