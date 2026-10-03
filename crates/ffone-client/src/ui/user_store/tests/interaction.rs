use super::*;

#[test]
fn both_row_buttons_share_popup_boundary_and_keep_slot_semantics() {
    let (mut state, mut authority, mut outbox) = ready_state();
    authority.listings[0] = Some(UserStoreListing0104 {
        list_slot: 0,
        inventory_slot: Some(2),
        item: item(0, 11, 1),
        price: 10,
    });
    let primary =
        state.activate_store_row(&authority, 0, UserStoreRowButton0104::Primary, &mut outbox);
    state.modal.item_popup = false;
    let secondary = state.activate_store_row(
        &authority,
        0,
        UserStoreRowButton0104::Secondary,
        &mut outbox,
    );
    assert_eq!(primary, secondary);
    let UserStoreActionOutcome0104::Popup(popup) = primary else {
        panic!("popup expected")
    };
    assert_eq!(popup.slot_type, USER_STORE_MY_SLOT_TYPE);

    state.mode = UserStoreMode0104::UserStore;
    state.modal.item_popup = false;
    authority.target_pc_id = Some(TARGET);
    authority.listings[0]
        .as_mut()
        .expect("listing")
        .inventory_slot = None;
    let UserStoreActionOutcome0104::Popup(popup) =
        state.activate_store_row(&authority, 0, UserStoreRowButton0104::Primary, &mut outbox)
    else {
        panic!("buyer popup expected")
    };
    assert_eq!(popup.slot_type, USER_STORE_OTHER_SLOT_TYPE);
}
