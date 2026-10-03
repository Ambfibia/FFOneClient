use super::*;

pub(super) struct AllowEquip;

impl Pc2pcEquipEligibility for AllowEquip {
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }
}
