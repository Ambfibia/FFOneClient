use super::*;

pub(super) fn equipment_slot_localized_text(visual_index: usize) -> LocalizedText {
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
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
                _ => "ui.content.passthrough",
            };
            if key == "ui.content.passthrough" {
                pc2pc_passthrough_text(equipment_slot_label(visual_index))
            } else {
                LocalizedText::new(key, equipment_slot_label(visual_index))
            }
        }
    }
}

pub(super) fn bind_localized_text(text: Option<Mut<LocalizedText>>, value: LocalizedText) {
    if let Some(mut text) = text {
        *text = value;
    }
}
