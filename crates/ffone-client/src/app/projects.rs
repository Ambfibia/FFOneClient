use super::*;

#[derive(Debug, Default, Resource)]
pub(super) struct NetworkLifecycleSession {
    pub(super) next_epoch: u64,
    pub(super) active: Option<NetworkSessionEpoch0104>,
}

impl NetworkLifecycleSession {
    pub(super) fn begin(&mut self) -> NetworkSessionEpoch0104 {
        self.next_epoch = self.next_epoch.saturating_add(1).max(1);
        let epoch = NetworkSessionEpoch0104(self.next_epoch);
        self.active = Some(epoch);
        epoch
    }

    pub(super) fn take(&mut self) -> Option<NetworkSessionEpoch0104> {
        self.active.take()
    }
}
