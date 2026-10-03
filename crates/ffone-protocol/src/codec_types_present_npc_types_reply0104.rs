use super::*;

impl WirePayload for PcRegenRequest0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.regen_type);
        write_i32(&mut out, 4, self.e_il);
        write_i32(&mut out, 8, self.index);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            regen_type: read_i32(bytes, 0),
            e_il: read_i32(bytes, 4),
            index: read_i32(bytes, 8),
        })
    }
}

/// Protocol-0104 `sPCRegenData` (`#pragma pack(4)`, 40 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRegenData0104 {
    pub hp: i32,
    pub map_number: i32,
    pub position: [i32; 3],
    pub active_nano_slot: i16,
    pub nanos: [Nano0104; 3],
}

impl PcRegenData0104 {
    pub const SIZE: usize = 40;

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        write_i32(bytes, 0, self.hp);
        write_i32(bytes, 4, self.map_number);
        write_i32(bytes, 8, self.position[0]);
        write_i32(bytes, 12, self.position[1]);
        write_i32(bytes, 16, self.position[2]);
        bytes[20..22].copy_from_slice(&self.active_nano_slot.to_le_bytes());
        for (index, nano) in self.nanos.iter().copied().enumerate() {
            let start = 22 + index * Nano0104::SIZE;
            nano.encode_into(&mut bytes[start..start + Nano0104::SIZE]);
        }
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            hp: read_i32(bytes, 0),
            map_number: read_i32(bytes, 4),
            position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            active_nano_slot: read_i16(bytes, 20),
            nanos: std::array::from_fn(|index| {
                let start = 22 + index * Nano0104::SIZE;
                Nano0104::decode_exact(&bytes[start..start + Nano0104::SIZE])
            }),
        }
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_REGEN_SUCC`
/// (`#pragma pack(4)`, 48 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRegenSuccess0104 {
    pub regen_data: PcRegenData0104,
    pub move_location: i32,
    pub fusion_matter: i32,
}

impl WirePayload for PcRegenSuccess0104 {
    const SIZE: usize = 48;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.regen_data
            .encode_into(&mut out[0..PcRegenData0104::SIZE]);
        write_i32(&mut out, 40, self.move_location);
        write_i32(&mut out, 44, self.fusion_matter);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            regen_data: PcRegenData0104::decode_exact(&bytes[0..PcRegenData0104::SIZE]),
            move_location: read_i32(bytes, 40),
            fusion_matter: read_i32(bytes, 44),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_PC_REGEN` broadcast for another PC
/// (`#pragma pack(4)`, 36 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRegen0104 {
    pub pc_id: i32,
    pub hp: i32,
    pub position: [i32; 3],
    pub angle: i32,
    pub condition_bit_flag: i32,
    pub pc_state: i8,
    pub special_state: i8,
    pub nano: Nano0104,
}

impl WirePayload for PcRegen0104 {
    const SIZE: usize = 36;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.hp);
        write_i32(&mut out, 8, self.position[0]);
        write_i32(&mut out, 12, self.position[1]);
        write_i32(&mut out, 16, self.position[2]);
        write_i32(&mut out, 20, self.angle);
        write_i32(&mut out, 24, self.condition_bit_flag);
        out[28] = self.pc_state as u8;
        out[29] = self.special_state as u8;
        self.nano.encode_into(&mut out[30..36]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            hp: read_i32(bytes, 4),
            position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            angle: read_i32(bytes, 20),
            condition_bit_flag: read_i32(bytes, 24),
            pc_state: bytes[28] as i8,
            special_state: bytes[29] as i8,
            nano: Nano0104::decode_exact(&bytes[30..36]),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_PC_SUDDEN_DEAD`
/// (`#pragma pack(4)`, 16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcSuddenDead0104 {
    pub pc_id: i32,
    pub sudden_dead_reason: i32,
    pub damage: i32,
    pub hp: i32,
}

impl WirePayload for PcSuddenDead0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.sudden_dead_reason);
        write_i32(&mut out, 8, self.damage);
        write_i32(&mut out, 12, self.hp);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            sudden_dead_reason: read_i32(bytes, 4),
            damage: read_i32(bytes, 8),
            hp: read_i32(bytes, 12),
        })
    }
}

/// The complete fixed-layout protocol-0104 PC regeneration packet family.
///
/// The clean ABI defines no PC-regeneration failure packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcRegenPacket0104 {
    Request(PcRegenRequest0104),
    Success(PcRegenSuccess0104),
    Broadcast(PcRegen0104),
    SuddenDead(PcSuddenDead0104),
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_EXIT` (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcExitRequest0104 {
    pub pc_id: i32,
}

impl WirePayload for PcExitRequest0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.pc_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_EXIT_FAIL` (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcExitFailure0104 {
    pub pc_id: i32,
    pub error_code: i32,
}

impl WirePayload for PcExitFailure0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            error_code: read_i32(bytes, 4),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_EXIT_SUCC` (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcExitSuccess0104 {
    pub pc_id: i32,
    pub exit_code: i32,
}

impl WirePayload for PcExitSuccess0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.exit_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            exit_code: read_i32(bytes, 4),
        })
    }
}

/// The protocol-0104 request/reply PC-exit handshake.
///
/// `P_FE2CL_PC_EXIT` (without `REP`) is deliberately excluded: it is the
/// separate broadcast announcing another player's departure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcExitPacket0104 {
    Request(PcExitRequest0104),
    Failure(PcExitFailure0104),
    Success(PcExitSuccess0104),
}

/// Protocol-0104 `sP_FE2CL_PC_EXIT` (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcExit0104 {
    pub pc_id: i32,
    pub exit_type: i32,
}

impl WirePayload for PcExit0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.exit_type);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            exit_type: read_i32(bytes, 4),
        })
    }
}

/// Protocol-0104 `sNPCAppearanceData` (36 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcAppearance0104 {
    pub npc_id: i32,
    pub npc_type: i32,
    pub hp: i32,
    pub condition_bit_flag: i32,
    pub position: [i32; 3],
    pub angle: i32,
    pub barker_type: i32,
}

impl NpcAppearance0104 {
    pub const SIZE: usize = 36;

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.npc_type);
        write_i32(&mut out, 8, self.hp);
        write_i32(&mut out, 12, self.condition_bit_flag);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 16 + index * 4, value);
        }
        write_i32(&mut out, 28, self.angle);
        write_i32(&mut out, 32, self.barker_type);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: read_i32(bytes, 0),
            npc_type: read_i32(bytes, 4),
            hp: read_i32(bytes, 8),
            condition_bit_flag: read_i32(bytes, 12),
            position: [
                read_i32(bytes, 16),
                read_i32(bytes, 20),
                read_i32(bytes, 24),
            ],
            angle: read_i32(bytes, 28),
            barker_type: read_i32(bytes, 32),
        }
    }
}

/// Protocol-0104 `sP_FE2CL_NPC_ENTER` (`#pragma pack(4)`, 36 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcEnter0104 {
    pub appearance: NpcAppearance0104,
}

impl WirePayload for NpcEnter0104 {
    const SIZE: usize = NpcAppearance0104::SIZE;

    fn encode(&self) -> Vec<u8> {
        self.appearance.encode()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        Ok(Self {
            appearance: NpcAppearance0104::decode(bytes)?,
        })
    }
}

/// Protocol-0104 `sP_FE2CL_NPC_EXIT` (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcExit0104 {
    pub npc_id: i32,
}

impl WirePayload for NpcExit0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.npc_id.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_NPC_MOVE` (`#pragma pack(4)`, 24 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcMove0104 {
    pub npc_id: i32,
    pub destination: [i32; 3],
    pub speed: i32,
    /// `0` is walk and non-zero is run in the effective legacy client.
    pub move_style: i16,
}

impl WirePayload for NpcMove0104 {
    const SIZE: usize = 24;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        write_i32(&mut out, 16, self.speed);
        out[20..22].copy_from_slice(&self.move_style.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            destination: [read_i32(bytes, 4), read_i32(bytes, 8), read_i32(bytes, 12)],
            speed: read_i32(bytes, 16),
            move_style: read_i16(bytes, 20),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_NPC_NEW` (`#pragma pack(4)`, 36 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcNew0104 {
    pub appearance: NpcAppearance0104,
}

impl WirePayload for NpcNew0104 {
    const SIZE: usize = NpcAppearance0104::SIZE;

    fn encode(&self) -> Vec<u8> {
        self.appearance.encode()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        Ok(Self {
            appearance: NpcAppearance0104::decode(bytes)?,
        })
    }
}

/// Protocol-0104 `sAttackResult` (`#pragma pack(4)`, 24 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackResult0104 {
    pub entity_type: i32,
    pub id: i32,
    pub protected: i32,
    pub damage: i32,
    pub hp: i32,
    pub hit_flag: i8,
}

impl AttackResult0104 {
    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            entity_type: read_i32(bytes, 0),
            id: read_i32(bytes, 4),
            protected: read_i32(bytes, 8),
            damage: read_i32(bytes, 12),
            hp: read_i32(bytes, 16),
            hit_flag: bytes[20] as i8,
        }
    }
}

impl WirePayload for AttackResult0104 {
    const SIZE: usize = 24;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.entity_type);
        write_i32(&mut out, 4, self.id);
        write_i32(&mut out, 8, self.protected);
        write_i32(&mut out, 12, self.damage);
        write_i32(&mut out, 16, self.hp);
        out[20] = self.hit_flag as u8;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PRESENT_NPC_TYPES`
/// (`#pragma pack(4)`, 8 bytes).
///
/// The clean client sends the server time recorded after its preceding
/// present-NPC-types update. Zero requests the initial snapshot. This value is
/// transported losslessly; the request does not imply how the server will
/// choose between a full snapshot and an incremental response.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PresentNpcTypesRequest0104 {
    pub last_sync_time: u64,
}

impl WirePayload for PresentNpcTypesRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        self.last_sync_time.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            last_sync_time: read_u64(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PRESENT_NPC_TYPES` followed by `iCnt`
/// `int32_t` NPC type IDs.
///
/// The clean client treats every nonzero `bClear` value as "clear the current
/// presence set before adding this tail". OpenFusion sends `1` for the first
/// chunk and `0` for continuation chunks. The raw `i32` is retained so the
/// transport never normalizes a wire value. This response has no server-time
/// field; callers must use the independently observed server clock when
/// applying it to client state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentNpcTypesReply0104 {
    pub clear: i32,
    pub npc_types: Vec<i32>,
}

impl PresentNpcTypesReply0104 {
    pub const HEADER_SIZE: usize = 8;
    pub const MAX_NPC_TYPES_PER_PACKET: usize =
        (OPENFUSION_PAYLOAD_CAPACITY_0104 - Self::HEADER_SIZE) / size_of::<i32>();

    #[must_use]
    pub fn clears_existing(&self) -> bool {
        self.clear != 0
    }

    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(
            self.npc_types.len(),
            Self::HEADER_SIZE,
            size_of::<i32>(),
            Some(Self::MAX_NPC_TYPES_PER_PACKET),
        )?;
        write_i32(&mut out, 0, self.clear);
        write_i32(&mut out, 4, self.npc_types.len() as i32);
        for (index, npc_type) in self.npc_types.iter().copied().enumerate() {
            write_i32(
                &mut out,
                Self::HEADER_SIZE + index * size_of::<i32>(),
                npc_type,
            );
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        let npc_types = decode_counted_i32(
            payload,
            Self::HEADER_SIZE,
            4,
            Some(Self::MAX_NPC_TYPES_PER_PACKET),
        )?;
        Ok(Self {
            clear: read_i32(payload, 0),
            npc_types,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentNpcTypesPacket0104 {
    Request(PresentNpcTypesRequest0104),
    Reply(PresentNpcTypesReply0104),
}

/// Protocol-0104 `sPCGroupMemberInfo` (`#pragma pack(4)`, 112 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupPcMemberInfo0104 {
    pub pc_id: i32,
    pub pc_uid: i64,
    pub name_check: u8,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
    pub special_state: u8,
    pub level: i16,
    pub hp: i32,
    pub max_hp: i32,
    pub map_type: i32,
    pub map_number: i32,
    pub position: [i32; 3],
    pub nano_active: i32,
    pub nano: Nano0104,
}

impl GroupPcMemberInfo0104 {
    pub const SIZE: usize = 112;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: read_i32(bytes, 0),
            pc_uid: read_i64(bytes, 4),
            name_check: bytes[12],
            first_name: read_utf16(bytes, 14),
            last_name: read_utf16(bytes, 32),
            special_state: bytes[66],
            level: read_i16(bytes, 68),
            hp: read_i32(bytes, 72),
            max_hp: read_i32(bytes, 76),
            map_type: read_i32(bytes, 80),
            map_number: read_i32(bytes, 84),
            position: [
                read_i32(bytes, 88),
                read_i32(bytes, 92),
                read_i32(bytes, 96),
            ],
            nano_active: read_i32(bytes, 100),
            nano: Nano0104::decode_exact(&bytes[104..110]),
        }
    }
}

/// Protocol-0104 `sNPCGroupMemberInfo` (`#pragma pack(4)`, 32 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupNpcMemberInfo0104 {
    pub npc_id: i32,
    pub npc_type: i32,
    pub hp: i32,
    pub map_type: i32,
    pub map_number: i32,
    pub position: [i32; 3],
}

impl GroupNpcMemberInfo0104 {
    pub const SIZE: usize = 32;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: read_i32(bytes, 0),
            npc_type: read_i32(bytes, 4),
            hp: read_i32(bytes, 8),
            map_type: read_i32(bytes, 12),
            map_number: read_i32(bytes, 16),
            position: [
                read_i32(bytes, 20),
                read_i32(bytes, 24),
                read_i32(bytes, 28),
            ],
        }
    }
}

/// Complete group roster carried by the 0104 group HUD packet family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupRoster0104 {
    /// The first header ID: member/group context for PC packets, PC ID for NPC responses.
    pub context_id: i32,
    /// Present only in NPC invite/kick responses, where the second header ID is the NPC ID.
    pub npc_id: Option<i32>,
    pub pc_members: Vec<GroupPcMemberInfo0104>,
    pub npc_members: Vec<GroupNpcMemberInfo0104>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupPacket0104 {
    Roster(GroupRoster0104),
    LeaveSuccess,
}

/// `P_CL2FE_REQ_PC_ATTACK_NPCs`: four-byte count followed by `int32_t` NPC IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAttackNpcsRequest0104 {
    pub npc_ids: Vec<i32>,
}

impl PcAttackNpcsRequest0104 {
    /// OpenFusion's combat handler rejects more than three hitscan targets.
    pub const MAX_TARGETS: usize = 3;

    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(
            self.npc_ids.len(),
            4,
            size_of::<i32>(),
            Some(Self::MAX_TARGETS),
        )?;
        write_i32(&mut out, 0, self.npc_ids.len() as i32);
        for (index, id) in self.npc_ids.iter().copied().enumerate() {
            write_i32(&mut out, 4 + index * 4, id);
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        Ok(Self {
            npc_ids: decode_counted_i32(payload, 4, 0, Some(Self::MAX_TARGETS))?,
        })
    }
}

/// Clean protocol-0104 `sP_CL2FE_REQ_PC_ROCKET_STYLE_FIRE`
/// (`Pack = 4`, 28 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRocketStyleFireRequest0104 {
    pub skill_id: i32,
    pub position: [i32; 3],
    pub destination: [i32; 3],
}

impl WirePayload for PcRocketStyleFireRequest0104 {
    const SIZE: usize = 28;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.skill_id);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        for (index, value) in self.destination.into_iter().enumerate() {
            write_i32(&mut out, 16 + index * 4, value);
        }
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
        })
    }
}

/// Clean protocol-0104 `sP_CL2FE_REQ_PC_GRENADE_STYLE_FIRE`
/// (`Pack = 4`, 16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcGrenadeStyleFireRequest0104 {
    pub skill_id: i32,
    pub destination: [i32; 3],
}
