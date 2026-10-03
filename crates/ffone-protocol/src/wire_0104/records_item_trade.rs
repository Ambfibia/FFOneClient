// Generated wire layouts; do not edit by hand.
use super::*;
/// `sItemTrade` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemTrade0104 {
    /// `iType` at offset 0.
    pub type_: i16,
    /// `iID` at offset 2.
    pub id: i16,
    /// `iOpt` at offset 4.
    pub opt: i32,
    /// `iInvenNum` at offset 8.
    pub inven_num: i32,
    /// `iSlotNum` at offset 12.
    pub slot_num: i32,
}

impl ItemTrade0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.type_.to_le_bytes());
        write_prim(out, 2, &self.id.to_le_bytes());
        write_prim(out, 4, &self.opt.to_le_bytes());
        write_prim(out, 8, &self.inven_num.to_le_bytes());
        write_prim(out, 12, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            type_: i16::from_le_bytes(read_array(bytes, 0)),
            id: i16::from_le_bytes(read_array(bytes, 2)),
            opt: i32::from_le_bytes(read_array(bytes, 4)),
            inven_num: i32::from_le_bytes(read_array(bytes, 8)),
            slot_num: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for ItemTrade0104 {
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

/// `sItemVendor` (`#pragma pack(4)`, 24 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemVendor0104 {
    /// `iVendorID` at offset 0.
    pub vendor_id: i32,
    /// `fBuyCost` at offset 4.
    pub buy_cost: f32,
    /// `item` at offset 8.
    pub item: ItemBase0104,
    /// `iSortNum` at offset 20.
    pub sort_num: i32,
}

impl ItemVendor0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.vendor_id.to_le_bytes());
        write_prim(out, 4, &self.buy_cost.to_le_bytes());
        self.item.write_into(&mut out[8..20]);
        write_prim(out, 20, &self.sort_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            vendor_id: i32::from_le_bytes(read_array(bytes, 0)),
            buy_cost: f32::from_le_bytes(read_array(bytes, 4)),
            item: ItemBase0104::read_from(&bytes[8..20]),
            sort_num: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for ItemVendor0104 {
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

/// `sNPCAppearanceData` (`#pragma pack(4)`, 36 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcAppearanceData0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iNPCType` at offset 4.
    pub npc_type: i32,
    /// `iHP` at offset 8.
    pub hp: i32,
    /// `iConditionBitFlag` at offset 12.
    pub condition_bit_flag: i32,
    /// `iX` at offset 16.
    pub x: i32,
    /// `iY` at offset 20.
    pub y: i32,
    /// `iZ` at offset 24.
    pub z: i32,
    /// `iAngle` at offset 28.
    pub angle: i32,
    /// `iBarkerType` at offset 32.
    pub barker_type: i32,
}

impl NpcAppearanceData0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.npc_type.to_le_bytes());
        write_prim(out, 8, &self.hp.to_le_bytes());
        write_prim(out, 12, &self.condition_bit_flag.to_le_bytes());
        write_prim(out, 16, &self.x.to_le_bytes());
        write_prim(out, 20, &self.y.to_le_bytes());
        write_prim(out, 24, &self.z.to_le_bytes());
        write_prim(out, 28, &self.angle.to_le_bytes());
        write_prim(out, 32, &self.barker_type.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            npc_type: i32::from_le_bytes(read_array(bytes, 4)),
            hp: i32::from_le_bytes(read_array(bytes, 8)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 12)),
            x: i32::from_le_bytes(read_array(bytes, 16)),
            y: i32::from_le_bytes(read_array(bytes, 20)),
            z: i32::from_le_bytes(read_array(bytes, 24)),
            angle: i32::from_le_bytes(read_array(bytes, 28)),
            barker_type: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for NpcAppearanceData0104 {
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

/// `sNPCBullet` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcBullet0104 {
    /// `eAT` at offset 0.
    pub e_at: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bCharged` at offset 8.
    pub charged: i32,
    /// `eST` at offset 12.
    pub e_st: i32,
}

impl NpcBullet0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_at.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.charged.to_le_bytes());
        write_prim(out, 12, &self.e_st.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_at: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            charged: i32::from_le_bytes(read_array(bytes, 8)),
            e_st: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for NpcBullet0104 {
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

/// `sNPCGroupMemberInfo` (`#pragma pack(4)`, 32 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGroupMemberInfo0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iNPC_Type` at offset 4.
    pub npc_type: i32,
    /// `iHP` at offset 8.
    pub hp: i32,
    /// `iMapType` at offset 12.
    pub map_type: i32,
    /// `iMapNum` at offset 16.
    pub map_num: i32,
    /// `iX` at offset 20.
    pub x: i32,
    /// `iY` at offset 24.
    pub y: i32,
    /// `iZ` at offset 28.
    pub z: i32,
}

impl NpcGroupMemberInfo0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.npc_type.to_le_bytes());
        write_prim(out, 8, &self.hp.to_le_bytes());
        write_prim(out, 12, &self.map_type.to_le_bytes());
        write_prim(out, 16, &self.map_num.to_le_bytes());
        write_prim(out, 20, &self.x.to_le_bytes());
        write_prim(out, 24, &self.y.to_le_bytes());
        write_prim(out, 28, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            npc_type: i32::from_le_bytes(read_array(bytes, 4)),
            hp: i32::from_le_bytes(read_array(bytes, 8)),
            map_type: i32::from_le_bytes(read_array(bytes, 12)),
            map_num: i32::from_le_bytes(read_array(bytes, 16)),
            x: i32::from_le_bytes(read_array(bytes, 20)),
            y: i32::from_le_bytes(read_array(bytes, 24)),
            z: i32::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for NpcGroupMemberInfo0104 {
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

/// `sNPCLocationData` (`#pragma pack(4)`, 24 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcLocationData0104 {
    /// `iNPC_Type` at offset 0.
    pub npc_type: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iAngle` at offset 16.
    pub angle: i32,
    /// `iRoute` at offset 20.
    pub route: i32,
}

impl NpcLocationData0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_type.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.angle.to_le_bytes());
        write_prim(out, 20, &self.route.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_type: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            angle: i32::from_le_bytes(read_array(bytes, 16)),
            route: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for NpcLocationData0104 {
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

/// `sNano` (`#pragma pack(2)`, 6 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Nano0104 {
    /// `iID` at offset 0.
    pub id: i16,
    /// `iSkillID` at offset 2.
    pub skill_id: i16,
    /// `iStamina` at offset 4.
    pub stamina: i16,
}

impl Nano0104 {
    pub const SIZE: usize = 6;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 2, &self.skill_id.to_le_bytes());
        write_prim(out, 4, &self.stamina.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i16::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 2)),
            stamina: i16::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for Nano0104 {
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

/// `sNanoBank` (`#pragma pack(2)`, 4 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoBank0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i16,
    /// `iStamina` at offset 2.
    pub stamina: i16,
}

impl NanoBank0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
        write_prim(out, 2, &self.stamina.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i16::from_le_bytes(read_array(bytes, 0)),
            stamina: i16::from_le_bytes(read_array(bytes, 2)),
        }
    }
}

impl WirePayload for NanoBank0104 {
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

/// `sNanoTuneNeedItemInfo2CL` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoTuneNeedItemInfo0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i32,
    /// `ItemBase` at offset 4.
    pub item_base: ItemBase0104,
}

impl NanoTuneNeedItemInfo0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
        self.item_base.write_into(&mut out[4..16]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            item_base: ItemBase0104::read_from(&bytes[4..16]),
        }
    }
}

impl WirePayload for NanoTuneNeedItemInfo0104 {
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

/// `sOnItem` (`#pragma pack(2)`, 14 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct OnItem0104 {
    /// `iEquipHandID` at offset 0.
    pub equip_hand_id: i16,
    /// `iEquipUBID` at offset 2.
    pub equip_ubid: i16,
    /// `iEquipLBID` at offset 4.
    pub equip_lbid: i16,
    /// `iEquipFootID` at offset 6.
    pub equip_foot_id: i16,
    /// `iEquipHeadID` at offset 8.
    pub equip_head_id: i16,
    /// `iEquipFaceID` at offset 10.
    pub equip_face_id: i16,
    /// `iEquipBackID` at offset 12.
    pub equip_back_id: i16,
}

impl OnItem0104 {
    pub const SIZE: usize = 14;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.equip_hand_id.to_le_bytes());
        write_prim(out, 2, &self.equip_ubid.to_le_bytes());
        write_prim(out, 4, &self.equip_lbid.to_le_bytes());
        write_prim(out, 6, &self.equip_foot_id.to_le_bytes());
        write_prim(out, 8, &self.equip_head_id.to_le_bytes());
        write_prim(out, 10, &self.equip_face_id.to_le_bytes());
        write_prim(out, 12, &self.equip_back_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            equip_hand_id: i16::from_le_bytes(read_array(bytes, 0)),
            equip_ubid: i16::from_le_bytes(read_array(bytes, 2)),
            equip_lbid: i16::from_le_bytes(read_array(bytes, 4)),
            equip_foot_id: i16::from_le_bytes(read_array(bytes, 6)),
            equip_head_id: i16::from_le_bytes(read_array(bytes, 8)),
            equip_face_id: i16::from_le_bytes(read_array(bytes, 10)),
            equip_back_id: i16::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for OnItem0104 {
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

/// `sOnItem_Index` (`#pragma pack(2)`, 10 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct OnItemIndex0104 {
    /// `iEquipUBID_index` at offset 0.
    pub equip_ubid_index: i16,
    /// `iEquipLBID_index` at offset 2.
    pub equip_lbid_index: i16,
    /// `iEquipFootID_index` at offset 4.
    pub equip_foot_id_index: i16,
    /// `iFaceStyle` at offset 6.
    pub face_style: i16,
    /// `iHairStyle` at offset 8.
    pub hair_style: i16,
}

impl OnItemIndex0104 {
    pub const SIZE: usize = 10;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.equip_ubid_index.to_le_bytes());
        write_prim(out, 2, &self.equip_lbid_index.to_le_bytes());
        write_prim(out, 4, &self.equip_foot_id_index.to_le_bytes());
        write_prim(out, 6, &self.face_style.to_le_bytes());
        write_prim(out, 8, &self.hair_style.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            equip_ubid_index: i16::from_le_bytes(read_array(bytes, 0)),
            equip_lbid_index: i16::from_le_bytes(read_array(bytes, 2)),
            equip_foot_id_index: i16::from_le_bytes(read_array(bytes, 4)),
            face_style: i16::from_le_bytes(read_array(bytes, 6)),
            hair_style: i16::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for OnItemIndex0104 {
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

/// `sPCAppearanceData` (`#pragma pack(4)`, 232 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAppearanceData0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `PCStyle` at offset 4.
    pub pc_style: PcStyle0104,
    /// `iConditionBitFlag` at offset 80.
    pub condition_bit_flag: i32,
    /// `iPCState` at offset 84.
    pub pc_state: i8,
    /// `iSpecialState` at offset 85.
    pub special_state: i8,
    /// `iLv` at offset 86.
    pub lv: i16,
    /// `iHP` at offset 88.
    pub hp: i32,
    /// `iMapNum` at offset 92.
    pub map_num: i32,
    /// `iX` at offset 96.
    pub x: i32,
    /// `iY` at offset 100.
    pub y: i32,
    /// `iZ` at offset 104.
    pub z: i32,
    /// `iAngle` at offset 108.
    pub angle: i32,
    /// `ItemEquip` at offset 112.
    pub item_equip: [ItemBase0104; 9],
    /// `Nano` at offset 220.
    pub nano: Nano0104,
    /// `eRT` at offset 228.
    pub e_rt: i32,
}

impl PcAppearanceData0104 {
    pub const SIZE: usize = 232;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        self.pc_style.write_into(&mut out[4..80]);
        write_prim(out, 80, &self.condition_bit_flag.to_le_bytes());
        write_prim(out, 84, &self.pc_state.to_le_bytes());
        write_prim(out, 85, &self.special_state.to_le_bytes());
        write_prim(out, 86, &self.lv.to_le_bytes());
        write_prim(out, 88, &self.hp.to_le_bytes());
        write_prim(out, 92, &self.map_num.to_le_bytes());
        write_prim(out, 96, &self.x.to_le_bytes());
        write_prim(out, 100, &self.y.to_le_bytes());
        write_prim(out, 104, &self.z.to_le_bytes());
        write_prim(out, 108, &self.angle.to_le_bytes());
        for (index, value) in self.item_equip.iter().enumerate() {
            let start = 112 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
        self.nano.write_into(&mut out[220..226]);
        write_prim(out, 228, &self.e_rt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            pc_style: PcStyle0104::read_from(&bytes[4..80]),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 80)),
            pc_state: i8::from_le_bytes(read_array(bytes, 84)),
            special_state: i8::from_le_bytes(read_array(bytes, 85)),
            lv: i16::from_le_bytes(read_array(bytes, 86)),
            hp: i32::from_le_bytes(read_array(bytes, 88)),
            map_num: i32::from_le_bytes(read_array(bytes, 92)),
            x: i32::from_le_bytes(read_array(bytes, 96)),
            y: i32::from_le_bytes(read_array(bytes, 100)),
            z: i32::from_le_bytes(read_array(bytes, 104)),
            angle: i32::from_le_bytes(read_array(bytes, 108)),
            item_equip: std::array::from_fn(|index| {
                let start = 112 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
            nano: Nano0104::read_from(&bytes[220..226]),
            e_rt: i32::from_le_bytes(read_array(bytes, 228)),
        }
    }
}

impl WirePayload for PcAppearanceData0104 {
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

/// `sPCBullet` (`#pragma pack(4)`, 12 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBullet0104 {
    /// `eAT` at offset 0.
    pub e_at: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bCharged` at offset 8.
    pub charged: i32,
}

impl PcBullet0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_at.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.charged.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_at: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            charged: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcBullet0104 {
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
