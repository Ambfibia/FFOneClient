use super::*;

pub(super) fn write_item(bytes: &mut [u8], offset: usize, value: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}

#[test]
fn vendor_dynamic_copy_uses_semantic_templates_and_exact_arguments() {
    let title = vendor_title_localized("Dexter");
    assert_eq!(title.key, "ui.vendor.shopkeeper");
    assert_eq!(title.fallback, "SHOPKEEPER - {name}");
    assert_eq!(title.args.get("name").map(String::as_str), Some("Dexter"));

    let level = vendor_level_localized(Some(17));
    assert_eq!(level.key, "ui.vendor.item.level");
    assert_eq!(level.args.get("level").map(String::as_str), Some("17"));
    let price = vendor_price_localized(Some(1_250));
    assert_eq!(price.key, "ui.vendor.item.price");
    assert_eq!(price.args.get("price").map(String::as_str), Some("1250"));
    let missing_price = vendor_price_localized(None);
    assert_eq!(
        missing_price.args.get("price").map(String::as_str),
        Some("--")
    );

    let service = vendor_service_localized("Rare gear");
    assert_eq!(service.key, "ui.vendor.service");
    assert_eq!(
        service.args.get("service").map(String::as_str),
        Some("Rare gear")
    );
    let name = vendor_item_name_localized("Skyboard");
    assert_eq!(name.key, "ui.vendor.item.name");
    assert_eq!(name.args.get("name").map(String::as_str), Some("Skyboard"));
    let speed = vendor_vehicle_speed_localized(700);
    assert_eq!(speed.key, "ui.vendor.item.vehicle_speed");
    assert_eq!(speed.args.get("speed").map(String::as_str), Some("700"));
    let quest = inventory_quest_localized(42);
    assert_eq!(quest.key, "ui.inventory.item.quest");
    assert_eq!(quest.args.get("item_id").map(String::as_str), Some("42"));
    let digit = vendor_taros_digit_localized("7");
    assert_eq!(digit.key, "ui.vendor.taros.digit");
    assert_eq!(digit.args.get("digit").map(String::as_str), Some("7"));
    let battery = vendor_battery_count_localized(25);
    assert_eq!(battery.key, "ui.enchant.battery.count");
    assert_eq!(battery.args.get("count").map(String::as_str), Some("25"));
    assert_eq!(vendor_button_text_color(false), Color::srgb(0.9, 0.9, 0.9));
    assert_eq!(
        vendor_button_text_color(true),
        Color::srgb(0.229_838_71, 0.463_709_68, 1.0)
    );
}
