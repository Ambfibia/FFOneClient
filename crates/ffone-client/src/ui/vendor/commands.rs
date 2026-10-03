use super::*;

pub const VENDOR_HELP_EVENT_ID: i32 = 24;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct VendorRequestIdentity0104 {
    pub npc_id: i32,
    pub vendor_id: i32,
    pub list_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorActionOutcome0104 {
    Intent(VendorIntent0104),
    SystemMessage(VendorSystemMessage0104),
    Confirmation(VendorConfirmation0104),
    SilentBlocked(VendorSilentBlock0104),
}

/// Small typed boundary for the clean `EquipPopup`/`GumPopup`/`TuringPopup`
/// owner. It preserves which buttons and quantity rules that owner must render
/// without changing a primary click into an invented immediate transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorItemActionPopup0104 {
    pub source: VendorItemPopupSource0104,
    pub buy: Option<VendorQuantityContract0104>,
    pub buyback: bool,
    pub sell: Option<VendorQuantityContract0104>,
    pub delete: bool,
    pub open_chest: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorActionBlocked0104 {
    ControlsDisabled,
    CloseGateRejected,
}
