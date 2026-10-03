use super::*;

/// Protocol-0104 `sP_CL2FE_REQ_GET_BUDDY_STATE`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuddyStateRequest0104 {
    pub unused: u8,
}

impl WirePayload for BuddyStateRequest0104 {
    const SIZE: usize = 1;

    fn encode(&self) -> Vec<u8> {
        vec![self.unused]
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self { unused: bytes[0] })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_GET_BUDDY_STATE_SUCC`.
///
/// The final two bytes are pack(4) tail padding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyStateSuccess0104 {
    pub buddy_ids: [i32; BUDDY_LIST_CAPACITY_0104],
    pub buddy_states: [u8; BUDDY_LIST_CAPACITY_0104],
}

impl WirePayload for BuddyStateSuccess0104 {
    const SIZE: usize = 252;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        for (index, buddy_id) in self.buddy_ids.iter().copied().enumerate() {
            write_i32(&mut out, index * 4, buddy_id);
        }
        out[200..250].copy_from_slice(&self.buddy_states);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            buddy_ids: std::array::from_fn(|index| read_i32(bytes, index * 4)),
            buddy_states: bytes[200..250]
                .try_into()
                .expect("fixed fifty-byte state array"),
        })
    }
}

/// Shared 8-byte `#pragma pack(4)` layout used by
/// `sP_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC` and
/// `sP_FE2CL_PC_SPECIAL_STATE_CHANGE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecialStateChange0104 {
    pub pc_id: i32,
    pub requested_flag: i8,
    pub special_state: i8,
}

impl WirePayload for SpecialStateChange0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        out[4] = self.requested_flag as u8;
        out[5] = self.special_state as u8;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            requested_flag: bytes[4] as i8,
            special_state: bytes[5] as i8,
        })
    }
}

/// Fixed-layout authoritative inventory/equipment packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryPacket0104 {
    ItemMoveSuccess(ItemMoveSuccessPacket0104),
    EquipChange(EquipChangePacket0104),
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH`
/// (`#pragma pack(4)`, 8 bytes).
///
/// Bytes 5..8 are alignment padding. Native encoding emits zeroes and decoding
/// deliberately ignores their contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcSpecialStateSwitchRequest0104 {
    pub pc_id: i32,
    pub special_state_flag: i8,
}

impl WirePayload for PcSpecialStateSwitchRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        out[4] = self.special_state_flag as u8;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            special_state_flag: bytes[4] as i8,
        })
    }
}

/// Both `sP_CL2FE_REQ_PC_COMBAT_BEGIN` and `_END` are this exact four-byte body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcCombatStateRequest0104 {
    pub pc_id: i32,
}

impl WirePayload for PcCombatStateRequest0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.pc_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
        })
    }
}
