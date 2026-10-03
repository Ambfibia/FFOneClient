use super::*;

#[test]
fn nano_gallery_opens_read_only_primary_detail_viewer() {
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiElement::NanoSlotFrame"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipNanoSlotControl"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipUiElement::NanoViewer"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipStaticAssetRole::NanoPopupNext"));
    assert!(!USER_EQUIP_SOURCE.contains("UserEquipUiAction::TuneNano"));
}
