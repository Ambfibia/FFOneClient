use super::*;

#[test]
fn ordinary_world_combat_edges_match_clean_timeout_and_death() {
    let mut lifecycle = WorldCombatLifecycle::default();
    lifecycle.observe();
    assert_eq!(lifecycle.advance(0.0, false), (true, false));
    assert!(lifecycle.active);
    assert_eq!(lifecycle.advance(5.0, false), (false, false));
    assert_eq!(lifecycle.advance(0.001, false), (false, true));
    assert!(!lifecycle.active);

    lifecycle.observe();
    assert_eq!(lifecycle.advance(0.0, true), (true, true));
    assert!(!lifecycle.active);
}

#[test]
fn world_overhead_effects_use_the_avatar_composed_hnpc_root_name() {
    assert_eq!(
        world_network_npc_visual_root_name(41, 972, false),
        "network NPC 41 type 972"
    );
    assert_eq!(
        world_network_npc_visual_root_name(41, 972, true),
        "network HNPC 41 type 972"
    );
    assert_eq!(
        world_network_npc_visual_root_name(73, 3281, true),
        "network HNPC 73 type 3281"
    );
}

#[test]
fn server_slash_commands_are_distinct_from_muted_freechat_text() {
    assert!(is_server_chat_command_0104("/help"));
    assert!(is_server_chat_command_0104("/summonGroup 123"));
    assert!(!is_server_chat_command_0104("/"));
    assert!(!is_server_chat_command_0104(" /help"));
    assert!(!is_server_chat_command_0104("ordinary chat"));
}

#[test]
fn world_map_uses_the_exact_clean_default_map_key() {
    let mut input = InputSettings::default();
    let mut keyboard = ButtonInput::<KeyCode>::default();
    let mut mouse = ButtonInput::<MouseButton>::default();
    keyboard.press(KeyCode::KeyM);
    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::WorldMap,
        &keyboard,
        &mouse
    ));
    let mapping = input
        .mappings
        .iter_mut()
        .find(|row| row.action == LegacyOptionAction::WorldMap)
        .unwrap();
    mapping.primary = LegacyInputBinding::Key(LegacyPhysicalKey::V);
    mapping.alternate = LegacyInputBinding::Key(LegacyPhysicalKey::Mouse1);
    assert!(!option_action_just_pressed(
        &input,
        LegacyOptionAction::WorldMap,
        &keyboard,
        &mouse
    ));
    mouse.press(MouseButton::Right);
    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::WorldMap,
        &keyboard,
        &mouse
    ));
    clear_option_action_press(
        &input,
        LegacyOptionAction::WorldMap,
        &mut keyboard,
        &mut mouse,
    );
    assert!(!mouse.just_pressed(MouseButton::Right));
    assert!(keyboard.just_pressed(KeyCode::KeyM));
}

#[test]
fn world_map_clock_tracks_only_exact_observations_and_resets() {
    let mut clock = WorldMapServerClock::default();
    assert_eq!(clock.last_observed_server_time, None);

    clock.observe(0x1122_3344_5566_7788);
    assert_eq!(clock.last_observed_server_time, Some(0x1122_3344_5566_7788));
    // This deliberately overwrites rather than extrapolating or imposing
    // a synthetic monotonic clock.
    clock.observe(17);
    assert_eq!(clock.last_observed_server_time, Some(17));

    clock.reset();
    assert_eq!(clock.last_observed_server_time, None);
}

#[test]
fn world_map_sources_use_the_global_clean_clientnpc_array_without_live_entities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let assets = AssetLocator::open(root).unwrap();
    let client_npcs = ClientNpcWaypointCatalog::open(&assets).unwrap();

    // No ECS NPC entity participates in this adapter. Every ordered source
    // comes directly from clean MiniMapNpc.m_pElements, including distant
    // mission waypoint type 1111 at source row 473.
    let sources = world_map_npc_sources(&client_npcs);
    assert_eq!(sources.len(), 2_903);
    assert_eq!(sources[0].npc_type, 2_379);
    assert_eq!(
        sources[0].position,
        WorldMapPoint::new(
            client_npcs.rows()[0].client_position[0],
            client_npcs.rows()[0].client_position[1],
            client_npcs.rows()[0].client_position[2],
        )
    );
    assert_eq!(sources[473].npc_type, 1_111);
    assert_eq!(
        sources[473].position,
        WorldMapPoint::new(
            client_npcs.rows()[473].client_position[0],
            client_npcs.rows()[473].client_position[1],
            client_npcs.rows()[473].client_position[2],
        )
    );
}

#[test]
fn every_foreign_modal_owner_blocks_world_map_open() {
    assert!(!WorldMapOpenBlockers::default().blocked());
    for blockers in [
        WorldMapOpenBlockers {
            chat: true,
            ..default()
        },
        WorldMapOpenBlockers {
            buddy: true,
            ..default()
        },
        WorldMapOpenBlockers {
            mission: true,
            ..default()
        },
        WorldMapOpenBlockers {
            system_popup: true,
            ..default()
        },
        WorldMapOpenBlockers {
            nanocom: true,
            ..default()
        },
        WorldMapOpenBlockers {
            quit: true,
            ..default()
        },
        WorldMapOpenBlockers {
            option: true,
            ..default()
        },
        WorldMapOpenBlockers {
            resurrect: true,
            ..default()
        },
        WorldMapOpenBlockers {
            upsell: true,
            ..default()
        },
        WorldMapOpenBlockers {
            guide: true,
            ..default()
        },
        WorldMapOpenBlockers {
            bank: true,
            ..default()
        },
        WorldMapOpenBlockers {
            vendor: true,
            ..default()
        },
        WorldMapOpenBlockers {
            rule: true,
            ..default()
        },
        WorldMapOpenBlockers {
            user_equip: true,
            ..default()
        },
        WorldMapOpenBlockers {
            transportation: true,
            ..default()
        },
        WorldMapOpenBlockers {
            race: true,
            ..default()
        },
        WorldMapOpenBlockers {
            email: true,
            ..default()
        },
        WorldMapOpenBlockers {
            combi: true,
            ..default()
        },
        WorldMapOpenBlockers {
            cashmall: true,
            ..default()
        },
        WorldMapOpenBlockers {
            user_store: true,
            ..default()
        },
    ] {
        assert!(blockers.blocked());
    }
}

#[test]
fn every_foreign_modal_owner_blocks_option_open() {
    assert!(!OptionOpenBlockers::default().blocked());
    for blockers in [
        OptionOpenBlockers {
            chat: true,
            ..default()
        },
        OptionOpenBlockers {
            buddy: true,
            ..default()
        },
        OptionOpenBlockers {
            mission: true,
            ..default()
        },
        OptionOpenBlockers {
            system_popup: true,
            ..default()
        },
        OptionOpenBlockers {
            nanocom: true,
            ..default()
        },
        OptionOpenBlockers {
            quit: true,
            ..default()
        },
        OptionOpenBlockers {
            resurrect: true,
            ..default()
        },
        OptionOpenBlockers {
            upsell: true,
            ..default()
        },
        OptionOpenBlockers {
            guide: true,
            ..default()
        },
        OptionOpenBlockers {
            bank: true,
            ..default()
        },
        OptionOpenBlockers {
            vendor: true,
            ..default()
        },
        OptionOpenBlockers {
            rule: true,
            ..default()
        },
        OptionOpenBlockers {
            user_equip: true,
            ..default()
        },
        OptionOpenBlockers {
            world_map: true,
            ..default()
        },
        OptionOpenBlockers {
            transportation: true,
            ..default()
        },
        OptionOpenBlockers {
            race: true,
            ..default()
        },
        OptionOpenBlockers {
            email: true,
            ..default()
        },
        OptionOpenBlockers {
            combi: true,
            ..default()
        },
        OptionOpenBlockers {
            cashmall: true,
            ..default()
        },
        OptionOpenBlockers {
            user_store: true,
            ..default()
        },
    ] {
        assert!(blockers.blocked());
    }
}

#[test]
fn transportation_skyway_feed_decodes_only_exact_pinned_0104_frames() {
    let mut movement = Vec::new();
    for value in [77_i32, 100, 200, 300, 1_500] {
        movement.extend_from_slice(&value.to_le_bytes());
    }
    assert_eq!(
        decode_transportation_skyway_frame_0104(
            TRANSPORTATION_BROOMSTICK_MOVE_PACKET_ID_0104,
            &movement,
        ),
        Ok(Some(TransportationSkywayFrame0104::Move {
            pc_id: 77,
            position: [100, 200, 300],
            speed: 1_500,
        }))
    );
    let mut dismount = Vec::new();
    dismount.extend_from_slice(&77_i32.to_le_bytes());
    dismount.extend_from_slice(&0_i32.to_le_bytes());
    assert_eq!(
        decode_transportation_skyway_frame_0104(
            TRANSPORTATION_RIDING_SUCCESS_PACKET_ID_0104,
            &dismount,
        ),
        Ok(Some(TransportationSkywayFrame0104::Dismount {
            pc_id: 77,
            riding_type: 0,
        }))
    );
    assert!(
        decode_transportation_skyway_frame_0104(
            TRANSPORTATION_BROOMSTICK_MOVE_PACKET_ID_0104,
            &movement[..16],
        )
        .is_err()
    );
    assert_eq!(
        decode_transportation_skyway_frame_0104(0x3100_00d9, &dismount),
        Ok(None)
    );
}

#[test]
fn production_ui_uses_one_logical_canvas_instead_of_post_layout_scaling() {
    let runtime = OptionProductionRuntime::default();
    assert!(runtime.options.display.scale_ui);
    assert_eq!(runtime.effective_ui_scale(681.0), 1.0);
    assert_eq!(runtime.effective_ui_scale(1_080.0), 1.0);

    assert_eq!(
        native_ui_window_scale_factor(UVec2::new(1_264, 681), true),
        Some(1.0)
    );
    let hd = native_ui_window_scale_factor(UVec2::new(1_366, 768), true).unwrap();
    assert!((hd - 1.05).abs() < 0.000_001);
    let full_hd = native_ui_window_scale_factor(UVec2::new(1_920, 1_080), true).unwrap();
    assert!((full_hd - 1.476_562_5).abs() < 0.000_001);
    let ultrawide = native_ui_window_scale_factor(UVec2::new(2_560, 1_080), true).unwrap();
    assert!((ultrawide - 1.476_562_5).abs() < 0.000_001);
    let narrow = native_ui_window_scale_factor(UVec2::new(1_024, 768), true).unwrap();
    assert!((narrow - (1_024.0 / 1_264.0)).abs() < 0.000_001);
    assert_eq!(
        native_ui_window_scale_factor(UVec2::new(1_920, 1_080), false),
        None
    );
}

#[test]
fn ui_scale_ignores_backend_pixel_clamps_and_updates_only_on_toggle() {
    let requested = UVec2::new(1_920, 1_009);
    let backend_clamped = UVec2::new(1_920, 1_008);
    let startup_target = native_ui_window_scale_factor(requested, true).unwrap();

    // The startup WindowResolution already owns startup_target. Windows may
    // clamp the client area by one pixel; that must not start a second resize
    // while Bevy's color/depth attachments still describe the first size.
    assert_eq!(
        native_ui_scale_factor_update(None, backend_clamped, true),
        None
    );
    assert_eq!(
        native_ui_scale_factor_update(Some(true), backend_clamped, true),
        None
    );
    assert_eq!(
        native_ui_scale_factor_update(Some(true), backend_clamped, false),
        Some(None)
    );
    assert_eq!(
        native_ui_scale_factor_update(Some(false), requested, true),
        Some(Some(startup_target))
    );
}

#[test]
fn fusion_star_is_sky_distant_without_changing_its_angular_size() {
    let transform = legacy_fusion_star_plane_transform(1.0);
    assert!(
        (transform.translation.length() - LEGACY_FUSION_STAR_BACKGROUND_DISTANCE).abs() <= 0.0001
    );
    assert!(transform.translation.length() < LEGACY_SKYBOX_SIZE * 0.5);
    let source_angular_ratio = LEGACY_FUSION_STAR_SIZE / LEGACY_FUSION_STAR_POSITION_UNITY.length();
    let background_angular_ratio =
        LEGACY_FUSION_STAR_SIZE * transform.scale.x / transform.translation.length();
    assert!((source_angular_ratio - background_angular_ratio).abs() <= 0.0001);
}

#[test]
fn skybox_background_projection_preserves_the_intentional_world_extension() {
    assert_eq!(ffone_client::world::LEGACY_WORLD_CAMERA_FAR_NATIVE, 300.0);
    assert_eq!(EXTENDED_WORLD_CAMERA_FAR_NATIVE, 340.0);
    let world = Projection::Perspective(PerspectiveProjection {
        fov: 45.0_f32.to_radians(),
        near: 0.15,
        far: EXTENDED_WORLD_CAMERA_FAR_NATIVE,
        ..default()
    });
    let sky = legacy_skybox_projection(&world);

    let Projection::Perspective(world) = world else {
        unreachable!()
    };
    let Projection::Perspective(sky) = sky else {
        unreachable!()
    };
    assert_eq!(world.far, 340.0);
    assert_eq!(sky.fov, world.fov);
    assert_eq!(sky.near, world.near);
    assert_eq!(sky.far, LEGACY_SKYBOX_CAMERA_FAR);
    assert!(LEGACY_SKYBOX_SIZE * 0.5 < sky.far);
    assert!(LEGACY_SKYBOX_SIZE * 0.5 > world.far);
}

#[test]
fn terminal_look_at_updates_orbit_yaw_once_without_cutscene_rotation_cut() {
    let camera = ffone_client::tutorial_choreography_runtime::TutorialCameraPresentation {
        resolved_look_at: Some(Vec3::X),
        look_at_revision: 1,
        ..default()
    };
    let initial = Transform::from_rotation(Quat::from_rotation_y(0.4));
    let initial_rotation = initial.rotation;
    let (mut app, camera_entity) = tutorial_camera_test_app(initial, camera);
    {
        let mut camera_entity_mut = app.world_mut().entity_mut(camera_entity);
        let mut orbit = camera_entity_mut.get_mut::<LegacyOrbitCamera>().unwrap();
        orbit.pitch_degrees = 25.0;
    }
    app.update();

    let entity = app.world().entity(camera_entity);
    let transform = entity.get::<Transform>().unwrap();
    let orbit = entity.get::<LegacyOrbitCamera>().unwrap();
    assert!((orbit.yaw_degrees + 90.0).abs() < 0.0001);
    assert_eq!(orbit.pitch_degrees, 0.0);
    assert!(transform.rotation.abs_diff_eq(initial_rotation, 0.0001));
}

#[test]
fn ordinary_world_weapon_visuals_use_only_validated_bullets_and_hitscan_endpoints() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&asset_root).unwrap();
    assert_eq!(
        validated_world_weapon_bullet_links(&library, 113),
        Some(("Gtag01", "center"))
    );
    // Grenade bullet 72 exists in clean TableData, but its complete native
    // closure is not in the validated tutorial projectile publication.
    assert_eq!(validated_world_weapon_bullet_links(&library, 72), None);

    let target_entity = Entity::from_bits(77);
    let targets = [LegacyAttackTarget {
        entity: target_entity,
        kind: LegacyTargetKind::Npc { team: 2 },
        distance: 3.0,
    }];
    let hitscan = WorldPrimaryAttack0104::Hitscan(PcAttackNpcsRequest0104 {
        npc_ids: vec![7007],
    });
    assert_eq!(
        world_weapon_projectile_endpoints(
            &hitscan,
            &Transform::IDENTITY,
            12.0,
            &targets,
            |entity| (entity == target_entity).then_some((7007, Vec3::new(3.0, 1.2, 4.0), 2)),
        ),
        vec![WorldWeaponProjectileEndpoint {
            target: Vec3::new(3.0, 1.2, 4.0),
            target_exists: true,
            target_style: 2,
        }]
    );

    let miss = WorldPrimaryAttack0104::Hitscan(PcAttackNpcsRequest0104 {
        npc_ids: Vec::new(),
    });
    assert_eq!(
        world_weapon_projectile_endpoints(&miss, &Transform::IDENTITY, 12.0, &[], |_| None,),
        vec![WorldWeaponProjectileEndpoint {
            target: Vec3::new(0.0, 1.0, -12.0),
            target_exists: false,
            target_style: -1,
        }]
    );

    let rocket = build_world_primary_attack_0104(
        LegacyWeaponTargetMode::Rocket,
        &Transform::IDENTITY,
        20.0,
        &[],
        |_| None,
    )
    .unwrap();
    assert!(
        world_weapon_projectile_endpoints(&rocket, &Transform::IDENTITY, 20.0, &[], |_| None,)
            .is_empty()
    );
    assert!(matches!(
        rocket,
        WorldPrimaryAttack0104::Rocket(ref request)
            if request.destination == ProtocolPosition::from_native(Vec3::NEG_Z * 20.0).raw()
    ));
}

#[test]
fn ordinary_world_weapon_profile_throttles_network_attacks_and_unavailable_fails_closed() {
    let mut context = LegacyAvatarActionContext::default();
    apply_ordinary_world_weapon_combat_profile(
        &mut context,
        OrdinaryWorldWeaponCombatProfile::Armed(PlayerWeaponCombatProfile {
            attack_half_angle_degrees: 20.0,
            attack_range: 12.0,
            blast_radius: 0.5,
            attack_cooldown_seconds: 0.8,
            target_capacity: 5,
            target_mode: LegacyWeaponTargetMode::Normal,
            effect1_bullet_type: 113,
            effect2_bullet_type: 151,
            warhead_duration_seconds: 0.0,
            grenade_initial_vertical_speed: 80.0,
        }),
    );
    assert!(!context.attack_locked);
    assert_eq!(context.attack_half_angle_degrees, 20.0);
    assert_eq!(context.attack_range, 12.0);
    assert_eq!(context.attack_cooldown_seconds, 0.8);
    assert_eq!(
        context.target_capacity,
        PcAttackNpcsRequest0104::MAX_TARGETS
    );
    let held = ffone_client::avatar_action::LegacyAvatarActionInput {
        primary_held: true,
        ..default()
    };
    let mut state = LegacyAvatarActionState::default();
    let first = ffone_client::avatar_action::schedule_legacy_action_frame(
        held,
        &context,
        &default(),
        &mut state,
        &LegacyAvatarClipBindings::default(),
        0.0,
    );
    assert_eq!(first.actions.len(), 1);
    for _ in 0..10 {
        let rapid_repeat = ffone_client::avatar_action::schedule_legacy_action_frame(
            held,
            &context,
            &default(),
            &mut state,
            &LegacyAvatarClipBindings::default(),
            1.0 / 60.0,
        );
        assert!(rapid_repeat.actions.is_empty());
    }

    apply_ordinary_world_weapon_combat_profile(
        &mut context,
        OrdinaryWorldWeaponCombatProfile::Unavailable,
    );
    assert!(context.attack_locked);
    assert_eq!(context.attack_range, 0.0);
    assert!(context.attack_cooldown_seconds.is_infinite());
    assert_eq!(context.target_capacity, 0);
}

#[test]
fn empty_hand_uses_exact_unarmed_profile_and_keeps_npc_actions_enabled() {
    let mut inventory = LocalInventoryRuntime::default();
    assert_eq!(
        ordinary_world_weapon_combat_profile(&inventory, &PlayerWeaponAnimationCatalog::default(),),
        OrdinaryWorldWeaponCombatProfile::Unavailable,
        "an inventory that has not loaded must remain fail-closed"
    );

    inventory.seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    let profile =
        ordinary_world_weapon_combat_profile(&inventory, &PlayerWeaponAnimationCatalog::default());
    assert_eq!(profile, OrdinaryWorldWeaponCombatProfile::Unarmed);

    let mut context = LegacyAvatarActionContext::default();
    apply_ordinary_world_weapon_combat_profile(&mut context, profile);
    assert!(!context.attack_locked);
    assert_eq!(context.attack_half_angle_degrees, 90.0);
    assert_eq!(context.attack_range, 2.0);
    assert_eq!(context.attack_cooldown_seconds, 1.0);
    assert_eq!(context.target_capacity, 1);
    assert_eq!(context.weapon_target_mode, LegacyWeaponTargetMode::Normal);

    let npc = Entity::from_bits(77);
    let friendly = LegacyTargetSelection {
        focused_npc: Some(LegacyFocusedTarget {
            entity: npc,
            kind: LegacyTargetKind::Npc { team: 1 },
            distance: 1.0,
            talk_enabled: true,
        }),
        check_attack_target: false,
        ..default()
    };
    let talk = ffone_client::avatar_action::schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_just_pressed: true,
            ..default()
        },
        &context,
        &friendly,
        &mut LegacyAvatarActionState::default(),
        &LegacyAvatarClipBindings::default(),
        0.0,
    );
    assert_eq!(talk.actions, vec![LegacyAvatarActionIntent::TalkNpc(npc)]);

    let hostile = LegacyTargetSelection {
        attack_targets: vec![LegacyAttackTarget {
            entity: npc,
            kind: LegacyTargetKind::Npc { team: 2 },
            distance: 1.0,
        }],
        check_attack_target: true,
        ..default()
    };
    let attack = ffone_client::avatar_action::schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            ..default()
        },
        &context,
        &hostile,
        &mut LegacyAvatarActionState::default(),
        &LegacyAvatarClipBindings::default(),
        0.0,
    );
    assert!(matches!(
        attack.actions.as_slice(),
        [LegacyAvatarActionIntent::PrimaryAttack { mode: LegacyWeaponTargetMode::Normal, targets }]
            if targets == &hostile.attack_targets
    ));
}

#[test]
fn normal_world_npc_dialogue_turn_is_horizontal_immediate_and_client_only() {
    let appearance = NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
        npc_id: 41,
        npc_type: 2671,
        hp: 500,
        condition_bit_flag: 0,
        position: [0, 0, 0],
        angle: -45,
        barker_type: 0,
    });
    let authoritative_appearance = appearance;
    let mut npc = Transform::from_xyz(2.0, 1.5, -3.0)
        .with_rotation(ProtocolYawDegrees::new(appearance.0.angle).native_root_rotation())
        .with_scale(Vec3::splat(1.25));
    let original_translation = npc.translation;
    let original_scale = npc.scale;
    let player_position = Vec3::new(7.0, 9.0, 1.0);

    assert!(face_world_npc_toward_player_locally(
        &mut npc,
        player_position
    ));
    let expected_forward = Vec3::new(5.0, 0.0, 4.0).normalize();
    assert!((npc.rotation * Vec3::NEG_Z).abs_diff_eq(expected_forward, 0.000_01));
    assert_eq!(npc.translation, original_translation);
    assert_eq!(npc.scale, original_scale);
    assert_eq!(
        appearance, authoritative_appearance,
        "the local dialogue turn must not rewrite the server appearance angle"
    );

    let turned_rotation = npc.rotation;
    assert!(!face_world_npc_toward_player_locally(
        &mut npc,
        original_translation
    ));
    assert_eq!(npc.rotation, turned_rotation);
}

#[test]
fn normal_world_nano_shortcuts_cover_all_three_loadout_slots() {
    for (key, expected) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
    ] {
        let mut keys = ButtonInput::default();
        keys.press(key);
        assert_eq!(
            requested_world_nano_slot(&InputSettings::default(), &keys, &ButtonInput::default()),
            Some(expected)
        );
    }
    let keys = ButtonInput::<KeyCode>::default();
    assert_eq!(
        requested_world_nano_slot(&InputSettings::default(), &keys, &ButtonInput::default()),
        None
    );

    let mut runtime = RuntimeStatus::default();
    runtime.nano_slots[0] = RuntimeNanoSlot {
        nano_id: Some(TUTORIAL_BUTTERCUP_NANO_ID),
        skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
        stamina: 77,
        active: false,
    };
    assert_eq!(
        active_world_nano_slot(&runtime).map(|(_, slot)| slot.stamina),
        None
    );
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 150), Some(0));
    runtime.nano_slots[0].active = true;
    assert_eq!(
        active_world_nano_slot(&runtime).map(|(_, slot)| slot.stamina),
        Some(77)
    );
    assert_eq!(
        world_nano_active_request_slot(&runtime, 0, 150),
        Some(-1),
        "pressing the active clean-client slot recalls it"
    );
    assert_eq!(world_nano_active_request_slot(&runtime, 1, 150), None);
    runtime.nano_slots[0].stamina = 0;
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 150), Some(-1));
    runtime.nano_slots[0].active = false;
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 150), None);
    runtime.nano_slots[0].stamina = -1;
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 150), None);
    runtime.nano_slots[0].stamina = 31;
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 150), Some(0));
    for stamina in [0, 1, 29, 30] {
        runtime.nano_slots[0].stamina = stamina;
        assert_eq!(world_nano_active_request_slot(&runtime, 0, 150), None);
    }
    runtime.nano_slots[0].stamina = 20;
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 100), None);
    runtime.nano_slots[0].stamina = 21;
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 100), Some(0));
    assert_eq!(world_nano_active_request_slot(&runtime, 0, 0), None);
}
