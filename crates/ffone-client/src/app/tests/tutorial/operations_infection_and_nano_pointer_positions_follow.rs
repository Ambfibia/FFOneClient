use super::*;

#[test]
fn tutorial_hud_visibility_combines_cinematic_modal_and_gm_ownership() {
    let mut app = App::new();
    app.init_resource::<RuntimeStatus>()
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<MissionUiModel>()
        .add_systems(Update, sync_tutorial_choreography_visibility);
    let hud = app
        .world_mut()
        .spawn((GameplayHud, Visibility::Inherited))
        .id();
    {
        let mut runtime = app.world_mut().resource_mut::<RuntimeStatus>();
        runtime.player_id = Some(1);
        runtime.user_level = 30;
    }
    // Closing one owner must not reveal the HUD while another still hides it.
    for (depth, modal, gm, expected) in [
        (1, false, false, Visibility::Hidden),
        (1, false, true, Visibility::Hidden),
        (1, false, false, Visibility::Hidden),
        (0, false, true, Visibility::Hidden),
        (0, true, false, Visibility::Hidden),
        (0, false, false, Visibility::Inherited),
    ] {
        app.world_mut()
            .resource_mut::<TutorialChoreographyPresentation>()
            .hud_hide_depth = depth;
        app.world_mut()
            .resource_mut::<RuntimeStatus>()
            .chat
            .gm
            .hide_ui = gm;
        app.world_mut()
            .resource_mut::<MissionUiModel>()
            .warp_transition_active = modal;
        for _ in 0..16 {
            app.update();
            assert_eq!(*app.world().get::<Visibility>(hud).unwrap(), expected);
        }
    }
    let mut runtime = app.world_mut().resource_mut::<RuntimeStatus>();
    runtime.user_level = 100;
    runtime.chat.gm.hide_ui = true;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Inherited
    );
}

#[test]
fn tutorial_buttercup_mission_dialogue_edges_are_once_and_ordered() {
    let content = runtime_test_mission_content();
    let mut runtime = TutorialMissionRuntime::default();
    assert!(runtime.start_task(&content, 2250).unwrap());
    assert!(!runtime.start_task(&content, 2250).unwrap());
    runtime.complete_task(&content, 2250).unwrap();
    runtime.complete_task(&content, 2250).unwrap();
    assert_eq!(
        runtime.dialogue_edges,
        vec![
            (2250, WorldMissionDialogueEdge::Start),
            (2250, WorldMissionDialogueEdge::Complete),
            (2251, WorldMissionDialogueEdge::Start),
        ]
    );
    let dialogue = content
        .mission(2250)
        .unwrap()
        .success_dialogue
        .as_ref()
        .unwrap();
    assert_eq!(dialogue.npc_type, 2672);
    assert_eq!(dialogue.string_id, 11684);
}

#[test]
fn tutorial_chat_history_appends_each_stage_once_and_keeps_the_last_fifty_lines() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, language) = Localization::open(&asset_root, "en").unwrap();
    let stage = TutorialStage::Movement(MovementStage::LookRight);
    let mut runtime = TutorialMissionRuntime::default();

    runtime.push_voice_chat_line(String::from("COMPUTRESS: Voice line."));
    append_tutorial_stage_instruction(None, stage, &mut runtime, &localization, &language);
    let entered_history = runtime.chat_lines.clone();
    append_tutorial_stage_instruction(Some(stage), stage, &mut runtime, &localization, &language);

    assert_eq!(runtime.chat_lines, entered_history);
    assert_eq!(runtime.chat_lines.len(), 2);
    assert_eq!(
        runtime.chat_lines[0].kind,
        ffone_client::gameplay_ui::ChatLineKind::Normal
    );
    assert_eq!(
        runtime.chat_lines[1].kind,
        ffone_client::gameplay_ui::ChatLineKind::Tutorial
    );
    assert_eq!(
        runtime.chat_lines[1].text,
        "Move your mouse to the right and find the marker."
    );

    for index in 0..=TUTORIAL_CHAT_HISTORY_LIMIT {
        runtime.push_voice_chat_line(format!("line {index}"));
    }
    assert_eq!(runtime.chat_lines.len(), TUTORIAL_CHAT_HISTORY_LIMIT);
    assert_eq!(runtime.chat_lines.first().unwrap().text, "line 1");
    assert_eq!(runtime.chat_lines.last().unwrap().text, "line 50");
}

#[test]
fn live_tutorial_issue_reporter_surfaces_and_drains_actionable_failures() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<RuntimeStatus>()
        .init_resource::<TutorialChoreographyIssueQueue>()
        .init_resource::<TutorialEffectRuntime>()
        .init_resource::<TutorialActorIssueQueue>()
        .init_resource::<TutorialPlayerRigIssueQueue>()
        .add_systems(Update, report_tutorial_runtime_issues);
    for source_line in 4717..4723 {
        app.world_mut()
            .resource_mut::<TutorialChoreographyIssueQueue>()
            .push(TutorialChoreographyIssue::MissingEntityReference {
                source_line,
                detail: "finale NPC is unavailable",
            });
    }

    app.update();

    let runtime = app.world().resource::<RuntimeStatus>();
    assert!(runtime.message.contains("finale NPC is unavailable"));
    assert!(runtime.message.contains("+2 more"));
    assert_eq!(runtime.diagnostics.tutorial_issue_history.len(), 6);
    assert!(
        runtime
            .diagnostics
            .tutorial_issue_history
            .back()
            .unwrap()
            .contains("source_line: 4722")
    );
    assert!(
        app.world()
            .resource::<TutorialChoreographyIssueQueue>()
            .is_empty()
    );
}

#[test]
fn dexter_ship_drive_queries_are_pairwise_disjoint() {
    let mut world = World::new();
    let _ = bevy::ecs::system::SystemState::<DexterShipDriveQueries>::new(&mut world);
}

#[test]
fn tutorial_from_to_camera_uses_authored_temp_position_and_interpolates_focus() {
    let mut camera = ffone_client::tutorial_choreography_runtime::TutorialCameraPresentation {
        mode: CameraMode::FromToCamera,
        resolved_start: Some(Vec3::new(10.0, 0.0, 10.0)),
        resolved_stored_start: Some(Vec3::new(0.0, 0.0, 10.0)),
        resolved_current_target: Some(Vec3::new(-10.0, 0.0, 0.0)),
        resolved_target: Some(Vec3::ZERO),
        ..default()
    };
    let (mut app, camera_entity) =
        tutorial_camera_test_app(Transform::from_xyz(100.0, 0.0, 100.0), camera.clone());
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.5));
    app.update();

    let transform = app
        .world()
        .entity(camera_entity)
        .get::<Transform>()
        .unwrap();
    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(5.0, 0.0, 10.0), 0.0001)
    );
    let expected_forward = (Vec3::new(-5.0, 0.0, 0.0) - transform.translation).normalize();
    assert!(
        transform
            .forward()
            .as_vec3()
            .abs_diff_eq(expected_forward, 0.0001)
    );

    camera.resolved_stored_start = None;
    camera.resolved_start = Some(Vec3::new(5.0, 0.0, 10.0));
    camera.resolved_target = Some(Vec3::new(10.0, 0.0, 0.0));
    app.insert_resource(TutorialChoreographyPresentation {
        camera: camera.clone(),
        ..default()
    });
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.5));
    app.update();

    let transform = app
        .world()
        .entity(camera_entity)
        .get::<Transform>()
        .unwrap();
    let expected_forward = (Vec3::new(2.5, 0.0, 0.0) - transform.translation).normalize();
    assert!(
        transform
            .forward()
            .as_vec3()
            .abs_diff_eq(expected_forward, 0.0001)
    );

    camera.resolved_current_target = Some(Vec3::new(20.0, 0.0, 0.0));
    app.insert_resource(TutorialChoreographyPresentation {
        camera,
        ..default()
    });
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.5));
    app.update();

    let transform = app
        .world()
        .entity(camera_entity)
        .get::<Transform>()
        .unwrap();
    let expected_forward = (Vec3::new(15.0, 0.0, 0.0) - transform.translation).normalize();
    assert!(
        transform
            .forward()
            .as_vec3()
            .abs_diff_eq(expected_forward, 0.0001)
    );
}

#[test]
fn dexter_ship_background_preserves_four_by_three_and_never_shrinks() {
    let (origin, size) = dexter_ship_background_layout(Vec2::new(1_280.0, 720.0));
    assert_eq!(size, Vec2::new(1_920.0, 1_440.0));
    assert_eq!(origin, Vec2::new(-320.0, -360.0));

    let (origin, size) = dexter_ship_background_layout(Vec2::new(2_560.0, 1_440.0));
    assert!((size.x - 2_560.0).abs() < 0.001);
    assert!((size.y - 1_920.0).abs() < 0.001);
    assert!((origin.x - 0.0).abs() < 0.001);
    assert!((origin.y + 240.0).abs() < 0.001);
}

#[test]
fn dexter_ship_location_title_types_unicode_then_fades_with_the_black_screen() {
    assert_eq!(dexter_ship_typewriter_text("ЛАБОРАТОРИЯ", 0.0), "Л");
    assert_eq!(dexter_ship_typewriter_text("ЛАБОРАТОРИЯ", 0.10), "ЛАБ");
    assert_eq!(
        dexter_ship_typewriter_text("ЛАБОРАТОРИЯ", 10.0),
        "ЛАБОРАТОРИЯ"
    );
    assert_eq!(dexter_ship_name_title_alpha(0.0), 1.0);
    assert_eq!(dexter_ship_name_title_alpha(2.0), 1.0);
    assert_eq!(dexter_ship_name_title_alpha(2.5), 0.5);
    assert_eq!(dexter_ship_name_title_alpha(3.0), 0.0);
}

#[test]
fn tutorial_weapon_profiles_match_recovered_item_rows() {
    let melee = tutorial_weapon_combat_profile(Some(43));
    assert_eq!(
        (
            melee.half_angle_degrees,
            melee.range,
            melee.cooldown_seconds,
            melee.target_capacity,
            melee.bullet_type,
            melee.fire_link,
        ),
        (90.0, 3.0, 1.0, 3, Some(5), None)
    );
    let pistol = tutorial_weapon_combat_profile(Some(197));
    assert_eq!(
        (
            pistol.half_angle_degrees,
            pistol.range,
            pistol.cooldown_seconds,
            pistol.target_capacity,
            pistol.bullet_type,
            pistol.fire_link,
        ),
        (20.0, 12.0, 0.8, 1, Some(113), Some("Gtag01"))
    );
    let sniper = tutorial_weapon_combat_profile(Some(328));
    assert_eq!(
        (
            sniper.half_angle_degrees,
            sniper.range,
            sniper.cooldown_seconds,
            sniper.target_capacity,
            sniper.bullet_type,
            sniper.fire_link,
        ),
        (20.0, 16.0, 1.0, 1, Some(13), Some("Gtag01"))
    );
}

#[test]
fn sampled_projectile_commands_consume_one_continuous_six_then_four_draw_stream() {
    let seed = 0x1234_5678;
    let mut sampler = TutorialProjectileRandomStream::with_seed(seed);
    let mut expected_state = seed;
    let mut normal_draws = [0.0; 6];
    for draw in &mut normal_draws {
        let value = advance_native_xorshift32(&mut expected_state);
        *draw = (f64::from(value) / f64::from(u32::MAX)) as f32;
    }
    let expected_normal =
        tutorial_oni_velocity_samples_from_unity_unit_draws(false, &normal_draws).unwrap();
    let normal = sampled_oni_projectile_pair_command(
        [76, 77],
        Vec3::ZERO,
        Vec3::ONE,
        false,
        3199,
        &mut sampler,
    );
    assert!(matches!(
        normal,
        TutorialEffectRuntimeCommand::ProjectilePairSampled {
            reverse: false,
            sampled_initial_velocity,
            ..
        } if sampled_initial_velocity == expected_normal
    ));
    assert_eq!(
        sampler.state, expected_state,
        "normal mode must consume six draws"
    );

    let mut reverse_draws = [0.0; 4];
    for draw in &mut reverse_draws {
        let value = advance_native_xorshift32(&mut expected_state);
        *draw = (f64::from(value) / f64::from(u32::MAX)) as f32;
    }
    let expected_reverse =
        tutorial_oni_velocity_samples_from_unity_unit_draws(true, &reverse_draws).unwrap();
    let reverse = sampled_oni_projectile_pair_command(
        [76, 77],
        Vec3::ONE,
        Vec3::ZERO,
        true,
        4042,
        &mut sampler,
    );
    assert!(matches!(
        reverse,
        TutorialEffectRuntimeCommand::ProjectilePairSampled {
            reverse: true,
            sampled_initial_velocity,
            ..
        } if sampled_initial_velocity == expected_reverse
    ));
    assert_eq!(
        sampler.state, expected_state,
        "reverse mode must consume four draws"
    );
}

#[test]
fn chapter_five_pointer_positions_follow_the_live_retrobution_screen_formulas() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, language) = Localization::open(&asset_root, "en").unwrap();
    let arrow = |stage, elapsed, width, height| {
        tutorial_ui_for_stage(
            TutorialStage::Mission(stage),
            elapsed,
            width,
            height,
            &localization,
            &language,
        )
        .arrow
        .expect("mission stage arrow")
    };

    let accept = arrow(MissionStage::AcceptMission, 0.0, 1264, 681);
    assert_eq!(accept.scale_pivot, TutorialCueScalePivot::Center);
    assert_eq!(
        accept.position,
        ffone_client::gameplay_ui::TutorialCuePosition::TopLeft {
            left: 534.0,
            top: 503.0,
        }
    );
    assert_eq!(
        arrow(MissionStage::AcceptMission, 0.0, 1920, 1080).position,
        ffone_client::gameplay_ui::TutorialCuePosition::TopLeft {
            left: 862.0,
            top: 703.0,
        }
    );
    let journal_detail = arrow(MissionStage::JournalDetail, 0.0, 1264, 681);
    assert_eq!(journal_detail.scale_pivot, TutorialCueScalePivot::Center);
    assert_eq!(
        journal_detail.position,
        ffone_client::gameplay_ui::TutorialCuePosition::TopLeft {
            left: 409.0,
            top: 393.0,
        }
    );
    let close_accept = arrow(MissionStage::CloseAccept, 0.0, 1264, 681);
    assert_eq!(
        close_accept.scale_pivot,
        TutorialCueScalePivot::Point(Vec2::new(948.0, 340.0))
    );
    assert_eq!(
        close_accept.position,
        ffone_client::gameplay_ui::TutorialCuePosition::TopLeft {
            left: 1094.0,
            top: 314.0,
        }
    );
    let early_menu_ui = tutorial_ui_for_stage(
        TutorialStage::Mission(MissionStage::MissionMenu),
        4.49,
        1264,
        681,
        &localization,
        &language,
    );
    assert_eq!(early_menu_ui.instruction, None);
    assert_eq!(early_menu_ui.secondary_instruction, None);
    let early_menu = early_menu_ui.arrow.expect("early mission-menu pointer");
    assert_eq!(
        early_menu.scale_pivot,
        TutorialCueScalePivot::Point(Vec2::new(632.0, 170.0))
    );
    assert_eq!(
        early_menu.position,
        ffone_client::gameplay_ui::TutorialCuePosition::TopLeft {
            left: 532.0,
            top: 170.0,
        }
    );
    let late_menu_ui = tutorial_ui_for_stage(
        TutorialStage::Mission(MissionStage::MissionMenu),
        4.5,
        1264,
        681,
        &localization,
        &language,
    );
    assert_eq!(late_menu_ui.instruction, None);
    assert_eq!(late_menu_ui.secondary_instruction, None);
    let late_menu = late_menu_ui.arrow.expect("late mission-menu pointer");
    assert_eq!(
        late_menu.scale_pivot,
        TutorialCueScalePivot::Point(Vec2::new(948.0, 340.0))
    );
    assert_eq!(
        late_menu.position,
        ffone_client::gameplay_ui::TutorialCuePosition::TopLeft {
            left: 702.0,
            top: 299.0,
        }
    );

    for stage in [MissionStage::CloseAccept, MissionStage::CloseReward] {
        let ui = tutorial_ui_for_stage(
            TutorialStage::Mission(stage),
            0.0,
            1264,
            681,
            &localization,
            &language,
        );
        assert_eq!(ui.instruction, None);
        assert_eq!(
            ui.secondary_instruction.as_deref(),
            Some("CLICK THE \"CLOSE\" BUTTON.")
        );
    }
}

#[test]
fn infection_and_nano_pointer_positions_follow_the_live_retrobution_screen_formulas() {
    use ffone_client::gameplay_ui::TutorialCuePosition;

    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, language) = Localization::open(&asset_root, "en").unwrap();
    let arrow = |stage, width, height| {
        tutorial_ui_for_stage(stage, 0.0, width, height, &localization, &language)
            .arrow
            .expect("tutorial stage arrow")
    };

    for stage in [
        InfectionStage::ApproachButtercup,
        InfectionStage::TalkButtercup,
        InfectionStage::TravelToGate,
        InfectionStage::SelectAttendant,
    ] {
        let cue = arrow(TutorialStage::Infection(stage), 1264, 681);
        assert_eq!(cue.direction, TutorialArrowDirection::Right);
        assert_eq!(cue.scale_pivot, TutorialCueScalePivot::TopRight);
        assert_eq!(
            cue.position,
            TutorialCuePosition::TopLeft {
                left: 1005.0,
                top: 200.0,
            }
        );

        assert_eq!(
            arrow(TutorialStage::Infection(stage), 1920, 1080).position,
            TutorialCuePosition::TopLeft {
                left: 1661.0,
                top: 200.0,
            }
        );
    }

    for stage in [
        InfectionStage::SelectMission,
        InfectionStage::SelectDexterMission,
    ] {
        let ui = tutorial_ui_for_stage(
            TutorialStage::Infection(stage),
            0.0,
            1264,
            681,
            &localization,
            &language,
        );
        assert_eq!(ui.instruction, None);
        assert_eq!(
            ui.secondary_instruction.as_deref(),
            Some("SELECT \"A FUSION MATTER\" FROM MISSION MENU.")
        );
        let cue = ui.arrow.expect("infection mission-menu pointer");
        assert_eq!(
            cue.scale_pivot,
            TutorialCueScalePivot::Point(Vec2::new(948.0, 340.0))
        );
        assert_eq!(
            cue.position,
            TutorialCuePosition::TopLeft {
                left: 702.0,
                top: 299.0,
            }
        );
        assert_eq!(
            arrow(TutorialStage::Infection(stage), 1920, 1080).position,
            TutorialCuePosition::TopLeft {
                left: 1194.0,
                top: 499.0,
            }
        );
    }

    let mut stale_runtime = TutorialMissionRuntime::default();
    stale_runtime.auxiliary_dialogue = Some(TutorialDialogue::TalkButtercup);
    let stale_buttercup = TutorialAuxiliaryPresentation {
        sequence: Some(TutorialAuxiliarySequence::TalkButtercup1),
        cursor_touched: true,
        ..default()
    };
    assert!(tutorial_auxiliary_presentation_matches(
        &stale_buttercup,
        &stale_runtime,
        Some(TutorialStage::Infection(InfectionStage::TalkButtercup))
    ));
    assert!(!tutorial_auxiliary_presentation_matches(
        &stale_buttercup,
        &stale_runtime,
        Some(TutorialStage::Infection(InfectionStage::SelectMission))
    ));

    for stage in [
        InfectionStage::CloseMission,
        InfectionStage::WarpFromTechSquare,
        InfectionStage::WarpIntoLair,
        InfectionStage::WarpOut,
    ] {
        let ui = tutorial_ui_for_stage(
            TutorialStage::Infection(stage),
            0.0,
            1264,
            681,
            &localization,
            &language,
        );
        assert_eq!(ui.instruction, None);
        assert!(ui.secondary_instruction.is_some());
    }

    for stage in [
        TutorialStage::Mission(MissionStage::CloseAccept),
        TutorialStage::Mission(MissionStage::CloseReward),
        TutorialStage::Infection(InfectionStage::CloseMission),
    ] {
        let cue = arrow(stage, 1264, 681);
        assert_eq!(cue.direction, TutorialArrowDirection::Left);
        assert_eq!(
            cue.scale_pivot,
            TutorialCueScalePivot::Point(Vec2::new(948.0, 340.0))
        );
        assert_eq!(
            cue.position,
            TutorialCuePosition::TopLeft {
                left: 1094.0,
                top: 314.0,
            }
        );

        assert_eq!(
            arrow(stage, 1920, 1080).position,
            TutorialCuePosition::TopLeft {
                left: 1586.0,
                top: 514.0,
            }
        );
    }

    for stage in [
        InfectionStage::WarpFromTechSquare,
        InfectionStage::WarpIntoLair,
        InfectionStage::WarpOut,
    ] {
        let cue = arrow(TutorialStage::Infection(stage), 1264, 681);
        assert_eq!(cue.direction, TutorialArrowDirection::Right);
        assert_eq!(
            cue.scale_pivot,
            TutorialCueScalePivot::Point(Vec2::new(948.0, 340.0))
        );
        assert_eq!(
            cue.position,
            TutorialCuePosition::TopLeft {
                left: 747.0,
                top: 303.0,
            }
        );

        assert_eq!(
            arrow(TutorialStage::Infection(stage), 1920, 1080).position,
            TutorialCuePosition::TopLeft {
                left: 1239.0,
                top: 503.0,
            }
        );
    }

    assert!(
        tutorial_ui_for_stage(
            TutorialStage::Infection(InfectionStage::SelectTentacles),
            0.0,
            1264,
            681,
            &localization,
            &language,
        )
        .arrow
        .is_none()
    );

    let nano = arrow(
        TutorialStage::NanoPower(NanoPowerStage::SummonNano),
        1264,
        681,
    );
    assert_eq!(nano.direction, TutorialArrowDirection::Down);
    assert_eq!(nano.scale_pivot, TutorialCueScalePivot::BottomRight);
    assert_eq!(
        nano.position,
        TutorialCuePosition::TopLeft {
            left: 1024.0,
            top: 501.0,
        }
    );
    assert_eq!(
        arrow(
            TutorialStage::NanoPower(NanoPowerStage::SummonNano),
            1920,
            1080,
        )
        .position,
        TutorialCuePosition::TopLeft {
            left: 1680.0,
            top: 900.0,
        }
    );
}

#[test]
fn entering_a_tutorial_character_runs_the_intro_then_selects_the_shard_character() {
    let character = CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid: 8,
        first_name: "Test".to_owned(),
        last_name: "Ser".to_owned(),
        position: [632_032, 187_177, -5_500],
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

    assert_eq!(runtime.roster.selected_uid, Some(character.pc_uid));
    assert_eq!(tutorial.character(), Some(&character));
    assert_eq!(
        tutorial.progress.stage(),
        Some(TutorialStage::Movement(MovementStage::AwaitIntro))
    );
    assert!(matches!(
        next_state,
        NextState::Pending(ClientState::TutorialIntro)
    ));
    let NetworkCommand::SelectCharacter { pc_uid, location } =
        tutorial_world_entry_command(&tutorial).expect("tutorial shard entry command")
    else {
        panic!("tutorial intro must finish with SelectCharacter");
    };
    assert_eq!(pc_uid, character.pc_uid);
    assert_eq!(
        location,
        CharacterEntryLocation0104::Scripted {
            position: TUTORIAL_START_SERVER_POSITION,
            angle: TUTORIAL_START_ANGLE,
        }
    );
    assert!(
        ProtocolPosition::new(TUTORIAL_START_SERVER_POSITION)
            .to_native()
            .abs_diff_eq(
                unity_to_native_vector(TUTORIAL_START_UNITY_POSITION),
                0.000_1
            ),
    );
}
