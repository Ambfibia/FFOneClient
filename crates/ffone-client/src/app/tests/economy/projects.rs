use super::*;

#[test]
fn user_equip_session_reset_hides_ui_clears_scroll_projection_and_outbox() {
    let content = runtime_test_mission_content();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let item = ffone_protocol::ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 3,
        time_limit: 0,
    };
    let offset =
        ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + 49 * ffone_protocol::ItemBase0104::SIZE;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&item.item_type.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&item.item_id.to_le_bytes());
    load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&item.option.to_le_bytes());
    let inventory = InventoryRuntime0104::from_pc_load(77, &load);
    let mut projection = UserEquipItemModeProjection::from_authoritative(&inventory, &content);
    let mut state = UserEquipUiState::default();
    state.open_item_mode();
    state.tick(1.0);
    state.set_scroll_y(190.0);
    let mut modal = UserEquipModalState {
        system_popup_active: true,
        ..default()
    };
    let mut outbox = UserEquipUiOutbox::default();
    outbox.push(UserEquipUiAction::RequestClose {
        source: UserEquipCloseSource::Escape,
    });

    reset_user_equip_shell(&mut state, &mut modal, &mut projection, &mut outbox);

    assert!(!state.is_active());
    assert_eq!(state.scroll_y(), 0.0);
    assert_eq!(modal, UserEquipModalState::default());
    assert_eq!(projection, UserEquipItemModeProjection::default());
    assert!(outbox.is_empty());
}
