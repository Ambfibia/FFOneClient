use super::*;
#[test] fn close_then_reopen_clears_item_nano_drag_and_popup_modal_ownership(){
    let mut app=App::new();app.init_resource::<UserEquipUiState>().init_resource::<UserEquipItemPopupState>()
        .init_resource::<UserEquipNanoViewerState>().init_resource::<UserEquipDragState>().init_resource::<UserEquipModalState>()
        .add_systems(PostUpdate,clear_closed_user_equip_selection);
    app.world_mut().resource_mut::<UserEquipUiState>().open_item_mode();
    let slot=UserEquipSlotEndpoint::Inventory {slot_index:0};
    app.world_mut().resource_mut::<UserEquipItemPopupState>().open(slot);
    app.world_mut().resource_mut::<UserEquipDragState>().begin(slot);
    app.world_mut().resource_mut::<UserEquipNanoViewerState>().open(2);
    app.world_mut().resource_mut::<UserEquipModalState>().item_popup_active=true;
    app.update();assert!(app.world().resource::<UserEquipItemPopupState>().selected().is_some());
    app.world_mut().resource_mut::<UserEquipUiState>().close();app.update();
    app.world_mut().resource_mut::<UserEquipUiState>().open_item_mode();app.update();
    assert_eq!(app.world().resource::<UserEquipItemPopupState>().selected(),None);
    assert_eq!(app.world().resource::<UserEquipNanoViewerState>().selected_visual_index(),None);
    assert_eq!(app.world().resource::<UserEquipDragState>().source(),None);
    assert!(!app.world().resource::<UserEquipModalState>().item_popup_active);
}
