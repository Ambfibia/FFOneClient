use super::*;

pub(super) fn remove_send_commit_action(
    actions: &mut EmailUiOutbox,
    expected: [EmailOutgoingItem; EMAIL_ATTACHMENT_COUNT],
) -> Result<(), EmailProductionError0104> {
    let Some(index) = actions.0.iter().position(
        |action| matches!(action, EmailUiAction::ApplySendSuccessItems(items) if *items == expected),
    ) else {
        return Err(EmailProductionError0104::MissingSendCommitAction);
    };
    actions.0.remove(index);
    Ok(())
}

pub(super) fn request_unreachable<T>(request: &EmailRequest) -> Result<T, EmailProductionError0104> {
    Err(EmailProductionError0104::RequestNotReachable {
        request: request.clone(),
    })
}
