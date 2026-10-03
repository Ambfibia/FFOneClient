use super::*;

pub fn encode_enchant_request_0104(
    request: EnchantRequest0104,
) -> Result<RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104> {
    RegisteredGameplayRequest0104::new(ENCHANT_REQUEST_PACKET_ID_0104, request.encode().to_vec())
}

pub fn encode_enchant_delete_request_0104(
    request: PcItemDeleteRequest0104,
) -> Result<RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104> {
    RegisteredGameplayRequest0104::new(ENCHANT_DELETE_REQUEST_PACKET_ID_0104, request.encode())
}

pub fn encode_enchant_redeem_request_0104(
    wire: &EnchantRedeemWire0104,
) -> Result<RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104> {
    RegisteredGameplayRequest0104::new(wire.packet_id, wire.payload.to_vec())
}

#[must_use]
pub fn decode_enchant_gameplay_frame_0104(frame: DecodedFrame) -> EnchantGameplayFrame0104 {
    let expected = match frame.packet_type {
        ENCHANT_SUCCESS_PACKET_ID_0104 => ENCHANT_SUCCESS_PACKET_SIZE_0104,
        ENCHANT_FAILURE_PACKET_ID_0104 => ENCHANT_FAILURE_PACKET_SIZE_0104,
        ENCHANT_DELETE_SUCCESS_PACKET_ID_0104 => PcItemDeleteSuccess0104::SIZE,
        ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104 => PcDisassembleItemSuccess0104::SIZE,
        ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104 => PcDisassembleItemFailure0104::SIZE,
        _ => return EnchantGameplayFrame0104::Passthrough(frame),
    };
    if frame.payload.len() != expected {
        return EnchantGameplayFrame0104::Malformed {
            error: PayloadError::WrongSize {
                expected,
                actual: frame.payload.len(),
            },
            frame,
        };
    }
    let packet = match frame.packet_type {
        ENCHANT_SUCCESS_PACKET_ID_0104 => match EnchantSuccess0104::decode(&frame.payload) {
            Ok(packet) => EnchantReplyPacket0104::Success(packet),
            Err(_) => unreachable!("EnchantSuccess exact size is its only decode invariant"),
        },
        ENCHANT_FAILURE_PACKET_ID_0104 => match EnchantFailure0104::decode(&frame.payload) {
            Ok(packet) => EnchantReplyPacket0104::Failure(packet),
            Err(_) => unreachable!("EnchantFailure exact size is its only decode invariant"),
        },
        ENCHANT_DELETE_SUCCESS_PACKET_ID_0104 => EnchantReplyPacket0104::DeleteSuccess(
            PcItemDeleteSuccess0104::decode(&frame.payload)
                .expect("delete success exact size was checked"),
        ),
        ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104 => EnchantReplyPacket0104::DisassembleSuccess(
            PcDisassembleItemSuccess0104::decode(&frame.payload)
                .expect("disassemble success exact size was checked"),
        ),
        ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104 => EnchantReplyPacket0104::DisassembleFailure(
            PcDisassembleItemFailure0104::decode(&frame.payload)
                .expect("disassemble failure exact size was checked"),
        ),
        _ => unreachable!("known packet ID was selected above"),
    };
    EnchantGameplayFrame0104::Decoded { frame, packet }
}
