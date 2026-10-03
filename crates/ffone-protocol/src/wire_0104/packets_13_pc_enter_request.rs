// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_PC_ENTER` (`#pragma pack(4)`, 80 bytes, packet ID `0x13000001`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcEnterRequest0104 {
    /// `szID` at offset 0.
    pub id: FixedUtf16<33>,
    /// `iTempValue` at offset 68.
    pub temp_value: i32,
    /// `iEnterSerialKey` at offset 72.
    pub enter_serial_key: i64,
}

impl PcEnterRequest0104 {
    pub const SIZE: usize = 80;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.id);
        write_prim(out, 68, &self.temp_value.to_le_bytes());
        write_prim(out, 72, &self.enter_serial_key.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: read_utf16(bytes, 0),
            temp_value: i32::from_le_bytes(read_array(bytes, 68)),
            enter_serial_key: i64::from_le_bytes(read_array(bytes, 72)),
        }
    }
}

impl WirePayload for PcEnterRequest0104 {
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

/// `sP_CL2FE_REQ_PC_EXIT` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000002`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcExitRequest0104 {
    /// `iID` at offset 0.
    pub id: i32,
}

impl PcExitRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcExitRequest0104 {
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

/// `sP_CL2FE_REQ_PC_MOVE` (`#pragma pack(4)`, 44 bytes, packet ID `0x13000003`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMoveRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `fVX` at offset 20.
    pub vx: f32,
    /// `fVY` at offset 24.
    pub vy: f32,
    /// `fVZ` at offset 28.
    pub vz: f32,
    /// `iAngle` at offset 32.
    pub angle: i32,
    /// `cKeyValue` at offset 36.
    pub c_key_value: u8,
    /// `iSpeed` at offset 40.
    pub speed: i32,
}

impl PcMoveRequest0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.vx.to_le_bytes());
        write_prim(out, 24, &self.vy.to_le_bytes());
        write_prim(out, 28, &self.vz.to_le_bytes());
        write_prim(out, 32, &self.angle.to_le_bytes());
        write_prim(out, 36, &self.c_key_value.to_le_bytes());
        write_prim(out, 40, &self.speed.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            vx: f32::from_le_bytes(read_array(bytes, 20)),
            vy: f32::from_le_bytes(read_array(bytes, 24)),
            vz: f32::from_le_bytes(read_array(bytes, 28)),
            angle: i32::from_le_bytes(read_array(bytes, 32)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 36)),
            speed: i32::from_le_bytes(read_array(bytes, 40)),
        }
    }
}

impl WirePayload for PcMoveRequest0104 {
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

/// `sP_CL2FE_REQ_PC_STOP` (`#pragma pack(4)`, 20 bytes, packet ID `0x13000004`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStopRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
}

impl PcStopRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcStopRequest0104 {
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

/// `sP_CL2FE_REQ_PC_JUMP` (`#pragma pack(4)`, 44 bytes, packet ID `0x13000005`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcJumpRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `iVX` at offset 20.
    pub vx: i32,
    /// `iVY` at offset 24.
    pub vy: i32,
    /// `iVZ` at offset 28.
    pub vz: i32,
    /// `iAngle` at offset 32.
    pub angle: i32,
    /// `cKeyValue` at offset 36.
    pub c_key_value: u8,
    /// `iSpeed` at offset 40.
    pub speed: i32,
}

impl PcJumpRequest0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.vx.to_le_bytes());
        write_prim(out, 24, &self.vy.to_le_bytes());
        write_prim(out, 28, &self.vz.to_le_bytes());
        write_prim(out, 32, &self.angle.to_le_bytes());
        write_prim(out, 36, &self.c_key_value.to_le_bytes());
        write_prim(out, 40, &self.speed.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            vx: i32::from_le_bytes(read_array(bytes, 20)),
            vy: i32::from_le_bytes(read_array(bytes, 24)),
            vz: i32::from_le_bytes(read_array(bytes, 28)),
            angle: i32::from_le_bytes(read_array(bytes, 32)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 36)),
            speed: i32::from_le_bytes(read_array(bytes, 40)),
        }
    }
}

impl WirePayload for PcJumpRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ATTACK_NPCs` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000006`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAttackNpcsRequest0104 {
    /// `iNPCCnt` at offset 0.
    pub npc_cnt: i32,
}

impl PcAttackNpcsRequest0104 {
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

impl WirePayload for PcAttackNpcsRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_FREECHAT_MESSAGE` (`#pragma pack(4)`, 260 bytes, packet ID `0x13000007`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendFreechatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
}

impl SendFreechatMessageRequest0104 {
    pub const SIZE: usize = 260;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.free_chat);
        write_prim(out, 256, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            free_chat: read_utf16(bytes, 0),
            emote_code: i32::from_le_bytes(read_array(bytes, 256)),
        }
    }
}

impl WirePayload for SendFreechatMessageRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_MENUCHAT_MESSAGE` (`#pragma pack(4)`, 260 bytes, packet ID `0x13000008`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendMenuchatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
}

impl SendMenuchatMessageRequest0104 {
    pub const SIZE: usize = 260;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.free_chat);
        write_prim(out, 256, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            free_chat: read_utf16(bytes, 0),
            emote_code: i32::from_le_bytes(read_array(bytes, 256)),
        }
    }
}

impl WirePayload for SendMenuchatMessageRequest0104 {
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

/// `sP_CL2FE_REQ_PC_REGEN` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000009`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegenRequest0104 {
    /// `iRegenType` at offset 0.
    pub regen_type: i32,
    /// `eIL` at offset 4.
    pub e_il: i32,
    /// `iIndex` at offset 8.
    pub index: i32,
}

impl PcRegenRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.regen_type.to_le_bytes());
        write_prim(out, 4, &self.e_il.to_le_bytes());
        write_prim(out, 8, &self.index.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            regen_type: i32::from_le_bytes(read_array(bytes, 0)),
            e_il: i32::from_le_bytes(read_array(bytes, 4)),
            index: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcRegenRequest0104 {
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

/// `sP_CL2FE_REQ_ITEM_MOVE` (`#pragma pack(4)`, 16 bytes, packet ID `0x1300000a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemMoveRequest0104 {
    /// `eFrom` at offset 0.
    pub from: i32,
    /// `iFromSlotNum` at offset 4.
    pub from_slot_num: i32,
    /// `eTo` at offset 8.
    pub to: i32,
    /// `iToSlotNum` at offset 12.
    pub to_slot_num: i32,
}

impl ItemMoveRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.from.to_le_bytes());
        write_prim(out, 4, &self.from_slot_num.to_le_bytes());
        write_prim(out, 8, &self.to.to_le_bytes());
        write_prim(out, 12, &self.to_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            from: i32::from_le_bytes(read_array(bytes, 0)),
            from_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            to: i32::from_le_bytes(read_array(bytes, 8)),
            to_slot_num: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for ItemMoveRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TASK_START` (`#pragma pack(4)`, 12 bytes, packet ID `0x1300000b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskStartRequest0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
    /// `iNPC_ID` at offset 4.
    pub npc_id: i32,
    /// `iEscortNPC_ID` at offset 8.
    pub escort_npc_id: i32,
}

impl PcTaskStartRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
        write_prim(out, 4, &self.npc_id.to_le_bytes());
        write_prim(out, 8, &self.escort_npc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
            npc_id: i32::from_le_bytes(read_array(bytes, 4)),
            escort_npc_id: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTaskStartRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TASK_END` (`#pragma pack(4)`, 16 bytes, packet ID `0x1300000c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskEndRequest0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
    /// `iNPC_ID` at offset 4.
    pub npc_id: i32,
    /// `iBox1Choice` at offset 8.
    pub box1_choice: i8,
    /// `iBox2Choice` at offset 9.
    pub box2_choice: i8,
    /// `iEscortNPC_ID` at offset 12.
    pub escort_npc_id: i32,
}

impl PcTaskEndRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
        write_prim(out, 4, &self.npc_id.to_le_bytes());
        write_prim(out, 8, &self.box1_choice.to_le_bytes());
        write_prim(out, 9, &self.box2_choice.to_le_bytes());
        write_prim(out, 12, &self.escort_npc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
            npc_id: i32::from_le_bytes(read_array(bytes, 4)),
            box1_choice: i8::from_le_bytes(read_array(bytes, 8)),
            box2_choice: i8::from_le_bytes(read_array(bytes, 9)),
            escort_npc_id: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTaskEndRequest0104 {
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

/// `sP_CL2FE_REQ_NANO_EQUIP` (`#pragma pack(2)`, 4 bytes, packet ID `0x1300000d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoEquipRequest0104 {
    /// `iNanoID` at offset 0.
    pub nano_id: i16,
    /// `iNanoSlotNum` at offset 2.
    pub nano_slot_num: i16,
}

impl NanoEquipRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_id.to_le_bytes());
        write_prim(out, 2, &self.nano_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_id: i16::from_le_bytes(read_array(bytes, 0)),
            nano_slot_num: i16::from_le_bytes(read_array(bytes, 2)),
        }
    }
}

impl WirePayload for NanoEquipRequest0104 {
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

/// `sP_CL2FE_REQ_NANO_UNEQUIP` (`#pragma pack(2)`, 2 bytes, packet ID `0x1300000e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoUnequipRequest0104 {
    /// `iNanoSlotNum` at offset 0.
    pub nano_slot_num: i16,
}

impl NanoUnequipRequest0104 {
    pub const SIZE: usize = 2;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_slot_num: i16::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for NanoUnequipRequest0104 {
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

/// `sP_CL2FE_REQ_NANO_ACTIVE` (`#pragma pack(2)`, 2 bytes, packet ID `0x1300000f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoActiveRequest0104 {
    /// `iNanoSlotNum` at offset 0.
    pub nano_slot_num: i16,
}

impl NanoActiveRequest0104 {
    pub const SIZE: usize = 2;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_slot_num: i16::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for NanoActiveRequest0104 {
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

/// `sP_CL2FE_REQ_NANO_TUNE` (`#pragma pack(4)`, 44 bytes, packet ID `0x13000010`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoTuneRequest0104 {
    /// `iNanoID` at offset 0.
    pub nano_id: i16,
    /// `iTuneID` at offset 2.
    pub tune_id: i16,
    /// `aiNeedItemSlotNum` at offset 4.
    pub ai_need_item_slot_num: [i32; 10],
}

impl NanoTuneRequest0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_id.to_le_bytes());
        write_prim(out, 2, &self.tune_id.to_le_bytes());
        for (index, value) in self.ai_need_item_slot_num.iter().enumerate() {
            write_prim(out, 4 + index * 4, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_id: i16::from_le_bytes(read_array(bytes, 0)),
            tune_id: i16::from_le_bytes(read_array(bytes, 2)),
            ai_need_item_slot_num: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 4 + index * 4))
            }),
        }
    }
}

impl WirePayload for NanoTuneRequest0104 {
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
