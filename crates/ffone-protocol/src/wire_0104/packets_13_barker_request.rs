// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_BARKER` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000065`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct BarkerRequest0104 {
    /// `iMissionTaskID` at offset 0.
    pub mission_task_id: i32,
    /// `iNPC_ID` at offset 4.
    pub npc_id: i32,
}

impl BarkerRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.mission_task_id.to_le_bytes());
        write_prim(out, 4, &self.npc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            mission_task_id: i32::from_le_bytes(read_array(bytes, 0)),
            npc_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for BarkerRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE` (`#pragma pack(4)`, 260 bytes, packet ID `0x13000066`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAllGroupMenuchatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
}

impl SendAllGroupMenuchatMessageRequest0104 {
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

impl WirePayload for SendAllGroupMenuchatMessageRequest0104 {
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

/// `sP_CL2FE_REQ_SEND_ANY_GROUP_MENUCHAT_MESSAGE` (`#pragma pack(4)`, 264 bytes, packet ID `0x13000067`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAnyGroupMenuchatMessageRequest0104 {
    /// `szFreeChat` at offset 0.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 256.
    pub emote_code: i32,
    /// `iGroupPC_ID` at offset 260.
    pub group_pc_id: i32,
}

impl SendAnyGroupMenuchatMessageRequest0104 {
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

impl WirePayload for SendAnyGroupMenuchatMessageRequest0104 {
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

/// `sP_CL2FE_REQ_REGIST_TRANSPORTATION_LOCATION` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000068`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistTransportationLocationRequest0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iNPC_ID` at offset 4.
    pub npc_id: i32,
    /// `iLocationID` at offset 8.
    pub location_id: i32,
}

impl RegistTransportationLocationRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.npc_id.to_le_bytes());
        write_prim(out, 8, &self.location_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            npc_id: i32::from_le_bytes(read_array(bytes, 4)),
            location_id: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for RegistTransportationLocationRequest0104 {
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

/// `sP_CL2FE_REQ_PC_WARP_USE_TRANSPORTATION` (`#pragma pack(4)`, 20 bytes, packet ID `0x13000069`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: source declares Size = 16 but the fields need 20 bytes; Mono marshals max(explicit, computed) = 20.
///
/// OpenFusion divergence: the pinned `structs/0104.hpp` declares this struct as 16 bytes. Wiring code must choose the server-compatible length explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpUseTransportationRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iTransporationID` at offset 4.
    pub transporation_id: i32,
    /// `eIL` at offset 8.
    pub e_il: i32,
    /// `iSlotNum` at offset 12.
    pub slot_num: i32,
    /// `iTurbo` at offset 16.
    pub turbo: i32,
}

impl PcWarpUseTransportationRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.transporation_id.to_le_bytes());
        write_prim(out, 8, &self.e_il.to_le_bytes());
        write_prim(out, 12, &self.slot_num.to_le_bytes());
        write_prim(out, 16, &self.turbo.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            transporation_id: i32::from_le_bytes(read_array(bytes, 4)),
            e_il: i32::from_le_bytes(read_array(bytes, 8)),
            slot_num: i32::from_le_bytes(read_array(bytes, 12)),
            turbo: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcWarpUseTransportationRequest0104 {
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

/// `sP_CL2FE_GM_REQ_PC_SPECIAL_STATE_SWITCH` (`#pragma pack(4)`, 8 bytes, packet ID `0x1300006a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcSpecialStateSwitchRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSpecialStateFlag` at offset 4.
    pub special_state_flag: i8,
}

impl GmPcSpecialStateSwitchRequest0104 {
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

impl WirePayload for GmPcSpecialStateSwitchRequest0104 {
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

/// `sP_CL2FE_GM_REQ_PC_SET_VALUE` (`#pragma pack(4)`, 12 bytes, packet ID `0x1300006b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcSetValueRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSetValueType` at offset 4.
    pub set_value_type: i32,
    /// `iSetValue` at offset 8.
    pub set_value: i32,
}

impl GmPcSetValueRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.set_value_type.to_le_bytes());
        write_prim(out, 8, &self.set_value.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            set_value_type: i32::from_le_bytes(read_array(bytes, 4)),
            set_value: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for GmPcSetValueRequest0104 {
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

/// `sP_CL2FE_GM_REQ_KICK_PLAYER` (`#pragma pack(4)`, 76 bytes, packet ID `0x1300006c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmKickPlayerRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `eTargetSearchBy` at offset 4.
    pub target_search_by: i32,
    /// `iTargetPC_ID` at offset 8.
    pub target_pc_id: i32,
    /// `szTargetPC_FirstName` at offset 12.
    pub target_pc_first_name: FixedUtf16<10>,
    /// `szTargetPC_LastName` at offset 32.
    pub target_pc_last_name: FixedUtf16<18>,
    /// `iTargetPC_UID` at offset 68.
    pub target_pc_uid: i64,
}

impl GmKickPlayerRequest0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.target_search_by.to_le_bytes());
        write_prim(out, 8, &self.target_pc_id.to_le_bytes());
        write_utf16(out, 12, &self.target_pc_first_name);
        write_utf16(out, 32, &self.target_pc_last_name);
        write_prim(out, 68, &self.target_pc_uid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            target_search_by: i32::from_le_bytes(read_array(bytes, 4)),
            target_pc_id: i32::from_le_bytes(read_array(bytes, 8)),
            target_pc_first_name: read_utf16(bytes, 12),
            target_pc_last_name: read_utf16(bytes, 32),
            target_pc_uid: i64::from_le_bytes(read_array(bytes, 68)),
        }
    }
}

impl WirePayload for GmKickPlayerRequest0104 {
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

/// `sP_CL2FE_GM_REQ_TARGET_PC_TELEPORT` (`#pragma pack(4)`, 172 bytes, packet ID `0x1300006d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmTargetPcTeleportRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `eTargetPCSearchBy` at offset 4.
    pub target_pc_search_by: i32,
    /// `iTargetPC_ID` at offset 8.
    pub target_pc_id: i32,
    /// `szTargetPC_FirstName` at offset 12.
    pub target_pc_first_name: FixedUtf16<10>,
    /// `szTargetPC_LastName` at offset 32.
    pub target_pc_last_name: FixedUtf16<18>,
    /// `iTargetPC_UID` at offset 68.
    pub target_pc_uid: i64,
    /// `eTeleportType` at offset 76.
    pub teleport_type: i32,
    /// `iToMapType` at offset 80.
    pub to_map_type: i32,
    /// `iToMap` at offset 84.
    pub to_map: i32,
    /// `iToX` at offset 88.
    pub to_x: i32,
    /// `iToY` at offset 92.
    pub to_y: i32,
    /// `iToZ` at offset 96.
    pub to_z: i32,
    /// `eGoalPCSearchBy` at offset 100.
    pub goal_pc_search_by: i32,
    /// `iGoalPC_ID` at offset 104.
    pub goal_pc_id: i32,
    /// `szGoalPC_FirstName` at offset 108.
    pub goal_pc_first_name: FixedUtf16<10>,
    /// `szGoalPC_LastName` at offset 128.
    pub goal_pc_last_name: FixedUtf16<18>,
    /// `iGoalPC_UID` at offset 164.
    pub goal_pc_uid: i64,
}

impl GmTargetPcTeleportRequest0104 {
    pub const SIZE: usize = 172;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.target_pc_search_by.to_le_bytes());
        write_prim(out, 8, &self.target_pc_id.to_le_bytes());
        write_utf16(out, 12, &self.target_pc_first_name);
        write_utf16(out, 32, &self.target_pc_last_name);
        write_prim(out, 68, &self.target_pc_uid.to_le_bytes());
        write_prim(out, 76, &self.teleport_type.to_le_bytes());
        write_prim(out, 80, &self.to_map_type.to_le_bytes());
        write_prim(out, 84, &self.to_map.to_le_bytes());
        write_prim(out, 88, &self.to_x.to_le_bytes());
        write_prim(out, 92, &self.to_y.to_le_bytes());
        write_prim(out, 96, &self.to_z.to_le_bytes());
        write_prim(out, 100, &self.goal_pc_search_by.to_le_bytes());
        write_prim(out, 104, &self.goal_pc_id.to_le_bytes());
        write_utf16(out, 108, &self.goal_pc_first_name);
        write_utf16(out, 128, &self.goal_pc_last_name);
        write_prim(out, 164, &self.goal_pc_uid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            target_pc_search_by: i32::from_le_bytes(read_array(bytes, 4)),
            target_pc_id: i32::from_le_bytes(read_array(bytes, 8)),
            target_pc_first_name: read_utf16(bytes, 12),
            target_pc_last_name: read_utf16(bytes, 32),
            target_pc_uid: i64::from_le_bytes(read_array(bytes, 68)),
            teleport_type: i32::from_le_bytes(read_array(bytes, 76)),
            to_map_type: i32::from_le_bytes(read_array(bytes, 80)),
            to_map: i32::from_le_bytes(read_array(bytes, 84)),
            to_x: i32::from_le_bytes(read_array(bytes, 88)),
            to_y: i32::from_le_bytes(read_array(bytes, 92)),
            to_z: i32::from_le_bytes(read_array(bytes, 96)),
            goal_pc_search_by: i32::from_le_bytes(read_array(bytes, 100)),
            goal_pc_id: i32::from_le_bytes(read_array(bytes, 104)),
            goal_pc_first_name: read_utf16(bytes, 108),
            goal_pc_last_name: read_utf16(bytes, 128),
            goal_pc_uid: i64::from_le_bytes(read_array(bytes, 164)),
        }
    }
}

impl WirePayload for GmTargetPcTeleportRequest0104 {
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

/// `sP_CL2FE_GM_REQ_PC_LOCATION` (`#pragma pack(4)`, 72 bytes, packet ID `0x1300006e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcLocationRequest0104 {
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
}

impl GmPcLocationRequest0104 {
    pub const SIZE: usize = 72;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.target_search_by.to_le_bytes());
        write_prim(out, 4, &self.target_pc_id.to_le_bytes());
        write_utf16(out, 8, &self.target_pc_first_name);
        write_utf16(out, 28, &self.target_pc_last_name);
        write_prim(out, 64, &self.target_pc_uid.to_le_bytes());
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
        }
    }
}

impl WirePayload for GmPcLocationRequest0104 {
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

/// `sP_CL2FE_GM_REQ_PC_ANNOUNCE` (`#pragma pack(4)`, 1032 bytes, packet ID `0x1300006f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcAnnounceRequest0104 {
    /// `iAreaType` at offset 0.
    pub area_type: i8,
    /// `iAnnounceType` at offset 1.
    pub announce_type: i8,
    /// `iDuringTime` at offset 4.
    pub during_time: i32,
    /// `szAnnounceMsg` at offset 8.
    pub announce_msg: FixedUtf16<512>,
}

impl GmPcAnnounceRequest0104 {
    pub const SIZE: usize = 1032;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.area_type.to_le_bytes());
        write_prim(out, 1, &self.announce_type.to_le_bytes());
        write_prim(out, 4, &self.during_time.to_le_bytes());
        write_utf16(out, 8, &self.announce_msg);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            area_type: i8::from_le_bytes(read_array(bytes, 0)),
            announce_type: i8::from_le_bytes(read_array(bytes, 1)),
            during_time: i32::from_le_bytes(read_array(bytes, 4)),
            announce_msg: read_utf16(bytes, 8),
        }
    }
}

impl WirePayload for GmPcAnnounceRequest0104 {
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

/// `sP_CL2FE_REQ_SET_PC_BLOCK` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000070`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SetPcBlockRequest0104 {
    /// `iBlock_ID` at offset 0.
    pub block_id: i32,
    /// `iBlock_PCUID` at offset 4.
    pub block_pcuid: i64,
}

impl SetPcBlockRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.block_id.to_le_bytes());
        write_prim(out, 4, &self.block_pcuid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            block_id: i32::from_le_bytes(read_array(bytes, 0)),
            block_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for SetPcBlockRequest0104 {
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

/// `sP_CL2FE_REQ_REGIST_RXCOM` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000071`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistRxcomRequest0104 {
    /// `iNPCID` at offset 0.
    pub npcid: i32,
}

impl RegistRxcomRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npcid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npcid: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for RegistRxcomRequest0104 {
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

/// `sP_CL2FE_GM_REQ_PC_MOTD_REGISTER` (`#pragma pack(2)`, 1026 bytes, packet ID `0x13000072`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcMotdRegisterRequest0104 {
    /// `iType` at offset 0.
    pub type_: i8,
    /// `szSystemMsg` at offset 2.
    pub system_msg: FixedUtf16<512>,
}

impl GmPcMotdRegisterRequest0104 {
    pub const SIZE: usize = 1026;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.type_.to_le_bytes());
        write_utf16(out, 2, &self.system_msg);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            type_: i8::from_le_bytes(read_array(bytes, 0)),
            system_msg: read_utf16(bytes, 2),
        }
    }
}

impl WirePayload for GmPcMotdRegisterRequest0104 {
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

/// `sP_CL2FE_REQ_ITEM_USE` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000073`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemUseRequest0104 {
    /// `eIL` at offset 0.
    pub e_il: i32,
    /// `iSlotNum` at offset 4.
    pub slot_num: i32,
    /// `iNanoSlot` at offset 8.
    pub nano_slot: i16,
}

impl ItemUseRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_il.to_le_bytes());
        write_prim(out, 4, &self.slot_num.to_le_bytes());
        write_prim(out, 8, &self.nano_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_il: i32::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            nano_slot: i16::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for ItemUseRequest0104 {
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

/// `sP_CL2FE_REQ_WARP_USE_RECALL` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000074`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct WarpUseRecallRequest0104 {
    /// `iGroupMemberID` at offset 0.
    pub group_member_id: i32,
}

impl WarpUseRecallRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.group_member_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            group_member_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for WarpUseRecallRequest0104 {
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
