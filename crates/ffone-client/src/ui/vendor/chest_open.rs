use super::*;

/// Captures the complete inventory authority at the moment OPEN is pressed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorChestOpenIntent {
    pub pc: i32,
    pub session: VendorSession0104,
    pub slot: usize,
    pub item: ItemBase0104,
}

impl VendorChestOpenIntent {
    pub fn valid(&self, projection: &VendorModeProjection0104) -> bool {
        self.pc == projection.owner_pc_id
            && self.session == projection.session
            && self.item.item_type == 9
            && self.item.item_id > 0
            && projection.inventory.get(self.slot).is_some_and(|row| {
                !row.item.empty && row.slot_index == self.slot && row.item.item == self.item
            })
    }
}

/// The app sends through the shared inventory request owner and reconciles its reply.
#[derive(Default, Resource)]
pub struct VendorChestOpenState {
    ready: Option<VendorChestOpenIntent>,
    sent: Option<VendorChestOpenIntent>,
}

impl VendorChestOpenState {
    pub(super) fn queue(&mut self, intent: VendorChestOpenIntent) -> bool {
        if self.ready.is_some() || self.sent.is_some() {
            return false;
        }
        self.ready = Some(intent);
        true
    }
    pub fn has_ready(&self) -> bool {
        self.ready.is_some()
    }
    pub fn take_ready(&mut self) -> Option<VendorChestOpenIntent> {
        self.ready.take()
    }
    pub fn sent(&self) -> Option<VendorChestOpenIntent> {
        self.sent
    }
    pub fn mark_sent(&mut self, intent: VendorChestOpenIntent) {
        self.sent = Some(intent);
    }
    pub fn finish(&mut self) {
        self.sent = None;
    }
}
