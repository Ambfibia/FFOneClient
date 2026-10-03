use super::*;

pub(in super::super) fn apply_correlated_race_reply_0104(
    packet_id: u32,
    payload: &[u8],
    reply_received_local_time: f32,
    model: &mut RaceModeModel,
    production: &mut RaceProductionRuntime,
) -> Result<(), String> {
    let reply = decode_race_reply_0104(packet_id, payload)
        .map_err(|error| format!("RaceMode reply rejected: {error}"))?;
    let request_id = model
        .pending_request_id()
        .ok_or_else(|| format!("RaceMode ignored uncorrelated reply {packet_id:#010x}"))?;
    let end_success = matches!(&reply, RaceModeReply::EndSuccess(_));
    let terminal = matches!(&reply,
        RaceModeReply::CancelSuccess { .. }
            | RaceModeReply::CancelFailure { .. }
            | RaceModeReply::EndSuccess(_)
            | RaceModeReply::EndFailure { .. });
    let start_success = matches!(&reply, RaceModeReply::StartSuccess { .. });
    model
        .apply_reply(
            RaceReplyEnvelope { request_id, reply },
            reply_received_local_time,
        )
        .map_err(|error| {
            format!("RaceMode rejected uncorrelated {packet_id:#010x} reply: {error:?}")
        })?;
    production.player = *model.player();
    if terminal { production.race_ui_complete = true; }
    if start_success { production.race_ui_complete = false; }
    if end_success
        && production
            .source_npc
            .as_ref()
            .is_some_and(|source| !source.voice_owner.is_empty())
    {
        production.pending_npc_voice = Some(PendingRaceNpcVoice::RaceFinished);
        production.finished_voice_npc_id = production.source_npc.as_ref().map(|source| source.runtime_npc_id);
    }
    Ok(())
}
