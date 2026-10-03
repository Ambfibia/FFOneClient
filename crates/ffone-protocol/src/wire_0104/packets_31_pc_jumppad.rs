// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_PC_JUMPPAD` (`#pragma pack(4)`, 52 bytes, packet ID `0x31000075`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcJumppad0104 {
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
    /// `iPC_ID` at offset 40.
    pub pc_id: i32,
    /// `iSvrTime` at offset 44.
    pub svr_time: u64,
}

impl PcJumppad0104 {
    pub const SIZE: usize = 52;

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
        write_prim(out, 40, &self.pc_id.to_le_bytes());
        write_prim(out, 44, &self.svr_time.to_le_bytes());
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
            pc_id: i32::from_le_bytes(read_array(bytes, 40)),
            svr_time: u64::from_le_bytes(read_array(bytes, 44)),
        }
    }
}

impl WirePayload for PcJumppad0104 {
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

/// `sP_FE2CL_PC_LAUNCHER` (`#pragma pack(4)`, 52 bytes, packet ID `0x31000076`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcLauncher0104 {
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
    /// `iPC_ID` at offset 40.
    pub pc_id: i32,
    /// `iSvrTime` at offset 44.
    pub svr_time: u64,
}

impl PcLauncher0104 {
    pub const SIZE: usize = 52;

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
        write_prim(out, 40, &self.pc_id.to_le_bytes());
        write_prim(out, 44, &self.svr_time.to_le_bytes());
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
            pc_id: i32::from_le_bytes(read_array(bytes, 40)),
            svr_time: u64::from_le_bytes(read_array(bytes, 44)),
        }
    }
}

impl WirePayload for PcLauncher0104 {
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

/// `sP_FE2CL_PC_ZIPLINE` (`#pragma pack(4)`, 88 bytes, packet ID `0x31000077`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcZipline0104 {
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
    /// `iPC_ID` at offset 76.
    pub pc_id: i32,
    /// `iSvrTime` at offset 80.
    pub svr_time: u64,
}

impl PcZipline0104 {
    pub const SIZE: usize = 88;

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
        write_prim(out, 76, &self.pc_id.to_le_bytes());
        write_prim(out, 80, &self.svr_time.to_le_bytes());
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
            pc_id: i32::from_le_bytes(read_array(bytes, 76)),
            svr_time: u64::from_le_bytes(read_array(bytes, 80)),
        }
    }
}

impl WirePayload for PcZipline0104 {
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

/// `sP_FE2CL_PC_MOVEPLATFORM` (`#pragma pack(4)`, 76 bytes, packet ID `0x31000078`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMoveplatform0104 {
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
    /// `iPC_ID` at offset 64.
    pub pc_id: i32,
    /// `iSvrTime` at offset 68.
    pub svr_time: u64,
}

impl PcMoveplatform0104 {
    pub const SIZE: usize = 76;

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
        write_prim(out, 64, &self.pc_id.to_le_bytes());
        write_prim(out, 68, &self.svr_time.to_le_bytes());
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
            pc_id: i32::from_le_bytes(read_array(bytes, 64)),
            svr_time: u64::from_le_bytes(read_array(bytes, 68)),
        }
    }
}

impl WirePayload for PcMoveplatform0104 {
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

/// `sP_FE2CL_PC_SLOPE` (`#pragma pack(4)`, 60 bytes, packet ID `0x31000079`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSlope0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `iAngle` at offset 20.
    pub angle: i32,
    /// `iSpeed` at offset 24.
    pub speed: i32,
    /// `cKeyValue` at offset 28.
    pub c_key_value: u8,
    /// `iPC_ID` at offset 32.
    pub pc_id: i32,
    /// `iSvrTime` at offset 36.
    pub svr_time: u64,
    /// `fVX` at offset 44.
    pub vx: f32,
    /// `fVY` at offset 48.
    pub vy: f32,
    /// `fVZ` at offset 52.
    pub vz: f32,
    /// `iSlopeID` at offset 56.
    pub slope_id: i32,
}

impl PcSlope0104 {
    pub const SIZE: usize = 60;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.angle.to_le_bytes());
        write_prim(out, 24, &self.speed.to_le_bytes());
        write_prim(out, 28, &self.c_key_value.to_le_bytes());
        write_prim(out, 32, &self.pc_id.to_le_bytes());
        write_prim(out, 36, &self.svr_time.to_le_bytes());
        write_prim(out, 44, &self.vx.to_le_bytes());
        write_prim(out, 48, &self.vy.to_le_bytes());
        write_prim(out, 52, &self.vz.to_le_bytes());
        write_prim(out, 56, &self.slope_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            angle: i32::from_le_bytes(read_array(bytes, 20)),
            speed: i32::from_le_bytes(read_array(bytes, 24)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 28)),
            pc_id: i32::from_le_bytes(read_array(bytes, 32)),
            svr_time: u64::from_le_bytes(read_array(bytes, 36)),
            vx: f32::from_le_bytes(read_array(bytes, 44)),
            vy: f32::from_le_bytes(read_array(bytes, 48)),
            vz: f32::from_le_bytes(read_array(bytes, 52)),
            slope_id: i32::from_le_bytes(read_array(bytes, 56)),
        }
    }
}

impl WirePayload for PcSlope0104 {
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

/// `sP_FE2CL_PC_STATE_CHANGE` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100007a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStateChange0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iState` at offset 4.
    pub state: i8,
}

impl PcStateChange0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            state: i8::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcStateChange0104 {
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

/// `sP_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC_TO_ACCEPTER` (`#pragma pack(4)`, 60 bytes, packet ID `0x3100007b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestMakeBuddySuccessToAccepter0104 {
    /// `iRequestID` at offset 0.
    pub request_id: i32,
    /// `iBuddyID` at offset 4.
    pub buddy_id: i32,
    /// `szFirstName` at offset 8.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 26.
    pub last_name: FixedUtf16<17>,
}

impl RequestMakeBuddySuccessToAccepter0104 {
    pub const SIZE: usize = 60;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.request_id.to_le_bytes());
        write_prim(out, 4, &self.buddy_id.to_le_bytes());
        write_utf16(out, 8, &self.first_name);
        write_utf16(out, 26, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            request_id: i32::from_le_bytes(read_array(bytes, 0)),
            buddy_id: i32::from_le_bytes(read_array(bytes, 4)),
            first_name: read_utf16(bytes, 8),
            last_name: read_utf16(bytes, 26),
        }
    }
}

impl WirePayload for RequestMakeBuddySuccessToAccepter0104 {
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

/// `sP_FE2CL_REP_REWARD_ITEM` (`#pragma pack(4)`, 36 bytes, packet ID `0x3100007c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RewardItemReply0104 {
    /// `m_iCandy` at offset 0.
    pub m_i_candy: i32,
    /// `m_iFusionMatter` at offset 4.
    pub m_i_fusion_matter: i32,
    /// `m_iBatteryN` at offset 8.
    pub m_i_battery_n: i32,
    /// `m_iBatteryW` at offset 12.
    pub m_i_battery_w: i32,
    /// `iItemCnt` at offset 16.
    pub item_cnt: i8,
    /// `iFatigue` at offset 20.
    pub fatigue: i32,
    /// `iFatigue_Level` at offset 24.
    pub fatigue_level: i32,
    /// `iNPC_TypeID` at offset 28.
    pub npc_type_id: i32,
    /// `iTaskID` at offset 32.
    pub task_id: i32,
}

impl RewardItemReply0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.m_i_candy.to_le_bytes());
        write_prim(out, 4, &self.m_i_fusion_matter.to_le_bytes());
        write_prim(out, 8, &self.m_i_battery_n.to_le_bytes());
        write_prim(out, 12, &self.m_i_battery_w.to_le_bytes());
        write_prim(out, 16, &self.item_cnt.to_le_bytes());
        write_prim(out, 20, &self.fatigue.to_le_bytes());
        write_prim(out, 24, &self.fatigue_level.to_le_bytes());
        write_prim(out, 28, &self.npc_type_id.to_le_bytes());
        write_prim(out, 32, &self.task_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            m_i_candy: i32::from_le_bytes(read_array(bytes, 0)),
            m_i_fusion_matter: i32::from_le_bytes(read_array(bytes, 4)),
            m_i_battery_n: i32::from_le_bytes(read_array(bytes, 8)),
            m_i_battery_w: i32::from_le_bytes(read_array(bytes, 12)),
            item_cnt: i8::from_le_bytes(read_array(bytes, 16)),
            fatigue: i32::from_le_bytes(read_array(bytes, 20)),
            fatigue_level: i32::from_le_bytes(read_array(bytes, 24)),
            npc_type_id: i32::from_le_bytes(read_array(bytes, 28)),
            task_id: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for RewardItemReply0104 {
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

/// `sP_FE2CL_REP_ITEM_CHEST_OPEN_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100007d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemChestOpenSuccess0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i32,
}

impl ItemChestOpenSuccess0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for ItemChestOpenSuccess0104 {
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

/// `sP_FE2CL_REP_ITEM_CHEST_OPEN_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100007e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemChestOpenFailure0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl ItemChestOpenFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for ItemChestOpenFailure0104 {
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

/// `sP_FE2CL_CHAR_TIME_BUFF_TIME_TICK` (`#pragma pack(4)`, 12 bytes, packet ID `0x3100007f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct CharTimeBuffTimeTick0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `iTB_ID` at offset 8.
    pub tb_id: i16,
}

impl CharTimeBuffTimeTick0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.tb_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            tb_id: i16::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for CharTimeBuffTimeTick0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000080`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorBatteryBuySuccess0104 {
    /// `iCandy` at offset 0.
    pub candy: i32,
    /// `iBatteryW` at offset 4.
    pub battery_w: i32,
    /// `iBatteryN` at offset 8.
    pub battery_n: i32,
}

impl PcVendorBatteryBuySuccess0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.candy.to_le_bytes());
        write_prim(out, 4, &self.battery_w.to_le_bytes());
        write_prim(out, 8, &self.battery_n.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            candy: i32::from_le_bytes(read_array(bytes, 0)),
            battery_w: i32::from_le_bytes(read_array(bytes, 4)),
            battery_n: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcVendorBatteryBuySuccess0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_BATTERY_BUY_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000081`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorBatteryBuyFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVendorBatteryBuyFailure0104 {
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

impl WirePayload for PcVendorBatteryBuyFailure0104 {
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
