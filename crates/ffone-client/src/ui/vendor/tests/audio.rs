use super::*;

#[test]
fn vendor_tabs_and_help_emit_their_clean_audio_routes() {
    let mut app = App::new();
    app.init_resource::<VendorModalState>()
        .init_resource::<VendorCloseGate0104>()
        .init_resource::<VendorModeProjection0104>()
        .init_resource::<VendorUiState>()
        .init_resource::<VendorHoverState>()
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorUiAudioOutbox0104>()
        .add_systems(Update, collect_vendor_ui_input);
    app.world_mut().resource_mut::<VendorUiState>().phase = VendorLifecyclePhase::Visible;
    app.world_mut()
        .spawn((VendorInteractiveControl::BuybackTab, Interaction::Pressed));

    app.update();

    assert_eq!(
        app.world().resource::<VendorUiState>().tab,
        VendorTab0104::Buyback
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<VendorUiAudioOutbox0104>()
            .pop_front(),
        Some(VendorUiAudioCue0104::TabClick01)
    );
    assert!(app.world().resource::<VendorUiAudioOutbox0104>().is_empty());

    app.world_mut()
        .spawn((VendorInteractiveControl::Help, Interaction::Pressed));
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<VendorUiAudioOutbox0104>()
            .pop_front(),
        Some(VendorUiAudioCue0104::ButtonSound)
    );
}
