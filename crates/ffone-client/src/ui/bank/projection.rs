use super::*;

pub(super) fn project_bank_icon(
    item: ItemBase0104,
    catalog: &impl UserEquipItemCatalog,
) -> UserEquipProjectedIcon {
    if InventoryRuntime0104::item_is_empty(item) {
        return UserEquipProjectedIcon::Empty;
    }
    match UserEquipCatalogQuery::from_non_empty_item(item) {
        Err(UserEquipCatalogQueryError::MalformedIdentity { item_type, item_id }) => {
            UserEquipProjectedIcon::MissingChecker(UserEquipMissingIconReason::MalformedIdentity {
                item_type,
                item_id,
            })
        }
        Err(UserEquipCatalogQueryError::EmptyItem { .. }) => {
            unreachable!("non-empty bank item was checked before catalog query")
        }
        Ok(query) if query.kind.legacy_returns_null() => {
            UserEquipProjectedIcon::MissingChecker(UserEquipMissingIconReason::QuestLegacyNull {
                query,
            })
        }
        Ok(query) => {
            match catalog.resolve_icon(query) {
                Some(icon) => UserEquipProjectedIcon::Resolved(icon),
                None => UserEquipProjectedIcon::MissingChecker(
                    UserEquipMissingIconReason::CatalogMiss { query },
                ),
            }
        }
    }
}
