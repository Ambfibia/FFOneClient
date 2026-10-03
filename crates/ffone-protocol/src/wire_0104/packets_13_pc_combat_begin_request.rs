// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_PC_COMBAT_BEGIN` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000033`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcCombatBeginRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl PcCombatBeginRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcCombatBeginRequest0104 {
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

/// `sP_CL2FE_REQ_PC_COMBAT_END` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000034`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcCombatEndRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl PcCombatEndRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcCombatEndRequest0104 {
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

/// `sP_CL2FE_REQ_REQUEST_MAKE_BUDDY` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000035`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestMakeBuddyRequest0104 {
    /// `iBuddyID` at offset 0.
    pub buddy_id: i32,
    /// `iBuddyPCUID` at offset 4.
    pub buddy_pcuid: i64,
}

impl RequestMakeBuddyRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_id.to_le_bytes());
        write_prim(out, 4, &self.buddy_pcuid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_id: i32::from_le_bytes(read_array(bytes, 0)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for RequestMakeBuddyRequest0104 {
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

/// `sP_CL2FE_REQ_ACCEPT_MAKE_BUDDY` (`#pragma pack(4)`, 16 bytes, packet ID `0x13000036`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptMakeBuddyRequest0104 {
    /// `iAcceptFlag` at offset 0.
    pub accept_flag: i8,
    /// `iBuddyID` at offset 4.
    pub buddy_id: i32,
    /// `iBuddyPCUID` at offset 8.
    pub buddy_pcuid: i64,
}

impl AcceptMakeBuddyRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.accept_flag.to_le_bytes());
        write_prim(out, 4, &self.buddy_id.to_le_bytes());
        write_prim(out, 8, &self.buddy_pcuid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            accept_flag: i8::from_le_bytes(read_array(bytes, 0)),
            buddy_id: i32::from_le_bytes(read_array(bytes, 4)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for AcceptMakeBuddyRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE` (`#pragma pack(4)`, 272 bytes, packet ID `0x13000037`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendBuddyFreechatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
    /// `iBuddyPCUID` at offset 260.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 268.
    pub buddy_slot: i8,
}

impl SendBuddyFreechatMessageRequest0104 {
    pub const SIZE: usize = 272;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.free_chat);
        write_prim(out, 256, &self.emote_code.to_le_bytes());
        write_prim(out, 260, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 268, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            free_chat: read_utf16(bytes, 0),
            emote_code: i32::from_le_bytes(read_array(bytes, 256)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 260)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 268)),
        }
    }
}

impl WirePayload for SendBuddyFreechatMessageRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE` (`#pragma pack(4)`, 272 bytes, packet ID `0x13000038`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendBuddyMenuchatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
    /// `iBuddyPCUID` at offset 260.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 268.
    pub buddy_slot: i8,
}

impl SendBuddyMenuchatMessageRequest0104 {
    pub const SIZE: usize = 272;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.free_chat);
        write_prim(out, 256, &self.emote_code.to_le_bytes());
        write_prim(out, 260, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 268, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            free_chat: read_utf16(bytes, 0),
            emote_code: i32::from_le_bytes(read_array(bytes, 256)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 260)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 268)),
        }
    }
}

impl WirePayload for SendBuddyMenuchatMessageRequest0104 {
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

/// `sP_CL2FE_REQ_GET_BUDDY_STYLE` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000039`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyStyleRequest0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 8.
    pub buddy_slot: i8,
}

impl GetBuddyStyleRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for GetBuddyStyleRequest0104 {
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

/// `sP_CL2FE_REQ_SET_BUDDY_BLOCK` (`#pragma pack(4)`, 12 bytes, packet ID `0x1300003a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SetBuddyBlockRequest0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 8.
    pub buddy_slot: i8,
}

impl SetBuddyBlockRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for SetBuddyBlockRequest0104 {
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

/// `sP_CL2FE_REQ_REMOVE_BUDDY` (`#pragma pack(4)`, 12 bytes, packet ID `0x1300003b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveBuddyRequest0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 8.
    pub buddy_slot: i8,
}

impl RemoveBuddyRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for RemoveBuddyRequest0104 {
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

/// `sP_CL2FE_REQ_GET_BUDDY_STATE` (`#pragma pack(8)`, 1 bytes, packet ID `0x1300003c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyStateRequest0104;

impl GetBuddyStateRequest0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        out[0] = 0;
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self
    }
}

impl WirePayload for GetBuddyStateRequest0104 {
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

/// `sP_CL2FE_REQ_PC_JUMPPAD` (`#pragma pack(4)`, 40 bytes, packet ID `0x1300003d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcJumppadRequest0104 {
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
}

impl PcJumppadRequest0104 {
    pub const SIZE: usize = 40;

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
        }
    }
}

impl WirePayload for PcJumppadRequest0104 {
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

/// `sP_CL2FE_REQ_PC_LAUNCHER` (`#pragma pack(4)`, 40 bytes, packet ID `0x1300003e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcLauncherRequest0104 {
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
    /// `iSpeed` at offset 36.
    pub speed: i32,
}

impl PcLauncherRequest0104 {
    pub const SIZE: usize = 40;

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
        write_prim(out, 36, &self.speed.to_le_bytes());
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
            speed: i32::from_le_bytes(read_array(bytes, 36)),
        }
    }
}

impl WirePayload for PcLauncherRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ZIPLINE` (`#pragma pack(4)`, 76 bytes, packet ID `0x1300003f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcZiplineRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iStX` at offset 8.
    pub st_x: i32,
    /// `iStY` at offset 12.
    pub st_y: i32,
    /// `iStZ` at offset 16.
    pub st_z: i32,
    /// `fMovDistance` at offset 20.
    pub mov_distance: f32,
    /// `fMaxDistance` at offset 24.
    pub max_distance: f32,
    /// `fDummy` at offset 28.
    pub dummy: f32,
    /// `iX` at offset 32.
    pub x: i32,
    /// `iY` at offset 36.
    pub y: i32,
    /// `iZ` at offset 40.
    pub z: i32,
    /// `fVX` at offset 44.
    pub vx: f32,
    /// `fVY` at offset 48.
    pub vy: f32,
    /// `fVZ` at offset 52.
    pub vz: f32,
    /// `bDown` at offset 56.
    pub down: i32,
    /// `iRollMax` at offset 60.
    pub roll_max: i32,
    /// `iRoll` at offset 64.
    pub roll: u8,
    /// `iAngle` at offset 68.
    pub angle: i32,
    /// `iSpeed` at offset 72.
    pub speed: i32,
}

impl PcZiplineRequest0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.st_x.to_le_bytes());
        write_prim(out, 12, &self.st_y.to_le_bytes());
        write_prim(out, 16, &self.st_z.to_le_bytes());
        write_prim(out, 20, &self.mov_distance.to_le_bytes());
        write_prim(out, 24, &self.max_distance.to_le_bytes());
        write_prim(out, 28, &self.dummy.to_le_bytes());
        write_prim(out, 32, &self.x.to_le_bytes());
        write_prim(out, 36, &self.y.to_le_bytes());
        write_prim(out, 40, &self.z.to_le_bytes());
        write_prim(out, 44, &self.vx.to_le_bytes());
        write_prim(out, 48, &self.vy.to_le_bytes());
        write_prim(out, 52, &self.vz.to_le_bytes());
        write_prim(out, 56, &self.down.to_le_bytes());
        write_prim(out, 60, &self.roll_max.to_le_bytes());
        write_prim(out, 64, &self.roll.to_le_bytes());
        write_prim(out, 68, &self.angle.to_le_bytes());
        write_prim(out, 72, &self.speed.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            st_x: i32::from_le_bytes(read_array(bytes, 8)),
            st_y: i32::from_le_bytes(read_array(bytes, 12)),
            st_z: i32::from_le_bytes(read_array(bytes, 16)),
            mov_distance: f32::from_le_bytes(read_array(bytes, 20)),
            max_distance: f32::from_le_bytes(read_array(bytes, 24)),
            dummy: f32::from_le_bytes(read_array(bytes, 28)),
            x: i32::from_le_bytes(read_array(bytes, 32)),
            y: i32::from_le_bytes(read_array(bytes, 36)),
            z: i32::from_le_bytes(read_array(bytes, 40)),
            vx: f32::from_le_bytes(read_array(bytes, 44)),
            vy: f32::from_le_bytes(read_array(bytes, 48)),
            vz: f32::from_le_bytes(read_array(bytes, 52)),
            down: i32::from_le_bytes(read_array(bytes, 56)),
            roll_max: i32::from_le_bytes(read_array(bytes, 60)),
            roll: u8::from_le_bytes(read_array(bytes, 64)),
            angle: i32::from_le_bytes(read_array(bytes, 68)),
            speed: i32::from_le_bytes(read_array(bytes, 72)),
        }
    }
}

impl WirePayload for PcZiplineRequest0104 {
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

/// `sP_CL2FE_REQ_PC_MOVEPLATFORM` (`#pragma pack(4)`, 64 bytes, packet ID `0x13000040`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMoveplatformRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iLcX` at offset 8.
    pub lc_x: i32,
    /// `iLcY` at offset 12.
    pub lc_y: i32,
    /// `iLcZ` at offset 16.
    pub lc_z: i32,
    /// `iX` at offset 20.
    pub x: i32,
    /// `iY` at offset 24.
    pub y: i32,
    /// `iZ` at offset 28.
    pub z: i32,
    /// `fVX` at offset 32.
    pub vx: f32,
    /// `fVY` at offset 36.
    pub vy: f32,
    /// `fVZ` at offset 40.
    pub vz: f32,
    /// `bDown` at offset 44.
    pub down: i32,
    /// `iPlatformID` at offset 48.
    pub platform_id: i32,
    /// `iAngle` at offset 52.
    pub angle: i32,
    /// `cKeyValue` at offset 56.
    pub c_key_value: u8,
    /// `iSpeed` at offset 60.
    pub speed: i32,
}

impl PcMoveplatformRequest0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.lc_x.to_le_bytes());
        write_prim(out, 12, &self.lc_y.to_le_bytes());
        write_prim(out, 16, &self.lc_z.to_le_bytes());
        write_prim(out, 20, &self.x.to_le_bytes());
        write_prim(out, 24, &self.y.to_le_bytes());
        write_prim(out, 28, &self.z.to_le_bytes());
        write_prim(out, 32, &self.vx.to_le_bytes());
        write_prim(out, 36, &self.vy.to_le_bytes());
        write_prim(out, 40, &self.vz.to_le_bytes());
        write_prim(out, 44, &self.down.to_le_bytes());
        write_prim(out, 48, &self.platform_id.to_le_bytes());
        write_prim(out, 52, &self.angle.to_le_bytes());
        write_prim(out, 56, &self.c_key_value.to_le_bytes());
        write_prim(out, 60, &self.speed.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            lc_x: i32::from_le_bytes(read_array(bytes, 8)),
            lc_y: i32::from_le_bytes(read_array(bytes, 12)),
            lc_z: i32::from_le_bytes(read_array(bytes, 16)),
            x: i32::from_le_bytes(read_array(bytes, 20)),
            y: i32::from_le_bytes(read_array(bytes, 24)),
            z: i32::from_le_bytes(read_array(bytes, 28)),
            vx: f32::from_le_bytes(read_array(bytes, 32)),
            vy: f32::from_le_bytes(read_array(bytes, 36)),
            vz: f32::from_le_bytes(read_array(bytes, 40)),
            down: i32::from_le_bytes(read_array(bytes, 44)),
            platform_id: i32::from_le_bytes(read_array(bytes, 48)),
            angle: i32::from_le_bytes(read_array(bytes, 52)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 56)),
            speed: i32::from_le_bytes(read_array(bytes, 60)),
        }
    }
}

impl WirePayload for PcMoveplatformRequest0104 {
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
