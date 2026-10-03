use super::*;
use std::time::Duration;

#[test]
fn path_npc_move_stream_advances_visible_root_and_holds_during_pause() {
    let mut app = test_app();
    app.insert_resource(Time::<()>::default());
    app.add_systems(
        Update,
        crate::network_world_runtime::advance_network_npc_motion_0104
            .after(consume_network_entity_lifecycle_0104),
    );
    begin(&mut app, EPOCH_ONE);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_ENTER,
            NpcEnter0104 {
                appearance: npc_appearance(21, 3285, [0, 0, 0]),
            }
            .encode(),
        ),
    );
    app.update();
    let entity = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();

    let send_move = |app: &mut App, x: i32| {
        push_frame(
            app,
            EPOCH_ONE,
            frame(
                P_FE2CL_NPC_MOVE,
                NpcMove0104 {
                    npc_id: 21,
                    destination: [x, 0, 0],
                    speed: 400,
                    move_style: 0,
                }
                .encode(),
            ),
        );
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(125));
        app.update();
    };
    send_move(&mut app, 80);
    let first = app.world().get::<Transform>(entity).unwrap().translation;
    assert!(
        (first.x + 0.5).abs() < 0.001,
        "first server step: {first:?}"
    );
    send_move(&mut app, 80);
    let arrived = app.world().get::<Transform>(entity).unwrap().translation;
    assert!((arrived.x + 0.8).abs() < 0.001, "arrival: {arrived:?}");
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(125));
    app.update();
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation,
        arrived
    );
    send_move(&mut app, 0);
    let returning = app.world().get::<Transform>(entity).unwrap().translation;
    assert!(
        (returning.x + 0.3).abs() < 0.001,
        "return leg: {returning:?}"
    );
}

#[test]
fn npc_enter_new_and_move_require_appearance_and_publish_typed_motion() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let early_move = NpcMove0104 {
        npc_id: 21,
        destination: [100, 200, 300],
        speed: 150,
        move_style: 1,
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_MOVE, early_move.encode()),
    );
    app.update();
    assert!(app.world().resource::<NetworkNpcRegistry0104>().is_empty());

    let appearance = npc_appearance(21, 4001, [0, 0, 0]);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_ENTER, NpcEnter0104 { appearance }.encode()),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_MOVE, early_move.encode()),
    );
    app.update();
    let entity = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();
    assert_eq!(
        *app.world().get::<NetworkNpcMotion0104>(entity).unwrap(),
        NetworkNpcMotion0104 {
            destination: Vec3::new(-1.0, 3.0, 2.0),
            speed: 1.5,
            move_style: 1,
        }
    );

    let replacement = npc_appearance(21, 4999, [400, 500, 600]);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_NEW,
            NpcNew0104 {
                appearance: replacement,
            }
            .encode(),
        ),
    );
    app.update();
    assert_eq!(
        app.world().get::<NetworkNpc0104>(entity).unwrap().npc_type,
        4999
    );
    assert_eq!(
        app.world()
            .get::<PendingNpcVisual0104>(entity)
            .unwrap()
            .npc_type,
        4999
    );
}
