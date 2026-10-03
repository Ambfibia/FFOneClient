use super::*;

pub(super) fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn transportation_and_shiny_bootstraps_publish_typed_epoch_state() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let transportation = TransportationAppearance0104 {
        transportation_kind: 2,
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
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(
            EPOCH_ONE,
            InitialAroundPacket0104::Transportation(vec![transportation]),
        );
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(EPOCH_ONE, InitialAroundPacket0104::Shinies(vec![shiny]));
    app.update();

    let transportation_entity = app
        .world()
        .resource::<NetworkTransportationRegistry0104>()
        .get(2, 71)
        .unwrap();
    assert_eq!(
        app.world()
            .get::<NetworkTransportationAppearance0104>(transportation_entity),
        Some(&NetworkTransportationAppearance0104(transportation))
    );
    assert_eq!(
        app.world()
            .get::<Transform>(transportation_entity)
            .unwrap()
            .translation,
        Vec3::new(-1.0, 3.0, 2.0)
    );
    let shiny_entity = app
        .world()
        .resource::<NetworkShinyRegistry0104>()
        .get(81)
        .unwrap();
    assert_eq!(
        app.world().get::<NetworkShinyAppearance0104>(shiny_entity),
        Some(&NetworkShinyAppearance0104(shiny))
    );
    assert_eq!(
        app.world()
            .get::<Transform>(shiny_entity)
            .unwrap()
            .translation,
        Vec3::new(-4.0, 6.0, 5.0)
    );
    assert_eq!(
        app.world()
            .get::<NetworkSessionEntity0104>(shiny_entity)
            .unwrap()
            .epoch,
        EPOCH_ONE
    );
    let stats = app.world().resource::<NetworkEntityLifecycleStats0104>();
    assert_eq!(stats.transportation_upserts, 1);
    assert_eq!(stats.shiny_upserts, 1);
    assert!(
        app.world()
            .resource::<LifecycleBootstrapDiagnostics0104>()
            .passthrough
            .is_empty()
    );
}
