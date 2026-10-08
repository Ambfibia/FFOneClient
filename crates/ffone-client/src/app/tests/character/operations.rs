use super::*;

#[cfg(windows)]
#[test]
fn startup_window_centering_uses_the_primary_monitor_origin_and_outer_size() {
    assert_eq!(
        centered_outer_window_position((-1_920, 0), (1_920, 1_040), (1_280, 720)),
        (-1_600, 160),
        "the 40-pixel taskbar is excluded from the vertical center"
    );
}

#[test]
fn production_login_surface_uses_only_owned_manual_and_auto_contexts() {
    let manual = PendingLogin {
        credentials: None,
        note: String::new(),
    };
    assert_eq!(
        login_surface_for_startup_context(&manual),
        LoginSurface::Manual
    );

    let automatic = PendingLogin {
        credentials: Some(Credentials {
            cookie: false,
            username: "Dexter".to_owned(),
            password: "nano".to_owned(),
        }),
        note: "OpenFusion auto-login queued".to_owned(),
    };
    let surface = login_surface_for_startup_context(&automatic);
    assert_eq!(surface, LoginSurface::AutoLogin);
    assert_ne!(surface, LoginSurface::WarpShard);
    assert_ne!(surface, LoginSurface::WaitingForWebAuthentication);
    assert_ne!(surface, LoginSurface::WebAuthentication);
}

#[test]
fn hidden_login_owner_resets_persistent_native_surface_to_clean_defaults() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<SystemMessageUiModel>()
        .insert_resource(LoginUiModel {
            visible: true,
            password: "temporary password".to_owned(),
            busy: true,
            surface: LoginSurface::AutoLogin,
            ..default()
        })
        .add_systems(Update, sync_login_ui);

    app.update();

    let login = app.world().resource::<LoginUiModel>();
    assert!(!login.visible);
    assert!(login.password.is_empty());
    assert!(!login.busy);
    assert_eq!(login.surface, LoginSurface::Manual);
}

#[test]
fn launcher_uses_clean_configurable_jump_and_escape_edges() {
    let mut input = InputSettings::default();
    let mut keyboard = ButtonInput::<KeyCode>::default();
    let mut mouse = ButtonInput::<MouseButton>::default();

    keyboard.press(KeyCode::Space);
    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::Jump,
        &keyboard,
        &mouse,
    ));
    assert!(!option_action_just_pressed(
        &input,
        LegacyOptionAction::Escape,
        &keyboard,
        &mouse,
    ));

    keyboard.clear_just_pressed(KeyCode::Space);
    keyboard.release(KeyCode::Space);
    assert!(option_action_just_released(
        &input,
        LegacyOptionAction::Jump,
        &keyboard,
        &mouse,
    ));

    let jump = input
        .mappings
        .iter_mut()
        .find(|row| row.action == LegacyOptionAction::Jump)
        .unwrap();
    jump.primary = LegacyInputBinding::Key(LegacyPhysicalKey::Mouse1);
    jump.alternate = LegacyInputBinding::Unbound;
    mouse.press(MouseButton::Right);
    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::Jump,
        &keyboard,
        &mouse,
    ));
    mouse.clear_just_pressed(MouseButton::Right);
    mouse.release(MouseButton::Right);
    assert!(option_action_just_released(
        &input,
        LegacyOptionAction::Jump,
        &keyboard,
        &mouse,
    ));

    let mut raw_escape = ButtonInput::<KeyCode>::default();
    raw_escape.press(KeyCode::Escape);
    assert!(!option_action_just_pressed(
        &input,
        LegacyOptionAction::Escape,
        &raw_escape,
        &mouse,
    ));
    raw_escape.press(KeyCode::Backquote);
    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::Escape,
        &raw_escape,
        &mouse,
    ));
}

#[test]
fn launcher_context_uses_authoritative_hp_and_masks_the_raw_reader() {
    let mut model = LauncherUiModel::default();
    let mut outbox = LauncherUiOutbox::default();
    model
        .open(launcher_test_trigger(), Vec3::ZERO, 6.0, &mut outbox)
        .unwrap();
    let mut status = RuntimeStatus::default();
    status.hp = Some(321);

    let mut app = App::new();
    app.insert_resource(status)
        .insert_resource(SystemMessageUiModel::default())
        .insert_resource(MissionUiModel::default())
        .insert_resource(model)
        .insert_resource(LauncherUiExternalState::default())
        .add_systems(Update, prepare_launcher_production_context);
    app.update();

    let external = app.world().resource::<LauncherUiExternalState>();
    assert_eq!(external.current_hp, 321);
    assert!(
        external.system_popup_active,
        "the compatibility Space/Escape reader must be masked while Launcher is visible"
    );
}

#[test]
fn launcher_production_driver_restores_real_context_and_resolves_configured_escape() {
    let mut model = LauncherUiModel::default();
    let mut outbox = LauncherUiOutbox::default();
    model
        .open(launcher_test_trigger(), Vec3::ZERO, 6.0, &mut outbox)
        .unwrap();
    outbox.clear();
    let mut keyboard = ButtonInput::<KeyCode>::default();
    keyboard.press(KeyCode::Backquote);
    let mut status = RuntimeStatus::default();
    status.hp = Some(444);

    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(keyboard)
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(OptionProductionRuntime::default())
        .insert_resource(SystemMessageUiModel::default())
        .insert_resource(MissionUiModel::default())
        .insert_resource(LauncherUiExternalState {
            system_popup_active: true,
            ..default()
        })
        .insert_resource(model)
        .insert_resource(outbox)
        .insert_resource(status)
        .add_systems(Update, drive_launcher_production);
    app.update();

    assert!(!app.world().resource::<LauncherUiModel>().visible());
    let external = app.world().resource::<LauncherUiExternalState>();
    assert_eq!(external.current_hp, 444);
    assert!(!external.system_popup_active);
    assert_eq!(
        app.world().resource::<RuntimeStatus>().message.as_str(),
        LAUNCHER_NAME_VISIBILITY_OWNER_GAP
    );
    let forwarded = app
        .world_mut()
        .resource_mut::<LauncherUiOutbox>()
        .drain()
        .collect::<Vec<_>>();
    assert!(forwarded.iter().any(|effect| matches!(
        effect,
        LauncherUiEffect::ExitMode {
            source: ffone_client::launcher_ui::LauncherUiDismissalSource::EscapeGate,
            event_group: 2,
            event_function: 1,
        }
    )));
    assert!(!forwarded.iter().any(|effect| matches!(
        effect,
        LauncherUiEffect::RequestEscapeCloseGate { .. }
            | LauncherUiEffect::SetCombatIcon(_)
            | LauncherUiEffect::SetNameVisible(_)
    )));
}

#[test]
fn launcher_escape_gate_resolves_and_forwards_clean_exit_effects() {
    let mut model = LauncherUiModel::default();
    let mut outbox = LauncherUiOutbox::default();
    model
        .open(launcher_test_trigger(), Vec3::ZERO, 6.0, &mut outbox)
        .unwrap();
    outbox.clear();
    assert!(model.request_escape_close(false, &mut outbox));

    let issues = bridge_launcher_production_effects(&mut model, &mut outbox, false);
    assert!(!model.visible());
    assert_eq!(issues, vec![LAUNCHER_NAME_VISIBILITY_OWNER_GAP]);
    let forwarded = outbox.drain().collect::<Vec<_>>();
    assert!(forwarded.iter().any(|effect| matches!(
        effect,
        LauncherUiEffect::ExitMode {
            source: ffone_client::launcher_ui::LauncherUiDismissalSource::EscapeGate,
            event_group: 2,
            event_function: 1,
        }
    )));
    assert!(!forwarded.iter().any(|effect| matches!(
        effect,
        LauncherUiEffect::RequestEscapeCloseGate { .. }
            | LauncherUiEffect::SetCombatIcon(_)
            | LauncherUiEffect::SetNameVisible(_)
    )));

    let mut model = LauncherUiModel::default();
    let mut outbox = LauncherUiOutbox::default();
    model
        .open(launcher_test_trigger(), Vec3::ZERO, 6.0, &mut outbox)
        .unwrap();
    outbox.clear();
    assert!(model.request_escape_close(false, &mut outbox));
    assert!(bridge_launcher_production_effects(&mut model, &mut outbox, true).is_empty());
    assert_eq!(
        model.phase,
        ffone_client::launcher_ui::LauncherUiPhase::Aiming
    );
    assert!(outbox.is_empty());

    let mut model = LauncherUiModel::default();
    let mut outbox = LauncherUiOutbox::default();
    model
        .open(launcher_test_trigger(), Vec3::ZERO, 6.0, &mut outbox)
        .unwrap();
    outbox.clear();
    assert!(model.request_escape_close(false, &mut outbox));
    outbox.clear();
    outbox.push(LauncherUiEffect::RequestEscapeCloseGate {
        event_group: 2,
        event_function: 23,
    });
    assert_eq!(
        bridge_launcher_production_effects(&mut model, &mut outbox, false),
        vec![LAUNCHER_ESCAPE_ROUTE_GAP]
    );
    assert_eq!(
        model.phase,
        ffone_client::launcher_ui::LauncherUiPhase::Aiming
    );
    assert!(outbox.is_empty());
}

#[test]
fn launcher_hp_popup_and_missing_print_name_owners_fail_closed() {
    assert_eq!(launcher_combat_icon_fail_closed(-1), Ok(()));
    assert_eq!(
        launcher_combat_icon_fail_closed(0),
        Err(LAUNCHER_COMBAT_ICON_OWNER_GAP)
    );
    assert_eq!(launcher_name_visibility_fail_closed(false), Ok(()));
    assert_eq!(
        launcher_name_visibility_fail_closed(true),
        Err(LAUNCHER_NAME_VISIBILITY_OWNER_GAP)
    );
    let mut owner_model = LauncherUiModel::default();
    let mut owner_outbox = LauncherUiOutbox::default();
    owner_outbox.push(LauncherUiEffect::SetCombatIcon(-1));
    owner_outbox.push(LauncherUiEffect::SetNameVisible(false));
    owner_outbox.push(LauncherUiEffect::SetCombatIcon(0));
    owner_outbox.push(LauncherUiEffect::SetNameVisible(true));
    assert_eq!(
        bridge_launcher_production_effects(&mut owner_model, &mut owner_outbox, false),
        vec![
            LAUNCHER_COMBAT_ICON_OWNER_GAP,
            LAUNCHER_NAME_VISIBILITY_OWNER_GAP
        ]
    );
    assert!(owner_outbox.is_empty());

    let mut messages = SystemMessageUiModel::default();
    let mission = MissionUiModel::default();
    assert!(!launcher_system_popup_active(&messages, &mission));
    messages.push(SystemMessageRequest::new(
        1,
        "Launcher popup gate",
        SystemMessageButtonType::Ok,
    ));
    assert!(launcher_system_popup_active(&messages, &mission));
    let mut mission_popup = MissionUiModel::default();
    assert!(mission_popup.open_tutorial_exit_dialog());
    assert!(launcher_system_popup_active(
        &SystemMessageUiModel::default(),
        &mission_popup
    ));

    let mut model = LauncherUiModel::default();
    let mut outbox = LauncherUiOutbox::default();
    model
        .open(
            launcher_test_trigger(),
            Vec3::new(1.0, 2.0, 3.0),
            6.0,
            &mut outbox,
        )
        .unwrap();
    outbox.clear();
    advance_launcher_production_frame(&mut model, &mut outbox, 1.0, false, false, false, 0, true);
    assert!(model.visible(), "a real popup must freeze the death branch");
    advance_launcher_production_frame(&mut model, &mut outbox, 1.0, false, false, false, 0, false);
    assert!(!model.visible());
    assert!(outbox.drain().any(|effect| matches!(
        effect,
        LauncherUiEffect::RestoreAvatarPosition(position)
            if position == Vec3::new(1.0, 2.0, 3.0)
    )));
}

#[test]
fn unfinished_character_always_resumes_appearance_before_entry() {
    let character = entry_test_character(77, 0, 1);
    assert_eq!(
        character_entry_route(&character),
        Ok(CharacterEntryRoute::ResumeAppearance)
    );

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

    assert_eq!(runtime.roster.pending_character_entry_uid, None);
    assert_eq!(creation_ui.pc_uid, Some(character.pc_uid));
    assert!(matches!(
        next_state,
        NextState::Pending(ClientState::CharacterCreate)
    ));
}

#[test]
fn ordinary_character_select_is_claimed_once_during_shard_handshake() {
    let character = entry_test_character(88, 1, 1);
    let bridge = NetworkBridge::start();
    let mut runtime = RuntimeStatus::default();
    let mut loading = GameplayLoadingState::default();
    let mut creation_session = CharacterCreationSession::default();
    let mut creation_ui = CharacterCreationUiModel::default();
    let mut preview = NativePlayerPreviewModel::default();
    let mut tutorial = TutorialSession::default();
    let mut next_state = NextState::<ClientState>::default();

    for _ in 0..2 {
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
    }

    assert_eq!(
        runtime.roster.pending_character_entry_uid,
        Some(character.pc_uid)
    );
    assert!(loading.visible);
    assert_eq!(loading.scope, Some(ResourceLoadingScope::World));
    assert_eq!(
        runtime.message,
        format!(
            "Character {} is already entering the shard...",
            character.pc_uid
        )
    );
}

#[test]
fn authoritative_character_delete_removes_the_roster_entry_and_closes_the_modal() {
    let mut first = entry_test_character(41, 1, 1);
    first.slot = 3;
    let mut deleted = entry_test_character(77, 1, 1);
    deleted.slot = 1;
    let mut runtime = RuntimeStatus::default();
    runtime.roster.characters = vec![first, deleted];
    runtime.roster.selected_uid = Some(77);

    let mut selection_ui = CharacterSelectionUiModel {
        delete_confirmation_pc_uid: Some(77),
        delete_name_input: "Test".to_owned(),
        delete_request_pending: true,
        ..default()
    };

    network_ingress::accept_character_delete(&mut runtime, &mut selection_ui, 77);

    assert_eq!(
        runtime
            .roster
            .characters
            .iter()
            .map(|character| character.pc_uid)
            .collect::<Vec<_>>(),
        vec![41]
    );
    assert_eq!(runtime.roster.selected_uid, Some(41));
    assert_eq!(selection_ui.delete_confirmation_pc_uid, None);
    assert!(selection_ui.delete_name_input.is_empty());
    assert!(!selection_ui.delete_request_pending);
}

#[test]
fn unfinished_login_character_resumes_native_appearance_screen() {
    let character = CharacterSummary {
        slot: 2,
        level: 1,
        pc_uid: 77,
        first_name: "Test".to_owned(),
        last_name: "Creator".to_owned(),
        position: [0; 3],
        style: ffone_protocol::CharacterStyle0104 {
            name_check: 1,
            gender: 2,
            face_style: 0,
            hair_style: 0,
            hair_color: 7,
            skin_color: 4,
            eye_color: 3,
            height: 4,
            body: 0,
            class: 0,
            appearance_flag: 0,
            tutorial_flag: 0,
            payzone_flag: 0,
        },
        equipment: [ffone_protocol::EquippedItem0104::default();
            ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
    };
    let mut session = CharacterCreationSession::default();
    let mut model = CharacterCreationUiModel::default();
    resume_incomplete_character(&character, &mut session, &mut model).unwrap();
    assert_eq!(model.screen, CharacterCreationScreen::Appearance);
    assert_eq!(model.slot, Some(2));
    assert_eq!(model.pc_uid, Some(77));
    assert_eq!(
        model.appearance.gender,
        ffone_client::character_creation_ui::CharacterGender::Boy
    );
    assert_eq!(model.appearance, CharacterAppearance::default());
    assert!(model.randomize_on_appearance_open);
    let saved = session.saved_name.as_ref().unwrap();
    assert_eq!(saved.first_name.to_string_lossy(), "Test");
    assert_eq!(saved.last_name.to_string_lossy(), "Creator");
    assert!(session.generated_name);
}

#[test]
fn rejected_world_ready_releases_the_character_entry_loading_barrier() {
    let bridge = NetworkBridge::start();
    let config = ClientConfig::from_args(std::iter::empty())
        .expect("default config parses")
        .expect("default config is not help");
    let mut next_state = NextState::<ClientState>::default();
    let mut loading = GameplayLoadingState::default();
    loading.begin(ResourceLoadingScope::World);

    let message = recover_failed_world_ready_connection(
        &bridge,
        &config,
        None,
        &mut next_state,
        &mut loading,
        "WorldReady rejected".to_owned(),
    );

    assert!(!loading.visible);
    assert!(loading.scope.is_none());
    assert!(matches!(next_state, NextState::Pending(ClientState::Login)));
    assert_eq!(
        message,
        "WorldReady rejected; shard disconnected safely, sign in again"
    );
}

pub(super) fn creation_loading_test_app(screen: CharacterCreationScreen) -> App {
    let mut app = App::new();
    let mut loading = GameplayLoadingState::default();
    loading.begin(ResourceLoadingScope::CharacterCreation);
    app.insert_resource(State::new(ClientState::CharacterCreate))
        .insert_resource(CharacterCreationUiModel {
            screen,
            ..default()
        })
        .insert_resource(loading)
        .init_resource::<NativePlayerPreviewModel>()
        .init_resource::<CharacterCreationAssetLease>()
        .add_systems(Update, gate_character_creation_loading);
    app
}

#[test]
fn character_creation_name_waits_for_static_assets_but_not_an_unrequested_avatar() {
    let mut app = creation_loading_test_app(CharacterCreationScreen::Name);
    for _ in 0..10 {
        app.update();
        assert!(app.world().resource::<GameplayLoadingState>().visible);
    }
    app.world_mut()
        .resource_mut::<CharacterCreationUiModel>()
        .asset_status = CharacterCreationAssetStatus::Ready;
    for _ in 1..GAMEPLAY_LOADING_RENDER_SETTLE_FRAMES {
        app.update();
        assert!(app.world().resource::<GameplayLoadingState>().visible);
    }
    app.update();
    assert!(!app.world().resource::<GameplayLoadingState>().visible);
}

#[test]
fn character_creation_appearance_waits_for_every_resource_in_any_completion_order() {
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut app = creation_loading_test_app(CharacterCreationScreen::Appearance);
        for (index, completion) in order.into_iter().enumerate() {
            match completion {
                0 => {
                    app.world_mut()
                        .resource_mut::<CharacterCreationUiModel>()
                        .asset_status = CharacterCreationAssetStatus::Ready
                }
                1 => {
                    app.world_mut()
                        .resource_mut::<NativePlayerPreviewModel>()
                        .status = NativePlayerPreviewStatus::ReadyAnimated { parts: 6 }
                }
                _ => {
                    *app.world_mut()
                        .resource_mut::<CharacterCreationAssetLease>() =
                        CharacterCreationAssetLease {
                            total: 2,
                            completed: 2,
                            ..default()
                        }
                }
            }
            if index < 2 {
                for _ in 0..10 {
                    app.update();
                    assert!(
                        app.world().resource::<GameplayLoadingState>().visible,
                        "released early for {order:?}"
                    );
                }
            }
        }
        for _ in 1..GAMEPLAY_LOADING_RENDER_SETTLE_FRAMES {
            app.update();
            assert!(app.world().resource::<GameplayLoadingState>().visible);
        }
        app.update();
        let loading = app.world().resource::<GameplayLoadingState>();
        assert!(!loading.visible && loading.scope.is_none());
    }
}

#[test]
fn character_creation_reports_resource_failures_and_recovers_after_readiness() {
    for failure in 0..3 {
        let mut app = creation_loading_test_app(CharacterCreationScreen::Appearance);
        app.world_mut()
            .resource_mut::<CharacterCreationUiModel>()
            .asset_status = CharacterCreationAssetStatus::Ready;
        app.world_mut()
            .resource_mut::<NativePlayerPreviewModel>()
            .status = NativePlayerPreviewStatus::ReadyAnimated { parts: 6 };
        *app.world_mut()
            .resource_mut::<CharacterCreationAssetLease>() = CharacterCreationAssetLease {
            total: 2,
            completed: 2,
            ..default()
        };
        match failure {
            0 => {
                app.world_mut()
                    .resource_mut::<CharacterCreationUiModel>()
                    .asset_status = CharacterCreationAssetStatus::Failed {
                    path: "ui/missing.png",
                }
            }
            1 => {
                app.world_mut()
                    .resource_mut::<NativePlayerPreviewModel>()
                    .status = NativePlayerPreviewStatus::Blocked("missing rig".into())
            }
            _ => {
                app.world_mut()
                    .resource_mut::<CharacterCreationAssetLease>()
                    .blocker = Some("missing preload".into())
            }
        }
        app.update();
        assert!(
            app.world()
                .resource::<GameplayLoadingState>()
                .blocked
                .is_some()
        );
        assert!(app.world().resource::<GameplayLoadingState>().visible);
        app.world_mut()
            .resource_mut::<CharacterCreationUiModel>()
            .asset_status = CharacterCreationAssetStatus::Ready;
        app.world_mut()
            .resource_mut::<NativePlayerPreviewModel>()
            .status = NativePlayerPreviewStatus::ReadyAnimated { parts: 6 };
        app.world_mut()
            .resource_mut::<CharacterCreationAssetLease>()
            .blocker = None;
        for _ in 0..GAMEPLAY_LOADING_RENDER_SETTLE_FRAMES {
            app.update();
        }
        assert!(!app.world().resource::<GameplayLoadingState>().visible);
        assert!(
            app.world()
                .resource::<GameplayLoadingState>()
                .blocked
                .is_none()
        );
    }
}

#[test]
fn character_entry_during_shard_loading_is_not_replayed_after_world_ready() {
    for scope in [ResourceLoadingScope::World, ResourceLoadingScope::Tutorial] {
        let mut loading = GameplayLoadingState::default();
        loading.begin(scope);
        let mut entry = BufferedCharacterEntry::default();
        entry.queue(42, &loading);
        loading.finish();
        assert_eq!(entry.take_when_ready(&loading), None);
    }
}

#[test]
fn npc_subtarget_hides_and_restores_the_local_character_scene() {
    let mut model = MissionUiModel::default();
    model.enabled = true;
    model.npc_interaction = Some(tutorial_gate_test_interaction());
    model.npc_icon_mode_visible = true;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<TutorialChoreographyPresentation>()
        .insert_resource(model)
        .add_systems(PostUpdate, sync_tutorial_choreography_visibility);
    let player_scene = app
        .world_mut()
        .spawn((LocalCharacterScene, Visibility::Inherited))
        .id();

    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(player_scene).unwrap(),
        Visibility::Hidden
    );

    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .npc_icon_mode_visible = false;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(player_scene).unwrap(),
        Visibility::Inherited
    );

    app.world_mut()
        .resource_mut::<TutorialChoreographyPresentation>()
        .player_hidden = true;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(player_scene).unwrap(),
        Visibility::Hidden,
        "closing SubTarget must not override an authored choreography hide"
    );
}

#[test]
fn cannon_aim_reads_configured_keys_while_walking_is_blocked() {
    let mut app = App::new();
    let mut model = LauncherUiModel::default();
    model.open(launcher_test_trigger(), Vec3::ZERO, 6.0, &mut LauncherUiOutbox::default()).unwrap();
    app.insert_resource(model).insert_resource(OptionProductionRuntime::default())
        .init_resource::<ButtonInput<KeyCode>>().init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<LauncherUiExternalState>()
        .add_systems(Update, read_launcher_configured_aim);
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyW);
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyD);
    app.update();
    let axes = app.world().resource::<LauncherUiExternalState>();
    assert_eq!(axes.aim_vertical_axis, 1.0);
    assert_eq!(axes.aim_horizontal_axis, 1.0);
}
