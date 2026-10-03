use super::*;
use crate::app::guide::PendingGuideWarp;
use crate::app::npc_warp::TRANSPORTATION_WARP_PRESENTATION;
use crate::app::race::RACE_INSTANCE_MAP_INFO_BASE_SIZE;
use crate::app::race::RACE_INSTANCE_MAP_INFO_PACKET_ID;
use ffone_client::mission_ui::WarpUiEntry;
use ffone_client::transportation_ui::TransportationUnlocks;
use ffone_client::tutorial_effects_runtime::TutorialEffectRuntimeDisposition;

#[test]
fn time_machine_warp_waits_for_native_effect_and_keeps_transition_during_loading() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut effects =
        TutorialEffectRuntime::with_library(TutorialEffectLibrary::load(root).unwrap());
    effects.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: NORMAL_NPC_WARP_EFFECT_ID,
        placement: TutorialEffectPlacement::World {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        },
        scale: 1.0,
        tracked: false,
        name: Some(crate::app::guide::GUIDE_FIRST_WARP_PRESENTATION.to_owned()),
        destroy_after_seconds: None,
        source_line: line!(),
    });
    effects.process_pending();
    let instance_id = effects
        .drain_records()
        .find_map(|record| match record.disposition {
            TutorialEffectRuntimeDisposition::NativeEffectQueued { instance_id, .. } => {
                Some(instance_id)
            }
            _ => None,
        })
        .unwrap();

    let mut guide = GuideProductionRuntime::default();
    guide.pending_warp = Some(PendingGuideWarp {
        npc_id: 123,
        map_number: 2,
        target: [0; 3],
        elapsed_seconds: 0.0,
        sent: false,
    });
    let mut app = App::new();
    app.init_resource::<Time>()
        .insert_resource(State::new(ClientState::World))
        .init_resource::<GameplayLoadingState>()
        .init_resource::<NormalNpcWarpRuntime>()
        .init_resource::<TransportationModel>()
        .init_resource::<MissionUiModel>()
        .insert_resource(NetworkBridge::start())
        .insert_resource(effects)
        .insert_resource(guide)
        .init_resource::<RuntimeStatus>()
        .add_systems(
            Update,
            (advance_pending_guide_warp, sync_warp_presentation).chain(),
        );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(5));
    app.update();
    assert!(
        !app.world()
            .resource::<GuideProductionRuntime>()
            .pending_warp
            .unwrap()
            .sent
    );
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .mark_native_presentation_ready(instance_id);
    app.update();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(2.0));
    app.update();
    assert!(
        !app.world()
            .resource::<GuideProductionRuntime>()
            .pending_warp
            .unwrap()
            .sent
    );
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .clear_scene_instances();
    app.update();
    assert!(
        app.world()
            .resource::<GuideProductionRuntime>()
            .pending_warp
            .unwrap()
            .sent
    );
    app.world_mut()
        .resource_mut::<GuideProductionRuntime>()
        .pending_warp = None;
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .begin(ResourceLoadingScope::World);
    app.update();
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
}

#[test]
fn warp_instance_notification_enables_lair_audio_without_a_race() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<RaceNetworkFrameInbox>()
        .init_resource::<RaceModeModel>()
        .init_resource::<RaceProductionRuntime>()
        .init_resource::<RaceRewardPresentation>()
        .insert_resource(runtime_test_mission_content())
        .init_resource::<RetrobutionInstanceAudioState>()
        .init_resource::<RuntimeStatus>()
        .add_systems(Update, consume_race_network_frames);
    let frame = DecodedFrame {
        packet_type: RACE_INSTANCE_MAP_INFO_PACKET_ID,
        flags: 0,
        checksum: 0,
        payload: vec![0; RACE_INSTANCE_MAP_INFO_BASE_SIZE],
    };
    app.world_mut()
        .resource_mut::<RaceNetworkFrameInbox>()
        .push_if_owned(DecodedFrame {
            payload: vec![0; RACE_INSTANCE_MAP_INFO_BASE_SIZE - 1],
            ..frame.clone()
        });
    app.update();
    assert!(
        !app.world()
            .resource::<RetrobutionInstanceAudioState>()
            .active
    );
    app.world_mut()
        .resource_mut::<RaceNetworkFrameInbox>()
        .push_if_owned(frame);
    app.update();
    assert!(
        app.world()
            .resource::<RetrobutionInstanceAudioState>()
            .active
    );
    assert_eq!(
        app.world()
            .resource::<RaceProductionRuntime>()
            .player
            .current_ep_id,
        0
    );
}

#[test]
fn warp_loading_acknowledgment_waits_for_ready_destination_and_sends_once() {
    let request = warp_loading_complete_request();
    assert_eq!(
        request.packet_type(),
        packet::P_CL2FE_REQ_PC_LOADING_COMPLETE
    );
    assert_eq!(request.payload(), 0_i32.to_le_bytes());

    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .init_resource::<GameplayLoadingState>()
        .init_resource::<RuntimeStatus>()
        .insert_resource(NetworkBridge::start())
        .add_systems(Update, acknowledge_warp_loading);
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            WarpLoadingAcknowledgment,
            LegacyWorldColliderPending,
        ))
        .id();
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .begin(ResourceLoadingScope::World);
    app.update();
    assert!(
        app.world()
            .entity(player)
            .contains::<WarpLoadingAcknowledgment>()
    );
    // Closing the screen alone must not bypass destination readiness.
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .finish();
    app.update();
    assert!(
        app.world()
            .entity(player)
            .contains::<WarpLoadingAcknowledgment>()
    );
    app.world_mut()
        .entity_mut(player)
        .remove::<LegacyWorldColliderPending>();
    app.update();
    assert!(
        !app.world()
            .entity(player)
            .contains::<WarpLoadingAcknowledgment>()
    );
    app.update();
    assert!(
        !app.world()
            .entity(player)
            .contains::<WarpLoadingAcknowledgment>()
    );

    // A subsequent warp has its own acknowledgment, but a disconnected/menu
    // frame cannot acknowledge the abandoned destination.
    app.world_mut()
        .entity_mut(player)
        .insert(WarpLoadingAcknowledgment);
    app.world_mut()
        .insert_resource(State::new(ClientState::Login));
    app.update();
    assert!(
        app.world()
            .entity(player)
            .contains::<WarpLoadingAcknowledgment>()
    );
}

#[test]
fn warp_presentation_hides_hud_through_arrival_without_hiding_player() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(State::new(ClientState::World))
        .init_resource::<GameplayLoadingState>()
        .init_resource::<NormalNpcWarpRuntime>()
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<TransportationModel>()
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialChoreographyPresentation>()
        .add_systems(
            Update,
            (
                sync_warp_presentation,
                sync_tutorial_choreography_visibility,
            )
                .chain(),
        );
    let hud = app
        .world_mut()
        .spawn((GameplayHud, Visibility::Inherited))
        .id();
    let player = app
        .world_mut()
        .spawn((LocalCharacterScene, Visibility::Inherited))
        .id();
    {
        let mut mission = app.world_mut().resource_mut::<MissionUiModel>();
        mission.enabled = true;
        mission.npc_interaction = Some(tutorial_gate_test_interaction());
        mission.pending_warp = Some(WarpUiEntry::default());
    }
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        *app.world().get::<Visibility>(player).unwrap(),
        Visibility::Inherited
    );
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .npc_letterbox_visible()
    );

    // The reply consumes the pending warp before the destination is renderable.
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .pending_warp = None;
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .begin(ResourceLoadingScope::World);
    app.update();
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Hidden
    );
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .visible = false;
    app.update();
    assert!(
        !app.world()
            .resource::<MissionUiModel>()
            .npc_letterbox_visible()
    );
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Inherited
    );

    // A failed request without a load must release the transition too.
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .pending_warp = Some(WarpUiEntry::default());
    app.update();
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .pending_warp = None;
    app.update();
    assert!(
        !app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Inherited
    );

    // Leaving gameplay cannot leave the presentation latch set.
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .warp_transition_active = true;
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .begin(ResourceLoadingScope::World);
    app.insert_resource(State::new(ClientState::Login));
    app.update();
    assert!(
        !app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
}

#[test]
fn transportation_warp_waits_for_visible_departure_before_sending() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let assets = AssetLocator::open(&root).unwrap();
    let catalog = TransportationCatalog::open(&assets).unwrap();
    let mut model = TransportationModel::default();
    model
        .open(
            &catalog,
            TransportationOpenContext {
                player: TransportationPlayerSnapshot {
                    position: TransportationWorldPoint::new(4_000.0, 120.0, 4_000.0),
                    taros: 193,
                    unlocks: TransportationUnlocks {
                        warp_location_flags: 2,
                        wyvern_location_flags: [0; 2],
                    },
                    cursor_was_locked: true,
                },
                target: TransportationTarget::Npc {
                    npc_instance_id: 9_001,
                    npc_table_id: 964,
                    npc_position: TransportationWorldPoint::new(4_100.0, 100.0, 4_100.0),
                    has_move_ok_voice: true,
                },
            },
        )
        .unwrap();
    model.select_route(0).unwrap();
    model.press_go().unwrap();
    let mut effects =
        TutorialEffectRuntime::with_library(TutorialEffectLibrary::load(root).unwrap());
    effects.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: NORMAL_NPC_WARP_EFFECT_ID,
        placement: TutorialEffectPlacement::World {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        },
        scale: 1.0,
        tracked: false,
        name: Some(TRANSPORTATION_WARP_PRESENTATION.to_owned()),
        destroy_after_seconds: None,
        source_line: line!(),
    });
    effects.process_pending();
    let instance_id = effects
        .drain_records()
        .find_map(|record| match record.disposition {
            TutorialEffectRuntimeDisposition::NativeEffectQueued { instance_id, .. } => {
                Some(instance_id)
            }
            _ => None,
        })
        .unwrap();
    let mut app = App::new();
    let mut production = TransportationProductionRuntime::default();
    production.begin_npc(9_001, false, TransportationService::Warp, None);
    app.init_resource::<Time>()
        .insert_resource(State::new(ClientState::World))
        .init_resource::<GameplayLoadingState>()
        .init_resource::<NormalNpcWarpRuntime>()
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<MissionUiModel>()
        .insert_resource(NetworkBridge::start())
        .insert_resource(model)
        .insert_resource(effects)
        .insert_resource(production)
        .init_resource::<RuntimeStatus>()
        .add_systems(
            Update,
            (advance_transportation_production, sync_warp_presentation).chain(),
        );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(5));
    app.update();
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
    assert_eq!(
        app.world().resource::<TransportationModel>().phase(),
        TransportationPhase::PendingWarp
    );
    assert!(
        app.world()
            .resource::<TransportationProductionRuntime>()
            .pending_warp
            .is_none()
    );

    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .mark_native_presentation_ready(instance_id);
    app.update(); // The five-second loading frame is not part of the departure.
    assert_eq!(
        app.world().resource::<TransportationModel>().phase(),
        TransportationPhase::PendingWarp
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(1.5));
    app.update();
    assert_eq!(
        app.world().resource::<TransportationModel>().phase(),
        TransportationPhase::PendingWarp
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.4));
    app.update();
    assert_eq!(
        app.world().resource::<TransportationModel>().phase(),
        TransportationPhase::PendingWarp,
        "a fast server must not receive the request while the effect tail is alive"
    );
    // The clock must release the elapsed wait when the departure has expired.
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .clear_scene_instances();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(101));
    app.update();
    assert_eq!(
        app.world().resource::<TransportationModel>().phase(),
        TransportationPhase::AwaitingServer
    );
    assert!(
        app.world()
            .resource::<TransportationProductionRuntime>()
            .pending_warp
            .is_some()
    );
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
    app.world_mut()
        .resource_mut::<TransportationModel>()
        .receive_success();
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .begin(ResourceLoadingScope::World);
    app.update();
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .visible = false;
    app.update();
    assert!(
        !app.world()
            .resource::<MissionUiModel>()
            .warp_transition_active
    );
}
