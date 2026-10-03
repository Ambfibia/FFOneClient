use super::*;

#[test]
fn held_repeat_button_rotates_once_per_ui_frame_in_both_book_tabs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<UserEquipUiState>()
        .init_resource::<UserEquipModalState>()
        .init_resource::<UserEquipAvatarPreviewPresentation>()
        .add_systems(Update, collect_user_equip_avatar_rotation);
    let turn = app
        .world_mut()
        .spawn((
            Interaction::Pressed,
            UserEquipAvatarTurnControl(UserEquipAvatarTurnDirection::LeftPositioned),
        ))
        .id();
    {
        let mut state = app.world_mut().resource_mut::<UserEquipUiState>();
        state.open_item_mode();
        state.tick(USER_EQUIP_OPEN_SECONDS);
    }

    app.update();
    app.update();
    assert_eq!(
        app.world()
            .resource::<UserEquipAvatarPreviewPresentation>()
            .yaw_degrees(),
        2.0,
        "held RepeatButton semantics must not require Changed<Interaction>"
    );

    app.world_mut()
        .resource_mut::<UserEquipUiState>()
        .select_nano_tab();
    app.update();
    assert_eq!(
        app.world()
            .resource::<UserEquipAvatarPreviewPresentation>()
            .yaw_degrees(),
        3.0,
        "normal UserEquip Nano-book mode retains the avatar controls"
    );

    app.world_mut()
        .resource_mut::<UserEquipModalState>()
        .system_popup_active = true;
    app.update();
    assert_eq!(
        app.world()
            .resource::<UserEquipAvatarPreviewPresentation>()
            .yaw_degrees(),
        3.0
    );
    app.world_mut().entity_mut(turn).insert(Interaction::None);
}

#[test]
fn equipment_projection_uses_clean_visual_to_wire_order() {
    let equipment: Vec<_> = (0..EQUIPMENT_SLOT_COUNT_0104)
        .map(|wire| (wire, item(0, 100 + wire as i16, 0, 0)))
        .collect();
    let runtime = runtime_with(&equipment, &[]);
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);

    assert_eq!(
        USER_EQUIP_EQUIPMENT_STRIP_ORDER.map(|slot| slot.wire_slot_index),
        [4, 5, 6, 1, 2, 3, 0, 7, 8]
    );
    assert_eq!(
        USER_EQUIP_EQUIPMENT_STRIP_ORDER.map(|slot| slot.legacy_panel_item_type),
        [4, 5, 6, 1, 2, 3, 0, 7, 10]
    );
    assert_eq!(
        projection
            .equipment
            .each_ref()
            .map(|slot| slot.item.item.item_id),
        [104, 105, 106, 101, 102, 103, 100, 107, 108]
    );
    assert_eq!(
        projection
            .equipment
            .each_ref()
            .map(|slot| (slot.spec.label_key, slot.spec.label_ordinal)),
        [
            ("HEAD", None),
            ("FACE", None),
            ("BACK", None),
            ("CHEST", None),
            ("LEGS", None),
            ("FEET", None),
            ("WEAPON", Some(1)),
            ("WEAPON", Some(2)),
            ("VEHICLE", None),
        ]
    );
}
