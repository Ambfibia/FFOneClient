use crate::app::*;

pub(super) fn run(
    config: ClientConfig,
    asset_locator: AssetLocator,
    localization: Localization,
    language: Language,
    voice_language: VoiceLanguage,
    mut prepared_user_settings: PreparedUserSettings,
    native_world_catalog_task: thread::JoinHandle<Result<NativeWorldCatalog, String>>,
    native_audio_catalog: NativeAudioCatalog,
    retrobution_world_audio_catalog: RetrobutionWorldAudioCatalog,
    character_creation_data: CharacterCreationData,
    native_player_rig_catalog: NativePlayerRigCatalog,
    pending_login: PendingLogin,
) -> ExitCode {
    let character_creation_data = Arc::new(character_creation_data);
    let mut option_runtime = prepared_user_settings.option_runtime();
    if !option_runtime.initialized_from_live_runtime {
        // Match the actual first visible window before the gameplay camera is
        // available to perform the full live-owner seed. This also prevents an
        // early character-screen fullscreen toggle from persisting the model's
        // clean constructor-only 1024x768/glow values.
        option_runtime.options.graphics.width = 1_264;
        option_runtime.options.graphics.height = 681;
        option_runtime.options.graphics.windowed = true;
        option_runtime.options.graphics.glow = LEGACY_REFERENCE_GLOW_ENABLED;
    }
    if performance_probe::requested() {
        option_runtime.options.graphics.width = 1920;
        option_runtime.options.graphics.height = 1080;
        option_runtime.options.graphics.windowed = true;
        option_runtime.initialized_from_live_runtime = true;
    }
    let startup_uses_persisted_graphics = option_runtime.initialized_from_live_runtime;
    let startup_width = if startup_uses_persisted_graphics {
        option_runtime.options.graphics.width.max(1)
    } else {
        1_264
    };
    let startup_height = if startup_uses_persisted_graphics {
        option_runtime.options.graphics.height.max(1)
    } else {
        681
    };
    let startup_windowed =
        !startup_uses_persisted_graphics || option_runtime.options.graphics.windowed;
    // WindowResolution values are physical pixels, but winit interprets an
    // override-free creation request as logical pixels and expands it by the
    // Windows DPI factor. Supplying the game/UI factor up front makes the
    // first native window use the requested physical client size immediately.
    // A temporary 1.0 override serves the same purpose when Scale UI is off;
    // the runtime synchronizer removes it without resizing the native window.
    let startup_scale_factor = native_ui_window_scale_factor(
        UVec2::new(startup_width, startup_height),
        option_runtime.options.display.scale_ui,
    )
    .unwrap_or(1.0);
    let startup_resolution = WindowResolution::new(startup_width, startup_height)
        .with_scale_factor_override(startup_scale_factor);
    let mut character_selection = CharacterSelectionUiModel::default();
    let startup_character_music = prepared_user_settings.character_selection_music();
    apply_startup_character_selection_settings(
        &prepared_user_settings.settings,
        &mut character_selection,
    );
    debug_assert_eq!(character_selection.music_enabled, startup_character_music);
    character_selection.fullscreen = !startup_windowed;
    let mut active_settings = collect_user_settings_snapshot(
        &option_runtime,
        &language,
        &voice_language,
        &character_selection,
    );
    active_settings.map = prepared_user_settings.settings.map.clone();
    let mut startup_world_map = WorldMapPresentation::default();
    startup_world_map.model.preferences = active_settings.map.clone();
    active_settings.window_maximized = prepared_user_settings.settings.window_maximized;
    prepared_user_settings
        .persistence
        .observe_without_saving(active_settings);
    let user_settings_persistence = prepared_user_settings.persistence;
    debug!(
        "user settings persistence path: {}",
        user_settings_persistence.path().display()
    );
    let asset_file_path = config.asset_root.to_string_lossy().into_owned();
    let character_asset_path = config.character_asset_root.to_string_lossy().into_owned();
    let tutorial_mission_content = match TutorialMissionContent::open(&asset_locator) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("native tutorial mission-content validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let combi_catalog =
        match CombiProductionCatalog0104::open(&asset_locator, &tutorial_mission_content) {
            Ok(catalog) => catalog,
            Err(error) => {
                eprintln!("native Combi production-catalog validation failed: {error}");
                return ExitCode::FAILURE;
            }
        };
    let email_catalog =
        match EmailProductionCatalog0104::open(&asset_locator, &tutorial_mission_content) {
            Ok(catalog) => catalog,
            Err(error) => {
                eprintln!("native Email production-catalog validation failed: {error}");
                return ExitCode::FAILURE;
            }
        };
    let transportation_catalog = match TransportationCatalog::open(&asset_locator) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("native transportation catalog validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let client_npc_waypoint_catalog = match ClientNpcWaypointCatalog::open(&asset_locator) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("native client-NPC waypoint validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let gameplay_nano_portrait_catalog = match GameplayNanoPortraitCatalog::open(&asset_locator) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("native gameplay Nano portrait validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let skill_buff_ui_catalog =
        match SkillBuffUiCatalog::open(&tutorial_mission_content, &asset_locator) {
            Ok(catalog) => catalog,
            Err(error) => {
                eprintln!("native gameplay skill-buff UI validation failed: {error}");
                return ExitCode::FAILURE;
            }
        };
    let player_weapon_animation_catalog = match PlayerWeaponAnimationCatalog::open(&asset_locator) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("native player weapon-animation catalog validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let native_world_catalog = match native_world_catalog_task.join() {
        Ok(Ok(catalog)) => catalog,
        Ok(Err(error)) => {
            eprintln!("native world-catalog validation failed: {error}");
            return ExitCode::FAILURE;
        }
        Err(_) => {
            eprintln!("native world-catalog loader panicked");
            return ExitCode::FAILURE;
        }
    };
    let mut app = App::new();
    app.register_asset_source(
        CHARACTER_ASSET_SOURCE,
        AssetSourceBuilder::platform_default(&character_asset_path, None),
    );
    let behaviour_root = NativeWorldBehaviourRoot(config.asset_root.clone());
    app.insert_resource(config)
        .insert_resource(asset_locator)
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(voice_language)
        .insert_resource(user_settings_persistence)
        .insert_resource(startup_world_map)
        .insert_resource(option_runtime)
        .init_resource::<GamepadActionState>()
        .init_resource::<gamepad_ui::PadUiFocus>()
        .init_resource::<ffone_client::ui::shared::controller::ControllerUiInput>()
        .init_resource::<LegacyCameraKeyInput>()
        .insert_resource(character_selection)
        .insert_resource(native_world_catalog)
        .insert_resource(behaviour_root)
        .insert_resource(native_audio_catalog)
        .insert_resource(retrobution_world_audio_catalog)
        .insert_resource(CharacterCreationDataResource(
            character_creation_data.clone(),
        ))
        .insert_resource(LoadedCharacterCreationData(character_creation_data))
        .insert_resource(native_player_rig_catalog)
        .insert_resource(player_weapon_animation_catalog)
        .insert_resource(pending_login)
        .insert_resource(QuitMenuRuntime::default())
        .insert_resource(CharacterCreationSession::default())
        .insert_resource(DexterShipCutsceneRuntime::default())
        .init_resource::<WorldMissionRuntime>()
        .init_resource::<MissionDeleteConfirmationRuntime>()
        .init_resource::<WorldNanoCooldownRuntime>()
        .init_resource::<movement_buffs::MovementBuffs>()
        .init_resource::<WorldCombatLifecycle>()
        .init_resource::<WorldNanoAuthorityInbox0104>()
        .init_resource::<WorldMissionWaypointRuntime>()
        .init_resource::<WorldMissionIndicatorRuntime>()
        .init_resource::<WorldMissionBarkerRequestRuntime>()
        .init_resource::<TutorialEffectLibraryLoadStatus>()
        .init_resource::<TutorialEffectRuntime>()
        .insert_resource(tutorial_mission_content)
        .insert_resource(combi_catalog)
        .insert_resource(email_catalog)
        .insert_resource(transportation_catalog)
        .insert_resource(client_npc_waypoint_catalog)
        .insert_resource(gameplay_nano_portrait_catalog)
        .insert_resource(skill_buff_ui_catalog)
        .init_resource::<WorldMapProductionRuntime>()
        .init_resource::<WorldMapServerClock>()
        .init_resource::<BankProductionRuntime0104>()
        .init_resource::<BankSystemMessageRuntime>()
        .init_resource::<VendorProductionRuntime0104>()
        .init_resource::<VendorSystemMessageRuntime>()
        .init_resource::<UserEquipProductionRuntime0104>()
        .init_resource::<UserEquipSystemMessageRuntime0104>()
        .init_resource::<UserEquipAvatarPresentationRuntime>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        // FFOne's production backend is OpenFusion 0104. The explicit
        // profile preserves raw mentorCount while adapting only the proven
        // first-vs-later follow-up decision.
        .insert_resource(GuideRuntime::new(GuideServerProfile::OpenFusion0104))
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<NormalNpcWarpRuntime>()
        .init_resource::<TransportationProductionRuntime>()
        .add_plugins(transportation_portrait::TransportationPortraitPlugin)
        .add_plugins(service_portrait::ServicePortraitPlugin)
        .add_plugins(ffone_client::input_focus::GameInputFocusPlugin)
        .add_systems(
            PreUpdate,
            sync_legacy_gameplay_cursor
                .after(ffone_client::input_focus::GameInputFocusSet)
                .before(bevy::ui::UiSystems::Focus)
                .run_if(ffone_client::input_focus::game_input_focus_regained),
        )
        .init_resource::<RaceProductionRuntime>()
        .init_resource::<RaceNetworkFrameInbox>()
        .init_resource::<CombiProductionRuntime0104>()
        .init_resource::<CombiProductionShell0104>()
        .init_resource::<CombiNetworkFrameInbox0104>()
        .init_resource::<EmailProductionRuntime0104>()
        .init_resource::<EmailProductionShell0104>()
        .init_resource::<EnchantProductionRuntime0104>()
        .init_resource::<EnchantProductionShell0104>()
        .init_resource::<EnchantNetworkFrameInbox0104>()
        .init_resource::<UserStoreProductionRuntime0104>()
        .init_resource::<RuleRuntime>()
        .init_resource::<NanoFreeTuningBank0104>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .insert_resource(ClearColor(Color::srgb(0.075, 0.13, 0.18)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_file_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FusionFall One".into(),
                        // The Retrobution reference window is 1280x720 including
                        // Win32 decorations; its measured Unity client area is
                        // exactly 1264x681. A supported saved document replaces
                        // that first-run size before the backend shows a frame.
                        resolution: startup_resolution,
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        mode: if startup_windowed {
                            WindowMode::Windowed
                        } else {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                        },
                        // Primary Retrobution standalone quality index 3
                        // (`Good`) stores `syncToVBL=false`. Matching it also
                        // avoids FIFO's visible 60 -> 30 FPS step whenever a
                        // busy streamed-world frame misses one refresh.
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(stable_native_render_plugin()),
        );
    // Keep gameplay-only UI trees and their texture/font requests out of the
    // login critical path. They are materialized once, under the next loading
    // barrier, immediately before the first gameplay/cutscene presentation.
    app.insert_state(NativeUiStartupPhase::Deferred);
    app.add_plugins(ffone_client::shared_input_ui::SharedInputUiPlugin);
    app.add_systems(
        Update,
        shared_redeem::consume_shared_redeem
            .in_set(ffone_client::ui_startup::NativeUiStartupSet)
            .after(ffone_client::shared_input_ui::SharedInputSet::Input)
            .after(BankUiSet::Lifecycle)
            .after(VendorUiSet::Lifecycle)
            .after(UserEquipUiSet::Lifecycle)
            .after(sync_bank_ui_context)
            .after(sync_vendor_ui_context)
            .before(BankUiSet::Interaction)
            .before(VendorUiSet::Interaction)
            .before(UserEquipUiSet::Interaction)
            .before(ffone_client::shared_input_ui::SharedInputSet::Bind),
    );
    app.add_systems(
        PostUpdate,
        flush_user_settings.in_set(UserSettingsSet::Flush),
    );
    #[cfg(windows)]
    app.add_systems(Startup, set_primary_window_icon);
    // QualitySettings.defaultStandaloneQuality=Good applies antiAliasing=0
    // to every primary/offscreen camera, including UI previews and event
    // cameras created after startup. Keep that global source behavior rather
    // than relying only on the initial gameplay camera bundle.
    app.add_systems(Update, enforce_primary_good_msaa);

    // Swapping the window-target UI cameras between native startup phases
    // leaves the deactivated camera's `ViewTarget` holding a swap-chain view.
    // D3D12 cannot resize a swap chain whose back buffers are still
    // referenced, so drop those before Bevy reconfigures the surface.
    app.add_plugins(ReleaseUnextractedViewTargetsPlugin);

    // Embedded assets require AssetPlugin's registry, which DefaultPlugins
    // creates above. Register the exact Hologram shader before its material
    // plugin builds the render pipeline.
    register_dexter_hologram_asset(&mut app);
    app.add_plugins((
        LegacyMovementPlugin,
        LegacyAvatarActionPlugin,
        RemotePlayerPlugin,
        NetworkEntityLifecycle0104Plugin,
        LegacyGlowPlugin,
        LegacyModelMaterialPlugin,
        NativeWorldPlugin,
        WorldBehaviourPlugin,
        GameplayUiPlugin,
        TutorialOverlayUiPlugin,
        GroupUiPlugin,
        SystemMessageUiPlugin,
        OverheatUiPlugin,
        GameplayNanoPortraitPlugin,
        NativeLoginUiPlugin,
    ))
    .add_plugins((
        NetworkSessionLifecyclePlugin,
        NetworkIngressPlugin,
        SocialIngressPlugin,
    ))
    .add_plugins((
        LauncherUiPlugin,
        EmailUiPlugin,
        ServerSelectionUiPlugin,
        EnchantUiPlugin0104,
        NativeCharacterSelectionUiPlugin,
        CombiUiPlugin,
        TransportationUiPlugin,
        RaceModeUiPlugin,
        RaceHudPlugin,
        RaceRankUiPlugin,
        CashmallUiPlugin0104,
        UserStoreUiPlugin0104,
    ))
    .add_plugins(NanocomMessageUiPlugin)
    .add_plugins(SkillBuffUiPlugin)
    .add_plugins(QuitMenuUiPlugin)
    .add_plugins(OptionUiPlugin)
    .add_plugins(ResurrectUiPlugin)
    .add_plugins(GuideUiPlugin)
    .add_plugins(GameGuideUiPlugin)
    .add_plugins(UpsellUiPlugin)
    .add_plugins(NativeCharacterCreationUiPlugin)
    .add_plugins(ffone_client::character_creation_ui::barber_ui::BarberUiPlugin)
    .add_plugins(barber::BarberRuntimePlugin)
    .add_plugins(QuickSlotUiPlugin)
    .add_plugins(UserEquipUiPlugin)
    .add_plugins(Pc2pcUiPlugin)
    .add_plugins(BankUiPlugin)
    .add_plugins(VendorUiPlugin)
    .add_plugins(RuleUiPlugin)
    .add_plugins(NanoFreeTuningUiPlugin)
    .add_plugins(WorldMapPresentationPlugin)
    .add_plugins(BuddyUiPlugin)
    .add_plugins(UiMaterialPlugin::<DexterHologramMaterial>::default())
    .add_plugins((
        NativePlayerPreviewPlugin,
        NativePlayerSharedRigPlugin,
        TutorialPlayerRigRuntimePlugin,
        GameplayAudioPlugin,
        NativeCharacterSelectionPortraitsPlugin,
        LocalizationPlugin,
        TutorialChoreographyRuntimePlugin,
        TutorialCinematicTitlePlugin,
        MissionUiPlugin,
        TutorialVoiceSubtitlePlugin,
        TutorialEffectsRuntimePlugin,
        TutorialEpBarrierPlugin,
        TutorialNanoGameplayPlugin,
        TutorialNanoPresentationPlugin,
        TutorialNanocomMessagePlugin,
    ))
    .add_plugins(RetrobutionWorldAudioPlugin)
    .add_plugins(CharacterFlowPlugin)
    .add_plugins(TutorialPresentationPlugin)
    .add_plugins((TutorialRuntimePlugin, TutorialChoreographyAdapterPlugin))
    .init_state::<ClientState>()
    .init_resource::<GameplayLoadingState>()
    .init_resource::<AssetResidency>()
    .init_resource::<LegacyWorldAmbienceCache>()
    .init_resource::<LocalInventoryRuntime>()
    .init_resource::<WorldPlayerEquipmentProjection>()
    .add_systems(
        Startup,
        (
            setup_scene,
            spawn_gameplay_loading_screen,
            initialize_character_creation_data,
            begin_login,
        )
            .chain(),
    )
    .add_systems(Update, poll_tutorial_effect_library)
    .add_systems(
        Update,
        sync_asset_residency_groups
            .before(drive_dexter_ship_cutscene)
            .before(enable_player_after_native_collider_ready),
    )
    .add_systems(Update, drive_character_creation_asset_prewarm)
    .add_systems(
        Update,
        route_world_ui_shortcuts
            .after(NetworkSessionLifecycleSet::Apply)
            .before(drive_email_production_pre_interaction_0104)
            .before(route_option_mode_input)
            .before(handle_world_map_input)
            .before(capture_quit_menu_open_intent)
            .before(UserEquipUiSet::Interaction)
            .before(GameplayUiSet::Input),
    )
    .add_systems(Update, sync_retrobution_audio_mix)
    .add_systems(
        Update,
        read_configured_movement_input
            .after(LegacyMovementSet::ReadInput)
            .before(LegacyMovementSet::CameraInput),
    )
    .add_systems(
        Update,
        read_configured_avatar_action_input
            .after(LegacyAvatarActionSet::ReadInput)
            .after(sync_tutorial_input_gate)
            .before(LegacyAvatarActionSet::Resolve),
    )
    .add_systems(
        PreUpdate,
        apply_option_camera_input_settings.after(InputSystems),
    )
    .add_systems(PreUpdate, sample_gamepad_actions.after(InputSystems))
    .add_systems(PreUpdate, gamepad_ui::prepare_gamepad_ui
        .after(InputSystems).before(bevy::ui::UiSystems::Focus))
    .add_systems(PreUpdate, gamepad_ui::navigate_gamepad_ui
        .after(sample_gamepad_actions).after(bevy::ui::UiSystems::Focus))
    .add_systems(PreUpdate, gamepad_ui::highlight_gamepad_ui.after(gamepad_ui::navigate_gamepad_ui))
    .add_systems(
        Update,
        prepare_launcher_production_context
            .after(NetworkSessionLifecycleSet::Apply)
            .after(process_world_trigger_uses)
            .before(LauncherUiSet::Interaction)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        FixedUpdate,
        read_launcher_configured_aim.before(LauncherUiSet::Aim).run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        drive_launcher_production
            .after(LauncherUiSet::Interaction)
            .before(LauncherUiSet::Bind)
            .before(consume_world_launcher_outbox)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        (
            sync_native_ui_window_scale,
            sync_option_live_runtime,
            sync_option_settings_to_live_owners,
            sync_option_buddy_projection,
            route_option_mode_input,
        )
            .chain()
            .after(NetworkSessionLifecycleSet::Apply)
            .after(OptionUiSet::AssetGate)
            .before(OptionUiSet::Interaction)
            .before(LoginUiSet::Bind)
            .before(CharacterSelectionUiSet::Layout)
            .before(CharacterCreationUiSet::Layout)
            .before(TutorialPresentationSet::GameplayHud)
            .before(handle_world_map_input)
            .before(capture_quit_menu_open_intent)
            .before(GameplayUiSet::Input),
    )
    .add_systems(
        Update,
        emit_nano_free_tuning_preview_animation_sounds
            .after(sync_nano_free_tuning_preview_animation)
            .in_set(GameplayAudioSet::Collect),
    )
    .add_systems(
        Update,
        consume_option_ui_outbox
            .after(OptionUiSet::Interaction)
            .after(consume_world_gameplay_ui_outbox)
            .before(OptionUiSet::Bind),
    )
    .add_systems(
        Update,
        (handle_world_map_input, consume_world_map_outbox)
            .chain()
            .after(NetworkSessionLifecycleSet::Apply)
            .after(WorldMapPresentationSet::Preload)
            .before(sync_tutorial_input_gate)
            .before(sync_tutorial_action_gate)
            .before(capture_quit_menu_open_intent)
            .before(UserEquipUiSet::Interaction)
            .before(MissionUiSet::Interaction)
            .before(GameplayUiSet::Input),
    )
    .add_systems(
        Update,
        sync_world_map_projection
            .after(consume_world_map_outbox)
            .after(consume_network_entity_lifecycle_0104)
            .after(sync_world_mission_waypoint)
            .after(LegacyMovementSet::Simulate)
            .before(WorldMapPresentationSet::Bind),
    )
    .add_systems(
        Update,
        sync_transportation_presentation_input
            .after(NetworkSessionLifecycleSet::Apply)
            .before(TransportationPresentationSet::Input),
    )
    .add_systems(
        Update,
        (
            consume_transportation_ui_commands,
            advance_transportation_production,
            consume_transportation_model_outbox,
        )
            .chain()
            .after(TransportationPresentationSet::Input)
            .after(NetworkSessionLifecycleSet::Apply)
            .before(TransportationPresentationSet::Bind)
            .before(sync_tutorial_input_gate)
            .before(sync_tutorial_action_gate)
            .before(flush_movement_intents)
            .before(GameplayUiSet::Input),
    )
    .add_systems(
        Update,
        consume_race_network_frames
            .after(NetworkSessionLifecycleSet::Apply)
            .before(RaceModePresentationSet::Input)
            .before(RaceRankPresentationSet::Input),
    )
    .add_systems(Update, animate_local_player_damage
        .after(NetworkSessionLifecycleSet::Apply)
        .after(LegacyAvatarActionSet::Resolve)
        .before(LegacyAvatarActionSet::Locomotion))
    .add_systems(Update, sync_race_world_rings.after(consume_race_production_outputs).before(flush_world_gameplay_intents))
    .add_systems(
        Update,
        sync_race_presentation_input
            .after(consume_race_network_frames)
            .before(RaceModePresentationSet::Input)
            .before(RaceRankPresentationSet::Input),
    )
    .add_systems(Update, sync_race_hud
        .after(consume_race_network_frames)
        .before(RaceHudSet::Bind))
    .add_systems(Update, cancel_expired_race
        .after(sync_race_hud)
        .before(consume_race_production_outputs))
    .add_systems(
        Update,
        consume_race_mode_ui_commands
            .after(RaceModePresentationSet::Input)
            .before(RaceModePresentationSet::Bind),
    )
    .add_systems(
        Update,
        consume_race_rank_ui_commands
            .after(RaceRankPresentationSet::Input)
            .before(RaceRankPresentationSet::Bind),
    )
    .add_systems(
        Update,
        consume_race_production_outputs
            .after(consume_race_network_frames)
            .after(consume_race_mode_ui_commands)
            .after(consume_race_rank_ui_commands)
            .after(collect_world_npc_interactions)
            .after(consume_world_gameplay_ui_outbox)
            .before(RaceModePresentationSet::Bind)
            .before(RaceRankPresentationSet::Bind),
    )
    .add_systems(
        Update,
        consume_email_network_frames_0104
            .after(NetworkSessionLifecycleSet::Apply)
            .before(drive_email_production_pre_interaction_0104)
            .before(EmailUiSet::Interaction),
    )
    .add_systems(
        Update,
        drive_email_production_pre_interaction_0104
            .after(consume_email_network_frames_0104)
            .before(EmailUiSet::Interaction),
    )
    .add_systems(
        Update,
        drive_email_production_post_interaction_0104
            .after(EmailUiSet::Interaction)
            .after(consume_email_system_message_outbox_0104)
            .before(consume_email_production_outputs_0104)
            .before(EmailUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_email_production_outputs_0104
            .after(drive_email_production_post_interaction_0104)
            .after(consume_world_gameplay_ui_outbox)
            .before(EmailUiSet::Bind),
    )
    .add_systems(
        Update,
        poll_email_update_check_0104
            .after(NetworkSessionLifecycleSet::Apply)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        consume_combi_network_frames_0104
            .after(NetworkSessionLifecycleSet::Apply)
            .before(CombiUiSet0104::Interaction),
    )
    .add_systems(
        Update,
        drive_combi_production_0104
            .after(CombiUiSet0104::Interaction)
            .after(consume_combi_network_frames_0104)
            .after(consume_combi_system_message_outbox_0104)
            .before(CombiUiSet0104::Bind),
    )
    .add_systems(
        Update,
        consume_combi_production_outputs_0104
            .after(drive_combi_production_0104)
            .after(consume_world_gameplay_ui_outbox)
            .before(sync_combi_presentation_0104)
            .before(CombiUiSet0104::Bind),
    )
    .add_systems(
        Update,
        sync_combi_presentation_0104
            .after(consume_combi_production_outputs_0104)
            .before(CombiUiSet0104::Bind),
    )
    .add_systems(
        Update,
        consume_enchant_network_frames_0104
            .after(NetworkSessionLifecycleSet::Apply)
            .before(EnchantUiSet0104::Interaction),
    )
    .add_systems(
        Update,
        drive_enchant_production_0104
            .after(consume_enchant_text_input)
            .after(EnchantUiSet0104::Interaction)
            .after(consume_enchant_network_frames_0104)
            .after(consume_enchant_system_message_outbox_0104)
            .before(EnchantUiSet0104::Bind),
    )
    .add_systems(
        Update,
        consume_enchant_text_input
            .after(ffone_client::shared_input_ui::SharedInputSet::Input)
            .after(consume_enchant_network_frames_0104)
            .before(ffone_client::shared_input_ui::SharedInputSet::Bind),
    )
    .add_systems(
        Update,
        consume_enchant_production_outputs_0104
            .after(drive_enchant_production_0104)
            .after(consume_world_gameplay_ui_outbox)
            .before(sync_enchant_presentation_0104)
            .before(EnchantUiSet0104::Bind),
    )
    .add_systems(
        Update,
        sync_enchant_presentation_0104
            .after(consume_enchant_production_outputs_0104)
            .before(EnchantUiSet0104::Bind),
    )
    .add_systems(OnExit(ClientState::World), reset_world_map_session)
    .add_systems(OnExit(ClientState::World), reset_transportation_session)
    .add_systems(OnExit(ClientState::World), reset_race_session)
    .add_systems(OnExit(ClientState::World), reset_email_session_0104)
    .add_systems(OnExit(ClientState::World), reset_combi_session_0104)
    .add_systems(OnExit(ClientState::World), reset_enchant_session_0104)
    .add_systems(OnExit(ClientState::World), reset_cashmall_session_0104)
    .add_systems(OnExit(ClientState::World), reset_user_store_session_0104)
    .add_systems(OnExit(ClientState::World), reset_option_session)
    .add_systems(OnExit(ClientState::World), reset_user_equip_session)
    .add_systems(OnExit(ClientState::World), reset_local_vehicle_presentation)
    .add_systems(OnExit(ClientState::World), reset_bank_session)
    .add_systems(OnExit(ClientState::World), reset_vendor_session)
    .add_systems(OnExit(ClientState::World), reset_rule_session)
    .add_systems(OnExit(ClientState::World), reset_nano_free_tuning_session)
    .add_systems(OnExit(ClientState::World), reset_world_mission_session)
    .add_systems(
        OnExit(ClientState::World),
        reset_world_nano_and_mission_presentation,
    )
    .add_systems(
        OnExit(ClientState::World),
        (
            reset_world_player_equipment_projection,
            cleanup_tutorial_nano_gameplay,
            cleanup_tutorial_effect_runtime,
        ),
    )
    .add_systems(
        Update,
        (
            open_pending_nano_free_tuning,
            consume_nano_free_tuning_production,
            sync_nano_free_tuning_preview_animation,
        )
            .chain()
            .run_if(world_nano_authority_active)
            .after(NetworkSessionLifecycleSet::Apply)
            .after(NanoFreeTuningPresentationSet::Input)
            .before(NanoFreeTuningPresentationSet::Bind)
            .before(sync_tutorial_input_gate)
            .before(sync_tutorial_action_gate)
            .before(capture_quit_menu_open_intent)
            .before(GameplayUiSet::Input),
    )
    .add_systems(
        Update,
        sync_rule_ui_context
            .after(NetworkSessionLifecycleSet::Apply)
            .before(RuleUiSet::Interaction),
    )
    .add_systems(
        Update,
        sync_user_equip_ui_context
            .after(NetworkSessionLifecycleSet::Apply)
            .before(UserEquipUiSet::Lifecycle),
    )
    .add_systems(
        Update,
        sync_cashmall_production_context_0104
            .after(CashmallUiSet0104::Lifecycle)
            .after(sync_user_equip_ui_context)
            .before(CashmallUiSet0104::Interaction)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        route_cashmall_production_input_0104
            .after(sync_cashmall_production_context_0104)
            .before(CashmallUiSet0104::Interaction)
            .before(capture_quit_menu_open_intent)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        consume_cashmall_production_effects_0104
            .after(CashmallUiSet0104::Interaction)
            .after(consume_world_gameplay_ui_outbox)
            .before(CashmallUiSet0104::Bind)
            .before(UserEquipUiSet::Bind)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        guard_user_store_before_interaction_0104
            .after(UserStoreUiSet0104::Lifecycle)
            .before(UserStoreUiSet0104::Interaction)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        consume_user_store_outbox_0104
            .after(UserStoreUiSet0104::Interaction)
            .before(UserStoreUiSet0104::Bind)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        consume_gameplay_ui_audio_outbox
            .after(poll_network)
            .after(BuddyUiSet::Interaction)
            .after(MissionUiSet::Interaction)
            .after(GameplayUiSet::Input)
            .after(ffone_client::pc2pc_ui::Pc2pcUiSet::Interaction)
            .after(handle_world_map_input)
            .in_set(GameplayAudioSet::Collect),
    )
    .add_systems(
        Update,
        audio_consume_gameplay_ui_audio_outbox::sync_service_inventory_audio
            .after(consume_enchant_production_outputs_0104)
            .after(consume_bank_ui_outbox)
            .after(consume_user_store_outbox_0104)
            .after(group_pc2pc::drain_world_pc2pc_session_outbox)
            .in_set(GameplayAudioSet::Collect),
    )
    .add_systems(
        Update,
        audio_consume_gameplay_ui_audio_outbox::sync_item_popup_audio
            .after(consume_bank_ui_outbox)
            .after(consume_vendor_ui_outbox)
            .after(UserEquipUiSet::Interaction)
            .in_set(GameplayAudioSet::Collect),
    )
    .add_systems(
        Update,
        audio_consume_gameplay_ui_audio_outbox::sync_world_map_mode_audio
            .after(handle_world_map_input)
            .after(consume_world_map_outbox)
            .in_set(GameplayAudioSet::Collect),
    )
    .add_systems(
        Update,
        consume_system_message_ui_audio_outbox
            .after(SystemMessageUiSet::Interaction)
            .in_set(GameplayAudioSet::Collect),
    )
    .add_systems(
        Update,
        consume_user_equip_ui_outbox
            .after(UserEquipUiSet::Interaction)
            .in_set(GameplayAudioSet::Collect)
            .before(UserEquipUiSet::Bind),
    )
    .add_systems(
        Update,
        sync_bank_ui_context
            .after(NetworkSessionLifecycleSet::Apply)
            .after(sync_user_equip_ui_context)
            .after(sync_resurrect_ui_context)
            .before(BankUiSet::Lifecycle),
    )
    .add_systems(
        Update,
        bank_delete::consume
            .after(BankUiSet::Interaction)
            .after(consume_bank_ui_outbox)
            .before(BankUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_bank_ui_outbox
            .after(BankUiSet::Interaction)
            .after(consume_world_gameplay_ui_outbox)
            .before(BankUiSet::Bind)
            .before(capture_quit_menu_open_intent),
    )
    .add_systems(
        Update,
        sync_vendor_ui_context
            .after(NetworkSessionLifecycleSet::Apply)
            .after(sync_user_equip_ui_context)
            .after(sync_resurrect_ui_context)
            .before(VendorUiSet::Lifecycle),
    )
    .add_systems(
        Update,
        consume_vendor_ui_outbox
            .after(VendorUiSet::Interaction)
            .after(drive_enchant_production_0104)
            .before(consume_enchant_production_outputs_0104)
            .before(VendorUiSet::Bind),
    )
    .add_systems(
        Update,
        vendor_chest::consume
            .after(consume_vendor_ui_outbox)
            .after(sync_user_equip_ui_context)
            .before(VendorUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_vendor_ui_audio_outbox
            .after(VendorUiSet::Interaction)
            .in_set(GameplayAudioSet::Collect),
    )
    .add_systems(
        Update,
        handle_login_ui_requests
            .after(LoginUiSet::Interaction)
            .before(poll_network),
    )
    .add_systems(
        Update,
        consume_login_ui_effects
            .after(LoginUiSet::Interaction)
            .before(sync_login_ui),
    )
    .add_systems(
        Update,
        (sync_quit_menu_context, close_quit_menu_from_gamepad, capture_quit_menu_open_intent)
            .chain()
            .after(sync_resurrect_ui_context)
            .after(sync_guide_ui_context)
            .after(sync_bank_ui_context)
            .after(sync_vendor_ui_context)
            .after(capture_guide_escape)
            .after(sync_upsell_ui_context)
            .after(UpsellUiSet::Interaction)
            .before(consume_upsell_ui_outbox)
            .before(QuitMenuUiSet::Interaction),
    )
    .add_systems(
        Update,
        consume_quit_menu_ui_outbox
            .after(QuitMenuUiSet::Interaction)
            .before(QuitMenuUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_quit_menu_audio_outbox.after(QuitMenuUiSet::Bind),
    )
    .add_systems(
        Update,
        (sync_guide_ui_context, capture_guide_escape)
            .chain()
            .after(sync_resurrect_ui_context)
            .before(GuideUiSet::Interaction),
    )
    .add_systems(
        Update,
        consume_guide_ui_outbox
            .after(GuideUiSet::Interaction)
            .before(GuideUiSet::Bind),
    )
    .add_systems(
        Update,
        advance_pending_guide_warp.after(consume_guide_ui_outbox),
    )
    .add_systems(
        Update,
        consume_guide_ui_audio_outbox.after(GuideUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_mission_delete_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_buddy_system_message_outbox)
            .before(consume_normal_warp_system_message_outbox)
            .before(GuideUiSet::Bind)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        consume_normal_warp_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_mission_delete_system_message_outbox)
            .before(consume_transportation_system_message_outbox)
            .before(GuideUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_transportation_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_normal_warp_system_message_outbox)
            .before(consume_race_system_message_outbox)
            .before(TransportationPresentationSet::Bind),
    )
    .add_systems(
        Update,
        consume_race_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_transportation_system_message_outbox)
            .after(consume_race_production_outputs)
            .before(consume_combi_system_message_outbox_0104)
            .before(RaceModePresentationSet::Bind),
    )
    .add_systems(
        Update,
        consume_combi_system_message_outbox_0104
            .after(SystemMessageUiSet::Interaction)
            .after(consume_race_system_message_outbox)
            .before(consume_enchant_system_message_outbox_0104)
            .before(CombiUiSet0104::Bind),
    )
    .add_systems(
        Update,
        consume_enchant_system_message_outbox_0104
            .after(SystemMessageUiSet::Interaction)
            .after(consume_combi_system_message_outbox_0104)
            .before(consume_email_system_message_outbox_0104)
            .before(EnchantUiSet0104::Bind),
    )
    .add_systems(
        Update,
        consume_email_system_message_outbox_0104
            .after(SystemMessageUiSet::Interaction)
            .after(consume_enchant_system_message_outbox_0104)
            .before(consume_guide_system_message_outbox)
            .before(EmailUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_guide_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_email_system_message_outbox_0104)
            .before(GuideUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_vendor_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_guide_system_message_outbox)
            .before(VendorUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_bank_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_vendor_system_message_outbox)
            .after(consume_bank_ui_outbox)
            .before(BankUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_user_equip_system_message_outbox
            .after(SystemMessageUiSet::Interaction)
            .after(consume_bank_system_message_outbox)
            .after(consume_user_equip_ui_outbox)
            .before(UserEquipUiSet::Bind)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        sync_upsell_ui_context
            .after(sync_resurrect_ui_context)
            .before(UpsellUiSet::Interaction),
    )
    .add_systems(
        Update,
        consume_upsell_ui_outbox
            .after(UpsellUiSet::Interaction)
            .before(UpsellUiSet::Bind),
    )
    .add_systems(Update, consume_upsell_audio_outbox.after(UpsellUiSet::Bind))
    .add_systems(
        Update,
        sync_login_ui
            .after(NetworkSessionLifecycleSet::Apply)
            .before(LoginUiSet::Bind),
    )
    .add_systems(
        Update,
        drive_tutorial_ambient.run_if(in_state(ClientState::Tutorial)),
    )
    .add_systems(
        Update,
        enable_player_after_native_collider_ready
            .after(drive_tutorial_ambient)
            .after(TutorialNanoPresentationSet::BindMaterials)
            .after(NativeWorldSet::RevealPresentation),
    )
    .add_systems(
        Update,
        acknowledge_warp_loading
            .after(enable_player_after_native_collider_ready)
            .after(NetworkSessionLifecycleSet::Apply)
            .before(flush_movement_intents),
    )
    .add_systems(OnEnter(ClientState::Login), defer_native_ui)
    .add_systems(
        OnEnter(ClientState::CharacterSelect),
        (
            activate_character_selection_ui,
            reset_character_selection_asset_lease,
            begin_tutorial_effect_library_load,
            begin_character_creation_asset_prewarm,
            begin_character_selection_loading,
        )
            .chain(),
    )
    .add_systems(
        OnEnter(ClientState::CharacterCreate),
        (
            activate_character_creation_ui,
            begin_character_creation_loading,
            begin_character_creation_asset_prewarm,
        )
            .chain(),
    )
    .add_systems(
        OnExit(ClientState::CharacterCreate),
        release_character_creation_asset_lease,
    )
    .add_systems(
        Update,
        gate_character_selection_loading
            .after(CharacterSelectionUiSet::Assets)
            .after(CharacterCreationUiSet::Assets)
            .after(CharacterSelectionPortraitsSet::Status)
            .after(NativePlayerPreviewSet::Rebuild)
            .after(sync_asset_residency_groups)
            .after(drive_character_creation_asset_prewarm)
            .after(poll_tutorial_effect_library),
    )
    .add_systems(
        Update,
        gate_character_creation_loading
            .after(CharacterCreationUiSet::Assets)
            .after(NativePlayerPreviewSet::Rebuild)
            .after(drive_character_creation_asset_prewarm),
    )
    .add_systems(Update, gate_login_loading.after(LoginUiSet::Bind))
    .add_systems(
        Update,
        sync_gameplay_loading_screen
            .after(enable_player_after_native_collider_ready)
            .after(gate_login_loading)
            .after(gate_character_selection_loading)
            .after(gate_character_creation_loading)
            .after(drive_dexter_ship_cutscene)
            .after(NativeWorldSet::RevealPresentation),
    )
    .add_systems(
        Update,
        ensure_and_update_legacy_skybox
            .after(LegacyMovementSet::CameraPose)
            .after(apply_tutorial_choreography_camera),
    )
    .add_systems(
        Update,
        ensure_and_update_legacy_fusion_star.after(LegacyMovementSet::CameraPose),
    )
    .add_systems(
        Update,
        animate_tutorial_dome_material.run_if(in_state(ClientState::Tutorial)),
    )
    .add_systems(
        Update,
        apply_legacy_world_ambience
            .after(LegacyMovementSet::Simulate)
            .after(apply_tutorial_choreography_camera)
            .before(ensure_and_update_legacy_skybox)
            .run_if(legacy_world_ambience_active),
    )
    .add_systems(
        OnEnter(ClientState::CharacterCreateIntro),
        (
            activate_gameplay_ui,
            begin_tutorial_effect_library_load,
            begin_character_creation_asset_prewarm,
        )
            .chain(),
    )
    .add_systems(
        OnEnter(ClientState::TutorialIntro),
        (activate_gameplay_ui, begin_tutorial_effect_library_load).chain(),
    )
    .add_systems(
        Update,
        spawn_dexter_ship_cutscene.before(drive_dexter_ship_cutscene),
    )
    .add_systems(
        Update,
        (drive_dexter_ship_cutscene, apply_dexter_ship_animations)
            .chain()
            .after(bind_network_npc_texture_variants_0104)
            .before(LocalizationSet::Apply),
    )
    .add_systems(
        OnExit(ClientState::CharacterCreateIntro),
        cleanup_dexter_ship_cutscene,
    )
    .add_systems(
        OnExit(ClientState::TutorialIntro),
        cleanup_dexter_ship_cutscene,
    )
    .add_systems(
        OnEnter(ClientState::World),
        (
            activate_gameplay_ui,
            begin_tutorial_effect_library_load,
            reset_world_mission_barker_request,
        )
            .chain(),
    )
    .add_systems(
        Update,
        sync_tutorial_input_gate
            .after(TutorialActorSet::ApplyCommands)
            // `CnGuiChat.Update` calls `ShowNanoMenu(true)` before the clean
            // avatar controller samples input. Preserve that same-frame gate
            // so Enter cannot leak movement/action input into the world.
            .after(GameplayUiSet::Input)
            .before(LegacyMovementSet::ReadInput),
    )
    .add_systems(
        Update,
        sync_world_player_equipment
            .after(NetworkSessionLifecycleSet::Apply)
            .before(sync_tutorial_action_gate)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        advance_world_nano_cooldowns
            .after(NetworkSessionLifecycleSet::Apply)
            .before(sync_tutorial_action_gate)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        advance_world_combat_lifecycle
            .after(NetworkSessionLifecycleSet::Apply)
            .before(sync_tutorial_action_gate)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        apply_world_nano_authority_inbox
            .after(consume_network_entity_lifecycle_0104)
            .before(sync_tutorial_action_gate)
            .before(sync_world_nano_presentation)
            .before(sync_resurrect_ui_context)
            .before(sync_skill_buff_ui_context)
            .before(TutorialPresentationSet::GameplayHud)
            .run_if(world_nano_authority_active),
    )
    .add_systems(
        Update,
        movement_buffs::sync_movement_buffs
            .after(NetworkSessionLifecycleSet::Apply)
            .after(apply_world_nano_authority_inbox)
            .before(LegacyMovementSet::Simulate)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        sync_tutorial_action_gate
            .after(TutorialActorSet::ApplyCommands)
            .after(TutorialNanoGameplaySet::AdvanceCooldown)
            // Read the still-open NPC modal before its close button commits.
            // Otherwise that same primary click can fall through to
            // AvatarAction and immediately enqueue a second TalkNpc/greeting.
            .before(MissionUiSet::Interaction)
            .before(LegacyAvatarActionSet::Resolve),
    )
    .add_systems(
        Update,
        activate_world_nano_from_keyboard
            .after(sync_tutorial_action_gate)
            .before(LegacyAvatarActionSet::Resolve)
            .run_if(world_nano_authority_active),
    )
    .add_systems(
        Update,
        recall_world_nano_for_zipline
            .after(process_world_trigger_uses)
            .before(flush_world_gameplay_intents)
            .run_if(world_nano_authority_active),
    )
    .add_systems(
        Update,
        charge_world_nano_from_keyboard
            .after(sync_tutorial_action_gate)
            .before(LegacyAvatarActionSet::Resolve)
            .run_if(world_nano_authority_active),
    )
    .add_systems(
        Update,
        sync_world_nano_presentation
            .after(NetworkSessionLifecycleSet::Apply)
            .after(apply_world_nano_authority_inbox)
            .after(sync_world_player_equipment)
            .before(collect_world_npc_interactions)
            .before(TutorialNanoGameplaySet::ApplyCommands)
            .run_if(world_nano_authority_active),
    )
    .add_systems(
        Update,
        sync_tutorial_network_npc_visibility
            .after(consume_network_entity_lifecycle_0104)
            .before(TutorialActorSet::ProduceTargets)
            .before(produce_world_avatar_target_feed),
    )
    .add_systems(
        Update,
        produce_world_avatar_target_feed
            .after(LegacyMovementSet::CameraInput)
            .after(consume_network_entity_lifecycle_0104)
            .before(LegacyAvatarActionSet::Resolve)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        advance_skyway_traversal
            .after(poll_network)
            .before(LegacyMovementSet::Simulate)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        collect_world_npc_interactions
            .after(LegacyAvatarActionSet::Resolve)
            .after(movement_buffs::sync_movement_buffs)
            .in_set(GameplayAudioSet::Collect)
            .before(TutorialNanoGameplaySet::ApplyCommands)
            .before(LegacyMovementSet::Simulate)
            // Tutorial and ordinary world movement share one shard, but
            // avatar action authority does not. The tutorial collector must
            // be the only owner which drains LegacyAvatarActionQueue while
            // cntutorialscript's local actors are active.
            .run_if(ordinary_world_action_collector_active),
    )
    .add_systems(Update, super::group_pc2pc::drain_world_pc2pc_session_outbox
        .after(ffone_client::pc2pc_ui::Pc2pcUiSet::Interaction)
        .run_if(shared_gameplay_world_active))
    .add_systems(ffone_client::ui_startup::NativeGameplayUiStartup, super::player_interaction::spawn)
    .add_systems(Update, super::player_interaction::interact
        .before(drain_world_pc2pc_offer_outbox).run_if(shared_gameplay_world_active))
    .add_systems(Update, super::player_interaction::bind
        .after(collect_world_npc_interactions)
        .after(super::player_interaction::interact)
        .before(ffone_client::localization::LocalizationSet::Apply))
    .add_systems(
        Update,
        drain_world_pc2pc_offer_outbox
            .after(collect_world_npc_interactions)
            .after(consume_world_pc2pc_offer_prompt)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        (flush_movement_intents, flush_world_gameplay_intents)
            .after(LegacyMovementSet::Simulate)
            .after(NativeWorldSet::ResolveCollision),
    )
    .add_systems(
        Update,
        update_legacy_avatar_environment
            .in_set(LegacyEnvironmentSet::Update)
            .before(GameplayAudioSet::Drive)
            .after(NativeWorldSet::ResolveCollision)
            .after(collect_tutorial_actor_events)
            .before(LegacyMovementSet::CameraPose),
    )
    .add_systems(
        Update,
        toggle_player_interaction
            .after(NetworkSessionLifecycleSet::Apply)
            .before(sync_tutorial_action_gate),
    )
    .add_systems(
        Update,
        sync_quick_slot_ui_context
            .after(TutorialPresentationSet::GameplayHud)
            .after(sync_user_equip_ui_context)
            .after(sync_resurrect_ui_context)
            .after(sync_guide_ui_context)
            .after(sync_vendor_ui_context)
            .after(sync_upsell_ui_context)
            .after(consume_quit_menu_ui_outbox)
            .before(QuickSlotUiSet::Bind),
    )
    .add_systems(
        Update,
        sync_skill_buff_ui_context
            .after(TutorialPresentationSet::GameplayHud)
            .after(LegacyAvatarActionSet::Resolve)
            .before(SkillBuffUiSet::ApplyNanoGumballs)
            .before(SkillBuffUiSet::Bind),
    )
    .add_systems(
        Update,
        sync_resurrect_ui_context
            .after(TutorialPresentationSet::GameplayHud)
            .before(ResurrectUiSet::Domain),
    )
    .add_systems(
        Update,
        sync_world_mission_ui
            .after(NetworkSessionLifecycleSet::Apply)
            .before(collect_world_npc_interactions)
            .before(TutorialPresentationSet::GameplayHud)
            .before(MissionUiSet::Interaction)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        mission_escort::sync_escort_requests
            .after(NetworkSessionLifecycleSet::Apply)
            .after(consume_network_entity_lifecycle_0104)
            .before(sync_world_mission_ui)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        mission_dialogue::present_world_mission_dialogue
            .after(NetworkSessionLifecycleSet::Apply)
            .before(GameplayUiSet::NpcSpeech)
            .before(GameplayAudioSet::Drive)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        sync_world_mission_waypoint
            .after(sync_world_mission_ui)
            .before(TutorialPresentationSet::GameplayHud)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        sync_world_mission_indicators
            .after(sync_world_mission_ui)
            .after(consume_network_entity_lifecycle_0104)
            .before(process_tutorial_effect_runtime)
            .run_if(in_state(ClientState::World)),
    )
    .add_systems(
        Update,
        sync_local_infection_status_effect
            .after(poll_network)
            .before(process_tutorial_effect_runtime)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        sync_world_npc_game_icons
            .after(sync_world_mission_indicators)
            .after(consume_network_entity_lifecycle_0104)
            .before(process_tutorial_effect_runtime)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        (world_skill_effects::sync_world_skill_effects,
         world_skill_effects::spawn_world_instant_skill_effects,
         world_skill_effects::spawn_healing_tick_effects,
         npc_skill_presentation::present_npc_skills)
            .after(poll_network)
            .after(consume_network_entity_lifecycle_0104)
            .before(process_tutorial_effect_runtime),
    )
    .add_systems(
        Update,
        collect_world_npc_attack_visuals
            .after(consume_network_entity_lifecycle_0104)
            .in_set(GameplayAudioSet::Collect)
            .before(process_tutorial_effect_runtime)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        (
            sync_nanocom_foreign_modal_suppression,
            sync_guide_chat_input_gate,
        )
            .chain()
            .after(TutorialPresentationSet::GameplayHud)
            .after(sync_user_equip_ui_context)
            .after(sync_guide_ui_context)
            .after(sync_vendor_ui_context)
            .after(sync_upsell_ui_context)
            .before(MissionUiSet::Interaction)
            .before(GameplayUiSet::Input),
    )
    .add_systems(
        Update,
        consume_resurrect_ui_outbox
            .after(ResurrectUiSet::Interaction)
            .before(ResurrectUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_quick_slot_ui_outbox
            .after(QuickSlotUiSet::Interaction)
            .after(sync_user_equip_ui_context)
            .after(consume_resurrect_ui_outbox)
            .after(consume_quit_menu_ui_outbox)
            .before(QuickSlotUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_world_gameplay_ui_outbox
            .after(collect_world_npc_interactions)
            .after(sync_guide_ui_context)
            .after(sync_vendor_ui_context)
            .after(sync_upsell_ui_context)
            .in_set(GameplayAudioSet::Collect)
            .after(MissionUiSet::Interaction)
            .after(GameplayUiSet::Input)
            .before(UserEquipUiSet::Bind)
            .before(GuideUiSet::Interaction)
            .before(VendorUiSet::Interaction)
            .before(RuleUiSet::Interaction)
            .before(UpsellUiSet::Interaction)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        gameplay_ui_actions::gm_runtime::pump
            .after(consume_world_gameplay_ui_outbox)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(Update, gameplay_ui_actions::gm_runtime::presentation.after(gameplay_ui_actions::gm_runtime::pump).before(ffone_client::localization::LocalizationSet::Apply))
    .add_systems(PostUpdate, gameplay_ui_actions::gm_world_labels::presentation.after(bevy::transform::TransformSystems::Propagate))
    .add_systems(
        Update,
        sync_local_avatar_presentation
            .after(ffone_client::world_behaviour::finish_world_zipline_steps)
            .after(consume_world_gameplay_ui_outbox)
            .after(NativeWorldSet::ResolveCollision)
            .after(ffone_client::world_behaviour::update_world_launcher_traversals)
            .before(LegacyAvatarActionSet::Locomotion)
            .before(process_tutorial_effect_runtime)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        drive_local_vehicle_toggle
            .after(LegacyMovementSet::ReadInput)
            .after(sync_tutorial_action_gate)
            .before(LegacyAvatarActionSet::Resolve)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        advance_pending_normal_npc_warp
            .after(consume_world_gameplay_ui_outbox)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        render_normal_npc_warp_window_in_fade
            .after(advance_pending_normal_npc_warp)
            .after(advance_tutorial_npc_warp)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        Update,
        sync_warp_presentation
            .after(advance_pending_guide_warp)
            .after(advance_pending_normal_npc_warp)
            .after(advance_tutorial_npc_warp)
            .after(consume_transportation_model_outbox)
            .after(enable_player_after_native_collider_ready)
            .before(MissionUiSet::Presentation)
            .before(sync_tutorial_choreography_visibility)
            .before(QuickSlotUiSet::Bind)
            .before(SkillBuffUiSet::Bind)
            .before(ffone_client::group_ui::GroupUiSet::Bind)
            .before(ffone_client::overheat_ui::OverheatUiSet::Bind),
    )
    .add_systems(
        Update,
        consume_rule_runtime
            .after(consume_world_gameplay_ui_outbox)
            .after(RuleUiSet::Interaction)
            .before(RuleUiSet::Bind),
    )
    .add_systems(
        Update,
        apply_world_npc_subtarget_camera
            .after(collect_world_npc_interactions)
            .after(consume_world_gameplay_ui_outbox)
            .after(consume_race_production_outputs)
            .after(LegacyMovementSet::CameraPose)
            // SubTargetCamera owns the final transform while an NPC mode is
            // open; normal player-camera collision must not overwrite it.
            .after(NativeWorldSet::ResolveCameraOcclusion)
            .before(ensure_and_update_legacy_skybox)
            .before(ensure_and_update_legacy_fusion_star)
            .run_if(shared_gameplay_world_active),
    )
    .add_systems(
        PostUpdate,
        (
            // Update the next frame's gate only after mission UI button
            // handlers have committed their final model state. The early
            // Update copy above still catches a newly applied actor
            // interaction before movement input is sampled.
            sync_tutorial_input_gate,
            sync_legacy_gameplay_cursor.in_set(ffone_client::input_focus::GameplayCursorSyncSet),
        ),
    );
    performance_probe::install(&mut app);
    gamepad_probe::install(&mut app);
    window_render_sync::install(&mut app);
    app.run();
    ExitCode::SUCCESS
}
