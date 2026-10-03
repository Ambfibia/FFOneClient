use super::*;

#[test]
fn enabled_create_and_delete_buttons_emit_protocol_ready_actions() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_plugins(NativeCharacterSelectionUiPlugin);
    app.update();

    {
        let mut model = app.world_mut().resource_mut::<CharacterSelectionUiModel>();
        model.visible = true;
        model.slots = [
            occupied(77, CharacterLocationBackground::Future),
            CharacterSlotUi::SubscriptionLocked,
            CharacterSlotUi::Empty,
            CharacterSlotUi::Empty,
        ];
        model.selected_slot = Some(0);
        model.create = CharacterSelectionCapability::Enabled;
        model.delete = CharacterSelectionCapability::Enabled;
    }
    let (create, delete, confirm) = {
        let world = app.world_mut();
        let mut create_query = world.query_filtered::<Entity, With<SelectionCreate>>();
        let create = create_query.single(world).unwrap();
        let mut delete_query = world.query_filtered::<Entity, With<SelectionDelete>>();
        let delete = delete_query.single(world).unwrap();
        let mut confirm_query = world.query_filtered::<Entity, With<SelectionDeleteConfirm>>();
        let confirm = confirm_query.single(world).unwrap();
        (create, delete, confirm)
    };
    app.world_mut()
        .entity_mut(create)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<CharacterSelectionUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![CharacterSelectionUiAction::CreateCharacter { slot: 3 }]
    );

    app.world_mut().entity_mut(create).insert(Interaction::None);
    app.world_mut()
        .entity_mut(delete)
        .insert(Interaction::Pressed);
    app.update();
    assert!(
        app.world_mut()
            .resource_mut::<CharacterSelectionUiOutbox>()
            .drain()
            .next()
            .is_none(),
        "opening the source confirmation dialog must not delete immediately"
    );
    assert_eq!(
        app.world()
            .resource::<CharacterSelectionUiModel>()
            .delete_confirmation_pc_uid,
        Some(77)
    );
    app.world_mut()
        .resource_mut::<CharacterSelectionUiModel>()
        .delete_name_input = "Player".to_owned();
    app.world_mut()
        .entity_mut(confirm)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<CharacterSelectionUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![CharacterSelectionUiAction::DeleteSelected { pc_uid: 77 }]
    );
    {
        let model = app.world().resource::<CharacterSelectionUiModel>();
        assert_eq!(model.delete_confirmation_pc_uid, Some(77));
        assert_eq!(model.delete_name_input, "Player");
        assert!(model.delete_request_pending);
    }

    app.world_mut()
        .entity_mut(confirm)
        .insert(Interaction::None);
    app.update();
    app.world_mut()
        .entity_mut(confirm)
        .insert(Interaction::Pressed);
    app.update();
    assert!(
        app.world_mut()
            .resource_mut::<CharacterSelectionUiOutbox>()
            .drain()
            .next()
            .is_none(),
        "a pending authoritative delete must not emit a duplicate packet"
    );

    app.world_mut()
        .resource_mut::<CharacterSelectionUiModel>()
        .reject_delete_request();
    app.world_mut()
        .entity_mut(confirm)
        .insert(Interaction::None);
    app.update();
    app.world_mut()
        .entity_mut(confirm)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<CharacterSelectionUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![CharacterSelectionUiAction::DeleteSelected { pc_uid: 77 }],
        "an authoritative rejection must preserve the dialog and allow a retry"
    );

    app.world_mut()
        .resource_mut::<CharacterSelectionUiModel>()
        .complete_delete_request(77);
    let model = app.world().resource::<CharacterSelectionUiModel>();
    assert_eq!(model.delete_confirmation_pc_uid, None);
    assert!(model.delete_name_input.is_empty());
    assert!(!model.delete_request_pending);
}
