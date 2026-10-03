use super::*;

pub const ENCHANT_REQUEST_MANAGED_SOURCE_SHA256: &str =
    "039E3B16D3D7189AC2AB860787DB4BDD70375AEAA785EE2759B38494C2B8E178";

pub const ENCHANT_DELETE_REQUEST_MANAGED_SOURCE_SHA256: &str =
    "9017539F451A0F97CB558DC10036819F41BC6453755B233E5DFC5F06F0E2EFDA";

pub const ENCHANT_DISASSEMBLE_REQUEST_MANAGED_SOURCE_SHA256: &str =
    "7F74C188A7B0AB36D71BEFC7A20A32FD6197008E83518132FD9476C77A1B92C8";

pub const ENCHANT_REDEEM_REQUEST_MANAGED_SOURCE_SHA256: &str =
    "C94D43FF206B11733722BC2FC0401BEB65811CDBB9E4E5CBEE0CEE2BADED32E6";

pub const ENCHANT_POPUP_ACTION_0104: i32 = 7;

pub const ENCHANT_REQUEST_TARGET_SLOT_OFFSET_0104: usize = 0;

pub const ENCHANT_REQUEST_WEAPON_SLOT_OFFSET_0104: usize = 4;

pub const ENCHANT_REQUEST_ARMOR_SLOT_OFFSET_0104: usize = 8;

pub const ENCHANT_REQUEST_CASH_SLOT_1_OFFSET_0104: usize = 12;

pub const ENCHANT_REQUEST_CASH_SLOT_2_OFFSET_0104: usize = 16;

pub const ENCHANT_REQUEST_ABI_0104: [EnchantAbiField0104; 5] = [
    EnchantAbiField0104 {
        name: "iEnchantItemSlot",
        offset: 0,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iWeaponMaterialItemSlot",
        offset: 4,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iDefenceMaterialItemSlot",
        offset: 8,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iCashItemSlot1",
        offset: 12,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iCashItemSlot2",
        offset: 16,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
];

/// `sP_CL2FE_REQ_PC_ITEM_DELETE` declares `eIL` before `iSlotNum`, even
/// though the managed object initializer writes the slot first.
pub const ENCHANT_DELETE_REQUEST_ABI_0104: [EnchantAbiField0104; 2] = [
    EnchantAbiField0104 {
        name: "eIL",
        offset: 0,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iSlotNum",
        offset: 4,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
];

pub const ENCHANT_DISASSEMBLE_REQUEST_ABI_0104: [EnchantAbiField0104; 1] = [EnchantAbiField0104 {
    name: "iItemSlot",
    offset: 0,
    scalar: EnchantAbiScalar0104::I32,
    count: 1,
}];
