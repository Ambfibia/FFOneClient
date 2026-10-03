use super::*;

#[test]
fn silent_rejection_expires_and_a_later_attempt_can_complete() {
    let start = Instant::now();
    let mut pending = Some(PendingVehicleRequest::new(true, Some(10), start));
    expire_pending(
        &mut pending,
        Some(10),
        start + REPLY_TIMEOUT - Duration::from_millis(1),
    );
    assert!(
        pending.is_some(),
        "suppress duplicate input while awaiting a reply"
    );
    expire_pending(&mut pending, Some(10), start + REPLY_TIMEOUT);
    assert!(
        pending.is_none(),
        "a silent rejection cannot lock the session"
    );
    pending = Some(PendingVehicleRequest::new(
        true,
        Some(10),
        start + REPLY_TIMEOUT,
    ));
    finish_pending(&mut pending, true);
    assert!(pending.is_none());
}

#[test]
fn leaving_restricted_map_releases_pending_immediately() {
    let start = Instant::now();
    let mut pending = Some(PendingVehicleRequest::new(true, Some(10), start));
    expire_pending(&mut pending, Some(0), start + Duration::from_millis(1));
    assert!(pending.is_none());
}

#[test]
fn explicit_failure_releases_pending_without_waiting_for_timeout() {
    let mut pending = Some(PendingVehicleRequest::new(true, Some(0), Instant::now()));
    finish_pending(&mut pending, true);
    assert!(pending.is_none());
}

#[test]
fn opposite_reply_does_not_clear_a_new_request() {
    let start = Instant::now();
    let mut pending = Some(PendingVehicleRequest::new(false, Some(0), start));
    finish_pending(&mut pending, true);
    assert!(pending.is_some());
    expire_pending(&mut pending, Some(0), start + REPLY_TIMEOUT);
    assert!(
        pending.is_none(),
        "off requests must also recover from a missing reply"
    );
}
