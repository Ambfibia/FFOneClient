use super::*;

impl WirePayload for PcGrenadeStyleFireRequest0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.skill_id);
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            skill_id: read_i32(bytes, 0),
            destination: [read_i32(bytes, 4), read_i32(bytes, 8), read_i32(bytes, 12)],
        })
    }
}

/// Clean protocol-0104 `sPCBullet` (`Pack = 4`, 12 bytes). For weapon
/// warheads `id` is the WeaponItemTable index; attack type 2 instead makes it
/// a Nano skill index.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PcBullet0104 {
    pub attack_type: i32,
    pub id: i32,
    pub charged: i32,
}

impl PcBullet0104 {
    pub(super) fn encode_at(self, out: &mut [u8], offset: usize) {
        write_i32(out, offset, self.attack_type);
        write_i32(out, offset + 4, self.id);
        write_i32(out, offset + 8, self.charged);
    }

    pub(super) fn decode_at(bytes: &[u8], offset: usize) -> Self {
        Self {
            attack_type: read_i32(bytes, offset),
            id: read_i32(bytes, offset + 4),
            charged: read_i32(bytes, offset + 8),
        }
    }
}

/// Clean caster-only `sP_FE2CL_REP_PC_ROCKET_STYLE_FIRE_SUCC`
/// (`Pack = 4`, 56 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRocketStyleFireSuccess0104 {
    pub skill_id: i32,
    pub position: [i32; 3],
    pub destination: [i32; 3],
    pub bullet_id: i8,
    pub pack_padding: [u8; 3],
    pub bullet: PcBullet0104,
    pub weapon_battery: i32,
    pub nano_deactivated: i32,
    pub nano_id: i16,
    pub nano_stamina: i16,
}

impl WirePayload for PcRocketStyleFireSuccess0104 {
    const SIZE: usize = 56;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.skill_id);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 16 + index * 4, value);
        }
        out[28] = self.bullet_id as u8;
        out[29..32].copy_from_slice(&self.pack_padding);
        self.bullet.encode_at(&mut out, 32);
        write_i32(&mut out, 44, self.weapon_battery);
        write_i32(&mut out, 48, self.nano_deactivated);
        out[52..54].copy_from_slice(&self.nano_id.to_le_bytes());
        out[54..56].copy_from_slice(&self.nano_stamina.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            skill_id: read_i32(bytes, 0),
            position: [read_i32(bytes, 4), read_i32(bytes, 8), read_i32(bytes, 12)],
            destination: [
                read_i32(bytes, 16),
                read_i32(bytes, 20),
                read_i32(bytes, 24),
            ],
            bullet_id: bytes[28] as i8,
            pack_padding: bytes[29..32].try_into().expect("fixed three-byte padding"),
            bullet: PcBullet0104::decode_at(bytes, 32),
            weapon_battery: read_i32(bytes, 44),
            nano_deactivated: read_i32(bytes, 48),
            nano_id: read_i16(bytes, 52),
            nano_stamina: read_i16(bytes, 54),
        })
    }
}

/// Clean viewer-facing `sP_FE2CL_PC_ROCKET_STYLE_FIRE`
/// (`Pack = 4`, 48 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRocketStyleFire0104 {
    pub pc_id: i32,
    pub position: [i32; 3],
    pub destination: [i32; 3],
    pub bullet_id: i8,
    pub pack_padding: [u8; 3],
    pub bullet: PcBullet0104,
    pub nano_deactivated: i32,
}

impl WirePayload for PcRocketStyleFire0104 {
    const SIZE: usize = 48;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 16 + index * 4, value);
        }
        out[28] = self.bullet_id as u8;
        out[29..32].copy_from_slice(&self.pack_padding);
        self.bullet.encode_at(&mut out, 32);
        write_i32(&mut out, 44, self.nano_deactivated);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            position: [read_i32(bytes, 4), read_i32(bytes, 8), read_i32(bytes, 12)],
            destination: [
                read_i32(bytes, 16),
                read_i32(bytes, 20),
                read_i32(bytes, 24),
            ],
            bullet_id: bytes[28] as i8,
            pack_padding: bytes[29..32].try_into().expect("fixed three-byte padding"),
            bullet: PcBullet0104::decode_at(bytes, 32),
            nano_deactivated: read_i32(bytes, 44),
        })
    }
}

/// Clean caster-only `sP_FE2CL_REP_PC_GRENADE_STYLE_FIRE_SUCC`
/// (`Pack = 4`, 44 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcGrenadeStyleFireSuccess0104 {
    pub skill_id: i32,
    pub destination: [i32; 3],
    pub bullet_id: i8,
    pub pack_padding: [u8; 3],
    pub bullet: PcBullet0104,
    pub weapon_battery: i32,
    pub nano_deactivated: i32,
    pub nano_id: i16,
    pub nano_stamina: i16,
}

impl WirePayload for PcGrenadeStyleFireSuccess0104 {
    const SIZE: usize = 44;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.skill_id);
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        out[16] = self.bullet_id as u8;
        out[17..20].copy_from_slice(&self.pack_padding);
        self.bullet.encode_at(&mut out, 20);
        write_i32(&mut out, 32, self.weapon_battery);
        write_i32(&mut out, 36, self.nano_deactivated);
        out[40..42].copy_from_slice(&self.nano_id.to_le_bytes());
        out[42..44].copy_from_slice(&self.nano_stamina.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            skill_id: read_i32(bytes, 0),
            destination: [read_i32(bytes, 4), read_i32(bytes, 8), read_i32(bytes, 12)],
            bullet_id: bytes[16] as i8,
            pack_padding: bytes[17..20].try_into().expect("fixed three-byte padding"),
            bullet: PcBullet0104::decode_at(bytes, 20),
            weapon_battery: read_i32(bytes, 32),
            nano_deactivated: read_i32(bytes, 36),
            nano_id: read_i16(bytes, 40),
            nano_stamina: read_i16(bytes, 42),
        })
    }
}

/// Clean viewer-facing `sP_FE2CL_PC_GRENADE_STYLE_FIRE`
/// (`Pack = 4`, 36 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcGrenadeStyleFire0104 {
    pub pc_id: i32,
    pub destination: [i32; 3],
    pub bullet_id: i8,
    pub pack_padding: [u8; 3],
    pub bullet: PcBullet0104,
    pub nano_deactivated: i32,
}

impl WirePayload for PcGrenadeStyleFire0104 {
    const SIZE: usize = 36;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        out[16] = self.bullet_id as u8;
        out[17..20].copy_from_slice(&self.pack_padding);
        self.bullet.encode_at(&mut out, 20);
        write_i32(&mut out, 32, self.nano_deactivated);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            destination: [read_i32(bytes, 4), read_i32(bytes, 8), read_i32(bytes, 12)],
            bullet_id: bytes[16] as i8,
            pack_padding: bytes[17..20].try_into().expect("fixed three-byte padding"),
            bullet: PcBullet0104::decode_at(bytes, 20),
            nano_deactivated: read_i32(bytes, 32),
        })
    }
}

/// One authoritative weapon/Nano warhead delivery. OpenFusion intentionally
/// sends the grenade-success layout for weapon rockets, so consumers must use
/// the packet route and `bullet.id` independently rather than assuming that
/// the grenade layout implies grenade motion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcWarheadFirePacket0104 {
    LocalRocket(PcRocketStyleFireSuccess0104),
    RemoteRocket(PcRocketStyleFire0104),
    LocalGrenade(PcGrenadeStyleFireSuccess0104),
    RemoteGrenade(PcGrenadeStyleFire0104),
}

/// `P_FE2CL_AROUND_DEL_NPC`: four-byte count followed by NPC IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AroundDelNpc0104 {
    pub npc_ids: Vec<i32>,
}

impl AroundDelNpc0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(self.npc_ids.len(), 4, 4, None)?;
        write_i32(&mut out, 0, self.npc_ids.len() as i32);
        for (index, id) in self.npc_ids.iter().copied().enumerate() {
            write_i32(&mut out, 4 + index * 4, id);
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        Ok(Self {
            npc_ids: decode_counted_i32(payload, 4, 0, None)?,
        })
    }
}

/// `P_FE2CL_AROUND_DEL_PC`: four-byte count followed by player IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AroundDelPc0104 {
    pub pc_ids: Vec<i32>,
}

impl AroundDelPc0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(self.pc_ids.len(), 4, 4, None)?;
        write_i32(&mut out, 0, self.pc_ids.len() as i32);
        for (index, id) in self.pc_ids.iter().copied().enumerate() {
            write_i32(&mut out, 4 + index * 4, id);
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        Ok(Self {
            pc_ids: decode_counted_i32(payload, 4, 0, None)?,
        })
    }
}

/// Response to the local player's hitscan attack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAttackNpcsSuccess0104 {
    pub battery_w: i32,
    pub results: Vec<AttackResult0104>,
}

impl PcAttackNpcsSuccess0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        encode_attack_results(self.battery_w, &self.results)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 8)?;
        Ok(Self {
            battery_w: read_i32(payload, 0),
            results: decode_attack_results(payload)?,
        })
    }
}

/// Broadcast of another player's hitscan attack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAttackNpcs0104 {
    pub pc_id: i32,
    pub results: Vec<AttackResult0104>,
}

impl PcAttackNpcs0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        encode_attack_results(self.pc_id, &self.results)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 8)?;
        Ok(Self {
            pc_id: read_i32(payload, 0),
            results: decode_attack_results(payload)?,
        })
    }
}

/// One PvP-mode hitscan target as the clean client writes it after
/// `sP_CL2FE_REQ_PC_ATTACK_CHARs`.
///
/// `cnAvatarAttack` writes the `Status.iID` first and the entity type second
/// (`4` for an `NpcMoveController` target, `1` for a player). OpenFusion's
/// `sGM_PVPTarget` names the two words `eCT, iID` in the opposite order; the
/// clean client is the ABI authority here, and the divergence is recorded in
/// `../FusionForge/docs/reference/evidence/legacy/ffone/managed-code/wire-0104-openfusion-divergence.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcAttackCharsTarget0104 {
    pub id: i32,
    pub entity_type: i32,
}

impl PcAttackCharsTarget0104 {
    pub const NPC_ENTITY_TYPE: i32 = 4;
    pub const PC_ENTITY_TYPE: i32 = 1;
    pub(super) const SIZE: usize = 8;
}

/// `sP_CL2FE_REQ_PC_ATTACK_CHARs`: the PvP-mode (`bEnableAttackPC`) form of the
/// hitscan request, four header bytes plus eight bytes per target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAttackCharsRequest0104 {
    pub targets: Vec<PcAttackCharsTarget0104>,
}

impl PcAttackCharsRequest0104 {
    /// Registry bound from OpenFusion's `MAX_PC_ATTACK_CHARS` validation.
    pub const MAX_TARGETS: usize = 510;

    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(
            self.targets.len(),
            4,
            PcAttackCharsTarget0104::SIZE,
            Some(Self::MAX_TARGETS),
        )?;
        write_i32(&mut out, 0, self.targets.len() as i32);
        for (index, target) in self.targets.iter().enumerate() {
            let start = 4 + index * PcAttackCharsTarget0104::SIZE;
            write_i32(&mut out, start, target.id);
            write_i32(&mut out, start + 4, target.entity_type);
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 4)?;
        let signed_count = read_i32(payload, 0);
        if signed_count < 0 {
            return Err(CountedPayloadError0104::NegativeCount {
                count: signed_count,
            });
        }
        let count = signed_count as usize;
        let expected = allocate_counted_payload(
            count,
            4,
            PcAttackCharsTarget0104::SIZE,
            Some(Self::MAX_TARGETS),
        )?
        .len();
        if payload.len() != expected {
            return Err(CountedPayloadError0104::WrongSize {
                expected,
                actual: payload.len(),
            });
        }
        Ok(Self {
            targets: (0..count)
                .map(|index| {
                    let start = 4 + index * PcAttackCharsTarget0104::SIZE;
                    PcAttackCharsTarget0104 {
                        id: read_i32(payload, start),
                        entity_type: read_i32(payload, start + 4),
                    }
                })
                .collect(),
        })
    }
}

/// `sP_FE2CL_PC_ATTACK_CHARs_SUCC`: the caster's PvP hitscan reply. The clean
/// client stores `iBatteryW` and routes every result by `eCT` (`4` NPC,
/// otherwise player).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAttackCharsSuccess0104 {
    pub battery_w: i32,
    pub results: Vec<AttackResult0104>,
}

impl PcAttackCharsSuccess0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        encode_attack_results(self.battery_w, &self.results)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 8)?;
        Ok(Self {
            battery_w: read_i32(payload, 0),
            results: decode_attack_results(payload)?,
        })
    }
}

/// `sP_FE2CL_PC_ATTACK_CHARs`: broadcast of another player's PvP hitscan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAttackChars0104 {
    pub pc_id: i32,
    pub results: Vec<AttackResult0104>,
}

impl PcAttackChars0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        encode_attack_results(self.pc_id, &self.results)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 8)?;
        Ok(Self {
            pc_id: read_i32(payload, 0),
            results: decode_attack_results(payload)?,
        })
    }
}

/// `sP_FE2CL_NPC_ATTACK_CHARs`: one NPC attacking a mixed player/NPC target
/// set. The clean client treats `eCT == 1` results as players (a local hit
/// spawns the damage bullet and enters combat) and every other result as an
/// NPC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcAttackChars0104 {
    pub npc_id: i32,
    pub results: Vec<AttackResult0104>,
}

impl NpcAttackChars0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        encode_attack_results(self.npc_id, &self.results)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 8)?;
        Ok(Self {
            npc_id: read_i32(payload, 0),
            results: decode_attack_results(payload)?,
        })
    }
}

/// `sP_FE2CL_CHARACTER_ATTACK_CHARACTERs` (12-byte header): an NPC-owned
/// attack whose results the clean client resolves only through
/// `NpcContainer.SearchNpc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterAttackCharacters0104 {
    pub entity_type: i32,
    pub character_id: i32,
    pub results: Vec<AttackResult0104>,
}

impl CharacterAttackCharacters0104 {
    pub const HEADER_SIZE: usize = 12;

    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        encode_attack_results_after(&[self.entity_type, self.character_id], &self.results)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, Self::HEADER_SIZE)?;
        Ok(Self {
            entity_type: read_i32(payload, 0),
            character_id: read_i32(payload, 4),
            results: decode_attack_results_after(payload, Self::HEADER_SIZE)?,
        })
    }
}

/// Broadcast of one NPC attacking one or more player characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcAttackPcs0104 {
    pub npc_id: i32,
    pub results: Vec<AttackResult0104>,
}

/// Protocol-0104 `sP_FE2CL_REP_BARKER` (pack(4), 8 bytes).
///
/// The clean client resolves `mission_string_id` through
/// `MissionTable.m_pMissionStringData` and appends the result to the named
/// NPC's `PrintName` Barker FIFO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcBarker0104 {
    pub npc_id: i32,
    pub mission_string_id: i32,
}

/// Protocol-0104 `sP_CL2FE_REQ_BARKER` (pack(4), 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcBarkerRequest0104 {
    pub mission_task_id: i32,
    pub npc_id: i32,
}

impl WirePayload for NpcBarkerRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.mission_task_id);
        write_i32(&mut out, 4, self.npc_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            mission_task_id: read_i32(bytes, 0),
            npc_id: read_i32(bytes, 4),
        })
    }
}

impl WirePayload for NpcBarker0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.mission_string_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            mission_string_id: read_i32(bytes, 4),
        })
    }
}

impl NpcAttackPcs0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        encode_attack_results(self.npc_id, &self.results)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 8)?;
        Ok(Self {
            npc_id: read_i32(payload, 0),
            results: decode_attack_results(payload)?,
        })
    }
}

/// Animation-bearing server signal emitted by the clean 0104 NPC skill
/// family. Skill-hit trailers are intentionally left to the ability decoder:
/// their element layout depends on `eST`, while animation playback only owns
/// the fixed header identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcSkillSignal0104 {
    pub npc_id: i32,
    pub skill_id: Option<i16>,
    pub kind: NpcSkillSignalKind0104,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcSkillSignalKind0104 {
    Ready,
    Fire,
    Hit,
    CorruptionReady,
    CorruptionHit,
    Cancel,
}

/// Exact 28-byte `#pragma pack(4)` prefix of
/// `sP_FE2CL_NPC_SKILL_HIT` before its `eST`-dependent result records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcSkillHitPrefix0104 {
    pub npc_id: i32,
    pub skill_id: i16,
    pub pack_padding: [u8; 2],
    pub position: [i32; 3],
    pub skill_type: i32,
    pub target_count: i32,
}

impl NpcSkillHitPrefix0104 {
    pub const SIZE: usize = 28;

    #[must_use]
    pub fn encode_prefix(self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        out[4..6].copy_from_slice(&self.skill_id.to_le_bytes());
        out[6..8].copy_from_slice(&self.pack_padding);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        write_i32(&mut out, 20, self.skill_type);
        write_i32(&mut out, 24, self.target_count);
        out
    }

    pub(super) fn decode_prefix(payload: &[u8]) -> Result<Self, PayloadError> {
        require_prefix(payload, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(payload, 0),
            skill_id: read_i16(payload, 4),
            pack_padding: payload[6..8]
                .try_into()
                .expect("fixed two-byte NPC skill padding"),
            position: [
                read_i32(payload, 8),
                read_i32(payload, 12),
                read_i32(payload, 16),
            ],
            skill_type: read_i32(payload, 20),
            target_count: read_i32(payload, 24),
        })
    }
}

/// Strict, lossless ordinary NPC skill-hit body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSkillHit0104 {
    pub(super) prefix: NpcSkillHitPrefix0104,
    pub(super) result_bytes: Vec<u8>,
    pub(super) results: Vec<NanoSkillResult0104>,
}

impl NpcSkillHit0104 {
    pub const MAX_TARGETS: usize =
        (OPENFUSION_PAYLOAD_CAPACITY_0104 - NpcSkillHitPrefix0104::SIZE) / 40;

    pub fn decode(payload: &[u8]) -> Result<Self, NpcSkillAuthorityDecodeError0104> {
        let prefix = NpcSkillHitPrefix0104::decode_prefix(payload)?;
        let target_count = checked_npc_skill_target_count(
            "NPC_SKILL_HIT",
            prefix.target_count,
            Self::MAX_TARGETS,
            true,
        )?;
        let record_size = npc_skill_result_size_0104(prefix.skill_type).ok_or(
            NpcSkillAuthorityDecodeError0104::UnsupportedSkillType {
                skill_type: prefix.skill_type,
            },
        )?;
        if record_size == 0 && target_count != 0 {
            return Err(NpcSkillAuthorityDecodeError0104::NoResultSkillHasTargets {
                skill_type: prefix.skill_type,
                target_count: prefix.target_count,
            });
        }
        let expected = target_count
            .checked_mul(record_size)
            .and_then(|tail_size| NpcSkillHitPrefix0104::SIZE.checked_add(tail_size))
            .ok_or(NpcSkillAuthorityDecodeError0104::PayloadSizeOverflow {
                packet: "NPC_SKILL_HIT",
                record_size,
                target_count: prefix.target_count,
            })?;
        require_size(payload, expected)?;
        let result_bytes = payload[NpcSkillHitPrefix0104::SIZE..].to_vec();
        let results = if record_size == 0 {
            Vec::new()
        } else {
            result_bytes
                .chunks_exact(record_size)
                .map(|record| {
                    decode_nano_skill_result_0104(prefix.skill_type, record).map_err(|_| {
                        NpcSkillAuthorityDecodeError0104::UnsupportedSkillType {
                            skill_type: prefix.skill_type,
                        }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        debug_assert_eq!(results.len(), target_count);
        Ok(Self {
            prefix,
            result_bytes,
            results,
        })
    }

    #[must_use]
    pub const fn prefix(&self) -> NpcSkillHitPrefix0104 {
        self.prefix
    }

    #[must_use]
    pub fn result_bytes(&self) -> &[u8] {
        &self.result_bytes
    }

    #[must_use]
    pub fn results(&self) -> &[NanoSkillResult0104] {
        &self.results
    }

    #[must_use]
    pub fn result_record_size(&self) -> usize {
        npc_skill_result_size_0104(self.prefix.skill_type)
            .expect("validated NPC skill hit retains a supported eST")
    }
}
