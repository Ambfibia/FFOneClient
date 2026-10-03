// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_PC_TIME_TO_GO_WARP` (`#pragma pack(4)`, 32 bytes, packet ID `0x13000088`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTimeToGoWarpRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iWarpID` at offset 4.
    pub warp_id: i32,
    /// `eIL1` at offset 8.
    pub e_il1: i32,
    /// `iItemSlot1` at offset 12.
    pub item_slot1: i32,
    /// `eIL2` at offset 16.
    pub e_il2: i32,
    /// `iItemSlot2` at offset 20.
    pub item_slot2: i32,
    /// `iPC_Level` at offset 24.
    pub pc_level: i32,
    /// `iPayFlag` at offset 28.
    pub pay_flag: i32,
}

impl PcTimeToGoWarpRequest0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.warp_id.to_le_bytes());
        write_prim(out, 8, &self.e_il1.to_le_bytes());
        write_prim(out, 12, &self.item_slot1.to_le_bytes());
        write_prim(out, 16, &self.e_il2.to_le_bytes());
        write_prim(out, 20, &self.item_slot2.to_le_bytes());
        write_prim(out, 24, &self.pc_level.to_le_bytes());
        write_prim(out, 28, &self.pay_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            warp_id: i32::from_le_bytes(read_array(bytes, 4)),
            e_il1: i32::from_le_bytes(read_array(bytes, 8)),
            item_slot1: i32::from_le_bytes(read_array(bytes, 12)),
            e_il2: i32::from_le_bytes(read_array(bytes, 16)),
            item_slot2: i32::from_le_bytes(read_array(bytes, 20)),
            pc_level: i32::from_le_bytes(read_array(bytes, 24)),
            pay_flag: i32::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for PcTimeToGoWarpRequest0104 {
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

/// `sP_CL2FE_REQ_PC_RECV_EMAIL_ITEM_ALL` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000089`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailItemAllRequest0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
}

impl PcRecvEmailItemAllRequest0104 {
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

impl WirePayload for PcRecvEmailItemAllRequest0104 {
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

/// `sP_CL2FE_REQ_CHANNEL_INFO` (`#pragma pack(8)`, 1 bytes, packet ID `0x1300008a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelInfoRequest0104;

impl ChannelInfoRequest0104 {
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

impl WirePayload for ChannelInfoRequest0104 {
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

/// `sP_CL2FE_REQ_PC_CHANNEL_NUM` (`#pragma pack(8)`, 1 bytes, packet ID `0x1300008b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcChannelNumRequest0104;

impl PcChannelNumRequest0104 {
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

impl WirePayload for PcChannelNumRequest0104 {
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

/// `sP_CL2FE_REQ_PC_WARP_CHANNEL` (`#pragma pack(4)`, 8 bytes, packet ID `0x1300008c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpChannelRequest0104 {
    /// `iChannelNum` at offset 0.
    pub channel_num: i32,
    /// `iWarpType` at offset 4.
    pub warp_type: i8,
}

impl PcWarpChannelRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.channel_num.to_le_bytes());
        write_prim(out, 4, &self.warp_type.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            channel_num: i32::from_le_bytes(read_array(bytes, 0)),
            warp_type: i8::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcWarpChannelRequest0104 {
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

/// `sP_CL2FE_REQ_PC_LOADING_COMPLETE` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300008d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcLoadingCompleteRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl PcLoadingCompleteRequest0104 {
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

impl WirePayload for PcLoadingCompleteRequest0104 {
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

/// `sP_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY` (`#pragma pack(2)`, 52 bytes, packet ID `0x1300008e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcFindNameMakeBuddyRequest0104 {
    /// `szFirstName` at offset 0.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 18.
    pub last_name: FixedUtf16<17>,
}

impl PcFindNameMakeBuddyRequest0104 {
    pub const SIZE: usize = 52;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.first_name);
        write_utf16(out, 18, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
        }
    }
}

impl WirePayload for PcFindNameMakeBuddyRequest0104 {
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

/// `sP_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY` (`#pragma pack(4)`, 64 bytes, packet ID `0x1300008f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcFindNameAcceptBuddyRequest0104 {
    /// `iAcceptFlag` at offset 0.
    pub accept_flag: i32,
    /// `iBuddyPCUID` at offset 4.
    pub buddy_pcuid: i64,
    /// `szFirstName` at offset 12.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 30.
    pub last_name: FixedUtf16<17>,
}

impl PcFindNameAcceptBuddyRequest0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.accept_flag.to_le_bytes());
        write_prim(out, 4, &self.buddy_pcuid.to_le_bytes());
        write_utf16(out, 12, &self.first_name);
        write_utf16(out, 30, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            accept_flag: i32::from_le_bytes(read_array(bytes, 0)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            first_name: read_utf16(bytes, 12),
            last_name: read_utf16(bytes, 30),
        }
    }
}

impl WirePayload for PcFindNameAcceptBuddyRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ATTACK_CHARs` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000090`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAttackCharsRequest0104 {
    /// `iTargetCnt` at offset 0.
    pub target_cnt: i32,
}

impl PcAttackCharsRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            target_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcAttackCharsRequest0104 {
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

/// `sP_CL2FE_PC_STREETSTALL_REQ_READY` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000091`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallReadyRequest0104 {
    /// `iStreetStallItemInvenSlotNum` at offset 0.
    pub street_stall_item_inven_slot_num: i32,
}

impl PcStreetstallReadyRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_item_inven_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcStreetstallReadyRequest0104 {
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

/// `sP_CL2FE_PC_STREETSTALL_REQ_CANCEL` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000092`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallCancelRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl PcStreetstallCancelRequest0104 {
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

impl WirePayload for PcStreetstallCancelRequest0104 {
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

/// `sP_CL2FE_PC_STREETSTALL_REQ_REGIST_ITEM` (`#pragma pack(4)`, 24 bytes, packet ID `0x13000093`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallRegistItemRequest0104 {
    /// `iItemListNum` at offset 0.
    pub item_list_num: i32,
    /// `iItemInvenSlotNum` at offset 4.
    pub item_inven_slot_num: i32,
    /// `Item` at offset 8.
    pub item: ItemBase0104,
    /// `iPrice` at offset 20.
    pub price: i32,
}

impl PcStreetstallRegistItemRequest0104 {
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

impl WirePayload for PcStreetstallRegistItemRequest0104 {
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

/// `sP_CL2FE_PC_STREETSTALL_REQ_UNREGIST_ITEM` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000094`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallUnregistItemRequest0104 {
    /// `iItemListNum` at offset 0.
    pub item_list_num: i32,
}

impl PcStreetstallUnregistItemRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.item_list_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            item_list_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcStreetstallUnregistItemRequest0104 {
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

/// `sP_CL2FE_PC_STREETSTALL_REQ_SALE_START` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000095`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallSaleStartRequest0104 {
    /// `iStreetStallItemInvenSlotNum` at offset 0.
    pub street_stall_item_inven_slot_num: i32,
}

impl PcStreetstallSaleStartRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_item_inven_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcStreetstallSaleStartRequest0104 {
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

/// `sP_CL2FE_PC_STREETSTALL_REQ_ITEM_LIST` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000096`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallItemListRequest0104 {
    /// `iStreetStallPC_ID` at offset 0.
    pub street_stall_pc_id: i32,
}

impl PcStreetstallItemListRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_pc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_pc_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcStreetstallItemListRequest0104 {
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

/// `sP_CL2FE_PC_STREETSTALL_REQ_ITEM_BUY` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000097`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallItemBuyRequest0104 {
    /// `iStreetStallPC_ID` at offset 0.
    pub street_stall_pc_id: i32,
    /// `iItemListNum` at offset 4.
    pub item_list_num: i32,
    /// `iEmptyInvenSlotNum` at offset 8.
    pub empty_inven_slot_num: i32,
}

impl PcStreetstallItemBuyRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_pc_id.to_le_bytes());
        write_prim(out, 4, &self.item_list_num.to_le_bytes());
        write_prim(out, 8, &self.empty_inven_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            item_list_num: i32::from_le_bytes(read_array(bytes, 4)),
            empty_inven_slot_num: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcStreetstallItemBuyRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ITEM_COMBINATION` (`#pragma pack(4)`, 16 bytes, packet ID `0x13000098`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemCombinationRequest0104 {
    /// `iCostumeItemSlot` at offset 0.
    pub costume_item_slot: i32,
    /// `iStatItemSlot` at offset 4.
    pub stat_item_slot: i32,
    /// `iCashItemSlot1` at offset 8.
    pub cash_item_slot1: i32,
    /// `iCashItemSlot2` at offset 12.
    pub cash_item_slot2: i32,
}

impl PcItemCombinationRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.costume_item_slot.to_le_bytes());
        write_prim(out, 4, &self.stat_item_slot.to_le_bytes());
        write_prim(out, 8, &self.cash_item_slot1.to_le_bytes());
        write_prim(out, 12, &self.cash_item_slot2.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            costume_item_slot: i32::from_le_bytes(read_array(bytes, 0)),
            stat_item_slot: i32::from_le_bytes(read_array(bytes, 4)),
            cash_item_slot1: i32::from_le_bytes(read_array(bytes, 8)),
            cash_item_slot2: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcItemCombinationRequest0104 {
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

/// `sP_CL2FE_GM_REQ_SET_PC_SKILL` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000099`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmSetPcSkillRequest0104 {
    /// `iSkillSlotNum` at offset 0.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
}

impl GmSetPcSkillRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_slot_num.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for GmSetPcSkillRequest0104 {
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
