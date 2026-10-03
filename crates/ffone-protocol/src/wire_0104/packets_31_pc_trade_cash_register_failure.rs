// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_PC_TRADE_CASH_REGISTER_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000051`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeCashRegisterFailure0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl PcTradeCashRegisterFailure0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTradeCashRegisterFailure0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_EMOTES_CHAT` (`#pragma pack(4)`, 272 bytes, packet ID `0x31000052`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeEmotesChatReply0104 {
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
}

impl PcTradeEmotesChatReply0104 {
    pub const SIZE: usize = 272;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        write_utf16(out, 12, &self.free_chat);
        write_prim(out, 268, &self.emote_code.to_le_bytes());
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
        }
    }
}

impl WirePayload for PcTradeEmotesChatReply0104 {
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

/// `sP_FE2CL_REP_PC_NANO_CREATE_SUCC` (`#pragma pack(4)`, 28 bytes, packet ID `0x31000053`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNanoCreateSuccess0104 {
    /// `iPC_FusionMatter` at offset 0.
    pub pc_fusion_matter: i32,
    /// `iQuestItemSlotNum` at offset 4.
    pub quest_item_slot_num: i32,
    /// `QuestItem` at offset 8.
    pub quest_item: ItemBase0104,
    /// `Nano` at offset 20.
    pub nano: Nano0104,
    /// `iPC_Level` at offset 26.
    pub pc_level: i16,
}

impl PcNanoCreateSuccess0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_fusion_matter.to_le_bytes());
        write_prim(out, 4, &self.quest_item_slot_num.to_le_bytes());
        self.quest_item.write_into(&mut out[8..20]);
        self.nano.write_into(&mut out[20..26]);
        write_prim(out, 26, &self.pc_level.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_fusion_matter: i32::from_le_bytes(read_array(bytes, 0)),
            quest_item_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            quest_item: ItemBase0104::read_from(&bytes[8..20]),
            nano: Nano0104::read_from(&bytes[20..26]),
            pc_level: i16::from_le_bytes(read_array(bytes, 26)),
        }
    }
}

impl WirePayload for PcNanoCreateSuccess0104 {
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

/// `sP_FE2CL_REP_PC_NANO_CREATE_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000054`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNanoCreateFailure0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcNanoCreateFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcNanoCreateFailure0104 {
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

/// `sP_FE2CL_REP_NANO_TUNE_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000055`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoTuneFailure0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl NanoTuneFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NanoTuneFailure0104 {
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

/// `sP_FE2CL_REP_PC_BANK_OPEN_SUCC` (`#pragma pack(4)`, 2404 bytes, packet ID `0x31000056`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// OpenFusion divergence: the pinned `structs/0104.hpp` declares this struct as 1432 bytes. Wiring code must choose the server-compatible length explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBankOpenSuccess0104 {
    /// `aBank` at offset 0.
    pub bank: [ItemBase0104; 200],
    /// `iExtraBank` at offset 2400.
    pub extra_bank: i32,
}

impl PcBankOpenSuccess0104 {
    pub const SIZE: usize = 2404;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.bank.iter().enumerate() {
            let start = 0 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
        write_prim(out, 2400, &self.extra_bank.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            bank: std::array::from_fn(|index| {
                let start = 0 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
            extra_bank: i32::from_le_bytes(read_array(bytes, 2400)),
        }
    }
}

impl WirePayload for PcBankOpenSuccess0104 {
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

/// `sP_FE2CL_REP_PC_BANK_OPEN_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000057`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBankOpenFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcBankOpenFailure0104 {
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

impl WirePayload for PcBankOpenFailure0104 {
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

/// `sP_FE2CL_REP_PC_BANK_CLOSE_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000058`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBankCloseSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl PcBankCloseSuccess0104 {
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

impl WirePayload for PcBankCloseSuccess0104 {
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

/// `sP_FE2CL_REP_PC_BANK_CLOSE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000059`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBankCloseFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcBankCloseFailure0104 {
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

impl WirePayload for PcBankCloseFailure0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_START_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100005a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorStartSuccess0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iVendorID` at offset 4.
    pub vendor_id: i32,
}

impl PcVendorStartSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.vendor_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            vendor_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcVendorStartSuccess0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_START_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100005b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorStartFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVendorStartFailure0104 {
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

impl WirePayload for PcVendorStartFailure0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC` (`#pragma pack(4)`, 1200 bytes, packet ID `0x3100005c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: source declares Size = 480 but the fields need 1200 bytes; Mono marshals max(explicit, computed) = 1200.
///
/// OpenFusion divergence: the pinned `structs/0104.hpp` declares this struct as 480 bytes. Wiring code must choose the server-compatible length explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorTableUpdateSuccess0104 {
    /// `item` at offset 0.
    pub item: [ItemVendor0104; 50],
}

impl PcVendorTableUpdateSuccess0104 {
    pub const SIZE: usize = 1200;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.item.iter().enumerate() {
            let start = 0 + index * 24;
            value.write_into(&mut out[start..start + 24]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            item: std::array::from_fn(|index| {
                let start = 0 + index * 24;
                ItemVendor0104::read_from(&bytes[start..start + 24])
            }),
        }
    }
}

impl WirePayload for PcVendorTableUpdateSuccess0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100005d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorTableUpdateFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVendorTableUpdateFailure0104 {
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

impl WirePayload for PcVendorTableUpdateFailure0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC` (`#pragma pack(4)`, 20 bytes, packet ID `0x3100005e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemRestoreBuySuccess0104 {
    /// `iCandy` at offset 0.
    pub candy: i32,
    /// `iInvenSlotNum` at offset 4.
    pub inven_slot_num: i32,
    /// `Item` at offset 8.
    pub item: ItemBase0104,
}

impl PcVendorItemRestoreBuySuccess0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.candy.to_le_bytes());
        write_prim(out, 4, &self.inven_slot_num.to_le_bytes());
        self.item.write_into(&mut out[8..20]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            candy: i32::from_le_bytes(read_array(bytes, 0)),
            inven_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            item: ItemBase0104::read_from(&bytes[8..20]),
        }
    }
}

impl WirePayload for PcVendorItemRestoreBuySuccess0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100005f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemRestoreBuyFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVendorItemRestoreBuyFailure0104 {
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

impl WirePayload for PcVendorItemRestoreBuyFailure0104 {
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

/// `sP_FE2CL_CHAR_TIME_BUFF_TIME_OUT` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000060`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct CharTimeBuffTimeOut0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `iConditionBitFlag` at offset 8.
    pub condition_bit_flag: i32,
}

impl CharTimeBuffTimeOut0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.condition_bit_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for CharTimeBuffTimeOut0104 {
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

/// `sP_FE2CL_REP_PC_GIVE_ITEM_SUCC` (`#pragma pack(4)`, 20 bytes, packet ID `0x31000061`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGiveItemSuccess0104 {
    /// `eIL` at offset 0.
    pub e_il: i32,
    /// `iSlotNum` at offset 4.
    pub slot_num: i32,
    /// `Item` at offset 8.
    pub item: ItemBase0104,
}

impl PcGiveItemSuccess0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_il.to_le_bytes());
        write_prim(out, 4, &self.slot_num.to_le_bytes());
        self.item.write_into(&mut out[8..20]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_il: i32::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            item: ItemBase0104::read_from(&bytes[8..20]),
        }
    }
}

impl WirePayload for PcGiveItemSuccess0104 {
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

/// `sP_FE2CL_REP_PC_GIVE_ITEM_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000062`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGiveItemFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcGiveItemFailure0104 {
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

impl WirePayload for PcGiveItemFailure0104 {
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
