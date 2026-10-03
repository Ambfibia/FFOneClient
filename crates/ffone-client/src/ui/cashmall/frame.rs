use super::*;

pub const CASHMALL_RECEIVE_PACKET_MUTATES_STATE: bool = false;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CashmallSlotFrameVisual0104 {
    #[default]
    Normal,
    Restricted,
}

pub(super) fn cashmall_shared_slot_frame_image_0104(
    visual: UserEquipSlotFrameVisual,
    assets: &CashmallUiAssets0104,
) -> Handle<Image> {
    assets.image(match visual {
        UserEquipSlotFrameVisual::Empty => CashmallStaticAssetRole0104::SlotEmpty,
        UserEquipSlotFrameVisual::Occupied => CashmallStaticAssetRole0104::SlotOccupied,
    })
}
