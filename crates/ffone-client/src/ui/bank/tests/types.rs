use super::*;

pub(super) struct AllowEquip;

impl BankEquipEligibility for AllowEquip {
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }
}
