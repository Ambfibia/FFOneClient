use super::*;

pub(super) struct PreviewCatalog;

impl VendorItemCatalog0104 for PreviewCatalog {
    fn resolve(&self, value: ItemBase0104) -> Option<VendorItemMetadata0104> {
        let path = match (value.item_type, value.item_id) {
            (0, 100) => ICON_WEAPON_01,
            (0, 108) => ICON_WEAPON_02,
            (1, 101) => ICON_COSMETIC_00,
            (2, 102) => ICON_COSMETIC_01,
            (3, 103) => ICON_COSMETIC_02,
            (4, 104) => ICON_COSMETIC_03,
            (5, 105) => ICON_COSMETIC_04,
            (6, 106) => ICON_COSMETIC_05,
            (10, 107) => ICON_VEHICLE_00,
            (7, 200) => ICON_GENERAL_00,
            (7, 201) => ICON_GENERAL_01,
            // Deliberate catalog miss proves the clean checker fallback.
            (_, 300) => return None,
            _ => return None,
        };
        Some(VendorItemMetadata0104 {
            name: match value.item_id {
                100 => "Retro Rocket".to_owned(),
                101 => "Dee Dee Jacket".to_owned(),
                102 => "Samurai Pants".to_owned(),
                103 => "Sector V Sneakers".to_owned(),
                104 => "Dexter Cap".to_owned(),
                105 => "Mandark Shades".to_owned(),
                106 => "Nano Backpack".to_owned(),
                107 => "Monkey Skyboard".to_owned(),
                108 => "Fusion Blaster".to_owned(),
                200 => "Health Pack".to_owned(),
                201 => "Nano Potion".to_owned(),
                _ => "Vendor Item".to_owned(),
            },
            level: match value.item_id {
                107 => 20,
                108 => 16,
                _ => 8 + i32::from(value.item_id % 8),
            },
            buy_price: match value.item_id {
                107 => 2_500,
                108 => 1_250,
                200 => 120,
                201 => 80,
                _ => 450 + i32::from(value.item_id % 5) * 100,
            },
            sell_price: 50 + i32::from(value.item_id % 7) * 15,
            sellable: true,
            general_item_type: (value.item_type == 7).then_some(1),
            battery_recharge: None,
            stack_size: (value.item_type == 7).then_some(20),
            icon: VendorIconRef::new(path).ok(),
        })
    }

    fn vehicle_speed_class(&self, value: ItemBase0104) -> Option<i32> {
        (value.item_type == 10 && value.item_id == 107).then_some(700)
    }
}
