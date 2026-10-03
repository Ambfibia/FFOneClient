use super::*;

pub fn resolve_guide_change_success(
    model: &mut GuideUiModel,
    outbox: &mut GuideUiOutbox,
    reply: GuideChangeSuccess,
) -> bool {
    resolve_correlated_guide_change_success(model, outbox, reply, reply.mentor_count == 1)
}

/// Applies a success whose first-vs-later interpretation was already
/// correlated by the owning server-profile runtime. The raw mentor count is
/// retained in the emitted action; `first_change` changes only follow-up
/// behavior and never rewrites authoritative reply data.
pub fn resolve_correlated_guide_change_success(
    model: &mut GuideUiModel,
    outbox: &mut GuideUiOutbox,
    reply: GuideChangeSuccess,
    first_change: bool,
) -> bool {
    if !model.awaiting_server || model.pending_mentor != Some(reply.mentor) {
        return false;
    }
    model.current = Some(reply.mentor);
    model.awaiting_server = false;
    model.pending_mentor = None;
    if first_change {
        outbox.push(GuideUiAction::FirstMentorChangeSucceeded {
            mentor: reply.mentor,
            mentor_count: reply.mentor_count,
            fusion_matter: reply.fusion_matter,
            next_mode: GUIDE_FIRST_CHANGE_NEXT_MODE,
            warp_npc_table_id: GUIDE_FIRST_CHANGE_WARP_NPC_TABLE_ID,
        });
    } else {
        outbox.push(GuideUiAction::MentorChangeSucceeded {
            mentor: reply.mentor,
            mentor_count: reply.mentor_count,
            fusion_matter: reply.fusion_matter,
            refresh_guide_missions: true,
        });
    }
    model.close();
    true
}

pub fn resolve_guide_change_failure(
    model: &mut GuideUiModel,
    outbox: &mut GuideUiOutbox,
    mentor: GuideMentor,
    error_code: i32,
) -> bool {
    if !model.awaiting_server || model.pending_mentor != Some(mentor) {
        return false;
    }
    outbox.push(GuideUiAction::MentorChangeFailed {
        mentor,
        error_code,
        system_message_id: GUIDE_CHANGE_FAILURE_MESSAGE_ID,
    });
    model.close();
    true
}
