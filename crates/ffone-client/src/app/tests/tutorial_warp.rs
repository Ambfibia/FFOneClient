use crate::app::tests::*;
use ffone_client::mission_ui::WarpUiEntry;
use std::time::Duration;

fn warp_app() -> (App, Entity, Entity, WarpUiEntry) {
    let content = runtime_test_mission_content();
    let definition = content.warp(2694).unwrap();
    let warp = WarpUiEntry {
        npc_id: 6001,
        npc_type: 2694,
        warp_id: definition.provenance.warp_id,
        required_task_id: definition.required_task_id,
        target: definition.target,
        label: definition.label.clone(),
    };
    let mut mission = TutorialMissionRuntime::default();
    mission.start_task(&content, 2251).unwrap();
    let mut status = RuntimeStatus::default();
    status.map_number = Some(0);
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(content)
        .insert_resource(mission)
        .insert_resource(NetworkBridge::start())
        .insert_resource(NextState::<ClientState>::default())
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialActorCommandQueue>()
        .init_resource::<TutorialAmbientRuntime>()
        .insert_resource(TutorialEffectRuntime::with_library(
            TutorialEffectLibrary::load(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
            )
            .unwrap(),
        ))
        .init_resource::<GameplayAudioRuntime>()
        .init_resource::<NormalNpcWarpRuntime>()
        .insert_resource(status)
        .add_systems(
            Update,
            (consume_tutorial_mission_outbox, advance_tutorial_npc_warp).chain(),
        );
    let npc = app
        .world_mut()
        .spawn(TutorialActor {
            id: warp.npc_id,
            npc_type: warp.npc_type,
            team: 1,
            hp: 100,
            max_hp: 100,
            damaged: false,
            interacting: true,
            invulnerable: false,
        })
        .id();
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            Transform::from_xyz(1.0, 2.0, 3.0),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();
    let mut outbox = GameplayUiOutbox::default();
    let mut model = app.world_mut().resource_mut::<MissionUiModel>();
    model.enabled = true;
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: warp.npc_id,
        npc_type: warp.npc_type,
        warp: Some(warp.clone()),
        ..default()
    });
    assert!(model.activate_npc_utility(0, &mut outbox));
    // Shared chat actions must survive whichever consumer runs first.
    outbox.push(GameplayUiAction::ToggleMenuChat);
    *app.world_mut().resource_mut::<GameplayUiOutbox>() = outbox;
    (app, player, npc, warp)
}

#[test]
fn tutorial_warp_click_starts_departure_before_delayed_local_teleport() {
    let (mut app, player, _, warp) = warp_app();
    // Exercise the shared world's selection before the tutorial consumer.
    let shared = app
        .world_mut()
        .resource_mut::<GameplayUiOutbox>()
        .drain_matching(|action| !tutorial_owns_gameplay_action(action));
    assert_eq!(shared, vec![GameplayUiAction::ToggleMenuChat]);
    app.update();
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        Vec3::new(1.0, 2.0, 3.0)
    );
    let mut effects = app.world_mut().resource_mut::<TutorialEffectRuntime>();
    effects.process_pending();
    let records = effects.drain_records().collect::<Vec<_>>();
    assert!(records.iter().any(|record| matches!(record.disposition,
        ffone_client::tutorial_effects_runtime::TutorialEffectRuntimeDisposition::NativeEffectQueued {
            rendered_nodes: 1.., ..
        })), "departure effect must compile from production assets: {records:?}");
    assert!(records.iter().any(|record| matches!(record.command,
        TutorialEffectRuntimeCommand::Add { effect_id: NORMAL_NPC_WARP_EFFECT_ID,
            placement: TutorialEffectPlacement::World { position, .. }, .. }
            if position == Vec3::new(1.0, 2.0, 3.0))));
    drop(effects);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.49));
    app.update();
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        Vec3::new(1.0, 2.0, 3.0)
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.02));
    app.update();
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        ProtocolPosition::new([warp.target.x, warp.target.y, warp.target.z]).to_native()
    );
    assert_eq!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .completed_warp_ids,
        vec![warp.warp_id]
    );
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .pending_warp
            .is_none()
    );
    assert_eq!(
        app.world()
            .resource::<NormalNpcWarpRuntime>()
            .window_in_fade_alpha,
        1.0
    );
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .completed_warp_ids,
        vec![warp.warp_id]
    );
}

#[test]
fn tutorial_warp_preserves_shared_actions_and_cancels_when_npc_disappears() {
    let (mut app, player, npc, _) = warp_app();
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::ToggleMenuChat]
    );
    app.world_mut().despawn(npc);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(2));
    app.update();
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        Vec3::new(1.0, 2.0, 3.0)
    );
    assert!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .pending_warp
            .is_none()
    );
    assert!(
        app.world()
            .resource::<MissionUiModel>()
            .pending_warp
            .is_none()
    );
}
