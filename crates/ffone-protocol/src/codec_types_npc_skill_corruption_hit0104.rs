use super::*;

/// Exact 24-byte `#pragma pack(4)` prefix of
/// `sP_FE2CL_NPC_SKILL_CORRUPTION_HIT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcSkillCorruptionHitPrefix0104 {
    pub npc_id: i32,
    pub skill_id: i16,
    pub style: i16,
    pub position: [i32; 3],
    pub target_count: i32,
}

impl NpcSkillCorruptionHitPrefix0104 {
    pub const SIZE: usize = 24;

    #[must_use]
    pub fn encode_prefix(self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        out[4..6].copy_from_slice(&self.skill_id.to_le_bytes());
        out[6..8].copy_from_slice(&self.style.to_le_bytes());
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        write_i32(&mut out, 20, self.target_count);
        out
    }

    pub(super) fn decode_prefix(payload: &[u8]) -> Result<Self, PayloadError> {
        require_prefix(payload, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(payload, 0),
            skill_id: read_i16(payload, 4),
            style: read_i16(payload, 6),
            position: [
                read_i32(payload, 8),
                read_i32(payload, 12),
                read_i32(payload, 16),
            ],
            target_count: read_i32(payload, 20),
        })
    }
}

/// Exact 40-byte OpenFusion `sCAttackResult` corruption trailer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcSkillCorruptionResult0104 {
    pub target: NanoSkillTarget0104,
    pub protected: i32,
    pub damage: i32,
    /// Authoritative signed post-hit HP. OpenFusion does not clamp corruption
    /// damage, so a lethal hit may serialize a negative value.
    pub hp: i32,
    pub hit_flag: i8,
    pub pack_padding: [u8; 1],
    pub active_nano_slot: i16,
    pub nano_deactivated: i32,
    pub nano_id: i16,
    pub nano_stamina: i16,
    pub condition_bit_flag: i32,
    pub condition_status_deleted: i32,
}

impl NpcSkillCorruptionResult0104 {
    pub const SIZE: usize = 40;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            target: decode_nano_skill_target_0104(bytes),
            protected: read_i32(bytes, 8),
            damage: read_i32(bytes, 12),
            hp: read_i32(bytes, 16),
            hit_flag: bytes[20] as i8,
            pack_padding: [bytes[21]],
            active_nano_slot: read_i16(bytes, 22),
            nano_deactivated: read_i32(bytes, 24),
            nano_id: read_i16(bytes, 28),
            nano_stamina: read_i16(bytes, 30),
            condition_bit_flag: read_i32(bytes, 32),
            condition_status_deleted: read_i32(bytes, 36),
        }
    }
}

/// Strict, lossless corruption-hit body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSkillCorruptionHit0104 {
    pub(super) prefix: NpcSkillCorruptionHitPrefix0104,
    pub(super) result_bytes: Vec<u8>,
    pub(super) results: Vec<NpcSkillCorruptionResult0104>,
}

impl NpcSkillCorruptionHit0104 {
    pub const MAX_TARGETS: usize = (OPENFUSION_PAYLOAD_CAPACITY_0104
        - NpcSkillCorruptionHitPrefix0104::SIZE)
        / NpcSkillCorruptionResult0104::SIZE;

    pub fn decode(payload: &[u8]) -> Result<Self, NpcSkillAuthorityDecodeError0104> {
        let prefix = NpcSkillCorruptionHitPrefix0104::decode_prefix(payload)?;
        let target_count = checked_npc_skill_target_count(
            "NPC_SKILL_CORRUPTION_HIT",
            prefix.target_count,
            Self::MAX_TARGETS,
            false,
        )?;
        let expected = target_count
            .checked_mul(NpcSkillCorruptionResult0104::SIZE)
            .and_then(|tail_size| NpcSkillCorruptionHitPrefix0104::SIZE.checked_add(tail_size))
            .ok_or(NpcSkillAuthorityDecodeError0104::PayloadSizeOverflow {
                packet: "NPC_SKILL_CORRUPTION_HIT",
                record_size: NpcSkillCorruptionResult0104::SIZE,
                target_count: prefix.target_count,
            })?;
        require_size(payload, expected)?;
        let result_bytes = payload[NpcSkillCorruptionHitPrefix0104::SIZE..].to_vec();
        let results = result_bytes
            .chunks_exact(NpcSkillCorruptionResult0104::SIZE)
            .map(NpcSkillCorruptionResult0104::decode_exact)
            .collect::<Vec<_>>();
        debug_assert_eq!(results.len(), target_count);
        Ok(Self {
            prefix,
            result_bytes,
            results,
        })
    }

    #[must_use]
    pub const fn prefix(&self) -> NpcSkillCorruptionHitPrefix0104 {
        self.prefix
    }

    #[must_use]
    pub fn result_bytes(&self) -> &[u8] {
        &self.result_bytes
    }

    #[must_use]
    pub fn results(&self) -> &[NpcSkillCorruptionResult0104] {
        &self.results
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcSkillAuthorityPacket0104 {
    SkillHit(NpcSkillHit0104),
    CorruptionHit(NpcSkillCorruptionHit0104),
}

/// Confirmed server-to-client NPC lifecycle and basic hitscan combat family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcCombatPacket0104 {
    NpcEnter(NpcEnter0104),
    NpcExit(NpcExit0104),
    NpcMove(NpcMove0104),
    NpcNew(NpcNew0104),
    NpcAround(Vec<NpcAppearance0104>),
    AroundDelNpc(AroundDelNpc0104),
    PcAttackNpcsSuccess(PcAttackNpcsSuccess0104),
    PcAttackNpcs(PcAttackNpcs0104),
    NpcAttackPcs(NpcAttackPcs0104),
    PcAttackCharsSuccess(PcAttackCharsSuccess0104),
    PcAttackChars(PcAttackChars0104),
    NpcAttackChars(NpcAttackChars0104),
    CharacterAttackCharacters(CharacterAttackCharacters0104),
}

/// Protocol-0104 `sTransportationAppearanceData` (24 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportationAppearance0104 {
    pub transportation_kind: i32,
    pub id: i32,
    pub transportation_type: i32,
    pub position: [i32; 3],
}

impl TransportationAppearance0104 {
    pub const SIZE: usize = 24;

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.transportation_kind);
        write_i32(&mut out, 4, self.id);
        write_i32(&mut out, 8, self.transportation_type);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 12 + index * 4, value);
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            transportation_kind: read_i32(bytes, 0),
            id: read_i32(bytes, 4),
            transportation_type: read_i32(bytes, 8),
            position: [
                read_i32(bytes, 12),
                read_i32(bytes, 16),
                read_i32(bytes, 20),
            ],
        }
    }
}

/// `P_FE2CL_TRANSPORTATION_EXIT` (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportationExit0104 {
    pub transportation_kind: i32,
    pub id: i32,
}

impl WirePayload for TransportationExit0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.transportation_kind);
        write_i32(&mut out, 4, self.id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            transportation_kind: read_i32(bytes, 0),
            id: read_i32(bytes, 4),
        })
    }
}

/// `P_FE2CL_TRANSPORTATION_MOVE` (`#pragma pack(4)`, 28 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportationMove0104 {
    pub transportation_kind: i32,
    pub id: i32,
    pub destination: [i32; 3],
    pub speed: i32,
    pub move_style: i16,
}

impl WirePayload for TransportationMove0104 {
    const SIZE: usize = 28;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.transportation_kind);
        write_i32(&mut out, 4, self.id);
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        write_i32(&mut out, 20, self.speed);
        out[24..26].copy_from_slice(&self.move_style.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            transportation_kind: read_i32(bytes, 0),
            id: read_i32(bytes, 4),
            destination: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            speed: read_i32(bytes, 20),
            move_style: read_i16(bytes, 24),
        })
    }
}

/// `P_FE2CL_AROUND_DEL_TRANSPORTATION`: kind/count followed by IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AroundDelTransportation0104 {
    pub transportation_kind: i32,
    pub ids: Vec<i32>,
}

impl AroundDelTransportation0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(self.ids.len(), 8, 4, None)?;
        write_i32(&mut out, 0, self.transportation_kind);
        write_i32(&mut out, 4, self.ids.len() as i32);
        for (index, id) in self.ids.iter().copied().enumerate() {
            write_i32(&mut out, 8 + index * 4, id);
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        require_counted_header(payload, 8)?;
        Ok(Self {
            transportation_kind: read_i32(payload, 0),
            ids: decode_counted_i32(payload, 8, 4, None)?,
        })
    }
}

/// Protocol-0104 `sShinyAppearanceData` (24 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShinyAppearance0104 {
    pub shiny_id: i32,
    pub shiny_type: i32,
    pub map_number: i32,
    pub position: [i32; 3],
}

impl ShinyAppearance0104 {
    pub const SIZE: usize = 24;

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.shiny_id);
        write_i32(&mut out, 4, self.shiny_type);
        write_i32(&mut out, 8, self.map_number);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 12 + index * 4, value);
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shiny_id: read_i32(bytes, 0),
            shiny_type: read_i32(bytes, 4),
            map_number: read_i32(bytes, 8),
            position: [
                read_i32(bytes, 12),
                read_i32(bytes, 16),
                read_i32(bytes, 20),
            ],
        }
    }
}

/// `P_FE2CL_SHINY_EXIT` (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShinyExit0104 {
    pub shiny_id: i32,
}

impl WirePayload for ShinyExit0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.shiny_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            shiny_id: read_i32(bytes, 0),
        })
    }
}

/// `P_FE2CL_AROUND_DEL_SHINY`: count followed by Shiny IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AroundDelShiny0104 {
    pub shiny_ids: Vec<i32>,
}

impl AroundDelShiny0104 {
    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(self.shiny_ids.len(), 4, 4, None)?;
        write_i32(&mut out, 0, self.shiny_ids.len() as i32);
        for (index, id) in self.shiny_ids.iter().copied().enumerate() {
            write_i32(&mut out, 4 + index * 4, id);
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        Ok(Self {
            shiny_ids: decode_counted_i32(payload, 4, 0, None)?,
        })
    }
}

/// Engine-neutral result of one initial OpenFusion AROUND packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitialAroundPacket0104 {
    Players(Vec<PcAppearance0104>),
    Npcs(Vec<NpcAppearance0104>),
    Transportation(Vec<TransportationAppearance0104>),
    Shinies(Vec<ShinyAppearance0104>),
}

impl InitialAroundPacket0104 {
    pub fn len(&self) -> usize {
        match self {
            Self::Players(entries) => entries.len(),
            Self::Npcs(entries) => entries.len(),
            Self::Transportation(entries) => entries.len(),
            Self::Shinies(entries) => entries.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
