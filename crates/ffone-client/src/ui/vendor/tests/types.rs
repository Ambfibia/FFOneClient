use super::*;

pub(super) struct TestEligibility;

impl VendorEquipEligibility0104 for TestEligibility {
    fn enable_equip(&self, value: ItemBase0104) -> Option<bool> {
        match value.item_id {
            6 => Some(false),
            90 => None,
            _ => Some(true),
        }
    }
}
