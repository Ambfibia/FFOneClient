use super::*;

pub(in super::super) fn apply_buddy_state_success(
    state: BuddyStateSuccess0104,
    model: &mut BuddyUiModel,
    runtime: &mut RuntimeStatus,
    selected_channel: ChatChannel,
) -> Result<(), String> {
    let updates = (0..BUDDY_MAX_SLOTS)
        .map(|slot| BuddyStateUpdate {
            slot,
            runtime_pc_id: state.buddy_ids[slot],
            legacy_state: state.buddy_states[slot] as i8,
        })
        .collect::<Vec<_>>();
    let notices = model
        .apply_state_snapshot(&updates)
        .map_err(|error| error.to_string())?;
    for notice in notices {
        push_buddy_presence_notice(runtime, notice, selected_channel);
    }
    model.reset_refresh_timer();
    Ok(())
}

#[derive(Debug, Resource)]
pub(in super::super) struct BuddyRuntimeIntegration {
    pub(in super::super) pending_remove: Option<BuddyTarget>,
    pub(in super::super) pending_warp: Option<PendingBuddyWarp>,
    pub(in super::super) pending_system_actions: BTreeMap<u64, PendingBuddySystemAction>,
    pub(in super::super) pending_nanocom_actions: BTreeMap<u64, PendingBuddyNanocomAction>,
    pub(in super::super) next_system_message_id: u64,
    pub(in super::super) next_nanocom_message_id: u64,
}

impl Default for BuddyRuntimeIntegration {
    fn default() -> Self {
        Self {
            pending_remove: None,
            pending_warp: None,
            pending_system_actions: BTreeMap::new(),
            pending_nanocom_actions: BTreeMap::new(),
            next_system_message_id: BUDDY_SYSTEM_MESSAGE_ID_BASE,
            next_nanocom_message_id: BUDDY_NANOCOM_MESSAGE_ID_BASE,
        }
    }
}

impl BuddyRuntimeIntegration {
    pub(in super::super) fn reset(&mut self) {
        self.pending_remove = None;
        self.pending_warp = None;
        self.pending_system_actions.clear();
        self.pending_nanocom_actions.clear();
        self.next_system_message_id = BUDDY_SYSTEM_MESSAGE_ID_BASE;
        self.next_nanocom_message_id = BUDDY_NANOCOM_MESSAGE_ID_BASE;
    }

    pub(in super::super) fn push_system_action(
        &mut self,
        system_messages: &mut SystemMessageUiModel,
        action: PendingBuddySystemAction,
        text: impl Into<String>,
        button_type: SystemMessageButtonType,
    ) {
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < BUDDY_SYSTEM_MESSAGE_ID_BASE {
            self.next_system_message_id = BUDDY_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending_system_actions.insert(request_id, action);
        system_messages.push(SystemMessageRequest::new(request_id, text, button_type));
    }

    pub(in super::super) fn push_nanocom_action(
        &mut self,
        nanocom_messages: &mut NanocomMessageUiModel,
        action: PendingBuddyNanocomAction,
        display_name: impl AsRef<str>,
    ) -> u64 {
        let request_id = self.next_nanocom_message_id;
        self.next_nanocom_message_id = self.next_nanocom_message_id.wrapping_add(1);
        if self.next_nanocom_message_id < BUDDY_NANOCOM_MESSAGE_ID_BASE {
            self.next_nanocom_message_id = BUDDY_NANOCOM_MESSAGE_ID_BASE;
        }
        self.pending_nanocom_actions.insert(request_id, action);
        nanocom_messages.enqueue_buddy_invite(request_id, display_name);
        request_id
    }
}
