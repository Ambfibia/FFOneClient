use super::*;

pub(super) fn bank_full_localized(location: BankSlotLocation0104) -> LocalizedText {
    match location {
        BankSlotLocation0104::Bank => LocalizedText::new("ui.bank.full", "Bank is full!"),
        BankSlotLocation0104::Inventory => {
            LocalizedText::new("ui.bank.inventory_full", "Inventory is full")
        }
    }
}

pub(super) fn bank_static_localized(value: &'static str) -> LocalizedText {
    let key = match value {
        "Morbucks Savings and Loan" => "ui.bank.title",
        "BANK" => "ui.bank.tab.vault",
        "EQUIPMENT" => "ui.inventory.tab.equipment",
        _ => "ui.content.passthrough",
    };
    if key == "ui.content.passthrough" {
        LocalizedText::new(key, "{text}").with_arg("text", value)
    } else {
        LocalizedText::new(key, value)
    }
}

pub(super) fn bank_count_localized(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.inventory.item.count", "{count}").with_arg("count", value)
}

pub(super) fn bank_taros_digit_localized(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.bank.taros.digit", "{digit}").with_arg("digit", value)
}

pub(super) fn bank_equipment_slot_localized(visual_index: usize) -> LocalizedText {
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
    match spec.label_ordinal {
        Some(ordinal) => LocalizedText::new("ui.inventory.slot.weapon", "WEAPON {ordinal}")
            .with_arg("ordinal", ordinal.to_string()),
        None => {
            let fallback = equipment_slot_label(visual_index);
            let key = match spec.label_key {
                "HEAD" => "ui.inventory.slot.head",
                "FACE" => "ui.inventory.slot.face",
                "BACK" => "ui.inventory.slot.back",
                "CHEST" => "ui.inventory.slot.chest",
                "LEGS" => "ui.inventory.slot.legs",
                "FEET" => "ui.inventory.slot.feet",
                "VEHICLE" => "ui.inventory.slot.vehicle",
                _ => "ui.content.passthrough",
            };
            if key == "ui.content.passthrough" {
                LocalizedText::new(key, "{text}").with_arg("text", fallback)
            } else {
                LocalizedText::new(key, fallback)
            }
        }
    }
}
