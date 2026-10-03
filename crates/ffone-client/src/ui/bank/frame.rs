use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BankSlotFrameVisual {
    #[default]
    Empty,
    Occupied,
    Rejected,
    Locked,
}

pub(super) fn bank_slot_frame_image(visual: BankSlotFrameVisual, assets: &BankUiAssets) -> ImageNode {
    match visual {
        BankSlotFrameVisual::Empty | BankSlotFrameVisual::Occupied => sliced_image(
            assets.image(BankStaticAssetRole::BankSlotButton),
            BANK_SLOT_BUTTON_BORDER,
        ),
        BankSlotFrameVisual::Rejected | BankSlotFrameVisual::Locked => {
            stretched_image(assets.image(BankStaticAssetRole::LockedSlot))
        }
    }
}

pub(super) fn inventory_slot_frame_image(visual: BankSlotFrameVisual, assets: &BankUiAssets) -> Handle<Image> {
    assets.image(match visual {
        BankSlotFrameVisual::Empty => BankStaticAssetRole::SlotEmpty,
        BankSlotFrameVisual::Occupied => BankStaticAssetRole::SlotOccupied,
        BankSlotFrameVisual::Rejected | BankSlotFrameVisual::Locked => {
            BankStaticAssetRole::LockedSlot
        }
    })
}
