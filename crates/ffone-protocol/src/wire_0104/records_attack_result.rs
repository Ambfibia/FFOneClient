// Generated wire layouts; do not edit by hand.
use super::*;
// ---- nested records -------------------------------------------------------

/// `sAttackResult` (`#pragma pack(4)`, 24 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AttackResult0104 {
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
    /// `iHitFlag` at offset 20.
    pub hit_flag: i8,
}

impl AttackResult0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.damage.to_le_bytes());
        write_prim(out, 16, &self.hp.to_le_bytes());
        write_prim(out, 20, &self.hit_flag.to_le_bytes());
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
            hit_flag: i8::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for AttackResult0104 {
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

/// `sBuddyBaseInfo` (`#pragma pack(4)`, 72 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct BuddyBaseInfo0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iPCUID` at offset 4.
    pub pcuid: i64,
    /// `bBlocked` at offset 12.
    pub blocked: i8,
    /// `bFreeChat` at offset 13.
    pub free_chat: i8,
    /// `iPCState` at offset 14.
    pub pc_state: i8,
    /// `szFirstName` at offset 16.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 34.
    pub last_name: FixedUtf16<17>,
    /// `iGender` at offset 68.
    pub gender: i8,
    /// `iNameCheckFlag` at offset 69.
    pub name_check_flag: i8,
}

impl BuddyBaseInfo0104 {
    pub const SIZE: usize = 72;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.pcuid.to_le_bytes());
        write_prim(out, 12, &self.blocked.to_le_bytes());
        write_prim(out, 13, &self.free_chat.to_le_bytes());
        write_prim(out, 14, &self.pc_state.to_le_bytes());
        write_utf16(out, 16, &self.first_name);
        write_utf16(out, 34, &self.last_name);
        write_prim(out, 68, &self.gender.to_le_bytes());
        write_prim(out, 69, &self.name_check_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            blocked: i8::from_le_bytes(read_array(bytes, 12)),
            free_chat: i8::from_le_bytes(read_array(bytes, 13)),
            pc_state: i8::from_le_bytes(read_array(bytes, 14)),
            first_name: read_utf16(bytes, 16),
            last_name: read_utf16(bytes, 34),
            gender: i8::from_le_bytes(read_array(bytes, 68)),
            name_check_flag: i8::from_le_bytes(read_array(bytes, 69)),
        }
    }
}

impl WirePayload for BuddyBaseInfo0104 {
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

/// `sBuddyStyleInfo` (`#pragma pack(4)`, 184 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct BuddyStyleInfo0104 {
    /// `sBuddyStyle` at offset 0.
    pub s_buddy_style: PcStyle0104,
    /// `aEquip` at offset 76.
    pub equip: [ItemBase0104; 9],
}

impl BuddyStyleInfo0104 {
    pub const SIZE: usize = 184;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.s_buddy_style.write_into(&mut out[0..76]);
        for (index, value) in self.equip.iter().enumerate() {
            let start = 76 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            s_buddy_style: PcStyle0104::read_from(&bytes[0..76]),
            equip: std::array::from_fn(|index| {
                let start = 76 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
        }
    }
}

impl WirePayload for BuddyStyleInfo0104 {
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

/// `sBulletAppearanceData` (`#pragma pack(4)`, 20 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct BulletAppearanceData0104 {
    /// `iBullet_ID` at offset 0.
    pub bullet_id: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iAngle` at offset 16.
    pub angle: i32,
}

impl BulletAppearanceData0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.bullet_id.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.angle.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            bullet_id: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            angle: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for BulletAppearanceData0104 {
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

/// `sCAttackResult` (`#pragma pack(4)`, 40 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct CAttackResult0104 {
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
    /// `iHitFlag` at offset 20.
    pub hit_flag: i8,
    /// `iActiveNanoSlotNum` at offset 22.
    pub active_nano_slot_num: i16,
    /// `bNanoDeactive` at offset 24.
    pub nano_deactive: i32,
    /// `iNanoID` at offset 28.
    pub nano_id: i16,
    /// `iNanoStamina` at offset 30.
    pub nano_stamina: i16,
    /// `iConditionBitFlag` at offset 32.
    pub condition_bit_flag: i32,
    /// `eCSTB___Del` at offset 36.
    pub cstb_del: i32,
}

impl CAttackResult0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.protected.to_le_bytes());
        write_prim(out, 12, &self.damage.to_le_bytes());
        write_prim(out, 16, &self.hp.to_le_bytes());
        write_prim(out, 20, &self.hit_flag.to_le_bytes());
        write_prim(out, 22, &self.active_nano_slot_num.to_le_bytes());
        write_prim(out, 24, &self.nano_deactive.to_le_bytes());
        write_prim(out, 28, &self.nano_id.to_le_bytes());
        write_prim(out, 30, &self.nano_stamina.to_le_bytes());
        write_prim(out, 32, &self.condition_bit_flag.to_le_bytes());
        write_prim(out, 36, &self.cstb_del.to_le_bytes());
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
            hit_flag: i8::from_le_bytes(read_array(bytes, 20)),
            active_nano_slot_num: i16::from_le_bytes(read_array(bytes, 22)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 24)),
            nano_id: i16::from_le_bytes(read_array(bytes, 28)),
            nano_stamina: i16::from_le_bytes(read_array(bytes, 30)),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 32)),
            cstb_del: i32::from_le_bytes(read_array(bytes, 36)),
        }
    }
}

impl WirePayload for CAttackResult0104 {
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

/// `sCNStreetStall_ItemInfo_for_Client` (`#pragma pack(4)`, 20 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct StreetStallItemInfo0104 {
    /// `iListNum` at offset 0.
    pub list_num: i32,
    /// `Item` at offset 4.
    pub item: ItemBase0104,
    /// `iPrice` at offset 16.
    pub price: i32,
}

impl StreetStallItemInfo0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.list_num.to_le_bytes());
        self.item.write_into(&mut out[4..16]);
        write_prim(out, 16, &self.price.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            list_num: i32::from_le_bytes(read_array(bytes, 0)),
            item: ItemBase0104::read_from(&bytes[4..16]),
            price: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for StreetStallItemInfo0104 {
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

/// `sChannelInfo` (`#pragma pack(4)`, 8 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelInfo0104 {
    /// `iChannelNum` at offset 0.
    pub channel_num: i32,
    /// `iCurrentUserCnt` at offset 4.
    pub current_user_cnt: i32,
}

impl ChannelInfo0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.channel_num.to_le_bytes());
        write_prim(out, 4, &self.current_user_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            channel_num: i32::from_le_bytes(read_array(bytes, 0)),
            current_user_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for ChannelInfo0104 {
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

/// `sEPElement` (`#pragma pack(4)`, 36 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpElement0104 {
    /// `iLID` at offset 0.
    pub lid: i32,
    /// `iGID` at offset 4.
    pub gid: i32,
    /// `iType` at offset 8.
    pub type_: i32,
    /// `iTargetGID` at offset 12.
    pub target_gid: i32,
    /// `iX` at offset 16.
    pub x: i32,
    /// `iY` at offset 20.
    pub y: i32,
    /// `iZ` at offset 24.
    pub z: i32,
    /// `iEnable` at offset 28.
    pub enable: i32,
    /// `iONOFF` at offset 32.
    pub onoff: i32,
}

impl EpElement0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.lid.to_le_bytes());
        write_prim(out, 4, &self.gid.to_le_bytes());
        write_prim(out, 8, &self.type_.to_le_bytes());
        write_prim(out, 12, &self.target_gid.to_le_bytes());
        write_prim(out, 16, &self.x.to_le_bytes());
        write_prim(out, 20, &self.y.to_le_bytes());
        write_prim(out, 24, &self.z.to_le_bytes());
        write_prim(out, 28, &self.enable.to_le_bytes());
        write_prim(out, 32, &self.onoff.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            lid: i32::from_le_bytes(read_array(bytes, 0)),
            gid: i32::from_le_bytes(read_array(bytes, 4)),
            type_: i32::from_le_bytes(read_array(bytes, 8)),
            target_gid: i32::from_le_bytes(read_array(bytes, 12)),
            x: i32::from_le_bytes(read_array(bytes, 16)),
            y: i32::from_le_bytes(read_array(bytes, 20)),
            z: i32::from_le_bytes(read_array(bytes, 24)),
            enable: i32::from_le_bytes(read_array(bytes, 28)),
            onoff: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for EpElement0104 {
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

/// `sEPRecord` (`#pragma pack(2)`, 6 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EpRecord0104 {
    /// `uiScore` at offset 0.
    pub score: i16,
    /// `uiRank` at offset 2.
    pub rank: i8,
    /// `uiRing` at offset 3.
    pub ring: i8,
    /// `uiTime` at offset 4.
    pub time: i16,
}

impl EpRecord0104 {
    pub const SIZE: usize = 6;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.score.to_le_bytes());
        write_prim(out, 2, &self.rank.to_le_bytes());
        write_prim(out, 3, &self.ring.to_le_bytes());
        write_prim(out, 4, &self.time.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            score: i16::from_le_bytes(read_array(bytes, 0)),
            rank: i8::from_le_bytes(read_array(bytes, 2)),
            ring: i8::from_le_bytes(read_array(bytes, 3)),
            time: i16::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for EpRecord0104 {
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

/// `sEmailInfo` (`#pragma pack(4)`, 204 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EmailInfo0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iFromPCUID` at offset 8.
    pub from_pcuid: i64,
    /// `szFirstName` at offset 16.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 34.
    pub last_name: FixedUtf16<17>,
    /// `szSubject` at offset 68.
    pub subject: FixedUtf16<32>,
    /// `iReadFlag` at offset 132.
    pub read_flag: i32,
    /// `SendTime` at offset 136.
    pub send_time: SystemTime0104,
    /// `DeleteTime` at offset 168.
    pub delete_time: SystemTime0104,
    /// `iItemCandyFlag` at offset 200.
    pub item_candy_flag: i32,
}

impl EmailInfo0104 {
    pub const SIZE: usize = 204;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_prim(out, 8, &self.from_pcuid.to_le_bytes());
        write_utf16(out, 16, &self.first_name);
        write_utf16(out, 34, &self.last_name);
        write_utf16(out, 68, &self.subject);
        write_prim(out, 132, &self.read_flag.to_le_bytes());
        self.send_time.write_into(&mut out[136..168]);
        self.delete_time.write_into(&mut out[168..200]);
        write_prim(out, 200, &self.item_candy_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            from_pcuid: i64::from_le_bytes(read_array(bytes, 8)),
            first_name: read_utf16(bytes, 16),
            last_name: read_utf16(bytes, 34),
            subject: read_utf16(bytes, 68),
            read_flag: i32::from_le_bytes(read_array(bytes, 132)),
            send_time: SystemTime0104::read_from(&bytes[136..168]),
            delete_time: SystemTime0104::read_from(&bytes[168..200]),
            item_candy_flag: i32::from_le_bytes(read_array(bytes, 200)),
        }
    }
}

impl WirePayload for EmailInfo0104 {
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

/// `sEmailItemInfoFromCL` (`#pragma pack(4)`, 16 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct EmailItemInfoFromClient0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i32,
    /// `ItemInven` at offset 4.
    pub item_inven: ItemBase0104,
}

impl EmailItemInfoFromClient0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
        self.item_inven.write_into(&mut out[4..16]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            item_inven: ItemBase0104::read_from(&bytes[4..16]),
        }
    }
}

impl WirePayload for EmailItemInfoFromClient0104 {
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

/// `sGroupNPCLocationData` (`#pragma pack(4)`, 44 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupNpcLocationData0104 {
    /// `iGroupType` at offset 0.
    pub group_type: i32,
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
    /// `aGroupNPCIDs` at offset 24.
    pub group_npci_ds: [i32; 5],
}

impl GroupNpcLocationData0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.group_type.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.angle.to_le_bytes());
        write_prim(out, 20, &self.route.to_le_bytes());
        for (index, value) in self.group_npci_ds.iter().enumerate() {
            write_prim(out, 24 + index * 4, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            group_type: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            angle: i32::from_le_bytes(read_array(bytes, 16)),
            route: i32::from_le_bytes(read_array(bytes, 20)),
            group_npci_ds: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 24 + index * 4))
            }),
        }
    }
}

impl WirePayload for GroupNpcLocationData0104 {
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

/// `sItemBase` (`#pragma pack(4)`, 12 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemBase0104 {
    /// `iType` at offset 0.
    pub type_: i16,
    /// `iID` at offset 2.
    pub id: i16,
    /// `iOpt` at offset 4.
    pub opt: i32,
    /// `iTimeLimit` at offset 8.
    pub time_limit: i32,
}

impl ItemBase0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.type_.to_le_bytes());
        write_prim(out, 2, &self.id.to_le_bytes());
        write_prim(out, 4, &self.opt.to_le_bytes());
        write_prim(out, 8, &self.time_limit.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            type_: i16::from_le_bytes(read_array(bytes, 0)),
            id: i16::from_le_bytes(read_array(bytes, 2)),
            opt: i32::from_le_bytes(read_array(bytes, 4)),
            time_limit: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for ItemBase0104 {
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

/// `sItemReward` (`#pragma pack(4)`, 20 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemReward0104 {
    /// `sItem` at offset 0.
    pub s_item: ItemBase0104,
    /// `eIL` at offset 12.
    pub e_il: i32,
    /// `iSlotNum` at offset 16.
    pub slot_num: i32,
}

impl ItemReward0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.s_item.write_into(&mut out[0..12]);
        write_prim(out, 12, &self.e_il.to_le_bytes());
        write_prim(out, 16, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            s_item: ItemBase0104::read_from(&bytes[0..12]),
            e_il: i32::from_le_bytes(read_array(bytes, 12)),
            slot_num: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for ItemReward0104 {
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
