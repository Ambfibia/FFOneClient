use super::*;
use crate::app::npc_warp::normal_npc_warp_departure_commands;

#[test]
fn attack_success_reconciles_authoritative_weapon_battery_and_malformed_payload_is_atomic() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            weapon_battery: 100,
            ..default()
        },
        ..default()
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_ATTACK_NPCS_SUCC,
        flags: 0,
        checksum: 0,
        payload: PcAttackNpcsSuccess0104 {
            battery_w: 93,
            results: Vec::new(),
        }
        .encode()
        .unwrap(),
    };
    assert_eq!(apply_runtime_frame(&frame, &mut runtime), None);
    assert_eq!(runtime.weapon_battery, 93);

    let malformed = DecodedFrame {
        payload: frame.payload[..frame.payload.len() - 1].to_vec(),
        ..frame
    };
    assert_eq!(apply_runtime_frame(&malformed, &mut runtime), None);
    assert_eq!(
        runtime.weapon_battery, 93,
        "a malformed counted attack reply cannot partially overwrite authority"
    );
}

#[test]
fn name_invite_payload_is_safely_correlated_without_system_message_state() {
    let mut integration = BuddyRuntimeIntegration::default();
    let mut nanocom = NanocomMessageUiModel::default();
    let request_id = integration.push_nanocom_action(
        &mut nanocom,
        PendingBuddyNanocomAction::NameInvite {
            pc_uid: 4_002,
            first_name: "Gaia".to_owned(),
            last_name: "Roundbreath".to_owned(),
        },
        "Gaia Roundbreath",
    );

    assert_eq!(request_id, BUDDY_NANOCOM_MESSAGE_ID_BASE);
    assert_eq!(
        integration.pending_nanocom_actions.get(&request_id),
        Some(&PendingBuddyNanocomAction::NameInvite {
            pc_uid: 4_002,
            first_name: "Gaia".to_owned(),
            last_name: "Roundbreath".to_owned(),
        })
    );
    assert!(integration.pending_system_actions.is_empty());
    let active = nanocom.active().expect("name invite NanoCom head");
    assert_eq!(active.request.request_id, request_id);
    assert_eq!(
        active.request.body,
        "Gaia Roundbreath has invited you to be buddies."
    );
}

#[test]
fn marquee_row_warp_sends_departure_position_before_npc_2248_request() {
    let request = PcWarpUseNpcRequest0104 {
        npc_id: 2248,
        warp_id: 89,
        e_il_1: 4,
        item_slot_1: 0,
        e_il_2: 4,
        item_slot_2: 0,
    };
    let commands = normal_npc_warp_departure_commands(
        Vec3::new(-8151.0, -74.0, 6390.0),
        request,
    );
    let [NetworkCommand::Stop(position), NetworkCommand::UseNpcWarp(warp)] = commands else {
        panic!("departure must synchronize position before requesting NPC warp");
    };
    assert_eq!(position.position, [815100, 639000, -7400]);
    assert_eq!(warp.npc_id, 2248);
    assert_eq!(warp.warp_id, 89);
}

#[test]
fn marquee_row_npc_2248_warp_send_edge_disables_local_movement_and_packet_emission() {
    let identity = NormalNpcWarpIdentity {
        npc_id: 2248,
        npc_type: 1502,
        warp_id: 89,
        required_task_id: None,
        target: TutorialWarpTarget {
            map_id: 26,
            x: 165510,
            y: 350783,
            z: -3197,
        },
    };
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(NetworkBridge::start())
        .insert_resource(NormalNpcWarpRuntime {
            departure_clock: WarpDepartureClock {
                started: true,
                ..default()
            },
            pending: Some(PendingNormalNpcWarp {
                identity,
                request: PcWarpUseNpcRequest0104 {
                    npc_id: identity.npc_id,
                    warp_id: identity.warp_id,
                    e_il_1: 4,
                    item_slot_1: 0,
                    e_il_2: 4,
                    item_slot_2: 0,
                },
                elapsed_seconds: 0.0,
                sent: false,
            }),
            ..default()
        })
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialEffectRuntime>()
        .init_resource::<RuntimeStatus>()
        .add_systems(Update, advance_pending_normal_npc_warp);
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            Transform::from_xyz(-8151.68, -74.23, 6390.79),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(
            NORMAL_NPC_WARP_DELAY_SECONDS,
        ));
    app.update();

    let production = app.world().resource::<NormalNpcWarpRuntime>();
    assert!(production.pending.is_some_and(|pending| pending.sent));
    assert!(!production.movement_packet_emission);
    assert_eq!(production.window_in_fade_alpha, 1.0);
    assert!(
        !app.world()
            .get::<LegacyPlayerController>(player)
            .unwrap()
            .movement_enabled
    );
}

#[test]
fn race_frame_inbox_owns_only_clean_gameframe_race_family() {
    for packet_id in [
        RACE_INSTANCE_MAP_INFO_PACKET_ID,
        RACE_START_SUCCESS_PACKET_ID,
        RACE_START_FAILURE_PACKET_ID,
        RACE_END_SUCCESS_PACKET_ID,
        RACE_END_FAILURE_PACKET_ID,
        RACE_CANCEL_SUCCESS_PACKET_ID,
        RACE_CANCEL_FAILURE_PACKET_ID,
        RACE_GET_RING_SUCCESS_PACKET_ID,
        RACE_GET_RING_FAILURE_PACKET_ID,
    ] {
        assert!(race_frame_owned(packet_id), "missing {packet_id:#010x}");
    }
    assert!(!race_frame_owned(0x3100_00cf));
    assert!(!race_frame_owned(packet::P_FE2CL_PC_MOVE));

    let mut inbox = RaceNetworkFrameInbox::default();
    assert!(inbox.push_if_owned(DecodedFrame {
        packet_type: RACE_GET_RING_SUCCESS_PACKET_ID,
        flags: 0,
        checksum: 0,
        payload: vec![0; 8],
    }));
    assert!(!inbox.push_if_owned(DecodedFrame {
        packet_type: packet::P_FE2CL_PC_MOVE,
        flags: 0,
        checksum: 0,
        payload: Vec::new(),
    }));
    assert_eq!(
        inbox.pop().unwrap().packet_type,
        RACE_GET_RING_SUCCESS_PACKET_ID
    );
    assert!(inbox.pop().is_none());
}
