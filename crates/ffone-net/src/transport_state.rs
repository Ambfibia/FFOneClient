use super::*;

/// Lossless engine-facing classification of authoritative fixed inventory
/// and equipment packets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: InventoryPacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Passthrough(DecodedFrame),
}

impl InventoryGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_inventory_packet_0104(frame.packet_type, &frame.payload) {
            Ok(Some(packet)) => Self::Decoded { frame, packet },
            Ok(None) => Self::Passthrough(frame),
            Err(error) => Self::Malformed { frame, error },
        }
    }

    pub fn raw_frame(&self) -> &DecodedFrame {
        match self {
            Self::Decoded { frame, .. } | Self::Malformed { frame, .. } => frame,
            Self::Passthrough(frame) => frame,
        }
    }
}
