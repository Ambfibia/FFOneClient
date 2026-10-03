use super::*;

#[test]
fn equipped_nanos_use_live_portraits_and_primary_rects() {
    assert!(USER_EQUIP_SOURCE.contains("gameplay_ui::GameplayNanoPortraitImages"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiElement::NanoStatusPortrait(slot)"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiRect::new(45.0, 110.0, 128.0, 128.0)"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiRect::new(355.0, 69.0, 128.0, 128.0)"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiRect::new(368.0, 244.0, 128.0, 128.0)"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiRect::new(5.0, 18.0, 48.0, 9.0)"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiRect::new(5.0, 33.0, 118.0, 14.0)"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiRect::new(101.0, 44.0, 20.0, 20.0)"));
}
