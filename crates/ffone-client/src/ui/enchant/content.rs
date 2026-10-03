use super::*;
use crate::{
    tutorial_mission_content::TutorialMissionContent, user_equip_ui::*,
    vendor_ui::VendorItemCatalog0104,
};

pub fn enchant_item_presentation_from_content(
    content: &TutorialMissionContent,
    item: ItemBase0104,
) -> Option<EnchantItemPresentation0104> {
    let metadata = VendorItemCatalog0104::resolve(content, item)?;
    let detail = content.gameplay_user_equip_item_detail(item.item_type, item.item_id)?;
    let text =
        content.gameplay_user_equip_item_text(item.item_type, user_equip_display_text_id(item));
    let combined = (0..=3).contains(&item.item_type) && (item.option >> 16) as i16 > 0;
    let icon_path = UserEquipCatalogQuery::from_non_empty_item(item)
        .ok()
        .and_then(|query| UserEquipItemCatalog::resolve_icon(content, query))
        .map(|icon| icon.runtime_path().to_owned())
        .or_else(|| {
            metadata
                .icon
                .as_ref()
                .map(|icon| icon.runtime_path().to_owned())
        });
    Some(EnchantItemPresentation0104 {
        name: text
            .as_ref()
            .map_or(metadata.name, |t| t.0.fallback.clone()),
        description: text.map_or(detail.description.clone(), |t| t.1.fallback),
        icon_path,
        minimum_level: detail.level,
        cashable: 0,
        can_equip: EnchantTargetKind0104::from_item_type(item.item_type).is_some(),
        point_rating: detail.point_rating,
        group_rating: detail.group_rating,
        defense_rating: detail.defense_rating,
        type_label: item_type(item.item_type, detail.target_mode).fallback,
        range_label: item_range(item.item_type, detail.equip_type).fallback,
        rarity_label: rarity(combined, detail.rarity).fallback,
        trade_label: trade(!combined && detail.tradeable).fallback,
    })
}

fn item_type(kind: i16, target: Option<i32>) -> LocalizedText {
    if kind == 0 {
        user_equip_weapon_type_localized(target)
    } else {
        user_equip_item_type_localized(kind)
    }
}
fn item_range(kind: i16, equip: Option<i32>) -> LocalizedText {
    user_equip_range_localized(if kind == 0 { equip } else { None })
}
fn rarity(combined: bool, value: Option<i32>) -> LocalizedText {
    if combined {
        LocalizedText::new("ui.inventory.rarity.special", "Special")
    } else {
        user_equip_rarity_localized(value)
    }
}
fn trade(allowed: bool) -> LocalizedText {
    if allowed {
        LocalizedText::new("ui.inventory.popup.trade_value", "Tradable")
    } else {
        LocalizedText::new("ui.inventory.popup.not_tradable", "Not tradable")
    }
}

#[cfg(test)]
mod tests;

pub(super) fn localized_field(
    content: &TutorialMissionContent,
    projection: &EnchantModeProjection0104,
    element: EnchantUiElement0104,
) -> Option<LocalizedText> {
    let item = match element {
        EnchantUiElement0104::TargetName | EnchantUiElement0104::TargetDescription => projection
            .selection
            .visual_item(EnchantAttachmentSlot0104::Target)
            .map(|i| i.item),
        EnchantUiElement0104::SupportName(slot)
        | EnchantUiElement0104::SupportDescription(slot) => projection
            .selection
            .visual_item(slot)
            .map(|i| i.item)
            .or_else(|| {
                let id = match slot {
                    EnchantAttachmentSlot0104::WeaponMaterial => ENCHANT_WEAPON_MATERIAL_ID_0104,
                    EnchantAttachmentSlot0104::ArmorMaterial => ENCHANT_ARMOR_MATERIAL_ID_0104,
                    EnchantAttachmentSlot0104::Helper1 => ENCHANT_HELP_ITEM_1_ID_0104,
                    EnchantAttachmentSlot0104::Helper2 => ENCHANT_HELP_ITEM_2_ID_0104,
                    _ => return None,
                };
                Some(ItemBase0104 {
                    item_type: 7,
                    item_id: id,
                    option: 1,
                    time_limit: 0,
                })
            }),
        EnchantUiElement0104::SuccessName
        | EnchantUiElement0104::SuccessDescription
        | EnchantUiElement0104::SuccessTypeValue
        | EnchantUiElement0104::SuccessRangeValue
        | EnchantUiElement0104::SuccessRarityValue
        | EnchantUiElement0104::SuccessTradeValue => projection.success.as_ref().map(|s| s.item),
        _ => None,
    };
    let item = item?;
    let id = user_equip_display_text_id(item);
    let combined = (0..=3).contains(&item.item_type) && (item.option >> 16) as i16 > 0;
    let text = content.gameplay_user_equip_item_text(item.item_type, id);
    let detail = content.gameplay_user_equip_item_detail(item.item_type, item.item_id);
    let value = match element {
        EnchantUiElement0104::TargetName
        | EnchantUiElement0104::SupportName(_)
        | EnchantUiElement0104::SuccessName => text.map(|t| t.0),
        EnchantUiElement0104::TargetDescription
        | EnchantUiElement0104::SupportDescription(_)
        | EnchantUiElement0104::SuccessDescription => text.map(|t| t.1),
        EnchantUiElement0104::SuccessTypeValue => {
            detail.map(|d| item_type(item.item_type, d.target_mode))
        }
        EnchantUiElement0104::SuccessRangeValue => {
            detail.map(|d| item_range(item.item_type, d.equip_type))
        }
        EnchantUiElement0104::SuccessRarityValue => detail.map(|d| rarity(combined, d.rarity)),
        EnchantUiElement0104::SuccessTradeValue => detail.map(|d| trade(!combined && d.tradeable)),
        _ => None,
    };
    value
}
