// Generated wire layouts; do not edit by hand.
use super::*;
/// `sPCGroupMemberInfo` (`#pragma pack(4)`, 112 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupMemberInfoRecord0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iPCUID` at offset 4.
    pub pcuid: u64,
    /// `iNameCheck` at offset 12.
    pub name_check: i8,
    /// `szFirstName` at offset 14.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 32.
    pub last_name: FixedUtf16<17>,
    /// `iSpecialState` at offset 66.
    pub special_state: i8,
    /// `iLv` at offset 68.
    pub lv: i16,
    /// `iHP` at offset 72.
    pub hp: i32,
    /// `iMaxHP` at offset 76.
    pub max_hp: i32,
    /// `iMapType` at offset 80.
    pub map_type: i32,
    /// `iMapNum` at offset 84.
    pub map_num: i32,
    /// `iX` at offset 88.
    pub x: i32,
    /// `iY` at offset 92.
    pub y: i32,
    /// `iZ` at offset 96.
    pub z: i32,
    /// `bNano` at offset 100.
    pub nano: i32,
    /// `Nano` at offset 104.
    pub nano_struct: Nano0104,
}

impl PcGroupMemberInfoRecord0104 {
    pub const SIZE: usize = 112;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.pcuid.to_le_bytes());
        write_prim(out, 12, &self.name_check.to_le_bytes());
        write_utf16(out, 14, &self.first_name);
        write_utf16(out, 32, &self.last_name);
        write_prim(out, 66, &self.special_state.to_le_bytes());
        write_prim(out, 68, &self.lv.to_le_bytes());
        write_prim(out, 72, &self.hp.to_le_bytes());
        write_prim(out, 76, &self.max_hp.to_le_bytes());
        write_prim(out, 80, &self.map_type.to_le_bytes());
        write_prim(out, 84, &self.map_num.to_le_bytes());
        write_prim(out, 88, &self.x.to_le_bytes());
        write_prim(out, 92, &self.y.to_le_bytes());
        write_prim(out, 96, &self.z.to_le_bytes());
        write_prim(out, 100, &self.nano.to_le_bytes());
        self.nano_struct.write_into(&mut out[104..110]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            pcuid: u64::from_le_bytes(read_array(bytes, 4)),
            name_check: i8::from_le_bytes(read_array(bytes, 12)),
            first_name: read_utf16(bytes, 14),
            last_name: read_utf16(bytes, 32),
            special_state: i8::from_le_bytes(read_array(bytes, 66)),
            lv: i16::from_le_bytes(read_array(bytes, 68)),
            hp: i32::from_le_bytes(read_array(bytes, 72)),
            max_hp: i32::from_le_bytes(read_array(bytes, 76)),
            map_type: i32::from_le_bytes(read_array(bytes, 80)),
            map_num: i32::from_le_bytes(read_array(bytes, 84)),
            x: i32::from_le_bytes(read_array(bytes, 88)),
            y: i32::from_le_bytes(read_array(bytes, 92)),
            z: i32::from_le_bytes(read_array(bytes, 96)),
            nano: i32::from_le_bytes(read_array(bytes, 100)),
            nano_struct: Nano0104::read_from(&bytes[104..110]),
        }
    }
}

impl WirePayload for PcGroupMemberInfoRecord0104 {
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

/// `sPCLoadData2CL` (`#pragma pack(4)`, 2552 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
///
/// OpenFusion divergence: the pinned `structs/0104.hpp` declares this struct as 2688 bytes. Wiring code must choose the server-compatible length explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct PcLoadData0104 {
    /// `iUserLevel` at offset 0.
    pub user_level: i16,
    /// `PCStyle` at offset 4.
    pub pc_style: PcStyle0104,
    /// `PCStyle2` at offset 80.
    pub pc_style2: PcStyle20104,
    /// `iLevel` at offset 84.
    pub level: i16,
    /// `iMentor` at offset 86.
    pub mentor: i16,
    /// `iMentorCount` at offset 88.
    pub mentor_count: i16,
    /// `iHP` at offset 92.
    pub hp: i32,
    /// `iBatteryW` at offset 96.
    pub battery_w: i32,
    /// `iBatteryN` at offset 100.
    pub battery_n: i32,
    /// `iCandy` at offset 104.
    pub candy: i32,
    /// `iFusionMatter` at offset 108.
    pub fusion_matter: i32,
    /// `iSpecialState` at offset 112.
    pub special_state: i8,
    /// `iMapNum` at offset 116.
    pub map_num: i32,
    /// `iX` at offset 120.
    pub x: i32,
    /// `iY` at offset 124.
    pub y: i32,
    /// `iZ` at offset 128.
    pub z: i32,
    /// `iAngle` at offset 132.
    pub angle: i32,
    /// `aEquip` at offset 136.
    pub equip: [ItemBase0104; 9],
    /// `aInven` at offset 244.
    pub inven: [ItemBase0104; 50],
    /// `aQInven` at offset 844.
    pub q_inven: [ItemBase0104; 50],
    /// `aNanoBank` at offset 1444.
    pub nano_bank: [Nano0104; 37],
    /// `aNanoSlots` at offset 1666.
    pub nano_slots: [i16; 3],
    /// `iActiveNanoSlotNum` at offset 1672.
    pub active_nano_slot_num: i16,
    /// `iConditionBitFlag` at offset 1676.
    pub condition_bit_flag: i32,
    /// `eCSTB___Add` at offset 1680.
    pub cstb_add: i32,
    /// `TimeBuff` at offset 1684.
    pub time_buff: TimeBuff0104,
    /// `aQuestFlag` at offset 1712.
    pub quest_flag: [i64; 32],
    /// `aRepeatQuestFlag` at offset 1968.
    pub repeat_quest_flag: [i64; 8],
    /// `aRunningQuest` at offset 2032.
    pub running_quest: [RunningQuest0104; 9],
    /// `iCurrentMissionID` at offset 2500.
    pub current_mission_id: i32,
    /// `iWarpLocationFlag` at offset 2504.
    pub warp_location_flag: i32,
    /// `aWyvernLocationFlag` at offset 2508.
    pub wyvern_location_flag: [i64; 2],
    /// `iBuddyWarpTime` at offset 2524.
    pub buddy_warp_time: i32,
    /// `iUnlockedFeatureFlag` at offset 2528.
    pub unlocked_feature_flag: i64,
    /// `iFirstUseFlag1` at offset 2536.
    pub first_use_flag1: i64,
    /// `iFirstUseFlag2` at offset 2544.
    pub first_use_flag2: i64,
}

impl PcLoadData0104 {
    pub const SIZE: usize = 2552;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.user_level.to_le_bytes());
        self.pc_style.write_into(&mut out[4..80]);
        self.pc_style2.write_into(&mut out[80..83]);
        write_prim(out, 84, &self.level.to_le_bytes());
        write_prim(out, 86, &self.mentor.to_le_bytes());
        write_prim(out, 88, &self.mentor_count.to_le_bytes());
        write_prim(out, 92, &self.hp.to_le_bytes());
        write_prim(out, 96, &self.battery_w.to_le_bytes());
        write_prim(out, 100, &self.battery_n.to_le_bytes());
        write_prim(out, 104, &self.candy.to_le_bytes());
        write_prim(out, 108, &self.fusion_matter.to_le_bytes());
        write_prim(out, 112, &self.special_state.to_le_bytes());
        write_prim(out, 116, &self.map_num.to_le_bytes());
        write_prim(out, 120, &self.x.to_le_bytes());
        write_prim(out, 124, &self.y.to_le_bytes());
        write_prim(out, 128, &self.z.to_le_bytes());
        write_prim(out, 132, &self.angle.to_le_bytes());
        for (index, value) in self.equip.iter().enumerate() {
            let start = 136 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
        for (index, value) in self.inven.iter().enumerate() {
            let start = 244 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
        for (index, value) in self.q_inven.iter().enumerate() {
            let start = 844 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
        for (index, value) in self.nano_bank.iter().enumerate() {
            let start = 1444 + index * 6;
            value.write_into(&mut out[start..start + 6]);
        }
        for (index, value) in self.nano_slots.iter().enumerate() {
            write_prim(out, 1666 + index * 2, &value.to_le_bytes());
        }
        write_prim(out, 1672, &self.active_nano_slot_num.to_le_bytes());
        write_prim(out, 1676, &self.condition_bit_flag.to_le_bytes());
        write_prim(out, 1680, &self.cstb_add.to_le_bytes());
        self.time_buff.write_into(&mut out[1684..1712]);
        for (index, value) in self.quest_flag.iter().enumerate() {
            write_prim(out, 1712 + index * 8, &value.to_le_bytes());
        }
        for (index, value) in self.repeat_quest_flag.iter().enumerate() {
            write_prim(out, 1968 + index * 8, &value.to_le_bytes());
        }
        for (index, value) in self.running_quest.iter().enumerate() {
            let start = 2032 + index * 52;
            value.write_into(&mut out[start..start + 52]);
        }
        write_prim(out, 2500, &self.current_mission_id.to_le_bytes());
        write_prim(out, 2504, &self.warp_location_flag.to_le_bytes());
        for (index, value) in self.wyvern_location_flag.iter().enumerate() {
            write_prim(out, 2508 + index * 8, &value.to_le_bytes());
        }
        write_prim(out, 2524, &self.buddy_warp_time.to_le_bytes());
        write_prim(out, 2528, &self.unlocked_feature_flag.to_le_bytes());
        write_prim(out, 2536, &self.first_use_flag1.to_le_bytes());
        write_prim(out, 2544, &self.first_use_flag2.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            user_level: i16::from_le_bytes(read_array(bytes, 0)),
            pc_style: PcStyle0104::read_from(&bytes[4..80]),
            pc_style2: PcStyle20104::read_from(&bytes[80..83]),
            level: i16::from_le_bytes(read_array(bytes, 84)),
            mentor: i16::from_le_bytes(read_array(bytes, 86)),
            mentor_count: i16::from_le_bytes(read_array(bytes, 88)),
            hp: i32::from_le_bytes(read_array(bytes, 92)),
            battery_w: i32::from_le_bytes(read_array(bytes, 96)),
            battery_n: i32::from_le_bytes(read_array(bytes, 100)),
            candy: i32::from_le_bytes(read_array(bytes, 104)),
            fusion_matter: i32::from_le_bytes(read_array(bytes, 108)),
            special_state: i8::from_le_bytes(read_array(bytes, 112)),
            map_num: i32::from_le_bytes(read_array(bytes, 116)),
            x: i32::from_le_bytes(read_array(bytes, 120)),
            y: i32::from_le_bytes(read_array(bytes, 124)),
            z: i32::from_le_bytes(read_array(bytes, 128)),
            angle: i32::from_le_bytes(read_array(bytes, 132)),
            equip: std::array::from_fn(|index| {
                let start = 136 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
            inven: std::array::from_fn(|index| {
                let start = 244 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
            q_inven: std::array::from_fn(|index| {
                let start = 844 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
            nano_bank: std::array::from_fn(|index| {
                let start = 1444 + index * 6;
                Nano0104::read_from(&bytes[start..start + 6])
            }),
            nano_slots: std::array::from_fn(|index| {
                i16::from_le_bytes(read_array(bytes, 1666 + index * 2))
            }),
            active_nano_slot_num: i16::from_le_bytes(read_array(bytes, 1672)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 1676)),
            cstb_add: i32::from_le_bytes(read_array(bytes, 1680)),
            time_buff: TimeBuff0104::read_from(&bytes[1684..1712]),
            quest_flag: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 1712 + index * 8))
            }),
            repeat_quest_flag: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 1968 + index * 8))
            }),
            running_quest: std::array::from_fn(|index| {
                let start = 2032 + index * 52;
                RunningQuest0104::read_from(&bytes[start..start + 52])
            }),
            current_mission_id: i32::from_le_bytes(read_array(bytes, 2500)),
            warp_location_flag: i32::from_le_bytes(read_array(bytes, 2504)),
            wyvern_location_flag: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 2508 + index * 8))
            }),
            buddy_warp_time: i32::from_le_bytes(read_array(bytes, 2524)),
            unlocked_feature_flag: i64::from_le_bytes(read_array(bytes, 2528)),
            first_use_flag1: i64::from_le_bytes(read_array(bytes, 2536)),
            first_use_flag2: i64::from_le_bytes(read_array(bytes, 2544)),
        }
    }
}

impl WirePayload for PcLoadData0104 {
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

/// `sPCRegenData` (`#pragma pack(4)`, 40 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegenData0104 {
    /// `iHP` at offset 0.
    pub hp: i32,
    /// `iMapNum` at offset 4.
    pub map_num: i32,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `iActiveNanoSlotNum` at offset 20.
    pub active_nano_slot_num: i16,
    /// `Nanos` at offset 22.
    pub nanos: [Nano0104; 3],
}

impl PcRegenData0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.hp.to_le_bytes());
        write_prim(out, 4, &self.map_num.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.active_nano_slot_num.to_le_bytes());
        for (index, value) in self.nanos.iter().enumerate() {
            let start = 22 + index * 6;
            value.write_into(&mut out[start..start + 6]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            hp: i32::from_le_bytes(read_array(bytes, 0)),
            map_num: i32::from_le_bytes(read_array(bytes, 4)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            active_nano_slot_num: i16::from_le_bytes(read_array(bytes, 20)),
            nanos: std::array::from_fn(|index| {
                let start = 22 + index * 6;
                Nano0104::read_from(&bytes[start..start + 6])
            }),
        }
    }
}

impl WirePayload for PcRegenData0104 {
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

/// `sPCRegenDataForOtherPC` (`#pragma pack(4)`, 36 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegenDataForOtherPc0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iHP` at offset 4.
    pub hp: i32,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `iAngle` at offset 20.
    pub angle: i32,
    /// `iConditionBitFlag` at offset 24.
    pub condition_bit_flag: i32,
    /// `iPCState` at offset 28.
    pub pc_state: i8,
    /// `iSpecialState` at offset 29.
    pub special_state: i8,
    /// `Nano` at offset 30.
    pub nano: Nano0104,
}

impl PcRegenDataForOtherPc0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.hp.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.angle.to_le_bytes());
        write_prim(out, 24, &self.condition_bit_flag.to_le_bytes());
        write_prim(out, 28, &self.pc_state.to_le_bytes());
        write_prim(out, 29, &self.special_state.to_le_bytes());
        self.nano.write_into(&mut out[30..36]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            hp: i32::from_le_bytes(read_array(bytes, 4)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            angle: i32::from_le_bytes(read_array(bytes, 20)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 24)),
            pc_state: i8::from_le_bytes(read_array(bytes, 28)),
            special_state: i8::from_le_bytes(read_array(bytes, 29)),
            nano: Nano0104::read_from(&bytes[30..36]),
        }
    }
}

impl WirePayload for PcRegenDataForOtherPc0104 {
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

/// `sPCStyle` (`#pragma pack(4)`, 76 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStyle0104 {
    /// `iPC_UID` at offset 0.
    pub pc_uid: i64,
    /// `iNameCheck` at offset 8.
    pub name_check: i8,
    /// `szFirstName` at offset 10.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 28.
    pub last_name: FixedUtf16<17>,
    /// `iGender` at offset 62.
    pub gender: i8,
    /// `iFaceStyle` at offset 63.
    pub face_style: i8,
    /// `iHairStyle` at offset 64.
    pub hair_style: i8,
    /// `iHairColor` at offset 65.
    pub hair_color: i8,
    /// `iSkinColor` at offset 66.
    pub skin_color: i8,
    /// `iEyeColor` at offset 67.
    pub eye_color: i8,
    /// `iHeight` at offset 68.
    pub height: i8,
    /// `iBody` at offset 69.
    pub body: i8,
    /// `iClass` at offset 72.
    pub class: i32,
}

impl PcStyle0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_uid.to_le_bytes());
        write_prim(out, 8, &self.name_check.to_le_bytes());
        write_utf16(out, 10, &self.first_name);
        write_utf16(out, 28, &self.last_name);
        write_prim(out, 62, &self.gender.to_le_bytes());
        write_prim(out, 63, &self.face_style.to_le_bytes());
        write_prim(out, 64, &self.hair_style.to_le_bytes());
        write_prim(out, 65, &self.hair_color.to_le_bytes());
        write_prim(out, 66, &self.skin_color.to_le_bytes());
        write_prim(out, 67, &self.eye_color.to_le_bytes());
        write_prim(out, 68, &self.height.to_le_bytes());
        write_prim(out, 69, &self.body.to_le_bytes());
        write_prim(out, 72, &self.class.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
            name_check: i8::from_le_bytes(read_array(bytes, 8)),
            first_name: read_utf16(bytes, 10),
            last_name: read_utf16(bytes, 28),
            gender: i8::from_le_bytes(read_array(bytes, 62)),
            face_style: i8::from_le_bytes(read_array(bytes, 63)),
            hair_style: i8::from_le_bytes(read_array(bytes, 64)),
            hair_color: i8::from_le_bytes(read_array(bytes, 65)),
            skin_color: i8::from_le_bytes(read_array(bytes, 66)),
            eye_color: i8::from_le_bytes(read_array(bytes, 67)),
            height: i8::from_le_bytes(read_array(bytes, 68)),
            body: i8::from_le_bytes(read_array(bytes, 69)),
            class: i32::from_le_bytes(read_array(bytes, 72)),
        }
    }
}

impl WirePayload for PcStyle0104 {
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

/// `sPCStyle2` (`#pragma pack(1)`, 3 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStyle20104 {
    /// `iAppearanceFlag` at offset 0.
    pub appearance_flag: i8,
    /// `iTutorialFlag` at offset 1.
    pub tutorial_flag: i8,
    /// `iPayzoneFlag` at offset 2.
    pub payzone_flag: i8,
}

impl PcStyle20104 {
    pub const SIZE: usize = 3;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.appearance_flag.to_le_bytes());
        write_prim(out, 1, &self.tutorial_flag.to_le_bytes());
        write_prim(out, 2, &self.payzone_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            appearance_flag: i8::from_le_bytes(read_array(bytes, 0)),
            tutorial_flag: i8::from_le_bytes(read_array(bytes, 1)),
            payzone_flag: i8::from_le_bytes(read_array(bytes, 2)),
        }
    }
}

impl WirePayload for PcStyle20104 {
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

/// `sPC_BATTERYs` (`#pragma pack(4)`, 12 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBatteries0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iBatteryW` at offset 4.
    pub battery_w: i32,
    /// `iBatteryN` at offset 8.
    pub battery_n: i32,
}

impl PcBatteries0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.battery_w.to_le_bytes());
        write_prim(out, 8, &self.battery_n.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            battery_w: i32::from_le_bytes(read_array(bytes, 4)),
            battery_n: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcBatteries0104 {
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

/// `sPC_HP` (`#pragma pack(4)`, 8 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcHp0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iHP` at offset 4.
    pub hp: i32,
}

impl PcHp0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.hp.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            hp: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcHp0104 {
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

/// `sPC_Nano` (`#pragma pack(4)`, 12 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNano0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `Nano` at offset 4.
    pub nano: Nano0104,
    /// `iActiveNanoSlotNum` at offset 10.
    pub active_nano_slot_num: i16,
}

impl PcNano0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        self.nano.write_into(&mut out[4..10]);
        write_prim(out, 10, &self.active_nano_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            nano: Nano0104::read_from(&bytes[4..10]),
            active_nano_slot_num: i16::from_le_bytes(read_array(bytes, 10)),
        }
    }
}

impl WirePayload for PcNano0104 {
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

/// `sPC_NanoSlots` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNanoSlots0104 {
    /// `aNanoSlots` at offset 0.
    pub nano_slots: [i32; 3],
    /// `iActiveNanoSlotNum` at offset 12.
    pub active_nano_slot_num: i16,
}

impl PcNanoSlots0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.nano_slots.iter().enumerate() {
            write_prim(out, 0 + index * 4, &value.to_le_bytes());
        }
        write_prim(out, 12, &self.active_nano_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_slots: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 0 + index * 4))
            }),
            active_nano_slot_num: i16::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcNanoSlots0104 {
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

/// `sQuickSlot` (`#pragma pack(2)`, 4 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct QuickSlot0104 {
    /// `iType` at offset 0.
    pub type_: i16,
    /// `iID` at offset 2.
    pub id: i16,
}

impl QuickSlot0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.type_.to_le_bytes());
        write_prim(out, 2, &self.id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            type_: i16::from_le_bytes(read_array(bytes, 0)),
            id: i16::from_le_bytes(read_array(bytes, 2)),
        }
    }
}

impl WirePayload for QuickSlot0104 {
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
