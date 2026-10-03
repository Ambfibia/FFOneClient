use super::*;

/// Exact clean-Retrobution protocol-0104
/// `sP_FE2CL_REP_PC_NANO_CREATE_FAIL` (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcNanoCreateFailure0104 {
    pub pc_id: i32,
    pub error_code: i32,
}

impl WirePayload for PcNanoCreateFailure0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            error_code: read_i32(bytes, 4),
        })
    }
}

/// Strict clean packet family used to trigger the first pending free tune.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcNanoCreatePacket0104 {
    Success(PcNanoCreateSuccess0104),
    Failure(PcNanoCreateFailure0104),
}

/// Protocol-0104 `sPCAppearanceData` (232 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAppearance0104 {
    pub id: i32,
    pub style: PcStyle0104,
    pub condition_bit_flag: i32,
    pub pc_state: i8,
    pub special_state: i8,
    pub level: i16,
    pub hp: i32,
    pub map_number: i32,
    pub position: [i32; 3],
    pub angle: i32,
    pub equipment: [ItemBase0104; 9],
    pub nano: Nano0104,
    pub render_type: i32,
}

impl PcAppearance0104 {
    pub const SIZE: usize = 232;

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.id);
        self.style.encode_into(&mut out[4..80]);
        write_i32(&mut out, 80, self.condition_bit_flag);
        out[84] = self.pc_state as u8;
        out[85] = self.special_state as u8;
        out[86..88].copy_from_slice(&self.level.to_le_bytes());
        write_i32(&mut out, 88, self.hp);
        write_i32(&mut out, 92, self.map_number);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 96 + index * 4, value);
        }
        write_i32(&mut out, 108, self.angle);
        for (index, item) in self.equipment.iter().copied().enumerate() {
            let start = 112 + index * ItemBase0104::SIZE;
            item.encode_into(&mut out[start..start + ItemBase0104::SIZE]);
        }
        self.nano.encode_into(&mut out[220..226]);
        write_i32(&mut out, 228, self.render_type);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: read_i32(bytes, 0),
            style: PcStyle0104::decode_exact(&bytes[4..80]),
            condition_bit_flag: read_i32(bytes, 80),
            pc_state: bytes[84] as i8,
            special_state: bytes[85] as i8,
            level: read_i16(bytes, 86),
            hp: read_i32(bytes, 88),
            map_number: read_i32(bytes, 92),
            position: [
                read_i32(bytes, 96),
                read_i32(bytes, 100),
                read_i32(bytes, 104),
            ],
            angle: read_i32(bytes, 108),
            equipment: std::array::from_fn(|index| {
                let start = 112 + index * ItemBase0104::SIZE;
                ItemBase0104::decode_exact(&bytes[start..start + ItemBase0104::SIZE])
            }),
            nano: Nano0104::decode_exact(&bytes[220..226]),
            render_type: read_i32(bytes, 228),
        }
    }
}

/// Protocol-0104 `sP_FE2CL_PC_NEW` (`#pragma pack(4)`, 232 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcNew0104 {
    pub appearance: PcAppearance0104,
}

impl WirePayload for PcNew0104 {
    const SIZE: usize = PcAppearance0104::SIZE;

    fn encode(&self) -> Vec<u8> {
        self.appearance.encode()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        Ok(Self {
            appearance: PcAppearance0104::decode(bytes)?,
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_CHANGE_MENTOR`
/// (`#pragma pack(2)`, 2 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcChangeMentorRequest0104 {
    pub mentor: i16,
}

impl WirePayload for PcChangeMentorRequest0104 {
    const SIZE: usize = 2;

    fn encode(&self) -> Vec<u8> {
        self.mentor.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            mentor: read_i16(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_CHANGE_MENTOR_SUCC`
/// (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcChangeMentorSuccess0104 {
    pub mentor: i16,
    pub mentor_count: i16,
    pub fusion_matter: i32,
}

impl WirePayload for PcChangeMentorSuccess0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..2].copy_from_slice(&self.mentor.to_le_bytes());
        out[2..4].copy_from_slice(&self.mentor_count.to_le_bytes());
        write_i32(&mut out, 4, self.fusion_matter);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            mentor: read_i16(bytes, 0),
            mentor_count: read_i16(bytes, 2),
            fusion_matter: read_i32(bytes, 4),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_CHANGE_MENTOR_FAIL`
/// (`#pragma pack(4)`, 8 bytes).
///
/// Bytes 2..4 are alignment padding. Native encoding emits zeroes, while
/// decoding accepts any padding because the clean ABI assigns it no meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcChangeMentorFailure0104 {
    pub mentor: i16,
    pub error_code: i32,
}

impl WirePayload for PcChangeMentorFailure0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..2].copy_from_slice(&self.mentor.to_le_bytes());
        write_i32(&mut out, 4, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            mentor: read_i16(bytes, 0),
            error_code: read_i32(bytes, 4),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_WARP_USE_NPC`
/// (`#pragma pack(4)`, 24 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcWarpUseNpcRequest0104 {
    /// Runtime NPC entity ID, not the TableData/template ID.
    pub npc_id: i32,
    pub warp_id: i32,
    pub e_il_1: i32,
    pub item_slot_1: i32,
    pub e_il_2: i32,
    pub item_slot_2: i32,
}

impl WirePayload for PcWarpUseNpcRequest0104 {
    const SIZE: usize = 24;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.warp_id);
        write_i32(&mut out, 8, self.e_il_1);
        write_i32(&mut out, 12, self.item_slot_1);
        write_i32(&mut out, 16, self.e_il_2);
        write_i32(&mut out, 20, self.item_slot_2);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            warp_id: read_i32(bytes, 4),
            e_il_1: read_i32(bytes, 8),
            item_slot_1: read_i32(bytes, 12),
            e_il_2: read_i32(bytes, 16),
            item_slot_2: read_i32(bytes, 20),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_WARP_USE_NPC_SUCC`
/// (`#pragma pack(4)`, 36 bytes).
///
/// The clean Retrobution managed declaration and OpenFusion `0104.hpp` both
/// prove 36 bytes. Later 0728/1013 revisions are 40 bytes because their
/// `sItemBase` is larger and are not compatible with this crate's 0104 ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcWarpUseNpcSuccess0104 {
    pub position: [i32; 3],
    pub e_il: i32,
    pub item_slot_num: i32,
    pub item: ItemBase0104,
    pub candy: i32,
}

impl WirePayload for PcWarpUseNpcSuccess0104 {
    const SIZE: usize = 36;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.position[0]);
        write_i32(&mut out, 4, self.position[1]);
        write_i32(&mut out, 8, self.position[2]);
        write_i32(&mut out, 12, self.e_il);
        write_i32(&mut out, 16, self.item_slot_num);
        self.item.encode_into(&mut out[20..32]);
        write_i32(&mut out, 32, self.candy);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            position: [read_i32(bytes, 0), read_i32(bytes, 4), read_i32(bytes, 8)],
            e_il: read_i32(bytes, 12),
            item_slot_num: read_i32(bytes, 16),
            item: ItemBase0104::decode_exact(&bytes[20..32]),
            candy: read_i32(bytes, 32),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_WARP_USE_NPC_FAIL`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcWarpUseNpcFailure0104 {
    pub error_code: i32,
}

impl WirePayload for PcWarpUseNpcFailure0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.error_code.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_GOTO_SUCC`
/// (`#pragma pack(4)`, 12 bytes).
///
/// OpenFusion emits this authoritative position after `sendPlayerTo`, including
/// buddy warps, recall skills and GM/server teleports. Some routes also emit
/// `P_FE2CL_REP_PC_WARP_USE_NPC_SUCC` immediately before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcGotoSuccess0104 {
    pub position: [i32; 3],
}

impl WirePayload for PcGotoSuccess0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.position[0]);
        write_i32(&mut out, 4, self.position[1]);
        write_i32(&mut out, 8, self.position[2]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            position: [read_i32(bytes, 0), read_i32(bytes, 4), read_i32(bytes, 8)],
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_TASK_START`
/// (`#pragma pack(4)`, 12 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskStartRequest0104 {
    pub task_id: i32,
    pub npc_id: i32,
    pub escort_npc_id: i32,
}

impl WirePayload for PcTaskStartRequest0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.task_id);
        write_i32(&mut out, 4, self.npc_id);
        write_i32(&mut out, 8, self.escort_npc_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
            npc_id: read_i32(bytes, 4),
            escort_npc_id: read_i32(bytes, 8),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_TASK_END`
/// (`#pragma pack(4)`, 16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskEndRequest0104 {
    pub task_id: i32,
    pub npc_id: i32,
    pub reward_box_1: i8,
    pub reward_box_2: i8,
    pub pack_padding: [u8; 2],
    pub escort_npc_id: i32,
}

impl WirePayload for PcTaskEndRequest0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.task_id);
        write_i32(&mut out, 4, self.npc_id);
        out[8] = self.reward_box_1 as u8;
        out[9] = self.reward_box_2 as u8;
        out[10..12].copy_from_slice(&self.pack_padding);
        write_i32(&mut out, 12, self.escort_npc_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
            npc_id: read_i32(bytes, 4),
            reward_box_1: bytes[8] as i8,
            reward_box_2: bytes[9] as i8,
            pack_padding: bytes[10..12].try_into().expect("fixed two-byte padding"),
            escort_npc_id: read_i32(bytes, 12),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskStartSuccess0104 {
    pub task_id: i32,
    pub remaining_time: i32,
}

impl WirePayload for PcTaskStartSuccess0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.task_id);
        write_i32(&mut out, 4, self.remaining_time);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
            remaining_time: read_i32(bytes, 4),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskFailure0104 {
    pub task_id: i32,
    pub error_code: i32,
}

impl WirePayload for PcTaskFailure0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.task_id);
        write_i32(&mut out, 4, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
            error_code: read_i32(bytes, 4),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskEndSuccess0104 {
    pub task_id: i32,
}

impl WirePayload for PcTaskEndSuccess0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.task_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_KILL_QUEST_NPCs_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcKillQuestNpcsSuccess0104 {
    pub npc_type_id: i32,
}

impl WirePayload for PcKillQuestNpcsSuccess0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.npc_type_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_type_id: read_i32(bytes, 0),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcSetCurrentMissionId0104 {
    pub mission_id: i32,
}

impl WirePayload for PcSetCurrentMissionId0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.mission_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            mission_id: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_TASK_STOP`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskStopRequest0104 {
    pub task_id: i32,
}

impl WirePayload for PcTaskStopRequest0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.task_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_TASK_STOP_SUCC`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskStopSuccess0104 {
    pub task_id: i32,
}

impl WirePayload for PcTaskStopSuccess0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.task_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_TASK_STOP_FAIL`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTaskStopFailure0104 {
    pub error_code: i32,
}

impl WirePayload for PcTaskStopFailure0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.error_code.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sItemReward` trailing a mission/item reward packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemReward0104 {
    pub item: ItemBase0104,
    pub inventory_location: i32,
    pub slot: i32,
}

impl WirePayload for ItemReward0104 {
    const SIZE: usize = 20;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.item.encode_into(&mut out[..ItemBase0104::SIZE]);
        write_i32(&mut out, 12, self.inventory_location);
        write_i32(&mut out, 16, self.slot);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            item: ItemBase0104::decode(&bytes[..ItemBase0104::SIZE])?,
            inventory_location: read_i32(bytes, 12),
            slot: read_i32(bytes, 16),
        })
    }
}

/// Variable protocol-0104 `sP_FE2CL_REP_REWARD_ITEM`. The server-authored
/// post-state in every trailing `sItemReward` is authoritative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardItemReply0104 {
    pub candy: i32,
    pub fusion_matter: i32,
    pub nano_battery: i32,
    pub weapon_battery: i32,
    pub pack_padding: [u8; 3],
    pub fatigue: i32,
    pub fatigue_level: i32,
    pub npc_type_id: i32,
    pub task_id: i32,
    pub items: Vec<ItemReward0104>,
}

impl RewardItemReply0104 {
    pub const HEADER_SIZE: usize = 36;

    pub fn encode(&self) -> Result<Vec<u8>, PayloadError> {
        let item_count =
            i8::try_from(self.items.len()).map_err(|_| PayloadError::ValueOutOfRange {
                field: "reward item count",
                value: i32::try_from(self.items.len()).unwrap_or(i32::MAX),
                minimum: 0,
                maximum: i32::from(i8::MAX),
            })?;
        let mut out = vec![0; Self::HEADER_SIZE + self.items.len() * ItemReward0104::SIZE];
        write_i32(&mut out, 0, self.candy);
        write_i32(&mut out, 4, self.fusion_matter);
        write_i32(&mut out, 8, self.nano_battery);
        write_i32(&mut out, 12, self.weapon_battery);
        out[16] = item_count as u8;
        out[17..20].copy_from_slice(&self.pack_padding);
        write_i32(&mut out, 20, self.fatigue);
        write_i32(&mut out, 24, self.fatigue_level);
        write_i32(&mut out, 28, self.npc_type_id);
        write_i32(&mut out, 32, self.task_id);
        for (index, item) in self.items.iter().enumerate() {
            let offset = Self::HEADER_SIZE + index * ItemReward0104::SIZE;
            out[offset..offset + ItemReward0104::SIZE].copy_from_slice(&item.encode());
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_prefix(bytes, Self::HEADER_SIZE)?;
        let item_count = bytes[16] as i8;
        if item_count < 0 {
            return Err(PayloadError::ValueOutOfRange {
                field: "reward item count",
                value: i32::from(item_count),
                minimum: 0,
                maximum: i32::from(i8::MAX),
            });
        }
        let item_count = item_count as usize;
        let expected = Self::HEADER_SIZE + item_count * ItemReward0104::SIZE;
        require_size(bytes, expected)?;
        let mut items = Vec::with_capacity(item_count);
        for index in 0..item_count {
            let offset = Self::HEADER_SIZE + index * ItemReward0104::SIZE;
            items.push(ItemReward0104::decode(
                &bytes[offset..offset + ItemReward0104::SIZE],
            )?);
        }
        Ok(Self {
            candy: read_i32(bytes, 0),
            fusion_matter: read_i32(bytes, 4),
            nano_battery: read_i32(bytes, 8),
            weapon_battery: read_i32(bytes, 12),
            pack_padding: bytes[17..20].try_into().expect("fixed three-byte padding"),
            fatigue: read_i32(bytes, 20),
            fatigue_level: read_i32(bytes, 24),
            npc_type_id: read_i32(bytes, 28),
            task_id: read_i32(bytes, 32),
            items,
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_NPC_INTERACTION`
/// (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcInteractionRequest0104 {
    /// Runtime NPC entity ID, not the TableData/template ID.
    pub npc_id: i32,
    pub flag: i32,
}

impl WirePayload for NpcInteractionRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.flag);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            flag: read_i32(bytes, 4),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_REGEN` (`#pragma pack(4)`, 12 bytes).
///
/// `e_il` is the clean client's `eIL` field. Retrobution sends zero for XCom
/// and Phoenix choices, and inventory location one for resurrection-item use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRegenRequest0104 {
    pub regen_type: i32,
    pub e_il: i32,
    pub index: i32,
}
