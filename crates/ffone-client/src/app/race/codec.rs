use super::*;

pub(in super::super) const RACE_INSTANCE_MAP_INFO_PACKET_ID: u32 = 0x3100_009a;

pub(in super::super) const RACE_GET_RING_SUCCESS_PACKET_ID: u32 = 0x3100_00aa;

pub(in super::super) const RACE_GET_RING_FAILURE_PACKET_ID: u32 = 0x3100_00ab;

pub(in super::super) fn decode_race_instance_map_info_0104(payload: &[u8]) -> Result<RaceInstanceMapInfo0104, String> {
    if payload.len() < RACE_INSTANCE_MAP_INFO_BASE_SIZE {
        return Err(format!(
            "INSTANCE_MAP_INFO requires at least {RACE_INSTANCE_MAP_INFO_BASE_SIZE} bytes, got {}",
            payload.len()
        ));
    }
    let switch_count = read_race_i32(payload, 56);
    let switch_count_usize = usize::try_from(switch_count)
        .map_err(|_| format!("INSTANCE_MAP_INFO has negative switch count {switch_count}"))?;
    let switch_bytes = switch_count_usize
        .checked_mul(RACE_INSTANCE_MAP_SWITCH_SIZE)
        .ok_or_else(|| "INSTANCE_MAP_INFO switch array overflows usize".to_owned())?;
    let expected = RACE_INSTANCE_MAP_INFO_BASE_SIZE
        .checked_add(switch_bytes)
        .ok_or_else(|| "INSTANCE_MAP_INFO payload size overflows usize".to_owned())?;
    if payload.len() != expected {
        return Err(format!(
            "INSTANCE_MAP_INFO with {switch_count} switches requires {expected} bytes, got {}",
            payload.len()
        ));
    }
    Ok(RaceInstanceMapInfo0104 {
        bounds: [read_race_i32(payload,12), read_race_i32(payload,16), read_race_i32(payload,20), read_race_i32(payload,24)],
        ep_id: read_race_i32(payload, 36),
        top_record: ffone_client::race_ui::mode::RaceTopRecord {
            score: read_race_i32(payload, 40),
            rank: read_race_i32(payload, 44),
            time_seconds: read_race_i32(payload, 48),
            rings: read_race_i32(payload, 52),
        },
        switch_count,
    })
}

pub(in super::super) fn decode_race_ring_success_0104(payload: &[u8]) -> Result<(i32, i32), String> {
    if payload.len() != 8 {
        return Err(format!(
            "EP_GET_RING_SUCC requires 8 bytes, got {}",
            payload.len()
        ));
    }
    Ok((read_race_i32(payload, 0), read_race_i32(payload, 4)))
}

pub(in super::super) fn decode_race_ring_failure_0104(payload: &[u8]) -> Result<i32, String> {
    if payload.len() != 4 {
        return Err(format!(
            "EP_GET_RING_FAIL requires 4 bytes, got {}",
            payload.len()
        ));
    }
    Ok(read_race_i32(payload, 0))
}

#[must_use]
pub(in super::super) const fn race_frame_owned(packet_id: u32) -> bool {
    matches!(
        packet_id,
        RACE_INSTANCE_MAP_INFO_PACKET_ID
            | RACE_START_SUCCESS_PACKET_ID
            | RACE_START_FAILURE_PACKET_ID
            | RACE_END_SUCCESS_PACKET_ID
            | RACE_END_FAILURE_PACKET_ID
            | RACE_CANCEL_SUCCESS_PACKET_ID
            | RACE_CANCEL_FAILURE_PACKET_ID
            | RACE_GET_RING_SUCCESS_PACKET_ID
            | RACE_GET_RING_FAILURE_PACKET_ID
    )
}

#[derive(Debug, Default, Resource)]
pub(in super::super) struct RaceNetworkFrameInbox(pub(in super::super) VecDeque<DecodedFrame>);

impl RaceNetworkFrameInbox {
    pub(in super::super) fn push_if_owned(&mut self, frame: DecodedFrame) -> bool {
        if !race_frame_owned(frame.packet_type) {
            return false;
        }
        self.0.push_back(frame);
        true
    }

    pub(in super::super) fn pop(&mut self) -> Option<DecodedFrame> {
        self.0.pop_front()
    }

    pub(in super::super) fn clear(&mut self) {
        self.0.clear();
    }
}
