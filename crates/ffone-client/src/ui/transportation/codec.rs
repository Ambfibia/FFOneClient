use super::*;

pub const TRANSPORTATION_REQUEST_PACKET_ID: u32 = 318_767_209;

pub const TRANSPORTATION_REGISTRATION_REQUEST_PACKET_ID: u32 = 318_767_208;

pub const TRANSPORTATION_REGISTRATION_FAILURE_PACKET_ID: u32 = 822_083_774;

pub const TRANSPORTATION_REGISTRATION_SUCCESS_PACKET_ID: u32 = 822_083_775;

pub const TRANSPORTATION_SUCCESS_PACKET_ID: u32 = 822_083_777;

pub const TRANSPORTATION_FAILURE_PACKET_ID: u32 = 822_083_776;

pub fn decode_transportation_registration_reply_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<TransportationRegistrationReply0104, TransportationReplyCodecError0104> {
    let expected = match packet_id {
        TRANSPORTATION_REGISTRATION_SUCCESS_PACKET_ID => 28,
        TRANSPORTATION_REGISTRATION_FAILURE_PACKET_ID => 12,
        _ => {
            return Err(TransportationReplyCodecError0104::UnexpectedPacketId(
                packet_id,
            ));
        }
    };
    if payload.len() != expected {
        return Err(TransportationReplyCodecError0104::WrongSize {
            packet_id,
            expected,
            actual: payload.len(),
        });
    }
    let read_i32 =
        |offset| i32::from_le_bytes(payload[offset..offset + 4].try_into().expect("four bytes"));
    let read_u64 =
        |offset| u64::from_le_bytes(payload[offset..offset + 8].try_into().expect("eight bytes"));
    Ok(match packet_id {
        TRANSPORTATION_REGISTRATION_SUCCESS_PACKET_ID => {
            TransportationRegistrationReply0104::Success {
                transportation_type: read_i32(0),
                location_id: read_i32(4),
                unlocks: TransportationUnlocks {
                    warp_location_flags: read_i32(8) as u32,
                    wyvern_location_flags: [read_u64(12), read_u64(20)],
                },
            }
        }
        TRANSPORTATION_REGISTRATION_FAILURE_PACKET_ID => {
            TransportationRegistrationReply0104::Failure {
                transportation_type: read_i32(0),
                location_id: read_i32(4),
                error_code: read_i32(8),
            }
        }
        _ => unreachable!("packet id checked above"),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationReplyCodecError0104 {
    UnexpectedPacketId(u32),
    WrongSize {
        packet_id: u32,
        expected: usize,
        actual: usize,
    },
}

pub fn decode_transportation_warp_reply_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<TransportationWarpReply0104, TransportationReplyCodecError0104> {
    let expected = match packet_id {
        TRANSPORTATION_SUCCESS_PACKET_ID => 20,
        TRANSPORTATION_FAILURE_PACKET_ID => 8,
        _ => {
            return Err(TransportationReplyCodecError0104::UnexpectedPacketId(
                packet_id,
            ));
        }
    };
    if payload.len() != expected {
        return Err(TransportationReplyCodecError0104::WrongSize {
            packet_id,
            expected,
            actual: payload.len(),
        });
    }
    let read_i32 =
        |offset| i32::from_le_bytes(payload[offset..offset + 4].try_into().expect("four bytes"));
    Ok(match packet_id {
        TRANSPORTATION_SUCCESS_PACKET_ID => TransportationWarpReply0104::Success {
            transportation_type: read_i32(0),
            position: [read_i32(4), read_i32(8), read_i32(12)],
            candy: read_i32(16),
        },
        TRANSPORTATION_FAILURE_PACKET_ID => TransportationWarpReply0104::Failure {
            transportation_id: read_i32(0),
            error_code: read_i32(4),
        },
        _ => unreachable!("packet id checked above"),
    })
}
