// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_EP_RACE_START_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000a5`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceStartFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl EpRaceStartFailure0104 {
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

impl WirePayload for EpRaceStartFailure0104 {
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

/// `sP_FE2CL_REP_EP_RACE_END_SUCC` (`#pragma pack(4)`, 72 bytes, packet ID `0x310000a6`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceEndSuccess0104 {
    /// `iEPRaceMode` at offset 0.
    pub ep_race_mode: i32,
    /// `iEPRaceTime` at offset 4.
    pub ep_race_time: i32,
    /// `iEPRingCnt` at offset 8.
    pub ep_ring_cnt: i32,
    /// `iEPScore` at offset 12.
    pub ep_score: i32,
    /// `iEPRank` at offset 16.
    pub ep_rank: i32,
    /// `iEPRewardFM` at offset 20.
    pub ep_reward_fm: i32,
    /// `iEPTopScore` at offset 24.
    pub ep_top_score: i32,
    /// `iEPTopRank` at offset 28.
    pub ep_top_rank: i32,
    /// `iEPTopTime` at offset 32.
    pub ep_top_time: i32,
    /// `iEPTopRingCount` at offset 36.
    pub ep_top_ring_count: i32,
    /// `iFusionMatter` at offset 40.
    pub fusion_matter: i32,
    /// `RewardItem` at offset 44.
    pub reward_item: ItemReward0104,
    /// `iFatigue` at offset 64.
    pub fatigue: i32,
    /// `iFatigue_Level` at offset 68.
    pub fatigue_level: i32,
}

impl EpRaceEndSuccess0104 {
    pub const SIZE: usize = 72;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.ep_race_mode.to_le_bytes());
        write_prim(out, 4, &self.ep_race_time.to_le_bytes());
        write_prim(out, 8, &self.ep_ring_cnt.to_le_bytes());
        write_prim(out, 12, &self.ep_score.to_le_bytes());
        write_prim(out, 16, &self.ep_rank.to_le_bytes());
        write_prim(out, 20, &self.ep_reward_fm.to_le_bytes());
        write_prim(out, 24, &self.ep_top_score.to_le_bytes());
        write_prim(out, 28, &self.ep_top_rank.to_le_bytes());
        write_prim(out, 32, &self.ep_top_time.to_le_bytes());
        write_prim(out, 36, &self.ep_top_ring_count.to_le_bytes());
        write_prim(out, 40, &self.fusion_matter.to_le_bytes());
        self.reward_item.write_into(&mut out[44..64]);
        write_prim(out, 64, &self.fatigue.to_le_bytes());
        write_prim(out, 68, &self.fatigue_level.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            ep_race_mode: i32::from_le_bytes(read_array(bytes, 0)),
            ep_race_time: i32::from_le_bytes(read_array(bytes, 4)),
            ep_ring_cnt: i32::from_le_bytes(read_array(bytes, 8)),
            ep_score: i32::from_le_bytes(read_array(bytes, 12)),
            ep_rank: i32::from_le_bytes(read_array(bytes, 16)),
            ep_reward_fm: i32::from_le_bytes(read_array(bytes, 20)),
            ep_top_score: i32::from_le_bytes(read_array(bytes, 24)),
            ep_top_rank: i32::from_le_bytes(read_array(bytes, 28)),
            ep_top_time: i32::from_le_bytes(read_array(bytes, 32)),
            ep_top_ring_count: i32::from_le_bytes(read_array(bytes, 36)),
            fusion_matter: i32::from_le_bytes(read_array(bytes, 40)),
            reward_item: ItemReward0104::read_from(&bytes[44..64]),
            fatigue: i32::from_le_bytes(read_array(bytes, 64)),
            fatigue_level: i32::from_le_bytes(read_array(bytes, 68)),
        }
    }
}

impl WirePayload for EpRaceEndSuccess0104 {
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

/// `sP_FE2CL_REP_EP_RACE_END_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000a7`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceEndFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl EpRaceEndFailure0104 {
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

impl WirePayload for EpRaceEndFailure0104 {
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

/// `sP_FE2CL_REP_EP_RACE_CANCEL_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000a8`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceCancelSuccess0104 {
    /// `iTemp` at offset 0.
    pub temp: i32,
}

impl EpRaceCancelSuccess0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.temp.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            temp: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for EpRaceCancelSuccess0104 {
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

/// `sP_FE2CL_REP_EP_RACE_CANCEL_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000a9`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceCancelFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl EpRaceCancelFailure0104 {
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

impl WirePayload for EpRaceCancelFailure0104 {
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

/// `sP_FE2CL_REP_EP_GET_RING_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000aa`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpGetRingSuccess0104 {
    /// `iRingLID` at offset 0.
    pub ring_lid: i32,
    /// `iRingCount_Get` at offset 4.
    pub ring_count_get: i32,
}

impl EpGetRingSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.ring_lid.to_le_bytes());
        write_prim(out, 4, &self.ring_count_get.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            ring_lid: i32::from_le_bytes(read_array(bytes, 0)),
            ring_count_get: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for EpGetRingSuccess0104 {
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

/// `sP_FE2CL_REP_EP_GET_RING_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000ab`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpGetRingFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl EpGetRingFailure0104 {
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

impl WirePayload for EpGetRingFailure0104 {
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

/// `sP_FE2CL_REP_IM_CHANGE_SWITCH_STATUS` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000ac`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ImChangeSwitchStatusReply0104 {
    /// `iMapNum` at offset 0.
    pub map_num: i32,
    /// `iSwitchLID` at offset 4.
    pub switch_lid: i32,
    /// `iSwitchGID` at offset 8.
    pub switch_gid: i32,
    /// `iSwitchStatus` at offset 12.
    pub switch_status: i32,
}

impl ImChangeSwitchStatusReply0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.map_num.to_le_bytes());
        write_prim(out, 4, &self.switch_lid.to_le_bytes());
        write_prim(out, 8, &self.switch_gid.to_le_bytes());
        write_prim(out, 12, &self.switch_status.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            map_num: i32::from_le_bytes(read_array(bytes, 0)),
            switch_lid: i32::from_le_bytes(read_array(bytes, 4)),
            switch_gid: i32::from_le_bytes(read_array(bytes, 8)),
            switch_status: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for ImChangeSwitchStatusReply0104 {
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

/// `sP_FE2CL_SHINY_ENTER` (`#pragma pack(4)`, 24 bytes, packet ID `0x310000ad`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyEnter0104 {
    /// `ShinyAppearanceData` at offset 0.
    pub shiny_appearance_data: ShinyAppearanceData0104,
}

impl ShinyEnter0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.shiny_appearance_data.write_into(&mut out[0..24]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_appearance_data: ShinyAppearanceData0104::read_from(&bytes[0..24]),
        }
    }
}

impl WirePayload for ShinyEnter0104 {
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

/// `sP_FE2CL_SHINY_EXIT` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000ae`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyExit0104 {
    /// `iShinyID` at offset 0.
    pub shiny_id: i32,
}

impl ShinyExit0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.shiny_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for ShinyExit0104 {
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

/// `sP_FE2CL_SHINY_NEW` (`#pragma pack(4)`, 24 bytes, packet ID `0x310000af`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyNew0104 {
    /// `ShinyAppearanceData` at offset 0.
    pub shiny_appearance_data: ShinyAppearanceData0104,
}

impl ShinyNew0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.shiny_appearance_data.write_into(&mut out[0..24]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_appearance_data: ShinyAppearanceData0104::read_from(&bytes[0..24]),
        }
    }
}

impl WirePayload for ShinyNew0104 {
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

/// `sP_FE2CL_SHINY_AROUND` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000b0`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyAround0104 {
    /// `iShinyCnt` at offset 0.
    pub shiny_cnt: i32,
}

impl ShinyAround0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.shiny_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for ShinyAround0104 {
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

/// `sP_FE2CL_AROUND_DEL_SHINY` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000b1`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AroundDelShiny0104 {
    /// `iShinyCnt` at offset 0.
    pub shiny_cnt: i32,
}

impl AroundDelShiny0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.shiny_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for AroundDelShiny0104 {
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

/// `sP_FE2CL_REP_SHINY_PICKUP_FAIL` (`#pragma pack(8)`, 1 bytes, packet ID `0x310000b2`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyPickupFailure0104;

impl ShinyPickupFailure0104 {
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

impl WirePayload for ShinyPickupFailure0104 {
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

/// `sP_FE2CL_REP_SHINY_PICKUP_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000b3`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyPickupSuccess0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i16,
    /// `eCSTB` at offset 4.
    pub cstb: i32,
}

impl ShinyPickupSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
        write_prim(out, 4, &self.cstb.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i16::from_le_bytes(read_array(bytes, 0)),
            cstb: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for ShinyPickupSuccess0104 {
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

/// `sP_FE2CL_PC_MOVETRANSPORTATION` (`#pragma pack(4)`, 72 bytes, packet ID `0x310000b4`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMovetransportation0104 {
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
    /// `iT_ID` at offset 44.
    pub t_id: i32,
    /// `iAngle` at offset 48.
    pub angle: i32,
    /// `cKeyValue` at offset 52.
    pub c_key_value: u8,
    /// `iSpeed` at offset 56.
    pub speed: i32,
    /// `iPC_ID` at offset 60.
    pub pc_id: i32,
    /// `iSvrTime` at offset 64.
    pub svr_time: u64,
}

impl PcMovetransportation0104 {
    pub const SIZE: usize = 72;

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
        write_prim(out, 44, &self.t_id.to_le_bytes());
        write_prim(out, 48, &self.angle.to_le_bytes());
        write_prim(out, 52, &self.c_key_value.to_le_bytes());
        write_prim(out, 56, &self.speed.to_le_bytes());
        write_prim(out, 60, &self.pc_id.to_le_bytes());
        write_prim(out, 64, &self.svr_time.to_le_bytes());
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
            t_id: i32::from_le_bytes(read_array(bytes, 44)),
            angle: i32::from_le_bytes(read_array(bytes, 48)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 52)),
            speed: i32::from_le_bytes(read_array(bytes, 56)),
            pc_id: i32::from_le_bytes(read_array(bytes, 60)),
            svr_time: u64::from_le_bytes(read_array(bytes, 64)),
        }
    }
}

impl WirePayload for PcMovetransportation0104 {
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

/// `sP_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 264 bytes, packet ID `0x310000b5`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAllGroupFreechatMessageSuccess0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
}

impl SendAllGroupFreechatMessageSuccess0104 {
    pub const SIZE: usize = 264;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
        }
    }
}

impl WirePayload for SendAllGroupFreechatMessageSuccess0104 {
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
