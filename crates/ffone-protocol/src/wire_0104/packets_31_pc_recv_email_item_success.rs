// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_PC_RECV_EMAIL_ITEM_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000e9`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailItemSuccess0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i32,
    /// `iEmailItemSlot` at offset 12.
    pub email_item_slot: i32,
}

impl PcRecvEmailItemSuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_prim(out, 12, &self.email_item_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 8)),
            email_item_slot: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcRecvEmailItemSuccess0104 {
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

/// `sP_FE2CL_REP_PC_RECV_EMAIL_ITEM_FAIL` (`#pragma pack(4)`, 20 bytes, packet ID `0x310000ea`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailItemFailure0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i32,
    /// `iEmailItemSlot` at offset 12.
    pub email_item_slot: i32,
    /// `iErrorCode` at offset 16.
    pub error_code: i32,
}

impl PcRecvEmailItemFailure0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_prim(out, 12, &self.email_item_slot.to_le_bytes());
        write_prim(out, 16, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 8)),
            email_item_slot: i32::from_le_bytes(read_array(bytes, 12)),
            error_code: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcRecvEmailItemFailure0104 {
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

/// `sP_FE2CL_REP_PC_RECV_EMAIL_CANDY_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000eb`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailCandySuccess0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iCandy` at offset 8.
    pub candy: i32,
}

impl PcRecvEmailCandySuccess0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_prim(out, 8, &self.candy.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            candy: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcRecvEmailCandySuccess0104 {
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

/// `sP_FE2CL_REP_PC_RECV_EMAIL_CANDY_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000ec`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailCandyFailure0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl PcRecvEmailCandyFailure0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_prim(out, 8, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcRecvEmailCandyFailure0104 {
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

/// `sP_FE2CL_PC_SUDDEN_DEAD` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000ed`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSuddenDead0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSuddenDeadReason` at offset 4.
    pub sudden_dead_reason: i32,
    /// `iDamage` at offset 8.
    pub damage: i32,
    /// `iHP` at offset 12.
    pub hp: i32,
}

impl PcSuddenDead0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.sudden_dead_reason.to_le_bytes());
        write_prim(out, 8, &self.damage.to_le_bytes());
        write_prim(out, 12, &self.hp.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            sudden_dead_reason: i32::from_le_bytes(read_array(bytes, 4)),
            damage: i32::from_le_bytes(read_array(bytes, 8)),
            hp: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcSuddenDead0104 {
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

/// `sP_FE2CL_REP_GM_REQ_TARGET_PC_SPECIAL_STATE_ONOFF_SUCC` (`#pragma pack(4)`, 64 bytes, packet ID `0x310000ee`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmTargetPcSpecialStateOnoffSuccess0104 {
    /// `iTargetPC_ID` at offset 0.
    pub target_pc_id: i32,
    /// `szTargetPC_FirstName` at offset 4.
    pub target_pc_first_name: FixedUtf16<10>,
    /// `szTargetPC_LastName` at offset 24.
    pub target_pc_last_name: FixedUtf16<18>,
    /// `iReqSpecialStateFlag` at offset 60.
    pub req_special_state_flag: i8,
    /// `iSpecialState` at offset 61.
    pub special_state: i8,
}

impl GmTargetPcSpecialStateOnoffSuccess0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.target_pc_id.to_le_bytes());
        write_utf16(out, 4, &self.target_pc_first_name);
        write_utf16(out, 24, &self.target_pc_last_name);
        write_prim(out, 60, &self.req_special_state_flag.to_le_bytes());
        write_prim(out, 61, &self.special_state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            target_pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            target_pc_first_name: read_utf16(bytes, 4),
            target_pc_last_name: read_utf16(bytes, 24),
            req_special_state_flag: i8::from_le_bytes(read_array(bytes, 60)),
            special_state: i8::from_le_bytes(read_array(bytes, 61)),
        }
    }
}

impl WirePayload for GmTargetPcSpecialStateOnoffSuccess0104 {
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

/// `sP_FE2CL_REP_PC_SET_CURRENT_MISSION_ID` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000ef`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSetCurrentMissionIdReply0104 {
    /// `iCurrentMissionID` at offset 0.
    pub current_mission_id: i32,
}

impl PcSetCurrentMissionIdReply0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.current_mission_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            current_mission_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcSetCurrentMissionIdReply0104 {
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

/// `sP_FE2CL_REP_NPC_GROUP_INVITE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000f0`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupInviteFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl NpcGroupInviteFailure0104 {
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

impl WirePayload for NpcGroupInviteFailure0104 {
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

/// `sP_FE2CL_REP_NPC_GROUP_INVITE_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000f1`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupInviteSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iNPC_ID` at offset 4.
    pub npc_id: i32,
    /// `iMemberPCCnt` at offset 8.
    pub member_pc_cnt: i32,
    /// `iMemberNPCCnt` at offset 12.
    pub member_npc_cnt: i32,
}

impl NpcGroupInviteSuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.npc_id.to_le_bytes());
        write_prim(out, 8, &self.member_pc_cnt.to_le_bytes());
        write_prim(out, 12, &self.member_npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            npc_id: i32::from_le_bytes(read_array(bytes, 4)),
            member_pc_cnt: i32::from_le_bytes(read_array(bytes, 8)),
            member_npc_cnt: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for NpcGroupInviteSuccess0104 {
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

/// `sP_FE2CL_REP_NPC_GROUP_KICK_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000f2`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupKickFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl NpcGroupKickFailure0104 {
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

impl WirePayload for NpcGroupKickFailure0104 {
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

/// `sP_FE2CL_REP_NPC_GROUP_KICK_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000f3`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupKickSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iNPC_ID` at offset 4.
    pub npc_id: i32,
    /// `iMemberPCCnt` at offset 8.
    pub member_pc_cnt: i32,
    /// `iMemberNPCCnt` at offset 12.
    pub member_npc_cnt: i32,
}

impl NpcGroupKickSuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.npc_id.to_le_bytes());
        write_prim(out, 8, &self.member_pc_cnt.to_le_bytes());
        write_prim(out, 12, &self.member_npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            npc_id: i32::from_le_bytes(read_array(bytes, 4)),
            member_pc_cnt: i32::from_le_bytes(read_array(bytes, 8)),
            member_npc_cnt: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for NpcGroupKickSuccess0104 {
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

/// `sP_FE2CL_PC_EVENT` (`#pragma pack(4)`, 20 bytes, packet ID `0x310000f4`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcEvent0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iEventID` at offset 4.
    pub event_id: i32,
    /// `iEventValue1` at offset 8.
    pub event_value1: i32,
    /// `iEventValue2` at offset 12.
    pub event_value2: i32,
    /// `iEventValue3` at offset 16.
    pub event_value3: i32,
}

impl PcEvent0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.event_id.to_le_bytes());
        write_prim(out, 8, &self.event_value1.to_le_bytes());
        write_prim(out, 12, &self.event_value2.to_le_bytes());
        write_prim(out, 16, &self.event_value3.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            event_id: i32::from_le_bytes(read_array(bytes, 4)),
            event_value1: i32::from_le_bytes(read_array(bytes, 8)),
            event_value2: i32::from_le_bytes(read_array(bytes, 12)),
            event_value3: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcEvent0104 {
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

/// `sP_FE2CL_REP_PC_TRANSPORT_WARP_SUCC` (`#pragma pack(4)`, 36 bytes, packet ID `0x310000f5`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTransportWarpSuccess0104 {
    /// `TransportationAppearanceData` at offset 0.
    pub transportation_appearance_data: TransportationAppearanceData0104,
    /// `iLcX` at offset 24.
    pub lc_x: i32,
    /// `iLcY` at offset 28.
    pub lc_y: i32,
    /// `iLcZ` at offset 32.
    pub lc_z: i32,
}

impl PcTransportWarpSuccess0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.transportation_appearance_data
            .write_into(&mut out[0..24]);
        write_prim(out, 24, &self.lc_x.to_le_bytes());
        write_prim(out, 28, &self.lc_y.to_le_bytes());
        write_prim(out, 32, &self.lc_z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            transportation_appearance_data: TransportationAppearanceData0104::read_from(
                &bytes[0..24],
            ),
            lc_x: i32::from_le_bytes(read_array(bytes, 24)),
            lc_y: i32::from_le_bytes(read_array(bytes, 28)),
            lc_z: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for PcTransportWarpSuccess0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_EMOTES_CHAT_FAIL` (`#pragma pack(4)`, 276 bytes, packet ID `0x310000f6`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeEmotesChatFailure0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `szFreeChat` at offset 12.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 268.
    pub emote_code: i32,
    /// `iErrorCode` at offset 272.
    pub error_code: i32,
}

impl PcTradeEmotesChatFailure0104 {
    pub const SIZE: usize = 276;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        write_utf16(out, 12, &self.free_chat);
        write_prim(out, 268, &self.emote_code.to_le_bytes());
        write_prim(out, 272, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            free_chat: read_utf16(bytes, 12),
            emote_code: i32::from_le_bytes(read_array(bytes, 268)),
            error_code: i32::from_le_bytes(read_array(bytes, 272)),
        }
    }
}

impl WirePayload for PcTradeEmotesChatFailure0104 {
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

/// `sP_FE2CL_REP_PC_RECV_EMAIL_ITEM_ALL_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000f7`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailItemAllSuccess0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
}

impl PcRecvEmailItemAllSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcRecvEmailItemAllSuccess0104 {
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

/// `sP_FE2CL_REP_PC_RECV_EMAIL_ITEM_ALL_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000f8`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailItemAllFailure0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl PcRecvEmailItemAllFailure0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_prim(out, 8, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcRecvEmailItemAllFailure0104 {
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

/// `sP_FE2CL_REP_PC_LOADING_COMPLETE_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000f9`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcLoadingCompleteSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl PcLoadingCompleteSuccess0104 {
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

impl WirePayload for PcLoadingCompleteSuccess0104 {
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
