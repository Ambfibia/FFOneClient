use super::*;

#[test]
fn shared_redeem_modal_preserves_gates_audio_validation_and_freechat_boundary() {
    let mut model = EnchantModeModel0104::default();
    model.open(1_000, false);
    model.clear_intents();
    model.open_redeem_code().unwrap();
    assert!(model.external_gates().inventory_popup_open);
    assert!(!model.input_capabilities().base_gui_enabled);
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Audio(EnchantAudioIntent0104::Button)
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Popup(EnchantPopupIntent0104::RedeemCode {
            window_id: 99,
            max_code_chars: 32,
        })
    )));

    model.clear_intents();
    assert_eq!(model.submit_redeem_code("ab").unwrap(), None);
    assert!(model.external_gates().inventory_popup_open);
    assert_eq!(
        model.pop_intent(),
        Some(EnchantIntent0104::Audio(EnchantAudioIntent0104::YesButton))
    );
    model.cancel_redeem_code().unwrap();
    assert!(!model.external_gates().inventory_popup_open);
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Audio(EnchantAudioIntent0104::NoButton)
    )));

    model.clear_intents();
    model.open_redeem_code().unwrap();
    model.clear_intents();
    let wire = model.submit_redeem_code("DEX123").unwrap().unwrap();
    assert_eq!(wire.request.message.to_string_lossy(), "/redeem DEX123 ");
    assert!(!model.external_gates().inventory_popup_open);
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::RedeemWire(value) if value == &wire
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Audio(EnchantAudioIntent0104::ActionSuccess)
    )));

    model.clear_intents();
    model.open_redeem_code().unwrap();
    model.clear_intents();
    assert_eq!(
        model.submit_redeem_code("BAD CODE"),
        Err(EnchantModelError0104::Redeem(
            EnchantRedeemError0104::ContainsSpace
        ))
    );
    assert!(!model.external_gates().inventory_popup_open);
    assert!(model.external_gates().system_popup_open);
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Popup(EnchantPopupIntent0104::RedeemCodeSpaceError {
            message: ENCHANT_REDEEM_SPACE_ERROR_TEXT_0104,
        })
    )));
}

#[test]
fn open_close_and_my_stuff_emit_exact_audio_camera_cursor_and_event_boundaries() {
    let mut model = EnchantModeModel0104::default();
    model.open(1_000, true);
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Audio(EnchantAudioIntent0104::PlayUiMode)
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::InitializeInventoryMode {
            gui_mode: ENCHANT_INVENTORY_GUI_MODE_0104,
            active_inventory_tab: 0,
            show_inventory_panel: true,
            show_equipment_panel: true,
            inventory_event_dispatch: 8,
        })
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::LoadPanelBackdrop {
            logical_path: "TutorialAssets/panelback.png"
        })
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Camera(EnchantCameraIntent0104::BindPrimaryNpcAvatar {
            camera_path_id: ENCHANT_PRIMARY_CAMERA_PATH_ID,
            controller_path_id: ENCHANT_PRIMARY_CAMERA_CONTROLLER_PATH_ID,
            distance_millimetres: 1_300,
            height_millimetres: 550,
            euler_degrees: [0, -20, 0],
            target: EnchantCameraTarget0104::Neck,
        })
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Camera(EnchantCameraIntent0104::BindWaitingNpcAvatar {
            camera_path_id: ENCHANT_WAITING_CAMERA_PATH_ID,
            controller_path_id: ENCHANT_WAITING_CAMERA_CONTROLLER_PATH_ID,
            distance_millimetres: 1_300,
            height_millimetres: 550,
            euler_degrees: [0, -20, 0],
            target: EnchantCameraTarget0104::Neck,
        })
    )));
    assert_eq!(
        model.close(false),
        Err(EnchantModelError0104::CloseRejectedByInventory)
    );
    model.close(true).unwrap();
    assert!(matches!(model.phase(), EnchantPhase0104::Closed));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::SetCursorLock(true))
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::RestoreGameplayInventory {
            inventory_event_dispatch: 10,
        })
    )));
    assert!(model.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Audio(EnchantAudioIntent0104::StopUiMode)
    )));

    let mut success = awaiting_weapon_model();
    success
        .receive_success(EnchantSuccess0104 {
            enchant_item_slot: 7,
            enchant_item: item(0, 501, 2),
            weapon_material_item_slot: 8,
            weapon_material_item: item(7, 101, 28),
            defence_material_item_slot: -1,
            defence_material_item: empty_enchant_item_0104(),
            cash_item_slot_1: -1,
            cash_item_slot_2: -1,
            taros: 9_800,
            success_flag: 1,
        })
        .unwrap();
    success.clear_intents();
    success.go_to_my_stuff().unwrap();
    assert!(matches!(success.phase(), EnchantPhase0104::Closed));
    assert!(success.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::EnterGameMode(6))
    )));
    assert!(success.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::DispatchLegacyEvent {
            channel: 2,
            element_func: 3,
            argument: Some(0),
            dispatch: Some(2),
        })
    )));
    assert!(success.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::CheckFirstUse(3))
    )));
    assert!(success.intents().any(|intent| matches!(
        intent,
        EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::DispatchLegacyEvent {
            channel: 11,
            element_func: 18,
            argument: None,
            dispatch: None,
        })
    )));
}
