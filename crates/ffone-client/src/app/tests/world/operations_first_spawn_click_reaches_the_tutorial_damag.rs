use super::*;

#[test]
fn first_spawn_click_reaches_the_tutorial_damage_queue_end_to_end() {
    let content = runtime_test_mission_content();
    let spawn_definition = content.gameplay_npc(2674).unwrap();
    let spawn_team = spawn_definition.team;
    let spawn_max_hp = spawn_definition.max_hp;
    let spawn_radius = spawn_definition.radius();
    let spawn_height = spawn_definition.height();
    let mut runtime = RuntimeStatus::default();
    runtime.tutorial_weapon_id = Some(328);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .add_plugins(LegacyAvatarActionPlugin)
        .init_resource::<TutorialSession>()
        .insert_resource(runtime)
        .insert_resource(content)
        .init_resource::<TutorialActorCommandQueue>()
        .init_resource::<TutorialNanoGameplayCommandQueue>()
        .init_resource::<TutorialEffectRuntime>()
        .init_resource::<GameplayAudioRuntime>()
        .init_resource::<PlayerWeaponAnimationCatalog>()
        .insert_resource(ButtonInput::<MouseButton>::default())
        .add_systems(
            Update,
            collect_tutorial_avatar_actions.after(LegacyAvatarActionSet::Resolve),
        );
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);

    let spawn = app
        .world_mut()
        .spawn((
            TutorialActor {
                id: 1001,
                npc_type: 2674,
                team: spawn_team,
                hp: spawn_max_hp,
                max_hp: spawn_max_hp,
                damaged: false,
                interacting: false,
                invulnerable: false,
            },
            Transform::from_xyz(0.0, 0.0, -10.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        ))
        .id();
    app.world_mut().spawn((
        LocalPlayer,
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
        LegacyPlayerController::from_baseline_table(),
        LegacyAvatarActionContext {
            combat_condition: true,
            attack_locked: false,
            attack_half_angle_degrees: 20.0,
            attack_range: 16.0,
            attack_cooldown_seconds: 1.0,
            target_capacity: 1,
            ..default()
        },
        LegacyAvatarTargetFeed {
            source_connected: true,
            samples: vec![ffone_client::avatar_action::LegacyTargetSample {
                entity: spawn,
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 10.0,
                in_view: true,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: true,
                talk_enabled: false,
                position: [0.0, 0.0, -10.0],
                radius: spawn_radius,
                height: spawn_height,
            }],
            ..default()
        },
        LegacyAvatarClipBindings::default(),
        LegacyAvatarActionState::default(),
    ));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);

    app.update();

    let commands = app
        .world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .take_all();
    assert!(commands.iter().any(|command| {
        matches!(
            command,
            ffone_client::tutorial_actors::TutorialActorCommand::Damage {
                id: 1001,
                amount: 100
            }
        )
    }));
}

#[test]
fn ordinary_world_npc_modal_blocks_click_through_avatar_actions() {
    let mut model = MissionUiModel::default();
    model.enabled = true;
    model.npc_interaction = Some(tutorial_gate_test_interaction());
    model.npc_icon_mode_visible = true;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialNanoGameplayState>()
        .insert_resource(model)
        .init_resource::<RuntimeStatus>()
        .init_resource::<WorldNanoCooldownRuntime>()
        .init_resource::<LocalInventoryRuntime>()
        .insert_resource(runtime_test_mission_content())
        .init_resource::<PlayerWeaponAnimationCatalog>()
        .add_systems(Update, sync_tutorial_action_gate);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::World);
    let player = app
        .world_mut()
        .spawn((LocalPlayer, LegacyAvatarActionContext::default()))
        .id();

    app.update();

    assert!(
        !app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .input_enabled,
        "an NPC UI click must not also become a world TalkNpc action"
    );

    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .npc_icon_mode_visible = false;
    app.update();

    assert!(
        app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .input_enabled,
        "ordinary world actions must resume after NpcIconMode closes"
    );
}

#[test]
fn vehicle_toggle_does_not_leak_from_chat_input() {
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<OptionProductionRuntime>()
        .init_resource::<LocalInventoryRuntime>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<MissionUiModel>()
        .init_resource::<GameplayUiModel>()
        .insert_resource(NetworkBridge::start())
        .init_resource::<RuntimeStatus>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        .init_resource::<RetrobutionInstanceAudioState>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, drive_local_vehicle_toggle);
    app.world_mut().spawn((
        LocalPlayer,
        LegacyAvatarEnvironmentState::default(),
        LegacyAvatarPresentationContext::default(),
    ));
    {
        let mut ui = app.world_mut().resource_mut::<GameplayUiModel>();
        ui.visible = true;
        ui.chat.input_enabled = true;
        ui.chat.active = true;
    }
    app.world_mut().resource_mut::<RuntimeStatus>().message = "chat owns V".to_owned();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyV);
    app.update();
    assert_eq!(
        app.world().resource::<RuntimeStatus>().message,
        "chat owns V"
    );
    assert!(
        app.world()
            .resource::<LocalVehiclePresentationRuntime>()
            .pending
            .is_none()
    );

    // With chat closed, the same hotkey reaches vehicle validation again.
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .active = false;
    app.update();
    assert_eq!(
        app.world().resource::<RuntimeStatus>().message,
        "Vehicle toggle requires an equipped vehicle"
    );

    // Every modal owner must stop V before vehicle validation, even without a vehicle.
    app.world_mut().resource_mut::<UserEquipUiState>().open_item_mode();
    app.world_mut().resource_mut::<RuntimeStatus>().message="inventory owns V".into();app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().message,"inventory owns V");
    app.world_mut().resource_mut::<UserEquipUiState>().close();
    app.world_mut().resource_mut::<MissionUiModel>().show_npc_interaction(NpcInteractionUi{npc_id:100,..default()});
    app.world_mut().resource_mut::<RuntimeStatus>().message="dialogue owns V".into();app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().message,"dialogue owns V");
    *app.world_mut().resource_mut::<MissionUiModel>()=default();
    let mut guide=GameGuideUiModel::default();guide.open_from_nanocom();app.insert_resource(guide);
    app.world_mut().resource_mut::<RuntimeStatus>().message="guide owns V".into();app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().message,"guide owns V");
    app.world_mut().remove_resource::<GameGuideUiModel>();
    let mut barber=ffone_client::barber::BarberModel::default();barber.request_open(8);app.insert_resource(barber);
    app.world_mut().resource_mut::<RuntimeStatus>().message="barber owns V".into();app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().message,"barber owns V");
    app.world_mut().remove_resource::<ffone_client::barber::BarberModel>();

    // Chat only owns manual toggles: loss of the equipped vehicle must still dismount.
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .active = true;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut()
        .resource_mut::<LocalVehiclePresentationRuntime>()
        .family = LegacyVehiclePresentationFamily::Board;
    app.update();
    assert!(
        app.world()
            .resource::<LocalVehiclePresentationRuntime>()
            .pending
            .is_some()
    );
}

#[test]
fn enter_menu_preserves_auto_run_until_a_mission_journal_opens() {
    let mut model = MissionUiModel::default();
    model.enabled = true;
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .init_resource::<TutorialSession>()
        .insert_resource(tutorial_gate_test_native_mechanics())
        .init_resource::<TutorialChoreographyPresentation>()
        .insert_resource(model)
        .init_resource::<GameplayUiModel>()
        .init_resource::<LegacyInputGate>()
        .add_systems(Update, sync_tutorial_input_gate);
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_auto_run(true);
    let player = app.world_mut().spawn((LocalPlayer, controller)).id();
    let mut outbox = GameplayUiOutbox::default();
    for _ in 0..2 {
        assert!(
            app.world_mut()
                .resource_mut::<MissionUiModel>()
                .open_nanocom_menu(&mut outbox)
        );
        {
            let mut ui = app.world_mut().resource_mut::<GameplayUiModel>();
            ui.visible = true;
            ui.chat.input_enabled = true;
            ui.chat.active = true;
        }
        app.update();
        assert!(!app.world().resource::<LegacyInputGate>().allow_forward);
        assert!(
            app.world()
                .get::<LegacyPlayerController>(player)
                .unwrap()
                .is_auto_running()
        );
        assert!(
            app.world_mut()
                .resource_mut::<MissionUiModel>()
                .close_nanocom_menu(&mut outbox)
        );
        app.world_mut()
            .resource_mut::<GameplayUiModel>()
            .chat
            .active = false;
        app.update();
        assert!(app.world().resource::<LegacyInputGate>().allow_forward);
        assert!(
            app.world()
                .get::<LegacyPlayerController>(player)
                .unwrap()
                .is_auto_running()
        );
    }
    {
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        assert!(model.open_nanocom_menu(&mut outbox));
        assert!(model.open_journal_from_nanocom(JournalOtherUi::default(), &mut outbox));
    }
    app.update();
    assert!(
        !app.world()
            .get::<LegacyPlayerController>(player)
            .unwrap()
            .is_auto_running()
    );
}

#[test]
fn user_equip_item_and_nano_modes_cancel_auto_run_and_release_input_on_close() {
    for nano_mode in [false, true] {
        let mut app = App::new();
        app.insert_resource(State::new(ClientState::World))
            .init_resource::<TutorialSession>()
            .insert_resource(tutorial_gate_test_native_mechanics())
            .init_resource::<TutorialChoreographyPresentation>()
            .init_resource::<MissionUiModel>()
            .init_resource::<UserEquipUiState>()
            .init_resource::<LegacyInputGate>()
            .add_systems(Update, sync_tutorial_input_gate);
        let mut controller = LegacyPlayerController::from_baseline_table();
        controller.set_auto_run(true);
        let player = app.world_mut().spawn((LocalPlayer, controller)).id();
        if nano_mode {
            app.world_mut().resource_mut::<UserEquipUiState>().open_from(ffone_client::user_equip_ui::UserEquipOpenSource::NanoBookShortcut);
        } else {
            app.world_mut().resource_mut::<UserEquipUiState>().open_item_mode();
        }

        app.update();
        assert!(!app.world().resource::<LegacyInputGate>().allow_forward);
        assert!(!app.world().get::<LegacyPlayerController>(player).unwrap().is_auto_running());

        app.world_mut().resource_mut::<UserEquipUiState>().close();
        app.update();
        assert!(app.world().resource::<LegacyInputGate>().allow_forward);
        assert!(!app.world().get::<LegacyPlayerController>(player).unwrap().is_auto_running());
    }
}

#[test]
fn world_mission_modals_cancel_auto_run_until_explicitly_restarted() {
    for journal in [false, true] {
        let mut model = MissionUiModel::default();
        model.enabled = true;
        let mut app = App::new();
        app.insert_resource(State::new(ClientState::World))
            .init_resource::<TutorialSession>()
            .insert_resource(tutorial_gate_test_native_mechanics())
            .init_resource::<TutorialChoreographyPresentation>()
            .insert_resource(model)
            .init_resource::<LegacyInputGate>()
            .init_resource::<GateObservedAtRead>()
            .add_systems(
                Update,
                (
                    sync_tutorial_input_gate,
                    observe_test_gate_at_read.in_set(LegacyMovementSet::ReadInput),
                )
                    .chain(),
            );
        let mut controller = LegacyPlayerController::from_baseline_table();
        controller.set_auto_run(true);
        let player = app.world_mut().spawn((LocalPlayer, controller)).id();
        app.update();
        assert!(
            app.world()
                .get::<LegacyPlayerController>(player)
                .unwrap()
                .is_auto_running()
        );

        {
            let mut model = app.world_mut().resource_mut::<MissionUiModel>();
            if journal {
                assert!(model.open_journal_from_shortcut(&mut GameplayUiOutbox::default()));
            } else {
                model.npc_interaction = Some(tutorial_gate_test_interaction());
                model.npc_icon_mode_visible = true;
            }
        }
        app.update();
        assert!(
            !app.world()
                .resource::<GateObservedAtRead>()
                .0
                .unwrap()
                .allow_forward
        );
        assert!(
            !app.world()
                .get::<LegacyPlayerController>(player)
                .unwrap()
                .is_auto_running()
        );

        {
            let mut model = app.world_mut().resource_mut::<MissionUiModel>();
            if journal {
                assert!(model.close_journal(&mut GameplayUiOutbox::default()));
            } else {
                model.npc_icon_mode_visible = false;
                model.npc_interaction = None;
            }
        }
        app.update();
        assert!(
            app.world()
                .resource::<GateObservedAtRead>()
                .0
                .unwrap()
                .allow_forward
        );
        assert!(
            !app.world()
                .get::<LegacyPlayerController>(player)
                .unwrap()
                .is_auto_running()
        );
    }
}

#[test]
fn death_modal_cancels_auto_run_and_does_not_resume_it_after_resurrection() {
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .init_resource::<TutorialSession>()
        .insert_resource(tutorial_gate_test_native_mechanics())
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<MissionUiModel>()
        .init_resource::<LegacyInputGate>()
        .init_resource::<ResurrectUiModel>()
        .add_systems(Update, sync_tutorial_input_gate);
    app.world_mut().resource_mut::<ResurrectUiModel>().visible = true;
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_auto_run(true);
    let player = app.world_mut().spawn((LocalPlayer, controller)).id();

    app.update();
    assert!(!app.world().resource::<LegacyInputGate>().allow_forward);
    assert!(
        !app.world()
            .get::<LegacyPlayerController>(player)
            .unwrap()
            .is_auto_running()
    );

    app.world_mut().resource_mut::<ResurrectUiModel>().visible = false;
    app.update();
    assert!(app.world().resource::<LegacyInputGate>().allow_forward);
    assert!(
        !app.world()
            .get::<LegacyPlayerController>(player)
            .unwrap()
            .is_auto_running()
    );
}

#[test]
fn late_gate_commits_npc_close_without_lingering_actor_lock() {
    let mut model = MissionUiModel::default();
    model.enabled = true;
    model.npc_interaction = Some(tutorial_gate_test_interaction());
    model.npc_icon_mode_visible = true;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .insert_resource(tutorial_gate_test_native_mechanics())
        .init_resource::<TutorialChoreographyPresentation>()
        .insert_resource(model)
        .init_resource::<LegacyInputGate>()
        .init_resource::<GateObservedAtRead>()
        .configure_sets(
            Update,
            (
                TutorialActorSet::ApplyCommands,
                LegacyMovementSet::ReadInput,
            )
                .chain(),
        )
        .add_systems(
            Update,
            sync_tutorial_input_gate
                .after(TutorialActorSet::ApplyCommands)
                .before(LegacyMovementSet::ReadInput),
        )
        .add_systems(
            Update,
            observe_test_gate_at_read.in_set(LegacyMovementSet::ReadInput),
        )
        .add_systems(
            Update,
            close_test_npc_modal.after(LegacyMovementSet::ReadInput),
        )
        .add_systems(PostUpdate, sync_tutorial_input_gate);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);
    app.world_mut().spawn(tutorial_gate_test_actor(true));

    app.update();

    assert_eq!(
        app.world().resource::<GateObservedAtRead>().0,
        Some(LegacyInputGate {
            allow_forward: false,
            allow_backward: false,
            allow_strafe: false,
            allow_keyboard_turning: false,
            allow_jump: false,
            allow_mouse_camera: false,
        })
    );
    assert_eq!(
        *app.world().resource::<LegacyInputGate>(),
        LegacyInputGate::default(),
        "PostUpdate must commit the closed model for the next input frame"
    );
}

#[test]
fn npc_icon_race_categories_match_clean_start_end_rank_and_recall_routes() {
    assert_eq!(
        clean_race_npc_route(13, false, false),
        CleanRaceNpcRoute::Menu
    );
    assert_eq!(
        clean_race_npc_route(13, true, false),
        CleanRaceNpcRoute::Menu
    );
    assert_eq!(
        clean_race_npc_route(14, true, false),
        CleanRaceNpcRoute::AutoEnd
    );
    assert_eq!(
        clean_race_npc_route(14, true, true),
        CleanRaceNpcRoute::Menu
    );
    assert_eq!(
        clean_race_npc_route(14, false, false),
        CleanRaceNpcRoute::Menu
    );
    assert_eq!(
        clean_race_npc_route(17, true, false),
        CleanRaceNpcRoute::RecallNano
    );
    assert_eq!(
        clean_race_npc_route(15, true, false),
        CleanRaceNpcRoute::None
    );

    assert!(race_service_allowed(13, NpcServiceKind::Race));
    assert!(race_service_allowed(13, NpcServiceKind::RaceRank));
    assert!(race_service_allowed(14, NpcServiceKind::RaceRank));
    assert!(!race_service_allowed(14, NpcServiceKind::Race));
    assert!(!race_service_allowed(17, NpcServiceKind::RaceRank));
}

#[test]
fn race_instance_map_decoder_owns_exact_variable_0104_layout() {
    let mut payload = vec![0_u8; RACE_INSTANCE_MAP_INFO_BASE_SIZE + 8];
    for (offset, value) in [
        (36, 15_i32),
        (40, 12_345),
        (44, 2),
        (48, 73),
        (52, 28),
        (56, 2),
    ] {
        payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    payload[60..64].copy_from_slice(&101_i32.to_le_bytes());
    payload[64..68].copy_from_slice(&202_i32.to_le_bytes());
    let decoded = decode_race_instance_map_info_0104(&payload).unwrap();
    assert_eq!(decoded.ep_id, 15);
    assert_eq!(decoded.top_record.score, 12_345);
    assert_eq!(decoded.top_record.rank, 2);
    assert_eq!(decoded.top_record.time_seconds, 73);
    assert_eq!(decoded.top_record.rings, 28);
    assert_eq!(decoded.switch_count, 2);

    assert!(decode_race_instance_map_info_0104(&payload[..64]).is_err());
    payload[56..60].copy_from_slice(&(-1_i32).to_le_bytes());
    assert!(decode_race_instance_map_info_0104(&payload).is_err());
}

#[test]
fn nano_create_invalid_quest_slot_preserves_all_authoritative_owners() {
    let load = ffone_protocol::PcLoadData0104::zeroed();
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let before_inventory = inventory.clone();
    let mut bank = NanoFreeTuningBank0104::default();
    bank.seed(&load);
    let before_bank = bank.clone();
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_level: 4,
            fusion_matter: 900,
            ..default()
        },
        ..default()
    };
    let packet = PcNanoCreateSuccess0104 {
        fusion_matter: 750,
        quest_item_slot: 50,
        quest_item: ItemBase0104 {
            item_type: 7,
            item_id: 411,
            option: 2,
            time_limit: 0,
        },
        nano: ffone_protocol::Nano0104 {
            id: 6,
            skill_id: 0,
            stamina: 150,
        },
        player_level: 5,
    };

    assert!(apply_nano_create_commit(packet, &mut inventory, &mut bank, &mut runtime).is_err());
    assert_eq!(inventory.quest_inventory, before_inventory.quest_inventory);
    assert_eq!(inventory.snapshot, before_inventory.snapshot);
    assert_eq!(bank, before_bank);
    assert_eq!(runtime.player_level, 4);
    assert_eq!(runtime.fusion_matter, 900);
}
