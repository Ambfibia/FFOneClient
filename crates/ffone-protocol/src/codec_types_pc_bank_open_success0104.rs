use super::*;

impl WirePayload for VendorItemSellRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.inventory_slot);
        write_i32(&mut out, 4, self.item_count);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            inventory_slot: read_i32(bytes, 0),
            item_count: read_i32(bytes, 4),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_ITEM_DELETE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcItemDeleteRequest0104 {
    pub item_location: i32,
    pub slot_num: i32,
}

impl WirePayload for PcItemDeleteRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.item_location);
        write_i32(&mut out, 4, self.slot_num);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            item_location: read_i32(bytes, 0),
            slot_num: read_i32(bytes, 4),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_VENDOR_START`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorStartRequest0104 {
    pub npc_id: i32,
    pub vendor_id: i32,
}

impl WirePayload for VendorStartRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.vendor_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            vendor_id: read_i32(bytes, 4),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorItemRestoreBuyRequest0104 {
    pub npc_id: i32,
    pub vendor_id: i32,
    pub list_id: i8,
    pub item: ItemBase0104,
    pub inventory_slot: i32,
}

impl WirePayload for VendorItemRestoreBuyRequest0104 {
    const SIZE: usize = 28;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.vendor_id);
        out[8] = self.list_id as u8;
        // Bytes 9..12 are pack(4) padding.
        self.item.encode_into(&mut out[12..24]);
        write_i32(&mut out, 24, self.inventory_slot);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            vendor_id: read_i32(bytes, 4),
            list_id: bytes[8] as i8,
            item: ItemBase0104::decode_exact(&bytes[12..24]),
            inventory_slot: read_i32(bytes, 24),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_VENDOR_BATTERY_BUY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorBatteryBuyRequest0104 {
    pub npc_id: i32,
    pub vendor_id: i32,
    pub list_id: i8,
    pub item: ItemBase0104,
}

impl WirePayload for VendorBatteryBuyRequest0104 {
    const SIZE: usize = 24;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.vendor_id);
        out[8] = self.list_id as u8;
        // Bytes 9..12 are pack(4) padding.
        self.item.encode_into(&mut out[12..24]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            vendor_id: read_i32(bytes, 4),
            list_id: bytes[8] as i8,
            item: ItemBase0104::decode_exact(&bytes[12..24]),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_DISASSEMBLE_ITEM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcDisassembleItemRequest0104 {
    pub item_slot: i32,
}

impl WirePayload for PcDisassembleItemRequest0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.item_slot.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            item_slot: read_i32(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorItemBuySuccess0104 {
    pub candy: i32,
    pub inventory_slot: i32,
    pub item: ItemBase0104,
}

impl WirePayload for VendorItemBuySuccess0104 {
    const SIZE: usize = 20;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.candy);
        write_i32(&mut out, 4, self.inventory_slot);
        self.item.encode_into(&mut out[8..20]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            candy: read_i32(bytes, 0),
            inventory_slot: read_i32(bytes, 4),
            item: ItemBase0104::decode_exact(&bytes[8..20]),
        })
    }
}

/// Exact four-byte failure body shared by the fixed VendorMode replies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorFailure0104 {
    pub error_code: i32,
}

impl WirePayload for VendorFailure0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.error_code.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorItemSellSuccess0104 {
    pub candy: i32,
    pub inventory_slot: i32,
    pub item: ItemBase0104,
    pub item_stay: ItemBase0104,
}

impl WirePayload for VendorItemSellSuccess0104 {
    const SIZE: usize = 32;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.candy);
        write_i32(&mut out, 4, self.inventory_slot);
        self.item.encode_into(&mut out[8..20]);
        self.item_stay.encode_into(&mut out[20..32]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            candy: read_i32(bytes, 0),
            inventory_slot: read_i32(bytes, 4),
            item: ItemBase0104::decode_exact(&bytes[8..20]),
            item_stay: ItemBase0104::decode_exact(&bytes[20..32]),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_ITEM_DELETE_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcItemDeleteSuccess0104 {
    pub item_location: i32,
    pub slot_num: i32,
}

impl WirePayload for PcItemDeleteSuccess0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.item_location);
        write_i32(&mut out, 4, self.slot_num);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            item_location: read_i32(bytes, 0),
            slot_num: read_i32(bytes, 4),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_VENDOR_START_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorStartSuccess0104 {
    pub npc_id: i32,
    pub vendor_id: i32,
}

impl WirePayload for VendorStartSuccess0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.vendor_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            vendor_id: read_i32(bytes, 4),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorItemRestoreBuySuccess0104 {
    pub candy: i32,
    pub inventory_slot: i32,
    pub item: ItemBase0104,
}

impl WirePayload for VendorItemRestoreBuySuccess0104 {
    const SIZE: usize = 20;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.candy);
        write_i32(&mut out, 4, self.inventory_slot);
        self.item.encode_into(&mut out[8..20]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            candy: read_i32(bytes, 0),
            inventory_slot: read_i32(bytes, 4),
            item: ItemBase0104::decode_exact(&bytes[8..20]),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorBatteryBuySuccess0104 {
    pub candy: i32,
    pub weapon_battery: i32,
    pub nano_battery: i32,
}

impl WirePayload for VendorBatteryBuySuccess0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.candy);
        write_i32(&mut out, 4, self.weapon_battery);
        write_i32(&mut out, 8, self.nano_battery);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            candy: read_i32(bytes, 0),
            weapon_battery: read_i32(bytes, 4),
            nano_battery: read_i32(bytes, 8),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcDisassembleItemSuccess0104 {
    pub new_item_slot: i32,
    pub new_item: ItemBase0104,
}

impl WirePayload for PcDisassembleItemSuccess0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.new_item_slot);
        self.new_item.encode_into(&mut out[4..16]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            new_item_slot: read_i32(bytes, 0),
            new_item: ItemBase0104::decode_exact(&bytes[4..16]),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcDisassembleItemFailure0104 {
    pub error_code: i32,
    pub item_slot: i32,
}

impl WirePayload for PcDisassembleItemFailure0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.error_code);
        write_i32(&mut out, 4, self.item_slot);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
            item_slot: read_i32(bytes, 4),
        })
    }
}

/// Strictly typed fixed VendorMode reply family. Unknown packet IDs remain
/// available to other decoders; every known packet rejects non-exact bodies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VendorPacket0104 {
    BuySuccess(VendorItemBuySuccess0104),
    BuyFailure(VendorFailure0104),
    SellSuccess(VendorItemSellSuccess0104),
    SellFailure(VendorFailure0104),
    ItemDeleteSuccess(PcItemDeleteSuccess0104),
    StartSuccess(VendorStartSuccess0104),
    StartFailure(VendorFailure0104),
    TableSuccess(VendorTableUpdateSuccess0104),
    TableFailure(VendorFailure0104),
    RestoreSuccess(VendorItemRestoreBuySuccess0104),
    RestoreFailure(VendorFailure0104),
    BatterySuccess(VendorBatteryBuySuccess0104),
    BatteryFailure(VendorFailure0104),
    DisassembleSuccess(PcDisassembleItemSuccess0104),
    DisassembleFailure(PcDisassembleItemFailure0104),
}

/// Exact protocol-0104 `sP_CL2FE_REQ_ITEM_MOVE`
/// (`#pragma pack(4)`, 16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemMoveRequest0104 {
    pub from_location: i32,
    pub from_slot_num: i32,
    pub to_location: i32,
    pub to_slot_num: i32,
}

impl WirePayload for ItemMoveRequest0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.from_location);
        write_i32(&mut out, 4, self.from_slot_num);
        write_i32(&mut out, 8, self.to_location);
        write_i32(&mut out, 12, self.to_slot_num);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            from_location: read_i32(bytes, 0),
            from_slot_num: read_i32(bytes, 4),
            to_location: read_i32(bytes, 8),
            to_slot_num: read_i32(bytes, 12),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_BANK_OPEN`
/// (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcBankOpenRequest0104 {
    pub pc_id: i32,
    pub npc_id: i32,
}

impl WirePayload for PcBankOpenRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.npc_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            npc_id: read_i32(bytes, 4),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_BANK_CLOSE`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcBankCloseRequest0104 {
    pub pc_id: i32,
}

impl WirePayload for PcBankCloseRequest0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.pc_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_BANK_OPEN_SUCC`
/// (`sItemBase[200] + int32`, `#pragma pack(4)`, 2404 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcBankOpenSuccess0104 {
    pub bank_items: [ItemBase0104; BANK_SLOT_COUNT_0104],
    pub extra_bank: i32,
}

impl WirePayload for PcBankOpenSuccess0104 {
    const SIZE: usize = BANK_SLOT_COUNT_0104 * ItemBase0104::SIZE + size_of::<i32>();

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        for (index, item) in self.bank_items.iter().copied().enumerate() {
            let start = index * ItemBase0104::SIZE;
            item.encode_into(&mut out[start..start + ItemBase0104::SIZE]);
        }
        write_i32(
            &mut out,
            BANK_SLOT_COUNT_0104 * ItemBase0104::SIZE,
            self.extra_bank,
        );
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            bank_items: std::array::from_fn(|index| {
                let start = index * ItemBase0104::SIZE;
                ItemBase0104::decode_exact(&bytes[start..start + ItemBase0104::SIZE])
            }),
            extra_bank: read_i32(bytes, BANK_SLOT_COUNT_0104 * ItemBase0104::SIZE),
        })
    }
}

/// Exact four-byte failure body shared by clean bank-open and bank-close
/// failure packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcBankFailure0104 {
    pub error_code: i32,
}

impl WirePayload for PcBankFailure0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.error_code.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_BANK_CLOSE_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcBankCloseSuccess0104 {
    pub pc_id: i32,
}

impl WirePayload for PcBankCloseSuccess0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.pc_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
        })
    }
}

/// Strictly typed fixed bank reply family. Unknown packet IDs are left to
/// other decoders; known IDs reject every non-exact body length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcBankReply0104 {
    OpenSuccess(PcBankOpenSuccess0104),
    OpenFailure(PcBankFailure0104),
    CloseSuccess(PcBankCloseSuccess0104),
    CloseFailure(PcBankFailure0104),
}

/// Exact protocol-0104 `sP_FE2CL_PC_ITEM_MOVE_SUCC`
/// (`#pragma pack(4)`, 40 bytes).
///
/// `from_slot_item` and `to_slot_item` are authoritative post-mutation slot
/// values. The clean `UserSlot.ItemMove` writes both records directly rather
/// than simulating the request against client state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemMoveSuccessPacket0104 {
    pub from_location: i32,
    pub from_slot_num: i32,
    pub from_slot_item: ItemBase0104,
    pub to_location: i32,
    pub to_slot_num: i32,
    pub to_slot_item: ItemBase0104,
}

impl WirePayload for ItemMoveSuccessPacket0104 {
    const SIZE: usize = 40;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.from_location);
        write_i32(&mut out, 4, self.from_slot_num);
        self.from_slot_item.encode_into(&mut out[8..20]);
        write_i32(&mut out, 20, self.to_location);
        write_i32(&mut out, 24, self.to_slot_num);
        self.to_slot_item.encode_into(&mut out[28..40]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            from_location: read_i32(bytes, 0),
            from_slot_num: read_i32(bytes, 4),
            from_slot_item: ItemBase0104::decode_exact(&bytes[8..20]),
            to_location: read_i32(bytes, 20),
            to_slot_num: read_i32(bytes, 24),
            to_slot_item: ItemBase0104::decode_exact(&bytes[28..40]),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_PC_EQUIP_CHANGE`
/// (`#pragma pack(4)`, 20 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquipChangePacket0104 {
    pub pc_id: i32,
    pub equip_slot_num: i32,
    pub equip_slot_item: ItemBase0104,
}

impl WirePayload for EquipChangePacket0104 {
    const SIZE: usize = 20;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.equip_slot_num);
        self.equip_slot_item.encode_into(&mut out[8..20]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            equip_slot_num: read_i32(bytes, 4),
            equip_slot_item: ItemBase0104::decode_exact(&bytes[8..20]),
        })
    }
}

/// Protocol-0104 `sNano` embedded in a player appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nano0104 {
    pub id: i16,
    pub skill_id: i16,
    pub stamina: i16,
}

impl Nano0104 {
    pub const SIZE: usize = 6;

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: read_i16(bytes, 0),
            skill_id: read_i16(bytes, 2),
            stamina: read_i16(bytes, 4),
        }
    }

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        bytes[0..2].copy_from_slice(&self.id.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.stamina.to_le_bytes());
    }
}

/// Exact clean-Retrobution protocol-0104
/// `sP_FE2CL_REP_PC_NANO_CREATE_SUCC` (`#pragma pack(4)`, 28 bytes).
///
/// The quest slot/item and resulting Nano are all authoritative server
/// post-state. `Nano.iID` is also the exact bank index consumed by the clean
/// client's `cnOwnAvatarStatus.ReceiveNanoCreate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcNanoCreateSuccess0104 {
    pub fusion_matter: i32,
    pub quest_item_slot: i32,
    pub quest_item: ItemBase0104,
    pub nano: Nano0104,
    pub player_level: i16,
}

impl WirePayload for PcNanoCreateSuccess0104 {
    const SIZE: usize = 28;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.fusion_matter);
        write_i32(&mut out, 4, self.quest_item_slot);
        self.quest_item.encode_into(&mut out[8..20]);
        self.nano.encode_into(&mut out[20..26]);
        out[26..28].copy_from_slice(&self.player_level.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            fusion_matter: read_i32(bytes, 0),
            quest_item_slot: read_i32(bytes, 4),
            quest_item: ItemBase0104::decode_exact(&bytes[8..20]),
            nano: Nano0104::decode_exact(&bytes[20..26]),
            player_level: read_i16(bytes, 26),
        })
    }
}
