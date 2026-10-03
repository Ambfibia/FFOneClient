// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_GET_GROUP_STYLE` (`#pragma pack(8)`, 1 bytes, packet ID `0x13000053`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct GetGroupStyleRequest0104;

impl GetGroupStyleRequest0104 {
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

impl WirePayload for GetGroupStyleRequest0104 {
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

/// `sP_CL2FE_REQ_PC_CHANGE_MENTOR` (`#pragma pack(2)`, 2 bytes, packet ID `0x13000054`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcChangeMentorRequest0104 {
    /// `iMentor` at offset 0.
    pub mentor: i16,
}

impl PcChangeMentorRequest0104 {
    pub const SIZE: usize = 2;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.mentor.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            mentor: i16::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcChangeMentorRequest0104 {
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

/// `sP_CL2FE_REQ_GET_BUDDY_LOCATION` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000055`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyLocationRequest0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
}

impl GetBuddyLocationRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for GetBuddyLocationRequest0104 {
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

/// `sP_CL2FE_REQ_NPC_GROUP_SUMMON` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000056`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupSummonRequest0104 {
    /// `iNPCGroupType` at offset 0.
    pub npc_group_type: i32,
}

impl NpcGroupSummonRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_group_type.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_group_type: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for NpcGroupSummonRequest0104 {
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

/// `sP_CL2FE_REQ_PC_WARP_TO_PC` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000057`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpToPcRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iPCUID` at offset 4.
    pub pcuid: i32,
}

impl PcWarpToPcRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.pcuid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            pcuid: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcWarpToPcRequest0104 {
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

/// `sP_CL2FE_REQ_EP_RANK_GET_LIST` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000058`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRankGetListRequest0104 {
    /// `iRankListPageNum` at offset 0.
    pub rank_list_page_num: i32,
}

impl EpRankGetListRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.rank_list_page_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            rank_list_page_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for EpRankGetListRequest0104 {
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

/// `sP_CL2FE_REQ_EP_RANK_GET_DETAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000059`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRankGetDetailRequest0104 {
    /// `iEP_ID` at offset 0.
    pub ep_id: i32,
}

impl EpRankGetDetailRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.ep_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            ep_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for EpRankGetDetailRequest0104 {
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

/// `sP_CL2FE_REQ_EP_RANK_GET_PC_INFO` (`#pragma pack(4)`, 56 bytes, packet ID `0x1300005a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRankGetPcInfoRequest0104 {
    /// `iEP_ID` at offset 0.
    pub ep_id: i32,
    /// `szFirstName` at offset 4.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 22.
    pub last_name: FixedUtf16<17>,
}

impl EpRankGetPcInfoRequest0104 {
    pub const SIZE: usize = 56;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.ep_id.to_le_bytes());
        write_utf16(out, 4, &self.first_name);
        write_utf16(out, 22, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            ep_id: i32::from_le_bytes(read_array(bytes, 0)),
            first_name: read_utf16(bytes, 4),
            last_name: read_utf16(bytes, 22),
        }
    }
}

impl WirePayload for EpRankGetPcInfoRequest0104 {
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

/// `sP_CL2FE_REQ_EP_RACE_START` (`#pragma pack(4)`, 12 bytes, packet ID `0x1300005b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceStartRequest0104 {
    /// `iStartEcomID` at offset 0.
    pub start_ecom_id: i32,
    /// `iEPRaceMode` at offset 4.
    pub ep_race_mode: i32,
    /// `iEPTicketItemSlotNum` at offset 8.
    pub ep_ticket_item_slot_num: i32,
}

impl EpRaceStartRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.start_ecom_id.to_le_bytes());
        write_prim(out, 4, &self.ep_race_mode.to_le_bytes());
        write_prim(out, 8, &self.ep_ticket_item_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            start_ecom_id: i32::from_le_bytes(read_array(bytes, 0)),
            ep_race_mode: i32::from_le_bytes(read_array(bytes, 4)),
            ep_ticket_item_slot_num: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for EpRaceStartRequest0104 {
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

/// `sP_CL2FE_REQ_EP_RACE_END` (`#pragma pack(4)`, 8 bytes, packet ID `0x1300005c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceEndRequest0104 {
    /// `iEndEcomID` at offset 0.
    pub end_ecom_id: i32,
    /// `iEPTicketItemSlotNum` at offset 4.
    pub ep_ticket_item_slot_num: i32,
}

impl EpRaceEndRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.end_ecom_id.to_le_bytes());
        write_prim(out, 4, &self.ep_ticket_item_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            end_ecom_id: i32::from_le_bytes(read_array(bytes, 0)),
            ep_ticket_item_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for EpRaceEndRequest0104 {
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

/// `sP_CL2FE_REQ_EP_RACE_CANCEL` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300005d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRaceCancelRequest0104 {
    /// `iStartEcomID` at offset 0.
    pub start_ecom_id: i32,
}

impl EpRaceCancelRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.start_ecom_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            start_ecom_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for EpRaceCancelRequest0104 {
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

/// `sP_CL2FE_REQ_EP_GET_RING` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300005e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpGetRingRequest0104 {
    /// `iRingLID` at offset 0.
    pub ring_lid: i32,
}

impl EpGetRingRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.ring_lid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            ring_lid: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for EpGetRingRequest0104 {
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

/// `sP_CL2FE_REQ_IM_CHANGE_SWITCH_STATUS` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300005f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ImChangeSwitchStatusRequest0104 {
    /// `iSwitchLID` at offset 0.
    pub switch_lid: i32,
}

impl ImChangeSwitchStatusRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.switch_lid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            switch_lid: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for ImChangeSwitchStatusRequest0104 {
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

/// `sP_CL2FE_REQ_SHINY_PICKUP` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000060`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyPickupRequest0104 {
    /// `iShinyID` at offset 0.
    pub shiny_id: i32,
}

impl ShinyPickupRequest0104 {
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

impl WirePayload for ShinyPickupRequest0104 {
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

/// `sP_CL2FE_REQ_SHINY_SUMMON` (`#pragma pack(4)`, 16 bytes, packet ID `0x13000061`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinySummonRequest0104 {
    /// `iShinyType` at offset 0.
    pub shiny_type: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
}

impl ShinySummonRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.shiny_type.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_type: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for ShinySummonRequest0104 {
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

/// `sP_CL2FE_REQ_PC_MOVETRANSPORTATION` (`#pragma pack(4)`, 60 bytes, packet ID `0x13000062`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMovetransportationRequest0104 {
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
}

impl PcMovetransportationRequest0104 {
    pub const SIZE: usize = 60;

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
        }
    }
}

impl WirePayload for PcMovetransportationRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE` (`#pragma pack(4)`, 260 bytes, packet ID `0x13000063`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAllGroupFreechatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
}

impl SendAllGroupFreechatMessageRequest0104 {
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

impl WirePayload for SendAllGroupFreechatMessageRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_ANY_GROUP_FREECHAT_MESSAGE` (`#pragma pack(4)`, 264 bytes, packet ID `0x13000064`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAnyGroupFreechatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
    /// `iGroupPC_ID` at offset 260.
    pub group_pc_id: i32,
}

impl SendAnyGroupFreechatMessageRequest0104 {
    pub const SIZE: usize = 264;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.free_chat);
        write_prim(out, 256, &self.emote_code.to_le_bytes());
        write_prim(out, 260, &self.group_pc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            free_chat: read_utf16(bytes, 0),
            emote_code: i32::from_le_bytes(read_array(bytes, 256)),
            group_pc_id: i32::from_le_bytes(read_array(bytes, 260)),
        }
    }
}

impl WirePayload for SendAnyGroupFreechatMessageRequest0104 {
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
