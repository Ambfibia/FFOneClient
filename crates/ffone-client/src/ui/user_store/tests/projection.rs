use super::*;

#[test]
fn ready_and_user_list_project_exact_slot_types_and_skip_holes() {
    let (state, authority, _) = ready_state();
    assert_eq!(state.mode, UserStoreMode0104::MyStore);
    assert!(!authority.store_open);
    assert_eq!(authority.maximum_list_slots, 5);
    let mine =
        project_user_store_ui_0104(&state, &authority, &UserStoreItemCatalog0104::default());
    assert_eq!(mine.listing_rows.len(), 5);
    assert!(
        mine.listing_rows
            .iter()
            .all(|row| row.slot_type == USER_STORE_MY_SLOT_TYPE)
    );

    let mut state = UserStoreUiState0104::default();
    let mut authority = UserStoreAuthority0104::default();
    let mut outbox = UserStoreUiOutbox0104::default();
    let _ = state.begin_user_store(&mut authority, OWNER, TARGET, false, &mut outbox);
    let packet = list_payload(TARGET, &[(1, item(0, 11, 1), 99), (4, item(7, 22, 3), 199)]);
    state
        .receive_reply(
            &mut authority,
            STREETSTALL_REP_ITEM_LIST,
            &packet,
            &mut outbox,
        )
        .expect("list reply");
    assert!(authority.listings[0].is_none());
    assert!(authority.listings[1].is_some());
    assert!(authority.listings[4].is_some());
    let other =
        project_user_store_ui_0104(&state, &authority, &UserStoreItemCatalog0104::default());
    assert_eq!(other.listing_rows.len(), 2);
    assert_eq!(other.listing_rows[0].list_slot, 1);
    assert_eq!(other.listing_rows[1].list_slot, 4);
    assert!(
        other
            .listing_rows
            .iter()
            .all(|row| row.slot_type == USER_STORE_OTHER_SLOT_TYPE)
    );
}
