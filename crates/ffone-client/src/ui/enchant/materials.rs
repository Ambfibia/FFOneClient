use super::*;

pub const ENCHANT_WEAPON_MATERIAL_ID_0104: i16 = 101;

pub const ENCHANT_ARMOR_MATERIAL_ID_0104: i16 = 102;

pub(super) fn enchant_material_required_count_0104(
    projection: &EnchantModeProjection0104,
    slot: EnchantAttachmentSlot0104,
    valid_target: bool,
) -> Option<String> {
    if !valid_target {
        return None;
    }
    if projection.selection.is_attached(slot) {
        return None;
    }
    let requirements = projection.requirements?;
    let (item_id, count) = match slot {
        EnchantAttachmentSlot0104::WeaponMaterial => (
            requirements.weapon_material_id,
            requirements.weapon_material_count,
        ),
        EnchantAttachmentSlot0104::ArmorMaterial => (
            requirements.armor_material_id,
            requirements.armor_material_count,
        ),
        _ => unreachable!(),
    };
    (item_id > 0).then(|| count.to_string())
}

pub(super) fn enchant_material_id_0104(
    projection: &EnchantModeProjection0104,
    slot: EnchantAttachmentSlot0104,
) -> i16 {
    let Some(requirements) = projection.requirements else {
        return 0;
    };
    match slot {
        EnchantAttachmentSlot0104::WeaponMaterial => requirements.weapon_material_id,
        EnchantAttachmentSlot0104::ArmorMaterial => requirements.armor_material_id,
        _ => 0,
    }
}

pub(super) fn enchant_material_error_0104(
    projection: &EnchantModeProjection0104,
    slot: EnchantAttachmentSlot0104,
) -> bool {
    match slot {
        EnchantAttachmentSlot0104::WeaponMaterial => projection.material_quantity_errors[0],
        EnchantAttachmentSlot0104::ArmorMaterial => projection.material_quantity_errors[1],
        _ => false,
    }
}
