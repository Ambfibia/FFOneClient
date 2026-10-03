use crate::app::session_lifecycle::*;

#[test]
fn only_connecting_boundary_resets_race_session() {
    assert!(NetworkSessionReset::new(NetworkSessionBoundary::Connecting).resets_race());
    assert!(!NetworkSessionReset::new(NetworkSessionBoundary::Error).resets_race());
    assert!(!NetworkSessionReset::new(NetworkSessionBoundary::Disconnected).resets_race());
}

#[test]
fn deferred_boundary_reset_preserves_same_poll_login_entitlement() {
    let mut app = App::new();
    app.add_message::<NetworkSessionReset>()
        .init_resource::<GuideUiModel>()
        .init_resource::<GuideUiOutbox>()
        .init_resource::<GuideUiAudioOutbox>()
        .insert_resource(GuideRuntime::new(GuideServerProfile::OpenFusion0104))
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<VendorUiState>()
        .init_resource::<VendorModalState>()
        .init_resource::<VendorModeProjection0104>()
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorProductionRuntime0104>()
        .init_resource::<VendorSystemMessageRuntime>()
        .add_systems(Update, reset_guide_vendor_on_network_boundary);
    // The network FIFO can be drained in one frame: Connecting, then
    // LoginMetadata. Ingress clears the old flag before setting the new one.
    app.world_mut()
        .resource_mut::<GuideProductionRuntime>()
        .payment_flag = Some(0);
    app.world_mut()
        .resource_mut::<GuideProductionRuntime>()
        .reset_session();
    app.world_mut()
        .write_message(NetworkSessionReset::new(NetworkSessionBoundary::Connecting));
    app.world_mut()
        .resource_mut::<GuideProductionRuntime>()
        .payment_flag = Some(1);
    app.update();
    let flag = app
        .world()
        .resource::<GuideProductionRuntime>()
        .payment_flag;
    assert_eq!(flag, Some(1));
    assert_eq!(
        guide_service_entry(23, flag).unwrap().service,
        NpcServiceKind::PastWarp
    );
    // Returning to the roster closes the shard, not the authenticated login.
    app.world_mut()
        .resource_mut::<GuideProductionRuntime>()
        .reset_world();
    app.world_mut().write_message(NetworkSessionReset::new(
        NetworkSessionBoundary::Disconnected,
    ));
    app.update();
    assert_eq!(
        app.world()
            .resource::<GuideProductionRuntime>()
            .payment_flag,
        Some(1)
    );
    // A later actual disconnect without new metadata must not inherit that flag.
    app.world_mut()
        .resource_mut::<GuideProductionRuntime>()
        .reset_session();
    app.world_mut().write_message(NetworkSessionReset::new(
        NetworkSessionBoundary::Disconnected,
    ));
    app.update();
    assert!(
        guide_service_entry(
            23,
            app.world()
                .resource::<GuideProductionRuntime>()
                .payment_flag
        )
        .is_none()
    );
}
