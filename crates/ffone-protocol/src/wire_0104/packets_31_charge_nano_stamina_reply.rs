// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_CHARGE_NANO_STAMINA` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000032`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ChargeNanoStaminaReply0104 {
    /// `iBatteryN` at offset 0.
    pub battery_n: i32,
    /// `iNanoID` at offset 4.
    pub nano_id: i16,
    /// `iNanoStamina` at offset 6.
    pub nano_stamina: i16,
}

impl ChargeNanoStaminaReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.battery_n.to_le_bytes());
        write_prim(out, 4, &self.nano_id.to_le_bytes());
        write_prim(out, 6, &self.nano_stamina.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            battery_n: i32::from_le_bytes(read_array(bytes, 0)),
            nano_id: i16::from_le_bytes(read_array(bytes, 4)),
            nano_stamina: i16::from_le_bytes(read_array(bytes, 6)),
        }
    }
}

impl WirePayload for ChargeNanoStaminaReply0104 {
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

/// `sP_FE2CL_REP_PC_TICK` (`#pragma pack(4)`, 32 bytes, packet ID `0x31000033`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTickReply0104 {
    /// `iHP` at offset 0.
    pub hp: i32,
    /// `aNano` at offset 4.
    pub nano: [Nano0104; 3],
    /// `iBatteryN` at offset 24.
    pub battery_n: i32,
    /// `bResetMissionFlag` at offset 28.
    pub reset_mission_flag: i32,
}

impl PcTickReply0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.hp.to_le_bytes());
        for (index, value) in self.nano.iter().enumerate() {
            let start = 4 + index * 6;
            value.write_into(&mut out[start..start + 6]);
        }
        write_prim(out, 24, &self.battery_n.to_le_bytes());
        write_prim(out, 28, &self.reset_mission_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            hp: i32::from_le_bytes(read_array(bytes, 0)),
            nano: std::array::from_fn(|index| {
                let start = 4 + index * 6;
                Nano0104::read_from(&bytes[start..start + 6])
            }),
            battery_n: i32::from_le_bytes(read_array(bytes, 24)),
            reset_mission_flag: i32::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for PcTickReply0104 {
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

/// `sP_FE2CL_REP_PC_KILL_QUEST_NPCs_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000034`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcKillQuestNpcsSuccess0104 {
    /// `iNPCID` at offset 0.
    pub npcid: i32,
}

impl PcKillQuestNpcsSuccess0104 {
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

impl WirePayload for PcKillQuestNpcsSuccess0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC` (`#pragma pack(4)`, 20 bytes, packet ID `0x31000035`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemBuySuccess0104 {
    /// `iCandy` at offset 0.
    pub candy: i32,
    /// `iInvenSlotNum` at offset 4.
    pub inven_slot_num: i32,
    /// `Item` at offset 8.
    pub item: ItemBase0104,
}

impl PcVendorItemBuySuccess0104 {
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

impl WirePayload for PcVendorItemBuySuccess0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_ITEM_BUY_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000036`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemBuyFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVendorItemBuyFailure0104 {
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

impl WirePayload for PcVendorItemBuyFailure0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC` (`#pragma pack(4)`, 32 bytes, packet ID `0x31000037`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemSellSuccess0104 {
    /// `iCandy` at offset 0.
    pub candy: i32,
    /// `iInvenSlotNum` at offset 4.
    pub inven_slot_num: i32,
    /// `Item` at offset 8.
    pub item: ItemBase0104,
    /// `ItemStay` at offset 20.
    pub item_stay: ItemBase0104,
}

impl PcVendorItemSellSuccess0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.candy.to_le_bytes());
        write_prim(out, 4, &self.inven_slot_num.to_le_bytes());
        self.item.write_into(&mut out[8..20]);
        self.item_stay.write_into(&mut out[20..32]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            candy: i32::from_le_bytes(read_array(bytes, 0)),
            inven_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            item: ItemBase0104::read_from(&bytes[8..20]),
            item_stay: ItemBase0104::read_from(&bytes[20..32]),
        }
    }
}

impl WirePayload for PcVendorItemSellSuccess0104 {
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

/// `sP_FE2CL_REP_PC_VENDOR_ITEM_SELL_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000038`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemSellFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcVendorItemSellFailure0104 {
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

impl WirePayload for PcVendorItemSellFailure0104 {
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

/// `sP_FE2CL_REP_PC_ITEM_DELETE_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000039`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemDeleteSuccess0104 {
    /// `eIL` at offset 0.
    pub e_il: i32,
    /// `iSlotNum` at offset 4.
    pub slot_num: i32,
}

impl PcItemDeleteSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_il.to_le_bytes());
        write_prim(out, 4, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_il: i32::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcItemDeleteSuccess0104 {
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

/// `sP_FE2CL_PC_ROCKET_STYLE_READY` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100003a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRocketStyleReady0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
}

impl PcRocketStyleReady0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcRocketStyleReady0104 {
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

/// `sP_FE2CL_REP_PC_ROCKET_STYLE_FIRE_SUCC` (`#pragma pack(4)`, 56 bytes, packet ID `0x3100003b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRocketStyleFireSuccess0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iToX` at offset 16.
    pub to_x: i32,
    /// `iToY` at offset 20.
    pub to_y: i32,
    /// `iToZ` at offset 24.
    pub to_z: i32,
    /// `iBulletID` at offset 28.
    pub bullet_id: i8,
    /// `Bullet` at offset 32.
    pub bullet: PcBullet0104,
    /// `iBatteryW` at offset 44.
    pub battery_w: i32,
    /// `bNanoDeactive` at offset 48.
    pub nano_deactive: i32,
    /// `iNanoID` at offset 52.
    pub nano_id: i16,
    /// `iNanoStamina` at offset 54.
    pub nano_stamina: i16,
}

impl PcRocketStyleFireSuccess0104 {
    pub const SIZE: usize = 56;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.to_x.to_le_bytes());
        write_prim(out, 20, &self.to_y.to_le_bytes());
        write_prim(out, 24, &self.to_z.to_le_bytes());
        write_prim(out, 28, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[32..44]);
        write_prim(out, 44, &self.battery_w.to_le_bytes());
        write_prim(out, 48, &self.nano_deactive.to_le_bytes());
        write_prim(out, 52, &self.nano_id.to_le_bytes());
        write_prim(out, 54, &self.nano_stamina.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            to_x: i32::from_le_bytes(read_array(bytes, 16)),
            to_y: i32::from_le_bytes(read_array(bytes, 20)),
            to_z: i32::from_le_bytes(read_array(bytes, 24)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 28)),
            bullet: PcBullet0104::read_from(&bytes[32..44]),
            battery_w: i32::from_le_bytes(read_array(bytes, 44)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 48)),
            nano_id: i16::from_le_bytes(read_array(bytes, 52)),
            nano_stamina: i16::from_le_bytes(read_array(bytes, 54)),
        }
    }
}

impl WirePayload for PcRocketStyleFireSuccess0104 {
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

/// `sP_FE2CL_PC_ROCKET_STYLE_FIRE` (`#pragma pack(4)`, 48 bytes, packet ID `0x3100003c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRocketStyleFire0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iToX` at offset 16.
    pub to_x: i32,
    /// `iToY` at offset 20.
    pub to_y: i32,
    /// `iToZ` at offset 24.
    pub to_z: i32,
    /// `iBulletID` at offset 28.
    pub bullet_id: i8,
    /// `Bullet` at offset 32.
    pub bullet: PcBullet0104,
    /// `bNanoDeactive` at offset 44.
    pub nano_deactive: i32,
}

impl PcRocketStyleFire0104 {
    pub const SIZE: usize = 48;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.to_x.to_le_bytes());
        write_prim(out, 20, &self.to_y.to_le_bytes());
        write_prim(out, 24, &self.to_z.to_le_bytes());
        write_prim(out, 28, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[32..44]);
        write_prim(out, 44, &self.nano_deactive.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            to_x: i32::from_le_bytes(read_array(bytes, 16)),
            to_y: i32::from_le_bytes(read_array(bytes, 20)),
            to_z: i32::from_le_bytes(read_array(bytes, 24)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 28)),
            bullet: PcBullet0104::read_from(&bytes[32..44]),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 44)),
        }
    }
}

impl WirePayload for PcRocketStyleFire0104 {
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

/// `sP_FE2CL_PC_ROCKET_STYLE_HIT` (`#pragma pack(4)`, 24 bytes, packet ID `0x3100003d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRocketStyleHit0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iBulletID` at offset 4.
    pub bullet_id: i8,
    /// `Bullet` at offset 8.
    pub bullet: PcBullet0104,
    /// `iTargetCnt` at offset 20.
    pub target_cnt: i32,
}

impl PcRocketStyleHit0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[8..20]);
        write_prim(out, 20, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 4)),
            bullet: PcBullet0104::read_from(&bytes[8..20]),
            target_cnt: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for PcRocketStyleHit0104 {
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

/// `sP_FE2CL_PC_GRENADE_STYLE_READY` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100003e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGrenadeStyleReady0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
}

impl PcGrenadeStyleReady0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcGrenadeStyleReady0104 {
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

/// `sP_FE2CL_REP_PC_GRENADE_STYLE_FIRE_SUCC` (`#pragma pack(4)`, 44 bytes, packet ID `0x3100003f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGrenadeStyleFireSuccess0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i32,
    /// `iToX` at offset 4.
    pub to_x: i32,
    /// `iToY` at offset 8.
    pub to_y: i32,
    /// `iToZ` at offset 12.
    pub to_z: i32,
    /// `iBulletID` at offset 16.
    pub bullet_id: i8,
    /// `Bullet` at offset 20.
    pub bullet: PcBullet0104,
    /// `iBatteryW` at offset 32.
    pub battery_w: i32,
    /// `bNanoDeactive` at offset 36.
    pub nano_deactive: i32,
    /// `iNanoID` at offset 40.
    pub nano_id: i16,
    /// `iNanoStamina` at offset 42.
    pub nano_stamina: i16,
}

impl PcGrenadeStyleFireSuccess0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
        write_prim(out, 4, &self.to_x.to_le_bytes());
        write_prim(out, 8, &self.to_y.to_le_bytes());
        write_prim(out, 12, &self.to_z.to_le_bytes());
        write_prim(out, 16, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[20..32]);
        write_prim(out, 32, &self.battery_w.to_le_bytes());
        write_prim(out, 36, &self.nano_deactive.to_le_bytes());
        write_prim(out, 40, &self.nano_id.to_le_bytes());
        write_prim(out, 42, &self.nano_stamina.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i32::from_le_bytes(read_array(bytes, 0)),
            to_x: i32::from_le_bytes(read_array(bytes, 4)),
            to_y: i32::from_le_bytes(read_array(bytes, 8)),
            to_z: i32::from_le_bytes(read_array(bytes, 12)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 16)),
            bullet: PcBullet0104::read_from(&bytes[20..32]),
            battery_w: i32::from_le_bytes(read_array(bytes, 32)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 36)),
            nano_id: i16::from_le_bytes(read_array(bytes, 40)),
            nano_stamina: i16::from_le_bytes(read_array(bytes, 42)),
        }
    }
}

impl WirePayload for PcGrenadeStyleFireSuccess0104 {
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

/// `sP_FE2CL_PC_GRENADE_STYLE_FIRE` (`#pragma pack(4)`, 36 bytes, packet ID `0x31000040`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGrenadeStyleFire0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iToX` at offset 4.
    pub to_x: i32,
    /// `iToY` at offset 8.
    pub to_y: i32,
    /// `iToZ` at offset 12.
    pub to_z: i32,
    /// `iBulletID` at offset 16.
    pub bullet_id: i8,
    /// `Bullet` at offset 20.
    pub bullet: PcBullet0104,
    /// `bNanoDeactive` at offset 32.
    pub nano_deactive: i32,
}

impl PcGrenadeStyleFire0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.to_x.to_le_bytes());
        write_prim(out, 8, &self.to_y.to_le_bytes());
        write_prim(out, 12, &self.to_z.to_le_bytes());
        write_prim(out, 16, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[20..32]);
        write_prim(out, 32, &self.nano_deactive.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            to_x: i32::from_le_bytes(read_array(bytes, 4)),
            to_y: i32::from_le_bytes(read_array(bytes, 8)),
            to_z: i32::from_le_bytes(read_array(bytes, 12)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 16)),
            bullet: PcBullet0104::read_from(&bytes[20..32]),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for PcGrenadeStyleFire0104 {
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
