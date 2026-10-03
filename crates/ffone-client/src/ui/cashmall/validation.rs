
pub(super) fn cashmall_item_needs_equip_validation(item_type: i16) -> bool {
    item_type <= 6 || item_type == 10
}
