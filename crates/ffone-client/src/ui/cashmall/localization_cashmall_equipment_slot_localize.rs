use super::*;

pub(super) fn cashmall_equipment_slot_localized_text_0104(visual_index: usize) -> LocalizedText {
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
    match spec.kind {
        UserEquipEquipmentSlotKind::Head => {
            LocalizedText::new("ui.cashmall.equipment.slot.head", spec.label_key)
        }
        UserEquipEquipmentSlotKind::Face => {
            LocalizedText::new("ui.cashmall.equipment.slot.face", spec.label_key)
        }
        UserEquipEquipmentSlotKind::Back => {
            LocalizedText::new("ui.cashmall.equipment.slot.back", spec.label_key)
        }
        UserEquipEquipmentSlotKind::UpperBody => {
            LocalizedText::new("ui.cashmall.equipment.slot.chest", spec.label_key)
        }
        UserEquipEquipmentSlotKind::LowerBody => {
            LocalizedText::new("ui.cashmall.equipment.slot.legs", spec.label_key)
        }
        UserEquipEquipmentSlotKind::Foot => {
            LocalizedText::new("ui.cashmall.equipment.slot.feet", spec.label_key)
        }
        UserEquipEquipmentSlotKind::PrimaryWeapon | UserEquipEquipmentSlotKind::SecondaryWeapon => {
            LocalizedText::new("ui.cashmall.equipment.slot.weapon", "WEAPON {ordinal}").with_arg(
                "ordinal",
                spec.label_ordinal.unwrap_or_default().to_string(),
            )
        }
        UserEquipEquipmentSlotKind::Vehicle => {
            LocalizedText::new("ui.cashmall.equipment.slot.vehicle", spec.label_key)
        }
    }
}

pub(super) fn cashmall_tab_localized_text_0104(tab: CashmallTab0104) -> LocalizedText {
    let key = match tab {
        CashmallTab0104::New => "ui.cashmall.tab.new",
        CashmallTab0104::Scroll => "ui.cashmall.tab.scroll",
        CashmallTab0104::Potion => "ui.cashmall.tab.potion",
        CashmallTab0104::Equipment => "ui.cashmall.tab.equipment",
        CashmallTab0104::Etc => "ui.cashmall.tab.etc",
    };
    LocalizedText::new(key, tab.legacy_name())
}

pub(super) fn bind_cashmall_localized_text_0104(text: Option<Mut<LocalizedText>>, value: LocalizedText) {
    let Some(mut text) = text else {
        return;
    };
    *text = value;
}

pub(super) fn bind_optional_cashmall_localized_text_0104(
    node: &mut Node,
    text: Option<Mut<LocalizedText>>,
    value: Option<LocalizedText>,
) {
    let Some(mut text) = text else {
        return;
    };
    if let Some(value) = value {
        node.display = Display::Flex;
        *text = value;
    } else {
        node.display = Display::None;
        *text = cashmall_passthrough_text_0104("");
    }
}
