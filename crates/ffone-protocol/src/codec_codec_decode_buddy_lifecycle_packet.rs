use super::*;

pub const PACKET_TYPE_MASK: u32 = 0xff00_0fff;

pub const PACKET_FLAGS_MASK: u32 = 0x00ff_f000;

pub const PACKET_FLAGS_SHIFT: u32 = 12;

pub const MAX_PACKET_FLAGS: u16 = 0x0fff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    MissingLengthPrefix { available: usize },
    InvalidBodyLength { declared: usize },
    BodyTooLarge { declared: usize, maximum: usize },
    IncompleteFrame { expected: usize, available: usize },
    TrailingBytes { frame_size: usize, available: usize },
    ChecksumMismatch { expected: u16, actual: u16 },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLengthPrefix { available } => {
                write!(f, "missing 4-byte length prefix (have {available})")
            }
            Self::InvalidBodyLength { declared } => {
                write!(f, "body must include a 4-byte type (declared {declared})")
            }
            Self::BodyTooLarge { declared, maximum } => {
                write!(f, "body is {declared} bytes, maximum is {maximum}")
            }
            Self::IncompleteFrame {
                expected,
                available,
            } => {
                write!(f, "incomplete frame: need {expected}, have {available}")
            }
            Self::TrailingBytes {
                frame_size,
                available,
            } => write!(f, "frame is {frame_size} bytes, input has {available}"),
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "checksum mismatch: expected {expected:#05x}, got {actual:#05x}"
                )
            }
        }
    }
}

impl std::error::Error for FrameError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFrame {
    pub packet_type: u32,
    /// OpenFusion sends `checksum`; the official client sends `checksum XOR sequence`.
    pub flags: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

impl DecodedFrame {
    pub fn has_openfusion_checksum(&self) -> bool {
        self.flags == self.checksum
    }

    pub fn has_legacy_client_checksum(&self, sequence: u16) -> bool {
        self.flags ^ (sequence & MAX_PACKET_FLAGS) == self.checksum
    }
}

/// OpenFusion's mod-255 two-sum over bytes after the four-byte type.
pub fn packet_checksum(decrypted_body: &[u8]) -> u16 {
    let mut sum_a = 0u32;
    let mut sum_b = 0u32;
    for byte in decrypted_body.iter().skip(4) {
        sum_a = (sum_a + u32::from(*byte)) % 0xff;
        sum_b = (sum_b + sum_a) % 0xff;
    }
    (((sum_b << 8) | sum_a) & u32::from(MAX_PACKET_FLAGS)) as u16
}

pub fn encode_server_frame(
    packet_type: u32,
    payload: &[u8],
    key: u64,
) -> Result<Vec<u8>, FrameError> {
    encode_with_sequence(packet_type, payload, key, 0)
}

/// Encode the exact legacy-client flag form accepted by OpenFusion.
pub fn encode_client_frame(
    packet_type: u32,
    payload: &[u8],
    key: u64,
    sequence: u16,
) -> Result<Vec<u8>, FrameError> {
    encode_with_sequence(packet_type, payload, key, sequence & MAX_PACKET_FLAGS)
}

pub(super) fn encode_with_sequence(
    packet_type: u32,
    payload: &[u8],
    key: u64,
    sequence: u16,
) -> Result<Vec<u8>, FrameError> {
    let body_len = 4 + payload.len();
    validate_body_len(body_len)?;
    let mut body = vec![0u8; body_len];
    body[..4].copy_from_slice(&(packet_type & PACKET_TYPE_MASK).to_le_bytes());
    body[4..].copy_from_slice(payload);
    let flags = packet_checksum(&body) ^ sequence;
    let raw_type = (packet_type & PACKET_TYPE_MASK) | (u32::from(flags) << PACKET_FLAGS_SHIFT);
    body[..4].copy_from_slice(&raw_type.to_le_bytes());
    encrypt_in_place(&mut body, key);

    let mut frame = Vec::with_capacity(body_len + 4);
    frame.extend_from_slice(&(body_len as u32).to_le_bytes());
    frame.extend_from_slice(&body);
    Ok(frame)
}

pub fn decode_frame(frame: &[u8], key: u64) -> Result<DecodedFrame, FrameError> {
    let (decoded, consumed) = decode_frame_prefix(frame, key)?;
    if consumed != frame.len() {
        return Err(FrameError::TrailingBytes {
            frame_size: consumed,
            available: frame.len(),
        });
    }
    Ok(decoded)
}

pub fn decode_server_frame(frame: &[u8], key: u64) -> Result<DecodedFrame, FrameError> {
    let decoded = decode_frame(frame, key)?;
    if !decoded.has_openfusion_checksum() {
        return Err(FrameError::ChecksumMismatch {
            expected: decoded.checksum,
            actual: decoded.flags,
        });
    }
    Ok(decoded)
}

pub fn decode_client_frame(
    frame: &[u8],
    key: u64,
    sequence: u16,
) -> Result<DecodedFrame, FrameError> {
    let decoded = decode_frame(frame, key)?;
    let actual = decoded.flags ^ (sequence & MAX_PACKET_FLAGS);
    if actual != decoded.checksum {
        return Err(FrameError::ChecksumMismatch {
            expected: decoded.checksum,
            actual,
        });
    }
    Ok(decoded)
}

pub(super) fn decode_frame_prefix(frame: &[u8], key: u64) -> Result<(DecodedFrame, usize), FrameError> {
    if frame.len() < 4 {
        return Err(FrameError::MissingLengthPrefix {
            available: frame.len(),
        });
    }
    let body_len = u32::from_le_bytes(frame[..4].try_into().expect("four bytes")) as usize;
    validate_body_len(body_len)?;
    let frame_len = 4 + body_len;
    if frame.len() < frame_len {
        return Err(FrameError::IncompleteFrame {
            expected: frame_len,
            available: frame.len(),
        });
    }
    let mut body = frame[4..frame_len].to_vec();
    decrypt_in_place(&mut body, key);
    let raw_type = u32::from_le_bytes(body[..4].try_into().expect("validated body"));
    Ok((
        DecodedFrame {
            packet_type: raw_type & PACKET_TYPE_MASK,
            flags: ((raw_type & PACKET_FLAGS_MASK) >> PACKET_FLAGS_SHIFT) as u16,
            checksum: packet_checksum(&body),
            payload: body[4..].to_vec(),
        },
        frame_len,
    ))
}

#[derive(Debug, Clone, Default)]
pub struct FrameBuffer {
    pub(super) bytes: Vec<u8>,
}

impl FrameBuffer {
    pub fn push(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn buffered_len(&self) -> usize {
        self.bytes.len()
    }

    pub fn next(&mut self, key: u64) -> Result<Option<DecodedFrame>, FrameError> {
        if self.bytes.len() < 4 {
            return Ok(None);
        }
        let body_len = u32::from_le_bytes(self.bytes[..4].try_into().expect("four bytes")) as usize;
        validate_body_len(body_len)?;
        let frame_len = 4 + body_len;
        if self.bytes.len() < frame_len {
            return Ok(None);
        }
        let decoded = decode_frame(&self.bytes[..frame_len], key)?;
        self.bytes.drain(..frame_len);
        Ok(Some(decoded))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadError {
    WrongSize {
        expected: usize,
        actual: usize,
    },
    Utf16TooLong {
        capacity: usize,
        actual: usize,
    },
    ValueOutOfRange {
        field: &'static str,
        value: i32,
        minimum: i32,
        maximum: i32,
    },
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongSize { expected, actual } => {
                write!(f, "payload must be {expected} bytes, got {actual}")
            }
            Self::Utf16TooLong { capacity, actual } => write!(
                f,
                "UTF-16 value needs {actual} units plus a terminator; capacity is {capacity}"
            ),
            Self::ValueOutOfRange {
                field,
                value,
                minimum,
                maximum,
            } => write!(f, "{field} must be in {minimum}..={maximum}, got {value}"),
        }
    }
}

impl std::error::Error for PayloadError {}

/// Fixed-size packet body with an exact protocol-0104 wire layout.
pub trait WirePayload: Sized {
    const SIZE: usize;
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Result<Self, PayloadError>;
}

/// Strict item-use variable-packet decode failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemUseDecodeError0104 {
    Fixed(PayloadError),
    NegativeTargetCount {
        target_count: i32,
    },
    UnsupportedPositiveTargetSkillType {
        skill_type: i32,
        target_count: i32,
    },
    PayloadSizeOverflow {
        prefix_size: usize,
        record_size: usize,
        target_count: i32,
    },
}

impl From<PayloadError> for ItemUseDecodeError0104 {
    fn from(error: PayloadError) -> Self {
        Self::Fixed(error)
    }
}

impl fmt::Display for ItemUseDecodeError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(error) => error.fmt(f),
            Self::NegativeTargetCount { target_count } => {
                write!(
                    f,
                    "item-use target count cannot be negative: {target_count}"
                )
            }
            Self::UnsupportedPositiveTargetSkillType {
                skill_type,
                target_count,
            } => write!(
                f,
                "item-use skill type {skill_type} with {target_count} targets has no proven clean-client result-tail ABI"
            ),
            Self::PayloadSizeOverflow {
                prefix_size,
                record_size,
                target_count,
            } => write!(
                f,
                "item-use payload size overflows for prefix {prefix_size}, record size {record_size}, and target count {target_count}"
            ),
        }
    }
}

impl std::error::Error for ItemUseDecodeError0104 {}

/// Decode the exact protocol-0104 item-use response/broadcast family.
///
/// Unknown packet IDs return `Ok(None)`. Known but unsupported positive-target
/// skill types fail closed so callers can retain their original raw frame
/// without applying an inventory mutation.
pub fn decode_item_use_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<ItemUsePacket0104>, ItemUseDecodeError0104> {
    let packet = match packet_type {
        packet::P_FE2CL_REP_PC_ITEM_USE_FAIL => {
            ItemUsePacket0104::Failure(ItemUseFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_ITEM_USE_SUCC => {
            ItemUsePacket0104::Success(ItemUseSuccessPacket0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_ITEM_USE => {
            ItemUsePacket0104::Broadcast(ItemUseBroadcastPacket0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub(super) fn decode_item_use_skill_results_0104(
    skill_type: i32,
    target_count: i32,
    prefix_size: usize,
    payload: &[u8],
) -> Result<ItemUseSkillResults0104, ItemUseDecodeError0104> {
    if target_count < 0 {
        return Err(ItemUseDecodeError0104::NegativeTargetCount { target_count });
    }
    if target_count == 0 {
        require_size(payload, prefix_size)?;
        return Ok(ItemUseSkillResults0104::None);
    }
    let count = usize::try_from(target_count).expect("positive i32 fits usize");
    macro_rules! records {
        ($record:ty, $variant:ident) => {{
            let records =
                decode_item_use_records_0104::<$record>(prefix_size, target_count, payload)?;
            ItemUseSkillResults0104::$variant(records)
        }};
    }
    let results = match skill_type {
        1 => records!(SkillResultDamage0104, Damage),
        2 | 34 => records!(SkillResultHealHp0104, HealHp),
        3 | 4 | 5 | 8 => records!(SkillResultDamageDebuff0104, DamageDebuff),
        6 => records!(SkillResultHealStamina0104, HealStamina),
        10 | 11 | 12 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 23 | 25 | 31 | 32 | 33 | 35 => {
            records!(SkillResultBuff0104, Buff)
        }
        21 => records!(SkillResultBatteryDrain0104, BatteryDrain),
        26 => records!(SkillResultResurrect0104, Resurrect),
        27 | 28 => records!(SkillResultMove0104, Move),
        30 => {
            let damage_bytes = count.checked_mul(SkillResultDamage0104::SIZE).ok_or(
                ItemUseDecodeError0104::PayloadSizeOverflow {
                    prefix_size,
                    record_size: SkillResultDamage0104::SIZE,
                    target_count,
                },
            )?;
            let expected = prefix_size
                .checked_add(SkillResultHealHp0104::SIZE)
                .and_then(|size| size.checked_add(damage_bytes))
                .ok_or(ItemUseDecodeError0104::PayloadSizeOverflow {
                    prefix_size,
                    record_size: SkillResultDamage0104::SIZE,
                    target_count,
                })?;
            require_size(payload, expected)?;
            let heal_end = prefix_size + SkillResultHealHp0104::SIZE;
            let self_heal = SkillResultHealHp0104::decode_exact(&payload[prefix_size..heal_end]);
            let targets = payload[heal_end..]
                .chunks_exact(SkillResultDamage0104::SIZE)
                .map(SkillResultDamage0104::decode_exact)
                .collect();
            ItemUseSkillResults0104::Bloodsucking { self_heal, targets }
        }
        _ => {
            return Err(ItemUseDecodeError0104::UnsupportedPositiveTargetSkillType {
                skill_type,
                target_count,
            });
        }
    };
    debug_assert_eq!(results.target_count(), count);
    Ok(results)
}

pub(super) fn decode_item_use_records_0104<R: ItemUseSkillResultRecord0104>(
    prefix_size: usize,
    target_count: i32,
    payload: &[u8],
) -> Result<Vec<R>, ItemUseDecodeError0104> {
    let count = usize::try_from(target_count).expect("caller requires positive count");
    let tail_size =
        count
            .checked_mul(R::SIZE)
            .ok_or(ItemUseDecodeError0104::PayloadSizeOverflow {
                prefix_size,
                record_size: R::SIZE,
                target_count,
            })?;
    let expected =
        prefix_size
            .checked_add(tail_size)
            .ok_or(ItemUseDecodeError0104::PayloadSizeOverflow {
                prefix_size,
                record_size: R::SIZE,
                target_count,
            })?;
    require_size(payload, expected)?;
    Ok(payload[prefix_size..]
        .chunks_exact(R::SIZE)
        .map(R::decode_exact)
        .collect())
}

/// Decode only proven fixed-layout QuickSlot/item-use packets.
///
/// Unknown packets and the two variable item-use success/broadcast IDs return
/// `Ok(None)`, allowing the dedicated item-use classifier to own those frames.
pub fn decode_quick_slot_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<QuickSlotPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT => {
            QuickSlotPacket0104::RegisterRequest(QuickSlotRegisterRequest0104::decode(payload)?)
        }
        packet::P_CL2FE_REQ_ITEM_USE => {
            QuickSlotPacket0104::ItemUseRequest(ItemUseRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_QUICK_SLOT_INFO => {
            QuickSlotPacket0104::Info(QuickSlotInfo0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL => {
            QuickSlotPacket0104::RegisterFailure(QuickSlotRegisterFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC => {
            QuickSlotPacket0104::RegisterSuccess(QuickSlotRegisterSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_ITEM_USE_FAIL => {
            QuickSlotPacket0104::ItemUseFailure(ItemUseFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_ITEM_USE_SUCC | packet::P_FE2CL_PC_ITEM_USE => return Ok(None),
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub fn decode_skill_buff_packet_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<Option<SkillBuffPacket0104>, PayloadError> {
    let packet = match packet_id {
        packet::P_FE2CL_PC_BUFF_UPDATE => {
            SkillBuffPacket0104::Pc(PcBuffUpdate0104::decode(payload)?)
        }
        packet::P_FE2CL_PC_CASH_BUFF_UPDATE => {
            SkillBuffPacket0104::Cash(PcCashBuffUpdate0104::decode(payload)?)
        }
        packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT => {
            SkillBuffPacket0104::Timeout(CharTimeBuffTimeout0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

/// Decode only normal FreeChat packets. Unknown packet types are deliberately
/// left untouched for other protocol slices through `Ok(None)`.
pub fn decode_freechat_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<FreeChatPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_PC_FREECHAT => {
            FreeChatPacket0104::Request(FreeChatRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_FREECHAT_SUCC => {
            FreeChatPacket0104::Success(FreeChatSuccess0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub fn decode_menu_chat_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<MenuChatPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE => {
            MenuChatPacket0104::Request(MenuChatRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC => {
            MenuChatPacket0104::Success(MenuChatSuccess0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub fn decode_avatar_emote_chat_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<AvatarEmoteChat0104>, PayloadError> {
    match packet_type {
        packet::P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT | packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT => {
            AvatarEmoteChat0104::decode(payload).map(Some)
        }
        _ => Ok(None),
    }
}

/// Decode only OpenFusion/legacy MOTD system-chat frames. Unrelated packet
/// IDs remain available to the next lossless gameplay owner.
pub fn decode_server_message_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<ServerMessage0104>, PayloadError> {
    match packet_type {
        packet::P_FE2CL_PC_MOTD_LOGIN => ServerMessage0104::decode(payload).map(Some),
        _ => Ok(None),
    }
}

/// Decode only the authoritative clean GM set-value reply. Unrelated packet
/// IDs remain available to their feature owner.
pub fn decode_gm_set_value_reply_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<GmSetValueReply0104>, PayloadError> {
    match packet_type {
        packet::P_FE2CL_GM_REP_PC_SET_VALUE => GmSetValueReply0104::decode(payload).map(Some),
        _ => Ok(None),
    }
}

/// Decode the proven Buddy lifecycle core, including the bounded variable
/// buddy-list packet. Unknown packets and the already-supported Buddy FreeChat
/// family return `Ok(None)`.
pub fn decode_buddy_lifecycle_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<BuddyLifecyclePacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_REQUEST_MAKE_BUDDY => {
            BuddyLifecyclePacket0104::MakeRequest(BuddyMakeRequest0104::decode(payload)?)
        }
        packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY => {
            BuddyLifecyclePacket0104::FindNameRequest(BuddyFindNameRequest0104::decode(payload)?)
        }
        packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY => {
            BuddyLifecyclePacket0104::FindNameAcceptRequest(BuddyFindNameAcceptRequest0104::decode(
                payload,
            )?)
        }
        packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY => {
            BuddyLifecyclePacket0104::AcceptRequest(BuddyAcceptRequest0104::decode(payload)?)
        }
        packet::P_CL2FE_REQ_GET_BUDDY_STATE => {
            BuddyLifecyclePacket0104::StateRequest(BuddyStateRequest0104::decode(payload)?)
        }
        packet::P_CL2FE_REQ_SET_BUDDY_BLOCK => {
            BuddyLifecyclePacket0104::SetBlockRequest(BuddySetBlockRequest0104::decode(payload)?)
        }
        packet::P_CL2FE_REQ_REMOVE_BUDDY => {
            BuddyLifecyclePacket0104::RemoveRequest(BuddyRemoveRequest0104::decode(payload)?)
        }
        packet::P_CL2FE_REQ_PC_BUDDY_WARP => {
            BuddyLifecyclePacket0104::WarpRequest(BuddyWarpRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC => {
            BuddyLifecyclePacket0104::ListInfo(BuddyListInfo0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_FAIL => {
            BuddyLifecyclePacket0104::ListFailure(BuddyListFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC => {
            BuddyLifecyclePacket0104::MakeSuccess(BuddyMakeSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_FAIL => {
            BuddyLifecyclePacket0104::MakeFailure(BuddyMakeFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC => {
            BuddyLifecyclePacket0104::FindNameSuccess(BuddyFindNameSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL => {
            BuddyLifecyclePacket0104::FindNameFailure(BuddyFindNameFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL => {
            BuddyLifecyclePacket0104::FindNameAcceptFailure(BuddyFindNameAcceptFailure0104::decode(
                payload,
            )?)
        }
        packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC => {
            BuddyLifecyclePacket0104::AcceptSuccess(BuddyAcceptSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_FAIL => {
            BuddyLifecyclePacket0104::AcceptFailure(BuddyAcceptFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC => {
            BuddyLifecyclePacket0104::StateSuccess(BuddyStateSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_GET_BUDDY_STATE_FAIL => {
            BuddyLifecyclePacket0104::StateFailure(BuddyStateFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_SET_BUDDY_BLOCK_SUCC => {
            BuddyLifecyclePacket0104::BlockSuccess(BuddyBlockSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_SET_BUDDY_BLOCK_FAIL => {
            BuddyLifecyclePacket0104::BlockFailure(BuddyBlockFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_REMOVE_BUDDY_SUCC => {
            BuddyLifecyclePacket0104::RemoveSuccess(BuddyRemoveSuccess0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_REMOVE_BUDDY_FAIL => {
            BuddyLifecyclePacket0104::RemoveFailure(BuddyRemoveFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC_TO_ACCEPTER => {
            BuddyLifecyclePacket0104::IncomingRequest(BuddyIncomingRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_BUDDY_WARP_FAIL => {
            BuddyLifecyclePacket0104::WarpFailure(BuddyWarpFailure0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC => {
            BuddyLifecyclePacket0104::WarpOtherShardSuccess(BuddyWarpOtherShardSuccess0104::decode(
                payload,
            )?)
        }
        packet::P_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC => {
            BuddyLifecyclePacket0104::WarpSameShardSuccess(BuddyWarpSameShardSuccess0104::decode(
                payload,
            )?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

/// Decode only buddy FreeChat packets. Unknown packet types remain
/// losslessly available to other protocol slices through `Ok(None)`.
pub fn decode_buddy_freechat_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<BuddyFreeChatPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE => {
            BuddyFreeChatPacket0104::Request(BuddyFreeChatRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC => {
            BuddyFreeChatPacket0104::Success(BuddyFreeChatSuccess0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub fn decode_buddy_menu_chat_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<BuddyMenuChatPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE => {
            BuddyMenuChatPacket0104::Request(BuddyMenuChatRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC => {
            BuddyMenuChatPacket0104::Success(BuddyMenuChatSuccess0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

/// Decode only ALL GROUP FreeChat packets. Unknown packet types remain
/// losslessly available to other protocol slices through `Ok(None)`.
pub fn decode_all_group_freechat_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<AllGroupFreeChatPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE => {
            AllGroupFreeChatPacket0104::Request(AllGroupFreeChatRequest0104::decode(payload)?)
        }

        packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC => {
            AllGroupFreeChatPacket0104::Success(AllGroupFreeChatSuccess0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub fn decode_all_group_menu_chat_packet_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<AllGroupMenuChatPacket0104>, PayloadError> {
    let packet = match packet_type {
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE => {
            AllGroupMenuChatPacket0104::Request(AllGroupMenuChatRequest0104::decode(payload)?)
        }
        packet::P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC => {
            AllGroupMenuChatPacket0104::Success(AllGroupMenuChatSuccess0104::decode(payload)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

/// Decode the two server-to-client special-state notifications consumed by
/// `GameFrame`. Both carry the same exact wire struct.
pub fn decode_special_state_change_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<SpecialStateChange0104>, PayloadError> {
    match packet_type {
        packet::P_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC
        | packet::P_FE2CL_PC_SPECIAL_STATE_CHANGE => {
            SpecialStateChange0104::decode(payload).map(Some)
        }
        _ => Ok(None),
    }
}
