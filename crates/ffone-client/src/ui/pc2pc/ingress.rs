use super::*;

/// Decode only exact 0104 trade replies; validate their pair before model mutation.
pub fn decode_pc2pc_session_frame_0104(
    frame: &DecodedFrame,
) -> Result<Option<Pc2pcServerOutcome0104>, String> {
    let size = match frame.packet_type {
        0x31000047..=0x31000049 => 12,
        0x3100004a => 400,
        0x3100004c | 0x3100004e => 44,
        0x3100004b | 0x3100004d | 0x3100004f | 0x31000050 | 0x31000051 => 16,
        _ => return Ok(None),
    };
    if frame.payload.len() != size {
        return Err(format!(
            "Trade reply size {} != {size}",
            frame.payload.len()
        ));
    }
    let bytes = &frame.payload;
    let number = |offset| i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    let envelope = Pc2pcEnvelope0104 {
        requester_pc_id: number(0),
        pair: Pc2pcPair0104::new(number(4), number(8)).map_err(|e| e.to_string())?,
    };
    if envelope.requester_pc_id != envelope.pair.from_pc_id
        && envelope.requester_pc_id != envelope.pair.to_pc_id
    {
        return Err("Trade requester outside pair".to_owned());
    }
    let item = |offset| Pc2pcTradeItem0104 {
        item_type: i16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()),
        item_id: i16::from_le_bytes(bytes[offset + 2..offset + 4].try_into().unwrap()),
        option: number(offset + 4),
        inventory_slot: number(offset + 8),
        offer_slot: number(offset + 12),
    };
    Ok(Some(match frame.packet_type {
        0x31000047 => Pc2pcServerOutcome0104::Confirmed { envelope },
        0x31000048 | 0x31000049 => Pc2pcServerOutcome0104::SessionEnded {
            envelope,
            reason: if frame.packet_type == 0x31000048 {
                Pc2pcSessionEndReason0104::ConfirmCancelled
            } else {
                Pc2pcSessionEndReason0104::ConfirmAborted
            },
            error_code: None,
        },
        0x3100004a => Pc2pcServerOutcome0104::ConfirmSuccess {
            envelope,
            received: array::from_fn(|i| item(12 + i * 16)),
            wallet_taros: number(204),
            item_stay: array::from_fn(|i| item(208 + i * 16)),
        },
        0x3100004b => Pc2pcServerOutcome0104::ConfirmFailure {
            envelope,
            error_code: number(12),
        },
        0x3100004c => Pc2pcServerOutcome0104::RegisterItemSuccess {
            envelope,
            trade_item: item(12),
            inventory_item: item(28),
        },
        0x3100004e => Pc2pcServerOutcome0104::UnregisterItemSuccess {
            envelope,
            trade_item: item(12),
            inventory_item: item(28),
        },
        0x31000050 => Pc2pcServerOutcome0104::RegisterCashSuccess {
            envelope,
            taros: number(12),
        },
        _ => Pc2pcServerOutcome0104::RequestFailure {
            envelope,
            kind: match frame.packet_type {
                0x3100004d => Pc2pcRequestFailureKind0104::RegisterItem,
                0x3100004f => Pc2pcRequestFailureKind0104::UnregisterItem,
                _ => Pc2pcRequestFailureKind0104::RegisterCash,
            },
            error_code: number(12),
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(id: u32, size: usize) -> DecodedFrame {
        let mut payload = vec![0; size];
        for (offset, value) in [(0, 2i32), (4, 1), (8, 2)] {
            if size >= offset + 4 {
                payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        DecodedFrame {
            packet_type: id,
            payload,
            flags: 0,
            checksum: 0,
        }
    }
    #[test]
    fn final_layout_keeps_wallet_between_the_two_item_arrays() {
        let mut packet = frame(0x3100004a, 400);
        packet.payload[204..208].copy_from_slice(&12345i32.to_le_bytes());
        let Some(Pc2pcServerOutcome0104::ConfirmSuccess {
            wallet_taros,
            received,
            item_stay,
            ..
        }) = decode_pc2pc_session_frame_0104(&packet).unwrap()
        else {
            panic!();
        };
        assert_eq!(wallet_taros, 12345);
        assert_eq!(received.len(), 12);
        assert_eq!(item_stay.len(), 12);
    }
    #[test]
    fn malformed_frames_and_unrelated_frames_do_not_change_trade() {
        for (id, size) in [
            (0x31000047, 12),
            (0x3100004a, 400),
            (0x3100004c, 44),
            (0x31000050, 16),
        ] {
            assert!(
                decode_pc2pc_session_frame_0104(&frame(id, size))
                    .unwrap()
                    .is_some()
            );
            assert!(decode_pc2pc_session_frame_0104(&frame(id, size - 1)).is_err());
        }
        assert!(
            decode_pc2pc_session_frame_0104(&frame(123, 0))
                .unwrap()
                .is_none()
        );
        let mut bad = frame(0x31000047, 12);
        bad.payload[0..4].copy_from_slice(&3i32.to_le_bytes());
        assert!(decode_pc2pc_session_frame_0104(&bad).is_err());
    }
}
