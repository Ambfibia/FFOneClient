use super::*;

#[test]
fn tutorial_mob_bullets_compile_from_exact_runtime_assets() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&asset_root).unwrap();
    let mut runtime = TutorialEffectRuntime::with_library(library);
    for bullet_type in [66, 106] {
        runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
            bullet_type,
            source: Vec3::new(1.0, 2.0, 3.0),
            target: Vec3::new(4.0, 2.5, 6.0),
            target_exists: true,
            source_style: 0,
            target_style: 0,
            motion: TutorialProjectileMotion::BulletMove,
            source_line: line!(),
        });
    }
    runtime.process_pending();
    let issues = runtime.drain_issues().collect::<Vec<_>>();
    let records = runtime.drain_records().collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| matches!(
            record.disposition,
            ffone_client::tutorial_effects_runtime::TutorialEffectRuntimeDisposition::NativeProjectileQueued {
                rendered_nodes,
                ..
            } if rendered_nodes > 0
        )), "records={records:#?}, issues={issues:#?}");
}

#[test]
fn runtime_hp_tracks_regeneration_and_exact_infection_ticks() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            hp: Some(1),
            ..Default::default()
        },
        ..default()
    };
    assert_eq!(
        apply_runtime_frame(
            &ffone_protocol::DecodedFrame {
                packet_type: packet::P_FE2CL_REP_PC_TICK,
                flags: 0,
                checksum: 0,
                payload: PcTick0104 {
                    hp: 625,
                    remaining: [0; 28],
                }
                .encode(),
            },
            &mut runtime,
        ),
        Some(625)
    );
    assert_eq!(runtime.hp, Some(625));

    let infection_tick = |character_id, hp| ffone_protocol::DecodedFrame {
        packet_type: packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK,
        flags: 0,
        checksum: 0,
        payload: TimeBuffDotDamageTick0104 {
            character_type: 1,
            character_id,
            time_buff_id: 17,
            result_character_type: 1,
            result_character_id: character_id,
            protected: false,
            damage: 150,
            hp,
            stamina: 0,
            nano_deactivated: false,
            condition_bit_flag: 0,
        }
        .encode(),
    };
    assert_eq!(
        apply_runtime_frame(&infection_tick(99, 475), &mut runtime),
        None
    );
    assert_eq!(runtime.hp, Some(625));
    assert_eq!(
        apply_runtime_frame(&infection_tick(77, 475), &mut runtime),
        Some(475)
    );
    assert_eq!(runtime.hp, Some(475));

    runtime.nano_slots[1] = RuntimeNanoSlot {
        nano_id: Some(2),
        skill_id: 8,
        stamina: 75,
        active: true,
    };
    let protected_tick = ffone_protocol::DecodedFrame {
        packet_type: packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK,
        flags: 0,
        checksum: 0,
        payload: TimeBuffDotDamageTick0104 {
            character_type: 1,
            character_id: 77,
            time_buff_id: 17,
            result_character_type: 1,
            result_character_id: 77,
            protected: true,
            damage: -2,
            hp: 475,
            stamina: 3,
            nano_deactivated: true,
            condition_bit_flag: 0x10000,
        }
        .encode(),
    };
    assert_eq!(
        apply_runtime_frame(&protected_tick, &mut runtime),
        Some(475)
    );
    assert_eq!(runtime.nano_slots[1].stamina, 3);
    assert!(!runtime.nano_slots[1].active);

    let mut colliding_npc_tick = infection_tick(77, 300);
    colliding_npc_tick.payload[0..4].copy_from_slice(&4_i32.to_le_bytes());
    assert_eq!(apply_runtime_frame(&colliding_npc_tick, &mut runtime), None);
    assert_eq!(
        runtime.hp,
        Some(475),
        "NPC/player numeric ID collisions must not overwrite local HP"
    );
}

#[test]
fn projectile_action_reverse_mode_matches_original_source_style_contract() {
    assert!(!projectile_action_reverse_mode(
        ProjectileAction::NpcToPlayerPair {
            npc_id: 1004,
            types: [76, 77],
        }
    ));
    assert!(projectile_action_reverse_mode(
        ProjectileAction::PlayerToPositionPair { types: [76, 77] }
    ));
}

#[test]
fn protocol_tutorial_flag_selects_a_shared_gameplay_scope_and_state() {
    assert_eq!(native_world_scope(0).unwrap(), NativeWorldScope::Tutorial);
    assert_eq!(native_world_scope(1).unwrap(), NativeWorldScope::WorldMap);
    assert_eq!(
        client_state_for_world_scope(NativeWorldScope::Tutorial),
        ClientState::Tutorial
    );
    assert_eq!(
        client_state_for_world_scope(NativeWorldScope::WorldMap),
        ClientState::World
    );
    let error = native_world_scope(-1).unwrap_err();
    assert!(error.contains("expected 0 or 1"), "{error}");
}

#[test]
fn tutorial_completion_failure_recovers_to_character_selection() {
    let mut tutorial = TutorialSession {
        completion_requested: true,
        exit_teardown_applied: true,
        ..default()
    };
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(7),
            ..default()
        },
        roster: RuntimeRosterStatus {
            selected_uid: Some(42),
            ..default()
        },
        ..default()
    };
    let mut next_state = NextState::<ClientState>::default();

    recover_tutorial_completion_to_character_selection(
        &mut tutorial,
        &mut runtime,
        &mut next_state,
        "tutorial completion failed",
    );

    assert!(!tutorial.completion_requested);
    assert!(!tutorial.exit_teardown_applied);
    assert!(tutorial.character().is_none());
    assert!(runtime.roster.characters.is_empty());
    assert_eq!(runtime.roster.selected_uid, Some(42));
    assert_eq!(runtime.player_id, None);
    assert_eq!(runtime.message, "tutorial completion failed");
    assert!(matches!(
        next_state,
        NextState::Pending(ClientState::CharacterSelect)
    ));
}

#[test]
fn tutorial_current_objective_tracks_the_selected_active_task_and_option() {
    let content = runtime_test_mission_content();
    let mut runtime = TutorialMissionRuntime::default();

    assert_eq!(
        tutorial_current_objective_ui(&runtime, &content, true),
        CurrentObjectiveUi::default()
    );
    assert!(runtime.start_task(&content, 2248).unwrap());
    assert_eq!(
        tutorial_current_objective_ui(&runtime, &content, true),
        CurrentObjectiveUi {
            visible: true,
            task_id: Some(2248),
            title: "Transmitter Critters".to_owned(),
            body: "Defeat the Oil Ogre.".to_owned(),
            remaining_time_seconds: None,
            enemies: Vec::new(),
            quest_items: vec![CurrentObjectiveProgressUi {
                content_id: 537,
                name: "Transmitter".to_owned(),
                complete: 0,
                needed: 1,
            }],
        }
    );
    assert_eq!(
        tutorial_current_objective_ui(&runtime, &content, false),
        CurrentObjectiveUi::default()
    );
}

#[test]
fn tutorial_actor_system_and_cleanup_are_state_gated() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .insert_resource(runtime_test_mission_content())
        .init_state::<ClientState>()
        .init_resource::<TutorialActorRegistry>()
        .init_resource::<TutorialActorCommandQueue>()
        .init_resource::<TutorialActorEventQueue>()
        .init_resource::<TutorialActorIssueQueue>()
        .init_resource::<TutorialNpcObservationSnapshot>()
        .init_resource::<TutorialActorCombatConfig>()
        .init_resource::<TutorialActorStandRandomStream>()
        .add_systems(
            Update,
            apply_tutorial_actor_commands.run_if(in_state(ClientState::Tutorial)),
        )
        .add_systems(OnExit(ClientState::Tutorial), cleanup_tutorial_actors);
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(ffone_client::tutorial_logic::TutorialNpcSpawn::new(
            1005,
            2674,
            ffone_client::tutorial_logic::LegacySpawnPosition::centiunits(0, 0, 0),
            None,
        ));

    app.update();
    assert!(
        app.world().resource::<TutorialActorRegistry>().is_empty(),
        "Bootstrap must not run tutorial actor systems"
    );

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);
    app.update();
    assert_eq!(app.world().resource::<TutorialActorRegistry>().len(), 1);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::World);
    app.update();
    assert!(app.world().resource::<TutorialActorRegistry>().is_empty());
    assert!(app.world().resource::<TutorialActorIssueQueue>().is_empty());
}

#[test]
fn buttercup_stun_condition_projects_es366_on_state_at_the_npc_diameter() {
    let content = runtime_test_mission_content();
    let actor = TutorialActor {
        id: 5000,
        npc_type: 2677,
        team: 2,
        hp: 400,
        max_hp: 400,
        damaged: false,
        interacting: false,
        invulnerable: true,
    };
    let entity = Entity::from_bits(42);
    let actor_rotation = Quat::from_rotation_y(0.75);
    let local_rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let command = tutorial_stun_effect_command(
        entity,
        &actor,
        actor_rotation,
        &content,
        TUTORIAL_BUTTERCUP_STUN_EFFECT_NODE.to_owned(),
        local_rotation,
    )
    .expect("the tutorial demo monster must have an XDT radius");

    let TutorialEffectRuntimeCommand::Add {
        effect_id,
        placement:
            TutorialEffectPlacement::ExactEntityBone {
                root_entity,
                node_name,
                local_translation_after_parenting,
                local_rotation_after_parenting,
                ..
            },
        scale,
        name,
        destroy_after_seconds,
        ..
    } = command
    else {
        panic!("stun condition must enqueue one attached effect");
    };
    assert_eq!(effect_id, TUTORIAL_BUTTERCUP_STUN_EFFECT_ID);
    assert_eq!(root_entity, entity);
    assert_eq!(node_name, TUTORIAL_BUTTERCUP_STUN_EFFECT_NODE);
    assert_eq!(local_translation_after_parenting, Vec3::ZERO);
    assert_eq!(local_rotation_after_parenting, local_rotation);
    assert_eq!(
        scale,
        content.gameplay_npc(actor.npc_type).unwrap().radius() * 2.0
    );
    assert_eq!(name.as_deref(), Some("tutorial NPC 5000 stun condition"));
    assert_eq!(destroy_after_seconds, None);
}
