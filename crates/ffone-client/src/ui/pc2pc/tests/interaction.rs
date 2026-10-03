use super::*;

#[test]
fn clean_chat_history_retains_at_most_forty_one_lines_and_autoscrolls() {
    let mut model = model_with(500, &[]);
    let capabilities = Pc2pcBackendCapabilities {
        free_chat_backend: true,
        ..default()
    };
    for index in 0..44 {
        model
            .apply_server_outcome(
                Pc2pcServerOutcome0104::ChatMessage {
                    envelope: remote_envelope(),
                    text: format!("line {index}"),
                    emote_code: 1,
                },
                capabilities,
                &Catalog,
                &AllowEquip,
            )
            .unwrap();
    }
    assert_eq!(model.chat.len(), PC2PC_CHAT_MAX_STORED_LINES);
    assert_eq!(model.chat.front().unwrap().text, "line 3");
    assert_eq!(
        model.state.chat_scroll_y,
        PC2PC_CHAT_MAX_STORED_LINES as f32 * PC2PC_CHAT_LINE_HEIGHT
    );
}

fn gesture_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<bevy::input::keyboard::KeyboardInput>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<super::super::gestures::TarosEditor>()
        .init_resource::<Pc2pcModalState>()
        .insert_resource(Pc2pcBackendCapabilities { numeric_popup_backend: true, ..default() })
        .insert_resource(model_with(500, &[(0, item(7, 90, 12))]))
        .add_systems(Update, super::super::gestures::interact);
    app
}

#[test]
fn inventory_click_queues_typed_offer_without_mutating_owned_items() {
    let mut app = gesture_app();
    app.world_mut().spawn((Pc2pcUiElement::InventorySlotFrame(0), Interaction::Pressed));
    app.update();
    let model = app.world_mut().resource_mut::<Pc2pcUiModel0104>();
    assert_eq!(model.snapshot.as_ref().unwrap().base_inventory()[0], item(7, 90, 12));
    assert_eq!(model.outbox.len(), 1);
    assert!(matches!(model.state.pending, Some(Pc2pcPendingRequest0104::RegisterItem(_))));
    app.update();
    assert_eq!(app.world().resource::<Pc2pcUiModel0104>().outbox.len(), 1);
}

#[test]
fn submit_close_and_popup_gate_route_the_real_controls() {
    for (element, expected_packet) in [(Pc2pcUiElement::MainButton, 0x13000027), (Pc2pcUiElement::Close, 0x13000028)] {
        let mut app = gesture_app();
        app.world_mut().spawn((element, Interaction::Pressed));
        app.update();
        let intent = app.world_mut().resource_mut::<Pc2pcUiModel0104>().outbox.pop_front().unwrap();
        assert_eq!(intent.packet_id(), expected_packet);
    }
    let mut app = gesture_app();
    app.world_mut().resource_mut::<Pc2pcModalState>().system_popup_active = true;
    app.world_mut().spawn((Pc2pcUiElement::MainButton, Interaction::Pressed));
    app.update();
    assert!(app.world().resource::<Pc2pcUiModel0104>().outbox.is_empty());
}

#[test]
fn escape_in_taros_editor_does_not_also_cancel_the_session() {
    let mut app = gesture_app();
    app.world_mut().spawn((Pc2pcUiElement::AddTaros, Interaction::Pressed));
    app.update();
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
    app.update();
    let model = app.world().resource::<Pc2pcUiModel0104>();
    assert!(model.outbox.is_empty());
    assert_eq!(model.state.phase, Pc2pcLifecyclePhase::Trading);
}
