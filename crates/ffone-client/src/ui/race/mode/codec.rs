use super::*;

pub const RACE_START_REQUEST_PACKET_ID: u32 = 318_767_195;

pub const RACE_END_REQUEST_PACKET_ID: u32 = 318_767_196;

pub const RACE_CANCEL_REQUEST_PACKET_ID: u32 = 318_767_197;

pub const RACE_START_SUCCESS_PACKET_ID: u32 = 822_083_748;

pub const RACE_START_FAILURE_PACKET_ID: u32 = 822_083_749;

pub const RACE_END_SUCCESS_PACKET_ID: u32 = 822_083_750;

pub const RACE_END_FAILURE_PACKET_ID: u32 = 822_083_751;

pub const RACE_CANCEL_SUCCESS_PACKET_ID: u32 = 822_083_752;

pub const RACE_CANCEL_FAILURE_PACKET_ID: u32 = 822_083_753;

/// Routed to `cnRaceMode.ReceivePacket` in the component, but not by the
/// clean `GameFrame` switch. Consequently the inventory-full result copy is
/// dormant in the normal clean runtime.
pub const RACE_DORMANT_INVENTORY_FULL_PACKET_ID: u32 = 822_083_791;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceReplyCodecError0104 {
    UnexpectedPacketId(u32),
    WrongSize {
        packet_id: u32,
        expected: usize,
        actual: usize,
    },
}

impl std::fmt::Display for RaceReplyCodecError0104 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedPacketId(packet_id) => {
                write!(formatter, "unexpected race reply {packet_id:#010x}")
            }
            Self::WrongSize {
                packet_id,
                expected,
                actual,
            } => write!(
                formatter,
                "race reply {packet_id:#010x} must be {expected} bytes, got {actual}"
            ),
        }
    }
}

impl std::error::Error for RaceReplyCodecError0104 {}

/// Strict decoder for the complete OpenFusion race start/end/cancel reply
/// family. No trailing bytes are accepted.
pub fn decode_race_reply_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<RaceModeReply, RaceReplyCodecError0104> {
    let expected = match packet_id {
        RACE_START_SUCCESS_PACKET_ID => RACE_START_SUCCESS_SIZE,
        RACE_END_SUCCESS_PACKET_ID => RACE_END_SUCCESS_SIZE,
        RACE_START_FAILURE_PACKET_ID
        | RACE_END_FAILURE_PACKET_ID
        | RACE_CANCEL_SUCCESS_PACKET_ID
        | RACE_CANCEL_FAILURE_PACKET_ID => 4,
        _ => return Err(RaceReplyCodecError0104::UnexpectedPacketId(packet_id)),
    };
    if payload.len() != expected {
        return Err(RaceReplyCodecError0104::WrongSize {
            packet_id,
            expected,
            actual: payload.len(),
        });
    }
    Ok(match packet_id {
        RACE_START_SUCCESS_PACKET_ID => RaceModeReply::StartSuccess {
            start_tick: read_u64(payload, 0),
            limit_time: read_i32(payload, 8),
        },
        RACE_START_FAILURE_PACKET_ID => RaceModeReply::StartFailure {
            error_code: read_i32(payload, 0),
        },
        RACE_END_SUCCESS_PACKET_ID => RaceModeReply::EndSuccess(RaceEndSuccess {
            race_mode: read_i32(payload, 0),
            race_time_seconds: read_i32(payload, 4),
            ring_count: read_i32(payload, 8),
            score: read_i32(payload, 12),
            rank: read_i32(payload, 16),
            reward_fusion_matter: read_i32(payload, 20),
            top_score: read_i32(payload, 24),
            top_rank: read_i32(payload, 28),
            top_time_seconds: read_i32(payload, 32),
            top_ring_count: read_i32(payload, 36),
            fusion_matter: read_i32(payload, 40),
            reward_item: RaceRewardItem {
                item_type: read_i16(payload, 44),
                item_id: read_i16(payload, 46),
                item_opt: read_i32(payload, 48),
                time_limit: read_i32(payload, 52),
                e_il: read_i32(payload, 56),
                slot: read_i32(payload, 60),
            },
            fatigue: read_i32(payload, 64),
            fatigue_level: read_i32(payload, 68),
        }),
        RACE_END_FAILURE_PACKET_ID => RaceModeReply::EndFailure {
            error_code: read_i32(payload, 0),
        },
        RACE_CANCEL_SUCCESS_PACKET_ID => RaceModeReply::CancelSuccess {
            temporary: read_i32(payload, 0),
        },
        RACE_CANCEL_FAILURE_PACKET_ID => RaceModeReply::CancelFailure {
            error_code: read_i32(payload, 0),
        },
        _ => unreachable!("packet id checked above"),
    })
}
