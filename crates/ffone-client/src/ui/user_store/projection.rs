use super::*;

pub(super) fn project_user_store_item(
    item: ItemBase0104,
    catalog: &UserStoreItemCatalog0104,
) -> UserStoreProjectedItem0104 {
    if user_store_item_is_empty(item) {
        return UserStoreProjectedItem0104 {
            item,
            name: String::new(),
            level: 0,
            description: String::new(),
            icon: UserStorePresentationIcon0104::Empty,
            count_label: None,
        };
    }
    let metadata = catalog.resolve(item);
    UserStoreProjectedItem0104 {
        item,
        name: metadata
            .map(|value| value.name.clone())
            .unwrap_or_else(|| format!("ITEM {}:{}", item.item_type, item.item_id)),
        level: metadata.map_or(0, |value| value.level),
        description: metadata
            .map(|value| value.description.clone())
            .unwrap_or_default(),
        icon: metadata
            .and_then(|value| value.icon_path.clone())
            .map(UserStorePresentationIcon0104::Resolved)
            .unwrap_or(UserStorePresentationIcon0104::MissingChecker),
        count_label: (item.item_type == USER_STORE_GENERAL_ITEM_TYPE)
            .then(|| item.option.to_string()),
    }
}

#[must_use]
pub fn project_user_store_ui_0104(
    state: &UserStoreUiState0104,
    authority: &UserStoreAuthority0104,
    catalog: &UserStoreItemCatalog0104,
) -> UserStoreUiProjection0104 {
    let mut rows = Vec::new();
    match state.mode {
        UserStoreMode0104::MyStore => {
            for list_slot in 0..authority.maximum_list_slots.min(USER_STORE_LIST_CAPACITY) {
                let listing = authority.listings[list_slot];
                rows.push(UserStoreListingRowProjection0104 {
                    visual_row: rows.len(),
                    list_slot: list_slot as i32,
                    slot_type: USER_STORE_MY_SLOT_TYPE,
                    item: listing.map(|value| project_user_store_item(value.item, catalog)),
                    price: listing.map(|value| value.price),
                    seller_display_taros: listing
                        .filter(|value| value.price > 0)
                        .map(|value| user_store_seller_display_taros(value.price)),
                    sold: listing.is_some_and(|value| value.price < 0),
                    affordable: true,
                });
            }
        }
        UserStoreMode0104::UserStore => {
            // Clean skips list holes and packs surviving records visually.
            for listing in authority.listings.iter().flatten().copied() {
                rows.push(UserStoreListingRowProjection0104 {
                    visual_row: rows.len(),
                    list_slot: listing.list_slot,
                    slot_type: USER_STORE_OTHER_SLOT_TYPE,
                    item: Some(project_user_store_item(listing.item, catalog)),
                    price: Some(listing.price),
                    seller_display_taros: None,
                    sold: listing.price < 0,
                    affordable: listing.price >= 0 && listing.price <= authority.taros,
                });
            }
        }
        UserStoreMode0104::ReturnStore => {}
    }
    UserStoreUiProjection0104 {
        // `mUserName` has no observed assignment in the clean class.
        title: USER_STORE_TITLE_SUFFIX_TYPO.into(),
        listing_rows: rows,
        inventory: authority
            .inventory
            .iter()
            .copied()
            .enumerate()
            .map(|(slot, item)| UserStoreInventoryProjection0104 {
                slot,
                slot_type: 1,
                item: project_user_store_item(item, catalog),
            })
            .collect(),
        primary_button: if state.mode == UserStoreMode0104::MyStore && !authority.store_open {
            "OPEN STORE".into()
        } else {
            "CLOSE STORE".into()
        },
        show_go_to_game: state.mode == UserStoreMode0104::MyStore && authority.store_open,
        busy: state.busy(),
        error: state.last_error,
    }
}
