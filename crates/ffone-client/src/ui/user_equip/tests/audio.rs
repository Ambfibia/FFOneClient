use super::*;

#[test]
fn chest_right_click_and_open_button_emit_clean_inventory_audio() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_resource(ButtonInput::<MouseButton>::default())
        .add_plugins(UserEquipUiPlugin);
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(1_264, 681),
            ..default()
        },
        PrimaryWindow,
    ));
    app.insert_resource(UserEquipItemModeProjection::from_authoritative(
        &runtime_with(&[], &[(1, item(9, 40, 1, 0))]),
        &AllCatalog,
    ));
    {
        let mut state = app.world_mut().resource_mut::<UserEquipUiState>();
        state.open_item_mode();
        state.tick(USER_EQUIP_OPEN_SECONDS);
    }
    app.update();

    let chest = UserEquipSlotEndpoint::Inventory { slot_index: 1 };
    let chest_entity = {
        let world = app.world_mut();
        let mut slots = world.query::<(Entity, &UserEquipSlotControl)>();
        slots
            .iter(world)
            .find_map(|(entity, control)| (control.0 == chest).then_some(entity))
            .unwrap()
    };
    app.world_mut()
        .entity_mut(chest_entity)
        .insert(Interaction::Hovered);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    {
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.clear_just_pressed(MouseButton::Left);
        mouse.release(MouseButton::Left);
    }
    app.update();

    assert!(app.world().resource::<UserEquipUiOutbox>().is_empty());
    assert_eq!(
        app.world_mut()
            .resource_mut::<UserEquipUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![
            UserEquipUiAudioCue::ButtonSound,
            UserEquipUiAudioCue::OpenScreen,
        ]
    );
    app.world_mut()
        .resource_mut::<UserEquipItemPopupState>()
        .close();
    app.world_mut()
        .resource_mut::<UserEquipModalState>()
        .item_popup_active = false;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();

    assert_eq!(
        app.world_mut()
            .resource_mut::<UserEquipUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![UserEquipUiAction::OpenInventoryChest { slot_index: 1 }]
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<UserEquipUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![UserEquipUiAudioCue::ButtonSound]
    );

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<UserEquipItemPopupState>()
        .open(chest);
    app.world_mut()
        .resource_mut::<UserEquipModalState>()
        .item_popup_active = true;
    let open_button = {
        let world = app.world_mut();
        let mut buttons = world.query::<(Entity, &UserEquipPopupButtonControl)>();
        buttons
            .iter(world)
            .find_map(|(entity, control)| (control.0 == 0).then_some(entity))
            .unwrap()
    };
    app.world_mut()
        .entity_mut(open_button)
        .insert(Interaction::Pressed);
    app.update();

    assert_eq!(
        app.world_mut()
            .resource_mut::<UserEquipUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![UserEquipUiAction::OpenInventoryChest { slot_index: 1 }]
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<UserEquipUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![UserEquipUiAudioCue::ButtonSound]
    );
}
