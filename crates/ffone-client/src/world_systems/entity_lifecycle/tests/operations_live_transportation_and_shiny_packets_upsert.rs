use super::*;

#[test]
fn live_transportation_and_shiny_packets_upsert_move_and_delete_same_entities() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let transportation = TransportationAppearance0104 {
        transportation_kind: 3,
        id: 71,
        transportation_type: 12,
        position: [100, 200, 300],
    };
    let shiny = ShinyAppearance0104 {
        shiny_id: 81,
        shiny_type: 3,
        map_number: 0,
        position: [400, 500, 600],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_TRANSPORTATION_ENTER, transportation.encode()),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_SHINY_NEW, shiny.encode()),
    );
    let movement = TransportationMove0104 {
        transportation_kind: 3,
        id: 71,
        destination: [700, 800, 900],
        speed: 250,
        move_style: 1,
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_TRANSPORTATION_MOVE, movement.encode()),
    );
    app.update();

    let transportation_entity = app
        .world()
        .resource::<NetworkTransportationRegistry0104>()
        .get(3, 71)
        .unwrap();
    assert_eq!(
        *app.world()
            .get::<NetworkTransportationMotion0104>(transportation_entity)
            .unwrap(),
        NetworkTransportationMotion0104 {
            destination: Vec3::new(-7.0, 9.0, 8.0),
            speed: 2.5,
            move_style: 1,
        }
    );
    let shiny_entity = app
        .world()
        .resource::<NetworkShinyRegistry0104>()
        .get(81)
        .unwrap();

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_TRANSPORTATION_EXIT,
            TransportationExit0104 {
                transportation_kind: 3,
                id: 71,
            }
            .encode(),
        ),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_AROUND_DEL_SHINY,
            AroundDelShiny0104 {
                shiny_ids: vec![81],
            }
            .encode()
            .unwrap(),
        ),
    );
    app.update();

    assert!(app.world().get_entity(transportation_entity).is_err());
    assert!(app.world().get_entity(shiny_entity).is_err());
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
    let stats = app.world().resource::<NetworkEntityLifecycleStats0104>();
    assert_eq!(stats.transportation_despawns, 1);
    assert_eq!(stats.shiny_despawns, 1);
}

#[test]
fn non_finite_pc_move_is_malformed_not_an_orphan_spawn_signal() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let mut movement = pc_move(11, [0, 0, 0]);
    movement.movement.velocity[1] = f32::NAN;
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(packet::P_FE2CL_PC_MOVE, movement.encode()),
    );
    app.update();

    assert_eq!(
        app.world()
            .resource::<MalformedLifecycleFrames0104>()
            .frames[0]
            .error,
        EntityLifecycleDecodeError0104::NonFinitePcMoveVelocity
    );
    assert!(app.world().resource::<RemotePcRegistry0104>().is_empty());
    assert!(
        app.world()
            .resource::<IgnoredLifecycleFrames0104>()
            .frames
            .is_empty()
    );
}
