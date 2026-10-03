use super::*;
use crate::app::npc_warp::normal_npc_warp_ui_entry;
use ffone_client::mission_ui::WarpUiEntry;
use std::time::Duration;

// Exercise the real shared-world consumer, including all modal/resource gates.
pub(super) fn world_warp_app() -> (App, Entity, WarpUiEntry) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let assets = AssetLocator::open(root.clone()).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    let warp = normal_npc_warp_ui_entry(&content, 44_001, 682).unwrap();
    let (localization, language) = Localization::open(&root, "ru").unwrap();
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(Time::<()>::default())
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(TransportationCatalog::open(&assets).unwrap())
        .insert_resource(EmailProductionCatalog0104::open(&assets, &content).unwrap())
        .insert_resource(CombiProductionCatalog0104::open(&assets, &content).unwrap())
        .insert_resource(NativeAudioCatalog::open(&root, false).unwrap())
        .insert_resource(GuideRuntime::new(GuideServerProfile::OpenFusion0104))
        .insert_resource(NetworkBridge::start())
        .insert_resource(TutorialEffectRuntime::with_library(
            TutorialEffectLibrary::load(&root).unwrap(),
        ))
        .insert_resource(content)
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<WorldMissionRuntime>()
        .init_resource::<MissionDeleteConfirmationRuntime>()
        .init_resource::<TutorialMissionRuntime>()
        .init_resource::<GuideUiOutbox>()
        .init_resource::<BankUiState>()
        .init_resource::<BankUiOutbox0104>()
        .init_resource::<BankProductionRuntime0104>()
        .init_resource::<VendorUiState>()
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorProductionRuntime0104>()
        .init_resource::<CashmallUiState0104>()
        .init_resource::<CashmallUiOutbox0104>()
        .init_resource::<UserStoreUiState0104>()
        .init_resource::<WorldMapPresentation>()
        .init_resource::<WorldMapPresentationAssetStatus>()
        .init_resource::<OptionUiModel>()
        .init_resource::<OptionUiOutbox>()
        .init_resource::<OptionUiAssetGate>()
        .init_resource::<OptionProductionRuntime>()
        .init_resource::<RuleUiModel>()
        .init_resource::<RuleUiOutbox>()
        .init_resource::<RuleRuntime>()
        .init_resource::<NanoFreeTuningModel>()
        .init_resource::<NanoFreeTuningBank0104>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .init_resource::<TransportationModel>()
        .init_resource::<TransportationProductionRuntime>()
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<QuitMenuUiModel>()
        .init_resource::<QuitMenuRuntime>()
        .init_resource::<ResurrectUiModel>()
        .init_resource::<RaceModeModel>()
        .init_resource::<RaceRankCatalog>()
        .init_resource::<RaceProductionRuntime>()
        .init_resource::<EmailProductionRuntime0104>()
        .init_resource::<EmailUiModel>()
        .init_resource::<EmailUiOutbox>()
        .init_resource::<EmailUiAudioOutbox>()
        .init_resource::<EmailTransportOutbox>()
        .init_resource::<EmailNetworkRuntime0104>()
        .init_resource::<EmailNetworkInbox0104>()
        .init_resource::<EmailProductionShell0104>()
        .init_resource::<CombiProductionRuntime0104>()
        .init_resource::<ffone_client::barber::BarberModel>()
        .init_resource::<CombiProductionShell0104>()
        .init_resource::<EnchantProductionRuntime0104>()
        .init_resource::<EnchantProductionShell0104>()
        .init_resource::<GameplayAudioRuntime>()
        .init_resource::<GroupUiModel>()
        .init_resource::<LocalInventoryRuntime>()
        .init_resource::<NormalNpcWarpRuntime>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<BuddyUiModel>()
        .init_resource::<MissionUiModel>()
        .init_resource::<NanocomMessageUiModel>()
        .init_resource::<GuideUiModel>()
        .init_resource::<GameGuideUiModel>()
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<UpsellUiModel>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<ffone_client::user_equip_ui::UserEquipNanoViewerState>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        .init_resource::<RuntimeStatus>()
        .add_systems(Update, consume_world_gameplay_ui_outbox);
    {
        let mut status = app.world_mut().resource_mut::<RuntimeStatus>();
        status.map_number = Some(0);
        status.player_level = 36;
        status.candy = 1_000_000;
    }
    app.world_mut().resource_mut::<GroupUiModel>().local_pc_uid = Some(123);
    app.world_mut().spawn((
        NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
            npc_id: warp.npc_id,
            npc_type: warp.npc_type,
            hp: 100,
            condition_bit_flag: 0,
            position: [100, 200, 300],
            angle: 0,
            barker_type: 0,
        }),
        Transform::from_xyz(1.0, 2.0, 3.0),
        GlobalTransform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
    ));
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            Transform::from_xyz(4.0, 5.0, 6.0),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();
    let mut outbox = GameplayUiOutbox::default();
    {
        let mut ui = app.world_mut().resource_mut::<MissionUiModel>();
        ui.enabled = true;
        ui.show_npc_interaction(NpcInteractionUi {
            npc_id: warp.npc_id,
            npc_type: warp.npc_type,
            warp: Some(warp.clone()),
            ..default()
        });
        assert!(ui.activate_npc_utility(0, &mut outbox));
    }
    *app.world_mut().resource_mut::<GameplayUiOutbox>() = outbox;
    (app, player, warp)
}

#[test]
fn ordinary_world_warp_click_reaches_departure_effect_and_network_delay() {
    let (mut app, player, warp) = world_warp_app();
    app.update();
    assert!(
        app.world()
            .resource::<NormalNpcWarpRuntime>()
            .pending
            .is_some(),
        "{}",
        app.world().resource::<RuntimeStatus>().message
    );
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    let mut effects = app.world_mut().resource_mut::<TutorialEffectRuntime>();
    effects.process_pending();
    let records = effects.drain_records().collect::<Vec<_>>();
    assert!(records.iter().any(|record| matches!(record.disposition,
        ffone_client::tutorial_effects_runtime::TutorialEffectRuntimeDisposition::NativeEffectQueued {
            rendered_nodes: 1.., ..
        })), "{records:?}");
    assert!(records.iter().any(|record| matches!(record.command,
        TutorialEffectRuntimeCommand::Add { effect_id: NORMAL_NPC_WARP_EFFECT_ID,
            placement: TutorialEffectPlacement::World { position, .. }, .. }
            if position == Vec3::new(4.0, 5.0, 6.0))));
    let instance_id = records.iter().find_map(|record| match record.disposition {
        ffone_client::tutorial_effects_runtime::TutorialEffectRuntimeDisposition::NativeEffectQueued {
            instance_id, ..
        } => Some(instance_id),
        _ => None,
    }).unwrap();
    drop(effects);
    app.add_systems(
        Update,
        advance_pending_normal_npc_warp.after(consume_world_gameplay_ui_outbox),
    );
    // A cold mesh can take longer than the entire departure delay. Queuing
    // its instance must not authorize a packet or consume the visible wait.
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(5));
    app.update();
    let pending = app
        .world()
        .resource::<NormalNpcWarpRuntime>()
        .pending
        .unwrap();
    assert!(!pending.sent);
    assert_eq!(pending.elapsed_seconds, 0.0);
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .mark_native_presentation_ready(instance_id);
    app.update();
    assert_eq!(
        app.world()
            .resource::<NormalNpcWarpRuntime>()
            .pending
            .unwrap()
            .elapsed_seconds,
        0.0
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.49));
    app.update();
    assert!(
        !app.world()
            .resource::<NormalNpcWarpRuntime>()
            .pending
            .unwrap()
            .sent
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.02));
    app.update();
    assert!(
        !app.world()
            .resource::<NormalNpcWarpRuntime>()
            .pending
            .unwrap()
            .sent,
        "the original 1.5-second wait must not cut off the live effect tail"
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.49));
    app.update();
    assert!(
        !app.world()
            .resource::<NormalNpcWarpRuntime>()
            .pending
            .unwrap()
            .sent
    );
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .clear_scene_instances();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(1));
    app.update();
    let pending = app
        .world()
        .resource::<NormalNpcWarpRuntime>()
        .pending
        .unwrap();
    assert!(pending.sent);
    assert_eq!(pending.request.npc_id, warp.npc_id);
    assert_eq!(pending.request.warp_id, warp.warp_id);
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        Vec3::new(4.0, 5.0, 6.0),
        "ordinary world coordinates stay server-authoritative"
    );
}
