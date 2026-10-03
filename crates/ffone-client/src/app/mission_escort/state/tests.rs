use super::*;

#[test]
fn escort_invites_retry_until_acknowledged_and_forget_owner_on_task_exit() {
    let mut state = EscortRequests::default();
    let epoch = Some(1);
    let tasks = [(576, Some(42))];
    assert!(matches!(
        state.update(epoch, 0.0, &tasks, false).as_slice(),
        [42]
    ));
    assert!(state.update(epoch, 0.9, &tasks, false).is_empty());
    assert!(matches!(
        state.update(epoch, 1.0, &tasks, false).as_slice(),
        [42]
    ));
    assert!(state.update(epoch, 2.0, &tasks, true).is_empty());
    assert!(state.update(epoch, 3.0, &[(576, None)], true).is_empty());
    assert!(state.update(epoch, 4.0, &[], true).is_empty());
    assert!(state.owners.is_empty());
    assert!(state.update(epoch, 5.0, &[], false).is_empty());
}

#[test]
fn escort_late_appearance_and_new_session_never_reuse_an_old_owner() {
    let mut state = EscortRequests::default();
    let epoch = Some(1);
    assert!(state.update(epoch, 0.0, &[(576, None)], false).is_empty());
    assert!(matches!(
        state
            .update(epoch, 0.1, &[(576, Some(42))], false)
            .as_slice(),
        [42]
    ));
    assert!(state.update(None, 0.2, &[], false).is_empty());
    assert!(matches!(
        state
            .update(Some(2), 0.3, &[(576, Some(73))], false)
            .as_slice(),
        [73]
    ));
}
