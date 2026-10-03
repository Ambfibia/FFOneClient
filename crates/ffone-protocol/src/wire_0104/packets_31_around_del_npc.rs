// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_AROUND_DEL_NPC` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000011`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AroundDelNpc0104 {
    /// `iNPCCnt` at offset 0.
    pub npc_cnt: i32,
}

impl AroundDelNpc0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for AroundDelNpc0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_SEND_FREECHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 264 bytes, packet ID `0x31000012`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendFreechatMessageSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
}

impl SendFreechatMessageSuccess0104 {
    pub const SIZE: usize = 264;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
        }
    }
}

impl WirePayload for SendFreechatMessageSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_SEND_FREECHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 264 bytes, packet ID `0x31000013`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendFreechatMessageFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
}

impl SendFreechatMessageFailure0104 {
    pub const SIZE: usize = 264;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
        }
    }
}

impl WirePayload for SendFreechatMessageFailure0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_PC_ATTACK_NPCs_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000014`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAttackNpcsSuccess0104 {
    /// `iBatteryW` at offset 0.
    pub battery_w: i32,
    /// `iNPCCnt` at offset 4.
    pub npc_cnt: i32,
}

impl PcAttackNpcsSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.battery_w.to_le_bytes());
        write_prim(out, 4, &self.npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            battery_w: i32::from_le_bytes(read_array(bytes, 0)),
            npc_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcAttackNpcsSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_PC_ATTACK_NPCs` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000015`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAttackNpcs0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iNPCCnt` at offset 4.
    pub npc_cnt: i32,
}

impl PcAttackNpcs0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            npc_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcAttackNpcs0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NPC_ATTACK_PCs` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000016`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcAttackPcs0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iPCCnt` at offset 4.
    pub pc_cnt: i32,
}

impl NpcAttackPcs0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.pc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            pc_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NpcAttackPcs0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_REGEN_SUCC` (`#pragma pack(4)`, 48 bytes, packet ID `0x31000017`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegenSuccess0104 {
    /// `PCRegenData` at offset 0.
    pub pc_regen_data: PcRegenData0104,
    /// `bMoveLocation` at offset 40.
    pub move_location: i32,
    /// `iFusionMatter` at offset 44.
    pub fusion_matter: i32,
}

impl PcRegenSuccess0104 {
    pub const SIZE: usize = 48;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.pc_regen_data.write_into(&mut out[0..40]);
        write_prim(out, 40, &self.move_location.to_le_bytes());
        write_prim(out, 44, &self.fusion_matter.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_regen_data: PcRegenData0104::read_from(&bytes[0..40]),
            move_location: i32::from_le_bytes(read_array(bytes, 40)),
            fusion_matter: i32::from_le_bytes(read_array(bytes, 44)),
        }
    }
}

impl WirePayload for PcRegenSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 264 bytes, packet ID `0x31000018`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendMenuchatMessageSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
}

impl SendMenuchatMessageSuccess0104 {
    pub const SIZE: usize = 264;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
        }
    }
}

impl WirePayload for SendMenuchatMessageSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_SEND_MENUCHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 264 bytes, packet ID `0x31000019`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendMenuchatMessageFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
}

impl SendMenuchatMessageFailure0104 {
    pub const SIZE: usize = 264;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
        }
    }
}

impl WirePayload for SendMenuchatMessageFailure0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_PC_ITEM_MOVE_SUCC` (`#pragma pack(4)`, 40 bytes, packet ID `0x3100001a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemMoveSuccess0104 {
    /// `eFrom` at offset 0.
    pub from: i32,
    /// `iFromSlotNum` at offset 4.
    pub from_slot_num: i32,
    /// `FromSlotItem` at offset 8.
    pub from_slot_item: ItemBase0104,
    /// `eTo` at offset 20.
    pub to: i32,
    /// `iToSlotNum` at offset 24.
    pub to_slot_num: i32,
    /// `ToSlotItem` at offset 28.
    pub to_slot_item: ItemBase0104,
}

impl PcItemMoveSuccess0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.from.to_le_bytes());
        write_prim(out, 4, &self.from_slot_num.to_le_bytes());
        self.from_slot_item.write_into(&mut out[8..20]);
        write_prim(out, 20, &self.to.to_le_bytes());
        write_prim(out, 24, &self.to_slot_num.to_le_bytes());
        self.to_slot_item.write_into(&mut out[28..40]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            from: i32::from_le_bytes(read_array(bytes, 0)),
            from_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            from_slot_item: ItemBase0104::read_from(&bytes[8..20]),
            to: i32::from_le_bytes(read_array(bytes, 20)),
            to_slot_num: i32::from_le_bytes(read_array(bytes, 24)),
            to_slot_item: ItemBase0104::read_from(&bytes[28..40]),
        }
    }
}

impl WirePayload for PcItemMoveSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_PC_EQUIP_CHANGE` (`#pragma pack(4)`, 20 bytes, packet ID `0x3100001b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcEquipChange0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iEquipSlotNum` at offset 4.
    pub equip_slot_num: i32,
    /// `EquipSlotItem` at offset 8.
    pub equip_slot_item: ItemBase0104,
}

impl PcEquipChange0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.equip_slot_num.to_le_bytes());
        self.equip_slot_item.write_into(&mut out[8..20]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            equip_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            equip_slot_item: ItemBase0104::read_from(&bytes[8..20]),
        }
    }
}

impl WirePayload for PcEquipChange0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_START_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100001c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskStartSuccess0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
    /// `iRemainTime` at offset 4.
    pub remain_time: i32,
}

impl PcTaskStartSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
        write_prim(out, 4, &self.remain_time.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
            remain_time: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcTaskStartSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_START_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100001d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskStartFailure0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcTaskStartFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcTaskStartFailure0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_END_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100001e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskEndSuccess0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
}

impl PcTaskEndSuccess0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcTaskEndSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_END_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100001f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskEndFailure0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcTaskEndFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcTaskEndFailure0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NPC_SKILL_READY` (`#pragma pack(4)`, 24 bytes, packet ID `0x31000020`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// OpenFusion divergence: the pinned `structs/0104.hpp` declares this struct as 20 bytes. Wiring code must choose the server-compatible length explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSkillReady0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i16,
    /// `iValue1` at offset 8.
    pub value1: i32,
    /// `iValue2` at offset 12.
    pub value2: i32,
    /// `iValue3` at offset 16.
    pub value3: i32,
    /// `iValue4` at offset 20.
    pub value4: i32,
}

impl NpcSkillReady0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.value1.to_le_bytes());
        write_prim(out, 12, &self.value2.to_le_bytes());
        write_prim(out, 16, &self.value3.to_le_bytes());
        write_prim(out, 20, &self.value4.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 4)),
            value1: i32::from_le_bytes(read_array(bytes, 8)),
            value2: i32::from_le_bytes(read_array(bytes, 12)),
            value3: i32::from_le_bytes(read_array(bytes, 16)),
            value4: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for NpcSkillReady0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NPC_SKILL_FIRE` (`#pragma pack(4)`, 20 bytes, packet ID `0x31000021`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSkillFire0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i16,
    /// `iVX` at offset 8.
    pub vx: i32,
    /// `iVY` at offset 12.
    pub vy: i32,
    /// `iVZ` at offset 16.
    pub vz: i32,
}

impl NpcSkillFire0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.vx.to_le_bytes());
        write_prim(out, 12, &self.vy.to_le_bytes());
        write_prim(out, 16, &self.vz.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 4)),
            vx: i32::from_le_bytes(read_array(bytes, 8)),
            vy: i32::from_le_bytes(read_array(bytes, 12)),
            vz: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for NpcSkillFire0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}
