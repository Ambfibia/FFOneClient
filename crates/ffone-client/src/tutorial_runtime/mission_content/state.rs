
/// The only two `eItemLocation` values selected by clean warp gating.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum GameplayWarpInventoryLocation {
    Inventory = 1,
    Quest = 2,
}

impl GameplayWarpInventoryLocation {
    #[must_use]
    pub const fn wire_value(self) -> i32 {
        self as i32
    }
}
