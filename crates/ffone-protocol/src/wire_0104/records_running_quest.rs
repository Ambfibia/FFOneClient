// Generated wire layouts; do not edit by hand.
use super::*;
/// `sRunningQuest` (`#pragma pack(4)`, 52 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RunningQuest0104 {
    /// `m_aCurrTaskID` at offset 0.
    pub m_a_curr_task_id: i32,
    /// `m_aKillNPCID` at offset 4.
    pub m_a_kill_npcid: [i32; 3],
    /// `m_aKillNPCCount` at offset 16.
    pub m_a_kill_npc_count: [i32; 3],
    /// `m_aNeededItemID` at offset 28.
    pub m_a_needed_item_id: [i32; 3],
    /// `m_aNeededItemCount` at offset 40.
    pub m_a_needed_item_count: [i32; 3],
}

impl RunningQuest0104 {
    pub const SIZE: usize = 52;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.m_a_curr_task_id.to_le_bytes());
        for (index, value) in self.m_a_kill_npcid.iter().enumerate() {
            write_prim(out, 4 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.m_a_kill_npc_count.iter().enumerate() {
            write_prim(out, 16 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.m_a_needed_item_id.iter().enumerate() {
            write_prim(out, 28 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.m_a_needed_item_count.iter().enumerate() {
            write_prim(out, 40 + index * 4, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            m_a_curr_task_id: i32::from_le_bytes(read_array(bytes, 0)),
            m_a_kill_npcid: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 4 + index * 4))
            }),
            m_a_kill_npc_count: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 16 + index * 4))
            }),
            m_a_needed_item_id: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 28 + index * 4))
            }),
            m_a_needed_item_count: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 40 + index * 4))
            }),
        }
    }
}

impl WirePayload for RunningQuest0104 {
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

/// `sSYSTEMTIME` (`#pragma pack(4)`, 32 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemTime0104 {
    /// `wYear` at offset 0.
    pub year: i32,
    /// `wMonth` at offset 4.
    pub month: i32,
    /// `wDayOfWeek` at offset 8.
    pub day_of_week: i32,
    /// `wDay` at offset 12.
    pub day: i32,
    /// `wHour` at offset 16.
    pub hour: i32,
    /// `wMinute` at offset 20.
    pub minute: i32,
    /// `wSecond` at offset 24.
    pub second: i32,
    /// `wMilliseconds` at offset 28.
    pub milliseconds: i32,
}

impl SystemTime0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.year.to_le_bytes());
        write_prim(out, 4, &self.month.to_le_bytes());
        write_prim(out, 8, &self.day_of_week.to_le_bytes());
        write_prim(out, 12, &self.day.to_le_bytes());
        write_prim(out, 16, &self.hour.to_le_bytes());
        write_prim(out, 20, &self.minute.to_le_bytes());
        write_prim(out, 24, &self.second.to_le_bytes());
        write_prim(out, 28, &self.milliseconds.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            year: i32::from_le_bytes(read_array(bytes, 0)),
            month: i32::from_le_bytes(read_array(bytes, 4)),
            day_of_week: i32::from_le_bytes(read_array(bytes, 8)),
            day: i32::from_le_bytes(read_array(bytes, 12)),
            hour: i32::from_le_bytes(read_array(bytes, 16)),
            minute: i32::from_le_bytes(read_array(bytes, 20)),
            second: i32::from_le_bytes(read_array(bytes, 24)),
            milliseconds: i32::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for SystemTime0104 {
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

/// `sShinyAppearanceData` (`#pragma pack(4)`, 24 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ShinyAppearanceData0104 {
    /// `iShiny_ID` at offset 0.
    pub shiny_id: i32,
    /// `iShinyType` at offset 4.
    pub shiny_type: i32,
    /// `iMapNum` at offset 8.
    pub map_num: i32,
    /// `iX` at offset 12.
    pub x: i32,
    /// `iY` at offset 16.
    pub y: i32,
    /// `iZ` at offset 20.
    pub z: i32,
}

impl ShinyAppearanceData0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.shiny_id.to_le_bytes());
        write_prim(out, 4, &self.shiny_type.to_le_bytes());
        write_prim(out, 8, &self.map_num.to_le_bytes());
        write_prim(out, 12, &self.x.to_le_bytes());
        write_prim(out, 16, &self.y.to_le_bytes());
        write_prim(out, 20, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_id: i32::from_le_bytes(read_array(bytes, 0)),
            shiny_type: i32::from_le_bytes(read_array(bytes, 4)),
            map_num: i32::from_le_bytes(read_array(bytes, 8)),
            x: i32::from_le_bytes(read_array(bytes, 12)),
            y: i32::from_le_bytes(read_array(bytes, 16)),
            z: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for ShinyAppearanceData0104 {
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

/// `sSkillResult_BatteryDrain` (`#pragma pack(4)`, 40 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultBatteryDrain0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bProtected` at offset 8.
    pub protected: i32,
    /// `iDrainW` at offset 12.
    pub drain_w: i32,
    /// `iBatteryW` at offset 16.
    pub battery_w: i32,
    /// `iDrainN` at offset 20.
    pub drain_n: i32,
    /// `iBatteryN` at offset 24.
    pub battery_n: i32,
    /// `iStamina` at offset 28.
    pub stamina: i16,
    /// `bNanoDeactive` at offset 32.
    pub nano_deactive: i32,
    /// `iConditionBitFlag` at offset 36.
    pub condition_bit_flag: i32,
}

impl SkillResultBatteryDrain0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.drain_w.to_le_bytes());
        write_prim(out, 16, &self.battery_w.to_le_bytes());
        write_prim(out, 20, &self.drain_n.to_le_bytes());
        write_prim(out, 24, &self.battery_n.to_le_bytes());
        write_prim(out, 28, &self.stamina.to_le_bytes());
        write_prim(out, 32, &self.nano_deactive.to_le_bytes());
        write_prim(out, 36, &self.condition_bit_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            protected: i32::from_le_bytes(read_array(bytes, 8)),
            drain_w: i32::from_le_bytes(read_array(bytes, 12)),
            battery_w: i32::from_le_bytes(read_array(bytes, 16)),
            drain_n: i32::from_le_bytes(read_array(bytes, 20)),
            battery_n: i32::from_le_bytes(read_array(bytes, 24)),
            stamina: i16::from_le_bytes(read_array(bytes, 28)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 32)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 36)),
        }
    }
}

impl WirePayload for SkillResultBatteryDrain0104 {
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

/// `sSkillResult_Buff` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultBuff0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bProtected` at offset 8.
    pub protected: i32,
    /// `iConditionBitFlag` at offset 12.
    pub condition_bit_flag: i32,
}

impl SkillResultBuff0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.condition_bit_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            protected: i32::from_le_bytes(read_array(bytes, 8)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for SkillResultBuff0104 {
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

/// `sSkillResult_Damage` (`#pragma pack(4)`, 20 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultDamage0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bProtected` at offset 8.
    pub protected: i32,
    /// `iDamage` at offset 12.
    pub damage: i32,
    /// `iHP` at offset 16.
    pub hp: i32,
}

impl SkillResultDamage0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.damage.to_le_bytes());
        write_prim(out, 16, &self.hp.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            protected: i32::from_le_bytes(read_array(bytes, 8)),
            damage: i32::from_le_bytes(read_array(bytes, 12)),
            hp: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for SkillResultDamage0104 {
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

/// `sSkillResult_Damage_N_Debuff` (`#pragma pack(4)`, 32 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultDamageAndDebuff0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bProtected` at offset 8.
    pub protected: i32,
    /// `iDamage` at offset 12.
    pub damage: i32,
    /// `iHP` at offset 16.
    pub hp: i32,
    /// `iStamina` at offset 20.
    pub stamina: i16,
    /// `bNanoDeactive` at offset 24.
    pub nano_deactive: i32,
    /// `iConditionBitFlag` at offset 28.
    pub condition_bit_flag: i32,
}

impl SkillResultDamageAndDebuff0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.damage.to_le_bytes());
        write_prim(out, 16, &self.hp.to_le_bytes());
        write_prim(out, 20, &self.stamina.to_le_bytes());
        write_prim(out, 24, &self.nano_deactive.to_le_bytes());
        write_prim(out, 28, &self.condition_bit_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            protected: i32::from_le_bytes(read_array(bytes, 8)),
            damage: i32::from_le_bytes(read_array(bytes, 12)),
            hp: i32::from_le_bytes(read_array(bytes, 16)),
            stamina: i16::from_le_bytes(read_array(bytes, 20)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 24)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for SkillResultDamageAndDebuff0104 {
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

/// `sSkillResult_Damage_N_Move` (`#pragma pack(4)`, 36 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultDamageAndMove0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bProtected` at offset 8.
    pub protected: i32,
    /// `iDamage` at offset 12.
    pub damage: i32,
    /// `iHP` at offset 16.
    pub hp: i32,
    /// `iMoveX` at offset 20.
    pub move_x: i32,
    /// `iMoveY` at offset 24.
    pub move_y: i32,
    /// `iMoveZ` at offset 28.
    pub move_z: i32,
    /// `iBlockMove` at offset 32.
    pub block_move: i32,
}

impl SkillResultDamageAndMove0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.damage.to_le_bytes());
        write_prim(out, 16, &self.hp.to_le_bytes());
        write_prim(out, 20, &self.move_x.to_le_bytes());
        write_prim(out, 24, &self.move_y.to_le_bytes());
        write_prim(out, 28, &self.move_z.to_le_bytes());
        write_prim(out, 32, &self.block_move.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            protected: i32::from_le_bytes(read_array(bytes, 8)),
            damage: i32::from_le_bytes(read_array(bytes, 12)),
            hp: i32::from_le_bytes(read_array(bytes, 16)),
            move_x: i32::from_le_bytes(read_array(bytes, 20)),
            move_y: i32::from_le_bytes(read_array(bytes, 24)),
            move_z: i32::from_le_bytes(read_array(bytes, 28)),
            block_move: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for SkillResultDamageAndMove0104 {
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

/// `sSkillResult_DotDamage` (`#pragma pack(4)`, 32 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultDotDamage0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `bProtected` at offset 8.
    pub protected: i32,
    /// `iDamage` at offset 12.
    pub damage: i32,
    /// `iHP` at offset 16.
    pub hp: i32,
    /// `iStamina` at offset 20.
    pub stamina: i16,
    /// `bNanoDeactive` at offset 24.
    pub nano_deactive: i32,
    /// `iConditionBitFlag` at offset 28.
    pub condition_bit_flag: i32,
}

impl SkillResultDotDamage0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.damage.to_le_bytes());
        write_prim(out, 16, &self.hp.to_le_bytes());
        write_prim(out, 20, &self.stamina.to_le_bytes());
        write_prim(out, 24, &self.nano_deactive.to_le_bytes());
        write_prim(out, 28, &self.condition_bit_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            protected: i32::from_le_bytes(read_array(bytes, 8)),
            damage: i32::from_le_bytes(read_array(bytes, 12)),
            hp: i32::from_le_bytes(read_array(bytes, 16)),
            stamina: i16::from_le_bytes(read_array(bytes, 20)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 24)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for SkillResultDotDamage0104 {
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

/// `sSkillResult_Heal_HP` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultHealHp0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `iHealHP` at offset 8.
    pub heal_hp: i32,
    /// `iHP` at offset 12.
    pub hp: i32,
}

impl SkillResultHealHp0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.heal_hp.to_le_bytes());
        write_prim(out, 12, &self.hp.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            heal_hp: i32::from_le_bytes(read_array(bytes, 8)),
            hp: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for SkillResultHealHp0104 {
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

/// `sSkillResult_Heal_Stamina` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultHealStamina0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `iHealNanoStamina` at offset 8.
    pub heal_nano_stamina: i16,
    /// `Nano` at offset 10.
    pub nano: Nano0104,
}

impl SkillResultHealStamina0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.heal_nano_stamina.to_le_bytes());
        self.nano.write_into(&mut out[10..16]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            heal_nano_stamina: i16::from_le_bytes(read_array(bytes, 8)),
            nano: Nano0104::read_from(&bytes[10..16]),
        }
    }
}

impl WirePayload for SkillResultHealStamina0104 {
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

/// `sSkillResult_Move` (`#pragma pack(4)`, 24 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultMove0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `iMapNum` at offset 8.
    pub map_num: i32,
    /// `iMoveX` at offset 12.
    pub move_x: i32,
    /// `iMoveY` at offset 16.
    pub move_y: i32,
    /// `iMoveZ` at offset 20.
    pub move_z: i32,
}

impl SkillResultMove0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.map_num.to_le_bytes());
        write_prim(out, 12, &self.move_x.to_le_bytes());
        write_prim(out, 16, &self.move_y.to_le_bytes());
        write_prim(out, 20, &self.move_z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            map_num: i32::from_le_bytes(read_array(bytes, 8)),
            move_x: i32::from_le_bytes(read_array(bytes, 12)),
            move_y: i32::from_le_bytes(read_array(bytes, 16)),
            move_z: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for SkillResultMove0104 {
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

/// `sSkillResult_Resurrect` (`#pragma pack(4)`, 12 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultResurrect0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `iRegenHP` at offset 8.
    pub regen_hp: i32,
}

impl SkillResultResurrect0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.regen_hp.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            regen_hp: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for SkillResultResurrect0104 {
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
