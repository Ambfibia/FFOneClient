// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_PC_SKILL_USE_SUCC` (`#pragma pack(4)`, 60 bytes, packet ID `0x3100011d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillUseSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSkillSlotNum` at offset 4.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 8.
    pub skill_id: i32,
    /// `iX` at offset 12.
    pub x: i32,
    /// `iY` at offset 16.
    pub y: i32,
    /// `iZ` at offset 20.
    pub z: i32,
    /// `iAngle` at offset 24.
    pub angle: i32,
    /// `iBlockMove` at offset 28.
    pub block_move: i32,
    /// `eST` at offset 32.
    pub e_st: i32,
    /// `iTargetID` at offset 36.
    pub target_id: i32,
    /// `iTargetType` at offset 40.
    pub target_type: i32,
    /// `iTargetLocationX` at offset 44.
    pub target_location_x: i32,
    /// `iTargetLocationY` at offset 48.
    pub target_location_y: i32,
    /// `iTargetLocationZ` at offset 52.
    pub target_location_z: i32,
    /// `iTargetCnt` at offset 56.
    pub target_cnt: i32,
}

impl PcSkillUseSuccess0104 {
    pub const SIZE: usize = 60;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_slot_num.to_le_bytes());
        write_prim(out, 8, &self.skill_id.to_le_bytes());
        write_prim(out, 12, &self.x.to_le_bytes());
        write_prim(out, 16, &self.y.to_le_bytes());
        write_prim(out, 20, &self.z.to_le_bytes());
        write_prim(out, 24, &self.angle.to_le_bytes());
        write_prim(out, 28, &self.block_move.to_le_bytes());
        write_prim(out, 32, &self.e_st.to_le_bytes());
        write_prim(out, 36, &self.target_id.to_le_bytes());
        write_prim(out, 40, &self.target_type.to_le_bytes());
        write_prim(out, 44, &self.target_location_x.to_le_bytes());
        write_prim(out, 48, &self.target_location_y.to_le_bytes());
        write_prim(out, 52, &self.target_location_z.to_le_bytes());
        write_prim(out, 56, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            skill_id: i32::from_le_bytes(read_array(bytes, 8)),
            x: i32::from_le_bytes(read_array(bytes, 12)),
            y: i32::from_le_bytes(read_array(bytes, 16)),
            z: i32::from_le_bytes(read_array(bytes, 20)),
            angle: i32::from_le_bytes(read_array(bytes, 24)),
            block_move: i32::from_le_bytes(read_array(bytes, 28)),
            e_st: i32::from_le_bytes(read_array(bytes, 32)),
            target_id: i32::from_le_bytes(read_array(bytes, 36)),
            target_type: i32::from_le_bytes(read_array(bytes, 40)),
            target_location_x: i32::from_le_bytes(read_array(bytes, 44)),
            target_location_y: i32::from_le_bytes(read_array(bytes, 48)),
            target_location_z: i32::from_le_bytes(read_array(bytes, 52)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 56)),
        }
    }
}

impl WirePayload for PcSkillUseSuccess0104 {
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

/// `sP_FE2CL_REP_PC_SKILL_USE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100011e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillUseFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcSkillUseFailure0104 {
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

impl WirePayload for PcSkillUseFailure0104 {
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

/// `sP_FE2CL_PC_SKILL_USE` (`#pragma pack(4)`, 60 bytes, packet ID `0x3100011f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillUse0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSkillSlotNum` at offset 4.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 8.
    pub skill_id: i32,
    /// `iX` at offset 12.
    pub x: i32,
    /// `iY` at offset 16.
    pub y: i32,
    /// `iZ` at offset 20.
    pub z: i32,
    /// `iAngle` at offset 24.
    pub angle: i32,
    /// `iBlockMove` at offset 28.
    pub block_move: i32,
    /// `eST` at offset 32.
    pub e_st: i32,
    /// `iTargetID` at offset 36.
    pub target_id: i32,
    /// `iTargetType` at offset 40.
    pub target_type: i32,
    /// `iTargetLocationX` at offset 44.
    pub target_location_x: i32,
    /// `iTargetLocationY` at offset 48.
    pub target_location_y: i32,
    /// `iTargetLocationZ` at offset 52.
    pub target_location_z: i32,
    /// `iTargetCnt` at offset 56.
    pub target_cnt: i32,
}

impl PcSkillUse0104 {
    pub const SIZE: usize = 60;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_slot_num.to_le_bytes());
        write_prim(out, 8, &self.skill_id.to_le_bytes());
        write_prim(out, 12, &self.x.to_le_bytes());
        write_prim(out, 16, &self.y.to_le_bytes());
        write_prim(out, 20, &self.z.to_le_bytes());
        write_prim(out, 24, &self.angle.to_le_bytes());
        write_prim(out, 28, &self.block_move.to_le_bytes());
        write_prim(out, 32, &self.e_st.to_le_bytes());
        write_prim(out, 36, &self.target_id.to_le_bytes());
        write_prim(out, 40, &self.target_type.to_le_bytes());
        write_prim(out, 44, &self.target_location_x.to_le_bytes());
        write_prim(out, 48, &self.target_location_y.to_le_bytes());
        write_prim(out, 52, &self.target_location_z.to_le_bytes());
        write_prim(out, 56, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            skill_id: i32::from_le_bytes(read_array(bytes, 8)),
            x: i32::from_le_bytes(read_array(bytes, 12)),
            y: i32::from_le_bytes(read_array(bytes, 16)),
            z: i32::from_le_bytes(read_array(bytes, 20)),
            angle: i32::from_le_bytes(read_array(bytes, 24)),
            block_move: i32::from_le_bytes(read_array(bytes, 28)),
            e_st: i32::from_le_bytes(read_array(bytes, 32)),
            target_id: i32::from_le_bytes(read_array(bytes, 36)),
            target_type: i32::from_le_bytes(read_array(bytes, 40)),
            target_location_x: i32::from_le_bytes(read_array(bytes, 44)),
            target_location_y: i32::from_le_bytes(read_array(bytes, 48)),
            target_location_z: i32::from_le_bytes(read_array(bytes, 52)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 56)),
        }
    }
}

impl WirePayload for PcSkillUse0104 {
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

/// `sP_FE2CL_PC_ROPE` (`#pragma pack(4)`, 60 bytes, packet ID `0x31000120`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRope0104 {
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
    /// `iRopeID` at offset 32.
    pub rope_id: i32,
    /// `iAngle` at offset 36.
    pub angle: i32,
    /// `cKeyValue` at offset 40.
    pub c_key_value: u8,
    /// `iSpeed` at offset 44.
    pub speed: i32,
    /// `iPC_ID` at offset 48.
    pub pc_id: i32,
    /// `iSvrTime` at offset 52.
    pub svr_time: u64,
}

impl PcRope0104 {
    pub const SIZE: usize = 60;

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
        write_prim(out, 32, &self.rope_id.to_le_bytes());
        write_prim(out, 36, &self.angle.to_le_bytes());
        write_prim(out, 40, &self.c_key_value.to_le_bytes());
        write_prim(out, 44, &self.speed.to_le_bytes());
        write_prim(out, 48, &self.pc_id.to_le_bytes());
        write_prim(out, 52, &self.svr_time.to_le_bytes());
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
            rope_id: i32::from_le_bytes(read_array(bytes, 32)),
            angle: i32::from_le_bytes(read_array(bytes, 36)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 40)),
            speed: i32::from_le_bytes(read_array(bytes, 44)),
            pc_id: i32::from_le_bytes(read_array(bytes, 48)),
            svr_time: u64::from_le_bytes(read_array(bytes, 52)),
        }
    }
}

impl WirePayload for PcRope0104 {
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

/// `sP_FE2CL_PC_BELT` (`#pragma pack(4)`, 76 bytes, packet ID `0x31000121`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBelt0104 {
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
    /// `iBeltID` at offset 48.
    pub belt_id: i32,
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

impl PcBelt0104 {
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
        write_prim(out, 48, &self.belt_id.to_le_bytes());
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
            belt_id: i32::from_le_bytes(read_array(bytes, 48)),
            angle: i32::from_le_bytes(read_array(bytes, 52)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 56)),
            speed: i32::from_le_bytes(read_array(bytes, 60)),
            pc_id: i32::from_le_bytes(read_array(bytes, 64)),
            svr_time: u64::from_le_bytes(read_array(bytes, 68)),
        }
    }
}

impl WirePayload for PcBelt0104 {
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

/// `sP_FE2CL_PC_VEHICLE_ON_SUCC` (`#pragma pack(8)`, 1 bytes, packet ID `0x31000122`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVehicleOnSuccess0104;

impl PcVehicleOnSuccess0104 {
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

impl WirePayload for PcVehicleOnSuccess0104 {
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

/// `sP_FE2CL_PC_VEHICLE_ON_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000123`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVehicleOnFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVehicleOnFailure0104 {
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

impl WirePayload for PcVehicleOnFailure0104 {
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

/// `sP_FE2CL_PC_VEHICLE_OFF_SUCC` (`#pragma pack(8)`, 1 bytes, packet ID `0x31000124`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVehicleOffSuccess0104;

impl PcVehicleOffSuccess0104 {
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

impl WirePayload for PcVehicleOffSuccess0104 {
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

/// `sP_FE2CL_PC_VEHICLE_OFF_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000125`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVehicleOffFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVehicleOffFailure0104 {
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

impl WirePayload for PcVehicleOffFailure0104 {
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

/// `sP_FE2CL_PC_QUICK_SLOT_INFO` (`#pragma pack(2)`, 32 bytes, packet ID `0x31000126`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcQuickSlotInfo0104 {
    /// `aQuickSlot` at offset 0.
    pub quick_slot: [QuickSlot0104; 8],
}

impl PcQuickSlotInfo0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.quick_slot.iter().enumerate() {
            let start = 0 + index * 4;
            value.write_into(&mut out[start..start + 4]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            quick_slot: std::array::from_fn(|index| {
                let start = 0 + index * 4;
                QuickSlot0104::read_from(&bytes[start..start + 4])
            }),
        }
    }
}

impl WirePayload for PcQuickSlotInfo0104 {
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

/// `sP_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000127`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegistQuickSlotFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcRegistQuickSlotFailure0104 {
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

impl WirePayload for PcRegistQuickSlotFailure0104 {
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

/// `sP_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000128`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegistQuickSlotSuccess0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i32,
    /// `iItemType` at offset 4.
    pub item_type: i16,
    /// `iItemID` at offset 6.
    pub item_id: i16,
}

impl PcRegistQuickSlotSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
        write_prim(out, 4, &self.item_type.to_le_bytes());
        write_prim(out, 6, &self.item_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            item_type: i16::from_le_bytes(read_array(bytes, 4)),
            item_id: i16::from_le_bytes(read_array(bytes, 6)),
        }
    }
}

impl WirePayload for PcRegistQuickSlotSuccess0104 {
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

/// `sP_FE2CL_PC_DELETE_TIME_LIMIT_ITEM` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000129`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcDeleteTimeLimitItem0104 {
    /// `iItemListCount` at offset 0.
    pub item_list_count: i32,
}

impl PcDeleteTimeLimitItem0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.item_list_count.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            item_list_count: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcDeleteTimeLimitItem0104 {
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

/// `sP_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x3100012a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcDisassembleItemSuccess0104 {
    /// `iNewItemSlot` at offset 0.
    pub new_item_slot: i32,
    /// `sNewItem` at offset 4.
    pub s_new_item: ItemBase0104,
}

impl PcDisassembleItemSuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.new_item_slot.to_le_bytes());
        self.s_new_item.write_into(&mut out[4..16]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            new_item_slot: i32::from_le_bytes(read_array(bytes, 0)),
            s_new_item: ItemBase0104::read_from(&bytes[4..16]),
        }
    }
}

impl WirePayload for PcDisassembleItemSuccess0104 {
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

/// `sP_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100012b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcDisassembleItemFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `iItemSlot` at offset 4.
    pub item_slot: i32,
}

impl PcDisassembleItemFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_prim(out, 4, &self.item_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            item_slot: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcDisassembleItemFailure0104 {
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
