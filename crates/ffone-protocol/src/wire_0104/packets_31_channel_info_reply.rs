// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_CHANNEL_INFO` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000fa`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelInfoReply0104 {
    /// `iCurrChannelNum` at offset 0.
    pub curr_channel_num: i32,
    /// `iChannelCnt` at offset 4.
    pub channel_cnt: i32,
}

impl ChannelInfoReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.curr_channel_num.to_le_bytes());
        write_prim(out, 4, &self.channel_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            curr_channel_num: i32::from_le_bytes(read_array(bytes, 0)),
            channel_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for ChannelInfoReply0104 {
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

/// `sP_FE2CL_REP_PC_CHANNEL_NUM` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000fb`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcChannelNumReply0104 {
    /// `iChannelNum` at offset 0.
    pub channel_num: i32,
}

impl PcChannelNumReply0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.channel_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            channel_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcChannelNumReply0104 {
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

/// `sP_FE2CL_REP_PC_WARP_CHANNEL_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000fc`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpChannelFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcWarpChannelFailure0104 {
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

impl WirePayload for PcWarpChannelFailure0104 {
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

/// `sP_FE2CL_REP_PC_WARP_CHANNEL_SUCC` (`#pragma pack(8)`, 1 bytes, packet ID `0x310000fd`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpChannelSuccess0104;

impl PcWarpChannelSuccess0104 {
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

impl WirePayload for PcWarpChannelSuccess0104 {
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

/// `sP_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC` (`#pragma pack(4)`, 64 bytes, packet ID `0x310000fe`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcFindNameMakeBuddySuccess0104 {
    /// `szFirstName` at offset 0.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 18.
    pub last_name: FixedUtf16<17>,
    /// `iPCUID` at offset 52.
    pub pcuid: i64,
    /// `iNameCheckFlag` at offset 60.
    pub name_check_flag: i8,
}

impl PcFindNameMakeBuddySuccess0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.first_name);
        write_utf16(out, 18, &self.last_name);
        write_prim(out, 52, &self.pcuid.to_le_bytes());
        write_prim(out, 60, &self.name_check_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
            pcuid: i64::from_le_bytes(read_array(bytes, 52)),
            name_check_flag: i8::from_le_bytes(read_array(bytes, 60)),
        }
    }
}

impl WirePayload for PcFindNameMakeBuddySuccess0104 {
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

/// `sP_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL` (`#pragma pack(4)`, 56 bytes, packet ID `0x310000ff`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcFindNameMakeBuddyFailure0104 {
    /// `szFirstName` at offset 0.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 18.
    pub last_name: FixedUtf16<17>,
    /// `iErrorCode` at offset 52.
    pub error_code: i32,
}

impl PcFindNameMakeBuddyFailure0104 {
    pub const SIZE: usize = 56;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.first_name);
        write_utf16(out, 18, &self.last_name);
        write_prim(out, 52, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
            error_code: i32::from_le_bytes(read_array(bytes, 52)),
        }
    }
}

impl WirePayload for PcFindNameMakeBuddyFailure0104 {
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

/// `sP_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL` (`#pragma pack(4)`, 68 bytes, packet ID `0x31000100`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcFindNameAcceptBuddyFailure0104 {
    /// `szFirstName` at offset 0.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 18.
    pub last_name: FixedUtf16<17>,
    /// `iPCUID` at offset 52.
    pub pcuid: i64,
    /// `iNameCheckFlag` at offset 60.
    pub name_check_flag: i8,
    /// `iErrorCode` at offset 64.
    pub error_code: i32,
}

impl PcFindNameAcceptBuddyFailure0104 {
    pub const SIZE: usize = 68;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.first_name);
        write_utf16(out, 18, &self.last_name);
        write_prim(out, 52, &self.pcuid.to_le_bytes());
        write_prim(out, 60, &self.name_check_flag.to_le_bytes());
        write_prim(out, 64, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
            pcuid: i64::from_le_bytes(read_array(bytes, 52)),
            name_check_flag: i8::from_le_bytes(read_array(bytes, 60)),
            error_code: i32::from_le_bytes(read_array(bytes, 64)),
        }
    }
}

impl WirePayload for PcFindNameAcceptBuddyFailure0104 {
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

/// `sP_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC` (`#pragma pack(8)`, 1 bytes, packet ID `0x31000101`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBuddyWarpSameShardSuccess0104;

impl PcBuddyWarpSameShardSuccess0104 {
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

impl WirePayload for PcBuddyWarpSameShardSuccess0104 {
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

/// `sP_FE2CL_PC_ATTACK_CHARs_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000102`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAttackCharsSuccess0104 {
    /// `iBatteryW` at offset 0.
    pub battery_w: i32,
    /// `iTargetCnt` at offset 4.
    pub target_cnt: i32,
}

impl PcAttackCharsSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.battery_w.to_le_bytes());
        write_prim(out, 4, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            battery_w: i32::from_le_bytes(read_array(bytes, 0)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcAttackCharsSuccess0104 {
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

/// `sP_FE2CL_PC_ATTACK_CHARs` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000103`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAttackChars0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iTargetCnt` at offset 4.
    pub target_cnt: i32,
}

impl PcAttackChars0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcAttackChars0104 {
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

/// `sP_FE2CL_NPC_ATTACK_CHARs` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000104`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcAttackChars0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iTargetCnt` at offset 4.
    pub target_cnt: i32,
}

impl NpcAttackChars0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NpcAttackChars0104 {
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

/// `sP_FE2CL_REP_PC_CHANGE_LEVEL_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000105`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcChangeLevelSuccess0104 {
    /// `iLevel` at offset 0.
    pub level: i32,
    /// `iFusionMatter` at offset 4.
    pub fusion_matter: i32,
}

impl PcChangeLevelSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.level.to_le_bytes());
        write_prim(out, 4, &self.fusion_matter.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            level: i32::from_le_bytes(read_array(bytes, 0)),
            fusion_matter: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcChangeLevelSuccess0104 {
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

/// `sP_FE2CL_REP_PC_NANO_CREATE` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000106`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNanoCreateReply0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iNanoID` at offset 4.
    pub nano_id: i16,
}

impl PcNanoCreateReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.nano_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            nano_id: i16::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcNanoCreateReply0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_READY_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000107`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallReadySuccess0104 {
    /// `iStreetStallItemInvenSlotNum` at offset 0.
    pub street_stall_item_inven_slot_num: i32,
    /// `iItemListCountMax` at offset 4.
    pub item_list_count_max: i32,
    /// `fTaxPercentage` at offset 8.
    pub tax_percentage: f32,
    /// `iPCCharState` at offset 12.
    pub pc_char_state: i8,
}

impl PcStreetstallReadySuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_item_inven_slot_num.to_le_bytes());
        write_prim(out, 4, &self.item_list_count_max.to_le_bytes());
        write_prim(out, 8, &self.tax_percentage.to_le_bytes());
        write_prim(out, 12, &self.pc_char_state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            item_list_count_max: i32::from_le_bytes(read_array(bytes, 4)),
            tax_percentage: f32::from_le_bytes(read_array(bytes, 8)),
            pc_char_state: i8::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcStreetstallReadySuccess0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_READY_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000108`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallReadyFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcStreetstallReadyFailure0104 {
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

impl WirePayload for PcStreetstallReadyFailure0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_CANCEL_SUCC` (`#pragma pack(1)`, 1 bytes, packet ID `0x31000109`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallCancelSuccess0104 {
    /// `iPCCharState` at offset 0.
    pub pc_char_state: i8,
}

impl PcStreetstallCancelSuccess0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_char_state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_char_state: i8::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcStreetstallCancelSuccess0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_CANCEL_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100010a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallCancelFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcStreetstallCancelFailure0104 {
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

impl WirePayload for PcStreetstallCancelFailure0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_REGIST_ITEM_SUCC` (`#pragma pack(4)`, 24 bytes, packet ID `0x3100010b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallRegistItemSuccess0104 {
    /// `iItemListNum` at offset 0.
    pub item_list_num: i32,
    /// `iItemInvenSlotNum` at offset 4.
    pub item_inven_slot_num: i32,
    /// `Item` at offset 8.
    pub item: ItemBase0104,
    /// `iPrice` at offset 20.
    pub price: i32,
}

impl PcStreetstallRegistItemSuccess0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.item_list_num.to_le_bytes());
        write_prim(out, 4, &self.item_inven_slot_num.to_le_bytes());
        self.item.write_into(&mut out[8..20]);
        write_prim(out, 20, &self.price.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            item_list_num: i32::from_le_bytes(read_array(bytes, 0)),
            item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            item: ItemBase0104::read_from(&bytes[8..20]),
            price: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for PcStreetstallRegistItemSuccess0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_REGIST_ITEM_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100010c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallRegistItemFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcStreetstallRegistItemFailure0104 {
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

impl WirePayload for PcStreetstallRegistItemFailure0104 {
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
