use super::*;

pub const VENDOR_RESTRICTED_ITEM_FRAME_PATH: &str = "ui/en/vendor/restricted-item-frame.png";

pub const VENDOR_BATTERY_FRAME_RECTS: [VendorUiRect; 2] = [
    VendorUiRect::new(0.0, 572.0, 64.0, 30.0),
    VendorUiRect::new(0.0, 602.0, 64.0, 30.0),
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VendorSlotFrameVisual0104 {
    #[default]
    Empty,
    Occupied,
    Restricted,
}

pub(super) fn vendor_slot_frame_image(
    visual: VendorSlotFrameVisual0104,
    assets: &VendorUiAssets,
) -> Handle<Image> {
    assets.image(match visual {
        VendorSlotFrameVisual0104::Empty => VendorStaticAssetRole::SlotEmpty,
        VendorSlotFrameVisual0104::Occupied => VendorStaticAssetRole::SlotOccupied,
        VendorSlotFrameVisual0104::Restricted => VendorStaticAssetRole::Restricted,
    })
}
