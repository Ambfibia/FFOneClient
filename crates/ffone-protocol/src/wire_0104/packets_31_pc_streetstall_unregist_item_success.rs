// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_PC_STREETSTALL_REP_UNREGIST_ITEM_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100010d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallUnregistItemSuccess0104 {
    /// `iItemListNum` at offset 0.
    pub item_list_num: i32,
}

impl PcStreetstallUnregistItemSuccess0104 {
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

impl WirePayload for PcStreetstallUnregistItemSuccess0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_UNREGIST_ITEM_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100010e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallUnregistItemFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcStreetstallUnregistItemFailure0104 {
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

impl WirePayload for PcStreetstallUnregistItemFailure0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_SALE_START_SUCC` (`#pragma pack(4)`, 20 bytes, packet ID `0x3100010f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallSaleStartSuccess0104 {
    /// `iStreetStallItemInvenSlotNum` at offset 0.
    pub street_stall_item_inven_slot_num: i32,
    /// `OpenItem` at offset 4.
    pub open_item: ItemBase0104,
    /// `ePCCharState` at offset 16.
    pub pc_char_state: i32,
}

impl PcStreetstallSaleStartSuccess0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_item_inven_slot_num.to_le_bytes());
        self.open_item.write_into(&mut out[4..16]);
        write_prim(out, 16, &self.pc_char_state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            open_item: ItemBase0104::read_from(&bytes[4..16]),
            pc_char_state: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcStreetstallSaleStartSuccess0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_SALE_START_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000110`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallSaleStartFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcStreetstallSaleStartFailure0104 {
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

impl WirePayload for PcStreetstallSaleStartFailure0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_ITEM_LIST` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000111`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallItemListReply0104 {
    /// `iStreetStallPC_ID` at offset 0.
    pub street_stall_pc_id: i32,
    /// `iItemListCount` at offset 4.
    pub item_list_count: i32,
}

impl PcStreetstallItemListReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_pc_id.to_le_bytes());
        write_prim(out, 4, &self.item_list_count.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            item_list_count: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcStreetstallItemListReply0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_ITEM_LIST_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000112`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallItemListFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcStreetstallItemListFailure0104 {
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

impl WirePayload for PcStreetstallItemListFailure0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_ITEM_BUY_SUCC_BUYER` (`#pragma pack(4)`, 28 bytes, packet ID `0x31000113`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallItemBuySuccessBuyer0104 {
    /// `iStreetStallPC_ID` at offset 0.
    pub street_stall_pc_id: i32,
    /// `iPC_Candy` at offset 4.
    pub pc_candy: i32,
    /// `iPC_ItemInvenSlotNum` at offset 8.
    pub pc_item_inven_slot_num: i32,
    /// `PC_Item` at offset 12.
    pub pc_item: ItemBase0104,
    /// `iItemListNum` at offset 24.
    pub item_list_num: i32,
}

impl PcStreetstallItemBuySuccessBuyer0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.street_stall_pc_id.to_le_bytes());
        write_prim(out, 4, &self.pc_candy.to_le_bytes());
        write_prim(out, 8, &self.pc_item_inven_slot_num.to_le_bytes());
        self.pc_item.write_into(&mut out[12..24]);
        write_prim(out, 24, &self.item_list_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            street_stall_pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            pc_candy: i32::from_le_bytes(read_array(bytes, 4)),
            pc_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 8)),
            pc_item: ItemBase0104::read_from(&bytes[12..24]),
            item_list_num: i32::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for PcStreetstallItemBuySuccessBuyer0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_ITEM_BUY_SUCC_SELLER` (`#pragma pack(4)`, 28 bytes, packet ID `0x31000114`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallItemBuySuccessSeller0104 {
    /// `iBuyerPC_ID` at offset 0.
    pub buyer_pc_id: i32,
    /// `iStreetStallPC_Candy` at offset 4.
    pub street_stall_pc_candy: i32,
    /// `iStreetStallPC_ItemInvenSlotNum` at offset 8.
    pub street_stall_pc_item_inven_slot_num: i32,
    /// `StreetStallPC_Item` at offset 12.
    pub street_stall_pc_item: ItemBase0104,
    /// `iItemListNum` at offset 24.
    pub item_list_num: i32,
}

impl PcStreetstallItemBuySuccessSeller0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buyer_pc_id.to_le_bytes());
        write_prim(out, 4, &self.street_stall_pc_candy.to_le_bytes());
        write_prim(
            out,
            8,
            &self.street_stall_pc_item_inven_slot_num.to_le_bytes(),
        );
        self.street_stall_pc_item.write_into(&mut out[12..24]);
        write_prim(out, 24, &self.item_list_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buyer_pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            street_stall_pc_candy: i32::from_le_bytes(read_array(bytes, 4)),
            street_stall_pc_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 8)),
            street_stall_pc_item: ItemBase0104::read_from(&bytes[12..24]),
            item_list_num: i32::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for PcStreetstallItemBuySuccessSeller0104 {
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

/// `sP_FE2CL_PC_STREETSTALL_REP_ITEM_BUY_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000115`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStreetstallItemBuyFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcStreetstallItemBuyFailure0104 {
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

impl WirePayload for PcStreetstallItemBuyFailure0104 {
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

/// `sP_FE2CL_REP_PC_ITEM_COMBINATION_SUCC` (`#pragma pack(4)`, 36 bytes, packet ID `0x31000116`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemCombinationSuccess0104 {
    /// `iNewItemSlot` at offset 0.
    pub new_item_slot: i32,
    /// `sNewItem` at offset 4.
    pub s_new_item: ItemBase0104,
    /// `iStatItemSlot` at offset 16.
    pub stat_item_slot: i32,
    /// `iCashItemSlot1` at offset 20.
    pub cash_item_slot1: i32,
    /// `iCashItemSlot2` at offset 24.
    pub cash_item_slot2: i32,
    /// `iCandy` at offset 28.
    pub candy: i32,
    /// `iSuccessFlag` at offset 32.
    pub success_flag: i32,
}

impl PcItemCombinationSuccess0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.new_item_slot.to_le_bytes());
        self.s_new_item.write_into(&mut out[4..16]);
        write_prim(out, 16, &self.stat_item_slot.to_le_bytes());
        write_prim(out, 20, &self.cash_item_slot1.to_le_bytes());
        write_prim(out, 24, &self.cash_item_slot2.to_le_bytes());
        write_prim(out, 28, &self.candy.to_le_bytes());
        write_prim(out, 32, &self.success_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            new_item_slot: i32::from_le_bytes(read_array(bytes, 0)),
            s_new_item: ItemBase0104::read_from(&bytes[4..16]),
            stat_item_slot: i32::from_le_bytes(read_array(bytes, 16)),
            cash_item_slot1: i32::from_le_bytes(read_array(bytes, 20)),
            cash_item_slot2: i32::from_le_bytes(read_array(bytes, 24)),
            candy: i32::from_le_bytes(read_array(bytes, 28)),
            success_flag: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for PcItemCombinationSuccess0104 {
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

/// `sP_FE2CL_REP_PC_ITEM_COMBINATION_FAIL` (`#pragma pack(4)`, 20 bytes, packet ID `0x31000117`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemCombinationFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `iCostumeItemSlot` at offset 4.
    pub costume_item_slot: i32,
    /// `iStatItemSlot` at offset 8.
    pub stat_item_slot: i32,
    /// `iCashItemSlot1` at offset 12.
    pub cash_item_slot1: i32,
    /// `iCashItemSlot2` at offset 16.
    pub cash_item_slot2: i32,
}

impl PcItemCombinationFailure0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_prim(out, 4, &self.costume_item_slot.to_le_bytes());
        write_prim(out, 8, &self.stat_item_slot.to_le_bytes());
        write_prim(out, 12, &self.cash_item_slot1.to_le_bytes());
        write_prim(out, 16, &self.cash_item_slot2.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            costume_item_slot: i32::from_le_bytes(read_array(bytes, 4)),
            stat_item_slot: i32::from_le_bytes(read_array(bytes, 8)),
            cash_item_slot1: i32::from_le_bytes(read_array(bytes, 12)),
            cash_item_slot2: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcItemCombinationFailure0104 {
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

/// `sP_FE2CL_PC_CASH_BUFF_UPDATE` (`#pragma pack(4)`, 40 bytes, packet ID `0x31000118`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcCashBuffUpdate0104 {
    /// `eCSTB` at offset 0.
    pub cstb: i32,
    /// `eTBU` at offset 4.
    pub e_tbu: i32,
    /// `TimeBuff` at offset 8.
    pub time_buff: TimeBuff0104,
    /// `iConditionBitFlag` at offset 36.
    pub condition_bit_flag: i32,
}

impl PcCashBuffUpdate0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cstb.to_le_bytes());
        write_prim(out, 4, &self.e_tbu.to_le_bytes());
        self.time_buff.write_into(&mut out[8..36]);
        write_prim(out, 36, &self.condition_bit_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cstb: i32::from_le_bytes(read_array(bytes, 0)),
            e_tbu: i32::from_le_bytes(read_array(bytes, 4)),
            time_buff: TimeBuff0104::read_from(&bytes[8..36]),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 36)),
        }
    }
}

impl WirePayload for PcCashBuffUpdate0104 {
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

/// `sP_FE2CL_REP_PC_SKILL_ADD_SUCC` (`#pragma pack(4)`, 24 bytes, packet ID `0x31000119`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillAddSuccess0104 {
    /// `iSkillSlotNum` at offset 0.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
    /// `iSkillItemInvenSlotNum` at offset 8.
    pub skill_item_inven_slot_num: i32,
    /// `SkillItem` at offset 12.
    pub skill_item: ItemBase0104,
}

impl PcSkillAddSuccess0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_slot_num.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.skill_item_inven_slot_num.to_le_bytes());
        self.skill_item.write_into(&mut out[12..24]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i32::from_le_bytes(read_array(bytes, 4)),
            skill_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 8)),
            skill_item: ItemBase0104::read_from(&bytes[12..24]),
        }
    }
}

impl WirePayload for PcSkillAddSuccess0104 {
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

/// `sP_FE2CL_REP_PC_SKILL_ADD_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100011a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillAddFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcSkillAddFailure0104 {
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

impl WirePayload for PcSkillAddFailure0104 {
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

/// `sP_FE2CL_REP_PC_SKILL_DEL_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100011b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillDelSuccess0104 {
    /// `iSkillSlotNum` at offset 0.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
}

impl PcSkillDelSuccess0104 {
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

impl WirePayload for PcSkillDelSuccess0104 {
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

/// `sP_FE2CL_REP_PC_SKILL_DEL_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100011c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillDelFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcSkillDelFailure0104 {
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

impl WirePayload for PcSkillDelFailure0104 {
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
