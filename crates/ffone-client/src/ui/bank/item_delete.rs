//! Bank-owned confirmation drafts; the shared inventory runtime owns wire replies.
use super::*;
use crate::system_message_ui::{
    SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
    SystemMessageUiModel, SystemMessageUiOutbox,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankItemDeleteIntent0104 {
    pub pc: i32,
    pub npc: i32,
    pub slot: usize,
    pub item: ItemBase0104,
}
impl BankItemDeleteIntent0104 {
    pub fn valid(&self, projection: &BankModeProjection0104) -> bool {
        self.pc == projection.owner_pc_id
            && self.npc == projection.npc_id
            && projection
                .item_mode
                .inventory
                .get(self.slot)
                .is_some_and(|slot| slot.item.item == self.item)
            && self.item.item_id > 0
    }
}

#[derive(Resource)]
pub struct BankItemDeleteState {
    next_id: u64,
    confirmation: Option<(u64, BankItemDeleteIntent0104)>,
    ready: Option<BankItemDeleteIntent0104>,
    sent: Option<BankItemDeleteIntent0104>,
}
impl Default for BankItemDeleteState {
    fn default() -> Self {
        Self {
            next_id: 0x4244_454c_0000_0000,
            confirmation: None,
            ready: None,
            sent: None,
        }
    }
}
impl BankItemDeleteState {
    pub fn sent(&self) -> Option<BankItemDeleteIntent0104> {
        self.sent
    }
    pub fn has_ready(&self) -> bool {
        self.ready.is_some()
    }
    pub fn take_ready(&mut self) -> Option<BankItemDeleteIntent0104> {
        self.ready.take()
    }
    pub fn mark_sent(&mut self, intent: BankItemDeleteIntent0104) {
        self.sent = Some(intent);
    }
    pub fn finish(&mut self) {
        self.sent = None;
    }
    pub(super) fn open(
        &mut self,
        projection: &BankModeProjection0104,
        slot: usize,
        messages: &mut SystemMessageUiModel,
    ) -> bool {
        if self.confirmation.is_some() || self.ready.is_some() || self.sent.is_some() {
            return false;
        }
        let Some(row) = projection.item_mode.inventory.get(slot) else {
            return false;
        };
        if row.item.item.item_id <= 0 {
            return false;
        }
        let intent = BankItemDeleteIntent0104 {
            pc: projection.owner_pc_id,
            npc: projection.npc_id,
            slot,
            item: row.item.item,
        };
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let mut prompt = SystemMessageRequest::new_localized(
            id,
            LocalizedText::new("ui.inventory.popup.confirm_delete", "DELETE THIS ITEM?"),
            SystemMessageButtonType::DeleteItem,
        );
        if let UserEquipProjectedIcon::Resolved(icon) = &row.item.icon {
            prompt = prompt.with_icon_path(icon.runtime_path());
        }
        if intent.item.item_type == 7 {
            prompt = prompt.with_icon_quantity(intent.item.option.max(0));
        }
        messages.push(prompt);
        self.confirmation = Some((id, intent));
        true
    }
}

pub(super) fn consume(
    mut deletion: ResMut<BankItemDeleteState>,
    projection: Res<BankModeProjection0104>,
    state: Res<BankUiState>,
    mut modal: ResMut<BankModalState>,
    messages: Option<ResMut<SystemMessageUiModel>>,
    outbox: Option<ResMut<SystemMessageUiOutbox>>,
) {
    let (Some(mut messages), Some(mut outbox)) = (messages, outbox) else {
        return;
    };
    let current_id = deletion.confirmation.map(|confirmation| confirmation.0);
    // Retire late choices from our own old dialogs; never retain an unbounded stale queue.
    let actions = if outbox.is_empty() {
        Vec::new()
    } else {
        outbox.drain_matching(|action| {
            matches!(action, SystemMessageUiAction::Chosen {request_id,..}
            if *request_id >= 0x4244_454c_0000_0000 && *request_id < deletion.next_id)
        })
    };
    let Some((id, intent)) = deletion.confirmation else {
        return;
    };
    if state.phase != BankLifecyclePhase::Visible
        || state.send_pending
        || !intent.valid(&projection)
    {
        messages.remove(id);
        deletion.confirmation = None;
        return;
    }
    if let Some(SystemMessageUiAction::Chosen {
        button_type,
        choice,
        ..
    }) = actions.iter().find(|action| matches!(action, SystemMessageUiAction::Chosen {request_id,..} if Some(*request_id) == current_id))
    {
        deletion.confirmation = None;
        messages.remove(id);
        if *button_type == SystemMessageButtonType::DeleteItem
            && *choice == SystemMessageChoice::Primary
        {
            deletion.ready = Some(intent);
        }
        // Do not let the same pointer press reach Bank controls beneath the prompt.
        if !modal.system_popup { modal.system_popup = true; }
    } else if !messages
        .stack()
        .iter()
        .any(|message| message.request_id == id)
    {
        deletion.confirmation = None;
    } else {
        if !modal.system_popup { modal.system_popup = true; }
    }
}

#[cfg(test)]
mod tests;
