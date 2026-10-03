// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_ERROR` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000000`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Error0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl Error0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for Error0104 {
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

/// `sP_FE2CL_REP_PC_ENTER_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000001`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcEnterFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcEnterFailure0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcEnterFailure0104 {
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

/// `sP_FE2CL_REP_PC_ENTER_SUCC` (`#pragma pack(4)`, 2564 bytes, packet ID `0x31000002`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// OpenFusion divergence: the pinned `structs/0104.hpp` declares this struct as 2700 bytes. Wiring code must choose the server-compatible length explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct PcEnterSuccess0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `PCLoadData2CL` at offset 4.
    pub pc_load_data2_cl: PcLoadData0104,
    /// `uiSvrTime` at offset 2556.
    pub svr_time: u64,
}

impl PcEnterSuccess0104 {
    pub const SIZE: usize = 2564;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        self.pc_load_data2_cl.write_into(&mut out[4..2556]);
        write_prim(out, 2556, &self.svr_time.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            pc_load_data2_cl: PcLoadData0104::read_from(&bytes[4..2556]),
            svr_time: u64::from_le_bytes(read_array(bytes, 2556)),
        }
    }
}

impl WirePayload for PcEnterSuccess0104 {
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

/// `sP_FE2CL_PC_NEW` (`#pragma pack(4)`, 232 bytes, packet ID `0x31000003`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNew0104 {
    /// `PCAppearanceData` at offset 0.
    pub pc_appearance_data: PcAppearanceData0104,
}

impl PcNew0104 {
    pub const SIZE: usize = 232;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.pc_appearance_data.write_into(&mut out[0..232]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_appearance_data: PcAppearanceData0104::read_from(&bytes[0..232]),
        }
    }
}

impl WirePayload for PcNew0104 {
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

/// `sP_FE2CL_REP_PC_EXIT_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000004`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcExitFailure0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcExitFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcExitFailure0104 {
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

/// `sP_FE2CL_REP_PC_EXIT_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000005`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcExitSuccess0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iExitCode` at offset 4.
    pub exit_code: i32,
}

impl PcExitSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.exit_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            exit_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcExitSuccess0104 {
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

/// `sP_FE2CL_PC_EXIT` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000006`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcExit0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iExitType` at offset 4.
    pub exit_type: i32,
}

impl PcExit0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.exit_type.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            exit_type: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcExit0104 {
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

/// `sP_FE2CL_PC_AROUND` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000007`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAround0104 {
    /// `iPCCnt` at offset 0.
    pub pc_cnt: i32,
}

impl PcAround0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcAround0104 {
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

/// `sP_FE2CL_PC_MOVE` (`#pragma pack(4)`, 56 bytes, packet ID `0x31000008`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMove0104 {
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
    /// `iID` at offset 44.
    pub id: i32,
    /// `iSvrTime` at offset 48.
    pub svr_time: u64,
}

impl PcMove0104 {
    pub const SIZE: usize = 56;

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
        write_prim(out, 44, &self.id.to_le_bytes());
        write_prim(out, 48, &self.svr_time.to_le_bytes());
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
            id: i32::from_le_bytes(read_array(bytes, 44)),
            svr_time: u64::from_le_bytes(read_array(bytes, 48)),
        }
    }
}

impl WirePayload for PcMove0104 {
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

/// `sP_FE2CL_PC_STOP` (`#pragma pack(4)`, 32 bytes, packet ID `0x31000009`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStop0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `iID` at offset 20.
    pub id: i32,
    /// `iSvrTime` at offset 24.
    pub svr_time: u64,
}

impl PcStop0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.id.to_le_bytes());
        write_prim(out, 24, &self.svr_time.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            id: i32::from_le_bytes(read_array(bytes, 20)),
            svr_time: u64::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for PcStop0104 {
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

/// `sP_FE2CL_PC_JUMP` (`#pragma pack(4)`, 56 bytes, packet ID `0x3100000a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcJump0104 {
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
    /// `iID` at offset 44.
    pub id: i32,
    /// `iSvrTime` at offset 48.
    pub svr_time: u64,
}

impl PcJump0104 {
    pub const SIZE: usize = 56;

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
        write_prim(out, 44, &self.id.to_le_bytes());
        write_prim(out, 48, &self.svr_time.to_le_bytes());
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
            id: i32::from_le_bytes(read_array(bytes, 44)),
            svr_time: u64::from_le_bytes(read_array(bytes, 48)),
        }
    }
}

impl WirePayload for PcJump0104 {
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

/// `sP_FE2CL_NPC_ENTER` (`#pragma pack(4)`, 36 bytes, packet ID `0x3100000b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcEnter0104 {
    /// `NPCAppearanceData` at offset 0.
    pub npc_appearance_data: NpcAppearanceData0104,
}

impl NpcEnter0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.npc_appearance_data.write_into(&mut out[0..36]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_appearance_data: NpcAppearanceData0104::read_from(&bytes[0..36]),
        }
    }
}

impl WirePayload for NpcEnter0104 {
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

/// `sP_FE2CL_NPC_EXIT` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100000c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcExit0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
}

impl NpcExit0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for NpcExit0104 {
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

/// `sP_FE2CL_NPC_MOVE` (`#pragma pack(4)`, 24 bytes, packet ID `0x3100000d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcMove0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iToX` at offset 4.
    pub to_x: i32,
    /// `iToY` at offset 8.
    pub to_y: i32,
    /// `iToZ` at offset 12.
    pub to_z: i32,
    /// `iSpeed` at offset 16.
    pub speed: i32,
    /// `iMoveStyle` at offset 20.
    pub move_style: i16,
}

impl NpcMove0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.to_x.to_le_bytes());
        write_prim(out, 8, &self.to_y.to_le_bytes());
        write_prim(out, 12, &self.to_z.to_le_bytes());
        write_prim(out, 16, &self.speed.to_le_bytes());
        write_prim(out, 20, &self.move_style.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            to_x: i32::from_le_bytes(read_array(bytes, 4)),
            to_y: i32::from_le_bytes(read_array(bytes, 8)),
            to_z: i32::from_le_bytes(read_array(bytes, 12)),
            speed: i32::from_le_bytes(read_array(bytes, 16)),
            move_style: i16::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for NpcMove0104 {
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

/// `sP_FE2CL_NPC_NEW` (`#pragma pack(4)`, 36 bytes, packet ID `0x3100000e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcNew0104 {
    /// `NPCAppearanceData` at offset 0.
    pub npc_appearance_data: NpcAppearanceData0104,
}

impl NpcNew0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.npc_appearance_data.write_into(&mut out[0..36]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_appearance_data: NpcAppearanceData0104::read_from(&bytes[0..36]),
        }
    }
}

impl WirePayload for NpcNew0104 {
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

/// `sP_FE2CL_NPC_AROUND` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100000f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcAround0104 {
    /// `iNPCCnt` at offset 0.
    pub npc_cnt: i32,
}

impl NpcAround0104 {
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

impl WirePayload for NpcAround0104 {
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

/// `sP_FE2CL_AROUND_DEL_PC` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000010`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AroundDelPc0104 {
    /// `iPCCnt` at offset 0.
    pub pc_cnt: i32,
}

impl AroundDelPc0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for AroundDelPc0104 {
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
