//! A vehicle request is not guaranteed a reply: the shard silently rejects
//! mounting inside an instance. Never turn that silence into a session lock.
use std::time::{Duration, Instant};

const REPLY_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub(crate) struct PendingVehicleRequest {
    mounting: bool,
    map_number: Option<i32>,
    sent_at: Instant,
}

impl PendingVehicleRequest {
    pub(crate) fn new(mounting: bool, map_number: Option<i32>, sent_at: Instant) -> Self {
        Self {
            mounting,
            map_number,
            sent_at,
        }
    }
}

/// Expiration only releases the input lock. It neither resends a packet nor
/// changes the server-confirmed mounted state.
pub(crate) fn expire_pending(
    pending: &mut Option<PendingVehicleRequest>,
    map_number: Option<i32>,
    now: Instant,
) {
    if pending.as_ref().is_some_and(|request| {
        request.map_number != map_number
            || now.saturating_duration_since(request.sent_at) >= REPLY_TIMEOUT
    }) {
        *pending = None;
    }
}

pub(crate) fn finish_pending(pending: &mut Option<PendingVehicleRequest>, mounting: bool) {
    if pending
        .as_ref()
        .is_some_and(|request| request.mounting == mounting)
    {
        *pending = None;
    }
}

#[cfg(test)]
mod tests;
