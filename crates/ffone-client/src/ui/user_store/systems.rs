use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserStoreApplyError0104 {
    InvalidMaximumListSlots(i32),
    InvalidListSlot(i32),
    ListSlotOverMaximum(i32),
    OccupiedListSlot(i32),
    EmptyListSlot(i32),
    InvalidInventorySlot(i32),
    EmptyInventorySlot(i32),
    OccupiedInventorySlot(i32),
    MissingOriginalInventorySlot(i32),
    OriginalInventorySlotMismatch { expected: Option<i32>, actual: i32 },
    InvalidStackQuantity { available: i32, requested: i32 },
    StackOverflow,
    ItemMismatch,
    AlreadySold(i32),
    MissingTargetPc,
    TargetPcMismatch { expected: i32, actual: i32 },
}

impl fmt::Display for UserStoreApplyError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "street-stall authoritative transaction rejected: {self:?}"
        )
    }
}

impl Error for UserStoreApplyError0104 {}

pub(super) fn advance_user_store_ui_0104(
    time: Res<Time>,
    mut state: ResMut<UserStoreUiState0104>,
    mut popup: ResMut<UserStorePopupPresentation0104>,
) {
    state.tick_opening(time.delta_secs());
    if !state.active || !state.modal.item_popup {
        popup.close();
    }
}
