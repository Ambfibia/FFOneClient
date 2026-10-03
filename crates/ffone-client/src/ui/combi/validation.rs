use super::*;

pub const COMBI_FAILURE_ERROR_OFFSET_0104: usize = 0;

pub const COMBI_LOOK_ERROR_PATH: &str = "ui/en/combi/look-error.png";

pub const COMBI_STAT_ERROR_PATH: &str = "ui/en/combi/stat-error.png";

pub(super) fn validate_reply_envelope(
    pending: CombiPendingAttempt0104,
    reply: CombiSuccessReply0104,
) -> Result<(), CombiReplyError0104> {
    if reply.new_item_slot != pending.style_slot as i32
        || reply.stat_item_slot != pending.stats_slot as i32
        || reply.cash_item_slot_1 != 0
        || reply.cash_item_slot_2 != 0
    {
        return Err(CombiReplyError0104::MismatchedSuccessEnvelope {
            reply,
            expected_style_slot: pending.style_slot,
            expected_stats_slot: pending.stats_slot,
        });
    }
    Ok(())
}
