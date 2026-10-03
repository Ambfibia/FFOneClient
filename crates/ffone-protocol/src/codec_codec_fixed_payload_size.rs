use super::*;

impl fmt::Display for NpcSkillAuthorityDecodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(error) => error.fmt(formatter),
            Self::NegativeTargetCount {
                packet,
                target_count,
            } => write!(
                formatter,
                "{packet} target count cannot be negative: {target_count}"
            ),
            Self::EmptyTargetResults { packet } => {
                write!(formatter, "{packet} requires at least one result record")
            }
            Self::TargetCountTooLarge {
                packet,
                target_count,
                maximum,
            } => write!(
                formatter,
                "{packet} target count {target_count} exceeds OpenFusion maximum {maximum}"
            ),
            Self::UnsupportedSkillType { skill_type } => write!(
                formatter,
                "NPC skill type {skill_type} has no proven OpenFusion result-tail ABI"
            ),
            Self::NoResultSkillHasTargets {
                skill_type,
                target_count,
            } => write!(
                formatter,
                "NPC no-result skill type {skill_type} cannot declare {target_count} result targets"
            ),
            Self::PayloadSizeOverflow {
                packet,
                record_size,
                target_count,
            } => write!(
                formatter,
                "{packet} payload size overflows for record size {record_size} and target count {target_count}"
            ),
        }
    }
}

impl std::error::Error for NpcSkillAuthorityDecodeError0104 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Fixed(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcCombatDecodeError0104 {
    Fixed(PayloadError),
    Counted(CountedPayloadError0104),
    Around(AroundDecodeError),
}

impl fmt::Display for NpcCombatDecodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(error) => write!(formatter, "fixed NPC/combat payload: {error}"),
            Self::Counted(error) => write!(formatter, "counted NPC/combat payload: {error}"),
            Self::Around(error) => write!(formatter, "NPC AROUND payload: {error}"),
        }
    }
}

impl std::error::Error for NpcCombatDecodeError0104 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Fixed(error) => Some(error),
            Self::Counted(error) => Some(error),
            Self::Around(error) => Some(error),
        }
    }
}

impl From<PayloadError> for NpcCombatDecodeError0104 {
    fn from(error: PayloadError) -> Self {
        Self::Fixed(error)
    }
}

impl From<CountedPayloadError0104> for NpcCombatDecodeError0104 {
    fn from(error: CountedPayloadError0104) -> Self {
        Self::Counted(error)
    }
}

impl From<AroundDecodeError> for NpcCombatDecodeError0104 {
    fn from(error: AroundDecodeError) -> Self {
        Self::Around(error)
    }
}

/// Decode only this confirmed family. `Ok(None)` preserves unrelated gameplay
/// frames for the next protocol slice rather than pretending to understand them.
pub fn decode_npc_combat_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<NpcCombatPacket0104>, NpcCombatDecodeError0104> {
    let packet = match packet_type {
        packet::P_FE2CL_NPC_ENTER => NpcCombatPacket0104::NpcEnter(NpcEnter0104::decode(payload)?),
        packet::P_FE2CL_NPC_EXIT => NpcCombatPacket0104::NpcExit(NpcExit0104::decode(payload)?),
        packet::P_FE2CL_NPC_MOVE => NpcCombatPacket0104::NpcMove(NpcMove0104::decode(payload)?),
        packet::P_FE2CL_NPC_NEW => NpcCombatPacket0104::NpcNew(NpcNew0104::decode(payload)?),
        packet::P_FE2CL_NPC_AROUND => {
            NpcCombatPacket0104::NpcAround(decode_npc_around_0104(payload)?)
        }
        packet::P_FE2CL_AROUND_DEL_NPC => {
            NpcCombatPacket0104::AroundDelNpc(AroundDelNpc0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_ATTACK_NPCS_SUCC => {
            NpcCombatPacket0104::PcAttackNpcsSuccess(PcAttackNpcsSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_ROCKET_STYLE_HIT | packet::P_FE2CL_PC_GRENADE_STYLE_HIT => {
            NpcCombatPacket0104::PcAttackNpcs(decode_pc_warhead_hit_0104(payload)?)
        }
        packet::P_FE2CL_PC_ATTACK_NPCS => {
            NpcCombatPacket0104::PcAttackNpcs(PcAttackNpcs0104::decode(payload)?)
        }
        packet::P_FE2CL_NPC_ATTACK_PCS => {
            NpcCombatPacket0104::NpcAttackPcs(NpcAttackPcs0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_ATTACK_CHARS_SUCC => {
            NpcCombatPacket0104::PcAttackCharsSuccess(PcAttackCharsSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_ATTACK_CHARS => {
            NpcCombatPacket0104::PcAttackChars(PcAttackChars0104::decode(payload)?)
        }
        packet::P_FE2CL_NPC_ATTACK_CHARS => {
            NpcCombatPacket0104::NpcAttackChars(NpcAttackChars0104::decode(payload)?)
        }
        packet::P_FE2CL_CHARACTER_ATTACK_CHARACTERS => {
            NpcCombatPacket0104::CharacterAttackCharacters(CharacterAttackCharacters0104::decode(
                payload,
            )?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub(super) fn counted_payload_len(
    count: usize,
    header_size: usize,
    element_size: usize,
) -> Result<usize, CountedPayloadError0104> {
    count
        .checked_mul(element_size)
        .and_then(|trailers| header_size.checked_add(trailers))
        .ok_or(CountedPayloadError0104::LengthOverflow {
            count,
            header_size,
            element_size,
        })
}

pub(super) fn allocate_counted_payload(
    count: usize,
    header_size: usize,
    element_size: usize,
    explicit_maximum: Option<usize>,
) -> Result<Vec<u8>, CountedPayloadError0104> {
    let wire_maximum = (OPENFUSION_PAYLOAD_CAPACITY_0104 - header_size) / element_size;
    let maximum = explicit_maximum.map_or(wire_maximum, |limit| limit.min(wire_maximum));
    if count > maximum {
        return Err(CountedPayloadError0104::CountTooLarge { count, maximum });
    }
    Ok(vec![
        0;
        counted_payload_len(count, header_size, element_size)?
    ])
}

pub(super) fn decode_counted_i32(
    payload: &[u8],
    header_size: usize,
    count_offset: usize,
    explicit_maximum: Option<usize>,
) -> Result<Vec<i32>, CountedPayloadError0104> {
    require_counted_header(payload, header_size)?;
    let signed_count = read_i32(payload, count_offset);
    if signed_count < 0 {
        return Err(CountedPayloadError0104::NegativeCount {
            count: signed_count,
        });
    }
    let count = signed_count as usize;
    let expected = allocate_counted_payload(count, header_size, 4, explicit_maximum)?.len();
    if payload.len() != expected {
        return Err(CountedPayloadError0104::WrongSize {
            expected,
            actual: payload.len(),
        });
    }
    Ok((0..count)
        .map(|index| read_i32(payload, header_size + index * 4))
        .collect())
}

pub(super) fn encode_attack_results(
    first_header_value: i32,
    results: &[AttackResult0104],
) -> Result<Vec<u8>, CountedPayloadError0104> {
    encode_attack_results_after(&[first_header_value], results)
}

/// Encode `header_words` followed by the `i32` result count and the
/// `sAttackResult` trailer. Every clean attack broadcast keeps its count as the
/// last header word.
pub(super) fn encode_attack_results_after(
    header_words: &[i32],
    results: &[AttackResult0104],
) -> Result<Vec<u8>, CountedPayloadError0104> {
    let header_size = (header_words.len() + 1) * size_of::<i32>();
    let mut out =
        allocate_counted_payload(results.len(), header_size, AttackResult0104::SIZE, None)?;
    for (index, word) in header_words.iter().copied().enumerate() {
        write_i32(&mut out, index * size_of::<i32>(), word);
    }
    write_i32(
        &mut out,
        header_words.len() * size_of::<i32>(),
        results.len() as i32,
    );
    for (index, result) in results.iter().enumerate() {
        let start = header_size + index * AttackResult0104::SIZE;
        out[start..start + AttackResult0104::SIZE].copy_from_slice(&result.encode());
    }
    Ok(out)
}

pub(super) fn decode_attack_results(payload: &[u8]) -> Result<Vec<AttackResult0104>, CountedPayloadError0104> {
    decode_attack_results_after(payload, 8)
}

/// Decode the `sAttackResult` trailer that follows a `header_size`-byte header
/// whose last `i32` is the result count.
pub(super) fn decode_attack_results_after(
    payload: &[u8],
    header_size: usize,
) -> Result<Vec<AttackResult0104>, CountedPayloadError0104> {
    require_counted_header(payload, header_size)?;
    let signed_count = read_i32(payload, header_size - size_of::<i32>());
    if signed_count < 0 {
        return Err(CountedPayloadError0104::NegativeCount {
            count: signed_count,
        });
    }
    let count = signed_count as usize;
    let expected =
        allocate_counted_payload(count, header_size, AttackResult0104::SIZE, None)?.len();
    if payload.len() != expected {
        return Err(CountedPayloadError0104::WrongSize {
            expected,
            actual: payload.len(),
        });
    }
    Ok((0..count)
        .map(|index| {
            let start = header_size + index * AttackResult0104::SIZE;
            AttackResult0104::decode_exact(&payload[start..start + AttackResult0104::SIZE])
        })
        .collect())
}

/// Decoding failures specific to OpenFusion's variable-length initial AROUND packets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AroundDecodeError {
    UnsupportedPacket {
        packet_type: u32,
    },
    MissingCount {
        actual: usize,
    },
    NegativeCount {
        count: i32,
    },
    CountTooLarge {
        count: usize,
        maximum: usize,
    },
    LengthOverflow {
        count: usize,
        element_size: usize,
        base_offset: usize,
    },
    WrongSize {
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for AroundDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPacket { packet_type } => {
                write!(f, "unsupported initial AROUND packet {packet_type:#010x}")
            }
            Self::MissingCount { actual } => {
                write!(f, "AROUND payload needs a 4-byte count, got {actual}")
            }
            Self::NegativeCount { count } => {
                write!(f, "AROUND count must be non-negative, got {count}")
            }
            Self::CountTooLarge { count, maximum } => {
                write!(
                    f,
                    "AROUND count {count} exceeds OpenFusion maximum {maximum}"
                )
            }
            Self::LengthOverflow {
                count,
                element_size,
                base_offset,
            } => write!(
                f,
                "AROUND length overflows: base {base_offset} + {count} * {element_size}"
            ),
            Self::WrongSize { expected, actual } => {
                write!(f, "AROUND payload must be {expected} bytes, got {actual}")
            }
        }
    }
}

impl std::error::Error for AroundDecodeError {}

/// Dispatch one of the four initial AROUND packet payloads emitted by OpenFusion chunking.
pub fn decode_initial_around_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<InitialAroundPacket0104, AroundDecodeError> {
    match packet_type {
        packet::P_FE2CL_PC_AROUND => Ok(InitialAroundPacket0104::Players(decode_pc_around_0104(
            payload,
        )?)),
        packet::P_FE2CL_NPC_AROUND => Ok(InitialAroundPacket0104::Npcs(decode_npc_around_0104(
            payload,
        )?)),
        packet::P_FE2CL_TRANSPORTATION_AROUND => Ok(InitialAroundPacket0104::Transportation(
            decode_transportation_around_0104(payload)?,
        )),
        packet::P_FE2CL_SHINY_AROUND => Ok(InitialAroundPacket0104::Shinies(
            decode_shiny_around_0104(payload)?,
        )),
        _ => Err(AroundDecodeError::UnsupportedPacket { packet_type }),
    }
}

/// Decode `P_FE2CL_PC_AROUND`.
///
/// OpenFusion places the first player at byte 232 rather than immediately after the count.
pub fn decode_pc_around_0104(payload: &[u8]) -> Result<Vec<PcAppearance0104>, AroundDecodeError> {
    decode_around_entries(
        payload,
        PcAppearance0104::SIZE,
        PcAppearance0104::SIZE,
        PcAppearance0104::decode_exact,
    )
}

/// Decode `P_FE2CL_NPC_AROUND`; entries begin at byte 36.
pub fn decode_npc_around_0104(payload: &[u8]) -> Result<Vec<NpcAppearance0104>, AroundDecodeError> {
    decode_around_entries(
        payload,
        NpcAppearance0104::SIZE,
        NpcAppearance0104::SIZE,
        NpcAppearance0104::decode_exact,
    )
}

/// Decode `P_FE2CL_TRANSPORTATION_AROUND`; entries begin at byte 24.
pub fn decode_transportation_around_0104(
    payload: &[u8],
) -> Result<Vec<TransportationAppearance0104>, AroundDecodeError> {
    decode_around_entries(
        payload,
        TransportationAppearance0104::SIZE,
        TransportationAppearance0104::SIZE,
        TransportationAppearance0104::decode_exact,
    )
}

/// Decode `P_FE2CL_SHINY_AROUND`; unlike the other three, entries begin after the 4-byte count.
pub fn decode_shiny_around_0104(
    payload: &[u8],
) -> Result<Vec<ShinyAppearance0104>, AroundDecodeError> {
    decode_around_entries(
        payload,
        4,
        ShinyAppearance0104::SIZE,
        ShinyAppearance0104::decode_exact,
    )
}

// OpenFusion 0104 uses a 4096-byte full packet buffer, minus length and packet type.
pub(super) const OPENFUSION_PAYLOAD_CAPACITY_0104: usize = 4096 - 2 * size_of::<i32>();

pub(super) fn decode_around_entries<T>(
    payload: &[u8],
    base_offset: usize,
    element_size: usize,
    decode_entry: fn(&[u8]) -> T,
) -> Result<Vec<T>, AroundDecodeError> {
    if payload.len() < size_of::<i32>() {
        return Err(AroundDecodeError::MissingCount {
            actual: payload.len(),
        });
    }

    let signed_count = read_i32(payload, 0);
    if signed_count < 0 {
        return Err(AroundDecodeError::NegativeCount {
            count: signed_count,
        });
    }
    let count = signed_count as usize;
    let expected = checked_around_payload_len(count, base_offset, element_size)?;
    let maximum = (OPENFUSION_PAYLOAD_CAPACITY_0104 - base_offset) / element_size;
    if count > maximum {
        return Err(AroundDecodeError::CountTooLarge { count, maximum });
    }
    if payload.len() != expected {
        return Err(AroundDecodeError::WrongSize {
            expected,
            actual: payload.len(),
        });
    }

    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let start = base_offset + index * element_size;
        entries.push(decode_entry(&payload[start..start + element_size]));
    }
    Ok(entries)
}

pub(super) fn checked_around_payload_len(
    count: usize,
    base_offset: usize,
    element_size: usize,
) -> Result<usize, AroundDecodeError> {
    count
        .checked_mul(element_size)
        .and_then(|entries_size| base_offset.checked_add(entries_size))
        .ok_or(AroundDecodeError::LengthOverflow {
            count,
            element_size,
            base_offset,
        })
}

/// OpenFusion's fixed packet sizes for the first vertical slice.
pub fn fixed_payload_size(packet_id: u32) -> Option<usize> {
    Some(match packet_id {
        packet::P_CL2LS_REQ_LOGIN => LoginRequest::SIZE,
        packet::P_CL2LS_REQ_CHECK_CHAR_NAME => CharacterNameCheckRequest0104::SIZE,
        packet::P_CL2LS_REQ_SAVE_CHAR_NAME => CharacterNameSaveRequest0104::SIZE,
        packet::P_CL2LS_REQ_CHAR_CREATE => CharacterCreateRequest0104::SIZE,
        packet::P_CL2LS_REQ_CHAR_SELECT => CharacterSelectRequest::SIZE,
        packet::P_CL2LS_REQ_CHAR_DELETE => CharacterDeleteRequest0104::SIZE,
        packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR => CharacterTutorialSaveRequest0104::SIZE,
        packet::P_CL2LS_REQ_PC_EXIT_DUPLICATE => DuplicateExitRequest0104::SIZE,
        packet::P_CL2LS_REQ_CHANGE_CHAR_NAME => CharacterNameChangeRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_ENTER => PcEnterRequest::SIZE,
        packet::P_CL2FE_REQ_PC_EXIT => PcExitRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_REGEN => PcRegenRequest0104::SIZE,
        packet::P_CL2FE_REQ_ITEM_MOVE => ItemMoveRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_TASK_START => PcTaskStartRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_TASK_END => PcTaskEndRequest0104::SIZE,
        packet::P_CL2FE_REQ_NANO_EQUIP => NanoEquipRequest0104::SIZE,
        packet::P_CL2FE_REQ_NANO_UNEQUIP => NanoUnequipRequest0104::SIZE,
        packet::P_CL2FE_REQ_NANO_ACTIVE => NanoActiveRequest0104::SIZE,
        packet::P_CL2FE_REQ_NANO_TUNE => NanoTuneRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_TASK_STOP => PcTaskStopRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VENDOR_ITEM_BUY => VendorItemBuyRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VENDOR_ITEM_SELL => VendorItemSellRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_ITEM_DELETE => PcItemDeleteRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VENDOR_START => VendorStartRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE => VendorTableUpdateRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY => VendorItemRestoreBuyRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY => VendorBatteryBuyRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_DISASSEMBLE_ITEM => PcDisassembleItemRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_BANK_OPEN => PcBankOpenRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_BANK_CLOSE => PcBankCloseRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_MOVE => PcMoveRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_STOP => PcStopRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_JUMP => PcJumpRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_JUMPPAD => PcJumppadRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_LAUNCHER => PcLauncherRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_ZIPLINE => PcZiplineRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_MOVEPLATFORM => PcMovePlatformRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_SLOPE => PcSlopeRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_MOVETRANSPORTATION => PcMoveTransportationRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE => PcRocketStyleFireRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE => PcGrenadeStyleFireRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_WARP_USE_NPC => PcWarpUseNpcRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_CHANGE_MENTOR => PcChangeMentorRequest0104::SIZE,
        packet::P_CL2FE_REQ_ITEM_CHEST_OPEN => ItemChestOpenRequest0104::SIZE,
        packet::P_CL2FE_REQ_ITEM_USE => ItemUseRequest0104::SIZE,
        packet::P_CL2FE_REQ_NPC_INTERACTION => NpcInteractionRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH => PcSpecialStateSwitchRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID => PcSetCurrentMissionId0104::SIZE,
        packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT => QuickSlotRegisterRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_FREECHAT => FreeChatRequest0104::SIZE,
        packet::P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE => MenuChatRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT => AvatarEmoteChat0104::SIZE,
        packet::P_CL2FE_GM_REQ_PC_SET_VALUE => GmSetValueRequest0104::SIZE,
        packet::P_CL2FE_REQ_REQUEST_MAKE_BUDDY => BuddyMakeRequest0104::SIZE,
        packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY => BuddyAcceptRequest0104::SIZE,
        packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE => BuddyFreeChatRequest0104::SIZE,
        packet::P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE => BuddyMenuChatRequest0104::SIZE,
        packet::P_CL2FE_REQ_SET_BUDDY_BLOCK => BuddySetBlockRequest0104::SIZE,
        packet::P_CL2FE_REQ_REMOVE_BUDDY => BuddyRemoveRequest0104::SIZE,
        packet::P_CL2FE_REQ_GET_BUDDY_STATE => BuddyStateRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_GROUP_LEAVE => GroupLeaveRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_BUDDY_WARP => BuddyWarpRequest0104::SIZE,
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE => AllGroupFreeChatRequest0104::SIZE,
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE => AllGroupMenuChatRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_COMBAT_BEGIN | packet::P_CL2FE_REQ_PC_COMBAT_END => {
            PcCombatStateRequest0104::SIZE
        }
        packet::P_CL2FE_DOT_DAMAGE_ONOFF | packet::P_CL2FE_DOT_HEAL_ONOFF => {
            EnvironmentDotToggle0104::SIZE
        }
        packet::P_CL2FE_REQ_PC_LOADING_COMPLETE => PcLoadingCompleteRequest::SIZE,
        packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY => BuddyFindNameRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY => BuddyFindNameAcceptRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VEHICLE_ON => PcVehicleOnRequest0104::SIZE,
        packet::P_CL2FE_REQ_PC_VEHICLE_OFF => PcVehicleOffRequest0104::SIZE,
        packet::P_CL2FE_REQ_PRESENT_NPC_TYPES => PresentNpcTypesRequest0104::SIZE,
        packet::P_LS2CL_REP_LOGIN_SUCC => LoginSuccess::SIZE,
        packet::P_LS2CL_REP_LOGIN_FAIL => LoginFailure::SIZE,
        packet::P_LS2CL_REP_CHAR_INFO => CharacterInfo0104::SIZE,
        packet::P_LS2CL_REP_CHECK_CHAR_NAME_SUCC => CharacterNameCheckSuccess0104::SIZE,
        packet::P_LS2CL_REP_CHECK_CHAR_NAME_FAIL => CharacterNameCheckFailure0104::SIZE,
        packet::P_LS2CL_REP_SAVE_CHAR_NAME_SUCC => CharacterNameSaveSuccess0104::SIZE,
        packet::P_LS2CL_REP_SAVE_CHAR_NAME_FAIL => CharacterNameSaveFailure0104::SIZE,
        packet::P_LS2CL_REP_CHAR_CREATE_SUCC => CharacterCreateSuccess0104::SIZE,
        packet::P_LS2CL_REP_CHAR_CREATE_FAIL => CharacterCreateFailure0104::SIZE,
        packet::P_LS2CL_REP_CHAR_DELETE_SUCC => CharacterDeleteSuccess0104::SIZE,
        packet::P_LS2CL_REP_CHAR_DELETE_FAIL => CharacterDeleteFailure0104::SIZE,
        packet::P_LS2CL_REP_SHARD_SELECT_SUCC => ShardSelectSuccess::SIZE,
        packet::P_LS2CL_REP_SHARD_SELECT_FAIL => ShardSelectFailure::SIZE,
        packet::P_LS2CL_REP_PC_EXIT_DUPLICATE => DuplicateExitReply0104::SIZE,
        packet::P_LS2CL_REP_CHANGE_CHAR_NAME_SUCC => CharacterNameChangeSuccess0104::SIZE,
        packet::P_LS2CL_REP_CHANGE_CHAR_NAME_FAIL => CharacterNameChangeFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_ENTER_FAIL => PcEnterFailure::SIZE,
        packet::P_FE2CL_REP_PC_ENTER_SUCC => PcEnterSuccess::SIZE,
        packet::P_FE2CL_PC_NEW => PcNew0104::SIZE,
        packet::P_FE2CL_REP_PC_EXIT_FAIL => PcExitFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_EXIT_SUCC => PcExitSuccess0104::SIZE,
        packet::P_FE2CL_PC_EXIT => PcExit0104::SIZE,
        packet::P_FE2CL_REP_PC_REGEN_SUCC => PcRegenSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_TASK_START_SUCC => PcTaskStartSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_TASK_START_FAIL | packet::P_FE2CL_REP_PC_TASK_END_FAIL => {
            PcTaskFailure0104::SIZE
        }
        packet::P_FE2CL_REP_PC_TASK_END_SUCC => PcTaskEndSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_TASK_STOP_SUCC => PcTaskStopSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_TASK_STOP_FAIL => PcTaskStopFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_KILL_QUEST_NPCS_SUCC => PcKillQuestNpcsSuccess0104::SIZE,
        packet::P_FE2CL_PC_REGEN => PcRegen0104::SIZE,
        packet::P_FE2CL_PC_SUDDEN_DEAD => PcSuddenDead0104::SIZE,
        packet::P_FE2CL_PC_ITEM_MOVE_SUCC => ItemMoveSuccessPacket0104::SIZE,
        packet::P_FE2CL_PC_EQUIP_CHANGE => EquipChangePacket0104::SIZE,
        packet::P_FE2CL_REP_PC_ROCKET_STYLE_FIRE_SUCC => PcRocketStyleFireSuccess0104::SIZE,
        packet::P_FE2CL_PC_ROCKET_STYLE_FIRE => PcRocketStyleFire0104::SIZE,
        packet::P_FE2CL_REP_PC_GRENADE_STYLE_FIRE_SUCC => PcGrenadeStyleFireSuccess0104::SIZE,
        packet::P_FE2CL_PC_GRENADE_STYLE_FIRE => PcGrenadeStyleFire0104::SIZE,
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC => VendorItemBuySuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_FAIL
        | packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_FAIL
        | packet::P_FE2CL_REP_PC_VENDOR_START_FAIL
        | packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_FAIL
        | packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_FAIL
        | packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_FAIL => VendorFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC => VendorItemSellSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC => PcItemDeleteSuccess0104::SIZE,
        packet::P_FE2CL_REP_NANO_ACTIVE_SUCC => NanoActiveSuccess0104::SIZE,
        packet::P_FE2CL_REP_NANO_TUNE_SUCC => NanoTuneSuccess0104::SIZE,
        packet::P_FE2CL_REP_NANO_TUNE_FAIL => NanoTuneFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC => PcNanoCreateSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL => PcNanoCreateFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_VENDOR_START_SUCC => VendorStartSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC => VendorTableUpdateSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC => {
            VendorItemRestoreBuySuccess0104::SIZE
        }
        packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC => VendorBatteryBuySuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC => PcDisassembleItemSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL => PcDisassembleItemFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC => PcBankOpenSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_BANK_OPEN_FAIL | packet::P_FE2CL_REP_PC_BANK_CLOSE_FAIL => {
            PcBankFailure0104::SIZE
        }
        packet::P_FE2CL_REP_PC_BANK_CLOSE_SUCC => PcBankCloseSuccess0104::SIZE,
        packet::P_FE2CL_PC_MOVE => PcMove0104::SIZE,
        packet::P_FE2CL_PC_STOP => PcStop0104::SIZE,
        packet::P_FE2CL_PC_JUMP => PcJump0104::SIZE,
        packet::P_FE2CL_REP_PC_FREECHAT_SUCC => FreeChatSuccess0104::SIZE,
        packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC => MenuChatSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT => AvatarEmoteChat0104::SIZE,
        packet::P_FE2CL_PC_MOTD_LOGIN => ServerMessage0104::SIZE,
        packet::P_FE2CL_GM_REP_PC_SET_VALUE => GmSetValueReply0104::SIZE,
        packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT => CharTimeBuffTimeout0104::SIZE,
        packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_FAIL => BuddyListFailure0104::SIZE,
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC => BuddyMakeSuccess0104::SIZE,
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_FAIL => BuddyMakeFailure0104::SIZE,
        packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC => BuddyAcceptSuccess0104::SIZE,
        packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_FAIL => BuddyAcceptFailure0104::SIZE,
        packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC => BuddyFreeChatSuccess0104::SIZE,
        packet::P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC => BuddyMenuChatSuccess0104::SIZE,
        packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC => BuddyStateSuccess0104::SIZE,
        packet::P_FE2CL_REP_GET_BUDDY_STATE_FAIL => BuddyStateFailure0104::SIZE,
        packet::P_FE2CL_REP_SET_BUDDY_BLOCK_SUCC => BuddyBlockSuccess0104::SIZE,
        packet::P_FE2CL_REP_SET_BUDDY_BLOCK_FAIL => BuddyBlockFailure0104::SIZE,
        packet::P_FE2CL_REP_REMOVE_BUDDY_SUCC => BuddyRemoveSuccess0104::SIZE,
        packet::P_FE2CL_REP_REMOVE_BUDDY_FAIL => BuddyRemoveFailure0104::SIZE,
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC_TO_ACCEPTER => BuddyIncomingRequest0104::SIZE,

        packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC => {
            AllGroupFreeChatSuccess0104::SIZE
        }
        packet::P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC => {
            AllGroupMenuChatSuccess0104::SIZE
        }
        packet::P_FE2CL_REP_PC_WARP_USE_NPC_SUCC => PcWarpUseNpcSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_WARP_USE_NPC_FAIL => PcWarpUseNpcFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_GOTO_SUCC => PcGotoSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_CHANGE_MENTOR_SUCC => PcChangeMentorSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_CHANGE_MENTOR_FAIL => PcChangeMentorFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_SET_CURRENT_MISSION_ID => PcSetCurrentMissionId0104::SIZE,
        packet::P_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC
        | packet::P_FE2CL_PC_SPECIAL_STATE_CHANGE => SpecialStateChange0104::SIZE,
        packet::P_FE2CL_REP_PC_BUDDY_WARP_FAIL => BuddyWarpFailure0104::SIZE,
        packet::P_FE2CL_REP_ITEM_CHEST_OPEN_SUCC => ItemChestOpenSuccess0104::SIZE,
        packet::P_FE2CL_REP_ITEM_CHEST_OPEN_FAIL => ItemChestOpenFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_ITEM_USE_FAIL => ItemUseFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC => BuddyWarpOtherShardSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC => BuddyWarpSameShardSuccess0104::SIZE,
        packet::P_FE2CL_PC_BUFF_UPDATE => PcBuffUpdate0104::SIZE,
        packet::P_FE2CL_PC_CASH_BUFF_UPDATE => PcCashBuffUpdate0104::SIZE,
        packet::P_FE2CL_PC_VEHICLE_ON_SUCC | packet::P_FE2CL_PC_VEHICLE_OFF_SUCC => 1,
        packet::P_FE2CL_PC_VEHICLE_ON_FAIL | packet::P_FE2CL_PC_VEHICLE_OFF_FAIL => 4,
        packet::P_FE2CL_PC_QUICK_SLOT_INFO => QuickSlotInfo0104::SIZE,
        packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL => QuickSlotRegisterFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC => QuickSlotRegisterSuccess0104::SIZE,
        packet::P_FE2CL_NPC_ENTER => NpcEnter0104::SIZE,
        packet::P_FE2CL_NPC_EXIT => NpcExit0104::SIZE,
        packet::P_FE2CL_NPC_MOVE => NpcMove0104::SIZE,
        packet::P_FE2CL_NPC_NEW => NpcNew0104::SIZE,
        packet::P_FE2CL_NPC_SKILL_READY
        | packet::P_FE2CL_NPC_SKILL_FIRE
        | packet::P_FE2CL_NPC_SKILL_CORRUPTION_READY => 20,
        packet::P_FE2CL_NPC_SKILL_CANCEL => 4,
        packet::P_FE2CL_REP_BARKER => NpcBarker0104::SIZE,
        packet::P_FE2CL_TRANSPORTATION_ENTER | packet::P_FE2CL_TRANSPORTATION_NEW => {
            TransportationAppearance0104::SIZE
        }
        packet::P_FE2CL_TRANSPORTATION_EXIT => TransportationExit0104::SIZE,
        packet::P_FE2CL_TRANSPORTATION_MOVE => TransportationMove0104::SIZE,
        packet::P_FE2CL_SHINY_ENTER | packet::P_FE2CL_SHINY_NEW => ShinyAppearance0104::SIZE,
        packet::P_FE2CL_SHINY_EXIT => ShinyExit0104::SIZE,
        packet::P_FE2CL_REP_PC_TICK => PcTick0104::SIZE,
        packet::P_FE2CL_REP_PC_LOADING_COMPLETE_SUCC => PcLoadingCompleteSuccess::SIZE,
        packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC => BuddyFindNameSuccess0104::SIZE,
        packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL => BuddyFindNameFailure0104::SIZE,
        packet::P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL => BuddyFindNameAcceptFailure0104::SIZE,
        packet::P_FE2CL_REP_NANO_BOOK_SUBSET => 76,
        _ => {
            // Client-to-server packets without a hand-written codec fall back
            // to the generated clean-client mirror, restricted to the shapes
            // OpenFusion's descriptor table validates as fixed-length on
            // receipt. Server-to-client packets get no fallback: the pinned
            // server sends several eST-dependent packets through its legacy
            // unvalidated path while its table still calls them fixed
            // (`P_FE2CL_NANO_SKILL_USE`, `P_FE2CL_PC_ITEM_USE`).
            return registered_request_layout_0104(packet_id)
                .and_then(|layout| layout.fixed_size())
                .or_else(|| {
                    is_client_to_server_0104(packet_id)
                        .then(|| wire_0104::fixed_frame_size(packet_id))
                        .flatten()
                });
        }
    })
}
