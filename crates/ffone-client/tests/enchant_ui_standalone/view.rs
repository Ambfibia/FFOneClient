use super::*;

pub(super) fn attach_ready_weapon(model: &mut EnchantModeModel0104) {
    model
        .attach(EnchantAttachmentSlot0104::Target, selectable(7, 0, 501, 1))
        .unwrap();
    model
        .attach(
            EnchantAttachmentSlot0104::WeaponMaterial,
            selectable(8, 7, ENCHANT_WEAPON_MATERIAL_ID_0104, 40),
        )
        .unwrap();
}
