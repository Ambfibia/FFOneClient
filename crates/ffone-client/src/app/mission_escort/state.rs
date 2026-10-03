//! Mission-owned NPC group requests; the server still owns NPC movement.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(crate) struct EscortRequests {
    epoch: Option<u64>,
    // Retain the concrete owner while it is temporarily outside visibility.
    owners: BTreeMap<i32, i32>,
    next_invite: f64,
}

impl EscortRequests {
    pub(crate) fn update(
        &mut self,
        epoch: Option<u64>,
        now: f64,
        tasks: &[(i32, Option<i32>)],
        group_has_npc: bool,
    ) -> Vec<i32> {
        if self.epoch != epoch || epoch.is_none() {
            *self = Self {
                epoch,
                ..Self::default()
            };
        }
        if epoch.is_none() {
            return Vec::new();
        }
        let mut requests = Vec::new();
        self.owners
            .retain(|task, _| tasks.iter().any(|(id, _)| id == task));
        let mut new_owner = false;
        for &(task, npc) in tasks {
            if let Some(npc) = npc {
                new_owner |= self.owners.insert(task, npc) != Some(npc);
            }
        }
        // ProcessStartSucc invites immediately; Update retries once a second
        // only while the authoritative roster has no NPC member.
        if !group_has_npc && (new_owner || now >= self.next_invite) {
            let mut invited = BTreeSet::new();
            for &(_, npc) in tasks {
                if let Some(npc) = npc
                    && invited.insert(npc)
                {
                    requests.push(npc);
                }
            }
            self.next_invite = now + 1.0;
        }
        requests
    }
}

#[cfg(test)]
mod tests;
