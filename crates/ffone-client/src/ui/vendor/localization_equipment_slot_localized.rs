use super::*;

pub(super) fn inventory_count_localized(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.inventory.item.count", "{count}").with_arg("count", value)
}

pub(super) fn equipment_slot_localized(visual_index: usize) -> LocalizedText {
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
    let fallback = equipment_slot_label(visual_index);
    match spec.label_ordinal {
        Some(ordinal) => LocalizedText::new("ui.inventory.slot.weapon", "WEAPON {ordinal}")
            .with_arg("ordinal", ordinal.to_string()),
        None => {
            let key = match spec.label_key {
                "HEAD" => "ui.inventory.slot.head",
                "FACE" => "ui.inventory.slot.face",
                "BACK" => "ui.inventory.slot.back",
                "CHEST" => "ui.inventory.slot.chest",
                "LEGS" => "ui.inventory.slot.legs",
                "FEET" => "ui.inventory.slot.feet",
                "VEHICLE" => "ui.inventory.slot.vehicle",
                unknown => panic!("unsupported clean equipment label {unknown}"),
            };
            LocalizedText::new(key, fallback)
        }
    }
}

pub(super) fn vendor_tab_localized(value: &'static str) -> LocalizedText {
    match value {
        "BUY" => LocalizedText::new("ui.vendor.tab.buy", value),
        "BUY BACK" => LocalizedText::new("ui.vendor.tab.buyback", value),
        unknown => panic!("unsupported clean vendor tab {unknown}"),
    }
}

pub(super) fn vendor_title_localized(name: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.vendor.shopkeeper", "SHOPKEEPER - {name}").with_arg("name", name)
}

pub(super) fn vendor_service_localized(service: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.vendor.service", "{service}").with_arg("service", service)
}

pub(super) fn vendor_item_name_localized(name: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.vendor.item.name", "{name}").with_arg("name", name)
}

pub(super) fn vendor_level_localized(level: Option<i32>) -> LocalizedText {
    LocalizedText::new("ui.vendor.item.level", "LEVEL {level}").with_arg(
        "level",
        level.map_or_else(|| "--".to_owned(), |value| value.to_string()),
    )
}

pub(super) fn vendor_price_localized(price: Option<i32>) -> LocalizedText {
    LocalizedText::new("ui.vendor.item.price", "{price} TAROS").with_arg(
        "price",
        price.map_or_else(|| "--".to_owned(), |value| value.to_string()),
    )
}

pub(super) fn vendor_vehicle_speed_localized(speed: i32) -> LocalizedText {
    LocalizedText::new("ui.vendor.item.vehicle_speed", "Speed: {speed} Class")
        .with_arg("speed", speed.to_string())
}

pub(super) fn inventory_quest_localized(item_id: i16) -> LocalizedText {
    LocalizedText::new("ui.inventory.item.quest", "Quest {item_id}")
        .with_arg("item_id", item_id.to_string())
}

pub(super) fn vendor_taros_digit_localized(digit: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.vendor.taros.digit", "{digit}").with_arg("digit", digit)
}

pub(super) fn vendor_battery_count_localized(count: i32) -> LocalizedText {
    LocalizedText::new("ui.enchant.battery.count", "{count}").with_arg("count", count.to_string())
}
