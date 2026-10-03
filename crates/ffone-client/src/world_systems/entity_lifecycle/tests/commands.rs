use super::*;

#[test]
fn repeated_appearance_upserts_preserve_entity_identity_and_refresh_visual_request() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let first = pc_appearance(11, [100, 200, 300]);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_NEW, PcNew0104 { appearance: first }.encode()),
    );
    app.update();
    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();

    let mut second = pc_appearance(11, [400, 500, 600]);
    second.hp = 777;
    second.equipment[0].item_id = 4321;
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_NEW, PcNew0104 { appearance: second }.encode()),
    );
    app.update();

    assert_eq!(
        app.world().resource::<RemotePcRegistry0104>().get(11),
        Some(entity)
    );
    assert_eq!(
        app.world()
            .get::<NetworkPcAppearance0104>(entity)
            .unwrap()
            .0
            .hp,
        777
    );
    assert_eq!(
        app.world()
            .get::<PendingPcVisual0104>(entity)
            .unwrap()
            .equipment[0]
            .item_id,
        4321
    );
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation,
        Vec3::new(-4.0, 6.0, 5.0)
    );
}
