use super::*;

pub(super) fn enchant_support_presentation_0104(
    projection: &EnchantModeProjection0104,
    slot: EnchantAttachmentSlot0104,
) -> Option<&EnchantItemPresentation0104> {
    if projection.selection.is_attached(slot) {
        return projection
            .selection
            .visual_item(slot)
            .map(|item| &item.presentation);
    }
    match slot {
        EnchantAttachmentSlot0104::Target => None,
        EnchantAttachmentSlot0104::WeaponMaterial => Some(&projection.support.weapon_material),
        EnchantAttachmentSlot0104::ArmorMaterial => Some(&projection.support.armor_material),
        EnchantAttachmentSlot0104::Helper1 => Some(&projection.support.helper_1),
        EnchantAttachmentSlot0104::Helper2 => Some(&projection.support.helper_2),
    }
}

pub(super) fn enchant_support_text_0104<'a>(
    projection: &'a EnchantModeProjection0104,
    slot: EnchantAttachmentSlot0104,
    valid_target: bool,
    description: bool,
) -> Option<&'a str> {
    if !valid_target {
        return None;
    }
    if matches!(
        slot,
        EnchantAttachmentSlot0104::WeaponMaterial | EnchantAttachmentSlot0104::ArmorMaterial
    ) && (enchant_material_id_0104(projection, slot) == 0
        || enchant_material_error_0104(projection, slot))
    {
        return None;
    }
    enchant_support_presentation_0104(projection, slot).map(|presentation| {
        if description {
            presentation.description.as_str()
        } else {
            presentation.name.as_str()
        }
    })
}
