// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_NPC_ROCKET_STYLE_FIRE` (`#pragma pack(4)`, 48 bytes, packet ID `0x31000082`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcRocketStyleFire0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
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
    pub bullet: NpcBullet0104,
}

impl NpcRocketStyleFire0104 {
    pub const SIZE: usize = 48;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.to_x.to_le_bytes());
        write_prim(out, 20, &self.to_y.to_le_bytes());
        write_prim(out, 24, &self.to_z.to_le_bytes());
        write_prim(out, 28, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[32..48]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            to_x: i32::from_le_bytes(read_array(bytes, 16)),
            to_y: i32::from_le_bytes(read_array(bytes, 20)),
            to_z: i32::from_le_bytes(read_array(bytes, 24)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 28)),
            bullet: NpcBullet0104::read_from(&bytes[32..48]),
        }
    }
}

impl WirePayload for NpcRocketStyleFire0104 {
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

/// `sP_FE2CL_NPC_GRENADE_STYLE_FIRE` (`#pragma pack(4)`, 36 bytes, packet ID `0x31000083`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcGrenadeStyleFire0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iToX` at offset 4.
    pub to_x: i32,
    /// `iToY` at offset 8.
    pub to_y: i32,
    /// `iToZ` at offset 12.
    pub to_z: i32,
    /// `iBulletID` at offset 16.
    pub bullet_id: i8,
    /// `Bullet` at offset 20.
    pub bullet: NpcBullet0104,
}

impl NpcGrenadeStyleFire0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.to_x.to_le_bytes());
        write_prim(out, 8, &self.to_y.to_le_bytes());
        write_prim(out, 12, &self.to_z.to_le_bytes());
        write_prim(out, 16, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[20..36]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            to_x: i32::from_le_bytes(read_array(bytes, 4)),
            to_y: i32::from_le_bytes(read_array(bytes, 8)),
            to_z: i32::from_le_bytes(read_array(bytes, 12)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 16)),
            bullet: NpcBullet0104::read_from(&bytes[20..36]),
        }
    }
}

impl WirePayload for NpcGrenadeStyleFire0104 {
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

/// `sP_FE2CL_NPC_BULLET_STYLE_HIT` (`#pragma pack(4)`, 28 bytes, packet ID `0x31000084`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcBulletStyleHit0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iBulletID` at offset 4.
    pub bullet_id: i8,
    /// `Bullet` at offset 8.
    pub bullet: NpcBullet0104,
    /// `iTargetCnt` at offset 24.
    pub target_cnt: i32,
}

impl NpcBulletStyleHit0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[8..24]);
        write_prim(out, 24, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 4)),
            bullet: NpcBullet0104::read_from(&bytes[8..24]),
            target_cnt: i32::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for NpcBulletStyleHit0104 {
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

/// `sP_FE2CL_CHARACTER_ATTACK_CHARACTERs` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000085`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterAttackCharacters0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iCharacterID` at offset 4.
    pub character_id: i32,
    /// `iTargetCnt` at offset 8.
    pub target_cnt: i32,
}

impl CharacterAttackCharacters0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.character_id.to_le_bytes());
        write_prim(out, 8, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            character_id: i32::from_le_bytes(read_array(bytes, 4)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for CharacterAttackCharacters0104 {
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

/// `sP_FE2CL_PC_GROUP_INVITE` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000086`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupInvite0104 {
    /// `iHostID` at offset 0.
    pub host_id: i32,
}

impl PcGroupInvite0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.host_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            host_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcGroupInvite0104 {
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

/// `sP_FE2CL_PC_GROUP_INVITE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000087`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupInviteFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcGroupInviteFailure0104 {
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

impl WirePayload for PcGroupInviteFailure0104 {
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

/// `sP_FE2CL_PC_GROUP_INVITE_REFUSE` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000088`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupInviteRefuse0104 {
    /// `iID_To` at offset 0.
    pub id_to: i32,
}

impl PcGroupInviteRefuse0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_to: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcGroupInviteRefuse0104 {
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

/// `sP_FE2CL_PC_GROUP_JOIN` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000089`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupJoin0104 {
    /// `iID_NewMember` at offset 0.
    pub id_new_member: i32,
    /// `iMemberPCCnt` at offset 4.
    pub member_pc_cnt: i32,
    /// `iMemberNPCCnt` at offset 8.
    pub member_npc_cnt: i32,
}

impl PcGroupJoin0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_new_member.to_le_bytes());
        write_prim(out, 4, &self.member_pc_cnt.to_le_bytes());
        write_prim(out, 8, &self.member_npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_new_member: i32::from_le_bytes(read_array(bytes, 0)),
            member_pc_cnt: i32::from_le_bytes(read_array(bytes, 4)),
            member_npc_cnt: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcGroupJoin0104 {
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

/// `sP_FE2CL_PC_GROUP_JOIN_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100008a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupJoinFailure0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcGroupJoinFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcGroupJoinFailure0104 {
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

/// `sP_FE2CL_PC_GROUP_JOIN_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x3100008b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupJoinSuccess0104 {
    /// `iID_NewMember` at offset 0.
    pub id_new_member: i32,
    /// `iMemberPCCnt` at offset 4.
    pub member_pc_cnt: i32,
    /// `iMemberNPCCnt` at offset 8.
    pub member_npc_cnt: i32,
}

impl PcGroupJoinSuccess0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_new_member.to_le_bytes());
        write_prim(out, 4, &self.member_pc_cnt.to_le_bytes());
        write_prim(out, 8, &self.member_npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_new_member: i32::from_le_bytes(read_array(bytes, 0)),
            member_pc_cnt: i32::from_le_bytes(read_array(bytes, 4)),
            member_npc_cnt: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcGroupJoinSuccess0104 {
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

/// `sP_FE2CL_PC_GROUP_LEAVE` (`#pragma pack(4)`, 12 bytes, packet ID `0x3100008c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupLeave0104 {
    /// `iID_LeaveMember` at offset 0.
    pub id_leave_member: i32,
    /// `iMemberPCCnt` at offset 4.
    pub member_pc_cnt: i32,
    /// `iMemberNPCCnt` at offset 8.
    pub member_npc_cnt: i32,
}

impl PcGroupLeave0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_leave_member.to_le_bytes());
        write_prim(out, 4, &self.member_pc_cnt.to_le_bytes());
        write_prim(out, 8, &self.member_npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_leave_member: i32::from_le_bytes(read_array(bytes, 0)),
            member_pc_cnt: i32::from_le_bytes(read_array(bytes, 4)),
            member_npc_cnt: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcGroupLeave0104 {
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

/// `sP_FE2CL_PC_GROUP_LEAVE_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x3100008d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupLeaveFailure0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcGroupLeaveFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcGroupLeaveFailure0104 {
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

/// `sP_FE2CL_PC_GROUP_LEAVE_SUCC` (`#pragma pack(8)`, 1 bytes, packet ID `0x3100008e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupLeaveSuccess0104;

impl PcGroupLeaveSuccess0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        out[0] = 0;
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self
    }
}

impl WirePayload for PcGroupLeaveSuccess0104 {
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

/// `sP_FE2CL_PC_GROUP_MEMBER_INFO` (`#pragma pack(4)`, 12 bytes, packet ID `0x3100008f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupMemberInfo0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iMemberPCCnt` at offset 4.
    pub member_pc_cnt: i32,
    /// `iMemberNPCCnt` at offset 8.
    pub member_npc_cnt: i32,
}

impl PcGroupMemberInfo0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.member_pc_cnt.to_le_bytes());
        write_prim(out, 8, &self.member_npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            member_pc_cnt: i32::from_le_bytes(read_array(bytes, 4)),
            member_npc_cnt: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcGroupMemberInfo0104 {
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

/// `sP_FE2CL_REP_PC_WARP_USE_NPC_SUCC` (`#pragma pack(4)`, 36 bytes, packet ID `0x31000090`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpUseNpcSuccess0104 {
    /// `iX` at offset 0.
    pub x: i32,
    /// `iY` at offset 4.
    pub y: i32,
    /// `iZ` at offset 8.
    pub z: i32,
    /// `eIL` at offset 12.
    pub e_il: i32,
    /// `iItemSlotNum` at offset 16.
    pub item_slot_num: i32,
    /// `Item` at offset 20.
    pub item: ItemBase0104,
    /// `iCandy` at offset 32.
    pub candy: i32,
}

impl PcWarpUseNpcSuccess0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.x.to_le_bytes());
        write_prim(out, 4, &self.y.to_le_bytes());
        write_prim(out, 8, &self.z.to_le_bytes());
        write_prim(out, 12, &self.e_il.to_le_bytes());
        write_prim(out, 16, &self.item_slot_num.to_le_bytes());
        self.item.write_into(&mut out[20..32]);
        write_prim(out, 32, &self.candy.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            x: i32::from_le_bytes(read_array(bytes, 0)),
            y: i32::from_le_bytes(read_array(bytes, 4)),
            z: i32::from_le_bytes(read_array(bytes, 8)),
            e_il: i32::from_le_bytes(read_array(bytes, 12)),
            item_slot_num: i32::from_le_bytes(read_array(bytes, 16)),
            item: ItemBase0104::read_from(&bytes[20..32]),
            candy: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for PcWarpUseNpcSuccess0104 {
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

/// `sP_FE2CL_REP_PC_WARP_USE_NPC_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000091`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpUseNpcFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcWarpUseNpcFailure0104 {
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

impl WirePayload for PcWarpUseNpcFailure0104 {
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

/// `sP_FE2CL_REP_PC_AVATAR_EMOTES_CHAT` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000092`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAvatarEmotesChatReply0104 {
    /// `iID_From` at offset 0.
    pub id_from: i32,
    /// `iEmoteCode` at offset 4.
    pub emote_code: i32,
}

impl PcAvatarEmotesChatReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_from.to_le_bytes());
        write_prim(out, 4, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_from: i32::from_le_bytes(read_array(bytes, 0)),
            emote_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcAvatarEmotesChatReply0104 {
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
