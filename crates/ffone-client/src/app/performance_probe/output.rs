use super::*;

pub(in super::super) fn install(app: &mut App) {
    if let Some(output) = env::var_os("FFONE_CIVILIAN_ROUTES_PROBE_OUTPUT") {
        civilian_routes_network::install(app, PathBuf::from(output));
        return;
    }
    if let Some(output) = env::var_os("FFONE_NPC_SKILL_NETWORK_PROBE_OUTPUT") {
        npc_skill_network::install(app, PathBuf::from(output));
        return;
    }
    if let Some(output) = env::var_os("FFONE_CHARACTER_SESSION_PROBE_OUTPUT") {
        character_session_network::install(app, PathBuf::from(output));
        return;
    }
    if let Some(output) = env::var_os("FFONE_QUIT_MENU_NETWORK_PROBE_OUTPUT") {
        quit_menu_network::install(app, PathBuf::from(output));
        return;
    }
    if let Some(output) = env::var_os("FFONE_NANO_ATTACK_PROBE_OUTPUT") {
        nano_attack_network::install(app, PathBuf::from(output));
        return;
    }
    if let Some(output) = env::var_os("FFONE_TUTORIAL_NETWORK_PROBE_OUTPUT") {
        tutorial_network::install(app, PathBuf::from(output));
        return;
    }
    if let Some(output) = env::var_os("FFONE_CUTSCENE_PROBE_OUTPUT") {
        dexter_cutscenes::install(app, PathBuf::from(output));
        return;
    }
    if let Some(output) = env::var_os("FFONE_CREATION_PROBE_OUTPUT") {
        character_creation::install(app, PathBuf::from(output));
        return;
    }
    let Some(output) = env::var_os("FFONE_PERF_OUTPUT") else {
        return;
    };
    if env::var_os("FFONE_PERF_CHAT_COMMANDS").is_some() { chat_commands::install(app); }
    visual_effects::install(app);
    if env::var_os("FFONE_PERF_TRAVERSAL").is_some() { traversal::install(app); }
    if env::var_os("FFONE_PERF_NPC_INTERACTION_RANGE").is_some() { npc_interaction_range::install(app); }
    if env::var_os("FFONE_PERF_NPC_BUBBLE_VISIBILITY").is_some() { npc_bubble_visibility::install(app); }
    if env::var_os("FFONE_PERF_INVENTORY_AVAILABILITY").is_some() {
        inventory_availability::install(app);
    }
    if env::var_os("FFONE_PERF_NANO_HUD").is_some() { nano_hud::install(app); }
    if env::var_os("FFONE_PERF_JOURNAL_SELECTION").is_some() { journal_selection::install(app); }
    if env::var_os("FFONE_PERF_AUDIO").is_some() { audio_regression::install(app); }
    if env::var_os("FFONE_PERF_UI_SFX_TRACE").is_some() { ui_sfx_trace::install(app); }
    if env::var_os("FFONE_PERF_NPC_SPEECH").is_some() { npc_speech::install(app); }
    if env::var_os("FFONE_PERF_LOADING_WARP").is_some() { location_loading::install(app); }
    if env::var_os("FFONE_PERF_PLAYER_FACE").is_some() { player_face::install(app); }
    if env::var_os("FFONE_PERF_NPC_FLOOR").is_some() { npc_floor::install(app); }
    if env::var_os("FFONE_PERF_RACE_PODS").is_some() { race_pods::install(app); }
    if env::var_os("FFONE_PERF_GAMEPLAY").is_some() { gameplay_regression::install(app); }
    if env::var_os("FFONE_PERF_TUTORIAL_FINALE").is_some() { tutorial_finale::install(app); }
    if env::var_os("FFONE_PERF_TUTORIAL_COMBAT_HUD").is_some() { tutorial_combat_hud::install(app); }
    if env::var_os("FFONE_PERF_QUICK_CHAT").is_some() { quick_chat::install(app); }
    if env::var_os("FFONE_PERF_SKILL_HITS").is_some() { skill_effects::install(app); }
    if env::var_os("FFONE_PERF_RESOLUTION").is_some() { resolution_regression::install(app); }
    if env::var_os("FFONE_PERF_ENCHANT").is_some() {
        enchant_regression::install(app);
    }
    if env::var_os("FFONE_PERF_EMAIL").is_some() {
        email_regression::install(app);
    }
    if env::var_os("FFONE_PERF_NANO_ACQUISITION").is_some() {
        nano_regression::install(app);
    }
    if let Ok(item) = env::var("FFONE_PERF_VEHICLE") {
        let item: i16 = item.parse().expect("positive vehicle item id");
        assert!(item > 0);
        app.insert_resource(VehicleFixture(item)).add_systems(
            Update,
            mount_vehicle_fixture
                .after(NetworkSessionLifecycleSet::Apply)
                .before(movement_buffs::sync_movement_buffs)
                .before(sync_local_avatar_presentation),
        );
    }
    if env::var_os("FFONE_PERF_ABILITY_PRESENTATION").is_some() {
        ability_presentation::install(app);
    }
    if env::var_os("FFONE_PERF_COCO").is_some() {
        coco::install(app);
    }
    if env::var_os("FFONE_PERF_MOB_ANIMATION").is_some()
        || env::var_os("FFONE_PERF_NPC_IDLE").is_some()
        || env::var_os("FFONE_PERF_SPECIAL_SKILLS").is_some()
    {
        mob_animation::install(app);
    }
    if env::var_os("FFONE_PERF_PC2PC").is_some() { pc2pc_interaction::install(app); }
    if env::var_os("FFONE_PERF_PLAYER_MENU").is_some() { player_menu::install(app); }
    if env::var_os("FFONE_PERF_SERVICE_DIALOGUE").is_some() {
        service_dialogue::install(app);
    }
    if env::var_os("FFONE_PERF_GUIDE_NANOCOM").is_some() {
        guide_nanocom::install(app);
    }
    // Restore the former repeated-construction bug only in this explicit A/B
    // fixture, so both sides use the same executable and input isolation.
    if env::var_os("FFONE_PERF_UI_REENTRY_BASELINE").is_some() {
        app.add_systems(
            OnEnter(NativeUiStartupPhase::Gameplay),
            (|world: &mut World| {
                world.run_schedule(ffone_client::ui_startup::NativeGameplayUiStartup);
            })
            .run_if(|capture: Res<Capture>| capture.entry > 1),
        );
    }
    if env::var_os("FFONE_PERF_VENDOR_PORTRAIT").is_some() {
        app.add_systems(
            Update,
            open_vendor_portrait_fixture
                .after(sync_vendor_ui_context)
                .before(VendorUiSet::Bind),
        );
    }
    if env::var_os("FFONE_PERF_VENDOR_REGRESSION").is_some() {
        vendor_regression::install(app);
    }
    if env::var_os("FFONE_PERF_REWARDS").is_some() {
        app.add_systems(
            Update,
            show_reward_fixture
                .after(NetworkSessionLifecycleSet::Apply)
                .after(network_ingress::sync_reward_inventory)
                .before(ffone_client::gameplay_ui::GameplayUiSet::Rewards),
        );
    }
    if env::var_os("FFONE_PERF_RETROBUTION_MAP").is_some() {
        app.add_systems(
            Update,
            open_retrobution_map_fixture
                .after(NetworkSessionLifecycleSet::Apply)
                .after(handle_world_map_input)
                .before(sync_world_map_projection),
        );
        app.add_systems(
            Update,
            highlight_retrobution_transport_fixture
                .after(sync_world_map_projection)
                .before(ffone_client::world_map::WorldMapPresentationSet::Bind),
        );
    }
    // An unfocused normal game window intentionally sleeps at 60 Hz. Wall-time
    // benchmarks must not measure that power-saving wait when focus changes.
    app.insert_resource(bevy::winit::WinitSettings::continuous());
    if env::var_os("FFONE_PERF_GPU").is_some() {
        app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
    }
    let tutorial_entry = env::var_os("FFONE_PERF_TUTORIAL").is_some();
    let position: Vec<f32> = env::var("FFONE_PERF_POSITION")
        .unwrap_or_else(|_| {
            if tutorial_entry {
                "-547 -105.4 655".into()
            } else if env::var_os("FFONE_PERF_NPC_SPEECH").is_some() {
                "-4779.07 -52.61 3482.06".into()
            } else {
                "-6374.3188 -56.4746 668.2039".into()
            }
        })
        .split_whitespace()
        .map(|s| s.parse().expect("native coordinate"))
        .collect();
    assert!(position.len() == 3 && position.iter().all(|v| v.is_finite()));
    let output = PathBuf::from(output);
    let entries = env::var("FFONE_PERF_ENTRIES")
        .map(|value| value.parse::<usize>().expect("positive entry count"))
        .unwrap_or(1);
    assert!(entries > 0);
    let output_root = output.clone();
    let output = if entries > 1 {
        output.join("entry-1")
    } else {
        output
    };
    fs::create_dir_all(&output).unwrap();
    // Interactive entry preloads this during character selection; the offline
    // fixture enters World directly and must complete the same prerequisite.
    let effects = TutorialEffectLibrary::load(&app.world().resource::<ClientConfig>().asset_root)
        .expect("native effect library");
    let effects = Arc::new(effects);
    let mut effect_runtime = TutorialEffectRuntime::default();
    effect_runtime.install_shared_library(Arc::clone(&effects));
    app.insert_resource(SharedTutorialEffectLibrary(effects))
        .insert_resource(effect_runtime)
        .insert_resource(TutorialEffectLibraryLoadStatus::Ready);
    let now = Instant::now();
    let frozen = env::var_os("FFONE_PERF_FREEZE").is_some();
    app.insert_resource(Capture {
        output,
        output_root,
        entries,
        entry: 1,
        restart: false,
        selection_since: None,
        position: Vec3::from_slice(&position),
        started: now,
        ready: None,
        previous: now,
        samples: Vec::new(),
        captured: false,
        frozen,
        orbit: env::var_os("FFONE_PERF_ORBIT").is_some(),
        transport_npc_type: env::var("FFONE_PERF_TRANSPORT_NPC_TYPE")
            .ok()
            .map(|v| v.parse().expect("NPC type")),
        transport_opened: false,
        transport_diagnostics_written: false,
        npc_chat_probe: env::var_os("FFONE_PERF_NPC_CHAT").is_some(),
        npc_chat_requested: false,
        focus_probe: env::var_os("FFONE_PERF_INPUT_FOCUS").is_some(),
        focus_phase: 0,
    })
    .add_systems(PostStartup, setup)
    .add_systems(First, drive)
    .add_systems(
        PreUpdate,
        discard_live_fixture_input.before(bevy::input::InputSystems),
    )
    .add_systems(
        Update,
        open_transport_fixture.after(NetworkSessionLifecycleSet::Apply),
    )
    .add_systems(Last, measure);
    // A fixed simulation step makes repeated captures and camera paths comparable.
    // Wall frame intervals below are measured independently of simulation time.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(if frozen {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(1.0 / 60.0)
    }));
}

pub(super) fn save(
    event: On<ScreenshotCaptured>,
    mut capture: ResMut<Capture>,
    mut exit: MessageWriter<AppExit>,
) {
    event
        .image
        .clone()
        .try_into_dynamic()
        .unwrap()
        .save(capture.output.join("frame.png"))
        .unwrap();
    if capture.entry < capture.entries {
        capture.restart = true;
    } else if env::var_os("FFONE_PERF_INTERACTIVE").is_none() {
        exit.write(AppExit::Success);
    }
}
