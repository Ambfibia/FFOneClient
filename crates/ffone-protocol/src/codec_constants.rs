use super::*;

/// The target Retrobution assembly has the 0104-only 2688-byte PC load block.
pub const PROTOCOL_VERSION: u16 = 104;

pub const DEFAULT_KEY_BYTES: [u8; 8] = *b"b>$rT~!Q";

pub const DEFAULT_KEY: u64 = u64::from_le_bytes(DEFAULT_KEY_BYTES);

pub const MAX_BODY_SIZE_0104: usize = 4096;

pub const QUICK_SLOT_COUNT_0104: usize = 8;

/// Clean protocol-0104 value selector used by `/speed`.
pub const GM_SET_VALUE_SPEED_0104: i32 = 6;

pub const BUDDY_LIST_CAPACITY_0104: usize = 50;

pub const CHARACTER_EQUIP_SLOT_COUNT_0104: usize = 9;

pub const NANO_TUNE_ITEM_SLOT_COUNT_0104: usize = 10;

/// Maximum result count accepted by OpenFusion's Nano-skill sender. The
/// server validates every result family against the largest 40-byte result
/// record before serializing the actual `eST`-specific records.
pub const NANO_SKILL_RESULT_MAX_TARGETS_0104: usize =
    (OPENFUSION_PAYLOAD_CAPACITY_0104 - NanoSkillUseSuccessPrefix0104::SIZE) / 40;

pub const VENDOR_TABLE_ITEM_COUNT_0104: usize = 20;

pub const BANK_SLOT_COUNT_0104: usize = 200;
