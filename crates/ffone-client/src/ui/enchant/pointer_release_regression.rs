use super::*;
#[test]
fn held_inventory_press_releases_once_into_attachment_and_respects_modal() {
    for cancellation in [0, 1, 2] {
        let mut app = App::new();
        let mut model = EnchantModeModel0104::default();
        model.open(10000, true);
        let mut projection = EnchantModeProjection0104::default();
        projection.visible = true;
        projection.capabilities = model.input_capabilities();
        let mut animation = EnchantInventoryUiState0104::default();
        animation.tick(2.0, true);
        app.insert_resource(projection)
            .insert_resource(animation)
            .init_resource::<EnchantUiOutbox0104>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Update, collect_enchant_ui_input_0104);
        let source = app
            .world_mut()
            .spawn((
                EnchantInteractiveControl0104::InventorySlot(0),
                Interaction::Pressed,
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                EnchantInteractiveControl0104::Attachment(EnchantAttachmentSlot0104::Target),
                Interaction::None,
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<EnchantUiOutbox0104>()
                .pop_front(),
            Some(EnchantUiCommand0104::BeginInventoryDrag(0))
        );
        *app.world_mut().get_mut::<Interaction>(source).unwrap() = Interaction::None;
        *app.world_mut().get_mut::<Interaction>(target).unwrap() = Interaction::Hovered;
        if cancellation == 2 {
            // The focus-loss owner cancels input without emitting mouse-up.
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .reset_all();
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        if cancellation == 1 {
            app.world_mut()
                .resource_mut::<EnchantModeProjection0104>()
                .capabilities
                .base_gui_enabled = false;
        }
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<EnchantUiOutbox0104>()
                .pop_front(),
            (cancellation == 0).then_some(EnchantUiCommand0104::DropOnAttachment(
                EnchantAttachmentSlot0104::Target
            ))
        );
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<EnchantUiOutbox0104>()
                .pop_front(),
            None
        );
    }
}
