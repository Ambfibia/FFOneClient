// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_PC_CHANGE_MENTOR_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000093`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcChangeMentorSuccess0104 {
    /// `iMentor` at offset 0.
    pub mentor: i16,
    /// `iMentorCnt` at offset 2.
    pub mentor_cnt: i16,
    /// `iFusionMatter` at offset 4.
    pub fusion_matter: i32,
}

impl PcChangeMentorSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.mentor.to_le_bytes());
        write_prim(out, 2, &self.mentor_cnt.to_le_bytes());
        write_prim(out, 4, &self.fusion_matter.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            mentor: i16::from_le_bytes(read_array(bytes, 0)),
            mentor_cnt: i16::from_le_bytes(read_array(bytes, 2)),
            fusion_matter: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcChangeMentorSuccess0104 {
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

/// `sP_FE2CL_REP_PC_CHANGE_MENTOR_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000094`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcChangeMentorFailure0104 {
    /// `iMentor` at offset 0.
    pub mentor: i16,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcChangeMentorFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.mentor.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            mentor: i16::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcChangeMentorFailure0104 {
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

/// `sP_FE2CL_REP_GET_MEMBER_STYLE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000095`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetMemberStyleFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl GetMemberStyleFailure0104 {
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

impl WirePayload for GetMemberStyleFailure0104 {
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

/// `sP_FE2CL_REP_GET_MEMBER_STYLE_SUCC` (`#pragma pack(4)`, 196 bytes, packet ID `0x31000096`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetMemberStyleSuccess0104 {
    /// `iMemberID` at offset 0.
    pub member_id: i32,
    /// `iMemberUID` at offset 4.
    pub member_uid: i64,
    /// `BuddyStyleInfo` at offset 12.
    pub buddy_style_info: BuddyStyleInfo0104,
}

impl GetMemberStyleSuccess0104 {
    pub const SIZE: usize = 196;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.member_id.to_le_bytes());
        write_prim(out, 4, &self.member_uid.to_le_bytes());
        self.buddy_style_info.write_into(&mut out[12..196]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            member_id: i32::from_le_bytes(read_array(bytes, 0)),
            member_uid: i64::from_le_bytes(read_array(bytes, 4)),
            buddy_style_info: BuddyStyleInfo0104::read_from(&bytes[12..196]),
        }
    }
}

impl WirePayload for GetMemberStyleSuccess0104 {
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

/// `sP_FE2CL_REP_GET_GROUP_STYLE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000097`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetGroupStyleFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl GetGroupStyleFailure0104 {
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

impl WirePayload for GetGroupStyleFailure0104 {
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

/// `sP_FE2CL_REP_GET_GROUP_STYLE_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000098`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetGroupStyleSuccess0104 {
    /// `iMemberCnt` at offset 0.
    pub member_cnt: i32,
}

impl GetGroupStyleSuccess0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.member_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            member_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for GetGroupStyleSuccess0104 {
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

/// `sP_FE2CL_PC_REGEN` (`#pragma pack(4)`, 36 bytes, packet ID `0x31000099`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegen0104 {
    /// `PCRegenDataForOtherPC` at offset 0.
    pub pc_regen_data_for_other_pc: PcRegenDataForOtherPc0104,
}

impl PcRegen0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.pc_regen_data_for_other_pc.write_into(&mut out[0..36]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_regen_data_for_other_pc: PcRegenDataForOtherPc0104::read_from(&bytes[0..36]),
        }
    }
}

impl WirePayload for PcRegen0104 {
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

/// `sP_FE2CL_INSTANCE_MAP_INFO` (`#pragma pack(4)`, 60 bytes, packet ID `0x3100009a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceMapInfo0104 {
    /// `iInstanceMapNum` at offset 0.
    pub instance_map_num: i32,
    /// `iCreateTick` at offset 4.
    pub create_tick: u64,
    /// `iMapCoordX_Min` at offset 12.
    pub map_coord_x_min: i32,
    /// `iMapCoordX_Max` at offset 16.
    pub map_coord_x_max: i32,
    /// `iMapCoordY_Min` at offset 20.
    pub map_coord_y_min: i32,
    /// `iMapCoordY_Max` at offset 24.
    pub map_coord_y_max: i32,
    /// `iMapCoordZ_Min` at offset 28.
    pub map_coord_z_min: i32,
    /// `iMapCoordZ_Max` at offset 32.
    pub map_coord_z_max: i32,
    /// `iEP_ID` at offset 36.
    pub ep_id: i32,
    /// `iEPTopRecord_Score` at offset 40.
    pub ep_top_record_score: i32,
    /// `iEPTopRecord_Rank` at offset 44.
    pub ep_top_record_rank: i32,
    /// `iEPTopRecord_Time` at offset 48.
    pub ep_top_record_time: i32,
    /// `iEPTopRecord_RingCount` at offset 52.
    pub ep_top_record_ring_count: i32,
    /// `iEPSwitch_StatusON_Cnt` at offset 56.
    pub ep_switch_status_on_cnt: i32,
}

impl InstanceMapInfo0104 {
    pub const SIZE: usize = 60;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.instance_map_num.to_le_bytes());
        write_prim(out, 4, &self.create_tick.to_le_bytes());
        write_prim(out, 12, &self.map_coord_x_min.to_le_bytes());
        write_prim(out, 16, &self.map_coord_x_max.to_le_bytes());
        write_prim(out, 20, &self.map_coord_y_min.to_le_bytes());
        write_prim(out, 24, &self.map_coord_y_max.to_le_bytes());
        write_prim(out, 28, &self.map_coord_z_min.to_le_bytes());
        write_prim(out, 32, &self.map_coord_z_max.to_le_bytes());
        write_prim(out, 36, &self.ep_id.to_le_bytes());
        write_prim(out, 40, &self.ep_top_record_score.to_le_bytes());
        write_prim(out, 44, &self.ep_top_record_rank.to_le_bytes());
        write_prim(out, 48, &self.ep_top_record_time.to_le_bytes());
        write_prim(out, 52, &self.ep_top_record_ring_count.to_le_bytes());
        write_prim(out, 56, &self.ep_switch_status_on_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            instance_map_num: i32::from_le_bytes(read_array(bytes, 0)),
            create_tick: u64::from_le_bytes(read_array(bytes, 4)),
            map_coord_x_min: i32::from_le_bytes(read_array(bytes, 12)),
            map_coord_x_max: i32::from_le_bytes(read_array(bytes, 16)),
            map_coord_y_min: i32::from_le_bytes(read_array(bytes, 20)),
            map_coord_y_max: i32::from_le_bytes(read_array(bytes, 24)),
            map_coord_z_min: i32::from_le_bytes(read_array(bytes, 28)),
            map_coord_z_max: i32::from_le_bytes(read_array(bytes, 32)),
            ep_id: i32::from_le_bytes(read_array(bytes, 36)),
            ep_top_record_score: i32::from_le_bytes(read_array(bytes, 40)),
            ep_top_record_rank: i32::from_le_bytes(read_array(bytes, 44)),
            ep_top_record_time: i32::from_le_bytes(read_array(bytes, 48)),
            ep_top_record_ring_count: i32::from_le_bytes(read_array(bytes, 52)),
            ep_switch_status_on_cnt: i32::from_le_bytes(read_array(bytes, 56)),
        }
    }
}

impl WirePayload for InstanceMapInfo0104 {
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

/// `sP_FE2CL_TRANSPORTATION_ENTER` (`#pragma pack(4)`, 24 bytes, packet ID `0x3100009b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportationEnter0104 {
    /// `AppearanceData` at offset 0.
    pub appearance_data: TransportationAppearanceData0104,
}

impl TransportationEnter0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.appearance_data.write_into(&mut out[0..24]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            appearance_data: TransportationAppearanceData0104::read_from(&bytes[0..24]),
        }
    }
}

impl WirePayload for TransportationEnter0104 {
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

/// `sP_FE2CL_TRANSPORTATION_EXIT` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100009c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportationExit0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iT_ID` at offset 4.
    pub t_id: i32,
}

impl TransportationExit0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.t_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            t_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for TransportationExit0104 {
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

/// `sP_FE2CL_TRANSPORTATION_MOVE` (`#pragma pack(4)`, 28 bytes, packet ID `0x3100009d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportationMove0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iT_ID` at offset 4.
    pub t_id: i32,
    /// `iToX` at offset 8.
    pub to_x: i32,
    /// `iToY` at offset 12.
    pub to_y: i32,
    /// `iToZ` at offset 16.
    pub to_z: i32,
    /// `iSpeed` at offset 20.
    pub speed: i32,
    /// `iMoveStyle` at offset 24.
    pub move_style: i16,
}

impl TransportationMove0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.t_id.to_le_bytes());
        write_prim(out, 8, &self.to_x.to_le_bytes());
        write_prim(out, 12, &self.to_y.to_le_bytes());
        write_prim(out, 16, &self.to_z.to_le_bytes());
        write_prim(out, 20, &self.speed.to_le_bytes());
        write_prim(out, 24, &self.move_style.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            t_id: i32::from_le_bytes(read_array(bytes, 4)),
            to_x: i32::from_le_bytes(read_array(bytes, 8)),
            to_y: i32::from_le_bytes(read_array(bytes, 12)),
            to_z: i32::from_le_bytes(read_array(bytes, 16)),
            speed: i32::from_le_bytes(read_array(bytes, 20)),
            move_style: i16::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for TransportationMove0104 {
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

/// `sP_FE2CL_TRANSPORTATION_NEW` (`#pragma pack(4)`, 24 bytes, packet ID `0x3100009e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportationNew0104 {
    /// `AppearanceData` at offset 0.
    pub appearance_data: TransportationAppearanceData0104,
}

impl TransportationNew0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.appearance_data.write_into(&mut out[0..24]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            appearance_data: TransportationAppearanceData0104::read_from(&bytes[0..24]),
        }
    }
}

impl WirePayload for TransportationNew0104 {
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

/// `sP_FE2CL_TRANSPORTATION_AROUND` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100009f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportationAround0104 {
    /// `iCnt` at offset 0.
    pub cnt: i32,
}

impl TransportationAround0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for TransportationAround0104 {
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

/// `sP_FE2CL_AROUND_DEL_TRANSPORTATION` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000a0`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AroundDelTransportation0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iCnt` at offset 4.
    pub cnt: i32,
}

impl AroundDelTransportation0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for AroundDelTransportation0104 {
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

/// `sP_FE2CL_REP_EP_RANK_LIST` (`#pragma pack(8)`, 1 bytes, packet ID `0x310000a1`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRankListReply0104;

impl EpRankListReply0104 {
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

impl WirePayload for EpRankListReply0104 {
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

/// `sP_FE2CL_REP_EP_RANK_DETAIL` (`#pragma pack(8)`, 1 bytes, packet ID `0x310000a2`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRankDetailReply0104;

impl EpRankDetailReply0104 {
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

impl WirePayload for EpRankDetailReply0104 {
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

/// `sP_FE2CL_REP_EP_RANK_PC_INFO` (`#pragma pack(8)`, 1 bytes, packet ID `0x310000a3`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRankPcInfoReply0104;

impl EpRankPcInfoReply0104 {
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

impl WirePayload for EpRankPcInfoReply0104 {
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

/// `sP_FE2CL_REP_EP_RACE_START_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000a4`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceStartSuccess0104 {
    /// `iStartTick` at offset 0.
    pub start_tick: u64,
    /// `iLimitTime` at offset 8.
    pub limit_time: i32,
}

impl EpRaceStartSuccess0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.start_tick.to_le_bytes());
        write_prim(out, 8, &self.limit_time.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            start_tick: u64::from_le_bytes(read_array(bytes, 0)),
            limit_time: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for EpRaceStartSuccess0104 {
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
