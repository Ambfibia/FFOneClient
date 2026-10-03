// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REP_LIVE_CHECK` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000075`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LiveCheckReply0104 {
    /// `iTempValue` at offset 0.
    pub temp_value: i32,
}

impl LiveCheckReply0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.temp_value.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            temp_value: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for LiveCheckReply0104 {
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

/// `sP_CL2FE_REQ_PC_MISSION_COMPLETE` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000076`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMissionCompleteRequest0104 {
    /// `iMissionNum` at offset 0.
    pub mission_num: i32,
}

impl PcMissionCompleteRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.mission_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            mission_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcMissionCompleteRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TASK_COMPLETE` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000077`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskCompleteRequest0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
}

impl PcTaskCompleteRequest0104 {
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

impl WirePayload for PcTaskCompleteRequest0104 {
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

/// `sP_CL2FE_REQ_NPC_INTERACTION` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000078`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcInteractionRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `bFlag` at offset 4.
    pub flag: i32,
}

impl NpcInteractionRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            flag: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NpcInteractionRequest0104 {
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

/// `sP_CL2FE_DOT_HEAL_ONOFF` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000079`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct DotHealOnoff0104 {
    /// `iFlag` at offset 0.
    pub flag: i32,
}

impl DotHealOnoff0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            flag: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for DotHealOnoff0104 {
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

/// `sP_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH` (`#pragma pack(4)`, 8 bytes, packet ID `0x1300007a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSpecialStateSwitchRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSpecialStateFlag` at offset 4.
    pub special_state_flag: i8,
}

impl PcSpecialStateSwitchRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.special_state_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            special_state_flag: i8::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcSpecialStateSwitchRequest0104 {
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

/// `sP_CL2FE_REQ_PC_EMAIL_UPDATE_CHECK` (`#pragma pack(8)`, 1 bytes, packet ID `0x1300007b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcEmailUpdateCheckRequest0104;

impl PcEmailUpdateCheckRequest0104 {
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

impl WirePayload for PcEmailUpdateCheckRequest0104 {
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

/// `sP_CL2FE_REQ_PC_READ_EMAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x1300007c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcReadEmailRequest0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
}

impl PcReadEmailRequest0104 {
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

impl WirePayload for PcReadEmailRequest0104 {
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

/// `sP_CL2FE_REQ_PC_RECV_EMAIL_PAGE_LIST` (`#pragma pack(1)`, 1 bytes, packet ID `0x1300007d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailPageListRequest0104 {
    /// `iPageNum` at offset 0.
    pub page_num: i8,
}

impl PcRecvEmailPageListRequest0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.page_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            page_num: i8::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcRecvEmailPageListRequest0104 {
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

/// `sP_CL2FE_REQ_PC_DELETE_EMAIL` (`#pragma pack(4)`, 40 bytes, packet ID `0x1300007e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcDeleteEmailRequest0104 {
    /// `iEmailIndexArray` at offset 0.
    pub email_index_array: [i64; 5],
}

impl PcDeleteEmailRequest0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.email_index_array.iter().enumerate() {
            write_prim(out, 0 + index * 8, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index_array: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 0 + index * 8))
            }),
        }
    }
}

impl WirePayload for PcDeleteEmailRequest0104 {
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

/// `sP_CL2FE_REQ_PC_SEND_EMAIL` (`#pragma pack(4)`, 1164 bytes, packet ID `0x1300007f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSendEmailRequest0104 {
    /// `iTo_PCUID` at offset 0.
    pub to_pcuid: i64,
    /// `szSubject` at offset 8.
    pub subject: FixedUtf16<32>,
    /// `szContent` at offset 72.
    pub content: FixedUtf16<512>,
    /// `aItem` at offset 1096.
    pub item: [EmailItemInfoFromClient0104; 4],
    /// `iCash` at offset 1160.
    pub cash: i32,
}

impl PcSendEmailRequest0104 {
    pub const SIZE: usize = 1164;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.to_pcuid.to_le_bytes());
        write_utf16(out, 8, &self.subject);
        write_utf16(out, 72, &self.content);
        for (index, value) in self.item.iter().enumerate() {
            let start = 1096 + index * 16;
            value.write_into(&mut out[start..start + 16]);
        }
        write_prim(out, 1160, &self.cash.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            to_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            subject: read_utf16(bytes, 8),
            content: read_utf16(bytes, 72),
            item: std::array::from_fn(|index| {
                let start = 1096 + index * 16;
                EmailItemInfoFromClient0104::read_from(&bytes[start..start + 16])
            }),
            cash: i32::from_le_bytes(read_array(bytes, 1160)),
        }
    }
}

impl WirePayload for PcSendEmailRequest0104 {
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

/// `sP_CL2FE_REQ_PC_RECV_EMAIL_ITEM` (`#pragma pack(4)`, 16 bytes, packet ID `0x13000080`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailItemRequest0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i32,
    /// `iEmailItemSlot` at offset 12.
    pub email_item_slot: i32,
}

impl PcRecvEmailItemRequest0104 {
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

impl WirePayload for PcRecvEmailItemRequest0104 {
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

/// `sP_CL2FE_REQ_PC_RECV_EMAIL_CANDY` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000081`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailCandyRequest0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
}

impl PcRecvEmailCandyRequest0104 {
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

impl WirePayload for PcRecvEmailCandyRequest0104 {
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

/// `sP_CL2FE_GM_REQ_TARGET_PC_SPECIAL_STATE_ONOFF` (`#pragma pack(4)`, 80 bytes, packet ID `0x13000082`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmTargetPcSpecialStateOnoffRequest0104 {
    /// `eTargetSearchBy` at offset 0.
    pub target_search_by: i32,
    /// `iTargetPC_ID` at offset 4.
    pub target_pc_id: i32,
    /// `szTargetPC_FirstName` at offset 8.
    pub target_pc_first_name: FixedUtf16<10>,
    /// `szTargetPC_LastName` at offset 28.
    pub target_pc_last_name: FixedUtf16<18>,
    /// `iTargetPC_UID` at offset 64.
    pub target_pc_uid: i64,
    /// `iONOFF` at offset 72.
    pub onoff: i32,
    /// `iSpecialStateFlag` at offset 76.
    pub special_state_flag: i8,
}

impl GmTargetPcSpecialStateOnoffRequest0104 {
    pub const SIZE: usize = 80;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.target_search_by.to_le_bytes());
        write_prim(out, 4, &self.target_pc_id.to_le_bytes());
        write_utf16(out, 8, &self.target_pc_first_name);
        write_utf16(out, 28, &self.target_pc_last_name);
        write_prim(out, 64, &self.target_pc_uid.to_le_bytes());
        write_prim(out, 72, &self.onoff.to_le_bytes());
        write_prim(out, 76, &self.special_state_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            target_search_by: i32::from_le_bytes(read_array(bytes, 0)),
            target_pc_id: i32::from_le_bytes(read_array(bytes, 4)),
            target_pc_first_name: read_utf16(bytes, 8),
            target_pc_last_name: read_utf16(bytes, 28),
            target_pc_uid: i64::from_le_bytes(read_array(bytes, 64)),
            onoff: i32::from_le_bytes(read_array(bytes, 72)),
            special_state_flag: i8::from_le_bytes(read_array(bytes, 76)),
        }
    }
}

impl WirePayload for GmTargetPcSpecialStateOnoffRequest0104 {
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

/// `sP_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000083`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSetCurrentMissionIdRequest0104 {
    /// `iCurrentMissionID` at offset 0.
    pub current_mission_id: i32,
}

impl PcSetCurrentMissionIdRequest0104 {
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

impl WirePayload for PcSetCurrentMissionIdRequest0104 {
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

/// `sP_CL2FE_REQ_NPC_GROUP_INVITE` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000084`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupInviteRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
}

impl NpcGroupInviteRequest0104 {
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

impl WirePayload for NpcGroupInviteRequest0104 {
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

/// `sP_CL2FE_REQ_NPC_GROUP_KICK` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000085`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupKickRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
}

impl NpcGroupKickRequest0104 {
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

impl WirePayload for NpcGroupKickRequest0104 {
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

/// `sP_CL2FE_REQ_PC_FIRST_USE_FLAG_SET` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000086`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcFirstUseFlagSetRequest0104 {
    /// `iFlagCode` at offset 0.
    pub flag_code: i32,
}

impl PcFirstUseFlagSetRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.flag_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            flag_code: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcFirstUseFlagSetRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRANSPORT_WARP` (`#pragma pack(4)`, 16 bytes, packet ID `0x13000087`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTransportWarpRequest0104 {
    /// `iTransport_ID` at offset 0.
    pub transport_id: i32,
    /// `iLcX` at offset 4.
    pub lc_x: i32,
    /// `iLcY` at offset 8.
    pub lc_y: i32,
    /// `iLcZ` at offset 12.
    pub lc_z: i32,
}

impl PcTransportWarpRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.transport_id.to_le_bytes());
        write_prim(out, 4, &self.lc_x.to_le_bytes());
        write_prim(out, 8, &self.lc_y.to_le_bytes());
        write_prim(out, 12, &self.lc_z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            transport_id: i32::from_le_bytes(read_array(bytes, 0)),
            lc_x: i32::from_le_bytes(read_array(bytes, 4)),
            lc_y: i32::from_le_bytes(read_array(bytes, 8)),
            lc_z: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTransportWarpRequest0104 {
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
