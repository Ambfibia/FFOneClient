use super::*;

/// `cnCombiMode.Exit` is local; it sends no gameplay request.
pub const COMBI_NORMAL_EXIT_SENDS_PACKET_0104: bool = false;

pub fn encode_combi_request_0104(
    request: CombiRequest0104,
) -> Result<RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104> {
    let mut payload = vec![0; COMBI_REQUEST_PACKET_SIZE_0104];
    write_i32(
        &mut payload,
        COMBI_REQUEST_COSTUME_SLOT_OFFSET_0104,
        request.costume_item_slot,
    );
    write_i32(
        &mut payload,
        COMBI_REQUEST_STAT_SLOT_OFFSET_0104,
        request.stat_item_slot,
    );
    write_i32(
        &mut payload,
        COMBI_REQUEST_CASH_SLOT_1_OFFSET_0104,
        request.cash_item_slot_1,
    );
    write_i32(
        &mut payload,
        COMBI_REQUEST_CASH_SLOT_2_OFFSET_0104,
        request.cash_item_slot_2,
    );
    RegisteredGameplayRequest0104::new(COMBI_REQUEST_PACKET_ID_0104, payload)
}

#[must_use]
pub fn decode_combi_gameplay_frame_0104(frame: DecodedFrame) -> CombiGameplayFrame0104 {
    let expected = match frame.packet_type {
        COMBI_SUCCESS_PACKET_ID_0104 => COMBI_SUCCESS_PACKET_SIZE_0104,
        COMBI_FAILURE_PACKET_ID_0104 => COMBI_FAILURE_PACKET_SIZE_0104,
        _ => return CombiGameplayFrame0104::Passthrough(frame),
    };
    if frame.payload.len() != expected {
        let actual = frame.payload.len();
        return CombiGameplayFrame0104::Malformed {
            frame,
            error: PayloadError::WrongSize { expected, actual },
        };
    }

    let packet = if frame.packet_type == COMBI_SUCCESS_PACKET_ID_0104 {
        let item_end = COMBI_SUCCESS_NEW_ITEM_OFFSET_0104 + ItemBase0104::SIZE;
        let new_item =
            ItemBase0104::decode(&frame.payload[COMBI_SUCCESS_NEW_ITEM_OFFSET_0104..item_end])
                .expect("success payload and ItemBase sizes were checked");
        CombiReplyPacket0104::Success(CombiSuccessReply0104 {
            new_item_slot: read_i32(&frame.payload, COMBI_SUCCESS_NEW_ITEM_SLOT_OFFSET_0104),
            new_item,
            stat_item_slot: read_i32(&frame.payload, COMBI_SUCCESS_STAT_SLOT_OFFSET_0104),
            cash_item_slot_1: read_i32(&frame.payload, COMBI_SUCCESS_CASH_SLOT_1_OFFSET_0104),
            cash_item_slot_2: read_i32(&frame.payload, COMBI_SUCCESS_CASH_SLOT_2_OFFSET_0104),
            taros_after: read_i32(&frame.payload, COMBI_SUCCESS_TAROS_OFFSET_0104),
            success_flag: read_i32(&frame.payload, COMBI_SUCCESS_FLAG_OFFSET_0104),
        })
    } else {
        CombiReplyPacket0104::Failure(CombiFailureReply0104 {
            error_code: read_i32(&frame.payload, COMBI_FAILURE_ERROR_OFFSET_0104),
            costume_item_slot: read_i32(&frame.payload, COMBI_FAILURE_COSTUME_SLOT_OFFSET_0104),
            stat_item_slot: read_i32(&frame.payload, COMBI_FAILURE_STAT_SLOT_OFFSET_0104),
            cash_item_slot_1: read_i32(&frame.payload, COMBI_FAILURE_CASH_SLOT_1_OFFSET_0104),
            cash_item_slot_2: read_i32(&frame.payload, COMBI_FAILURE_CASH_SLOT_2_OFFSET_0104),
        })
    };
    CombiGameplayFrame0104::Decoded { frame, packet }
}
