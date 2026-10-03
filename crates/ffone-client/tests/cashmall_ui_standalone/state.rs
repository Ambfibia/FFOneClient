use super::*;

#[cfg(test)]
pub(super) fn inventory_item(
    slot_type: i32,
    slot_id: i32,
    value: ItemBase0104,
    name: &str,
    level: i32,
    price: i32,
    icon: Option<&str>,
    equip_eligible: Option<bool>,
) -> CashmallPlayerInventoryItem0104 {
    CashmallPlayerInventoryItem0104 {
        slot_type,
        slot_id,
        item: value,
        name: name.to_owned(),
        level,
        item_price: price,
        icon: icon.map(|path| CashmallIconRef0104::new(path).unwrap()),
        equip_eligible,
    }
}

#[cfg(test)]
pub(super) fn visible_state(previous_cursor_locked: bool) -> (CashmallUiState0104, CashmallUiOutbox0104) {
    let mut state = CashmallUiState0104::default();
    let mut outbox = CashmallUiOutbox0104::default();
    state.open_from(
        CashmallOpenSource0104::HiddenChatCommand,
        previous_cursor_locked,
        &mut outbox,
    );
    state.tick(CASHMALL_OPEN_SECONDS);
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Visible);
    outbox.clear();
    (state, outbox)
}

#[test]
fn mode_view_requires_active_state_viewport_and_complete_static_assets() {
    let projection = CashmallModeProjection0104::default();
    let hidden = CashmallUiState0104::default();
    assert!(
        cashmall_mode_view_0104(
            1_264,
            681,
            &hidden,
            Default::default(),
            &projection,
            [false; 5],
            [false; 5],
            true,
        )
        .is_none()
    );
    let (state, _) = visible_state(false);
    assert!(
        cashmall_mode_view_0104(
            0,
            681,
            &state,
            Default::default(),
            &projection,
            [false; 5],
            [false; 5],
            true,
        )
        .is_none()
    );
    assert!(
        cashmall_mode_view_0104(
            1_264,
            681,
            &state,
            Default::default(),
            &projection,
            [false; 5],
            [false; 5],
            false,
        )
        .is_none()
    );
    let view = cashmall_mode_view_0104(
        1_264,
        681,
        &state,
        Default::default(),
        &projection,
        [false; 5],
        [false; 5],
        true,
    )
    .unwrap();
    assert_eq!(view.tabs.len(), 5);
    assert_eq!(view.cash_digits, [0; 9]);
    assert!(view.npc_name.is_empty());
}

#[test]
fn unowned_nano_and_redeem_runtime_actions_fail_closed_without_packets() {
    let (state, mut outbox) = visible_state(false);
    assert_eq!(
        state.request_nano_tab(Default::default()),
        Err(CashmallActionBlocked0104::NanoProjectionUnavailable)
    );
    assert_eq!(
        state.request_redeem_code(Default::default()),
        Err(CashmallActionBlocked0104::RedeemRuntimeUnavailable)
    );
    assert!(outbox.is_empty());
    assert!(!CASHMALL_PURCHASE_NETWORK_CONTRACT_PRESENT);
    outbox.clear();
}
