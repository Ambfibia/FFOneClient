//! Editable-layout keys of UserEquip elements.

use super::super::components::UserEquipUiElement;
use bevy::prelude::*;

pub(in super::super) fn user_equip_editable_layout_key(element: UserEquipUiElement) -> Option<String> {
    let key = match element {
        UserEquipUiElement::ItemTab => "item_tab",
        UserEquipUiElement::NanoTab => "nano_tab",
        UserEquipUiElement::ItemTabLabel => "item_tab_label",
        UserEquipUiElement::NanoTabLabel => "nano_tab_label",
        UserEquipUiElement::PcStuffPanelBackground => "inventory_panel",
        UserEquipUiElement::InventoryShadowA => "inventory_shadow_a",
        UserEquipUiElement::InventoryShadowB => "inventory_shadow_b",
        UserEquipUiElement::ScrollTrack => "scroll_track",
        UserEquipUiElement::ScrollUp => "scroll_up",
        UserEquipUiElement::ScrollDown => "scroll_down",
        UserEquipUiElement::Close => "close",
        UserEquipUiElement::Trash => "trash",
        UserEquipUiElement::Help => "help",
        UserEquipUiElement::TarosBack => "taros_back",
        UserEquipUiElement::Dexlabs => "dexlabs",
        UserEquipUiElement::BoostSlot => "boost_slot",
        UserEquipUiElement::BoostIcon => "boost_icon",
        UserEquipUiElement::BoostLabel => "boost_label",
        UserEquipUiElement::BoostValue => "boost_value",
        UserEquipUiElement::PotionSlot => "potion_slot",
        UserEquipUiElement::PotionIcon => "potion_icon",
        UserEquipUiElement::PotionLabel => "potion_label",
        UserEquipUiElement::PotionValue => "potion_value",
        UserEquipUiElement::EquipmentTitle => "equipment_title",
        UserEquipUiElement::InventorySlotFrame(index) => {
            return Some(format!("inventory_slot_{index:02}"));
        }
        UserEquipUiElement::NanoSlotFrame(index) => {
            return Some(format!("nano_slot_{index:02}"));
        }
        UserEquipUiElement::EquipmentSlotFrame(index) => {
            return Some(format!("equipment_slot_{index}"));
        }
        UserEquipUiElement::StatusPanel => "status_panel_back",
        UserEquipUiElement::StatusHpBack => "status_hp_back",
        UserEquipUiElement::StatusFusionBack => "status_fusion_back",
        UserEquipUiElement::StatusGuideBox => "status_guide_box",
        UserEquipUiElement::StatusName => "status_name",
        UserEquipUiElement::StatusLevel => "status_level",
        UserEquipUiElement::StatusHp => "status_hp",
        UserEquipUiElement::StatusHpValue => "status_hp_value",
        UserEquipUiElement::StatusFusionMatter => "status_fusion",
        UserEquipUiElement::StatusFusionMatterValue => "status_fusion_value",
        UserEquipUiElement::StatusGuideLabel => "status_guide_label",
        UserEquipUiElement::StatusGuideName => "status_guide_name",
        UserEquipUiElement::StatusGuideIcon => "status_guide_icon",
        UserEquipUiElement::ItemPopupBackdrop => "item_popup_backdrop",
        UserEquipUiElement::ItemPopupEquipInfo => "item_popup_equip_info",
        UserEquipUiElement::ItemPopupIcon => "item_popup_icon",
        UserEquipUiElement::ItemPopupTitle => "item_popup_title",
        UserEquipUiElement::ItemPopupClose => "item_popup_close",
        UserEquipUiElement::ItemPopupTrash => "item_popup_trash",
        UserEquipUiElement::ItemPopupIdentity => "item_popup_description",
        UserEquipUiElement::ItemPopupButton(index) => {
            return Some(format!("item_popup_button_{index}"));
        }
        UserEquipUiElement::ItemPopupGumNanoFrame(index) => {
            return Some(format!("item_popup_gum_nano_frame_{index}"));
        }
        UserEquipUiElement::ItemPopupGumNanoIcon(index) => {
            return Some(format!("item_popup_gum_nano_icon_{index}"));
        }
        UserEquipUiElement::ItemPopupGumNanoButton(index) => {
            return Some(format!("item_popup_gum_nano_button_{index}"));
        }
        UserEquipUiElement::ItemPopupField(index) => {
            return Some(format!("item_popup_field_{index}"));
        }
        _ => return None,
    };
    Some(key.to_owned())
}
