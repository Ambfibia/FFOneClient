use super::*;

pub(super) struct PreviewCatalog;

impl UserEquipItemCatalog for PreviewCatalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        let path = match (query.kind, query.base_item_id) {
            (UserEquipCatalogKind::Equipment { .. }, 328) => "icons/items/weapons/wpnicon_385.png",
            (UserEquipCatalogKind::General, 7) => "icons/items/general/generalitemicon_16.png",
            (UserEquipCatalogKind::Chest, 77) => ICON_GENERAL_01,
            (UserEquipCatalogKind::Equipment { .. }, 100) => ICON_WEAPON_01,
            (UserEquipCatalogKind::Equipment { .. }, 108) => ICON_WEAPON_02,
            (UserEquipCatalogKind::Equipment { .. }, 101) => ICON_COSMETIC_00,
            (UserEquipCatalogKind::Equipment { .. }, 102) => ICON_COSMETIC_01,
            (UserEquipCatalogKind::Equipment { .. }, 103) => ICON_COSMETIC_02,
            (UserEquipCatalogKind::Equipment { .. }, 104) => ICON_COSMETIC_03,
            (UserEquipCatalogKind::Equipment { .. }, 105) => ICON_COSMETIC_04,
            (UserEquipCatalogKind::Equipment { .. }, 106) => ICON_COSMETIC_05,
            (UserEquipCatalogKind::Equipment { .. }, 107) => ICON_VEHICLE_00,
            (UserEquipCatalogKind::General, 200) => ICON_GENERAL_00,
            (UserEquipCatalogKind::General, 201) => ICON_GENERAL_01,
            // Deliberate miss: exact clean AvatarUtil checker fallback.
            (UserEquipCatalogKind::Chest, 300) => return None,
            _ => return None,
        };
        UserEquipIconRef::new(path).ok()
    }
}
