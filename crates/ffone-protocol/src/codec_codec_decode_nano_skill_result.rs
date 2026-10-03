use super::*;

/// Strictly decode the local Nano-activation acknowledgement. Unknown packet
/// IDs remain available to the caller's normal passthrough route.
pub fn decode_nano_active_success_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<NanoActiveSuccess0104>, PayloadError> {
    if packet_type != packet::P_FE2CL_REP_NANO_ACTIVE_SUCC {
        return Ok(None);
    }
    NanoActiveSuccess0104::decode(payload).map(Some)
}

/// Strict Nano-skill success variable-packet decode failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NanoSkillUseDecodeError0104 {
    Fixed(PayloadError),
    EmptyTargetResults,
    NegativeTargetCount {
        target_count: i32,
    },
    TargetCountTooLarge {
        target_count: i32,
        maximum: usize,
    },
    UnsupportedSkillType {
        skill_type: i32,
    },
    PayloadSizeOverflow {
        record_size: usize,
        target_count: i32,
    },
}

impl From<PayloadError> for NanoSkillUseDecodeError0104 {
    fn from(error: PayloadError) -> Self {
        Self::Fixed(error)
    }
}

impl fmt::Display for NanoSkillUseDecodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(error) => error.fmt(formatter),
            Self::EmptyTargetResults => formatter.write_str(
                "Nano-skill success/use packets require at least one OpenFusion result record",
            ),
            Self::NegativeTargetCount { target_count } => write!(
                formatter,
                "Nano-skill result target count cannot be negative: {target_count}"
            ),
            Self::TargetCountTooLarge {
                target_count,
                maximum,
            } => write!(
                formatter,
                "Nano-skill result target count {target_count} exceeds OpenFusion maximum {maximum}"
            ),
            Self::UnsupportedSkillType { skill_type } => write!(
                formatter,
                "Nano-skill type {skill_type} has no OpenFusion success-tail ABI"
            ),
            Self::PayloadSizeOverflow {
                record_size,
                target_count,
            } => write!(
                formatter,
                "Nano-skill result payload size overflows for record size {record_size} and target count {target_count}"
            ),
        }
    }
}

impl std::error::Error for NanoSkillUseDecodeError0104 {}

pub(super) fn decode_nano_skill_target_0104(bytes: &[u8]) -> NanoSkillTarget0104 {
    debug_assert!(bytes.len() >= 8);
    NanoSkillTarget0104 {
        entity_type: read_i32(bytes, 0),
        id: read_i32(bytes, 4),
    }
}

pub(super) fn decode_nano_skill_damage_0104(bytes: &[u8]) -> NanoSkillDamageResult0104 {
    debug_assert_eq!(bytes.len(), 20);
    NanoSkillDamageResult0104 {
        target: decode_nano_skill_target_0104(bytes),
        protected: read_i32(bytes, 8),
        damage: read_i32(bytes, 12),
        hp: read_i32(bytes, 16),
    }
}

pub(super) fn decode_nano_skill_heal_hp_0104(bytes: &[u8]) -> NanoSkillHealHpResult0104 {
    debug_assert_eq!(bytes.len(), 16);
    NanoSkillHealHpResult0104 {
        target: decode_nano_skill_target_0104(bytes),
        healed_hp: read_i32(bytes, 8),
        hp: read_i32(bytes, 12),
    }
}

pub(super) fn decode_nano_skill_result_0104(
    skill_type: i32,
    bytes: &[u8],
) -> Result<NanoSkillResult0104, NanoSkillUseDecodeError0104> {
    let result = match skill_type {
        1 | 22 | 38 | 39 => NanoSkillResult0104::Damage(decode_nano_skill_damage_0104(bytes)),
        2 | 34 => NanoSkillResult0104::HealHp(decode_nano_skill_heal_hp_0104(bytes)),
        3 | 4 | 5 | 8 => {
            debug_assert_eq!(bytes.len(), 32);
            NanoSkillResult0104::DamageDebuff(NanoSkillDamageDebuffResult0104 {
                target: decode_nano_skill_target_0104(bytes),
                protected: read_i32(bytes, 8),
                damage: read_i32(bytes, 12),
                hp: read_i32(bytes, 16),
                nano_stamina: read_i16(bytes, 20),
                pack_padding: bytes[22..24]
                    .try_into()
                    .expect("fixed two-byte Nano result padding"),
                nano_deactivated: read_i32(bytes, 24),
                condition_bit_flag: read_i32(bytes, 28),
            })
        }
        7 | 10 | 11 | 12 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 25 | 31 | 32 | 33 | 35 => {
            debug_assert_eq!(bytes.len(), 16);
            NanoSkillResult0104::Buff(NanoSkillBuffResult0104 {
                target: decode_nano_skill_target_0104(bytes),
                protected: read_i32(bytes, 8),
                condition_bit_flag: read_i32(bytes, 12),
            })
        }
        21 => {
            debug_assert_eq!(bytes.len(), 40);
            NanoSkillResult0104::BatteryDrain(NanoSkillBatteryDrainResult0104 {
                target: decode_nano_skill_target_0104(bytes),
                protected: read_i32(bytes, 8),
                drained_weapon_battery: read_i32(bytes, 12),
                weapon_battery: read_i32(bytes, 16),
                drained_nano_battery: read_i32(bytes, 20),
                nano_battery: read_i32(bytes, 24),
                nano_stamina: read_i16(bytes, 28),
                pack_padding: bytes[30..32]
                    .try_into()
                    .expect("fixed two-byte Nano result padding"),
                nano_deactivated: read_i32(bytes, 32),
                condition_bit_flag: read_i32(bytes, 36),
            })
        }
        26 => {
            debug_assert_eq!(bytes.len(), 12);
            NanoSkillResult0104::Resurrect(NanoSkillResurrectResult0104 {
                target: decode_nano_skill_target_0104(bytes),
                hp: read_i32(bytes, 8),
            })
        }
        27 | 28 => {
            debug_assert_eq!(bytes.len(), 24);
            NanoSkillResult0104::Move(NanoSkillMoveResult0104 {
                target: decode_nano_skill_target_0104(bytes),
                map_number: read_i32(bytes, 8),
                position: [
                    read_i32(bytes, 12),
                    read_i32(bytes, 16),
                    read_i32(bytes, 20),
                ],
            })
        }
        30 => {
            debug_assert_eq!(bytes.len(), 36);
            NanoSkillResult0104::Leech(NanoSkillLeechResult0104 {
                heal: decode_nano_skill_heal_hp_0104(&bytes[..16]),
                damage: decode_nano_skill_damage_0104(&bytes[16..]),
            })
        }
        _ => {
            return Err(NanoSkillUseDecodeError0104::UnsupportedSkillType { skill_type });
        }
    };
    Ok(result)
}

/// Strictly decode either authoritative Nano-skill result packet. Unknown
/// packet IDs remain available to the caller's normal passthrough route.
pub fn decode_nano_skill_use_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<NanoSkillUsePacket0104>, NanoSkillUseDecodeError0104> {
    let delivery = match packet_type {
        packet::P_FE2CL_NANO_SKILL_USE_SUCC => NanoSkillUseDelivery0104::LocalSuccess,
        packet::P_FE2CL_NANO_SKILL_USE => NanoSkillUseDelivery0104::RemoteUse,
        _ => return Ok(None),
    };
    Ok(Some(NanoSkillUsePacket0104 {
        delivery,
        result: NanoSkillUseSuccess0104::decode(payload)?,
    }))
}

/// Backwards-compatible local-only decoder. Remote `P_FE2CL_NANO_SKILL_USE`
/// remains passthrough here; consumers which own both routes should call
/// [`decode_nano_skill_use_packet_0104`].
pub fn decode_nano_skill_use_success_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<NanoSkillUseSuccess0104>, NanoSkillUseDecodeError0104> {
    if packet_type != packet::P_FE2CL_NANO_SKILL_USE_SUCC {
        return Ok(None);
    }
    NanoSkillUseSuccess0104::decode(payload).map(Some)
}

/// Decode only the two fixed Nano tune reply packets. Unknown packet IDs are
/// preserved by the caller as passthrough frames.
pub fn decode_nano_tune_packet_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<Option<NanoTunePacket0104>, PayloadError> {
    Ok(match packet_id {
        packet::P_FE2CL_REP_NANO_TUNE_SUCC => Some(NanoTunePacket0104::Success(
            NanoTuneSuccess0104::decode(payload)?,
        )),
        packet::P_FE2CL_REP_NANO_TUNE_FAIL => Some(NanoTunePacket0104::Failure(
            NanoTuneFailure0104::decode(payload)?,
        )),
        _ => None,
    })
}

pub fn decode_vendor_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<VendorPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC => {
            VendorPacket0104::BuySuccess(VendorItemBuySuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_FAIL => {
            VendorPacket0104::BuyFailure(VendorFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC => {
            VendorPacket0104::SellSuccess(VendorItemSellSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_FAIL => {
            VendorPacket0104::SellFailure(VendorFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC => {
            VendorPacket0104::ItemDeleteSuccess(PcItemDeleteSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_START_SUCC => {
            VendorPacket0104::StartSuccess(VendorStartSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_START_FAIL => {
            VendorPacket0104::StartFailure(VendorFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC => {
            VendorPacket0104::TableSuccess(VendorTableUpdateSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_FAIL => {
            VendorPacket0104::TableFailure(VendorFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC => {
            VendorPacket0104::RestoreSuccess(VendorItemRestoreBuySuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_FAIL => {
            VendorPacket0104::RestoreFailure(VendorFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC => {
            VendorPacket0104::BatterySuccess(VendorBatteryBuySuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_FAIL => {
            VendorPacket0104::BatteryFailure(VendorFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC => {
            VendorPacket0104::DisassembleSuccess(PcDisassembleItemSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL => {
            VendorPacket0104::DisassembleFailure(PcDisassembleItemFailure0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub fn decode_pc_bank_reply_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<PcBankReply0104>, PayloadError> {
    let reply = match packet_type {
        packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC => {
            PcBankReply0104::OpenSuccess(PcBankOpenSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_BANK_OPEN_FAIL => {
            PcBankReply0104::OpenFailure(PcBankFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_BANK_CLOSE_SUCC => {
            PcBankReply0104::CloseSuccess(PcBankCloseSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_BANK_CLOSE_FAIL => {
            PcBankReply0104::CloseFailure(PcBankFailure0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(reply))
}

/// Strictly decode only the proven fixed protocol-0104 inventory family.
///
/// Unknown packet IDs return `Ok(None)`. Known IDs reject every non-exact
/// payload length through their `WirePayload` implementation.
pub fn decode_inventory_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<InventoryPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_FE2CL_PC_ITEM_MOVE_SUCC => {
            InventoryPacket0104::ItemMoveSuccess(ItemMoveSuccessPacket0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_EQUIP_CHANGE => {
            InventoryPacket0104::EquipChange(EquipChangePacket0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub fn decode_pc_nano_create_packet_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<Option<PcNanoCreatePacket0104>, PayloadError> {
    Ok(match packet_id {
        packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC => Some(PcNanoCreatePacket0104::Success(
            PcNanoCreateSuccess0104::decode(payload)?,
        )),
        packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL => Some(PcNanoCreatePacket0104::Failure(
            PcNanoCreateFailure0104::decode(payload)?,
        )),
        _ => None,
    })
}

/// Decode only proven fixed-layout regeneration packets. Every recognized ID
/// requires its exact payload size; unrelated packets remain `Ok(None)`.
pub fn decode_pc_regen_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<PcRegenPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_PC_REGEN => {
            PcRegenPacket0104::Request(PcRegenRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_REGEN_SUCC => {
            PcRegenPacket0104::Success(PcRegenSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_REGEN => PcRegenPacket0104::Broadcast(PcRegen0104::decode(payload)?),
        packet::P_FE2CL_PC_SUDDEN_DEAD => {
            PcRegenPacket0104::SuddenDead(PcSuddenDead0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

/// Decode only the fixed PC-exit handshake. Every recognized packet requires
/// its exact protocol-0104 payload size; unknown packet types remain available
/// to other slices through `Ok(None)`.
pub fn decode_pc_exit_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<PcExitPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_PC_EXIT => {
            PcExitPacket0104::Request(PcExitRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_EXIT_FAIL => {
            PcExitPacket0104::Failure(PcExitFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_EXIT_SUCC => {
            PcExitPacket0104::Success(PcExitSuccess0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

/// Exact validation failures for OpenFusion's count-plus-trailer packets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountedPayloadError0104 {
    MissingHeader {
        expected_at_least: usize,
        actual: usize,
    },
    NegativeCount {
        count: i32,
    },
    CountTooLarge {
        count: usize,
        maximum: usize,
    },
    WrongSize {
        expected: usize,
        actual: usize,
    },
    LengthOverflow {
        count: usize,
        header_size: usize,
        element_size: usize,
    },
}

impl fmt::Display for CountedPayloadError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader {
                expected_at_least,
                actual,
            } => write!(
                formatter,
                "counted payload needs at least {expected_at_least} header bytes, got {actual}"
            ),
            Self::NegativeCount { count } => {
                write!(formatter, "counted payload has negative count {count}")
            }
            Self::CountTooLarge { count, maximum } => write!(
                formatter,
                "counted payload count {count} exceeds maximum {maximum}"
            ),
            Self::WrongSize { expected, actual } => write!(
                formatter,
                "counted payload must be {expected} bytes, got {actual}"
            ),
            Self::LengthOverflow {
                count,
                header_size,
                element_size,
            } => write!(
                formatter,
                "count {count}, header {header_size}, and element {element_size} overflow payload length"
            ),
        }
    }
}

impl std::error::Error for CountedPayloadError0104 {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentNpcTypesDecodeError0104 {
    Fixed(PayloadError),
    Counted(CountedPayloadError0104),
}

impl fmt::Display for PresentNpcTypesDecodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(error) => {
                write!(formatter, "present-NPC-types request payload: {error}")
            }
            Self::Counted(error) => {
                write!(formatter, "present-NPC-types reply payload: {error}")
            }
        }
    }
}

impl std::error::Error for PresentNpcTypesDecodeError0104 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Fixed(error) => Some(error),
            Self::Counted(error) => Some(error),
        }
    }
}

impl From<PayloadError> for PresentNpcTypesDecodeError0104 {
    fn from(error: PayloadError) -> Self {
        Self::Fixed(error)
    }
}

impl From<CountedPayloadError0104> for PresentNpcTypesDecodeError0104 {
    fn from(error: CountedPayloadError0104) -> Self {
        Self::Counted(error)
    }
}

/// Decode only the exact request/reply pair used by Retrobution WorldMapMode.
/// Unrelated packet IDs remain lossless passthrough candidates for callers.
pub fn decode_present_npc_types_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<PresentNpcTypesPacket0104>, PresentNpcTypesDecodeError0104> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_PRESENT_NPC_TYPES => {
            PresentNpcTypesPacket0104::Request(PresentNpcTypesRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PRESENT_NPC_TYPES => {
            PresentNpcTypesPacket0104::Reply(PresentNpcTypesReply0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

/// Decode the server-to-client 0104 group packets consumed by the HUD.
///
/// Known packets are rejected unless their counts and total payload length match exactly.
/// Unknown packet types remain available to other protocol slices through `Ok(None)`.
pub fn decode_group_packet_0104(
    frame: &DecodedFrame,
) -> Result<Option<GroupPacket0104>, CountedPayloadError0104> {
    let roster_header_size = match frame.packet_type {
        packet::P_FE2CL_PC_GROUP_JOIN
        | packet::P_FE2CL_PC_GROUP_JOIN_SUCC
        | packet::P_FE2CL_PC_GROUP_LEAVE
        | packet::P_FE2CL_PC_GROUP_MEMBER_INFO => 12,
        packet::P_FE2CL_REP_NPC_GROUP_INVITE_SUCC | packet::P_FE2CL_REP_NPC_GROUP_KICK_SUCC => 16,
        packet::P_FE2CL_PC_GROUP_LEAVE_SUCC => {
            if frame.payload.len() != 1 {
                return Err(CountedPayloadError0104::WrongSize {
                    expected: 1,
                    actual: frame.payload.len(),
                });
            }
            return Ok(Some(GroupPacket0104::LeaveSuccess));
        }
        _ => return Ok(None),
    };

    Ok(Some(GroupPacket0104::Roster(decode_group_roster_0104(
        &frame.payload,
        roster_header_size,
    )?)))
}

pub(super) fn decode_group_roster_0104(
    payload: &[u8],
    header_size: usize,
) -> Result<GroupRoster0104, CountedPayloadError0104> {
    const MAX_PC_MEMBERS: usize = 4;
    const MAX_NPC_MEMBERS: usize = 5;

    require_counted_header(payload, header_size)?;
    let (npc_id, pc_count_offset, npc_count_offset) = if header_size == 16 {
        (Some(read_i32(payload, 4)), 8, 12)
    } else {
        (None, 4, 8)
    };
    let signed_pc_count = read_i32(payload, pc_count_offset);
    if signed_pc_count < 0 {
        return Err(CountedPayloadError0104::NegativeCount {
            count: signed_pc_count,
        });
    }
    let signed_npc_count = read_i32(payload, npc_count_offset);
    if signed_npc_count < 0 {
        return Err(CountedPayloadError0104::NegativeCount {
            count: signed_npc_count,
        });
    }
    let pc_count = signed_pc_count as usize;
    let npc_count = signed_npc_count as usize;
    if pc_count > MAX_PC_MEMBERS {
        return Err(CountedPayloadError0104::CountTooLarge {
            count: pc_count,
            maximum: MAX_PC_MEMBERS,
        });
    }
    if npc_count > MAX_NPC_MEMBERS {
        return Err(CountedPayloadError0104::CountTooLarge {
            count: npc_count,
            maximum: MAX_NPC_MEMBERS,
        });
    }

    let npc_start = counted_payload_len(pc_count, header_size, GroupPcMemberInfo0104::SIZE)?;
    let expected = counted_payload_len(npc_count, npc_start, GroupNpcMemberInfo0104::SIZE)?;
    if payload.len() != expected {
        return Err(CountedPayloadError0104::WrongSize {
            expected,
            actual: payload.len(),
        });
    }

    let pc_members = (0..pc_count)
        .map(|index| {
            let start = header_size + index * GroupPcMemberInfo0104::SIZE;
            GroupPcMemberInfo0104::decode_exact(
                &payload[start..start + GroupPcMemberInfo0104::SIZE],
            )
        })
        .collect();
    let npc_members = (0..npc_count)
        .map(|index| {
            let start = npc_start + index * GroupNpcMemberInfo0104::SIZE;
            GroupNpcMemberInfo0104::decode_exact(
                &payload[start..start + GroupNpcMemberInfo0104::SIZE],
            )
        })
        .collect();

    Ok(GroupRoster0104 {
        context_id: read_i32(payload, 0),
        npc_id,
        pc_members,
        npc_members,
    })
}

pub fn decode_pc_warhead_fire_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<PcWarheadFirePacket0104>, PayloadError> {
    Ok(match packet_type {
        packet::P_FE2CL_REP_PC_ROCKET_STYLE_FIRE_SUCC => Some(
            PcWarheadFirePacket0104::LocalRocket(PcRocketStyleFireSuccess0104::decode(payload)?),
        ),
        packet::P_FE2CL_PC_ROCKET_STYLE_FIRE => Some(PcWarheadFirePacket0104::RemoteRocket(
            PcRocketStyleFire0104::decode(payload)?,
        )),
        packet::P_FE2CL_REP_PC_GRENADE_STYLE_FIRE_SUCC => Some(
            PcWarheadFirePacket0104::LocalGrenade(PcGrenadeStyleFireSuccess0104::decode(payload)?),
        ),
        packet::P_FE2CL_PC_GRENADE_STYLE_FIRE => Some(PcWarheadFirePacket0104::RemoteGrenade(
            PcGrenadeStyleFire0104::decode(payload)?,
        )),
        _ => None,
    })
}

/// Shared 24-byte rocket/grenade hit header followed by authoritative NPC results.
/// RustyFusion broadcasts GRENADE_STYLE_HIT for both weapon families.
pub fn decode_pc_warhead_hit_0104(payload: &[u8]) -> Result<PcAttackNpcs0104, CountedPayloadError0104> {
    require_counted_header(payload, 24)?;
    Ok(PcAttackNpcs0104 { pc_id: read_i32(payload, 0), results: decode_attack_results_after(payload, 24)? })
}

/// Decodes only the fixed animation identity from `sP_FE2CL_NPC_SKILL_*`.
///
/// The exact sizes and aligned offsets come from the clean 0104 ABI. `HIT`
/// and `CORRUPTION_HIT` are variadic packets, so this function validates their
/// fixed prefix without guessing the result-trailer schema.
pub fn decode_npc_skill_signal_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<NpcSkillSignal0104>, PayloadError> {
    let (kind, fixed_prefix, exact_size) = match packet_type {
        packet::P_FE2CL_NPC_SKILL_READY => (NpcSkillSignalKind0104::Ready, 20, true),
        packet::P_FE2CL_NPC_SKILL_FIRE => (NpcSkillSignalKind0104::Fire, 20, true),
        packet::P_FE2CL_NPC_SKILL_HIT => (NpcSkillSignalKind0104::Hit, 28, false),
        packet::P_FE2CL_NPC_SKILL_CORRUPTION_READY => {
            (NpcSkillSignalKind0104::CorruptionReady, 20, true)
        }
        packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT => {
            (NpcSkillSignalKind0104::CorruptionHit, 24, false)
        }
        packet::P_FE2CL_NPC_SKILL_CANCEL => (NpcSkillSignalKind0104::Cancel, 4, true),
        _ => return Ok(None),
    };
    if exact_size {
        require_size(payload, fixed_prefix)?;
    } else {
        require_prefix(payload, fixed_prefix)?;
    }
    Ok(Some(NpcSkillSignal0104 {
        npc_id: read_i32(payload, 0),
        skill_id: (kind != NpcSkillSignalKind0104::Cancel).then(|| read_i16(payload, 4)),
        kind,
    }))
}

/// Strict decoder for the two authoritative variable NPC skill-hit packets.
/// Other skill signals remain available to the animation-only decoder.
pub fn decode_npc_skill_authority_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<NpcSkillAuthorityPacket0104>, NpcSkillAuthorityDecodeError0104> {
    match packet_type {
        packet::P_FE2CL_NPC_SKILL_HIT => NpcSkillHit0104::decode(payload)
            .map(NpcSkillAuthorityPacket0104::SkillHit)
            .map(Some),
        packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT => NpcSkillCorruptionHit0104::decode(payload)
            .map(NpcSkillAuthorityPacket0104::CorruptionHit)
            .map(Some),
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcSkillAuthorityDecodeError0104 {
    Fixed(PayloadError),
    NegativeTargetCount {
        packet: &'static str,
        target_count: i32,
    },
    EmptyTargetResults {
        packet: &'static str,
    },
    TargetCountTooLarge {
        packet: &'static str,
        target_count: i32,
        maximum: usize,
    },
    UnsupportedSkillType {
        skill_type: i32,
    },
    NoResultSkillHasTargets {
        skill_type: i32,
        target_count: i32,
    },
    PayloadSizeOverflow {
        packet: &'static str,
        record_size: usize,
        target_count: i32,
    },
}

impl From<PayloadError> for NpcSkillAuthorityDecodeError0104 {
    fn from(error: PayloadError) -> Self {
        Self::Fixed(error)
    }
}
