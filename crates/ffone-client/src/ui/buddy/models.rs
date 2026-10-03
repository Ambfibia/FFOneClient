use super::*;

#[derive(Debug, Resource)]
pub struct BuddyUiModel {
    pub(super) slots: Vec<Option<BuddyEntry>>,
    pub(super) selected_slot: Option<usize>,
    pub(super) pending_invites: VecDeque<BuddyInvite>,
    pub(super) group_member_uids: BTreeSet<i64>,
    pub(super) group_size: usize,
    pub(super) visible: bool,
    pub(super) social_buddy_enabled: bool,
    pub(super) player_moving: bool,
    pub(super) add_dialog_open: bool,
    pub(super) add_name_input: String,
    pub(super) chat_window_style: BuddyChatWindowStyle,
    pub(super) large_chat_width: f32,
    pub(super) quick_slot_active: bool,
    pub(super) ui_scale_override: Option<f32>,
    pub(super) scroll_y: f32,
    pub(super) refresh_elapsed: f32,
    pub(super) cooldown_elapsed: f32,
    pub(super) warp_cooldown_seconds: u32,
}

impl Default for BuddyUiModel {
    fn default() -> Self {
        Self {
            slots: vec![None; BUDDY_MAX_SLOTS],
            selected_slot: None,
            pending_invites: VecDeque::new(),
            group_member_uids: BTreeSet::new(),
            group_size: 1,
            visible: false,
            social_buddy_enabled: true,
            player_moving: false,
            add_dialog_open: false,
            add_name_input: String::new(),
            chat_window_style: BuddyChatWindowStyle::Small,
            large_chat_width: 440.0,
            quick_slot_active: false,
            ui_scale_override: None,
            scroll_y: 0.0,
            refresh_elapsed: 0.0,
            cooldown_elapsed: 0.0,
            warp_cooldown_seconds: 0,
        }
    }
}

impl BuddyUiModel {
    #[must_use]
    pub const fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    #[must_use]
    pub const fn chat_window_style(&self) -> BuddyChatWindowStyle {
        self.chat_window_style
    }

    pub fn set_chat_window_style(&mut self, style: BuddyChatWindowStyle) {
        self.chat_window_style = style;
    }

    /// Fresh `CnGuiChat.DoBuddyWindow` anchors the side panel to the current
    /// resizable chat width rather than to a second fixed screen coordinate.
    pub fn set_large_chat_width(&mut self, width: f32) {
        self.large_chat_width = width.clamp(325.0, 550.0);
    }

    pub fn set_quick_slot_active(&mut self, active: bool) {
        self.quick_slot_active = active;
    }

    #[must_use]
    pub const fn quick_slot_active(&self) -> bool {
        self.quick_slot_active
    }

    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale_override = Some(valid_ui_scale(scale));
    }

    pub fn clear_ui_scale_override(&mut self) {
        self.ui_scale_override = None;
    }

    #[must_use]
    pub fn effective_ui_scale(&self) -> f32 {
        self.ui_scale_override.unwrap_or(1.0)
    }

    #[must_use]
    pub fn slot(&self, slot: usize) -> Option<&BuddyEntry> {
        self.slots.get(slot).and_then(Option::as_ref)
    }

    #[must_use]
    pub const fn selected_slot(&self) -> Option<usize> {
        self.selected_slot
    }

    #[must_use]
    pub fn selected_target(&self) -> Option<BuddyTarget> {
        self.selected_slot
            .and_then(|slot| self.target_for_slot(slot).ok())
    }

    #[must_use]
    pub fn occupied_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.as_ref().is_some_and(|entry| entry.pc_uid != 0))
            .count()
    }

    #[must_use]
    pub fn visible_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| {
                slot.as_ref()
                    .is_some_and(|entry| entry.pc_uid != 0 && !entry.blocked)
            })
            .count()
    }

    pub fn set_entry(
        &mut self,
        slot: usize,
        entry: Option<BuddyEntry>,
    ) -> Result<Option<BuddyUiNotice>, BuddyUiError> {
        self.ensure_slot(slot)?;
        let entry = entry.filter(|entry| entry.pc_uid != 0);
        let previous = std::mem::replace(&mut self.slots[slot], entry);
        let selected_target_changed = self.selected_slot == Some(slot)
            && match (previous.as_ref(), self.slots[slot].as_ref()) {
                (Some(previous), Some(current)) => {
                    previous.pc_uid != current.pc_uid || current.blocked
                }
                _ => true,
            };
        if selected_target_changed {
            self.selected_slot = None;
        }
        self.clamp_scroll();

        let (Some(previous), Some(current)) = (previous.as_ref(), self.slots[slot].as_ref()) else {
            return Ok(None);
        };
        if previous.pc_uid != current.pc_uid
            || previous.presence == current.presence
            || current.blocked
        {
            return Ok(None);
        }
        let display_name = if previous.name_check_flag == 1 {
            format!("{} {}", current.first_name, current.last_name)
        } else {
            format!("Player {}", previous.pc_uid)
        };
        Ok(Some(BuddyUiNotice::PresenceChanged {
            target: BuddyTarget {
                slot,
                pc_uid: current.pc_uid,
            },
            display_name,
            presence: current.presence,
        }))
    }

    pub fn set_blocked(&mut self, target: BuddyTarget, blocked: bool) -> Result<(), BuddyUiError> {
        let entry = self.entry_for_target_mut(target)?;
        entry.blocked = blocked;
        if blocked && self.selected_slot == Some(target.slot) {
            self.selected_slot = None;
        }
        self.clamp_scroll();
        Ok(())
    }

    /// Applies a proven state response atomically. Any invalid slot rejects
    /// the entire batch; no partial state is retained.
    pub fn apply_state_snapshot(
        &mut self,
        updates: &[BuddyStateUpdate],
    ) -> Result<Vec<BuddyUiNotice>, BuddyUiError> {
        for update in updates {
            self.ensure_slot(update.slot)?;
        }
        let mut notices = Vec::new();
        let mut changed = false;
        for update in updates {
            let Some(entry) = self.slots[update.slot].as_mut() else {
                continue;
            };
            let presence = BuddyPresence::from_legacy(update.legacy_state);
            if entry.presence != presence {
                changed = true;
                if !entry.blocked {
                    notices.push(BuddyUiNotice::PresenceChanged {
                        target: BuddyTarget {
                            slot: update.slot,
                            pc_uid: entry.pc_uid,
                        },
                        display_name: entry.display_name(),
                        presence,
                    });
                }
            }
            entry.runtime_pc_id = update.runtime_pc_id;
            entry.presence = presence;
        }
        if changed {
            self.selected_slot = None;
        }
        Ok(notices)
    }

    pub fn confirm_removed(&mut self, target: BuddyTarget) -> Result<BuddyEntry, BuddyUiError> {
        self.validate_target(target)?;
        self.selected_slot = self.selected_slot.filter(|slot| *slot != target.slot);
        let removed = self.slots[target.slot]
            .take()
            .ok_or(BuddyUiError::EmptySlot(target.slot))?;
        self.clamp_scroll();
        Ok(removed)
    }

    pub fn select_slot(&mut self, slot: usize) -> Result<(), BuddyUiError> {
        self.ensure_slot(slot)?;
        let entry = self.slots[slot]
            .as_ref()
            .ok_or(BuddyUiError::EmptySlot(slot))?;
        if entry.pc_uid == 0 || entry.blocked {
            return Err(BuddyUiError::EmptySlot(slot));
        }
        self.selected_slot = Some(slot);
        Ok(())
    }

    pub fn select_visible_row(&mut self, visible_index: usize) -> Result<(), BuddyUiError> {
        let slot = self
            .visible_slot_indices()
            .nth(visible_index)
            .ok_or(BuddyUiError::NoSelection)?;
        self.selected_slot = Some(slot);
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selected_slot = None;
    }

    pub fn set_player_moving(&mut self, moving: bool) {
        self.player_moving = moving;
    }

    pub fn set_group<I>(&mut self, group_size: usize, member_pc_uids: I)
    where
        I: IntoIterator<Item = i64>,
    {
        self.group_size = group_size;
        self.group_member_uids.clear();
        self.group_member_uids.extend(member_pc_uids);
    }

    pub fn set_social_buddy_enabled(&mut self, enabled: bool) {
        self.social_buddy_enabled = enabled;
    }

    #[must_use]
    pub const fn social_buddy_enabled(&self) -> bool {
        self.social_buddy_enabled
    }

    #[must_use]
    pub fn is_blocked_runtime_pc_id(&self, runtime_pc_id: i32) -> bool {
        self.slots.iter().any(|slot| {
            slot.as_ref()
                .is_some_and(|entry| entry.runtime_pc_id == runtime_pc_id && entry.blocked)
        })
    }

    #[must_use]
    pub const fn warp_cooldown_seconds(&self) -> u32 {
        self.warp_cooldown_seconds
    }

    pub fn set_warp_cooldown_seconds(&mut self, seconds: u32) {
        self.warp_cooldown_seconds = seconds;
        self.cooldown_elapsed = 0.0;
    }

    pub fn reset_refresh_timer(&mut self) {
        self.refresh_elapsed = 0.0;
    }

    pub fn advance_time(&mut self, delta_seconds: f32) -> Vec<BuddyUiAction> {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return Vec::new();
        }
        let mut actions = Vec::new();
        self.refresh_elapsed += delta_seconds;
        if self.visible
            && self.occupied_count() > 0
            && self.refresh_elapsed > BUDDY_STATE_REFRESH_SECONDS
        {
            self.refresh_elapsed = 0.0;
            actions.push(BuddyUiAction::RefreshStatesRequested);
        }

        if self.warp_cooldown_seconds > 0 {
            self.cooldown_elapsed += delta_seconds;
            while self.warp_cooldown_seconds > 0 && self.cooldown_elapsed > 1.0 {
                self.warp_cooldown_seconds -= 1;
                self.cooldown_elapsed -= 1.0;
            }
        }
        actions
    }

    pub fn scroll_by_legacy_axis(&mut self, axis: f32) {
        if axis.is_finite() {
            self.scroll_y -= axis.clamp(-1.0, 1.0) * BUDDY_SCROLL_VELOCITY;
            self.clamp_scroll();
        }
    }

    #[must_use]
    pub const fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    pub fn set_scroll_y(&mut self, scroll_y: f32) {
        if scroll_y.is_finite() {
            self.scroll_y = scroll_y;
            self.clamp_scroll();
        }
    }

    pub fn open_add_dialog(&mut self) {
        self.add_dialog_open = true;
        self.add_name_input.clear();
    }

    pub fn cancel_add_dialog(&mut self) {
        self.add_dialog_open = false;
        self.add_name_input.clear();
    }

    #[must_use]
    pub const fn add_dialog_open(&self) -> bool {
        self.add_dialog_open
    }

    #[must_use]
    pub fn add_name_input(&self) -> &str {
        &self.add_name_input
    }

    pub fn set_add_name_input(&mut self, value: impl AsRef<str>) {
        self.add_name_input = value
            .as_ref()
            .chars()
            .filter(|character| *character != '\n' && *character != '\r')
            .take(32)
            .collect();
    }

    pub fn push_add_name_text(&mut self, value: &str) {
        let remaining = 32usize.saturating_sub(self.add_name_input.chars().count());
        self.add_name_input.extend(
            value
                .chars()
                .filter(|character| !character.is_control())
                .take(remaining),
        );
    }

    pub fn pop_add_name_character(&mut self) {
        self.add_name_input.pop();
    }

    pub fn submit_add_dialog(&mut self) -> BuddyUiAction {
        let input = std::mem::take(&mut self.add_name_input);
        self.add_dialog_open = false;
        let Some(space) = input.find(' ') else {
            return BuddyUiAction::AddNameRejected {
                message: BUDDY_ADD_NAME_ERROR.to_owned(),
            };
        };
        let (first_name, tail) = input.split_at(space);
        let last_name = &tail[1..];
        // The legacy code dereferences Last[0] and crashes on a trailing
        // separator. Native ingestion closes that edge without emitting an
        // incomplete network intent.
        if first_name.is_empty() || last_name.is_empty() {
            return BuddyUiAction::AddNameRejected {
                message: BUDDY_ADD_NAME_ERROR.to_owned(),
            };
        }
        BuddyUiAction::AddByNameRequested {
            first_name: first_name.to_owned(),
            last_name: last_name.to_owned(),
        }
    }

    pub fn request_remove(&self) -> Result<BuddyUiAction, BuddyUiError> {
        Ok(BuddyUiAction::ConfirmationRequested(
            BuddyConfirmation::Remove(self.require_selected_target()?),
        ))
    }

    pub fn request_warp(&self) -> Result<BuddyUiAction, BuddyUiError> {
        let target = self.require_selected_target()?;
        let entry = self
            .slot(target.slot)
            .ok_or(BuddyUiError::EmptySlot(target.slot))?;
        if !entry.presence.is_online() {
            return Err(BuddyUiError::SelectedBuddyOffline);
        }
        if self.player_moving {
            return Err(BuddyUiError::PlayerMoving);
        }
        if self.warp_cooldown_seconds > 0 {
            return Ok(BuddyUiAction::WarpCooldownNotice {
                remaining_seconds: self.warp_cooldown_seconds,
            });
        }
        Ok(BuddyUiAction::ConfirmationRequested(
            BuddyConfirmation::Warp(target),
        ))
    }

    pub fn resolve_confirmation(
        &mut self,
        confirmation: BuddyConfirmation,
        accepted: bool,
    ) -> Result<Option<BuddyUiAction>, BuddyUiError> {
        if !accepted {
            return Ok(None);
        }
        let target = match confirmation {
            BuddyConfirmation::Remove(target)
            | BuddyConfirmation::Warp(target)
            | BuddyConfirmation::LeaveGroupForWarp(target) => target,
        };
        if self.selected_target() != Some(target) || self.validate_target(target).is_err() {
            return Err(BuddyUiError::StaleConfirmation(confirmation));
        }

        match confirmation {
            BuddyConfirmation::Remove(target) => {
                self.selected_slot = None;
                Ok(Some(BuddyUiAction::RemoveRequested(target)))
            }
            BuddyConfirmation::Warp(target) => {
                if self.group_size > 1 && !self.group_member_uids.contains(&target.pc_uid) {
                    Ok(Some(BuddyUiAction::ConfirmationRequested(
                        BuddyConfirmation::LeaveGroupForWarp(target),
                    )))
                } else {
                    self.begin_warp(target, false).map(Some)
                }
            }
            BuddyConfirmation::LeaveGroupForWarp(target) => self.begin_warp(target, true).map(Some),
        }
    }

    pub fn receive_invite(&mut self, invite: BuddyInvite) -> BuddyInviteDisposition {
        if self.slots.iter().any(|slot| {
            slot.as_ref()
                .is_some_and(|entry| entry.pc_uid == invite.requester_pc_uid && entry.blocked)
        }) {
            return BuddyInviteDisposition::IgnoredBlocked;
        }
        if !self.social_buddy_enabled {
            return BuddyInviteDisposition::AutoDeclined(invite_response(&invite, false));
        }
        self.pending_invites.push_back(invite);
        BuddyInviteDisposition::Queued
    }

    #[must_use]
    pub fn current_invite(&self) -> Option<&BuddyInvite> {
        self.pending_invites.front()
    }

    pub fn respond_to_invite(
        &mut self,
        invite_id: u64,
        accepted: bool,
    ) -> Result<BuddyUiAction, BuddyUiError> {
        let Some(current) = self.pending_invites.front() else {
            return Err(BuddyUiError::InviteNotCurrent(invite_id));
        };
        if current.invite_id != invite_id {
            return Err(BuddyUiError::InviteNotCurrent(invite_id));
        }
        let invite = self
            .pending_invites
            .pop_front()
            .expect("front invite was checked above");
        Ok(invite_response(&invite, accepted))
    }

    pub(super) fn begin_warp(
        &mut self,
        target: BuddyTarget,
        leave_group: bool,
    ) -> Result<BuddyUiAction, BuddyUiError> {
        self.validate_target(target)?;
        self.warp_cooldown_seconds = BUDDY_WARP_COOLDOWN_SECONDS;
        self.cooldown_elapsed = 0.0;
        Ok(BuddyUiAction::WarpRequested {
            target,
            leave_group,
        })
    }

    pub(super) fn require_selected_target(&self) -> Result<BuddyTarget, BuddyUiError> {
        self.selected_target().ok_or(BuddyUiError::NoSelection)
    }

    pub(super) fn target_for_slot(&self, slot: usize) -> Result<BuddyTarget, BuddyUiError> {
        self.ensure_slot(slot)?;
        let entry = self.slots[slot]
            .as_ref()
            .ok_or(BuddyUiError::EmptySlot(slot))?;
        Ok(BuddyTarget {
            slot,
            pc_uid: entry.pc_uid,
        })
    }

    pub(super) fn validate_target(&self, target: BuddyTarget) -> Result<(), BuddyUiError> {
        self.ensure_slot(target.slot)?;
        let entry = self.slots[target.slot]
            .as_ref()
            .ok_or(BuddyUiError::EmptySlot(target.slot))?;
        if entry.pc_uid != target.pc_uid {
            return Err(BuddyUiError::TargetMismatch {
                slot: target.slot,
                expected_pc_uid: target.pc_uid,
                actual_pc_uid: entry.pc_uid,
            });
        }
        Ok(())
    }

    pub(super) fn entry_for_target_mut(
        &mut self,
        target: BuddyTarget,
    ) -> Result<&mut BuddyEntry, BuddyUiError> {
        self.ensure_slot(target.slot)?;
        let entry = self.slots[target.slot]
            .as_mut()
            .ok_or(BuddyUiError::EmptySlot(target.slot))?;
        if entry.pc_uid != target.pc_uid {
            return Err(BuddyUiError::TargetMismatch {
                slot: target.slot,
                expected_pc_uid: target.pc_uid,
                actual_pc_uid: entry.pc_uid,
            });
        }
        Ok(entry)
    }

    pub(super) fn ensure_slot(&self, slot: usize) -> Result<(), BuddyUiError> {
        if slot >= BUDDY_MAX_SLOTS {
            Err(BuddyUiError::SlotOutOfRange(slot))
        } else {
            Ok(())
        }
    }

    pub(super) fn visible_slot_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.slots.iter().enumerate().filter_map(|(slot, entry)| {
            entry
                .as_ref()
                .is_some_and(|entry| entry.pc_uid != 0 && !entry.blocked)
                .then_some(slot)
        })
    }

    pub(super) fn clamp_scroll(&mut self) {
        let maximum = (self.visible_count() as f32 * BUDDY_LIST_ROW_STEP
            - BUDDY_INNER_LIST_RECT.height)
            .max(0.0);
        self.scroll_y = self.scroll_y.clamp(0.0, maximum);
    }
}
