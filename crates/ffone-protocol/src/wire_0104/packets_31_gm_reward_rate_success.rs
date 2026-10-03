// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_GM_REP_REWARD_RATE_SUCC` (`#pragma pack(4)`, 40 bytes, packet ID `0x3100012c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmRewardRateSuccess0104 {
    /// `afRewardRate_Taros` at offset 0.
    pub af_reward_rate_taros: [f32; 5],
    /// `afRewardRate_FusionMatter` at offset 20.
    pub af_reward_rate_fusion_matter: [f32; 5],
}

impl GmRewardRateSuccess0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.af_reward_rate_taros.iter().enumerate() {
            write_prim(out, 0 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.af_reward_rate_fusion_matter.iter().enumerate() {
            write_prim(out, 20 + index * 4, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            af_reward_rate_taros: std::array::from_fn(|index| {
                f32::from_le_bytes(read_array(bytes, 0 + index * 4))
            }),
            af_reward_rate_fusion_matter: std::array::from_fn(|index| {
                f32::from_le_bytes(read_array(bytes, 20 + index * 4))
            }),
        }
    }
}

impl WirePayload for GmRewardRateSuccess0104 {
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

/// `sP_FE2CL_REP_PC_ITEM_ENCHANT_SUCC` (`#pragma pack(4)`, 64 bytes, packet ID `0x3100012d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemEnchantSuccess0104 {
    /// `iEnchantItemSlot` at offset 0.
    pub enchant_item_slot: i32,
    /// `sEnchantItem` at offset 4.
    pub s_enchant_item: ItemBase0104,
    /// `iWeaponMaterialItemSlot` at offset 16.
    pub weapon_material_item_slot: i32,
    /// `sWeaponMaterialItem` at offset 20.
    pub s_weapon_material_item: ItemBase0104,
    /// `iDefenceMaterialItemSlot` at offset 32.
    pub defence_material_item_slot: i32,
    /// `sDefenceMaterialItem` at offset 36.
    pub s_defence_material_item: ItemBase0104,
    /// `iCashItemSlot1` at offset 48.
    pub cash_item_slot1: i32,
    /// `iCashItemSlot2` at offset 52.
    pub cash_item_slot2: i32,
    /// `iCandy` at offset 56.
    pub candy: i32,
    /// `iSuccessFlag` at offset 60.
    pub success_flag: i32,
}

impl PcItemEnchantSuccess0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.enchant_item_slot.to_le_bytes());
        self.s_enchant_item.write_into(&mut out[4..16]);
        write_prim(out, 16, &self.weapon_material_item_slot.to_le_bytes());
        self.s_weapon_material_item.write_into(&mut out[20..32]);
        write_prim(out, 32, &self.defence_material_item_slot.to_le_bytes());
        self.s_defence_material_item.write_into(&mut out[36..48]);
        write_prim(out, 48, &self.cash_item_slot1.to_le_bytes());
        write_prim(out, 52, &self.cash_item_slot2.to_le_bytes());
        write_prim(out, 56, &self.candy.to_le_bytes());
        write_prim(out, 60, &self.success_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            enchant_item_slot: i32::from_le_bytes(read_array(bytes, 0)),
            s_enchant_item: ItemBase0104::read_from(&bytes[4..16]),
            weapon_material_item_slot: i32::from_le_bytes(read_array(bytes, 16)),
            s_weapon_material_item: ItemBase0104::read_from(&bytes[20..32]),
            defence_material_item_slot: i32::from_le_bytes(read_array(bytes, 32)),
            s_defence_material_item: ItemBase0104::read_from(&bytes[36..48]),
            cash_item_slot1: i32::from_le_bytes(read_array(bytes, 48)),
            cash_item_slot2: i32::from_le_bytes(read_array(bytes, 52)),
            candy: i32::from_le_bytes(read_array(bytes, 56)),
            success_flag: i32::from_le_bytes(read_array(bytes, 60)),
        }
    }
}

impl WirePayload for PcItemEnchantSuccess0104 {
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

/// `sP_FE2CL_REP_PC_ITEM_ENCHANT_FAIL` (`#pragma pack(4)`, 24 bytes, packet ID `0x3100012e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemEnchantFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `iEnchantItemSlot` at offset 4.
    pub enchant_item_slot: i32,
    /// `iWeaponMaterialItemSlot` at offset 8.
    pub weapon_material_item_slot: i32,
    /// `iDefenceMaterialItemSlot` at offset 12.
    pub defence_material_item_slot: i32,
    /// `iCashItemSlot1` at offset 16.
    pub cash_item_slot1: i32,
    /// `iCashItemSlot2` at offset 20.
    pub cash_item_slot2: i32,
}

impl PcItemEnchantFailure0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_prim(out, 4, &self.enchant_item_slot.to_le_bytes());
        write_prim(out, 8, &self.weapon_material_item_slot.to_le_bytes());
        write_prim(out, 12, &self.defence_material_item_slot.to_le_bytes());
        write_prim(out, 16, &self.cash_item_slot1.to_le_bytes());
        write_prim(out, 20, &self.cash_item_slot2.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            enchant_item_slot: i32::from_le_bytes(read_array(bytes, 4)),
            weapon_material_item_slot: i32::from_le_bytes(read_array(bytes, 8)),
            defence_material_item_slot: i32::from_le_bytes(read_array(bytes, 12)),
            cash_item_slot1: i32::from_le_bytes(read_array(bytes, 16)),
            cash_item_slot2: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for PcItemEnchantFailure0104 {
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

/// `sP_FE2CL_REP_NANO_BOOK_SUBSET` (`#pragma pack(4)`, 76 bytes, packet ID `0x31000134`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoBookSubsetReply0104 {
    /// `PCUID` at offset 0.
    pub pcuid: i64,
    /// `bookSize` at offset 8.
    pub book_size: i32,
    /// `elementOffset` at offset 12.
    pub element_offset: i32,
    /// `element` at offset 16.
    pub element: [Nano0104; 10],
}

impl NanoBookSubsetReply0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pcuid.to_le_bytes());
        write_prim(out, 8, &self.book_size.to_le_bytes());
        write_prim(out, 12, &self.element_offset.to_le_bytes());
        for (index, value) in self.element.iter().enumerate() {
            let start = 16 + index * 6;
            value.write_into(&mut out[start..start + 6]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            book_size: i32::from_le_bytes(read_array(bytes, 8)),
            element_offset: i32::from_le_bytes(read_array(bytes, 12)),
            element: std::array::from_fn(|index| {
                let start = 16 + index * 6;
                Nano0104::read_from(&bytes[start..start + 6])
            }),
        }
    }
}

impl WirePayload for NanoBookSubsetReply0104 {
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

/// `sP_FE2CL_NPC_CUSTOM_ATTACK_PCs` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000135`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcCustomAttackPcs0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iMode` at offset 4.
    pub mode: i32,
    /// `iProjectileType` at offset 8.
    pub projectile_type: i32,
    /// `iPCCnt` at offset 12.
    pub pc_cnt: i32,
}

impl NpcCustomAttackPcs0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.mode.to_le_bytes());
        write_prim(out, 8, &self.projectile_type.to_le_bytes());
        write_prim(out, 12, &self.pc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            mode: i32::from_le_bytes(read_array(bytes, 4)),
            projectile_type: i32::from_le_bytes(read_array(bytes, 8)),
            pc_cnt: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for NpcCustomAttackPcs0104 {
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

/// `sP_FE2CL_NPC_SELF_EFFECT` (`#pragma pack(4)`, 520 bytes, packet ID `0x31000136`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSelfEffect0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iEffect` at offset 4.
    pub effect: i32,
    /// `szSFX` at offset 8.
    pub sfx: FixedUtf16<128>,
    /// `szAnimation` at offset 264.
    pub animation: FixedUtf16<128>,
}

impl NpcSelfEffect0104 {
    pub const SIZE: usize = 520;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.effect.to_le_bytes());
        write_utf16(out, 8, &self.sfx);
        write_utf16(out, 264, &self.animation);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            effect: i32::from_le_bytes(read_array(bytes, 4)),
            sfx: read_utf16(bytes, 8),
            animation: read_utf16(bytes, 264),
        }
    }
}

impl WirePayload for NpcSelfEffect0104 {
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

/// `sP_FE2CL_REP_PRESENT_NPC_TYPES` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000137`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: source declares Size = 4 but the fields need 8 bytes; Mono marshals max(explicit, computed) = 8.
#[derive(Debug, Clone, PartialEq)]
pub struct PresentNpcTypesReply0104 {
    /// `bClear` at offset 0.
    pub clear: i32,
    /// `iCnt` at offset 4.
    pub cnt: i32,
}

impl PresentNpcTypesReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.clear.to_le_bytes());
        write_prim(out, 4, &self.cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            clear: i32::from_le_bytes(read_array(bytes, 0)),
            cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PresentNpcTypesReply0104 {
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

/// `sP_FE2CL_REP_PC_BARBER_OPEN_SUCC` (`#pragma pack(4)`, 288 bytes, packet ID `0x31000138`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBarberOpenSuccess0104 {
    /// `iChangeGender` at offset 0.
    pub change_gender: i32,
    /// `iGenderCost` at offset 4.
    pub gender_cost: i32,
    /// `iBodyCost` at offset 8.
    pub body_cost: i32,
    /// `iSkinColorCost` at offset 12.
    pub skin_color_cost: i32,
    /// `iHairStyleCost` at offset 16.
    pub hair_style_cost: i32,
    /// `iHairColorCost` at offset 20.
    pub hair_color_cost: i32,
    /// `iFaceStyleCost` at offset 24.
    pub face_style_cost: i32,
    /// `iEyeColorCost` at offset 28.
    pub eye_color_cost: i32,
    /// `aSpecialHair` at offset 32.
    pub special_hair: [i32; 16],
    /// `aSpecialFace` at offset 96.
    pub special_face: [i32; 16],
    /// `aSpecialHairCost` at offset 160.
    pub special_hair_cost: [i32; 16],
    /// `aSpecialFaceCost` at offset 224.
    pub special_face_cost: [i32; 16],
}

impl PcBarberOpenSuccess0104 {
    pub const SIZE: usize = 288;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.change_gender.to_le_bytes());
        write_prim(out, 4, &self.gender_cost.to_le_bytes());
        write_prim(out, 8, &self.body_cost.to_le_bytes());
        write_prim(out, 12, &self.skin_color_cost.to_le_bytes());
        write_prim(out, 16, &self.hair_style_cost.to_le_bytes());
        write_prim(out, 20, &self.hair_color_cost.to_le_bytes());
        write_prim(out, 24, &self.face_style_cost.to_le_bytes());
        write_prim(out, 28, &self.eye_color_cost.to_le_bytes());
        for (index, value) in self.special_hair.iter().enumerate() {
            write_prim(out, 32 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.special_face.iter().enumerate() {
            write_prim(out, 96 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.special_hair_cost.iter().enumerate() {
            write_prim(out, 160 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.special_face_cost.iter().enumerate() {
            write_prim(out, 224 + index * 4, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            change_gender: i32::from_le_bytes(read_array(bytes, 0)),
            gender_cost: i32::from_le_bytes(read_array(bytes, 4)),
            body_cost: i32::from_le_bytes(read_array(bytes, 8)),
            skin_color_cost: i32::from_le_bytes(read_array(bytes, 12)),
            hair_style_cost: i32::from_le_bytes(read_array(bytes, 16)),
            hair_color_cost: i32::from_le_bytes(read_array(bytes, 20)),
            face_style_cost: i32::from_le_bytes(read_array(bytes, 24)),
            eye_color_cost: i32::from_le_bytes(read_array(bytes, 28)),
            special_hair: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 32 + index * 4))
            }),
            special_face: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 96 + index * 4))
            }),
            special_hair_cost: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 160 + index * 4))
            }),
            special_face_cost: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 224 + index * 4))
            }),
        }
    }
}

impl WirePayload for PcBarberOpenSuccess0104 {
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

/// `sP_FE2CL_REP_PC_BARBER_CONFIRM` (`#pragma pack(4)`, 56 bytes, packet ID `0x31000139`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBarberConfirmReply0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `iTaros` at offset 4.
    pub taros: i32,
    /// `aUnequipSlots` at offset 8.
    pub unequip_slots: [i32; 3],
    /// `aUnequipItems` at offset 20.
    pub unequip_items: [ItemBase0104; 3],
}

impl PcBarberConfirmReply0104 {
    pub const SIZE: usize = 56;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_prim(out, 4, &self.taros.to_le_bytes());
        for (index, value) in self.unequip_slots.iter().enumerate() {
            write_prim(out, 8 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.unequip_items.iter().enumerate() {
            let start = 20 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            taros: i32::from_le_bytes(read_array(bytes, 4)),
            unequip_slots: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 8 + index * 4))
            }),
            unequip_items: std::array::from_fn(|index| {
                let start = 20 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
        }
    }
}

impl WirePayload for PcBarberConfirmReply0104 {
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

/// `sP_FE2CL_PC_STYLE_CHANGE` (`#pragma pack(4)`, 80 bytes, packet ID `0x3100013a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStyleChange0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `sPCStyle` at offset 4.
    pub s_pc_style: PcStyle0104,
}

impl PcStyleChange0104 {
    pub const SIZE: usize = 80;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        self.s_pc_style.write_into(&mut out[4..80]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            s_pc_style: PcStyle0104::read_from(&bytes[4..80]),
        }
    }
}

impl WirePayload for PcStyleChange0104 {
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
