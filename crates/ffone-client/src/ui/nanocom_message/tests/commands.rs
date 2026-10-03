use super::*;

#[test]
fn timeout_is_strictly_below_zero_and_uses_timeout_outbox_action() {
    let mut model = NanocomMessageUiModel::default();
    model.enqueue(buddy(23));
    model.pop_sound();

    assert_eq!(model.tick(20.0), None);
    assert_eq!(model.len(), 1);
    let action = model.tick(0.001).expect("strict negative timeout");
    assert_eq!(
        action,
        NanocomMessageUiAction {
            request_id: 23,
            kind: NanocomMessageKind::BuddyInvite,
            resolution: NanocomMessageResolution::TimedOut,
        }
    );
    assert!(model.is_empty());
    assert_eq!(model.pop_sound(), Some(NanocomMessageSound::SlideOut));
    assert_eq!(model.pop_sound(), None, "timeout has no NoButton sound");
}
