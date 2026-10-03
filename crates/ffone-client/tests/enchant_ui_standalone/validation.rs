use super::*;

#[test]
fn modal_gates_target_cash_error_and_unsupported_flag_keep_legacy_blank_or_lock_states() {
    let mut model = EnchantModeModel0104::default();
    model.open(10_000, false);
    let mut cash_target = selectable(7, 0, 501, 1);
    cash_target.presentation.cashable = 1;
    model
        .attach(EnchantAttachmentSlot0104::Target, cash_target)
        .unwrap();
    assert!(model.target_error());
    assert!(!model.input_capabilities().preview_enabled);
    let projection = EnchantModeProjection0104::from_model(
        &model,
        EnchantSupportPresentation0104::default(),
        std::array::from_fn(|_| EnchantInventorySlotProjection0104::default()),
        std::array::from_fn(|_| EnchantInventorySlotProjection0104::default()),
        468,
        392,
    );
    assert!(!projection.dead_cash_warning_visible());
    assert_eq!(projection.weapon_battery, 468);
    assert_eq!(projection.nano_battery, 392);

    model.set_external_gates(EnchantExternalGates0104 {
        system_popup_open: true,
        ..EnchantExternalGates0104::default()
    });
    assert!(!model.input_capabilities().base_gui_enabled);

    let mut awaiting = awaiting_weapon_model();
    let disposition = awaiting
        .receive_success(EnchantSuccess0104 {
            success_flag: 9,
            ..EnchantSuccess0104::default()
        })
        .unwrap();
    assert_eq!(disposition, EnchantReplyDisposition0104::IgnoredLegacyFlag);
    assert!(awaiting.send_locked());
    assert!(matches!(
        awaiting.phase(),
        EnchantPhase0104::AwaitingReply { .. }
    ));
}
