use super::*;

#[test]
fn vehicle_state_packets_change_existing_remote_player_without_replacing_equipment() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let mut appearance = pc_appearance(11, [100, 200, 300]);
    appearance.equipment[8].item_type = 10;
    appearance.equipment[8].item_id = 107;
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_NEW, PcNew0104 { appearance }.encode()),
    );
    app.update();
    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    for state in [8, 0, 8, 1] {
        let packet = ffone_protocol::wire_0104::PcStateChange0104 { pc_id: 11, state };
        push_frame(&mut app, EPOCH_ONE, frame(0x3100_007a, packet.encode()));
        app.update();
        assert_eq!(
            app.world().resource::<RemotePcRegistry0104>().get(11),
            Some(entity)
        );
        let appearance = &app
            .world()
            .get::<NetworkPcAppearance0104>(entity)
            .unwrap()
            .0;
        assert_eq!(appearance.pc_state, state);
        assert_eq!(appearance.equipment[8].item_id, 107);
    }
}

#[test]
fn reconnect_and_disconnect_fully_clear_epoch_tagged_state() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(
            EPOCH_ONE,
            InitialAroundPacket0104::Players(vec![pc_appearance(11, [0, 0, 0])]),
        );
    app.update();
    let old_entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();

    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .begin_session(EPOCH_TWO, 20);
    let stale = frame(
        P_FE2CL_PC_NEW,
        PcNew0104 {
            appearance: pc_appearance(12, [0, 0, 0]),
        }
        .encode(),
    );
    push_frame(&mut app, EPOCH_ONE, stale.clone());
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(
            EPOCH_TWO,
            InitialAroundPacket0104::Npcs(vec![npc_appearance(31, 5001, [0, 0, 0])]),
        );
    app.update();

    assert!(app.world().get_entity(old_entity).is_err());
    assert!(app.world().resource::<RemotePcRegistry0104>().is_empty());
    assert_eq!(app.world().resource::<NetworkNpcRegistry0104>().len(), 1);
    assert_eq!(
        app.world()
            .resource::<IgnoredLifecycleFrames0104>()
            .frames
            .last()
            .unwrap(),
        &IgnoredLifecycleFrame0104 {
            epoch: EPOCH_ONE,
            frame: stale,
            reason: IgnoredLifecycleFrameReason0104::StaleSession {
                active_epoch: Some(EPOCH_TWO),
            },
        }
    );

    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .disconnect(EPOCH_TWO);
    app.update();
    assert_eq!(
        *app.world().resource::<ActiveNetworkEntitySession0104>(),
        ActiveNetworkEntitySession0104::default()
    );
    assert!(app.world().resource::<RemotePcRegistry0104>().is_empty());
    assert!(app.world().resource::<NetworkNpcRegistry0104>().is_empty());
    assert!(
        app.world()
            .resource::<NetworkTransportationRegistry0104>()
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<NetworkShinyRegistry0104>()
            .is_empty()
    );
    let mut query = app.world_mut().query::<&NetworkSessionEntity0104>();
    assert_eq!(query.iter(app.world()).count(), 0);
}
