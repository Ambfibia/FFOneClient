use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuickSlotUiAction {
    pub logical_slot: usize,
    /// Raw value placed in `cnEvent(11, 16)[0]`.
    pub legacy_event_index: i32,
    pub source: QuickSlotActivationSource,
}
